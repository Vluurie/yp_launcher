use std::sync::Arc;
use std::time::Duration;

use yp_core::testing::FakeRunner;
use yp_core::{AppPaths, CancelToken, HostEnv, NamsCommandRequest, NamsInvocation, Os, PlatformAdapter, SettingsStore};
use yp_host::discovery::{installs_near_launcher, scan_roots};
use yp_host::platform::windows::WindowsAdapter;
use yp_host::session::AppSession;

#[test]
fn a_separate_game_on_another_drive_is_found_without_steam_or_vendor_paths() {
    let root = tempfile::tempdir().unwrap();
    let system_drive = root.path().join("system-drive");
    let storage_drive = root.path().join("mounted-storage");
    let game = storage_drive.join("arbitrary folder").join("my copy of the game");
    std::fs::create_dir_all(game.join("data")).unwrap();
    std::fs::write(game.join("NieRAutomata.exe"), "PRJ_028").unwrap();
    let host = HostEnv::for_test(Os::Windows, &[], &system_drive.join("Tools").join("YP"));
    std::fs::create_dir_all(host.exe_dir()).unwrap();
    assert!(installs_near_launcher(host.exe_dir()).is_empty());

    let found = scan_roots(
        &[system_drive, storage_drive],
        &CancelToken::new(),
        Duration::from_secs(5),
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, game);
    assert!(found[0].has_data);
    let paths = AppPaths::for_test(host.exe_dir());
    let platform = WindowsAdapter::new(host, Arc::new(FakeRunner::new()));
    let command = platform
        .build_nams_command(&NamsCommandRequest {
            nams_exe: &paths.nams_exe(),
            launcher_dir: &paths.runtime_dir,
            game_dir: &found[0].path,
            invocation: NamsInvocation::Run { vanilla: false },
        })
        .unwrap();
    assert_eq!(
        command.args_lossy(),
        ["run", "--nier-path", &game.to_string_lossy().replace('/', "\\")]
    );
}

#[test]
fn portable_game_beside_the_launcher_is_detected_and_passed_to_nams() {
    let root = tempfile::tempdir().unwrap();
    let game = root.path().join("mounted-drive").join("Games").join("installation-42");
    std::fs::create_dir_all(game.join("data")).unwrap();
    std::fs::write(game.join("NieRAutomata.exe"), "PRJ_028").unwrap();

    let host =
        HostEnv::for_test(Os::Windows, &[], &game.join("YoRHa Protocol Launcher")).with_windows_build(Some(19045));
    std::fs::create_dir_all(host.exe_dir()).unwrap();
    let runner = Arc::new(FakeRunner::new());
    let platform = WindowsAdapter::new(host.clone(), runner.clone());
    let found = platform.discover_fast();
    assert!(found.iter().any(|install| install.path == game && install.has_data));
    assert_eq!(found[0].path, game);
    assert!(runner.calls().is_empty());

    let settings = SettingsStore::in_memory();
    let mut session = AppSession::load(&settings);
    assert!(!session.is_directory_selected());
    session
        .set_directory(&settings, &found[0].path.to_string_lossy())
        .unwrap();
    assert_eq!(AppSession::load(&settings).directory(), Some(game.clone()));

    let paths = AppPaths::for_test(host.exe_dir());
    let nams = paths.nams_exe();
    let command = platform
        .build_nams_command(&NamsCommandRequest {
            nams_exe: &nams,
            launcher_dir: &paths.runtime_dir,
            game_dir: &session.directory().unwrap(),
            invocation: NamsInvocation::Run { vanilla: false },
        })
        .unwrap();
    assert_eq!(command.program, nams);
    assert_eq!(command.cwd.as_deref(), Some(paths.runtime_dir.as_path()));
    assert_eq!(
        command.args_lossy(),
        ["run", "--nier-path", &game.to_string_lossy().replace('/', "\\")]
    );
}
