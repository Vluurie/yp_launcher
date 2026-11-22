pub mod pack_install;

mod drop_check;

pub use drop_check::{
    bundled_origin_by_pack, dir_contains_dds, drop_contains_dds, reconciled_load_order,
};

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use yp_core::{CancelToken, GamePaths, IoOp, IoResultExt, NoProgress};

use crate::fsx;
use crate::staging::{DropStaging, source_base_name};
use crate::text::utf16_cmp;

const TEXTURE_WRAPPER_SEGMENTS: [&str; 5] = [
    "sk_res",
    "far_res",
    "inject",
    "textures",
    "nierautomata.exe",
];
const WAX_BASE_DIR: &str = "!base";
const COPY_CHUNK: usize = 1024 * 1024;
const PROGRESS_STEP: u64 = 2 * 1024 * 1024;

pub fn is_texture(path: &Path) -> bool {
    fsx::lower_path_ends_with(path, ".dds")
}

pub fn texture_inject_dir(game: &GamePaths) -> PathBuf {
    game.inject_textures_dir()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureRoots {
    pub texture_dir: PathBuf,
    pub sk_res_dir: PathBuf,
    pub wax_mods_dir: PathBuf,
}

impl TextureRoots {
    pub fn for_game(game: &GamePaths) -> Self {
        Self {
            texture_dir: game.inject_textures_dir(),
            sk_res_dir: game.special_k_textures_dir().join("NieRAutomata.exe"),
            wax_mods_dir: game.wax_mods_dir(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextureListing {
    pub nams: Vec<String>,
    pub sk: Vec<String>,
    pub wax: Vec<String>,
    pub nams_folders: Vec<String>,
    pub conflicts: IndexMap<String, Vec<String>>,
}

fn sort_names(names: &mut [String]) {
    names.sort_by(|a, b| utf16_cmp(a, b));
}

pub fn load_textures(roots: &TextureRoots, disabled_packs: &[String]) -> TextureListing {
    let mut listing = TextureListing::default();
    if roots.texture_dir.is_dir() {
        for entity in fsx::children(&roots.texture_dir) {
            if disabled_packs.contains(&entity.name) {
                continue;
            }
            if entity.is_dir {
                listing.nams_folders.push(entity.name.clone());
            }
            listing.nams.push(entity.name);
        }
        sort_names(&mut listing.nams);
        sort_names(&mut listing.nams_folders);
    }

    if roots.sk_res_dir.is_dir() {
        listing.sk = fsx::child_dirs(&roots.sk_res_dir)
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        sort_names(&mut listing.sk);
    }

    if roots.wax_mods_dir.is_dir() {
        listing.wax = fsx::child_dirs(&roots.wax_mods_dir)
            .into_iter()
            .filter(|entry| entry.name != WAX_BASE_DIR)
            .filter(|entry| fsx::any_walked_file(&entry.path, is_texture))
            .map(|entry| entry.name)
            .collect();
        sort_names(&mut listing.wax);
    }

    let unique_folders: IndexSet<String> = listing.nams_folders.iter().cloned().collect();
    if unique_folders.len() > 1 {
        let mut file_owners: IndexMap<String, IndexSet<String>> = IndexMap::new();
        for folder in &unique_folders {
            let folder_dir = roots.texture_dir.join(folder);
            if !folder_dir.is_dir() {
                continue;
            }
            for file in fsx::walk_files(&folder_dir) {
                file_owners
                    .entry(fsx::file_name(&file).to_lowercase())
                    .or_default()
                    .insert(folder.clone());
            }
        }
        for owners in file_owners.values() {
            if owners.len() <= 1 {
                continue;
            }
            for folder in owners {
                let list = listing.conflicts.entry(folder.clone()).or_default();
                for other in owners {
                    if other != folder && !list.contains(other) {
                        list.push(other.clone());
                    }
                }
            }
        }
    }
    listing
}

pub fn delete_texture(path: &Path) -> yp_core::Result<bool> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).io(IoOp::Remove, path)?;
        return Ok(true);
    }
    if path.is_file() {
        std::fs::remove_file(path).io(IoOp::Remove, path)?;
        return Ok(true);
    }
    Ok(false)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureSource {
    pub path: PathBuf,
    pub folder: Option<String>,
}

impl TextureSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            folder: None,
        }
    }

    pub fn into_folder(path: impl Into<PathBuf>, folder: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            folder: Some(folder.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TextureCopyProgress {
    pub copied: u64,
    pub total: u64,
    pub files: usize,
    pub total_files: usize,
}

impl TextureCopyProgress {
    pub fn percent(&self) -> f64 {
        if self.total > 0 {
            self.copied as f64 / self.total as f64
        } else {
            0.0
        }
    }
}

pub fn collect_texture_files(source: &Path, folder_name: &str, out: &mut Vec<(PathBuf, PathBuf)>) {
    if !source.is_dir() {
        return;
    }
    for file in fsx::walk_files(source) {
        if !is_texture(&file) {
            continue;
        }
        let Ok(relative) = file.strip_prefix(source) else {
            continue;
        };
        let kept: Vec<_> = relative
            .components()
            .map(|part| part.as_os_str())
            .filter(|part| {
                !TEXTURE_WRAPPER_SEGMENTS.contains(&part.to_string_lossy().to_lowercase().as_str())
            })
            .collect();
        let clean: PathBuf = if kept.is_empty() {
            PathBuf::from(fsx::file_name(&file))
        } else {
            kept.into_iter().collect()
        };
        out.push((file.clone(), Path::new(folder_name).join(clean)));
    }
}

fn is_same_file(source: &Path, target: &Path) -> bool {
    match (std::fs::canonicalize(source), std::fs::canonicalize(target)) {
        (Ok(source), Ok(target)) => source == target,
        _ => false,
    }
}

pub fn install_textures(
    staging: &DropStaging,
    sources: &[TextureSource],
    texture_dir: &Path,
    on_progress: &mut dyn FnMut(TextureCopyProgress),
    cancel: &CancelToken,
) -> yp_core::Result<usize> {
    std::fs::create_dir_all(texture_dir).io(IoOp::Create, texture_dir)?;

    let mut extracted_dirs = Vec::new();
    let mut all_files: Vec<(PathBuf, PathBuf)> = Vec::new();
    for source in sources {
        if DropStaging::is_archive(&source.path) {
            let extracted = match staging.extract(&source.path, None, &NoProgress, cancel) {
                Ok(dir) => dir,
                Err(err) if err.is_cancelled() => return Err(err),
                Err(_) => continue,
            };
            let folder = source
                .folder
                .clone()
                .unwrap_or_else(|| source_base_name(&source.path));
            collect_texture_files(extracted.path(), &folder, &mut all_files);
            extracted_dirs.push(extracted);
        } else if source.path.is_dir() {
            let folder = source
                .folder
                .clone()
                .unwrap_or_else(|| fsx::file_name(&source.path));
            collect_texture_files(&source.path, &folder, &mut all_files);
        } else if source.path.is_file() && is_texture(&source.path) {
            all_files.push((source.path.clone(), PathBuf::new()));
        }
    }

    if all_files.is_empty() {
        return Ok(0);
    }

    let mut total = 0u64;
    for (file, _) in &all_files {
        total += std::fs::metadata(file).io(IoOp::Metadata, file)?.len();
    }

    let total_files = all_files.len();
    let mut copied = 0u64;
    let mut files_done = 0usize;
    let mut last_report = 0u64;
    let mut buffer = vec![0u8; COPY_CHUNK];
    for (file, relative_target) in &all_files {
        let target = if relative_target.as_os_str().is_empty() {
            texture_dir.join(fsx::file_name(file))
        } else {
            texture_dir.join(relative_target)
        };
        let parent = target.parent().unwrap_or(texture_dir);
        std::fs::create_dir_all(parent).io(IoOp::Create, parent)?;
        if is_same_file(file, &target) {
            copied += std::fs::metadata(file).io(IoOp::Metadata, file)?.len();
            files_done += 1;
            continue;
        }
        let mut input = std::fs::File::open(file).io(IoOp::Read, file)?;
        let mut output = tempfile::NamedTempFile::new_in(parent).io(IoOp::Create, parent)?;
        loop {
            cancel.check()?;
            let read = input.read(&mut buffer).io(IoOp::Read, file)?;
            if read == 0 {
                break;
            }
            output.write_all(&buffer[..read]).io(IoOp::Write, &target)?;
            copied += read as u64;
            if copied - last_report > PROGRESS_STEP {
                last_report = copied;
                on_progress(TextureCopyProgress {
                    copied,
                    total,
                    files: files_done,
                    total_files,
                });
            }
        }
        output.flush().io(IoOp::Write, &target)?;
        output
            .persist(&target)
            .map_err(|error| error.error)
            .io(IoOp::Write, &target)?;
        files_done += 1;
    }

    on_progress(TextureCopyProgress {
        copied: total,
        total,
        files: files_done,
        total_files,
    });
    drop(extracted_dirs);
    Ok(files_done)
}
