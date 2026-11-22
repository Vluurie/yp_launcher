mod support;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use yp_core::testing::{FakeResponse, FakeRunner};
use yp_core::{AppPaths, Os, ProcessOutput};
use yp_host::nams_cli::{info, textures_list, verify, VerifyStatus};
use yp_host::platform::macos::macos_adapter_with;
use yp_host::platform::windows::WindowsAdapter;
use yp_host::wine::WineContext;

fn paths_with_nams(root: &Path) -> AppPaths {
    let paths = AppPaths::for_test(root);
    std::fs::create_dir_all(&paths.runtime_dir).unwrap();
    std::fs::write(paths.nams_exe(), "exe").unwrap();
    paths
}

fn windows(runner: &FakeRunner) -> WindowsAdapter {
    WindowsAdapter::new(support::host_for(Os::Windows, &[], "/r"), Arc::new(runner.clone()))
}

#[test]
fn textures_list_parses_packs_with_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths_with_nams(dir.path());
    let runner = FakeRunner::new().respond_program(
        "NAMS.exe",
        ProcessOutput::success(
            r#"{"packs":[{"name":"2B HD","source":"mod","mod":"hd","character":"2B","outfit_conditional":true,"enabled":false,"dds_count":12,"load_order_index":3,"path":"inject/textures/2B HD"},{"name":"bare"},"skip"]}"#,
        ),
    );
    let packs = textures_list(&runner, &paths, Path::new("G:/Game"), Some(Duration::from_secs(5))).unwrap();
    assert_eq!(packs.len(), 2);
    assert_eq!(packs[0].mod_id.as_deref(), Some("hd"));
    assert!(packs[0].outfit_conditional);
    assert!(!packs[0].enabled);
    assert_eq!(packs[0].dds_count, 12);
    assert_eq!(packs[0].load_order_index, Some(3));
    assert_eq!(packs[1].source, "standalone");
    assert!(packs[1].enabled);
    assert_eq!(packs[1].load_order_index, None);
    let call = &runner.calls()[0];
    assert_eq!(call.args_lossy(), ["textures", "list", "--nier-path", "G:/Game", "--json"]);
    assert_eq!(call.cwd.as_deref(), Some(paths.runtime_dir.as_path()));
    assert!(call.hide_console);
}

#[test]
fn textures_list_is_none_on_failure_bad_json_or_missing_exe() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths_with_nams(dir.path());
    let failing = FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::failure(1, "boom"));
    assert!(textures_list(&failing, &paths, Path::new("G"), None).is_none());
    let garbage = FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::success("{\"packs\":1}"));
    assert!(textures_list(&garbage, &paths, Path::new("G"), None).is_none());
    let slow = FakeRunner::new().respond(|_| true, FakeResponse::Running { output: String::new() });
    assert!(textures_list(&slow, &paths, Path::new("G"), Some(Duration::from_millis(1))).is_none());
    let empty = tempfile::tempdir().unwrap();
    assert!(textures_list(&FakeRunner::new(), &AppPaths::for_test(empty.path()), Path::new("G"), None).is_none());
}

#[test]
fn info_reads_version_game_bin_and_plugins() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths_with_nams(dir.path());
    let runner = FakeRunner::new().respond_program(
        "NAMS.exe",
        ProcessOutput::success(
            r#"{"version":"1.2.3","paths":{"nier_install_error":"bad"},"runtime_files":{"game_bin":{"present":true,"size":42}},"plugins":[{"name":"yorha_protocol.dll","size":10},{"size":2.0},{"name":7}]}"#,
        ),
    );
    let info = info(&runner, &paths, Path::new("G"), None).unwrap();
    assert_eq!(info.version.as_deref(), Some("1.2.3"));
    assert_eq!(info.nier_install_error.as_deref(), Some("bad"));
    assert!(info.game_bin_present);
    assert_eq!(info.game_bin_size, Some(42));
    let plugins: Vec<(&str, i64)> = info.plugins.iter().map(|p| (p.name.as_str(), p.size)).collect();
    assert_eq!(plugins, [("yorha_protocol.dll", 10), ("?", 2), ("7", 0)]);
    assert_eq!(runner.calls()[0].args_lossy(), ["info", "--nier-path", "G", "--json"]);
}

#[test]
fn verify_runs_through_the_platform_command() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths_with_nams(dir.path());
    let runner = FakeRunner::new()
        .respond_program("NAMS.exe", ProcessOutput::success("{\"ok\":true,\"checks\":[{\"name\":\"nier_exe\",\"status\":\"ok\"}]}"));
    let outcome = verify(&windows(&runner), &runner, &paths, Path::new("D:/Games/NieR"));
    assert_eq!(outcome.status, VerifyStatus::Ok);
    assert_eq!(outcome.exit_code, Some(0));
    assert_eq!(runner.calls()[0].args_lossy(), ["verify", "--nier-path", r"D:\Games\NieR", "--json"]);
}

#[test]
fn verify_maps_steam_timeouts_and_missing_runtimes() {
    let dir = tempfile::tempdir().unwrap();
    let paths = paths_with_nams(dir.path());
    let steam = FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::failure(20, ""));
    assert_eq!(verify(&windows(&steam), &steam, &paths, Path::new("G")).status, VerifyStatus::SteamNotRunning);
    let hung = FakeRunner::new().respond(|_| true, FakeResponse::Running { output: String::new() });
    assert_eq!(verify(&windows(&hung), &hung, &paths, Path::new("G")).status, VerifyStatus::TimedOut);
    let missing = FakeRunner::new().respond(|_| true, FakeResponse::NotFound);
    let errored = verify(&windows(&missing), &missing, &paths, Path::new("G"));
    assert_eq!(errored.status, VerifyStatus::Error);
    assert!(errored.raw.is_some());
    let mac = macos_adapter_with(WineContext::isolated(support::host_for(Os::MacOs, &[], "/r")), Arc::new(FakeRunner::new()));
    assert_eq!(verify(&mac, &FakeRunner::new(), &paths, Path::new("/g")).status, VerifyStatus::NoRuntime);
    let empty = tempfile::tempdir().unwrap();
    assert_eq!(
        verify(&windows(&steam), &steam, &AppPaths::for_test(empty.path()), Path::new("G")).status,
        VerifyStatus::Error
    );
}
