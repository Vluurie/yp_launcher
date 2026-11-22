#![allow(dead_code)]

use std::path::{Path, PathBuf};

use tempfile::TempDir;
use yp_core::GamePaths;

pub fn temp(prefix: &str) -> TempDir {
    tempfile::Builder::new().prefix(prefix).tempdir().unwrap()
}

pub fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

pub fn write_bytes(path: &Path, content: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

pub fn write_plain_dll(path: &Path) {
    write_plain_dll_with(path, "celestial path timing");
}

pub fn write_plain_dll_with(path: &Path, body: &str) {
    let mut bytes = vec![0x4D, 0x5A, 0x90, 0x00];
    bytes.extend_from_slice(body.as_bytes());
    write_bytes(path, &bytes);
}

pub fn write_marked_dll(path: &Path, marker: &str) {
    write_plain_dll_with(path, marker);
}

pub struct FreshGame {
    pub dir: TempDir,
    pub game: GamePaths,
}

pub fn fresh_game(prefix: &str, nams_toml: &str) -> FreshGame {
    let dir = temp(prefix);
    write(&dir.path().join("nams").join("nams.toml"), nams_toml);
    let game = GamePaths::new(dir.path());
    FreshGame { dir, game }
}

pub fn read_toml(path: &Path) -> toml::Table {
    toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

pub fn read_nams(game: &GamePaths) -> toml::Table {
    read_toml(&game.nams_toml())
}

pub fn nams_flag(game: &GamePaths, key: &str) -> Option<bool> {
    read_nams(game).get(key).and_then(toml::Value::as_bool)
}

pub fn file_names(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|path| path.file_name().unwrap().to_string_lossy().into_owned()).collect()
}

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

pub fn core_fixture(name: &str) -> PathBuf {
    yp_core::testing::fixtures::path(&format!("yp_core/{name}"))
}

pub fn archive_of_dir(content_dir: &Path, archive: &Path) {
    yp_core::testing::archives::write_archive_of_dir(content_dir, archive);
}

pub fn shader_corpus_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("YP_SHADER_CORPUS_DIR") {
        return PathBuf::from(dir);
    }
    if cfg!(windows) {
        PathBuf::from(r"E:\all-outfit-mods\downloads\shaders")
    } else {
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join("yp-shader-corpus")
    }
}

#[derive(Debug, Clone)]
pub struct CorpusArchive {
    pub path: PathBuf,
    pub mod_id: String,
    pub basename: String,
}

impl CorpusArchive {
    pub fn unique_id(&self) -> String {
        format!("{}_{}", self.mod_id, self.basename)
    }
}

pub fn discover_corpus(root: &Path) -> Vec<CorpusArchive> {
    let mut out = Vec::new();
    let Ok(mods) = std::fs::read_dir(root) else {
        return out;
    };
    for mod_dir in mods.filter_map(Result::ok).filter(|entry| entry.path().is_dir()) {
        let mod_id = mod_dir.file_name().to_string_lossy().into_owned();
        let Ok(files) = std::fs::read_dir(mod_dir.path()) else {
            continue;
        };
        for file in files.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.is_file()) {
            let ext = file.extension().map(|ext| ext.to_string_lossy().to_lowercase()).unwrap_or_default();
            if !matches!(ext.as_str(), "zip" | "7z" | "rar") {
                continue;
            }
            out.push(CorpusArchive {
                basename: file.file_stem().unwrap().to_string_lossy().into_owned(),
                mod_id: mod_id.clone(),
                path: file,
            });
        }
    }
    out.sort_by_cached_key(|archive| archive.path.to_string_lossy().to_lowercase());
    out
}

pub fn corpus_available() -> bool {
    shader_corpus_root().is_dir()
}

pub fn extract(archive: &Path, out_dir: &Path) -> bool {
    yp_core::Extractor::new()
        .extract(archive, out_dir, &yp_core::NoProgress, &yp_core::CancelToken::new())
        .is_ok()
}

pub fn find_archive<'a>(all: &'a [CorpusArchive], mod_id_part: &str) -> Option<&'a CorpusArchive> {
    let wanted = mod_id_part.to_lowercase();
    all.iter().find(|archive| archive.mod_id.to_lowercase().contains(&wanted))
}

pub fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}
