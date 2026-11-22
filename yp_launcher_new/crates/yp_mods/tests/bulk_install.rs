mod support;

use std::path::{Path, PathBuf};

use support::{dir_names, mod_dir, put_bytes, staging, temp};
use yp_core::{CancelToken, GamePaths, NoProgress};
use yp_mods::bulk::{find_archives, install_archive, install_auto_named};
use yp_mods::{DropRejection, InstallFailure, InstallResult};

fn weapon_drop(root: &Path, name: &str) -> PathBuf {
    let drop = root.join(name);
    put_bytes(&drop, "data/wp/wp0100.dat", 8, 1);
    drop
}

fn auto_named(game_dir: &Path, source: &Path, base_name: &str) -> InstallResult {
    install_auto_named(
        &staging(),
        &GamePaths::new(game_dir),
        source,
        base_name,
        &NoProgress,
        &CancelToken::new(),
    )
}

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

#[test]
fn finds_archives_in_the_folder_and_its_immediate_subfolders() {
    let dir = temp("yp_bulk_find_");
    let root = dir.path().join("Downloads");
    put_bytes(&root, "b.ZIP", 1, 0);
    put_bytes(&root, "A.7z", 1, 0);
    put_bytes(&root, "notes.txt", 1, 0);
    put_bytes(&root, "pack.tar.gz", 1, 0);
    put_bytes(&root, ".zip", 1, 0);
    put_bytes(&root, "335/Zeta.rar", 1, 0);
    put_bytes(&root, "335/alpha.zip", 1, 0);
    put_bytes(&root, "335/readme.md", 1, 0);
    put_bytes(&root, "335/deeper/hidden.zip", 1, 0);
    put_bytes(&root, "folder.zip/inner.7z", 1, 0);

    let found: Vec<String> = find_archives(&root)
        .iter()
        .map(|path| support::rel_slash(&root, path))
        .collect();
    assert_eq!(
        found,
        vec![
            "335/alpha.zip",
            "335/Zeta.rar",
            "A.7z",
            "b.ZIP",
            "folder.zip/inner.7z",
        ]
    );
}

#[test]
fn a_missing_folder_or_a_file_yields_no_archives() {
    let dir = temp("yp_bulk_missing_");
    let file = put_bytes(dir.path(), "single.zip", 1, 0);
    assert!(find_archives(&dir.path().join("missing")).is_empty());
    assert!(find_archives(&file).is_empty());
    assert_eq!(find_archives(dir.path()), vec![file]);
}

#[test]
fn name_collisions_get_a_numeric_suffix() {
    let drops = temp("yp_bulk_names_");
    let game_dir = temp("yp_bulk_names_g_");

    let first = auto_named(
        game_dir.path(),
        &weapon_drop(drops.path(), "one"),
        "Sword Pack",
    );
    let second = auto_named(
        game_dir.path(),
        &weapon_drop(drops.path(), "two"),
        "Sword Pack",
    );
    let third = auto_named(
        game_dir.path(),
        &weapon_drop(drops.path(), "three"),
        "Sword Pack",
    );

    assert_eq!(first.unwrap().id, "sword_pack");
    assert_eq!(second.unwrap().id, "sword_pack_2");
    assert_eq!(third.unwrap().id, "sword_pack_3");
    assert!(
        mod_dir(game_dir.path(), "sword_pack_3")
            .join("data")
            .join("wp")
            .join("wp0100.dat")
            .is_file()
    );
}

#[test]
fn gives_up_after_the_last_allowed_suffix() {
    let drops = temp("yp_bulk_full_");
    let game_dir = temp("yp_bulk_full_g_");
    let mods = GamePaths::new(game_dir.path()).mods_dir();
    std::fs::create_dir_all(mods.join("pack")).unwrap();
    for suffix in 2..=98 {
        std::fs::create_dir_all(mods.join(format!("pack_{suffix}"))).unwrap();
    }
    let drop = weapon_drop(drops.path(), "drop");

    let result = auto_named(game_dir.path(), &drop, "Pack");

    assert_eq!(
        result,
        Err(InstallFailure::Exists {
            id: "pack_98".to_owned()
        })
    );
    assert!(!mods.join("pack_99").exists());
    assert!(drop.join("data").join("wp").join("wp0100.dat").is_file());
}

#[test]
fn other_failures_are_returned_without_retrying() {
    let drops = temp("yp_bulk_reject_");
    let game_dir = temp("yp_bulk_reject_g_");
    let junk = drops.path().join("junk");
    put_bytes(&junk, "readme.txt", 4, 0);
    put_bytes(&junk, "notes.md", 4, 0);

    assert_eq!(
        auto_named(game_dir.path(), &junk, "Junk"),
        Err(InstallFailure::Rejected(DropRejection::UnknownDrop))
    );
    assert_eq!(
        bulk(game_dir.path(), &junk, "Junk"),
        Err(InstallFailure::Rejected(DropRejection::UnknownDrop))
    );
}

#[test]
fn bulk_install_of_a_plain_mod_uses_the_auto_naming() {
    let drops = temp("yp_bulk_plain_");
    let game_dir = temp("yp_bulk_plain_g_");
    std::fs::create_dir_all(GamePaths::new(game_dir.path()).mods_dir().join("blade")).unwrap();

    let installed = bulk(game_dir.path(), &weapon_drop(drops.path(), "drop"), "Blade").unwrap();

    assert_eq!(installed.id, "blade_2");
}

#[test]
fn bulk_install_of_a_variant_pack_installs_every_variant() {
    let drops = temp("yp_bulk_variants_");
    let game_dir = temp("yp_bulk_variants_g_");
    let pack = drops.path().join("pack");
    put_bytes(&pack, "Classic/data/pl/pl000d.dat", 8, 1);
    put_bytes(&pack, "Classic/data/pl/pl000d.dtt", 8, 1);
    put_bytes(&pack, "Sword/wp0100.dat", 8, 2);

    let installed = bulk(game_dir.path(), &pack, "Mock Machine").unwrap();

    assert_eq!(installed.id, "mock_machine_classic");
    assert!(installed.unpaired_warnings.is_empty());
    assert_eq!(
        dir_names(&GamePaths::new(game_dir.path()).mods_dir()),
        vec!["mock_machine_classic", "mock_machine_sword"]
    );
    assert!(
        mod_dir(game_dir.path(), "mock_machine_sword")
            .join("data")
            .join("wp")
            .join("wp0100.dat")
            .is_file()
    );
    assert!(pack.join("Classic").is_dir());
}

#[test]
fn a_cancelled_bulk_install_reports_cancellation() {
    let drops = temp("yp_bulk_cancel_");
    let game_dir = temp("yp_bulk_cancel_g_");
    let cancel = CancelToken::new();
    cancel.cancel();

    let result = install_archive(
        &staging(),
        &GamePaths::new(game_dir.path()),
        &weapon_drop(drops.path(), "drop"),
        "Blade",
        &NoProgress,
        &cancel,
    );

    assert_eq!(result, Err(InstallFailure::Cancelled));
    assert!(!GamePaths::new(game_dir.path()).mods_dir().exists());
}
