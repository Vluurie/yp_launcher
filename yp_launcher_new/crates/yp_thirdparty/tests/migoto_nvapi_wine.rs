mod common;

use std::path::Path;

use common::{temp, write};
use yp_core::{HostEnv, Os};
use yp_thirdparty::migoto::PARKED_SUFFIX;
use yp_thirdparty::MigotoRuntime;

fn host(os: Os, root: &Path) -> HostEnv {
    HostEnv::for_test(os, &[], root)
}

fn incoming(root: &Path) {
    let source = root.join("incoming");
    write(&source.join("d3d11.dll"), "migoto");
    write(&source.join("nvapi64.dll"), "nvidia shim");
    write(&source.join("d3dcompiler_46.dll"), "compiler");
}

#[test]
fn windows_keeps_nvapi64_dll() {
    let root = temp("yp_migoto_nvapi_");
    incoming(root.path());
    assert!(!host(Os::Windows, root.path()).runs_game_through_wine());
    assert!(!MigotoRuntime::new(&host(Os::Windows, root.path())).parks_nvapi());
}

#[test]
fn linux_reports_that_the_game_runs_through_wine() {
    let root = temp("yp_migoto_nvapi_");
    incoming(root.path());
    assert!(host(Os::Linux, root.path()).runs_game_through_wine());
    assert!(MigotoRuntime::new(&host(Os::Linux, root.path())).parks_nvapi());
}

#[test]
fn repair_parks_an_existing_nvapi64_dll_on_wine_hosts() {
    let root = temp("yp_migoto_nvapi_");
    incoming(root.path());
    let dest = root.path().join("installed");
    write(&dest.join("3dmigoto.dll"), "migoto");
    let nvapi = dest.join("nvapi64.dll");
    write(&nvapi, "nvidia shim");

    MigotoRuntime::new(&host(Os::Linux, root.path())).repair_in(&dest).unwrap();

    assert!(!nvapi.exists());
    assert!(dest.join(format!("nvapi64.dll{PARKED_SUFFIX}")).is_file());
    assert!(!dest.join("d3dcompiler_46.dll").exists(), "only nvapi is parked");
}

#[test]
fn repair_leaves_nvapi64_dll_alone_on_windows() {
    let root = temp("yp_migoto_nvapi_");
    incoming(root.path());
    let dest = root.path().join("installed_win");
    write(&dest.join("3dmigoto.dll"), "migoto");
    let nvapi = dest.join("nvapi64.dll");
    write(&nvapi, "nvidia shim");

    MigotoRuntime::new(&host(Os::Windows, root.path())).repair_in(&dest).unwrap();

    assert!(nvapi.is_file());
}

#[test]
fn install_on_wine_skips_nvapi_but_keeps_the_compiler() {
    let root = temp("yp_migoto_nvapi_");
    let source = root.path().join("incoming");
    write(&source.join("d3d11.dll"), "HackerDevice");
    write(&source.join("nvapi64.dll"), "nvidia shim");
    write(&source.join("d3dcompiler_46.dll"), "compiler");
    let game_dir = root.path().join("game");
    write(&game_dir.join("nams").join("nams.toml"), "disable_3dmigoto_loading = true\n");
    let game = yp_core::GamePaths::new(&game_dir);

    let runtime = MigotoRuntime::new(&host(Os::Linux, root.path()));
    let c = yp_thirdparty::classify(&source);
    yp_thirdparty::GraphicsRuntime::install(&runtime, &game, &c).unwrap();

    let dest = yp_thirdparty::migoto::install_dir(&game);
    assert!(dest.join("3dmigoto.dll").is_file());
    assert!(dest.join("d3dcompiler_46.dll").is_file());
    assert!(!dest.join("nvapi64.dll").exists());
}
