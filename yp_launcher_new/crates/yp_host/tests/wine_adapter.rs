mod support;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use support::{FakeBottleTree, FakeSteamTree};
use yp_core::testing::FakeRunner;
use yp_core::wine_paths::posix;
use yp_core::{
    AppPaths, CancelToken, LaunchUnavailable, NamsCommandRequest, NamsInvocation, Os, PlatformAdapter, ProcessOutput,
    SelectionRejection,
};
use yp_host::platform::linux::linux_adapter_with;
use yp_host::platform::macos::macos_adapter_with;
use yp_host::platform::wine::WineAdapter;
use yp_host::platform::{adapter_for, nams_settings_path, HostPlatform, VerifyTimeoutHint};

fn runner() -> Arc<FakeRunner> {
    Arc::new(FakeRunner::new())
}

fn macos(ctx: yp_host::wine::WineContext) -> WineAdapter {
    macos_adapter_with(ctx, runner())
}

fn request<'a>(nams: &'a Path, game: &'a Path, launcher: &'a Path, invocation: NamsInvocation<'a>) -> NamsCommandRequest<'a> {
    NamsCommandRequest {
        nams_exe: nams,
        launcher_dir: launcher,
        game_dir: game,
        invocation,
    }
}

#[test]
fn adapter_for_picks_the_host_adapter() {
    let windows = adapter_for(support::host_for(Os::Windows, &[], "/r"), runner());
    assert!(windows.supports_desktop_shortcut());
    assert!(!windows.uses_native_title_bar());
    assert!(windows.can_launch_game());
    let linux = adapter_for(support::host_for(Os::Linux, &[], "/r"), runner());
    assert!(linux.is_experimental());
    assert!(linux.uses_native_title_bar());
    assert_eq!(linux.verify_timeout_hint(), VerifyTimeoutHint::Linux);
    let mac = adapter_for(support::host_for(Os::MacOs, &[], "/r"), runner());
    assert!(!mac.is_experimental());
    assert_eq!(mac.verify_timeout_hint(), VerifyTimeoutHint::Generic);
    let upcast: Arc<dyn PlatformAdapter> = mac;
    assert_eq!(upcast.host().os(), Os::MacOs);
}

#[test]
fn the_oem_code_page_is_only_known_on_windows() {
    let code_page = yp_host::platform::oem_code_page();
    if cfg!(windows) {
        assert!(code_page.is_some_and(|code_page| code_page > 0), "{code_page:?}");
    } else {
        assert_eq!(code_page, None);
    }
}

#[test]
fn linux_runtime_dir_honours_xdg() {
    let host = support::host_for(Os::Linux, &[("XDG_DATA_HOME", "/data"), ("HOME", "/home/u")], "/r");
    let paths = AppPaths::detect(&host).unwrap();
    assert!(paths.runtime_dir.ends_with(Path::new("yp_launcher").join("bins")));
}

#[test]
fn linux_discover_fast_finds_a_native_steam_install() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let dir = tree.add_nier(None);
    let adapter = linux_adapter_with(tree.native(), runner());
    let found = adapter.discover_fast();
    assert!(found.iter().any(|install| install.path == Path::new(&dir)));
    assert_eq!(adapter.discover_deep(&CancelToken::new()), found);
}

#[test]
fn linux_merges_native_and_bottle_installs_without_duplicates() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let native_dir = tree.add_nier(None);
    let bottles = FakeBottleTree::create();
    bottles.add_bottle("Steam");
    let exe = bottles.add_nier("Steam");
    let ctx = tree.native().with_bottle_roots(vec![bottles.root.clone()]);
    let found = linux_adapter_with(ctx, runner()).discover_fast();
    let paths: Vec<PathBuf> = found.into_iter().map(|install| install.path).collect();
    assert_eq!(paths, [PathBuf::from(native_dir), PathBuf::from(posix::dirname(&exe))]);
}

#[test]
fn linux_accepts_a_native_steam_exe_path() {
    let adapter = linux_adapter_with(yp_host::wine::WineContext::isolated(support::linux_host("/h")), runner());
    assert_eq!(
        adapter.reject_game_selection(Path::new(
            "/home/u/.local/share/Steam/steamapps/common/NieRAutomata/NieRAutomata.exe"
        )),
        None
    );
}

#[test]
fn nams_settings_land_in_the_game_dir_cache() {
    let game = "/home/u/.local/share/Steam/steamapps/common/NieRAutomata";
    let expected = Path::new(game).join("nams").join("_internal").join("cache").join("settings.json");
    let adapter = linux_adapter_with(yp_host::wine::WineContext::isolated(support::linux_host("/h")), runner());
    assert_eq!(adapter.resolve_nams_settings_path(Some(Path::new(game))), Some(expected.clone()));
    assert_eq!(nams_settings_path(Some(Path::new(game))), Some(expected));
}

