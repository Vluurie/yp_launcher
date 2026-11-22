use yp_core::{CancelToken, GamePaths, Result};

use crate::observer::Silent;
use crate::scan::{CutsceneMod, ScanDirs, direct_overrides, scan_mods};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CutsceneDetection {
    pub has_hd_cutscenes: bool,
    pub needs_h264: bool,
    pub files_scanned: usize,
    pub largest_width: u32,
    pub largest_height: u32,
}

pub fn detection_of(mods: &[CutsceneMod]) -> CutsceneDetection {
    let mut detection = CutsceneDetection::default();
    let mut largest_pixels = 0;
    for video in mods.iter().flat_map(CutsceneMod::videos) {
        detection.files_scanned += 1;
        detection.needs_h264 |= video.codec.is_h264();
        if video.pixels() > largest_pixels {
            largest_pixels = video.pixels();
            detection.largest_width = video.width;
            detection.largest_height = video.height;
        }
    }
    detection.has_hd_cutscenes = detection.files_scanned > 0;
    detection
}

pub fn detect(game: &GamePaths, cancel: &CancelToken) -> Result<CutsceneDetection> {
    let mods = scan_mods(&ScanDirs::for_game(game), &Silent, cancel)?;
    Ok(detection_of(&mods))
}

pub fn has_custom_in_data_movie(game: &GamePaths, cancel: &CancelToken) -> Result<bool> {
    Ok(!direct_overrides(&game.movie_dir(), cancel)?.is_empty())
}
