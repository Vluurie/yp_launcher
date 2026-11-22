use std::path::{Path, PathBuf};
use std::sync::Arc;

use yp_core::{
    CancelToken, HostEnv, LaunchUnavailable, NamsCommandRequest, NierInstallation, Os, PlatformAdapter, ProcessRunner,
    ProcessSpec, Result, SelectionRejection,
};

use super::{nams_settings_path, path_text, HostPlatform};
use crate::launch::command::{build_native, LaunchCommand};

pub struct WindowsAdapter {
    host: HostEnv,
    runner: Arc<dyn ProcessRunner>,
}

impl WindowsAdapter {
    pub fn new(host: HostEnv, runner: Arc<dyn ProcessRunner>) -> Self {
        Self { host, runner }
    }
}

#[cfg(windows)]
fn game_process_ids() -> Vec<u32> {
    yp_core::process::excluding_helpers(|helpers| {
        [yp_core::consts::NAMS_EXE_NAME, yp_core::consts::GAME_EXE_NAME]
            .into_iter()
            .flat_map(super::win32::process_ids_named)
            .filter(|id| !helpers.contains(id))
            .collect()
    })
}

#[cfg(windows)]
fn nams_running() -> bool {
    !game_process_ids().is_empty()
}

#[cfg(not(windows))]
fn nams_running() -> bool {
    false
}

#[cfg(windows)]
fn terminate_nams() -> bool {
    super::win32::terminate_processes(&game_process_ids())
}

#[cfg(not(windows))]
fn terminate_nams() -> bool {
    false
}

impl PlatformAdapter for WindowsAdapter {
    fn host(&self) -> &HostEnv {
        &self.host
    }

    fn can_launch_game(&self) -> bool {
        true
    }

    fn is_experimental(&self) -> bool {
        false
    }

    fn uses_native_title_bar(&self) -> bool {
        false
    }

    fn supports_desktop_shortcut(&self) -> bool {
        true
    }

    fn build_nams_command(&self, request: &NamsCommandRequest<'_>) -> Result<ProcessSpec, LaunchUnavailable> {
        self.build_launch_command(request)
            .map(|command| command.to_process_spec())
    }

    fn resolve_nams_settings_path(&self, game_dir: Option<&Path>) -> Option<PathBuf> {
        nams_settings_path(game_dir)
    }

    fn is_game_running(&self) -> bool {
        nams_running()
    }

    fn terminate_game(&self) -> bool {
        terminate_nams()
    }

    fn discover_fast(&self) -> Vec<NierInstallation> {
        crate::discovery::installs_from_windows(&self.host, &crate::discovery::registry_hints())
    }

    fn discover_deep(&self, cancel: &CancelToken) -> Vec<NierInstallation> {
        crate::discovery::drive_scan(cancel)
    }

    fn reject_game_selection(&self, _exe: &Path) -> Option<SelectionRejection> {
        None
    }

    fn reveal(&self, target: &Path, is_file: bool) -> Result<()> {
        crate::reveal::reveal(self.runner.as_ref(), Os::Windows, target, is_file)
    }
}

impl HostPlatform for WindowsAdapter {
    fn build_launch_command(&self, request: &NamsCommandRequest<'_>) -> Result<LaunchCommand, LaunchUnavailable> {
        Ok(build_native(
            &path_text(request.nams_exe),
            &path_text(request.game_dir),
            &path_text(request.launcher_dir),
            request.invocation,
        ))
    }
}
