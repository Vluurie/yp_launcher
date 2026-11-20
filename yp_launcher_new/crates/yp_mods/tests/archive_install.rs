mod support;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use support::{archive_of, core_fixture, dir_names, mod_dir, put_bytes, staging, temp};
use yp_core::testing::archives::{noise, write_encrypted_zip};
use yp_core::{ArchiveFailure, CancelToken, GamePaths, NoProgress, Progress};
use yp_mods::bulk::{find_archives, install_archive};
use yp_mods::detect::detect_drop;
use yp_mods::drop_gate::{DropGate, classify};
use yp_mods::install::install;
use yp_mods::{DropRejection, InstallFailure, InstallRequest, InstallResult};

const FORMATS: [&str; 3] = ["zip", "7z", "rar"];

fn bulk(game_dir: &Path, source: &Path, name: &str) -> InstallResult {
    install_archive(
        &staging(),
        &GamePaths::new(game_dir),
        source,
        name,
        &NoProgress,
        &CancelToken::new(),
    )
}

fn temp_leftovers(game_dir: &Path) -> Vec<String> {
    dir_names(&GamePaths::new(game_dir).scoped_temp_root())
}

#[test]
fn a_downloads_folder_of_real_archives_installs_in_bulk() {
    let drops = temp("yp_arc_bulk_");
    let game_dir = temp("yp_arc_bulk_g_");
    let downloads = drops.path().join("Downloads");
    std::fs::create_dir_all(downloads.join("335")).unwrap();
    archive_of(&downloads, "Blade.zip", &[("data/wp/wp0100.dat", &[1; 64])]);
    archive_of(
        &downloads,
        "Coat.7z",
        &[
            ("Coat/data/pl/pl000d.dat", &[2; 64]),
            ("Coat/data/pl/pl000d.dtt", &[3; 64]),
            ("Coat/readme.txt", b"wear it"),
        ],
    );
    archive_of(
        &downloads.join("335"),
        "Machine.rar",
        &[
            ("Classic/data/pl/pl000d.dat", &[4; 64]),
            ("Classic/data/pl/pl000d.dtt", &[5; 64]),
            ("Sword/wp0100.dat", &[6; 64]),
        ],
    );
    put_bytes(&downloads, "notes.txt", 4, 0);
    put_bytes(&downloads, "pack.tar.gz", 64, 0);

    let archives = find_archives(&downloads);
    let names: Vec<String> = archives
        .iter()
        .map(|path| support::rel_slash(&downloads, path))
        .collect();
    assert_eq!(names, ["335/Machine.rar", "Blade.zip", "Coat.7z"]);

    let mut installed = Vec::new();
    for archive in &archives {
        assert_eq!(classify(&staging(), archive), DropGate::Mod, "{}", archive.display());
        let name = archive.file_stem().unwrap().to_string_lossy().into_owned();
        installed.push(bulk(game_dir.path(), archive, &name).unwrap().id);
    }

    assert_eq!(installed, ["machine_classic", "blade", "coat"]);
    assert_eq!(
        dir_names(&GamePaths::new(game_dir.path()).mods_dir()),
        ["blade", "coat", "machine_classic", "machine_sword"]
    );
    let blade = mod_dir(game_dir.path(), "blade").join("data").join("wp").join("wp0100.dat");
    assert_eq!(std::fs::read(blade).unwrap(), [1; 64]);
    let coat = mod_dir(game_dir.path(), "coat").join("data").join("pl");
    assert_eq!(std::fs::read(coat.join("pl000d.dat")).unwrap(), [2; 64]);
    assert_eq!(std::fs::read(coat.join("pl000d.dtt")).unwrap(), [3; 64]);
    let sword = mod_dir(game_dir.path(), "machine_sword").join("data").join("wp").join("wp0100.dat");
    assert_eq!(std::fs::read(sword).unwrap(), [6; 64]);
    assert_eq!(temp_leftovers(game_dir.path()), Vec::<String>::new());
    assert!(archives.iter().all(|archive| archive.is_file()));
}

#[test]
fn a_single_archive_installs_the_same_from_every_format() {
    for extension in FORMATS {
        let drops = temp("yp_arc_single_");
        let game_dir = temp("yp_arc_single_g_");
        let archive = archive_of(
            drops.path(),
            &format!("Kimono Pack.{extension}"),
            &[
                ("Kimono Pack/data/pl/pl000d.dat", &[7; 128]),
                ("Kimono Pack/data/pl/pl000d.dtt", &[8; 128]),
            ],
        );
        let detected = detect_drop(&staging(), &archive, &NoProgress, &CancelToken::new()).unwrap();
        assert_eq!(detected.rejection, None, "{extension}");
        assert!(!detected.has_variants(), "{extension}");
        assert_eq!(detected.suggested_id, "kimono_pack", "{extension}");

        let installed = install(
            &staging(),
            &GamePaths::new(game_dir.path()),
            &archive,
            &InstallRequest::default(),
            &NoProgress,
            &CancelToken::new(),
        )
        .unwrap();
        assert_eq!(installed.id, "kimono_pack", "{extension}");
        let data = mod_dir(game_dir.path(), "kimono_pack").join("data").join("pl");
        assert_eq!(std::fs::read(data.join("pl000d.dtt")).unwrap(), [8; 128], "{extension}");
        assert_eq!(temp_leftovers(game_dir.path()), Vec::<String>::new(), "{extension}");
    }
}

