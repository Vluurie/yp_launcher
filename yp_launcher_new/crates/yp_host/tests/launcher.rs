mod support;

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use support::FakeSteamTree;
use yp_core::testing::{FakeResponse, FakeRunner};
use yp_core::{AppPaths, LaunchUnavailable, Os, ProcessOutput, SettingsStore};
use yp_host::gpu::{GpuProbe, PREFER_DEDICATED_GPU};
use yp_host::launch::failure::{FailureCategory, FailureHint, FailureKind, HintLine};
use yp_host::launch::wrapper;
use yp_host::launch::{record, LaunchOutcome, Launcher};
use yp_host::platform::linux::linux_adapter_with;
use yp_host::platform::macos::macos_adapter_with;
use yp_host::platform::windows::WindowsAdapter;
use yp_host::wine::WineContext;

struct Setup {
    _dir: tempfile::TempDir,
    paths: AppPaths,
    game: PathBuf,
    settings: Arc<SettingsStore>,
}

fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_test(dir.path());
    std::fs::create_dir_all(paths.plugins_dir()).unwrap();
    std::fs::write(paths.nams_exe(), "exe").unwrap();
    std::fs::write(paths.yorha_dll(), "dll").unwrap();
    let game = dir.path().join("game");
    std::fs::create_dir_all(&game).unwrap();
    std::fs::write(game.join("NieRAutomata.exe"), "PRJ_028").unwrap();
    Setup {
        _dir: dir,
        paths,
        game,
        settings: Arc::new(SettingsStore::in_memory()),
    }
}

fn windows_launcher(setup: &Setup, runner: &FakeRunner) -> Launcher {
    let runner: Arc<FakeRunner> = Arc::new(runner.clone());
    let platform = Arc::new(WindowsAdapter::new(
        support::host_for(Os::Windows, &[], "/r"),
        runner.clone(),
    ));
    Launcher::new(platform, runner, setup.paths.clone(), setup.settings.clone())
        .with_grace(Duration::from_millis(50))
        .with_gpu_registry(false)
}

fn failure(outcome: LaunchOutcome) -> yp_host::launch::failure::LaunchFailure {
    match outcome {
        LaunchOutcome::Failed(failure) => failure,
        LaunchOutcome::Started { .. } => panic!("expected a failure"),
    }
}

#[test]
fn a_process_still_running_after_the_grace_window_counts_as_started() {
    let setup = setup();
    let runner = FakeRunner::new().respond(|_| true, FakeResponse::Running { output: "hosting game".into() });
    let launcher = windows_launcher(&setup, &runner);
    let (sender, receiver) = mpsc::channel();
    let outcome = launcher.start(&setup.game, true, move |finished| {
        let _ = sender.send(finished);
    });
    let LaunchOutcome::Started { record: started } = outcome else {
        panic!("expected a start");
    };
    assert_eq!(started.label, "Windows");
    assert!(started.still_running());
    let finished = receiver.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(finished.exit_code, Some(-1));
    assert_eq!(finished.output_tail, "hosting game");
    assert!(finished.duration().is_some());
    let stored = record::read(&setup.paths).unwrap();
    assert_eq!(stored.exit_code, Some(-1));
    let call = &runner.calls()[0];
    assert_eq!(call.args_lossy().last().unwrap(), "--vanilla");
    assert!(call.hide_console);
    assert_eq!(call.cwd.as_deref(), Some(setup.paths.runtime_dir.as_path()));
}

#[test]
fn an_early_exit_with_a_nams_error_is_parsed_and_captured() {
    let setup = setup();
    let output = "booting\nERROR[3] Could not create the nams folder\n  Phase: prepare\n  OS error: Access is denied. (os error 5)\n  Likely cause: read-only drive\n  Try: move the game\n";
    let runner = FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::failure(3, output));
    let launcher = windows_launcher(&setup, &runner);
    let failure = failure(launcher.start(&setup.game, false, |_| {}));
    let report = failure.report().unwrap();
    assert_eq!(report.code, Some(3));
    assert!(!report.panic);
    assert_eq!(report.headline, "Could not create the nams folder");
    assert_eq!(report.phase.as_deref(), Some("prepare"));
    assert_eq!(report.os_error.as_deref(), Some("Access is denied. (os error 5)"));
    assert_eq!(report.cause.as_deref(), Some("read-only drive"));
    assert_eq!(failure.category(), FailureCategory::FolderCreate);
    assert_eq!(
        failure.hints(),
        vec![
            HintLine::FromNams("move the game".into()),
            HintLine::Known(FailureHint::WritableFolder),
            HintLine::Known(FailureHint::ProgramFiles),
        ]
    );
    let log = failure.captured_log.clone().unwrap();
    assert!(log.starts_with(&setup.paths.logs_dir));
    let captured = std::fs::read_to_string(&log).unwrap();
    assert!(captured.starts_with("NAMS captured output\n"));
    assert!(captured.contains("Exit code: 3\n"));
    assert!(captured.ends_with(output));
    let stored = record::read(&setup.paths).unwrap();
    assert_eq!(stored.exit_code, Some(3));
    assert!(!stored.still_running());
}

#[test]
fn an_early_exit_without_a_report_keeps_the_exit_code() {
    let setup = setup();
    let runner = FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::failure(1337, "something odd"));
    let failure = failure(windows_launcher(&setup, &runner).start(&setup.game, false, |_| {}));
    assert_eq!(failure.kind, FailureKind::Exited { code: 1337 });
    assert_eq!(failure.raw_output, "something odd");
    assert_eq!(failure.category(), FailureCategory::ExitedUnexpectedly);
    assert!(failure.reported_by_nams());

    let silent = FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::failure(2, ""));
    let failure = failure_of(windows_launcher(&setup, &silent).start(&setup.game, false, |_| {}));
    assert_eq!(failure.kind, FailureKind::Exited { code: 2 });
    assert_eq!(failure.captured_log, None);
    assert_eq!(failure.category(), FailureCategory::InstallNotFound);
}

