mod support;

use support::{archive_of, core_fixture, put_bytes, staging, temp};
use yp_core::ArchiveFailure;
use yp_mods::drop_gate::{DropGate, classify, classify_dir, classify_entry_paths};

#[test]
fn recognised_mod_layouts_pass_the_gate() {
    let cases: [&[&str]; 12] = [
        &["mod.toml", "readme.txt"],
        &["entities", "entities/weapon_01.toml"],
        &["wax", "wax/mods", "wax/mods/Dress/outfits/dress.json"],
        &["data", "data/pl", "data/pl/custom.bin"],
        &["pl", "pl/custom.bin"],
        &["pl0000.dat", "pl0000.dtt"],
        &["data100.cpk"],
        &["a.dds", "b.dds"],
        &["Pack/SK_Res/inject/textures/04ad8a8f.dds"],
        &[
            "Variant A/data/wp/wp0100.bin",
            "Variant B/data/wp/wp0100.bin",
        ],
        &["Mod\\Data\\PL\\PL0000.DAT"],
        &[
            "Wrapper",
            "Wrapper/Inner",
            "Wrapper/Inner/effect",
            "Wrapper/Inner/effect/x.bin",
        ],
    ];
    for entries in cases {
        assert_eq!(classify_entry_paths(entries), DropGate::Mod, "{entries:?}");
    }
    assert!(DropGate::Mod.is_mod());
}

#[test]
fn cutscene_only_drops_are_sent_to_the_cutscenes_tab() {
    let cases: [&[&str]; 5] = [
        &["data", "data/movie", "data/movie/ev0010.usm"],
        &["movie", "movie/ev0010.usm"],
        &["HD Cutscenes", "HD Cutscenes/data/movie/ev0010.usm"],
        &["Pack/Inner/movie/ev0010.usm", "Pack/Inner/readme.txt"],
        &["movie/ev0010.usm", "readme.txt", "docs/notes.txt"],
    ];
    for entries in cases {
        assert_eq!(
            classify_entry_paths(entries),
            DropGate::MisroutedCutscenes,
            "{entries:?}"
        );
    }
    assert!(!DropGate::MisroutedCutscenes.is_mod());
}

#[test]
fn cutscenes_next_to_mod_content_are_a_mod() {
    assert_eq!(
        classify_entry_paths(["data/movie/ev0010.usm", "data/pl/custom.bin"]),
        DropGate::Mod
    );
    assert_eq!(
        classify_entry_paths(["movie/ev0010.usm", "wp/custom.bin"]),
        DropGate::Mod
    );
}

#[test]
fn unrelated_content_is_not_a_mod() {
    let cases: [&[&str]; 8] = [
        &[],
        &["readme.txt"],
        &["setup.exe", "docs/manual.pdf"],
        &["data", "data/flutter_assets", "data/flutter_assets/a.bin"],
        &["sound/bgm.bin", "font/a.bin", "ui/b.bin"],
        &["a/data/movie/x.usm", "b/readme.txt"],
        &["only.dds"],
        &["", "/rooted.dds"],
    ];
    for entries in cases {
        assert_eq!(
            classify_entry_paths(entries),
            DropGate::NotAMod,
            "{entries:?}"
        );
    }
}

#[test]
fn only_four_wrapper_levels_are_stripped() {
    assert_eq!(
        classify_entry_paths(["1/2/3/4/movie/ev0010.usm"]),
        DropGate::MisroutedCutscenes
    );
    assert_eq!(
        classify_entry_paths(["1/2/3/4/5/movie/ev0010.usm"]),
        DropGate::NotAMod
    );
    assert_eq!(
        classify_entry_paths(["1/2/3/4/5/wp/custom.bin"]),
        DropGate::NotAMod
    );
    assert_eq!(
        classify_entry_paths(["1/2/3/4/5/data/wp/custom.bin"]),
        DropGate::Mod
    );
}

