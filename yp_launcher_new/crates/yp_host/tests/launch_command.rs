mod support;

use indexmap::IndexMap;
use support::FakeSteamTree;
use yp_core::wine_paths::{posix, NIER_STEAM_APP_ID};
use yp_core::{NamsInvocation, Os};
use yp_host::launch::command::{
    build_crossover, build_native, build_plain_wine, build_proton, format_script, invocation_args, nams_run_args,
    nams_verify_args, CrossOverLaunch, LaunchCommand, ProtonLaunch,
};
use yp_host::wine::crossover::CrossOverBottle;
use yp_host::wine::WineContext;

const BOTTLE_PATH: &str = "/Users/d/Library/Application Support/CrossOver/Bottles/Steam";
const LAUNCHER_DIR: &str = "/Users/d/Library/Application Support/com.vluurie.yplauncher/bins";
const WINE: &str = "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine";
const RUN: NamsInvocation = NamsInvocation::Run { vanilla: false };

fn game_dir() -> String {
    format!("{BOTTLE_PATH}/drive_c/Program Files (x86)/Steam/steamapps/common/NieRAutomata")
}

fn nams_exe() -> String {
    format!("{LAUNCHER_DIR}/NAMS.exe")
}

fn ctx() -> WineContext {
    WineContext::isolated(support::host_for(Os::MacOs, &[], "/nonexistent-root"))
}

fn crossover_command() -> LaunchCommand {
    let bottle = CrossOverBottle::new("Steam", BOTTLE_PATH);
    let nams = nams_exe();
    let game = game_dir();
    build_crossover(
        &ctx(),
        &CrossOverLaunch {
            nams_exe: &nams,
            game_dir: &game,
            launcher_dir: LAUNCHER_DIR,
            wine_binary: WINE,
            bottle: &bottle,
            prefix: BOTTLE_PATH,
        },
        RUN,
    )
}

#[test]
fn a_normal_start_has_no_vanilla_flag() {
    assert_eq!(nams_run_args(r"C:\Games\NieR", false), ["run", "--nier-path", r"C:\Games\NieR"]);
}

#[test]
fn a_vanilla_start_passes_vanilla_to_nams_run() {
    assert_eq!(
        nams_run_args(r"C:\Games\NieR", true),
        ["run", "--nier-path", r"C:\Games\NieR", "--vanilla"]
    );
    assert_eq!(invocation_args(NamsInvocation::Run { vanilla: true }, "x").last().unwrap(), "--vanilla");
    assert_eq!(nams_verify_args("x"), ["verify", "--nier-path", "x", "--json"]);
}

#[test]
fn crossover_runs_the_wine_wrapper_not_the_exe() {
    assert_eq!(crossover_command().command, WINE);
}

#[test]
fn crossover_passes_the_bottle_and_the_launcher_workdir() {
    assert_eq!(crossover_command().args[..4], ["--bottle", "Steam", "--workdir", LAUNCHER_DIR]);
}

#[test]
fn crossover_keeps_the_exe_a_host_path() {
    assert_eq!(crossover_command().args[4], nams_exe());
}

#[test]
fn crossover_nier_path_is_a_windows_path() {
    assert_eq!(
        crossover_command().args[5..],
        ["run", "--nier-path", r"C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata"]
    );
}

#[test]
fn crossover_sets_both_cx_bottle_and_wineprefix() {
    let env = crossover_command().env.unwrap();
    assert_eq!(env["CX_BOTTLE"], "Steam");
    assert_eq!(env["WINEPREFIX"], BOTTLE_PATH);
    assert_eq!(env["WINEDEBUG"], "-all");
}

#[test]
fn crossover_passes_only_its_own_variables() {
    let env = crossover_command().env.unwrap();
    let mut keys: Vec<&str> = env.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["CX_BOTTLE", "WINEDEBUG", "WINEPREFIX"]);
}

