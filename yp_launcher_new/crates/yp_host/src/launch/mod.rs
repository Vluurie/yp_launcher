pub mod command;
pub mod failure;
pub mod record;
pub mod steam;
pub mod wrapper;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use yp_core::consts::GAME_EXE_NAME;
use yp_core::runtime::RuntimeSetup;
use yp_core::{AppPaths, ChildProcess, LaunchUnavailable, NamsCommandRequest, NamsInvocation, Os, ProcessRunner, SettingsStore};

use crate::gpu::{self, GpuProbe};
use crate::platform::HostPlatform;
use command::{format_script, LaunchCommand};
use failure::{FailureKind, LaunchFailure};
use record::{LaunchRecord, TAIL_CHARS};

pub const GRACE_PERIOD: Duration = Duration::from_secs(10);
pub const GRACE_POLL: Duration = Duration::from_millis(100);
pub const CAPTURED_LOG_PREFIX: &str = "nams_";
const CAPTURE_CHARS: usize = 64 * 1024;
const CAPTURED_LOG_RULE_WIDTH: usize = 60;

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchOutcome {
    Started { record: LaunchRecord },
    Failed(LaunchFailure),
}

impl LaunchOutcome {
    pub fn started(&self) -> bool {
        matches!(self, LaunchOutcome::Started { .. })
    }