#[test]
fn folders_are_scanned_four_levels_deep() {
    let dir = temp("yp_gate_dir_");

    let overlay = dir.path().join("overlay");
    put_bytes(&overlay, "Wrapper/data/pl/custom.bin", 4, 0);
    assert_eq!(classify_dir(&overlay), DropGate::Mod);

    let cutscenes = dir.path().join("cutscenes");
    put_bytes(&cutscenes, "data/movie/ev0010.usm", 4, 0);
    assert_eq!(classify_dir(&cutscenes), DropGate::MisroutedCutscenes);

    let shallow = dir.path().join("shallow");
    put_bytes(&shallow, "a/b/c/pl0000.dat", 4, 0);
    assert_eq!(classify_dir(&shallow), DropGate::Mod);

    let deep = dir.path().join("deep");
    put_bytes(&deep, "a/b/c/d/pl0000.dat", 4, 0);
    assert_eq!(classify_dir(&deep), DropGate::NotAMod);

    let junk = dir.path().join("junk");
    put_bytes(&junk, "readme.txt", 4, 0);
    put_bytes(&junk, "tools/setup.exe", 4, 0);
    assert_eq!(classify_dir(&junk), DropGate::NotAMod);

    let empty = dir.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    assert_eq!(classify_dir(&empty), DropGate::NotAMod);
    assert_eq!(classify_dir(&dir.path().join("missing")), DropGate::NotAMod);
}

#[test]
fn classify_handles_files_folders_and_missing_paths() {
    let dir = temp("yp_gate_paths_");
    let loose = put_bytes(dir.path(), "corehap.dat", 4, 0);
    let cpk = put_bytes(dir.path(), "data100.cpk", 4, 0);
    let readme = put_bytes(dir.path(), "readme.txt", 4, 0);
    let folder = dir.path().join("Mod");
    put_bytes(&folder, "entities/quest_01.toml", 4, 0);

    let staging = staging();
    assert_eq!(classify(&staging, &loose), DropGate::Mod);
    assert_eq!(classify(&staging, &cpk), DropGate::Mod);
    assert_eq!(classify(&staging, &readme), DropGate::NotAMod);
    assert_eq!(classify(&staging, &folder), DropGate::Mod);
    assert_eq!(
        classify(&staging, &dir.path().join("missing")),
        DropGate::NotAMod
    );
}

#[test]
fn archives_that_cannot_be_read_say_why() {
    let dir = temp("yp_gate_unlisted_");
    let folder_named_like_an_archive = dir.path().join("folder.7z");
    put_bytes(&folder_named_like_an_archive, "data/pl/pl0000.dat", 4, 0);
    assert_eq!(
        classify(&staging(), &folder_named_like_an_archive),
        DropGate::NotAMod
    );
    for name in ["broken.zip", "broken.7z", "broken.rar"] {
        let fake = put_bytes(dir.path(), name, 16, 7);
        assert_eq!(
            classify(&staging(), &fake),
            DropGate::UnreadableArchive(ArchiveFailure::Corrupt),
            "{name}"
        );
    }
    for name in ["enc_headers.7z", "enc_headers.rar"] {
        assert_eq!(
            classify(&staging(), &core_fixture(name)),
            DropGate::UnreadableArchive(ArchiveFailure::PasswordRequired),
            "{name}"
        );
    }
    assert_eq!(
        classify(&staging(), &dir.path().join("missing.zip")),
        DropGate::NotAMod
    );
}

#[test]
fn tarballs_are_unsupported_files_not_broken_archives() {
    let dir = temp("yp_gate_tar_");
    for name in ["pack.tar", "pack.tar.gz", "pack.gz", "pack.bz2", "pack.tgz"] {
        let tarball = put_bytes(dir.path(), name, 64, 3);
        assert_eq!(classify(&staging(), &tarball), DropGate::NotAMod, "{name}");
    }
}

#[test]
fn archives_are_classified_from_their_listing() {
    let dir = temp("yp_gate_archives_");
    for extension in ["zip", "7z", "rar"] {
        let mod_archive = archive_of(
            dir.path(),
            &format!("Outfit.{extension}"),
            &[
                ("Outfit/data/pl/pl000d.dat", &[1; 8]),
                ("Outfit/readme.txt", &[2; 8]),
            ],
        );
        let cutscene_archive = archive_of(
            dir.path(),
            &format!("Cutscenes.{extension}"),
            &[("HD/data/movie/ev0010.usm", &[3; 8])],
        );
        let junk_archive = archive_of(
            dir.path(),
            &format!("Junk.{extension}"),
            &[("docs/manual.txt", &[4; 8])],
        );

        assert_eq!(classify(&staging(), &mod_archive), DropGate::Mod, "{extension}");
        assert_eq!(
            classify(&staging(), &cutscene_archive),
            DropGate::MisroutedCutscenes,
            "{extension}"
        );
        assert_eq!(classify(&staging(), &junk_archive), DropGate::NotAMod, "{extension}");
    }
}
