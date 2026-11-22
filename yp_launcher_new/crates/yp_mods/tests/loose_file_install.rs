mod support;

use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use support::{detect, install_named, mod_dir, temp};
use tempfile::TempDir;
use yp_mods::detect::is_loose_data_file;
use yp_mods::loose_paths;
use yp_mods::{DropRejection, InstallFailure, ModKind};

fn drop_with(prefix: &str, files_by_stem: &[(&str, &[&str])]) -> TempDir {
    let dir = temp(prefix);
    for (stem, exts) in files_by_stem {
        for ext in *exts {
            std::fs::write(dir.path().join(format!("{stem}.{ext}")), [0u8; 32]).unwrap();
        }
    }
    dir
}

fn expected_dir(game_dir: &Path, mod_id: &str, subdir: &str) -> PathBuf {
    let base = mod_dir(game_dir, mod_id).join("data");
    if subdir.is_empty() {
        return base;
    }
    subdir.split('/').fold(base, |path, part| path.join(part))
}

#[test]
fn table_counts_match_the_generated_corpus() {
    let table = loose_paths::table();
    assert_eq!(table.subdir_for_stem.len(), 4632);
    assert_eq!(table.stems_with_vanilla_pair.len(), 2847);
    let subdirs: IndexSet<&str> = table.subdir_for_stem.values().map(String::as_str).collect();
    assert_eq!(subdirs.len(), 34);
}

#[test]
fn table_every_subdir_value_is_well_formed() {
    let bad: Vec<String> = loose_paths::table()
        .subdir_for_stem
        .iter()
        .filter(|(_, value)| {
            value.starts_with('/')
                || value.ends_with('/')
                || value.contains('\\')
                || value.contains("..")
                || value.contains("_extracted")
        })
        .map(|(key, value)| format!("{key} -> {value:?}"))
        .take(10)
        .collect();
    assert!(bad.is_empty(), "malformed subdirs: {bad:?}");
}

#[test]
fn table_paired_stems_are_a_subset_of_all_stems() {
    let table = loose_paths::table();
    let orphans: Vec<&String> = table
        .stems_with_vanilla_pair
        .iter()
        .filter(|stem| !table.subdir_for_stem.contains_key(*stem))
        .take(10)
        .collect();
    assert!(orphans.is_empty(), "paired-but-unknown: {orphans:?}");
}

#[test]
fn table_keeps_the_generated_order() {
    let table = loose_paths::table();
    assert_eq!(
        table
            .subdir_for_stem
            .get_index(0)
            .map(|(key, _)| key.as_str()),
        Some("0000")
    );
    assert_eq!(
        table.stems_with_vanilla_pair.first().map(String::as_str),
        Some("0000")
    );
    assert_eq!(
        table.subdir_for_stem.last().map(|(key, _)| key.as_str()),
        Some("wpf600")
    );
}

fn representative_stems() -> Vec<String> {
    let mut by_top: IndexMap<String, String> = IndexMap::new();
    let mut two_deep: Option<String> = None;
    let mut root_level: Option<String> = None;
    for (stem, sub) in &loose_paths::table().subdir_for_stem {
        let top = if sub.is_empty() {
            String::new()
        } else {
            sub.split('/').next().unwrap_or_default().to_owned()
        };
        by_top.entry(top).or_insert_with(|| stem.clone());
        if two_deep.is_none() && sub.split('/').count() == 2 {
            two_deep = Some(stem.clone());
        }
        if root_level.is_none() && sub.is_empty() {
            root_level = Some(stem.clone());
        }
    }
    let mut picked: IndexSet<String> = by_top.into_values().collect();
    picked.extend(two_deep);
    picked.extend(root_level);
    picked.into_iter().collect()
}

