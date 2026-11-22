use std::path::{Path, PathBuf};

use yp_core::{SettingKey, SettingsStore};

use super::failure::{FailureKind, LaunchFailure};

pub const NAMS_DLL_PATH: SettingKey<String> = SettingKey::new("nams_dll_path", String::new);
pub const PROXY_NAME: &str = "dinput8.dll";

pub fn picked(settings: &SettingsStore) -> Option<PathBuf> {
    let path = settings.get(&NAMS_DLL_PATH);
    let path = path.trim();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

pub fn is_nams_dll(path: &Path) -> bool {
    std::fs::read(path).is_ok_and(|bytes| yp_core::consts::is_nams_dll(&bytes))
}

pub fn deploy(game_dir: &Path, dll: &Path) -> Result<(), LaunchFailure> {
    if !dll.is_file() {
        return Err(LaunchFailure::new(
            FailureKind::NamsDllMissing { dll: dll.to_path_buf() },
            String::new(),
        ));
    }
    let target = game_dir.join(PROXY_NAME);
    let same_file = match (std::fs::canonicalize(dll), std::fs::canonicalize(&target)) {
        (Ok(dll), Ok(target)) => dll == target,
        _ => false,
    };
    if same_file {
        return Ok(());
    }
    std::fs::copy(dll, &target).map(|_| ()).map_err(|error| {
        LaunchFailure::new(
            FailureKind::NamsDllCopyFailed {
                dll: dll.to_path_buf(),
                target,
                detail: error.to_string(),
            },
            String::new(),
        )
    })
}

pub fn logs_dir_for(paths: &yp_core::AppPaths, dll_mode: bool, game_dir: Option<&Path>) -> PathBuf {
    match game_dir.filter(|_| dll_mode) {
        Some(game_dir) => yp_core::GamePaths::new(game_dir).nams_dir().join(yp_core::consts::LOGS_DIR_NAME),
        None => paths.logs_dir.clone(),
    }
}

pub fn logs_dir(paths: &yp_core::AppPaths, settings: &SettingsStore, game_dir: Option<&Path>) -> PathBuf {
    logs_dir_for(paths, picked(settings).is_some(), game_dir)
}

pub fn log_file(paths: &yp_core::AppPaths, settings: &SettingsStore, game_dir: Option<&Path>) -> PathBuf {
    logs_dir(paths, settings, game_dir).join(yp_core::consts::NAMS_LOG_NAME)
}
