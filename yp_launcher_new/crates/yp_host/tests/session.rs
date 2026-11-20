use std::path::{Path, PathBuf};
use std::sync::Arc;

use yp_core::{HostEnv, Os, ProcessOutput, SelectionRejection, SettingsStore};
use yp_core::testing::FakeRunner;
use yp_host::session::{
    AppSession, BusyCounter, LaunchStatus, PlayButtonState, SessionError, SessionFlags, NIER_DIRECTORY,
};
use yp_host::win10_compat::{ensure_ran_for_directory, normalize_key, Win10SetupOutcome, WIN10_SETUP_DONE};

#[test]
fn the_saved_directory_is_loaded_and_saved() {
    let settings = SettingsStore::in_memory();
    assert_eq!(AppSession::load(&settings), AppSession::default());
    assert_eq!(AppSession::saved_directory(&settings), None);
    let mut session = AppSession::load(&settings);
    assert!(!session.can_play());
    session.set_error(SessionError::InvalidExe);
    session.set_directory(&settings, "D:/Games/NieR").unwrap();
    assert_eq!(settings.get(&NIER_DIRECTORY), "D:/Games/NieR");
    assert_eq!(session.error, None);
    assert!(session.can_play());
    assert_eq!(session.directory(), Some(PathBuf::from("D:/Games/NieR")));
    assert_eq!(AppSession::load(&settings).selected_directory, "D:/Games/NieR");
    assert_eq!(AppSession::saved_directory(&settings), Some(PathBuf::from("D:/Games/NieR")));
}

#[test]
fn updates_clear_the_transient_error_and_status_like_copy_with() {
    let mut session = AppSession {
        selected_directory: "G".into(),
        ..AppSession::default()
    };
    session.set_status(Some(LaunchStatus::Launching));
    session.set_play_button(PlayButtonState::Loading);
    assert_eq!(session.status, None);
    assert!(!session.can_play());
    session.set_status(Some(LaunchStatus::Running));
    session.set_error(SessionError::Rejected(SelectionRejection::OutsidePrefix { exe: "x".into() }));
    assert_eq!(session.status, None);
    assert!(session.error.is_some());
    session.set_status(Some(LaunchStatus::Stopping));
    assert_eq!(session.error, None);
    session.set_error(SessionError::Message("custom".into()));
    session.clear_error();
    assert_eq!((session.error.clone(), session.status), (None, None));
}

#[test]
fn syncing_follows_the_real_process_state() {
    let mut session = AppSession::default();
    assert!(session.sync_game_running(true));
    assert_eq!((session.play_button, session.status), (PlayButtonState::Running, Some(LaunchStatus::Running)));
    assert!(!session.sync_game_running(true));
    assert!(session.sync_game_running(false));
    assert_eq!((session.play_button, session.status), (PlayButtonState::Idle, Some(LaunchStatus::Stopped)));
    session.set_play_button(PlayButtonState::Loading);
    assert!(!session.sync_game_running(true));
    assert!(!session.sync_game_running(false));
}

#[test]
fn busy_counter_counts_nested_and_concurrent_work() {
    let busy = BusyCounter::new();
    assert!(!busy.is_busy());
    let outer = busy.enter();
    let value = busy.run(|| busy.count());
    assert_eq!(value, 2);
    assert_eq!(busy.count(), 1);
    drop(outer);
    let shared = busy.clone();
    let worker = std::thread::spawn(move || shared.run(|| 7));
    assert_eq!(worker.join().unwrap(), 7);
    assert_eq!(busy.count(), 0);
}

#[test]
fn tab_selection_requests_are_taken_once_by_their_tab() {
    let mut flags = SessionFlags::default();
    flags.request_tab_selection(5, "mod-id");
    assert_eq!(flags.active_tab, 5);
    assert_eq!(flags.take_tab_selection(3), None);
    assert_eq!(flags.take_tab_selection(5).as_deref(), Some("mod-id"));
    assert_eq!(flags.take_tab_selection(5), None);
    assert_eq!(flags.bump_detection_refresh(), 1);
    assert_eq!(flags.bump_detection_refresh(), 2);
}

#[test]
fn win10_setup_keys_are_normalised_like_the_flutter_launcher() {
    assert_eq!(normalize_key(r"C:\Games\NieR Automata"), "c__games_nier_automata");
}

#[test]
fn win10_setup_runs_once_per_directory_on_windows_11_only() {
    let dir = tempfile::tempdir().unwrap();
    let game = dir.path().join("game");
    std::fs::create_dir_all(&game).unwrap();
    let settings = SettingsStore::in_memory();
    let runner = Arc::new(FakeRunner::new().respond_program("cmd.exe", ProcessOutput::success("")));
    let win11 = HostEnv::for_test(Os::Windows, &[], Path::new("/r")).with_windows_build(Some(22631));
    let win10 = HostEnv::for_test(Os::Windows, &[], Path::new("/r")).with_windows_build(Some(19045));
    let linux = HostEnv::for_test(Os::Linux, &[], Path::new("/r"));

    assert_eq!(ensure_ran_for_directory(&linux, runner.as_ref(), &settings, &game), Win10SetupOutcome::NotWindows);
    assert_eq!(ensure_ran_for_directory(&win10, runner.as_ref(), &settings, &game), Win10SetupOutcome::NotWindows11);
    assert_eq!(ensure_ran_for_directory(&win11, runner.as_ref(), &settings, &game), Win10SetupOutcome::NoBatchFile);
    std::fs::write(game.join("win10setup.bat"), "@echo off").unwrap();
    assert_eq!(ensure_ran_for_directory(&win11, runner.as_ref(), &settings, &game), Win10SetupOutcome::Ran);
    assert_eq!(ensure_ran_for_directory(&win11, runner.as_ref(), &settings, &game), Win10SetupOutcome::AlreadyDone);
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].args_lossy(), ["/c", "win10setup.bat"]);
    assert_eq!(calls[0].cwd.as_deref(), Some(game.as_path()));
    assert!(settings.get(&WIN10_SETUP_DONE).contains(&normalize_key(&game.to_string_lossy())));
}

#[test]
fn legacy_win10_setup_flags_are_honoured() {
    let dir = tempfile::tempdir().unwrap();
    let game = dir.path().join("game");
    std::fs::create_dir_all(&game).unwrap();
    std::fs::write(game.join("win10setup.bat"), "").unwrap();
    let legacy = dir.path().join("shared_preferences.json");
    let key = normalize_key(&game.to_string_lossy());
    std::fs::write(&legacy, format!("{{\"flutter.win10setup_done_{key}\":true}}")).unwrap();
    let settings = SettingsStore::in_memory();
    settings.import_flutter_prefs(&legacy).unwrap();
    let runner = FakeRunner::new();
    let win11 = HostEnv::for_test(Os::Windows, &[], Path::new("/r")).with_windows_build(Some(22000));
    assert_eq!(ensure_ran_for_directory(&win11, &runner, &settings, &game), Win10SetupOutcome::AlreadyDone);
    assert!(runner.calls().is_empty());
}
