mod support;

use std::path::Path;

use support::{detect, file_names, fixture, install_named, mod_dir, temp};
use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::scan::list_installed;
use yp_mods::{DataCategory, ModKind};

fn cpks_in(mod_root: &Path) -> Vec<String> {
    file_names(&mod_root.join("data"))
        .into_iter()
        .filter(|name| name.to_lowercase().ends_with(".cpk"))
        .collect()
}

fn make_cpk_drop(prefix: &str, cpk_names: &[&str], wrapper: Option<&str>) -> TempDir {
    let dir = temp(prefix);
    let root = match wrapper {
        Some(wrapper) => dir.path().join(wrapper),
        None => dir.path().to_path_buf(),
    };
    std::fs::create_dir_all(&root).unwrap();
    for name in cpk_names {
        std::fs::write(root.join(name), [0x43u8; 64]).unwrap();
    }
    dir
}

#[test]
fn bare_cpk_folder_is_a_data_mod_not_an_unknown_drop() {
    let drop = make_cpk_drop("yp_cpk_bare_", &["data100.cpk"], None);
    let detected = detect(drop.path());
    assert_eq!(detected.kind, ModKind::Data, "{:?}", detected.rejection);
    assert_eq!(detected.rejection, None);
}

#[test]
fn wrapped_cpk_folder_unwraps_and_still_detects_as_data() {
    let drop = make_cpk_drop(
        "yp_cpk_wrapped_",
        &["data100.cpk", "data201.cpk"],
        Some("Nierdatafile"),
    );
    let detected = detect(drop.path());
    assert_eq!(detected.kind, ModKind::Data, "{:?}", detected.rejection);
    assert_eq!(
        detected
            .unwrapped_root
            .file_name()
            .unwrap()
            .to_string_lossy(),
        "Nierdatafile"
    );
}

#[test]
fn install_normalizes_loose_cpks_into_mod_data() {
    let drop = make_cpk_drop(
        "yp_cpk_install_",
        &["data100.cpk", "data201.cpk"],
        Some("Nierdatafile"),
    );
    let game_dir = temp("yp_cpk_game_");

    let installed = install_named(game_dir.path(), drop.path(), "cpk_mod").unwrap();
    let root = mod_dir(game_dir.path(), &installed.id);
    assert!(root.is_dir());
    assert_eq!(cpks_in(&root), vec!["data100.cpk", "data201.cpk"]);
    let stray: Vec<String> = file_names(&root)
        .into_iter()
        .filter(|name| name.to_lowercase().ends_with(".cpk"))
        .collect();
    assert!(stray.is_empty(), "cpks must not stay at the mod root");
}

#[test]
fn installed_cpk_mod_is_listed_with_an_archive_data_entry() {
    let drop = make_cpk_drop("yp_cpk_list_", &["data100.cpk"], None);
    let game_dir = temp("yp_cpk_listgame_");

    let installed = install_named(game_dir.path(), drop.path(), "cpk_listed").unwrap();
    let mods = list_installed(&GamePaths::new(game_dir.path())).unwrap();
    let listed = mods.iter().find(|item| item.id == installed.id).unwrap();

    assert_eq!(listed.kind, ModKind::Data);
    let archive: Vec<_> = listed
        .data
        .as_ref()
        .unwrap()
        .entries
        .iter()
        .filter(|entry| entry.category == DataCategory::Archive)
        .collect();
    assert_eq!(archive.len(), 1);
    assert_eq!(archive[0].file_count, 1);
}

#[test]
fn nexus_709_cpk_pack_detects_and_installs_cpks_into_data() {
    let archive = fixture("cpk_mod_709.zip");
    if !archive.is_file() {
        eprintln!("skipped: fixture not present at {}", archive.display());
        return;
    }
    let game_dir = temp("yp_cpk_709_");

    let detected = detect(&archive);
    assert_eq!(
        detected.kind,
        ModKind::Data,
        "real cpk pack rejected: {:?}",
        detected.rejection
    );
    assert!(
        !detected.has_variants(),
        "a flat cpk pack must not ask for variant selection"
    );

    let installed = install_named(game_dir.path(), &archive, "nier_cpk_709").unwrap();
    let names = cpks_in(&mod_dir(game_dir.path(), &installed.id));
    assert!(names.contains(&"data100.cpk".to_owned()), "{names:?}");
    assert!(names.contains(&"data201.cpk".to_owned()), "{names:?}");
}