#[test]
fn every_subdir_type_places_files_at_the_exact_vanilla_location() {
    let stems = representative_stems();
    let drop = temp("yp_loose_cov_");
    for stem in &stems {
        std::fs::write(drop.path().join(format!("{stem}.dat")), [0u8; 32]).unwrap();
    }
    let game_dir = temp("yp_loose_cov_g_");

    let detected = detect(drop.path());
    assert_eq!(
        detected.kind,
        ModKind::Data,
        "loose drop rejected: {:?}",
        detected.rejection
    );

    let installed = install_named(game_dir.path(), drop.path(), "loose_coverage").unwrap();

    for stem in &stems {
        let subdir = loose_paths::subdir_for_stem(stem).unwrap();
        let dest = expected_dir(game_dir.path(), &installed.id, subdir).join(format!("{stem}.dat"));
        assert!(dest.is_file(), "{stem}.dat expected at {}", dest.display());
    }
}

#[test]
fn is_loose_data_file_recognizes_data_files_and_cpk() {
    assert!(is_loose_data_file("corehap.dat"));
    assert!(is_loose_data_file("pl0000.dtt"));
    assert!(is_loose_data_file("archive.cpk"));
    assert!(!is_loose_data_file("readme.txt"));
    assert!(!is_loose_data_file("setup.exe"));
}

#[test]
fn detect_accepts_a_bare_corehap_dat_file_path() {
    let dir = temp("yp_loose_file_");
    let file = dir.path().join("corehap.dat");
    std::fs::write(&file, [0u8; 32]).unwrap();

    let detected = detect(&file);
    assert_eq!(
        detected.kind,
        ModKind::Data,
        "bare file rejected: {:?}",
        detected.rejection
    );
    assert_eq!(detected.suggested_id, "corehap");
}

#[test]
fn install_from_a_bare_file_path_places_it_under_data_core() {
    let dir = temp("yp_loose_file_");
    let file = dir.path().join("corehap.dat");
    std::fs::write(&file, [0u8; 32]).unwrap();
    let game_dir = temp("yp_loose_file_g_");

    let installed = install_named(game_dir.path(), &file, "corehap").unwrap();
    let dest = expected_dir(game_dir.path(), &installed.id, "core").join("corehap.dat");
    assert!(dest.is_file(), "expected at {}", dest.display());
    assert!(file.is_file(), "source file must stay untouched");
}

#[test]
fn unrecognized_bare_file_is_rejected() {
    let dir = temp("yp_loose_file_");
    let file = dir.path().join("readme.txt");
    std::fs::write(&file, "x").unwrap();

    let detected = detect(&file);
    assert_eq!(detected.kind, ModKind::Unknown);
    assert_eq!(detected.rejection, Some(DropRejection::UnknownDrop));

    let game_dir = temp("yp_loose_file_g_");
    assert_eq!(
        install_named(game_dir.path(), &file, "readme"),
        Err(InstallFailure::Rejected(DropRejection::UnknownDrop))
    );
}

#[test]
fn a_paired_stem_dropped_without_its_dtt_warns() {
    let stem = loose_paths::table()
        .stems_with_vanilla_pair
        .first()
        .unwrap()
        .clone();
    let drop = drop_with("yp_loose_pair_", &[(&stem, &["dat"])]);
    let game_dir = temp("yp_loose_pair_g_");

    let installed = install_named(game_dir.path(), drop.path(), "loose_pair").unwrap();
    assert!(installed.unpaired_warnings.contains(&format!("{stem}.dtt")));
}

#[test]
fn a_vanilla_dat_only_stem_dropped_alone_does_not_warn() {
    let table = loose_paths::table();
    let stem = table
        .subdir_for_stem
        .keys()
        .find(|stem| !table.stems_with_vanilla_pair.contains(*stem))
        .unwrap()
        .clone();
    let drop = drop_with("yp_loose_solo_", &[(&stem, &["dat"])]);
    let game_dir = temp("yp_loose_solo_g_");

    let installed = install_named(game_dir.path(), drop.path(), "loose_solo").unwrap();
    assert!(installed.unpaired_warnings.is_empty());
}