fn failure_of(outcome: LaunchOutcome) -> yp_host::launch::failure::LaunchFailure {
    failure(outcome)
}

#[test]
fn a_panic_wins_over_an_error_header() {
    let setup = setup();
    let runner = FakeRunner::new().respond_program(
        "NAMS.exe",
        ProcessOutput::failure(101, "ERROR[1] first\nPANIC[9] thread main panicked\nTry: ignored for panics\n"),
    );
    let failure = failure(windows_launcher(&setup, &runner).start(&setup.game, false, |_| {}));
    assert!(failure.is_panic());
    assert_eq!(failure.code(), Some(9));
    assert_eq!(failure.category(), FailureCategory::Panic);
    assert_eq!(
        failure.hints(),
        vec![HintLine::Known(FailureHint::PanicShare), HintLine::Known(FailureHint::PanicReboot)]
    );
}

#[test]
fn preflight_reports_missing_files_and_a_missing_exe() {
    let setup = setup();
    std::fs::remove_file(setup.paths.yorha_dll()).unwrap();
    let runner = FakeRunner::new();
    let failure = failure(windows_launcher(&setup, &runner).start(&setup.game, false, |_| {}));
    assert!(matches!(&failure.kind, FailureKind::FilesMissing { missing, .. } if missing == &vec!["yorha_protocol.dll"]));
    assert_eq!(failure.category(), FailureCategory::Unknown);
    assert!(failure.hints().is_empty());
    assert!(runner.calls().is_empty());

    let setup = self::setup();
    let elsewhere = setup.game.join("missing");
    let failure = failure_of(windows_launcher(&setup, &runner).start(&elsewhere, false, |_| {}));
    assert!(matches!(&failure.kind, FailureKind::ExeNotFound { exe, .. } if exe == &elsewhere.join("NieRAutomata.exe")));
}

#[test]
fn the_nams_log_is_cleared_before_launching() {
    let setup = setup();
    std::fs::create_dir_all(&setup.paths.logs_dir).unwrap();
    std::fs::write(setup.paths.nams_log(), "old run").unwrap();
    let runner = FakeRunner::new().respond(|_| true, FakeResponse::Running { output: String::new() });
    assert!(windows_launcher(&setup, &runner).start(&setup.game, false, |_| {}).started());
    assert_eq!(std::fs::read_to_string(setup.paths.nams_log()).unwrap(), "");
    assert!(setup.game.join("nams").is_dir());
}

#[test]
fn spawn_errors_become_internal_failures() {
    let setup = setup();
    let runner = FakeRunner::new().respond(|_| true, FakeResponse::NotFound);
    let failure = failure(windows_launcher(&setup, &runner).start(&setup.game, false, |_| {}));
    assert!(matches!(failure.kind, FailureKind::Internal { .. }));
}

#[test]
fn no_compatibility_layer_is_reported_as_unavailable() {
    let setup = setup();
    let runner = Arc::new(FakeRunner::new());
    let platform = Arc::new(macos_adapter_with(
        WineContext::isolated(support::host_for(Os::MacOs, &[], "/r")),
        runner.clone(),
    ));
    let launcher = Launcher::new(platform, runner, setup.paths.clone(), setup.settings.clone());
    let failure = failure(launcher.start(&setup.game, false, |_| {}));
    assert_eq!(failure.kind, FailureKind::Unavailable(LaunchUnavailable::NoCompatLayer { os: Os::MacOs }));
    assert_eq!(launcher.preview(&setup.game), None);
}

#[test]
fn windows_preview_is_a_cmd_script() {
    let setup = setup();
    let preview = windows_launcher(&setup, &FakeRunner::new()).preview(&setup.game).unwrap();
    assert!(preview.starts_with("cd /d \""));
    assert!(preview.contains("\"run\" \"--nier-path\""));
    assert!(!preview.contains("--vanilla"));
}

#[test]
fn linux_launches_get_the_gpu_offload_env_and_the_wrapper() {
    let setup = setup();
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let game = tree.add_nier(None);
    let proton = tree.add_proton("GE-Proton");
    let runner = Arc::new(FakeRunner::new());
    let platform = Arc::new(linux_adapter_with(tree.native(), runner.clone()));
    wrapper::save(&setup.settings, "gamescope -f --").unwrap();
    let launcher = Launcher::new(platform, runner, setup.paths.clone(), setup.settings.clone()).with_gpu_probe(GpuProbe {
        nvidia_driver_loaded: false,
        render_nodes: 2,
    });
    let resolved = launcher.resolve(Path::new(&game), false).unwrap();
    assert!(resolved.prefer_gpu);
    assert_eq!(resolved.command.command, "gamescope");
    assert_eq!(resolved.command.args[..3], ["-f", "--", proton.as_str()]);
    let env = resolved.command.env.as_ref().unwrap();
    assert_eq!(env["DRI_PRIME"], "1");
    assert_eq!(env["SteamAppId"], "524220");
    let preview = launcher.preview(Path::new(&game)).unwrap();
    assert!(preview.starts_with("cd '"));
    assert!(preview.contains("DRI_PRIME='1'"));
    assert!(preview.contains("'gamescope' '-f' '--'"));

    setup.settings.set(&PREFER_DEDICATED_GPU, &false).unwrap();
    let without_gpu = launcher.resolve(Path::new(&game), false).unwrap();
    assert!(!without_gpu.command.env.unwrap().contains_key("DRI_PRIME"));
}