#[test]
fn accepts_an_exe_inside_a_prefix() {
    let adapter = macos(yp_host::wine::WineContext::isolated(support::host_for(Os::MacOs, &[], "/r")));
    assert_eq!(
        adapter.reject_game_selection(Path::new(
            "/u/Library/Application Support/CrossOver/Bottles/Steam/drive_c/game/NieRAutomata.exe"
        )),
        None
    );
}

#[test]
fn rejects_an_exe_outside_any_prefix() {
    let adapter = macos(yp_host::wine::WineContext::isolated(support::host_for(Os::MacOs, &[], "/r")));
    let exe = Path::new("/Users/d/Games/NieRAutomata.exe");
    assert_eq!(
        adapter.reject_game_selection(exe),
        Some(SelectionRejection::OutsidePrefix { exe: exe.to_path_buf() })
    );
}

#[test]
fn settings_path_is_in_the_game_dir_for_bottles_and_plain_dirs() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let adapter = macos(tree.ctx(Os::MacOs, None));
    let game = PathBuf::from(posix::join_all(&bottle, &["drive_c", "game"]));
    assert_eq!(
        adapter.resolve_nams_settings_path(Some(&game)),
        Some(game.join("nams").join("_internal").join("cache").join("settings.json"))
    );
    let plain = Path::new("/Users/d/Games");
    assert_eq!(
        adapter.resolve_nams_settings_path(Some(plain)),
        Some(plain.join("nams").join("_internal").join("cache").join("settings.json"))
    );
}

#[test]
fn settings_path_is_none_before_a_game_dir_is_known() {
    let adapter = macos(yp_host::wine::WineContext::isolated(support::host_for(Os::MacOs, &[], "/r")));
    assert_eq!(adapter.resolve_nams_settings_path(None), None);
    assert_eq!(adapter.resolve_nams_settings_path(Some(Path::new(""))), None);
}

#[test]
fn without_any_runtime_launching_is_unavailable() {
    let adapter = macos(yp_host::wine::WineContext::isolated(support::host_for(Os::MacOs, &[], "/r")));
    let (nams, game, launcher) = (Path::new("/run/bins/NAMS.exe"), Path::new("/g"), Path::new("/run/bins"));
    assert_eq!(
        adapter.build_launch_command(&request(nams, game, launcher, NamsInvocation::Run { vanilla: false })),
        Err(LaunchUnavailable::NoCompatLayer { os: Os::MacOs })
    );
    assert!(!adapter.can_launch_game());
}

#[test]
fn drives_crossover_inside_the_bottle_with_both_path_forms() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let exe = tree.add_nier("Steam");
    let game_dir = PathBuf::from(posix::dirname(&exe));
    let wine = tree.add_wine_script();
    let adapter = macos(tree.ctx(Os::MacOs, Some(wine.clone())));
    assert!(adapter.can_launch_game());
    let launcher_dir = posix::join_all(&bottle, &["drive_c", "yp"]);
    let nams_exe = posix::join(&launcher_dir, "NAMS.exe");
    let (nams, launcher) = (Path::new(&nams_exe), Path::new(&launcher_dir));
    let command = adapter
        .build_launch_command(&request(nams, &game_dir, launcher, NamsInvocation::Run { vanilla: false }))
        .unwrap();
    assert_eq!(command.command, wine);
    assert_eq!(command.args[..4], ["--bottle", "Steam", "--workdir", launcher_dir.as_str()]);
    assert_eq!(command.args[4], nams_exe);
    assert_eq!(command.args[5..], ["run", "--nier-path", r"C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata"]);
    let env = command.env.as_ref().unwrap();
    assert_eq!(env["CX_BOTTLE"], "Steam");
    assert_eq!(env["WINEPREFIX"], bottle);
    assert_eq!(command.label, "CrossOver Wine (Steam)");

    let spec = adapter
        .build_nams_command(&request(nams, &game_dir, launcher, NamsInvocation::Verify))
        .unwrap();
    let args = spec.args_lossy();
    assert_eq!(args[4], nams_exe);
    assert_eq!(
        args[5..],
        ["verify", "--nier-path", r"C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata", "--json"]
    );
    assert_eq!(spec.program, PathBuf::from(&wine));
}

#[cfg(unix)]
#[test]
fn refuses_a_prefix_without_a_z_drive_only_when_nams_lives_outside() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let exe = tree.add_nier("Steam");
    let wine = tree.add_wine_script();
    let adapter = macos(tree.ctx(Os::MacOs, Some(wine)));
    let game_dir = PathBuf::from(posix::dirname(&exe));
    let (nams, launcher) = (Path::new("/run/bins/NAMS.exe"), Path::new("/run/bins"));
    let run = NamsInvocation::Run { vanilla: false };
    assert_eq!(
        adapter.build_launch_command(&request(nams, &game_dir, launcher, run)),
        Err(LaunchUnavailable::NoZDrive { prefix: PathBuf::from(&bottle) })
    );
    support::mkdirs(&posix::join_all(&bottle, &["dosdevices", "z:"]));
    assert!(adapter.build_launch_command(&request(nams, &game_dir, launcher, run)).is_ok());
}