    pub fn failure(&self) -> Option<&LaunchFailure> {
        match self {
            LaunchOutcome::Failed(failure) => Some(failure),
            LaunchOutcome::Started { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLaunch {
    pub command: LaunchCommand,
    pub prefer_gpu: bool,
}

pub struct Launcher {
    platform: Arc<dyn HostPlatform>,
    runner: Arc<dyn ProcessRunner>,
    paths: AppPaths,
    settings: Arc<SettingsStore>,
    grace: Duration,
    gpu_probe: Option<GpuProbe>,
    write_gpu_registry: bool,
}

impl Launcher {
    pub fn new(
        platform: Arc<dyn HostPlatform>,
        runner: Arc<dyn ProcessRunner>,
        paths: AppPaths,
        settings: Arc<SettingsStore>,
    ) -> Self {
        let write_gpu_registry = platform.host().os() == Os::Windows;
        Self {
            platform,
            runner,
            paths,
            settings,
            grace: GRACE_PERIOD,
            gpu_probe: None,
            write_gpu_registry,
        }
    }

    pub fn with_grace(mut self, grace: Duration) -> Self {
        self.grace = grace;
        self
    }

    pub fn with_gpu_probe(mut self, probe: GpuProbe) -> Self {
        self.gpu_probe = Some(probe);
        self
    }

    pub fn with_gpu_registry(mut self, enabled: bool) -> Self {
        self.write_gpu_registry = enabled;
        self
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn platform(&self) -> &Arc<dyn HostPlatform> {
        &self.platform
    }

    fn os(&self) -> Os {
        self.platform.host().os()
    }

    pub fn resolve(&self, game_dir: &Path, vanilla: bool) -> Result<ResolvedLaunch, LaunchUnavailable> {
        let nams_exe = self.paths.nams_exe();
        let request = NamsCommandRequest {
            nams_exe: &nams_exe,
            launcher_dir: &self.paths.runtime_dir,
            game_dir,
            invocation: NamsInvocation::Run { vanilla },
        };
        let built = self.platform.build_launch_command(&request)?;
        let prefer_gpu = gpu::prefer_dedicated(&self.settings);
        let probe = self.gpu_probe.unwrap_or_else(|| GpuProbe::detect(self.platform.host()));
        let mut command = LaunchCommand {
            env: gpu::merged_launch_env(built.env.clone(), prefer_gpu, self.os(), &probe),
            ..built
        };
        if self.os() == Os::Linux {
            command = wrapper::wrap(self.os(), command, &wrapper::read(&self.settings));
        }
        Ok(ResolvedLaunch { command, prefer_gpu })
    }

    pub fn preview(&self, game_dir: &Path) -> Option<String> {
        let resolved = self.resolve(game_dir, false).ok()?;
        Some(format_script(&resolved.command, self.os() == Os::Windows))
    }

    pub fn is_running(&self) -> bool {
        self.platform.is_game_running()
    }

    pub fn terminate(&self) -> bool {
        self.platform.terminate_game()
    }

    fn preflight(&self, game_dir: &Path) -> Option<LaunchFailure> {
        let missing = RuntimeSetup::new(&self.paths, self.platform.host()).missing_files();
        if !missing.is_empty() {
            return Some(LaunchFailure::new(
                FailureKind::FilesMissing {
                    missing,
                    launcher_dir: self.paths.runtime_dir.clone(),
                },
                String::new(),
            ));
        }
        if !yp_core::fs::is_dir_writable(&self.paths.runtime_dir) {
            return Some(LaunchFailure::new(
                FailureKind::LauncherDirNotWritable {
                    dir: self.paths.runtime_dir.clone(),
                },
                String::new(),
            ));
        }
        let exe = game_dir.join(GAME_EXE_NAME);
        if !exe.is_file() {
            return Some(LaunchFailure::new(
                FailureKind::ExeNotFound {
                    game_dir: game_dir.to_path_buf(),
                    exe,
                },
                String::new(),
            ));
        }
        let nams_dir = game_dir.join("nams");
        if !yp_core::fs::is_dir_writable(&nams_dir) {
            return Some(LaunchFailure::new(
                FailureKind::GameDirNotWritable {
                    game_dir: game_dir.to_path_buf(),
                    nams_dir,
                },
                String::new(),
            ));
        }
        None
    }

    pub fn start(
        &self,
        game_dir: &Path,
        vanilla: bool,
        on_stopped: impl FnOnce(LaunchRecord) + Send + 'static,
    ) -> LaunchOutcome {
        if let Some(failure) = self.preflight(game_dir) {
            return LaunchOutcome::Failed(failure);
        }
        crate::nams_log::clear(&self.paths);
        let resolved = match self.resolve(game_dir, vanilla) {
            Ok(resolved) => resolved,
            Err(unavailable) => {
                return LaunchOutcome::Failed(LaunchFailure::new(FailureKind::Unavailable(unavailable), String::new()));
            }
        };
        if self.write_gpu_registry {
            gpu::apply(&self.paths.nams_exe(), resolved.prefer_gpu);
        }
        let mut child = match self.runner.spawn(&resolved.command.to_process_spec()) {
            Ok(child) => child,
            Err(err) => {
                let detail = err.to_string();
                return LaunchOutcome::Failed(LaunchFailure::new(FailureKind::Internal { detail: detail.clone() }, detail));
            }
        };
        let record = LaunchRecord::started(resolved.command.label.clone());
        record::write_quiet(&self.paths, &record);

        match wait_for_exit(child.as_mut(), self.grace) {
            None => {
                self.monitor(child, record.clone(), on_stopped);
                LaunchOutcome::Started { record }
            }
            Some(code) => LaunchOutcome::Failed(self.exited_early(child.as_ref(), &record, code)),
        }
    }

    fn monitor(
        &self,
        mut child: Box<dyn ChildProcess>,
        record: LaunchRecord,
        on_stopped: impl FnOnce(LaunchRecord) + Send + 'static,
    ) {
        let paths = self.paths.clone();
        let spawned = std::thread::Builder::new().name("yp-nams-monitor".to_owned()).spawn(move || {
            let code = child.wait_exit_code().unwrap_or(-1);
            let finished = record.finished(code, child.output_tail(TAIL_CHARS));
            record::write_quiet(&paths, &finished);
            on_stopped(finished);
        });
        if let Err(err) = spawned {
            log::warn!("could not start the NAMS monitor: {err}");
        }
    }

    fn exited_early(&self, child: &dyn ChildProcess, record: &LaunchRecord, code: i32) -> LaunchFailure {
        let combined = child.output_tail(CAPTURE_CHARS);
        record::write_quiet(&self.paths, &record.finished(code, record::tail_of(&combined)));
        let captured_log = write_captured_log(&self.paths.logs_dir, &combined, Some(code));
        if let Some(parsed) = failure::parse(&combined, captured_log.clone()) {
            return parsed;
        }
        LaunchFailure {
            kind: FailureKind::Exited { code },
            raw_output: combined,
            captured_log,
        }
    }
}

pub fn wait_for_exit(child: &mut dyn ChildProcess, grace: Duration) -> Option<i32> {
    let deadline = Instant::now() + grace;
    loop {
        match child.try_exit_code() {
            Ok(Some(code)) => return Some(code),
            Ok(None) => {}
            Err(err) => {
                log::warn!("could not poll NAMS: {err}");
                return None;
            }
        }
        let now = Instant::now();
        if now >= deadline {
            return None;
        }
        std::thread::sleep(GRACE_POLL.min(deadline - now));
    }
}

pub fn write_captured_log(logs_dir: &Path, content: &str, exit_code: Option<i32>) -> Option<PathBuf> {
    if content.trim().is_empty() {
        return None;
    }
    let now = jiff::Timestamp::now();
    let file = logs_dir.join(format!(
        "{CAPTURED_LOG_PREFIX}{}.log",
        now.strftime("%Y-%m-%dT%H-%M-%S-%6f")
    ));
    let exit = exit_code.map_or_else(|| "<not exited>".to_owned(), |code| code.to_string());
    let text = format!(
        "NAMS captured output\nGenerated: {}\nExit code: {exit}\n{}\n\n{content}",
        record::format_timestamp(now),
        "=".repeat(CAPTURED_LOG_RULE_WIDTH)
    );
    match yp_core::fs::write_text_atomic(&file, &text) {
        Ok(()) => Some(file),
        Err(err) => {
            log::warn!("could not write captured NAMS output: {err}");
            None
        }
    }
}

pub fn preview(launcher: &Launcher, game_dir: &Path) -> Option<String> {
    launcher.preview(game_dir)
}
