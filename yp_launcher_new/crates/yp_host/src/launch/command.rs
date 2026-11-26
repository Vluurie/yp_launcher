use std::ffi::OsString;
use std::path::PathBuf;

use indexmap::IndexMap;
use yp_core::consts::{ARG_CONFIG, ARG_CONSOLE, ARG_JSON, ARG_NIER_PATH, ARG_RUN, ARG_VANILLA, ARG_VERIFY};
use yp_core::wine_paths::{to_wine_path, wine_env, NIER_STEAM_APP_ID};
use yp_core::{NamsInvocation, ProcessSpec};

use crate::wine::crossover::{CrossOverBottle, BOTTLE_VAR};
use crate::wine::proton::{compat_data_path, steam_home_for_client, steam_root, CLIENT_INSTALL_VAR, COMPAT_DATA_VAR};
use crate::wine::{is_file, WineContext};

pub const NATIVE_LABEL: &str = "Windows";
pub const PROTON_LABEL: &str = "Proton";
pub const WINE_LABEL: &str = "Wine";

pub type LaunchEnv = IndexMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCommand {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub env: Option<LaunchEnv>,
    pub label: String,
}

impl LaunchCommand {
    pub fn display(&self) -> String {
        std::iter::once(self.command.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn to_process_spec(&self) -> ProcessSpec {
        ProcessSpec {
            program: PathBuf::from(&self.command),
            args: self.args.iter().map(OsString::from).collect(),
            cwd: Some(PathBuf::from(&self.cwd)),
            env: self
                .env
                .iter()
                .flatten()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            label: self.label.clone(),
            hide_console: true,
            stdin: None,
        }
    }
}

pub fn nams_run_args(nier_path: &str, vanilla: bool) -> Vec<String> {
    let mut args = vec![ARG_RUN.to_owned(), ARG_NIER_PATH.to_owned(), nier_path.to_owned()];
    if vanilla {
        args.push(ARG_VANILLA.to_owned());
    }
    args
}

pub fn game_args(vanilla: bool, console: bool) -> Vec<String> {
    let mut args = Vec::new();
    if vanilla {
        args.push(ARG_VANILLA.to_owned());
    }
    if console {
        args.push(ARG_CONSOLE.to_owned());
    }
    args
}

pub fn nams_verify_args(nier_path: &str) -> Vec<String> {
    vec![
        ARG_VERIFY.to_owned(),
        ARG_NIER_PATH.to_owned(),
        nier_path.to_owned(),
        ARG_JSON.to_owned(),
    ]
}

pub fn nams_config_args(args: &[String], nier_path: &str) -> Vec<String> {
    let mut out = vec![ARG_CONFIG.to_owned()];
    let mut rest = args.iter();
    if let Some(sub) = rest.next() {
        out.push(sub.clone());
    }
    out.push(ARG_NIER_PATH.to_owned());
    out.push(nier_path.to_owned());
    out.extend(rest.cloned());
    out
}

pub fn invocation_args(invocation: NamsInvocation<'_>, nier_path: &str) -> Vec<String> {
    match invocation {
        NamsInvocation::Run { vanilla } => nams_run_args(nier_path, vanilla),
        NamsInvocation::Game { vanilla, console } => game_args(vanilla, console),
        NamsInvocation::Verify => nams_verify_args(nier_path),
        NamsInvocation::Config(args) => nams_config_args(args, nier_path),
    }
}

pub fn build_native(nams_exe: &str, game_dir: &str, launcher_dir: &str, invocation: NamsInvocation<'_>) -> LaunchCommand {
    LaunchCommand {
        command: nams_exe.to_owned(),
        args: invocation_args(invocation, &game_dir.replace('/', "\\")),
        cwd: launcher_dir.to_owned(),
        env: None,
        label: NATIVE_LABEL.to_owned(),
    }
}

fn base_wine_env(ctx: &WineContext) -> LaunchEnv {
    wine_env(ctx.host()).into_iter().collect()
}

pub struct CrossOverLaunch<'a> {
    pub nams_exe: &'a str,
    pub game_dir: &'a str,
    pub launcher_dir: &'a str,
    pub wine_binary: &'a str,
    pub bottle: &'a CrossOverBottle,
    pub prefix: &'a str,
}

pub fn build_crossover(ctx: &WineContext, launch: &CrossOverLaunch<'_>, invocation: NamsInvocation<'_>) -> LaunchCommand {
    let mut args = vec![
        "--bottle".to_owned(),
        launch.bottle.name.clone(),
        "--workdir".to_owned(),
        launch.launcher_dir.to_owned(),
        launch.nams_exe.to_owned(),
    ];
    args.extend(invocation_args(invocation, &to_wine_path(launch.game_dir)));
    let mut env = base_wine_env(ctx);
    env.insert(BOTTLE_VAR.to_owned(), launch.bottle.name.clone());
    env.insert("WINEPREFIX".to_owned(), launch.prefix.to_owned());
    LaunchCommand {
        command: launch.wine_binary.to_owned(),
        args,
        cwd: launch.launcher_dir.to_owned(),
        env: Some(env),
        label: format!("CrossOver Wine ({})", launch.bottle.name),
    }
}

pub fn build_plain_wine(
    ctx: &WineContext,
    nams_exe: &str,
    game_dir: &str,
    launcher_dir: &str,
    wine_binary: &str,
    prefix: Option<&str>,
    invocation: NamsInvocation<'_>,
) -> LaunchCommand {
    let mut args = vec![nams_exe.to_owned()];
    args.extend(invocation_args(invocation, &to_wine_path(game_dir)));
    let mut env = base_wine_env(ctx);
    if let Some(prefix) = prefix {
        env.insert("WINEPREFIX".to_owned(), prefix.to_owned());
    }
    LaunchCommand {
        command: wine_binary.to_owned(),
        args,
        cwd: launcher_dir.to_owned(),
        env: Some(env),
        label: WINE_LABEL.to_owned(),
    }
}

pub struct ProtonLaunch<'a> {
    pub nams_exe: &'a str,
    pub game_dir: &'a str,
    pub game_exe: &'a str,
    pub launcher_dir: &'a str,
    pub proton_path: &'a str,
}

