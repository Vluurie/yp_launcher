use std::path::Path;

use yp_core::{AppPaths, GamePaths, PlatformAdapter, SettingsStore};

use crate::notifications::reset_detection_cache_quiet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheClearResult {
    pub logs_deleted: usize,
    pub detection_reset: bool,
    pub nams_settings_deleted: bool,
    pub stamp_deleted: bool,
}

fn clear_logs_quiet(logs_dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(logs_dir) else {
        return 0;
    };
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter(|entry| std::fs::remove_file(entry.path()).is_ok())
        .count()
}

fn delete_file_quiet(file: &Path) -> bool {
    file.is_file() && std::fs::remove_file(file).is_ok()
}

pub fn clear_all(
    paths: &AppPaths,
    settings: &SettingsStore,
    platform: &dyn PlatformAdapter,
    game: Option<&GamePaths>,
) -> CacheClearResult {
    let game_dir = game
        .map(GamePaths::root)
        .filter(|root| !root.as_os_str().is_empty());
    CacheClearResult {
        logs_deleted: clear_logs_quiet(&paths.logs_dir),
        detection_reset: reset_detection_cache_quiet(settings),
        nams_settings_deleted: platform
            .resolve_nams_settings_path(game_dir)
            .is_some_and(|file| delete_file_quiet(&file)),
        stamp_deleted: delete_file_quiet(&paths.runtime_stamp()),
    }
}
