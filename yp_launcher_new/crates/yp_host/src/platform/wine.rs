use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use yp_core::consts::{GAME_EXE_NAME, NAMS_EXE_NAME};
use yp_core::wine_paths::{infer_wine_prefix, posix, to_wine_path};
use yp_core::{
    CancelToken, HostEnv, LaunchUnavailable, NamsCommandRequest, NierInstallation, Os, PlatformAdapter, ProcessRunner,
    ProcessSpec, Result, SelectionRejection,
};

use super::{merge_installations, nams_settings_path, posix_text, HostPlatform, VerifyTimeoutHint};
use crate::launch::command::{
    build_crossover, build_plain_wine, build_proton, CrossOverLaunch, LaunchCommand, ProtonLaunch,
};
use crate::wine::native_steam::find_nier_in_native_steam;
use crate::wine::prefix_steam::find_nier_in_prefixes;
use crate::wine::runtime::{self, WineRuntime, WineSource};
use crate::wine::{is_dir, WineContext};

const PROCESS_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WineAdapter {
    ctx: WineContext,
    runner: Arc<dyn ProcessRunner>,
}

impl WineAdapter {
    pub fn new(ctx: WineContext, runner: Arc<dyn ProcessRunner>) -> Self {
        Self { ctx, runner }
    }

    pub fn context(&self) -> &WineContext {
        &self.ctx
    }

    fn require_host_drive_mapping(&self, runtime: &WineRuntime, nams_exe: &str) -> Result<(), LaunchUnavailable> {
        let Some(prefix) = runtime.prefix.as_deref() else {
            return Ok(());
        };
        if !to_wine_path(nams_exe).starts_with("Z:") || !is_dir(prefix) {
            return Ok(());
        }
        if is_dir(&posix::join_all(prefix, &["dosdevices", "z:"])) {
            return Ok(());
        }
        Err(LaunchUnavailable::NoZDrive {
            prefix: PathBuf::from(prefix),
        })
    }

    fn query(&self, program: &str) -> bool {
        let spec = ProcessSpec::new(program).args(["-f", NAMS_EXE_NAME]);
        self.runner
            .output(&spec, Some(PROCESS_QUERY_TIMEOUT))
            .map(|output| output.is_success())
            .unwrap_or(false)
    }
}

impl PlatformAdapter for WineAdapter {
    fn host(&self) -> &HostEnv {
        self.ctx.host()
    }

    fn can_launch_game(&self) -> bool {
        runtime::is_available(&self.ctx)
    }

    fn is_experimental(&self) -> bool {
        self.ctx.os() == Os::Linux
    }

    fn uses_native_title_bar(&self) -> bool {
        true
    }

    fn supports_desktop_shortcut(&self) -> bool {
        false
    }

    fn build_nams_command(&self, request: &NamsCommandRequest<'_>) -> Result<ProcessSpec, LaunchUnavailable> {
        self.build_launch_command(request).map(|command| command.to_process_spec())
    }

    fn resolve_nams_settings_path(&self, game_dir: Option<&Path>) -> Option<PathBuf> {
        nams_settings_path(game_dir)
    }

    fn is_game_running(&self) -> bool {
        self.query("pgrep")
    }

    fn terminate_game(&self) -> bool {
        self.query("pkill")
    }

    fn discover_fast(&self) -> Vec<NierInstallation> {
        let prefixes = find_nier_in_prefixes(&self.ctx);
        if self.ctx.os() == Os::Linux {
            return merge_installations(vec![find_nier_in_native_steam(&self.ctx), prefixes]);
        }
        prefixes
    }

    fn discover_deep(&self, _cancel: &CancelToken) -> Vec<NierInstallation> {
        self.discover_fast()
    }

    fn reject_game_selection(&self, exe: &Path) -> Option<SelectionRejection> {
        let picked = posix_text(exe);
        if infer_wine_prefix(&picked).is_some() || self.ctx.infer_steam_context(&picked).is_some() {
            return None;
        }
        Some(SelectionRejection::OutsidePrefix { exe: exe.to_path_buf() })
    }

    fn reveal(&self, target: &Path, is_file: bool) -> Result<()> {
        crate::reveal::reveal(self.runner.as_ref(), self.ctx.os(), target, is_file)
    }
}

impl HostPlatform for WineAdapter {
    fn build_launch_command(&self, request: &NamsCommandRequest<'_>) -> Result<LaunchCommand, LaunchUnavailable> {
        let nams_exe = posix_text(request.nams_exe);
        let game_dir = posix_text(request.game_dir);
        let launcher_dir = posix_text(request.launcher_dir);
        let game_exe = posix::join(&game_dir, GAME_EXE_NAME);
        let Some(runtime) = runtime::detect(&self.ctx, Some(&game_exe)) else {
            return Err(LaunchUnavailable::NoCompatLayer { os: self.ctx.os() });
        };
        self.require_host_drive_mapping(&runtime, &nams_exe)?;
        match runtime.source {
            WineSource::CrossOver => {
                let bottle = runtime.bottle.clone().ok_or(LaunchUnavailable::NoCompatLayer { os: self.ctx.os() })?;
                let prefix = runtime.prefix.clone().unwrap_or_else(|| bottle.path.clone());
                Ok(build_crossover(
                    &self.ctx,
                    &CrossOverLaunch {
                        nams_exe: &nams_exe,
                        game_dir: &game_dir,
                        launcher_dir: &launcher_dir,
                        wine_binary: &runtime.binary,
                        bottle: &bottle,
                        prefix: &prefix,
                    },
                    request.invocation,
                ))
            }
            WineSource::Proton => build_proton(
                &self.ctx,
                &ProtonLaunch {
                    nams_exe: &nams_exe,
                    game_dir: &game_dir,
                    game_exe: &game_exe,
                    launcher_dir: &launcher_dir,
                    proton_path: &runtime.binary,
                },
                request.invocation,
            )
            .ok_or_else(|| LaunchUnavailable::ProtonMissing {
                proton: PathBuf::from(&runtime.binary),
            }),
            WineSource::Wine => Ok(build_plain_wine(
                &self.ctx,
                &nams_exe,
                &game_dir,
                &launcher_dir,
                &runtime.binary,
                runtime.prefix.as_deref(),
                request.invocation,
            )),
        }
    }

    fn verify_timeout_hint(&self) -> VerifyTimeoutHint {
        match self.ctx.os() {
            Os::Linux => VerifyTimeoutHint::Linux,
            Os::Windows | Os::MacOs => VerifyTimeoutHint::Generic,
        }
    }
}
