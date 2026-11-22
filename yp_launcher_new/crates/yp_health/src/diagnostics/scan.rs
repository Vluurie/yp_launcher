use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use walkdir::WalkDir;
use yp_core::fs::{file_name_lossy, list_sorted};
use yp_core::{GamePaths, IoOp, IoResultExt, Result};

use super::model::{VANILLA_DROP_CAP, VanillaBucket, VanillaDirFinding};

pub const HIDDEN_MOD_PREFIX: char = '_';
pub const SPECIAL_K_GAME_EXE_DIR: &str = "NieRAutomata.exe";
pub const WAX_TEXTURES_DIR: &str = "textures";
pub const LEGACY_RESHADE_DIR: &str = "reshade";

pub const VANILLA_DATA_DIRS: [&str; 4] = ["movie", "enlighten", "sound", "movie_logo"];

pub const VANILLA_GAME_ROOT_ENTRIES: [&str; 11] = [
    "data",
    "wallpaper",
    "redist",
    "logs",
    "nams",
    "nierautomata.exe",
    "nierautomatacompat.exe",
    "latest_buildinfo.properties",
    "steam_api64.dll",
    "installscript.vdf",
    "steam_appid.txt",
];

pub const VANILLA_DATA_ROOT_FILES: [&str; 27] = [
    "data000.cpk",
    "data001.cpk",
    "data002.cpk",
    "data003.cpk",
    "data004.cpk",
    "data005.cpk",
    "data006.cpk",
    "data007.cpk",
    "data008.cpk",
    "data009.cpk",
    "data010.cpk",
    "data011.cpk",
    "data012.cpk",
    "data013.cpk",
    "data014.cpk",
    "data015.cpk",
    "data016.cpk",
    "data017.cpk",
    "data018.cpk",
    "data019.cpk",
    "data100.cpk",
    "data101.cpk",
    "data102.cpk",
    "data103.cpk",
    "data105.cpk",
    "data107.cpk",
    "data108.cpk",
];

fn entries(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    list_sorted(dir)
}

fn dir_names(dir: &Path) -> Result<Vec<String>> {
    Ok(entries(dir)?
        .iter()
        .filter(|path| path.is_dir())
        .map(|path| file_name_lossy(path))
        .collect())
}

fn is_plain_dir(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir())
}

fn is_plain_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file())
}

pub(crate) fn hidden_mod_folders(game: &GamePaths) -> Result<Vec<String>> {
    let mut names: Vec<String> = entries(&game.mods_dir())?
        .iter()
        .filter(|path| is_plain_dir(path))
        .map(|path| file_name_lossy(path))
        .filter(|name| name.starts_with(HIDDEN_MOD_PREFIX))
        .collect();
    names.sort();
    Ok(names)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TextureScan {
    pub nams: Vec<String>,
    pub special_k: Vec<String>,
    pub wax: Vec<String>,
}

pub(crate) fn scan_textures(game: &GamePaths) -> Result<TextureScan> {
    let wax = entries(&game.wax_mods_dir())?
        .iter()
        .filter(|path| path.is_dir() && path.join(WAX_TEXTURES_DIR).is_dir())
        .map(|path| file_name_lossy(path))
        .collect();
    Ok(TextureScan {
        nams: dir_names(&game.inject_textures_dir())?,
        special_k: dir_names(&game.special_k_textures_dir().join(SPECIAL_K_GAME_EXE_DIR))?,
        wax,
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExtraScan {
    pub data_dir_contents: IndexMap<String, usize>,
    pub wax_mods_root: Vec<String>,
    pub sk_res_packs: Vec<String>,
    pub reshade_contents: Vec<String>,
    pub game_root_extras: Vec<String>,
}

fn count_files(dir: &Path) -> usize {
    WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .count()
}

pub(crate) fn scan_extras(game: &GamePaths) -> Result<ExtraScan> {
    let mut scan = ExtraScan::default();
    for name in dir_names(&game.data_dir())? {
        if VANILLA_DATA_DIRS.contains(&name.to_lowercase().as_str()) {
            continue;
        }
        let count = count_files(&game.data_dir().join(&name));
        scan.data_dir_contents.insert(name, count);
    }
    scan.wax_mods_root = dir_names(&game.wax_mods_dir())?;
    let special_k = game.special_k_textures_dir();
    for exe_dir in dir_names(&special_k)? {
        scan.sk_res_packs
            .extend(dir_names(&special_k.join(exe_dir))?);
    }
    scan.reshade_contents = entries(&game.root().join(LEGACY_RESHADE_DIR))?
        .iter()
        .map(|path| file_name_lossy(path))
        .collect();
    for path in entries(game.root())? {
        let name = file_name_lossy(&path);
        if VANILLA_GAME_ROOT_ENTRIES.contains(&name.to_lowercase().as_str()) {
            continue;
        }
        scan.game_root_extras.push(if path.is_dir() {
            format!("{name}/")
        } else {
            name
        });
    }
    scan.game_root_extras
        .sort_by_key(|name| name.to_lowercase());
    Ok(scan)
}

pub(crate) fn scan_vanilla_drops(game: &GamePaths) -> Result<Vec<VanillaDirFinding>> {
    let data_dir = game.data_dir();
    if !data_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut findings: Vec<VanillaDirFinding> = std::fs::read_dir(&data_dir)
        .io(IoOp::List, &data_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| is_plain_file(path))
        .map(|path| (file_name_lossy(&path), path))
        .filter(|(name, _)| !VANILLA_DATA_ROOT_FILES.contains(&name.to_lowercase().as_str()))
        .map(|(name, path)| VanillaDirFinding {
            path: format!("data/{name}"),
            size_bytes: std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0),
            bucket: VanillaBucket::DataRootLoose,
        })
        .collect();
    findings.sort_by_cached_key(|finding| (finding.path.to_lowercase(), finding.path.clone()));
    findings.truncate(VANILLA_DROP_CAP);
    Ok(findings)
}
