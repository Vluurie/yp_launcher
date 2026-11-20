pub mod linux;
pub mod macos;
pub mod windows;
pub mod wine;

#[cfg(windows)]
#[allow(unsafe_code)]
pub mod win32;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use yp_core::wine_paths::to_posix;
use yp_core::{HostEnv, LaunchUnavailable, NamsCommandRequest, NierInstallation, Os, PlatformAdapter, ProcessRunner};

use crate::launch::command::LaunchCommand;

pub const NAMS_SETTINGS_SEGMENTS: [&str; 4] = ["nams", "_internal", "cache", "settings.json"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerifyTimeoutHint {
    Generic,
    Linux,
}

pub trait HostPlatform: PlatformAdapter {
    fn build_launch_command(&self, request: &NamsCommandRequest<'_>) -> Result<LaunchCommand, LaunchUnavailable>;
    fn verify_timeout_hint(&self) -> VerifyTimeoutHint {
        VerifyTimeoutHint::Generic
    }
}

pub fn adapter_for(host: HostEnv, runner: Arc<dyn ProcessRunner>) -> Arc<dyn HostPlatform> {
    match host.os() {
        Os::Windows => Arc::new(windows::WindowsAdapter::new(host, runner)),
        Os::Linux => Arc::new(linux::linux_adapter(host, runner)),
        Os::MacOs => Arc::new(macos::macos_adapter(host, runner)),
    }
}

#[cfg(windows)]
pub fn oem_code_page() -> Option<u32> {
    Some(win32::oem_code_page())
}

#[cfg(not(windows))]
pub fn oem_code_page() -> Option<u32> {
    None
}

pub fn nams_settings_path(game_dir: Option<&Path>) -> Option<PathBuf> {
    let game_dir = game_dir.filter(|dir| !dir.as_os_str().is_empty())?;
    Some(NAMS_SETTINGS_SEGMENTS.iter().fold(game_dir.to_path_buf(), |path, part| path.join(part)))
}

pub(crate) fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub(crate) fn posix_text(path: &Path) -> String {
    to_posix(path)
}

pub fn merge_installations(groups: Vec<Vec<NierInstallation>>) -> Vec<NierInstallation> {
    let mut seen = Vec::new();
    let mut merged = Vec::new();
    for install in groups.into_iter().flatten() {
        let key = yp_core::wine_paths::posix::normalize(&posix_text(&install.path)).to_lowercase();
        if !seen.contains(&key) {
            seen.push(key);
            merged.push(install);
        }
    }
    merged
}
