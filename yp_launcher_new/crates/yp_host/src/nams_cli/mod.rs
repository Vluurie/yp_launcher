pub mod verify;

use std::path::Path;
use std::time::Duration;

use serde_json::Value;
use yp_core::consts::ARG_NIER_PATH;
use yp_core::{AppPaths, NamsCommandRequest, NamsInvocation, ProcessRunner, ProcessSpec};

use crate::platform::HostPlatform;
pub use verify::{VerifyCheck, VerifyOutcome, VerifyResult, VerifyStatus};

pub const VERIFY_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_TEXTURE_SOURCE: &str = "standalone";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamsTexturePack {
    pub name: String,
    pub source: String,
    pub mod_id: Option<String>,
    pub character: Option<String>,
    pub outfit_conditional: bool,
    pub enabled: bool,
    pub dds_count: i64,
    pub load_order_index: Option<i64>,
    pub path: String,
}

fn text(json: &Value, key: &str) -> Option<String> {
    json.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn integer(json: &Value, key: &str) -> Option<i64> {
    let value = json.get(key)?;
    value.as_i64().or_else(|| value.as_f64().map(|number| number as i64))
}

impl NamsTexturePack {
    pub fn from_json(json: &Value) -> Self {
        Self {
            name: text(json, "name").unwrap_or_default(),
            source: text(json, "source").unwrap_or_else(|| DEFAULT_TEXTURE_SOURCE.to_owned()),
            mod_id: text(json, "mod"),
            character: text(json, "character"),
            outfit_conditional: json.get("outfit_conditional").and_then(Value::as_bool).unwrap_or(false),
            enabled: json.get("enabled").and_then(Value::as_bool).unwrap_or(true),
            dds_count: integer(json, "dds_count").unwrap_or(0),
            load_order_index: integer(json, "load_order_index"),
            path: text(json, "path").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamsPlugin {
    pub name: String,
    pub size: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamsInfo {
    pub version: Option<String>,
    pub nier_install_error: Option<String>,
    pub game_bin_present: bool,
    pub game_bin_size: Option<i64>,
    pub plugins: Vec<NamsPlugin>,
}

impl NamsInfo {
    pub fn from_json(json: &Value) -> Self {
        let game_bin = json
            .get("runtime_files")
            .filter(|runtime| runtime.is_object())
            .and_then(|runtime| runtime.get("game_bin"))
            .filter(|bin| bin.is_object());
        Self {
            version: text(json, "version"),
            nier_install_error: json
                .get("paths")
                .filter(|paths| paths.is_object())
                .and_then(|paths| text(paths, "nier_install_error")),
            game_bin_present: game_bin.and_then(|bin| bin.get("present")).and_then(Value::as_bool) == Some(true),
            game_bin_size: game_bin.and_then(|bin| integer(bin, "size")),
            plugins: json
                .get("plugins")
                .and_then(Value::as_array)
                .map(|plugins| {
                    plugins
                        .iter()
                        .filter(|plugin| plugin.is_object())
                        .map(|plugin| NamsPlugin {
                            name: match plugin.get("name") {
                                None | Some(Value::Null) => "?".to_owned(),
                                Some(Value::String(name)) => name.clone(),
                                Some(other) => other.to_string(),
                            },
                            size: integer(plugin, "size").unwrap_or(0),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

fn run_for_stdout(
    runner: &dyn ProcessRunner,
    paths: &AppPaths,
    args: &[&str],
    game_dir: &Path,
    timeout: Option<Duration>,
) -> Option<String> {
    let exe = paths.nams_exe();
    if !exe.is_file() {
        return None;
    }
    let spec = ProcessSpec::new(exe)
        .args(args.iter().copied())
        .args([ARG_NIER_PATH])
        .arg(game_dir.as_os_str())
        .arg("--json")
        .cwd(&paths.runtime_dir)
        .hide_console();
    let output = runner.output(&spec, timeout).ok()?;
    output.is_success().then_some(output.stdout)
}

pub fn textures_list(
    runner: &dyn ProcessRunner,
    paths: &AppPaths,
    game_dir: &Path,
    timeout: Option<Duration>,
) -> Option<Vec<NamsTexturePack>> {
    let stdout = run_for_stdout(runner, paths, &["textures", "list"], game_dir, timeout)?;
    let decoded: Value = serde_json::from_str(&stdout).ok()?;
    let packs = decoded.as_object()?.get("packs")?.as_array()?;
    Some(
        packs
            .iter()
            .filter(|pack| pack.is_object())
            .map(NamsTexturePack::from_json)
            .collect(),
    )
}

pub fn info(runner: &dyn ProcessRunner, paths: &AppPaths, game_dir: &Path, timeout: Option<Duration>) -> Option<NamsInfo> {
    let stdout = run_for_stdout(runner, paths, &["info"], game_dir, timeout)?;
    let decoded: Value = serde_json::from_str(&stdout).ok()?;
    decoded.is_object().then(|| NamsInfo::from_json(&decoded))
}

pub fn verify(platform: &dyn HostPlatform, runner: &dyn ProcessRunner, paths: &AppPaths, game_dir: &Path) -> VerifyOutcome {
    let nams_exe = paths.nams_exe();
    if !nams_exe.is_file() {
        return VerifyOutcome::status(VerifyStatus::Error);
    }
    let request = NamsCommandRequest {
        nams_exe: &nams_exe,
        launcher_dir: &paths.runtime_dir,
        game_dir,
        invocation: NamsInvocation::Verify,
    };
    let Ok(command) = platform.build_launch_command(&request) else {
        return VerifyOutcome::status(VerifyStatus::NoRuntime);
    };
    match runner.output(&command.to_process_spec(), Some(VERIFY_TIMEOUT)) {
        Err(err) => VerifyOutcome::status(VerifyStatus::Error).with_raw(err.to_string()),
        Ok(output) if output.timed_out => VerifyOutcome::status(VerifyStatus::TimedOut),
        Ok(output) => verify::outcome_from_output(output.exit_code, &output.stdout, &output.stderr),
    }
}
