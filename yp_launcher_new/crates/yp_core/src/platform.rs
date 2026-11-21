use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::host::{HostEnv, Os};
use crate::process::ProcessSpec;
use crate::task::CancelToken;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NierInstallation {
    pub path: PathBuf,
    pub has_data: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamsInvocation<'a> {
    Run { vanilla: bool },
    Game { vanilla: bool, console: bool },
    Verify,
    Config(&'a [String]),
}

#[derive(Debug, Clone, Copy)]
pub struct NamsCommandRequest<'a> {
    pub nams_exe: &'a Path,
    pub launcher_dir: &'a Path,
    pub game_dir: &'a Path,
    pub invocation: NamsInvocation<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchUnavailable {
    NoCompatLayer { os: Os },
    ProtonMissing { proton: PathBuf },
    NoZDrive { prefix: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionRejection {
    OutsidePrefix { exe: PathBuf },
}

pub trait PlatformAdapter: Send + Sync {
    fn host(&self) -> &HostEnv;
    fn can_launch_game(&self) -> bool;
    fn is_experimental(&self) -> bool;
    fn uses_native_title_bar(&self) -> bool;
    fn supports_desktop_shortcut(&self) -> bool;
    fn build_nams_command(
        &self,
        request: &NamsCommandRequest<'_>,
    ) -> std::result::Result<ProcessSpec, LaunchUnavailable>;
    fn resolve_nams_settings_path(&self, game_dir: Option<&Path>) -> Option<PathBuf>;
    fn is_game_running(&self) -> bool;
    fn terminate_game(&self) -> bool;
    fn discover_fast(&self) -> Vec<NierInstallation>;
    fn discover_deep(&self, cancel: &CancelToken) -> Vec<NierInstallation>;
    fn reject_game_selection(&self, exe: &Path) -> Option<SelectionRejection>;
    fn reveal(&self, target: &Path, is_file: bool) -> Result<()>;
}
