use std::path::{Path, PathBuf};

use yp_core::consts::{ARG_NIER_PATH, ARG_RUN, GAME_EXE_NAME, SHORTCUT_DESCRIPTION, SHORTCUT_NAME};
use yp_core::AppPaths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutSpec {
    pub shortcut_path: PathBuf,
    pub target: PathBuf,
    pub arguments: String,
    pub working_dir: PathBuf,
    pub icon: PathBuf,
    pub description: String,
}

pub fn shortcut_file_name() -> String {
    format!("{SHORTCUT_NAME}.lnk")
}

pub fn desktop_shortcut_spec(paths: &AppPaths, desktop: &Path, game_dir: &Path) -> ShortcutSpec {
    ShortcutSpec {
        shortcut_path: desktop.join(shortcut_file_name()),
        target: paths.nams_exe(),
        arguments: format!("{ARG_RUN} {ARG_NIER_PATH} \"{}\"", game_dir.display()),
        working_dir: paths.runtime_dir.clone(),
        icon: game_dir.join(GAME_EXE_NAME),
        description: SHORTCUT_DESCRIPTION.to_owned(),
    }
}

pub fn desktop_dir() -> Option<PathBuf> {
    dirs::desktop_dir().filter(|dir| dir.is_dir())
}

#[cfg(windows)]
pub fn create(spec: &ShortcutSpec) -> bool {
    match crate::platform::win32::create_shortcut(spec) {
        Ok(()) => spec.shortcut_path.is_file(),
        Err(err) => {
            log::warn!("could not create shortcut {}: {err}", spec.shortcut_path.display());
            false
        }
    }
}

#[cfg(not(windows))]
pub fn create(_spec: &ShortcutSpec) -> bool {
    false
}

pub fn create_desktop_shortcut(paths: &AppPaths, game_dir: &Path) -> bool {
    match desktop_dir() {
        Some(desktop) => create(&desktop_shortcut_spec(paths, &desktop, game_dir)),
        None => false,
    }
}