#[test]
fn crossover_names_the_bottle_in_the_label() {
    assert_eq!(crossover_command().label, "CrossOver Wine (Steam)");
}

#[test]
fn plain_wine_passes_the_exe_first_with_no_bottle_flags() {
    let command = build_plain_wine(&ctx(), &nams_exe(), &game_dir(), LAUNCHER_DIR, "/usr/bin/wine", Some(BOTTLE_PATH), RUN);
    assert_eq!(command.command, "/usr/bin/wine");
    assert_eq!(
        command.args,
        [
            nams_exe(),
            "run".to_owned(),
            "--nier-path".to_owned(),
            r"C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata".to_owned()
        ]
    );
    let env = command.env.unwrap();
    assert_eq!(env["WINEPREFIX"], BOTTLE_PATH);
    assert!(!env.contains_key("CX_BOTTLE"));
    assert_eq!(command.label, "Wine");
}

#[test]
fn plain_wine_omits_wineprefix_without_a_prefix() {
    let command = build_plain_wine(&ctx(), &nams_exe(), &game_dir(), LAUNCHER_DIR, "/usr/bin/wine", None, RUN);
    assert!(!command.env.unwrap().contains_key("WINEPREFIX"));
}

#[test]
fn native_runs_the_exe_directly_with_a_backslash_path() {
    let command = build_native(r"C:\yp\bins\NAMS.exe", "D:/Games/NieRAutomata", r"C:\yp\bins", RUN);
    assert_eq!(command.command, r"C:\yp\bins\NAMS.exe");
    assert_eq!(command.args, ["run", "--nier-path", r"D:\Games\NieRAutomata"]);
    assert_eq!(command.env, None);
    assert_eq!(command.label, "Windows");
    let spec = command.to_process_spec();
    assert!(spec.hide_console);
    assert_eq!(spec.cwd.unwrap(), std::path::PathBuf::from(r"C:\yp\bins"));
}

#[test]
fn native_verify_produces_the_verify_subcommand_with_json() {
    let command = build_native(r"C:\yp\bins\NAMS.exe", "D:/Games/NieRAutomata", r"C:\yp\bins", NamsInvocation::Verify);
    assert_eq!(command.args, ["verify", "--nier-path", r"D:\Games\NieRAutomata", "--json"]);
}

#[test]
fn proton_with_a_missing_binary_yields_no_command() {
    let nams = nams_exe();
    let game = game_dir();
    let exe = format!("{game}/NieRAutomata.exe");
    let launch = ProtonLaunch {
        nams_exe: &nams,
        game_dir: &game,
        game_exe: &exe,
        launcher_dir: LAUNCHER_DIR,
        proton_path: "/nonexistent/proton",
    };
    assert!(build_proton(&ctx(), &launch, RUN).is_none());
}

#[test]
fn proton_injects_the_confined_home_for_a_native_steam_install() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    tree.add_steam_client();
    let game = tree.add_nier(None);
    let proton = tree.add_proton("GE-Proton");
    let exe = posix::join(&game, "NieRAutomata.exe");
    let outside_home = WineContext::isolated(support::linux_host("/nonexistent-home"))
        .with_steam_roots(vec![tree.steam_root()]);
    let command = build_proton(
        &outside_home,
        &ProtonLaunch {
            nams_exe: "/run/bins/NAMS.exe",
            game_dir: &game,
            game_exe: &exe,
            launcher_dir: "/run/bins",
            proton_path: &proton,
        },
        RUN,
    )
    .unwrap();
    assert_eq!(command.label, "Proton");
    assert_eq!(
        command.args,
        [
            "run".to_owned(),
            "/run/bins/NAMS.exe".to_owned(),
            "run".to_owned(),
            "--nier-path".to_owned(),
            format!("Z:{}", game.replace('/', "\\")),
        ]
    );
    let env = command.env.unwrap();
    assert_eq!(env["HOME"], tree.home);
    assert_eq!(env["SteamAppId"], NIER_STEAM_APP_ID);
    assert_eq!(env["SteamGameId"], NIER_STEAM_APP_ID);
    assert!(env["STEAM_COMPAT_DATA_PATH"].ends_with("524220"));
    assert!(std::path::Path::new(&env["STEAM_COMPAT_DATA_PATH"]).is_dir());
}

