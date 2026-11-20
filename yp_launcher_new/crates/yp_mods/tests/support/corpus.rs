use std::path::{Path, PathBuf};

const ARCHIVE_EXTS: [&str; 3] = ["zip", "7z", "rar"];
const MOD_STEM_PREFIXES: [&str; 10] = ["pl", "wp", "em", "bg", "ba", "bh", "et", "it", "um", "wd"];

#[derive(Debug, Clone)]
pub struct CorpusArchive {
    pub path: PathBuf,
    pub ext: String,
    pub mod_id: String,
    pub basename: String,
}

impl CorpusArchive {
    pub fn unique_id(&self) -> String {
        format!("{}_{}", self.mod_id, self.basename)
    }
}

impl std::fmt::Display for CorpusArchive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}.{}", self.mod_id, self.basename, self.ext)
    }
}

pub fn discover_corpus(root: &Path) -> Vec<CorpusArchive> {
    let Ok(mods) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for mod_entry in mods.filter_map(Result::ok) {
        let mod_path = mod_entry.path();
        if !mod_path.is_dir() {
            continue;
        }
        let mod_id = mod_entry.file_name().to_string_lossy().into_owned();
        let Ok(files) = std::fs::read_dir(&mod_path) else {
            continue;
        };
        for file in files.filter_map(Result::ok) {
            let path = file.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .map(|ext| ext.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if !ARCHIVE_EXTS.contains(&ext.as_str()) {
                continue;
            }
            let basename = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push(CorpusArchive {
                path,
                ext,
                mod_id: mod_id.clone(),
                basename,
            });
        }
    }
    out.sort_by_key(|archive| archive.path.to_string_lossy().to_lowercase());
    out
}

pub fn archive_has_mod_content(entries: &[String]) -> bool {
    entries.iter().any(|raw| {
        let entry = raw.to_lowercase().replace('\\', "/");
        let segments: Vec<&str> = entry.split('/').collect();
        let (base, dirs) = segments
            .split_last()
            .map_or(("", &[][..]), |(last, dirs)| (*last, dirs));
        if base == "mod.toml"
            || dirs
                .iter()
                .any(|dir| ["entities", "wax", "data"].contains(dir))
        {
            return true;
        }
        if entry.ends_with(".dds") || entry.ends_with(".cpk") {
            return true;
        }
        (entry.ends_with(".dat") || entry.ends_with(".dtt"))
            && MOD_STEM_PREFIXES
                .iter()
                .any(|prefix| base.starts_with(prefix))
    })
}
