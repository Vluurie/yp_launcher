use std::path::{Path, PathBuf};
use std::sync::Mutex;

use yp_core::archive::Extractor;
use yp_core::testing::archives::{noise, write_archive, write_archive_of_dir, write_encrypted_zip, write_zip};
use yp_core::{ArchiveFailure, CancelToken, Error, GamePaths, HostEnv, NoProgress, Os, Progress};
use yp_cutscenes::install::{self, NameChoice, SourceKind};
use yp_cutscenes::{
    CopyProgress, InstallContext, InstallOutcome, PrepareFailure, ScanDirs, ScanProgress, Silent,
    delete_mod, detect, has_custom_in_data_movie, prepare, scan,
};
use yp_formats::testing::usm_file;

const SOFDEC: u32 = 1;
const H264: u32 = 5;

struct Game {
    root: tempfile::TempDir,
    paths: GamePaths,
    host: HostEnv,
    extractor: Extractor,
}

impl Game {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let paths = GamePaths::new(root.path().join("NieRAutomata"));
        std::fs::create_dir_all(paths.movie_dir()).unwrap();
        std::fs::create_dir_all(paths.cutscenes_dir()).unwrap();
        std::fs::create_dir_all(paths.mods_dir()).unwrap();
        let host = HostEnv::for_test(Os::Windows, &[], root.path());
        Self {
            root,
            paths,
            host,
            extractor: Extractor::new(),
        }
    }

    fn dirs(&self) -> ScanDirs {
        ScanDirs::for_game(&self.paths)
    }

    fn context(&self) -> InstallContext<'_> {
        InstallContext {
            game: &self.paths,
            host: &self.host,
            extractor: &self.extractor,
        }
    }

    fn outside(&self, name: &str) -> PathBuf {
        let downloads = self.root.path().join("downloads");
        std::fs::create_dir_all(&downloads).unwrap();
        downloads.join(name)
    }
}

fn write_usm(path: &Path, width: u32, height: u32, codec: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, usm_file(width, height, codec)).unwrap();
}

