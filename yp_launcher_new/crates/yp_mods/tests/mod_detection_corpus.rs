mod support;

use std::path::Path;

use support::corpus::{CorpusArchive, archive_has_mod_content, discover_corpus};
use support::{
    corpus_root, detect, install_named, install_with, staging, temp, walk_files,
};
use yp_core::GamePaths;
use yp_mods::{DropRejection, InstallFailure, InstallRequest, InstallResult};

const CORPUS_FILTER_VAR: &str = "YP_CORPUS_FILTER";

fn has_payload(mod_root: &Path) -> bool {
    ["entities", "data", "wax"]
        .iter()
        .any(|sub| !walk_files(&mod_root.join(sub)).is_empty())
}

fn check_install(
    game_dir: &Path,
    archive: &CorpusArchive,
    result: &InstallResult,
) -> Result<(), String> {
    let installed = result
        .as_ref()
        .map_err(|failure| format!("install failed for {archive}: {failure:?}"))?;
    let mod_root = GamePaths::new(game_dir).mods_dir().join(&installed.id);
    for pack in &installed.side_installed_texture_packs {
        if !GamePaths::new(game_dir)
            .inject_textures_dir()
            .join(pack)
            .is_dir()
        {
            return Err(format!("texture pack not placed: {pack} for {archive}"));
        }
    }
    if mod_root.is_dir() {
        if !has_payload(&mod_root) {
            return Err(format!("mod folder has no payload for {archive}"));
        }
    } else if installed.side_installed_texture_packs.is_empty() {
        return Err(format!("no mod folder and no textures for {archive}"));
    }
    Ok(())
}

fn run_case(archive: &CorpusArchive) -> Result<(), String> {
    let game_dir = temp("yp_corpus_");
    let detected = detect(&archive.path);
    if detected.has_variants() {
        for variant in &detected.variants {
            let request = InstallRequest {
                requested_name: Some(format!("{}__{}", archive.unique_id(), variant.label)),
                variant_sub_path: Some(variant.sub_path.clone()),
                texture_pack_sub_paths: Vec::new(),
            };
            let result = install_with(game_dir.path(), &archive.path, &request);
            check_install(game_dir.path(), archive, &result)?;
        }
        return Ok(());
    }
    let result = install_named(game_dir.path(), &archive.path, &archive.unique_id());
    match &result {
        Err(InstallFailure::Rejected(DropRejection::TextureOnly)) => {
            eprintln!("SKIP texture-only {archive} (belongs in Textures tab)");
            return Ok(());
        }
        Err(InstallFailure::Rejected(DropRejection::UnsupportedNasa)) => {
            eprintln!("SKIP NASA-compressed {archive} (format is not supported)");
            return Ok(());
        }
        Err(failure) => {
            let entries = staging().list_entries(&archive.path).unwrap_or_default();
            if !archive_has_mod_content(&entries) {
                eprintln!("SKIP non-mod {archive} ({failure:?}, no mod content)");
                return Ok(());
            }
        }
        Ok(_) => {}
    }
    check_install(game_dir.path(), archive, &result)
}

#[test]
#[ignore]
fn outfit_corpus_real_drop_detect_install() {
    let root = corpus_root();
    let filter = std::env::var(CORPUS_FILTER_VAR).ok();
    let corpus: Vec<CorpusArchive> = discover_corpus(&root)
        .into_iter()
        .filter(|archive| {
            filter
                .as_deref()
                .is_none_or(|needle| archive.to_string().contains(needle))
        })
        .collect();
    if corpus.is_empty() {
        eprintln!("skipped: corpus not mounted at {}", root.display());
        return;
    }
    let mut failures = Vec::new();
    for archive in &corpus {
        if let Err(failure) = run_case(archive) {
            eprintln!("WARN {failure}");
            failures.push(failure);
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} archives failed:\n{}",
        failures.len(),
        corpus.len(),
        failures.join("\n")
    );
}

#[test]
fn corpus_reader_recognises_mod_content() {
    let entries = |items: &[&str]| {
        items
            .iter()
            .map(|item| (*item).to_owned())
            .collect::<Vec<_>>()
    };
    assert!(archive_has_mod_content(&entries(&[
        "Mod/data/pl/pl0000.dat"
    ])));
    assert!(archive_has_mod_content(&entries(&["entities/x.toml"])));
    assert!(archive_has_mod_content(&entries(&["Pack\\WP0100.DTT"])));
    assert!(archive_has_mod_content(&entries(&["a/b/MOD.TOML"])));
    assert!(archive_has_mod_content(&entries(&["tex/a.dds"])));
    assert!(!archive_has_mod_content(&entries(&[
        "readme.txt",
        "zz0000.dat",
        "database/x.txt"
    ])));
}

#[test]
fn corpus_reader_discovers_archives_sorted_case_insensitively() {
    let root = temp("yp_corpus_reader_");
    for (dir, file) in [
        ("200", "b.ZIP"),
        ("100", "A.7z"),
        ("100", "notes.txt"),
        ("300", "c.rar"),
    ] {
        let path = root.path().join(dir);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join(file), b"").unwrap();
    }
    let found: Vec<String> = discover_corpus(root.path())
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(found, vec!["100/A.7z", "200/b.zip", "300/c.rar"]);
    assert_eq!(discover_corpus(root.path())[0].unique_id(), "100_A");
}
