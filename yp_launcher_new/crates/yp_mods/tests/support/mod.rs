#![allow(dead_code)]

pub mod corpus;

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use yp_core::archive::Extractor;
use yp_core::testing::archives::write_archive_with_folders;
use yp_core::{CancelToken, GamePaths, HostEnv, NoProgress};
use yp_mods::detect::detect_drop;
use yp_mods::install::{install, install_variants};
use yp_mods::{DetectedDrop, DropStaging, InstallRequest, InstallResult, VariantInstallRequest};

pub const FIXTURES_DIR_VAR: &str = "YP_FIXTURES_DIR";
pub const CORPUS_DIR_VAR: &str = "YP_CORPUS_DIR";

pub fn temp(prefix: &str) -> TempDir {
    tempfile::Builder::new().prefix(prefix).tempdir().unwrap()
}

pub fn put_bytes(root: &Path, rel: &str, len: usize, fill: u8) -> PathBuf {
    let file = rel
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, vec![fill; len]).unwrap();
    file
}

pub fn put_json(root: &Path, rel: &str, value: &serde_json::Value) -> PathBuf {
    let file = rel
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part));
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, serde_json::to_string(value).unwrap()).unwrap();
    file
}

pub fn staging() -> DropStaging {
    DropStaging::new(Extractor::new(), HostEnv::current())
}

pub fn archive_of(work: &Path, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let out = work.join(name);
    write_archive_with_folders(&out, files);
    out
}

pub fn zip_of(work: &Path, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    archive_of(work, name, files)
}

pub fn core_fixture(name: &str) -> PathBuf {
    yp_core::testing::fixtures::path(&format!("yp_core/{name}"))
}

pub fn detect(source: &Path) -> DetectedDrop {
    detect_drop(&staging(), source, &NoProgress, &CancelToken::new()).unwrap()
}

pub fn install_with(game_dir: &Path, source: &Path, request: &InstallRequest) -> InstallResult {
    install(
        &staging(),
        &GamePaths::new(game_dir),
        source,
        request,
        &NoProgress,
        &CancelToken::new(),
    )
}

pub fn install_named(game_dir: &Path, source: &Path, name: &str) -> InstallResult {
    install_with(game_dir, source, &InstallRequest::named(name))
}

pub fn install_batch(
    game_dir: &Path,
    source: &Path,
    requests: &[VariantInstallRequest],
) -> Vec<InstallResult> {
    install_variants(
        &staging(),
        &GamePaths::new(game_dir),
        source,
        requests,
        &NoProgress,
        &CancelToken::new(),
    )
}

pub fn mod_dir(game_dir: &Path, id: &str) -> PathBuf {
    GamePaths::new(game_dir).mods_dir().join(id)
}

pub fn fixture(name: &str) -> PathBuf {
    let dir = std::env::var(FIXTURES_DIR_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../test/fixtures"));
    dir.join(name)
}

pub fn corpus_root() -> PathBuf {
    if let Ok(dir) = std::env::var(CORPUS_DIR_VAR) {
        return PathBuf::from(dir);
    }
    if cfg!(windows) {
        PathBuf::from(r"E:\all-outfit-mods\downloads")
    } else {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join("yp-corpus")
    }
}

pub fn file_names(dir: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

pub fn dir_names(dir: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

pub fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

pub fn rel_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap()
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn has_pl_model(root: &Path) -> bool {
    file_names(&root.join("data").join("pl"))
        .iter()
        .any(|name| {
            let lower = name.to_lowercase();
            lower.starts_with("pl") && (lower.ends_with(".dat") || lower.ends_with(".dtt"))
        })
}