pub fn build_proton(ctx: &WineContext, launch: &ProtonLaunch<'_>, invocation: NamsInvocation<'_>) -> Option<LaunchCommand> {
    if !is_file(launch.proton_path) {
        return None;
    }
    let compat_data = compat_data_path(ctx, Some(launch.game_exe));
    let _ = std::fs::create_dir_all(&compat_data);
    let client_root = steam_root(ctx, Some(launch.game_exe));
    let steam_home = steam_home_for_client(ctx, &client_root);

    let verb = match invocation {
        NamsInvocation::Run { .. } | NamsInvocation::Game { .. } => "run",
        NamsInvocation::Verify | NamsInvocation::Config(_) => "runinprefix",
    };
    let mut args = vec![verb.to_owned(), launch.nams_exe.to_owned()];
    args.extend(invocation_args(invocation, &to_wine_path(launch.game_dir)));
    let mut env = base_wine_env(ctx);
    env.insert(CLIENT_INSTALL_VAR.to_owned(), client_root);
    env.insert(COMPAT_DATA_VAR.to_owned(), compat_data);
    env.insert("SteamAppId".to_owned(), NIER_STEAM_APP_ID.to_owned());
    env.insert("SteamGameId".to_owned(), NIER_STEAM_APP_ID.to_owned());
    if let Some(home) = steam_home {
        env.insert("HOME".to_owned(), home);
    }
    Some(LaunchCommand {
        command: launch.proton_path.to_owned(),
        args,
        cwd: launch.launcher_dir.to_owned(),
        env: Some(env),
        label: PROTON_LABEL.to_owned(),
    })
}

fn posix_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

fn cmd_quote(value: &str) -> String {
    format!("\"{value}\"")
}

pub fn format_script(command: &LaunchCommand, windows: bool) -> String {
    let quote: fn(&str) -> String = if windows { cmd_quote } else { posix_quote };
    let line = std::iter::once(&command.command)
        .chain(&command.args)
        .map(|part| quote(part))
        .collect::<Vec<_>>()
        .join(" ");
    let env: Vec<(&String, &String)> = command.env.iter().flatten().collect();
    if windows {
        let mut lines = vec![format!("cd /d {}", quote(&command.cwd))];
        lines.extend(env.iter().map(|(key, value)| format!("set {key}={value}")));
        lines.push(line);
        return lines.join("\r\n");
    }
    let mut invocation: Vec<String> = env.iter().map(|(key, value)| format!("{key}={}", quote(value))).collect();
    invocation.push(line);
    format!("cd {} && {}", quote(&command.cwd), invocation.join(" "))
}