#[test]
fn a_missing_z_drive_is_refused() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let exe = tree.add_nier("Steam");
    let wine = tree.add_wine_script();
    let adapter = macos(tree.ctx(Os::MacOs, Some(wine)));
    let game_dir = PathBuf::from(posix::dirname(&exe));
    let result = adapter.build_launch_command(&request(
        Path::new("/run/bins/NAMS.exe"),
        &game_dir,
        Path::new("/run/bins"),
        NamsInvocation::Run { vanilla: false },
    ));
    assert_eq!(result, Err(LaunchUnavailable::NoZDrive { prefix: PathBuf::from(&bottle) }));
}

#[test]
fn proton_launch_inside_a_steam_library_uses_the_compat_prefix() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let game = tree.add_nier(None);
    let proton = tree.add_proton("GE-Proton");
    let adapter = linux_adapter_with(tree.native(), runner());
    let command = adapter
        .build_launch_command(&request(
            Path::new("/run/bins/NAMS.exe"),
            Path::new(&game),
            Path::new("/run/bins"),
            NamsInvocation::Run { vanilla: true },
        ))
        .unwrap();
    assert_eq!(command.command, proton);
    assert_eq!(command.label, "Proton");
    assert_eq!(command.args.last().unwrap(), "--vanilla");
}

#[test]
fn proton_override_from_the_environment_wins() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let game = tree.add_nier(None);
    let custom = support::write(&posix::join(&tree.home, "custom/proton"), "#!/bin/sh\n");
    let host = support::host_for(Os::Linux, &[("HOME", &tree.home), ("YP_PROTON_PATH", &custom)], &tree.home);
    let ctx = yp_host::wine::WineContext::isolated(host).with_steam_roots(vec![tree.steam_root()]);
    let runtime = yp_host::wine::runtime::detect(&ctx, Some(&posix::join(&game, "NieRAutomata.exe"))).unwrap();
    assert_eq!(runtime.binary, custom);
    assert_eq!(runtime.source, yp_host::wine::runtime::WineSource::Proton);
    assert_eq!(
        runtime.prefix.as_deref(),
        Some(posix::join_all(&tree.steam_root(), &["steamapps", "compatdata", "524220", "pfx"]).as_str())
    );
}

#[test]
fn plain_wine_on_path_is_the_last_resort() {
    let tree = FakeBottleTree::create();
    let bin = support::mkdirs(&posix::join(&tree.root, "bin"));
    support::write(&posix::join(&bin, "wine"), "");
    let host = support::host_for(Os::Linux, &[("PATH", &bin), ("WINEPREFIX", "/custom/pfx")], &tree.root);
    let ctx = yp_host::wine::WineContext::isolated(host).with_path_search(true);
    let runtime = yp_host::wine::runtime::detect(&ctx, Some("/games/NieRAutomata/NieRAutomata.exe")).unwrap();
    assert_eq!(runtime.source, yp_host::wine::runtime::WineSource::Wine);
    assert_eq!(runtime.prefix.as_deref(), Some("/custom/pfx"));
    assert!(yp_host::wine::runtime::is_available(&ctx));
}

#[test]
fn process_queries_use_pgrep_and_pkill() {
    let fake = Arc::new(
        FakeRunner::new()
            .respond_program("pgrep", ProcessOutput::success("123\n"))
            .respond_program("pkill", ProcessOutput::failure(1, "")),
    );
    let adapter = linux_adapter_with(yp_host::wine::WineContext::isolated(support::linux_host("/h")), fake.clone());
    assert!(adapter.is_game_running());
    assert!(!adapter.terminate_game());
    let calls = fake.calls();
    assert_eq!(calls[0].program, PathBuf::from("pgrep"));
    assert_eq!(calls[0].args_lossy(), ["-f", "NAMS.exe"]);
    assert_eq!(calls[1].args_lossy(), ["-f", "NieRAutomata.exe"]);
    assert_eq!(calls[2].program, PathBuf::from("pkill"));
    assert_eq!(calls[2].args_lossy(), ["-f", "NAMS.exe"]);
    assert_eq!(calls[3].args_lossy(), ["-f", "NieRAutomata.exe"]);
}

#[test]
fn reveal_goes_through_the_runner() {
    let fake = Arc::new(FakeRunner::new());
    let adapter = macos_adapter_with(
        yp_host::wine::WineContext::isolated(support::host_for(Os::MacOs, &[], "/r")),
        fake.clone(),
    );
    adapter.reveal(Path::new("/x/file.txt"), true).unwrap();
    assert_eq!(fake.calls()[0].program, PathBuf::from("open"));
    assert_eq!(fake.calls()[0].args_lossy(), ["-R", "/x/file.txt"]);
}
