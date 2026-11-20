use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use yp_core::{CancelToken, GamePaths, Result};
use yp_formats::usm::{self, VideoInfo};

use crate::listing::{file_name, usm_files, visible_child_dirs};
use crate::observer::Observer;

const MOVIE_DIR: &str = "movie";
const BUNDLED_CUTSCENES_DIR: &str = "cutscenes";
const PROGRESS_EVERY: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanDirs {
    pub cutscenes: PathBuf,
    pub original_movie: PathBuf,
    pub mods: PathBuf,
}

impl ScanDirs {
    pub fn for_game(game: &GamePaths) -> Self {
        Self {
            cutscenes: game.cutscenes_dir(),
            original_movie: game.movie_dir(),
            mods: game.mods_dir(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutsceneFile {
    pub name: String,
    pub video: Option<VideoInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutsceneMod {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<CutsceneFile>,
    pub bundled_with_mod_id: Option<String>,
}

impl CutsceneMod {
    pub fn movie_dir(&self) -> PathBuf {
        self.path.join(MOVIE_DIR)
    }

    pub fn usm_count(&self) -> usize {
        self.files.len()
    }

    pub fn usm_files(&self) -> Vec<&str> {
        self.files.iter().map(|file| file.name.as_str()).collect()
    }

    pub(crate) fn videos(&self) -> impl Iterator<Item = &VideoInfo> {
        self.files.iter().filter_map(|file| file.video.as_ref())
    }

    pub fn has_h264(&self) -> bool {
        self.videos().any(|video| video.codec.is_h264())
    }

    pub fn max_width(&self) -> u32 {
        self.videos().map(|video| video.width).max().unwrap_or(0)
    }

    pub fn max_height(&self) -> u32 {
        self.videos().map(|video| video.height).max().unwrap_or(0)
    }

    pub fn is_bundled(&self) -> bool {
        self.bundled_with_mod_id.is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanResult {
    pub mods: Vec<CutsceneMod>,
    pub direct_overrides: Vec<String>,
    pub original_files: Vec<String>,
}

impl ScanResult {
    pub fn detection(&self) -> crate::detect::CutsceneDetection {
        crate::detect::detection_of(&self.mods)
    }

    pub fn has_custom_in_data_movie(&self) -> bool {
        !self.direct_overrides.is_empty()
    }

    pub fn find(&self, name: &str) -> Option<&CutsceneMod> {
        self.mods.iter().find(|candidate| candidate.name == name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanProgress {
    pub scanned: usize,
    pub total: usize,
}

struct ModFolder {
    dir: PathBuf,
    bundled_with_mod_id: Option<String>,
    usm: Vec<PathBuf>,
}

fn mod_folders(dirs: &ScanDirs) -> Result<Vec<ModFolder>> {
    let mut folders = Vec::new();
    for dir in visible_child_dirs(&dirs.cutscenes)? {
        let usm = usm_files(&dir.join(MOVIE_DIR))?;
        folders.push(ModFolder {
            dir,
            bundled_with_mod_id: None,
            usm,
        });
    }
    for mod_dir in visible_child_dirs(&dirs.mods)? {
        let mod_id = file_name(&mod_dir);
        for dir in visible_child_dirs(&mod_dir.join(BUNDLED_CUTSCENES_DIR))? {
            let usm = usm_files(&dir.join(MOVIE_DIR))?;
            folders.push(ModFolder {
                dir,
                bundled_with_mod_id: Some(mod_id.clone()),
                usm,
            });
        }
    }
    Ok(folders)
}

pub fn scan_mods(
    dirs: &ScanDirs,
    progress: &dyn Observer<ScanProgress>,
    cancel: &CancelToken,
) -> Result<Vec<CutsceneMod>> {
    let folders = mod_folders(dirs)?;
    let total = folders.iter().map(|folder| folder.usm.len()).sum();
    let mut scanned = 0;
    progress.observe(&ScanProgress { scanned, total });
    let mut mods = Vec::with_capacity(folders.len());
    for folder in folders {
        let mut files = Vec::with_capacity(folder.usm.len());
        for path in &folder.usm {
            cancel.check()?;
            files.push(CutsceneFile {
                name: file_name(path),
                video: usm::probe_quiet(path),
            });
            scanned += 1;
            if scanned % PROGRESS_EVERY == 0 || scanned == total {
                progress.observe(&ScanProgress { scanned, total });
            }
        }
        files.sort_by(|a, b| a.name.cmp(&b.name));
        mods.push(CutsceneMod {
            name: file_name(&folder.dir),
            path: folder.dir,
            files,
            bundled_with_mod_id: folder.bundled_with_mod_id,
        });
    }
    mods.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.bundled_with_mod_id.cmp(&b.bundled_with_mod_id))
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(mods)
}

pub fn original_files(original_movie: &Path) -> Result<Vec<String>> {
    let names: BTreeSet<String> = usm_files(original_movie)?
        .iter()
        .map(|path| file_name(path).to_lowercase())
        .collect();
    Ok(names.into_iter().collect())
}

pub fn direct_overrides(original_movie: &Path, cancel: &CancelToken) -> Result<Vec<String>> {
    let mut overrides = Vec::new();
    for path in usm_files(original_movie)? {
        cancel.check()?;
        let Some(video) = usm::probe_quiet(&path) else {
            continue;
        };
        if !video.is_vanilla_shape() {
            overrides.push(file_name(&path));
        }
    }
    Ok(overrides)
}

pub fn scan(
    dirs: &ScanDirs,
    progress: &dyn Observer<ScanProgress>,
    cancel: &CancelToken,
) -> Result<ScanResult> {
    let mods = scan_mods(dirs, progress, cancel)?;
    Ok(ScanResult {
        mods,
        direct_overrides: direct_overrides(&dirs.original_movie, cancel)?,
        original_files: original_files(&dirs.original_movie)?,
    })
}