#[test]
fn proton_in_a_second_library_keeps_compatdata_there() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let library = posix::join_all(&tree.home, &["ssd", "SteamLibrary"]);
    let game = tree.add_nier(Some(&library));
    let proton = tree.add_proton("GE-Proton");
    let exe = posix::join(&game, "NieRAutomata.exe");
    let command = build_proton(
        &tree.native(),
        &ProtonLaunch {
            nams_exe: "/run/bins/NAMS.exe",
            game_dir: &game,
            game_exe: &exe,
            launcher_dir: "/run/bins",
            proton_path: &proton,
        },
        RUN,
    )
    .unwrap();
    let env = command.env.unwrap();
    assert_eq!(env["STEAM_COMPAT_CLIENT_INSTALL_PATH"], root);
    assert_eq!(
        env["STEAM_COMPAT_DATA_PATH"],
        posix::join_all(&library, &["steamapps", "compatdata", NIER_STEAM_APP_ID])
    );
    assert!(!env.contains_key("HOME"));
}

fn command(label: &str, command: &str, args: &[&str], cwd: &str, env: &[(&str, &str)]) -> LaunchCommand {
    LaunchCommand {
        command: command.to_owned(),
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        cwd: cwd.to_owned(),
        env: if env.is_empty() {
            None
        } else {
            Some(env.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect::<IndexMap<_, _>>())
        },
        label: label.to_owned(),
    }
}

#[test]
fn posix_script_exports_the_env_inline_before_the_command() {
    let proton = command(
        "Proton",
        "/steam/GE-Proton/proton",
        &["run", "/run/bins/NAMS.exe"],
        "/run/bins",
        &[
            ("STEAM_COMPAT_DATA_PATH", "/mnt/games/Lib/steamapps/compatdata/524220"),
            ("STEAM_COMPAT_CLIENT_INSTALL_PATH", "/home/d/.steam/steam"),
        ],
    );
    assert_eq!(
        format_script(&proton, false),
        "cd '/run/bins' && STEAM_COMPAT_DATA_PATH='/mnt/games/Lib/steamapps/compatdata/524220' STEAM_COMPAT_CLIENT_INSTALL_PATH='/home/d/.steam/steam' '/steam/GE-Proton/proton' 'run' '/run/bins/NAMS.exe'"
    );
}

#[test]
fn posix_script_quotes_env_values_with_spaces_and_quotes() {
    let spaced = command("Wine", "wine", &["NAMS.exe"], "/r", &[("WINEPREFIX", "/home/d/My Games/pfx")]);
    assert_eq!(format_script(&spaced, false), "cd '/r' && WINEPREFIX='/home/d/My Games/pfx' 'wine' 'NAMS.exe'");
    let quoted = command("Wine", "wine", &["it's"], "/r", &[]);
    assert_eq!(format_script(&quoted, false), r"cd '/r' && 'wine' 'it'\''s'");
}

#[test]
fn windows_script_sets_the_env_line_by_line() {
    let win = command("Windows", r"C:\yp\NAMS.exe", &["run"], r"C:\yp", &[("SteamAppId", "524220")]);
    assert_eq!(
        format_script(&win, true),
        "cd /d \"C:\\yp\"\r\nset SteamAppId=524220\r\n\"C:\\yp\\NAMS.exe\" \"run\""
    );
}

#[test]
fn no_env_yields_the_bare_command() {
    let bare = command("Wine", "wine", &["NAMS.exe"], "/r", &[]);
    assert_eq!(format_script(&bare, false), "cd '/r' && 'wine' 'NAMS.exe'");
    assert_eq!(bare.display(), "wine NAMS.exe");
}