fn locked_zip(dir: &Path) -> PathBuf {
    let archive = dir.join("Locked.zip");
    write_encrypted_zip(&archive, b"secret", &[("data/wp/wp0100.dat", &[9; 64])]);
    archive
}

#[test]
fn a_failed_extraction_says_why() {
    let drops = temp("yp_arc_why_");
    let game_dir = temp("yp_arc_why_g_");
    let locked = locked_zip(drops.path());
    let damaged = put_bytes(drops.path(), "Damaged.7z", 256, 5);
    let cases = [
        (locked, ArchiveFailure::PasswordRequired),
        (core_fixture("enc_headers.7z"), ArchiveFailure::PasswordRequired),
        (core_fixture("enc_headers.rar"), ArchiveFailure::PasswordRequired),
        (damaged, ArchiveFailure::Corrupt),
    ];
    for (archive, failure) in cases {
        let rejected = InstallFailure::Rejected(DropRejection::ArchiveExtractFailed(failure));
        let detected = detect_drop(&staging(), &archive, &NoProgress, &CancelToken::new()).unwrap();
        assert_eq!(
            detected.rejection,
            Some(DropRejection::ArchiveExtractFailed(failure)),
            "{}",
            archive.display()
        );
        assert_eq!(bulk(game_dir.path(), &archive, "Pack"), Err(rejected.clone()), "{}", archive.display());
        let direct = install(
            &staging(),
            &GamePaths::new(game_dir.path()),
            &archive,
            &InstallRequest::named("Pack"),
            &NoProgress,
            &CancelToken::new(),
        );
        assert_eq!(direct, Err(rejected), "{}", archive.display());
    }
    assert!(!GamePaths::new(game_dir.path()).mods_dir().exists());
    assert_eq!(temp_leftovers(game_dir.path()), Vec::<String>::new());
}

#[test]
fn a_tarball_is_an_unknown_drop_not_a_broken_archive() {
    let drops = temp("yp_arc_tar_");
    let game_dir = temp("yp_arc_tar_g_");
    for name in ["Pack.tar", "Pack.tar.gz", "Pack.bz2"] {
        let tarball = put_bytes(drops.path(), name, 512, 1);
        assert_eq!(classify(&staging(), &tarball), DropGate::NotAMod, "{name}");
        let detected = detect_drop(&staging(), &tarball, &NoProgress, &CancelToken::new()).unwrap();
        assert_eq!(detected.rejection, Some(DropRejection::UnknownDrop), "{name}");
        assert_eq!(
            bulk(game_dir.path(), &tarball, "Pack"),
            Err(InstallFailure::Rejected(DropRejection::UnknownDrop)),
            "{name}"
        );
    }
    assert!(find_archives(drops.path()).is_empty());
}

#[test]
fn cancelling_while_an_archive_unpacks_installs_nothing() {
    let drops = temp("yp_arc_cancel_");
    let game_dir = temp("yp_arc_cancel_g_");
    let body = noise(1024 * 1024, 3);
    let archive = archive_of(drops.path(), "Big.zip", &[("data/wp/wp0100.dat", &body)]);
    let cancel = CancelToken::new();
    let seen = Mutex::new(Vec::new());
    let stop = cancel.clone();
    let sink = |progress: Progress| {
        if progress.fraction.is_some_and(|fraction| fraction > 0.0) {
            stop.cancel();
        }
        seen.lock().unwrap().push(progress.fraction);
    };
    let result = install(
        &staging(),
        &GamePaths::new(game_dir.path()),
        &archive,
        &InstallRequest::named("Big"),
        &sink,
        &cancel,
    );
    assert_eq!(result, Err(InstallFailure::Cancelled));
    let reached = seen.into_inner().unwrap().into_iter().flatten().fold(0.0, f32::max);
    assert!(reached > 0.0 && reached < 1.0, "{reached}");
    assert!(!GamePaths::new(game_dir.path()).mods_dir().join("big").exists());
    assert_eq!(temp_leftovers(game_dir.path()), Vec::<String>::new());
}
