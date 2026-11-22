use std::sync::Arc;
use std::time::Duration;

use yp_config::cli::{CliOutput, NamsCli};
use yp_config::{ConfigError, Result};
use yp_core::{AppPaths, GamePaths, NamsCommandRequest, NamsInvocation, PlatformAdapter, ProcessRunner};

pub const CONFIG_TIMEOUT: Duration = Duration::from_secs(60);

pub struct HostNamsCli {
    pub platform: Arc<dyn PlatformAdapter>,
    pub runner: Arc<dyn ProcessRunner>,
    pub paths: AppPaths,
    pub game: GamePaths,
}

impl NamsCli for HostNamsCli {
    fn run(&self, args: &[String], stdin: Option<&[u8]>) -> Result<CliOutput> {
        let nams_exe = self.paths.nams_exe();
        if !nams_exe.is_file() {
            return Err(ConfigError::Unavailable(nams_exe.to_string_lossy().into_owned()));
        }
        let request = NamsCommandRequest {
            nams_exe: &nams_exe,
            launcher_dir: &self.paths.runtime_dir,
            game_dir: self.game.root(),
            invocation: NamsInvocation::Config(args),
        };
        let mut spec = self
            .platform
            .build_nams_command(&request)
            .map_err(|reason| ConfigError::Unavailable(format!("{reason:?}")))?;
        if let Some(input) = stdin {
            spec = spec.stdin(input.to_vec());
        }
        let output = self
            .runner
            .output(&spec, Some(CONFIG_TIMEOUT))
            .map_err(|error| ConfigError::Unavailable(error.to_string()))?;
        Ok(CliOutput {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            timed_out: output.timed_out,
        })
    }
}