fn write_bytes(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn populate_mods(game: &Game) {
    let cutscenes = game.paths.cutscenes_dir();
    write_usm(
        &cutscenes.join("Zeta HD").join("movie").join("ev0020.usm"),
        1920,
        1080,
        SOFDEC,
    );
    write_usm(
        &cutscenes.join("Zeta HD").join("movie").join("ev0010.usm"),
        2560,
        1080,
        H264,
    );
    write_usm(
        &cutscenes.join("Zeta HD").join("movie").join("ev0030.usm"),
        1280,
        1440,
        SOFDEC,
    );
    write_bytes(
        &cutscenes.join("Zeta HD").join("movie").join("notes.txt"),
        b"x",
    );
    std::fs::create_dir_all(cutscenes.join("Alpha").join("movie")).unwrap();
    write_bytes(
        &cutscenes.join("Broken").join("movie").join("bad.usm"),
        b"not a usm",
    );
    write_usm(
        &cutscenes.join(".hidden").join("movie").join("ev0010.usm"),
        3840,
        2160,
        H264,
    );
    write_usm(
        &cutscenes.join("_disabled").join("movie").join("ev0010.usm"),
        3840,
        2160,
        H264,
    );
    write_bytes(&cutscenes.join("loose.usm"), b"x");
    let mods = game.paths.mods_dir();
    write_usm(
        &mods
            .join("mymod")
            .join("cutscenes")
            .join("Beta")
            .join("movie")
            .join("ev0040.usm"),
        1600,
        904,
        SOFDEC,
    );
    write_usm(
        &mods
            .join("mymod")
            .join("cutscenes")
            .join("_old")
            .join("movie")
            .join("ev0040.usm"),
        1,
        1,
        H264,
    );
    write_usm(
        &mods
            .join("_off")
            .join("cutscenes")
            .join("Gamma")
            .join("movie")
            .join("ev0040.usm"),
        1,
        1,
        H264,
    );
    write_bytes(&mods.join("other").join("data").join("pl0000.dat"), b"x");
}

fn populate_data_movie(game: &Game) {
    let movie = game.paths.movie_dir();
    write_usm(&movie.join("ev0010.usm"), 1600, 904, SOFDEC);
    write_usm(&movie.join("EV0020.USM"), 1920, 1080, SOFDEC);
    write_usm(&movie.join("ev0030.usm"), 2560, 1440, SOFDEC);
    write_usm(&movie.join("ev0040.usm"), 1920, 1080, H264);
    write_bytes(&movie.join("ev0050.usm"), b"garbage");
}

#[test]
fn scan_lists_standalone_and_bundled_mods_by_name() {
    let game = Game::new();
    populate_mods(&game);
    let result = scan(&game.dirs(), &Silent, &CancelToken::new()).unwrap();
    let summary: Vec<(&str, Option<&str>, usize)> = result
        .mods
        .iter()
        .map(|cutscene| {
            (
                cutscene.name.as_str(),
                cutscene.bundled_with_mod_id.as_deref(),
                cutscene.usm_count(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            ("Alpha", None, 0),
            ("Beta", Some("mymod"), 1),
            ("Broken", None, 1),
            ("Zeta HD", None, 3)
        ]
    );
    let zeta = result.find("Zeta HD").unwrap();
    assert_eq!(zeta.usm_files(), ["ev0010.usm", "ev0020.usm", "ev0030.usm"]);
    assert_eq!(zeta.path, game.paths.cutscenes_dir().join("Zeta HD"));
    assert_eq!(
        zeta.movie_dir(),
        game.paths.cutscenes_dir().join("Zeta HD").join("movie")
    );
    assert!(zeta.has_h264());
    assert_eq!((zeta.max_width(), zeta.max_height()), (2560, 1440));
    let broken = result.find("Broken").unwrap();
    assert_eq!(broken.files[0].video, None);
    assert!(!broken.has_h264());
    assert_eq!((broken.max_width(), broken.max_height()), (0, 0));
    let beta = result.find("Beta").unwrap();
    assert!(beta.is_bundled());
    assert!(!beta.has_h264());
    assert!(result.direct_overrides.is_empty());
    assert!(result.original_files.is_empty());
}

#[test]
fn custom_files_in_data_movie_are_direct_overrides() {
    let game = Game::new();
    populate_data_movie(&game);
    let result = scan(&game.dirs(), &Silent, &CancelToken::new()).unwrap();
    assert!(result.mods.is_empty());
    assert_eq!(result.direct_overrides, ["ev0030.usm", "ev0040.usm"]);
    assert_eq!(
        result.original_files,
        [
            "ev0010.usm",
            "ev0020.usm",
            "ev0030.usm",
            "ev0040.usm",
            "ev0050.usm"
        ]
    );
    assert!(result.has_custom_in_data_movie());
    assert!(has_custom_in_data_movie(&game.paths, &CancelToken::new()).unwrap());
    assert_eq!(result.detection().files_scanned, 0);
}

#[test]
fn vanilla_data_movie_has_no_overrides() {
    let game = Game::new();
    write_usm(
        &game.paths.movie_dir().join("ev0010.usm"),
        1600,
        904,
        SOFDEC,
    );
    write_usm(
        &game.paths.movie_dir().join("ev0020.usm"),
        1920,
        1080,
        SOFDEC,
    );
    assert!(!has_custom_in_data_movie(&game.paths, &CancelToken::new()).unwrap());
}

#[test]
fn detection_summarises_installed_mods_only() {
    let game = Game::new();
    populate_mods(&game);
    populate_data_movie(&game);
    let detection = detect(&game.paths, &CancelToken::new()).unwrap();
    assert!(detection.has_hd_cutscenes);
    assert!(detection.needs_h264);
    assert_eq!(detection.files_scanned, 4);
    assert_eq!(
        (detection.largest_width, detection.largest_height),
        (2560, 1080)
    );
    let from_scan = scan(&game.dirs(), &Silent, &CancelToken::new())
        .unwrap()
        .detection();
    assert_eq!(from_scan, detection);
}

#[test]
fn detection_of_an_empty_game_is_all_false() {
    let game = Game::new();
    let detection = detect(&game.paths, &CancelToken::new()).unwrap();
    assert_eq!(detection, yp_cutscenes::CutsceneDetection::default());
    let missing = GamePaths::new(game.root.path().join("missing"));
    assert!(
        !detect(&missing, &CancelToken::new())
            .unwrap()
            .has_hd_cutscenes
    );
}

#[test]
fn sofdec_only_mods_do_not_need_h264() {
    let game = Game::new();
    write_usm(
        &game
            .paths
            .cutscenes_dir()
            .join("HD")
            .join("movie")
            .join("ev0010.usm"),
        2560,
        1440,
        SOFDEC,
    );
    let detection = detect(&game.paths, &CancelToken::new()).unwrap();
    assert!(detection.has_hd_cutscenes);
    assert!(!detection.needs_h264);
}

#[test]
fn scan_reports_progress_over_mod_files() {
    let game = Game::new();
    populate_mods(&game);
    let events = Mutex::new(Vec::new());
    let observer = |progress: &ScanProgress| events.lock().unwrap().push(*progress);
    scan(&game.dirs(), &observer, &CancelToken::new()).unwrap();
    let events = events.into_inner().unwrap();
    assert_eq!(
        events.first(),
        Some(&ScanProgress {
            scanned: 0,
            total: 5
        })
    );
    assert_eq!(
        events.last(),
        Some(&ScanProgress {
            scanned: 5,
            total: 5
        })
    );
    assert!(
        events
            .windows(2)
            .all(|pair| pair[0].scanned <= pair[1].scanned)
    );
}

#[test]
fn scanning_honours_cancellation() {
    let game = Game::new();
    populate_mods(&game);
    let cancel = CancelToken::new();
    cancel.cancel();
    assert!(matches!(
        scan(&game.dirs(), &Silent, &cancel),
        Err(Error::Cancelled)
    ));
}

fn make_pack(game: &Game) -> PathBuf {
    let pack = game.outside("HD Pack v2");
    write_usm(
        &pack.join("Data").join("movie").join("ev0010.usm"),
        2560,
        1440,
        H264,
    );
    write_usm(
        &pack.join("Data").join("movie").join("ev0020.usm"),
        2560,
        1440,
        H264,
    );
    write_usm(
        &pack
            .join("Data")
            .join("movie")
            .join("extra")
            .join("ev0030.usm"),
        2560,
        1440,
        H264,
    );
    write_bytes(
        &pack.join("Data").join("movie").join("readme.txt"),
        b"hello",
    );
    write_bytes(&pack.join("Preview").join("ev0010.usm.png"), b"png");
    pack
}

#[test]
fn installs_a_folder_into_a_new_cutscene_mod() {
    let game = Game::new();
    let pack = make_pack(&game);
    let prepared = prepare(&pack, &game.context(), &NoProgress, &CancelToken::new()).unwrap();
    assert_eq!(prepared.movie_dir(), pack.join("Data").join("movie"));
    assert_eq!(prepared.default_name(), "HD Pack v2");
    assert_eq!(prepared.suggested_name(), "HD Pack v2");
    assert!(!prepared.was_extracted());

    let name = NameChoice::New("  HD Pack  ".to_string())
        .resolve()
        .unwrap();
    let events = Mutex::new(Vec::new());
    let observer = |progress: &CopyProgress| events.lock().unwrap().push(progress.clone());
    let outcome = install::install(
        &prepared,
        &game.paths.cutscenes_dir(),
        &name,
        &observer,
        &CancelToken::new(),
    )
    .unwrap();
    let target = game.paths.cutscenes_dir().join("HD Pack");
    assert_eq!(
        outcome,
        InstallOutcome::Installed {
            name: "HD Pack".to_string(),
            target: target.clone(),
            files: 3
        }
    );
    assert!(target.join("movie").join("ev0010.usm").is_file());
    assert!(
        target
            .join("movie")
            .join("extra")
            .join("ev0030.usm")
            .is_file()
    );
    assert!(!target.join("movie").join("readme.txt").exists());
    assert_eq!(
        std::fs::read(target.join("movie").join("ev0020.usm")).unwrap(),
        std::fs::read(pack.join("Data").join("movie").join("ev0020.usm")).unwrap()
    );

    let events = events.into_inner().unwrap();
    let indices: Vec<(usize, &str)> = events
        .iter()
        .map(|event| (event.file_index, event.file_name.as_str()))
        .collect();
    assert_eq!(
        indices,
        [(1, "ev0010.usm"), (2, "ev0020.usm"), (3, "ev0030.usm")]
    );
    assert!(events.iter().all(|event| event.file_count == 3));
    let total = usm_file(2560, 1440, H264).len() as u64 * 3;
    assert_eq!(events[0].bytes_total, total);
    assert_eq!(events[0].bytes_done, 0);

    let result = scan(&game.dirs(), &Silent, &CancelToken::new()).unwrap();
    let installed = result.find("HD Pack").unwrap();
    assert_eq!(installed.usm_files(), ["ev0010.usm", "ev0020.usm"]);
    assert!(installed.has_h264());
}

#[test]
fn merging_overwrites_and_adds_files() {
    let game = Game::new();
    let existing = game.paths.cutscenes_dir().join("HD").join("movie");
    write_bytes(&existing.join("ev0010.usm"), b"old");
    write_bytes(&existing.join("ev0099.usm"), b"keep");
    let pack = game.outside("patch");
    write_usm(&pack.join("ev0010.usm"), 2560, 1440, H264);
    write_usm(&pack.join("ev0040.usm"), 2560, 1440, H264);
    let prepared = prepare(&pack, &game.context(), &NoProgress, &CancelToken::new()).unwrap();
    assert_eq!(prepared.movie_dir(), pack.as_path());
    let name = NameChoice::MergeInto("HD".to_string()).resolve().unwrap();
    let outcome = install::install(
        &prepared,
        &game.paths.cutscenes_dir(),
        &name,
        &Silent,
        &CancelToken::new(),
    )
    .unwrap();
    assert!(matches!(
        outcome,
        InstallOutcome::Installed { files: 2, .. }
    ));
    assert_eq!(
        std::fs::read(existing.join("ev0010.usm")).unwrap(),
        usm_file(2560, 1440, H264)
    );
    assert_eq!(std::fs::read(existing.join("ev0099.usm")).unwrap(), b"keep");
    assert!(existing.join("ev0040.usm").is_file());
}

#[test]
fn installing_an_installed_folder_onto_itself_keeps_every_file() {
    let game = Game::new();
    let installed = game.paths.cutscenes_dir().join("HD");
    let movie = installed.join("movie");
    write_usm(&movie.join("ev0010.usm"), 2560, 1440, H264);
    write_usm(&movie.join("extra").join("ev0030.usm"), 2560, 1440, H264);
    let prepared = prepare(&installed, &game.context(), &NoProgress, &CancelToken::new()).unwrap();
    assert_eq!(prepared.default_name(), "HD");
    let name = NameChoice::MergeInto("HD".to_string()).resolve().unwrap();
    let events = Mutex::new(Vec::new());
    let observer = |progress: &CopyProgress| events.lock().unwrap().push(progress.clone());
    let outcome = install::install(
        &prepared,
        &game.paths.cutscenes_dir(),
        &name,
        &observer,
        &CancelToken::new(),
    )
    .unwrap();
    assert!(matches!(outcome, InstallOutcome::Installed { files: 2, .. }));
    let expected = usm_file(2560, 1440, H264);
    assert_eq!(std::fs::read(movie.join("ev0010.usm")).unwrap(), expected);
    assert_eq!(std::fs::read(movie.join("extra").join("ev0030.usm")).unwrap(), expected);
    let names: Vec<String> = std::fs::read_dir(&movie)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 2, "{names:?}");
    let last = events.into_inner().unwrap().pop().unwrap();
    assert_eq!(last.file_index, 2);
}

#[test]
fn a_cancelled_merge_leaves_the_installed_file_untouched() {
    let game = Game::new();
    let existing = game.paths.cutscenes_dir().join("HD").join("movie");
    write_bytes(&existing.join("ev0010.usm"), b"old");
    let pack = game.outside("patch");
    write_usm(&pack.join("ev0010.usm"), 2560, 1440, H264);
    let prepared = prepare(&pack, &game.context(), &NoProgress, &CancelToken::new()).unwrap();
    let cancel = CancelToken::new();
    let observer = |_: &CopyProgress| cancel.cancel();
    let result = install::install(&prepared, &game.paths.cutscenes_dir(), "HD", &observer, &cancel);
    assert!(matches!(result, Err(Error::Cancelled)));
    assert_eq!(std::fs::read(existing.join("ev0010.usm")).unwrap(), b"old");
    assert_eq!(std::fs::read_dir(&existing).unwrap().count(), 1);
}

#[test]
fn prepare_failures_are_typed() {
    let game = Game::new();
    let empty = game.outside("empty");
    write_bytes(&empty.join("readme.txt"), b"x");
    assert!(matches!(
        prepare(
            &empty,
            &game.context(),
            &NoProgress,
            &CancelToken::new()
        ),
        Err(PrepareFailure::NoUsm)
    ));
    assert!(matches!(
        prepare(
            &empty.join("readme.txt"),
            &game.context(),
            &NoProgress,
            &CancelToken::new()
        ),
        Err(PrepareFailure::NoUsm)
    ));
    assert!(matches!(
        prepare(
            &game.outside("missing"),
            &game.context(),
            &NoProgress,
            &CancelToken::new()
        ),
        Err(PrepareFailure::NoUsm)
    ));

    let videos = game.outside("videos");
    write_usm(&videos.join("cutscenes").join("ev0010.usm"), 1, 1, H264);
    assert!(matches!(
        prepare(
            &videos,
            &game.context(),
            &NoProgress,
            &CancelToken::new()
        ),
        Err(PrepareFailure::NoMovieFolder)
    ));
    assert!(matches!(
        prepare(
            &videos.join("cutscenes").join("ev0010.usm"),
            &game.context(),
            &NoProgress,
            &CancelToken::new()
        ),
        Err(PrepareFailure::NoMovieFolder)
    ));

    for name in ["pack.tar", "pack.tar.gz", "pack.bz2"] {
        let tarball = game.outside(name);
        write_bytes(&tarball, b"not something the launcher unpacks");
        assert_eq!(install::classify_source(&tarball), SourceKind::Unsupported, "{name}");
        assert!(
            matches!(
                prepare(&tarball, &game.context(), &NoProgress, &CancelToken::new()),
                Err(PrepareFailure::NoUsm)
            ),
            "{name}"
        );
    }
}

#[test]
fn sources_are_classified() {
    let game = Game::new();
    let pack = make_pack(&game);
    assert_eq!(install::classify_source(&pack), SourceKind::Directory);
    assert_eq!(
        install::classify_source(&game.outside("a.7Z")),
        SourceKind::Archive
    );
    assert_eq!(
        install::classify_source(&pack.join("Data").join("movie").join("ev0010.usm")),
        SourceKind::UsmFile
    );
    assert_eq!(
        install::classify_source(&pack.join("Data").join("movie").join("readme.txt")),
        SourceKind::Unsupported
    );
    assert_eq!(
        install::classify_source(&game.outside("nothing")),
        SourceKind::Unsupported
    );
    assert!(install::source_contains_usm(&pack, &game.extractor).unwrap());
    assert!(!install::source_contains_usm(&game.outside("nothing"), &game.extractor).unwrap());
    assert!(install::source_contains_usm(&game.outside("a.7z"), &game.extractor).is_err());
}

fn extract_failure(result: Result<yp_cutscenes::PreparedSource, PrepareFailure>) -> (bool, Option<ArchiveFailure>) {
    match result {
        Err(PrepareFailure::Extract { disk_full, error }) => (disk_full, error.archive_failure()),
        other => panic!("expected a failed extraction, got {other:?}"),
    }
}

#[test]
fn archives_without_usm_are_rejected_after_listing() {
    let game = Game::new();
    for extension in ["zip", "7z", "rar"] {
        let archive = game.outside(&format!("pack.{extension}"));
        let entries: [(&str, &[u8]); 2] = [("pack/readme.txt", b"read me"), ("pack/movie.png", b"png")];
        write_archive(&archive, &entries);
        assert!(
            matches!(
                prepare(&archive, &game.context(), &NoProgress, &CancelToken::new()),
                Err(PrepareFailure::NoUsm)
            ),
            "{extension}"
        );
    }
    assert!(!game.paths.scoped_temp_root().exists() || std::fs::read_dir(game.paths.scoped_temp_root()).unwrap().next().is_none());
}

#[test]
fn unreadable_archives_fail_with_their_reason() {
    let game = Game::new();
    let broken = game.outside("broken.7z");
    write_bytes(&broken, b"this is not an archive");
    assert_eq!(
        extract_failure(prepare(&broken, &game.context(), &NoProgress, &CancelToken::new())),
        (false, Some(ArchiveFailure::Corrupt))
    );

    let locked = game.outside("locked.zip");
    write_encrypted_zip(&locked, b"secret", &[("Data/movie/ev0010.usm", b"CRID")]);
    assert_eq!(
        extract_failure(prepare(&locked, &game.context(), &NoProgress, &CancelToken::new())),
        (false, Some(ArchiveFailure::PasswordRequired))
    );

    let headers = yp_core::testing::fixtures::path("yp_core/enc_headers.rar");
    assert_eq!(
        extract_failure(prepare(&headers, &game.context(), &NoProgress, &CancelToken::new())),
        (false, Some(ArchiveFailure::PasswordRequired))
    );
    assert!(!game.paths.scoped_temp_root().exists() || std::fs::read_dir(game.paths.scoped_temp_root()).unwrap().next().is_none());
}

#[test]
fn a_full_disk_is_flagged_on_the_failure() {
    let full = Error::archive(Path::new("pack.rar"), ArchiveFailure::DiskFull);
    assert!(install::is_disk_full(&full));
    let damaged = Error::archive(Path::new("pack.rar"), ArchiveFailure::Corrupt);
    assert!(!install::is_disk_full(&damaged));
    assert!(!install::is_disk_full(&Error::Cancelled));
}

#[test]
fn installs_from_a_real_archive_of_every_format_and_cleans_up() {
    for extension in ["zip", "7z", "rar"] {
        let game = Game::new();
        let pack = make_pack(&game);
        let archive = game.outside(&format!("HD Pack v2.{extension}"));
        write_archive_of_dir(&pack, &archive);
        let seen = Mutex::new(Vec::new());
        let sink = |progress: Progress| seen.lock().unwrap().push(progress.fraction);

        let prepared = prepare(&archive, &game.context(), &sink, &CancelToken::new()).unwrap();
        assert!(prepared.was_extracted(), "{extension}");
        assert_eq!(prepared.default_name(), "HD Pack v2", "{extension}");
        assert_eq!(seen.lock().unwrap().last(), Some(&Some(1.0)), "{extension}");
        let extracted_movie = prepared.movie_dir().to_path_buf();
        assert!(extracted_movie.starts_with(game.paths.scoped_temp_root()), "{extension}");
        let outcome = install::install(
            &prepared,
            &game.paths.cutscenes_dir(),
            "From Archive",
            &Silent,
            &CancelToken::new(),
        )
        .unwrap();
        assert!(
            matches!(outcome, InstallOutcome::Installed { files: 3, .. }),
            "{extension}"
        );
        drop(prepared);
        assert!(!extracted_movie.exists(), "{extension}");
        let installed = game
            .paths
            .cutscenes_dir()
            .join("From Archive")
            .join("movie")
            .join("extra")
            .join("ev0030.usm");
        assert_eq!(
            std::fs::read(installed).unwrap(),
            std::fs::read(pack.join("Data").join("movie").join("extra").join("ev0030.usm")).unwrap(),
            "{extension}"
        );
    }
}

#[test]
fn cancelling_while_a_cutscene_archive_unpacks_leaves_nothing_behind() {
    let game = Game::new();
    let archive = game.outside("Big Pack.zip");
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    write_zip(&archive, &[("Data/movie/ev0010.usm", &noise(1024 * 1024, 7))]);
    let cancel = CancelToken::new();
    let stop = cancel.clone();
    let sink = move |progress: Progress| {
        if progress.fraction.is_some_and(|fraction| fraction > 0.0) {
            stop.cancel();
        }
    };

    let result = prepare(&archive, &game.context(), &sink, &cancel);

    assert!(result.as_ref().is_err_and(PrepareFailure::is_cancelled), "{result:?}");
    assert!(std::fs::read_dir(game.paths.scoped_temp_root()).unwrap().next().is_none());
}

#[test]
fn installing_honours_cancellation() {
    let game = Game::new();
    let pack = make_pack(&game);
    let prepared = prepare(&pack, &game.context(), &NoProgress, &CancelToken::new()).unwrap();
    let cancel = CancelToken::new();
    let observer = |progress: &CopyProgress| {
        if progress.file_index == 2 {
            cancel.cancel();
        }
    };
    let result = install::install(
        &prepared,
        &game.paths.cutscenes_dir(),
        "HD",
        &observer,
        &cancel,
    );
    assert!(matches!(result, Err(Error::Cancelled)));
    let movie = game.paths.cutscenes_dir().join("HD").join("movie");
    assert!(movie.join("ev0010.usm").is_file());
    assert!(!movie.join("ev0020.usm").exists());
}

#[test]
fn deleting_a_scanned_mod_removes_its_folder() {
    let game = Game::new();
    populate_mods(&game);
    let result = scan(&game.dirs(), &Silent, &CancelToken::new()).unwrap();
    let beta = result.find("Beta").unwrap();
    assert!(delete_mod(&beta.path).unwrap());
    assert!(!beta.path.exists());
    assert!(game.paths.mods_dir().join("mymod").is_dir());
    assert!(!delete_mod(&beta.path).unwrap());
    let after = scan(&game.dirs(), &Silent, &CancelToken::new()).unwrap();
    assert!(after.find("Beta").is_none());
}

#[test]
#[ignore = "needs YP_GAME_DIR pointing at a NieR:Automata install with cutscene mods"]
fn scans_a_real_game_folder() {
    let root = PathBuf::from(std::env::var_os("YP_GAME_DIR").expect("YP_GAME_DIR"));
    let game = GamePaths::new(root);
    let result = scan(&ScanDirs::for_game(&game), &Silent, &CancelToken::new()).unwrap();
    for cutscene in &result.mods {
        assert!(
            cutscene.files.iter().all(|file| file.video.is_some()),
            "{}",
            cutscene.name
        );
    }
    let detection = result.detection();
    assert_eq!(detection.has_hd_cutscenes, detection.files_scanned > 0);
}
