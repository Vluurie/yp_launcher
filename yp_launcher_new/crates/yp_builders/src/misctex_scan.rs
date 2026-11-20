use std::path::{Path, PathBuf};

use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::error::Result;

pub const FILE_PREFIX: &str = "misctex_";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MisctexEntry {
    pub mod_id: String,
    pub texture_name: String,
    pub dat_path: PathBuf,
    pub dtt_path: PathBuf,
    pub total_bytes: u64,
}

impl MisctexEntry {
    pub fn file_stem(&self) -> String {
        format!("{FILE_PREFIX}{}", self.texture_name)
    }

    pub fn directory(&self) -> &Path {
        self.dat_path.parent().unwrap_or(Path::new(""))
    }
}

fn entries_of(dir: &Path) -> Vec<std::fs::DirEntry> {
    std::fs::read_dir(dir).map(|read| read.filter_map(|entry| entry.ok()).collect()).unwrap_or_default()
}

fn is_kind(entry: &std::fs::DirEntry, dir: bool) -> bool {
    entry.file_type().is_ok_and(|kind| if dir { kind.is_dir() } else { kind.is_file() })
}

pub fn scan(game: &GamePaths) -> Vec<MisctexEntry> {
    if game.root().as_os_str().is_empty() {
        return Vec::new();
    }
    let mods_root = game.mods_dir();
    let mut found = Vec::new();
    for mod_entry in entries_of(&mods_root).iter().filter(|entry| is_kind(entry, true)) {
        let mod_id = mod_entry.file_name().to_string_lossy().into_owned();
        let misctex_dir = mod_entry.path().join("data").join("misctex");
        for file in entries_of(&misctex_dir).iter().filter(|entry| is_kind(entry, false)) {
            let name = file.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(".dat") else {
                continue;
            };
            let Some(texture_name) = stem.strip_prefix(FILE_PREFIX) else {
                continue;
            };
            let dtt_path = misctex_dir.join(format!("{stem}.dtt"));
            let dat_bytes = file.metadata().map(|meta| meta.len()).unwrap_or(0);
            let dtt_bytes = std::fs::metadata(&dtt_path).ok().filter(|meta| meta.is_file()).map_or(0, |meta| meta.len());
            found.push(MisctexEntry {
                mod_id: mod_id.clone(),
                texture_name: texture_name.to_owned(),
                dat_path: file.path(),
                dtt_path,
                total_bytes: dat_bytes + dtt_bytes,
            });
        }
    }
    found.sort_by(|a, b| a.mod_id.cmp(&b.mod_id).then_with(|| a.texture_name.cmp(&b.texture_name)));
    found
}

pub fn remove(entry: &MisctexEntry) -> Result<()> {
    for path in [&entry.dat_path, &entry.dtt_path] {
        if path.is_file() {
            std::fs::remove_file(path).io(IoOp::Remove, path)?;
        }
    }
    Ok(())
}

pub fn watch_path(game: &GamePaths) -> Option<PathBuf> {
    if game.root().as_os_str().is_empty() {
        return None;
    }
    Some(game.mods_dir()).filter(|dir| dir.is_dir())
}

pub fn is_relevant_change(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.ends_with(".dat") || text.ends_with(".dtt")
}
