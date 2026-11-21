use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use gpui::TestAppContext;
use yp_core::ProcessOutput;
use yp_core::testing::{FakeResponse, FakeRunner};
use yp_host::launch::failure::FailureKind;

use super::Harness;
use crate::prefs::{APP_LOCALE, NIER_DIRECTORY};
use crate::session::{
    DirectoryChange, GameError, LaunchRequest, LaunchStatus, PlayState, SelectionCheck,
    SessionEvent,
};
use crate::{StateExt, Stores};

type Transitions = Rc<RefCell<Vec<(PlayState, Option<LaunchStatus>)>>>;

fn running_runner() -> FakeRunner {
    FakeRunner::new().respond(
        |_| true,
        FakeResponse::Running {
            output: String::new(),
        },
    )
}

fn record_transitions(
    stores: &Stores,
    cx: &mut TestAppContext,
) -> (Transitions, gpui::Subscription) {
    let transitions: Transitions = Rc::default();
    let seen = transitions.clone();
    let subscription = cx.update(|cx| {
        cx.subscribe(&stores.session, move |_, event, _| {
            if let SessionEvent::PlayStateChanged { state, status } = event {
                seen.borrow_mut().push((*state, *status));
            }
        })
    });
    (transitions, subscription)
}

fn wait_for(cx: &mut TestAppContext, mut done: impl FnMut(&mut TestAppContext) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for the game");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn play_state(stores: &Stores, cx: &mut TestAppContext) -> PlayState {
    stores
        .session
        .read_with(cx, |session, _| session.play_state())
}

#[gpui::test]
fn selecting_a_directory_persists_and_binds_every_store(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let seen = changes.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.session, move |_, event, _| {
            if let SessionEvent::DirectoryChanged { game, change } = event {
                seen.borrow_mut().push((game.clone(), *change));
            }
        })
    });
    stores.session.read_with(cx, |session, _| {
        assert!(!session.is_directory_selected());
        assert!(!session.can_play());
    });

    harness.select_game(&stores, cx);

    assert_eq!(
        *changes.borrow(),
        vec![(harness.game.clone(), DirectoryChange::Selected)]
    );
    stores.session.read_with(cx, |session, _| {
        assert_eq!(session.game(), Some(harness.game.clone()));
        assert!(session.can_play());
        assert!(!session.is_preparing());
    });
    let saved = cx.update(|cx| cx.prefs().get(&NIER_DIRECTORY));
    assert_eq!(saved, harness.game.root().to_string_lossy());
    cx.foreground_executor()
        .block_test(cx.update(|cx| cx.prefs().flush()))
        .unwrap();
    assert_eq!(harness.env.settings.get(&NIER_DIRECTORY), saved);
    assert!(stores.config.read_with(cx, |config, _| config.is_loaded()));
    assert!(
        stores
            .detection
            .read_with(cx, |detection, _| detection.snapshot().is_some())
    );
}

#[gpui::test]
fn startup_restores_the_saved_directory(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness
        .env
        .settings
        .set(
            &NIER_DIRECTORY,
            &harness.game.root().to_string_lossy().into_owned(),
        )
        .unwrap();
    let stores = harness.init(cx);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let seen = changes.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.session, move |_, event, _| {
            if let SessionEvent::DirectoryChanged { change, .. } = event {
                seen.borrow_mut().push(*change);
            }
        })
    });

    let startup = cx.update(crate::startup);
    cx.run_until_parked();

    assert!(cx.foreground_executor().block_test(startup).is_ok());
    assert_eq!(*changes.borrow(), vec![DirectoryChange::Restored]);
    assert!(stores.config.read_with(cx, |config, _| config.is_loaded()));
    assert!(
        stores
            .profiles
            .read_with(cx, |profiles, _| !profiles.profiles().is_empty())
    );
}

#[gpui::test]
fn launching_moves_through_loading_running_and_stopped(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let harness = Harness::with_runner(running_runner());
    harness
        .env
        .settings
        .set(&APP_LOCALE, &"de".to_owned())
        .unwrap();
    let stores = harness.start(cx);
    let (transitions, _subscription) = record_transitions(&stores, cx);
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    let _events = cx.update(|cx| {
        cx.subscribe(&stores.session, move |_, event, _| match event {
            SessionEvent::LaunchRequested { vanilla } => {
                seen.borrow_mut().push(format!("requested:{vanilla}"))
            }
            SessionEvent::Started(_) => seen.borrow_mut().push("started".to_owned()),
            SessionEvent::Stopped(record) => seen
                .borrow_mut()
                .push(format!("stopped:{}", record.is_some())),
            _ => {}
        })
    });

    stores.session.update(cx, |session, cx| {
        session.launch(LaunchRequest { vanilla: true }, cx);
        assert_eq!(session.play_state(), PlayState::Loading);
        assert_eq!(session.status(), Some(LaunchStatus::Launching));
        assert!(!session.can_play());
    });
    wait_for(cx, |cx| {
        stores.session.read_with(cx, |session, _| {
            session.play_state() == PlayState::Idle
                && session.status() == Some(LaunchStatus::Stopped)
        })
    });

    assert_eq!(
        *transitions.borrow(),
        vec![
            (PlayState::Loading, Some(LaunchStatus::Launching)),
            (PlayState::Running, Some(LaunchStatus::Running)),
            (PlayState::Idle, Some(LaunchStatus::Stopped)),
        ]
    );
    assert_eq!(
        *events.borrow(),
        vec![
            "requested:true".to_owned(),
            "started".to_owned(),
            "stopped:true".to_owned()
        ]
    );
    assert!(
        stores
            .session
            .read_with(cx, |session, _| session.error().is_none())
    );

    let calls = harness.runner.calls();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0]
            .args_lossy()
            .contains(&yp_core::consts::ARG_VANILLA.to_owned())
    );
    assert_eq!(
        harness.cli.value(crate::config::OVERLAY_LANGUAGE_ID),
        Some(serde_json::json!("de"))
    );
}

#[gpui::test]
fn a_game_that_exits_at_once_reports_a_launch_failure(cx: &mut TestAppContext) {
    let harness = Harness::with_runner(FakeRunner::new().respond(
        |_| true,
        FakeResponse::Output(ProcessOutput::failure(3, "boom")),
    ));
    let stores = harness.start(cx);
    let (transitions, _subscription) = record_transitions(&stores, cx);
    let failures = Rc::new(RefCell::new(Vec::new()));
    let seen = failures.clone();
    let _failures = cx.update(|cx| {
        cx.subscribe(&stores.session, move |_, event, _| {
            if let SessionEvent::LaunchFailed(failure) = event {
                seen.borrow_mut().push(failure.clone());
            }
        })
    });

    stores.session.update(cx, |session, cx| {
        session.launch(LaunchRequest::default(), cx);
    });
    cx.run_until_parked();

    assert_eq!(
        *transitions.borrow(),
        vec![
            (PlayState::Loading, Some(LaunchStatus::Launching)),
            (PlayState::Idle, None),
        ]
    );
    assert_eq!(failures.borrow().len(), 1);
    stores.session.read_with(cx, |session, _| {
        assert!(matches!(session.error(), Some(GameError::LaunchFailed(_))));
        assert_eq!(
            session.launch_failure().map(|failure| failure.code()),
            Some(Some(3))
        );
        assert!(session.can_play());
    });

    stores
        .session
        .update(cx, |session, cx| session.clear_error(cx));
    assert!(
        stores
            .session
            .read_with(cx, |session, _| session.error().is_none())
    );
}

#[gpui::test]
fn missing_runtime_files_fail_before_anything_is_spawned(cx: &mut TestAppContext) {
    let harness = Harness::with_runner(running_runner());
    std::fs::remove_file(harness.env.paths.nams_exe()).unwrap();
    let stores = harness.start(cx);

    stores.session.update(cx, |session, cx| {
        session.launch(LaunchRequest::default(), cx);
    });
    cx.run_until_parked();

    assert!(harness.runner.calls().is_empty());
    stores.session.read_with(cx, |session, _| {
        assert_eq!(session.play_state(), PlayState::Idle);
        assert!(matches!(
            session.launch_failure().map(|failure| &failure.kind),
            Some(FailureKind::FilesMissing { .. })
        ));
    });
}

#[gpui::test]
fn launching_needs_a_directory_and_an_idle_game(cx: &mut TestAppContext) {
    let harness = Harness::with_runner(running_runner());
    let stores = harness.init(cx);

    stores.session.update(cx, |session, cx| {
        session.launch(LaunchRequest::default(), cx);
    });
    cx.run_until_parked();

    assert_eq!(play_state(&stores, cx), PlayState::Idle);
    assert!(harness.runner.calls().is_empty());
}

#[gpui::test]
fn a_running_game_is_picked_up_and_can_be_stopped(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let (transitions, _subscription) = record_transitions(&stores, cx);

    harness.platform.running.store(true, Ordering::SeqCst);
    stores
        .session
        .update(cx, |session, cx| session.sync_game_running(cx));
    cx.run_until_parked();
    assert_eq!(play_state(&stores, cx), PlayState::Running);

    stores.session.update(cx, |session, cx| {
        session.stop(cx);
        assert_eq!(session.play_state(), PlayState::Loading);
        assert_eq!(session.status(), Some(LaunchStatus::Stopping));
    });
    cx.run_until_parked();

    assert_eq!(harness.platform.terminations.load(Ordering::SeqCst), 1);
    assert_eq!(
        *transitions.borrow(),
        vec![
            (PlayState::Running, Some(LaunchStatus::Running)),
            (PlayState::Loading, Some(LaunchStatus::Stopping)),
            (PlayState::Idle, Some(LaunchStatus::Stopped)),
        ]
    );

    stores
        .session
        .update(cx, |session, cx| session.sync_game_running(cx));
    cx.run_until_parked();
    assert_eq!(play_state(&stores, cx), PlayState::Idle);
}

#[gpui::test]
fn a_failed_stop_keeps_the_game_running(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    harness.platform.running.store(true, Ordering::SeqCst);
    harness
        .platform
        .terminate_succeeds
        .store(false, Ordering::SeqCst);
    stores
        .session
        .update(cx, |session, cx| session.sync_game_running(cx));
    cx.run_until_parked();

    stores.session.update(cx, |session, cx| session.stop(cx));
    cx.run_until_parked();

    stores.session.read_with(cx, |session, _| {
        assert_eq!(session.play_state(), PlayState::Running);
        assert_eq!(session.error(), Some(&GameError::StopFailed));
    });
}

#[gpui::test]
fn launch_warnings_show_when_loading_is_switched_off(cx: &mut TestAppContext) {
    let harness = Harness::with_runner(FakeRunner::new().respond(
        |_| true,
        FakeResponse::Output(ProcessOutput::failure(1, "")),
    ));
    let stores = harness.start(cx);
    stores.config.update(cx, |config, cx| {
        config.set_now(
            "nams.disable_plugin_loading",
            serde_json::json!(true),
            cx,
        );
    });
    cx.run_until_parked();
    assert!(
        !stores
            .launch_options
            .read_with(cx, |options, _| options.show_launch_warnings())
    );

    let request = stores
        .launch_options
        .read_with(cx, |options, _| options.request());
    stores
        .session
        .update(cx, |session, cx| session.launch(request, cx));
    cx.run_until_parked();

    assert!(
        stores
            .launch_options
            .read_with(cx, |options, _| options.show_launch_warnings())
    );
}

#[gpui::test]
fn maintenance_helpers_run_in_the_background(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let log_file = harness.env.paths.nams_log();
    std::fs::write(&log_file, "old run\n").unwrap();
    let cleared = Rc::new(RefCell::new(Vec::new()));
    let seen = cleared.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.session, move |_, event, _| {
            if let SessionEvent::CacheCleared(result) = event {
                seen.borrow_mut().push(*result);
            }
        })
    });

    let verify = stores
        .session
        .read_with(cx, |session, cx| session.verify_install(cx));
    let preview = stores
        .session
        .read_with(cx, |session, cx| session.launch_command_preview(cx));
    let report = stores
        .session
        .read_with(cx, |session, cx| session.collect_diagnostics(cx));
    let clear = stores
        .session
        .update(cx, |session, cx| session.clear_cache(cx));
    cx.run_until_parked();

    assert!(cx.foreground_executor().block_test(verify).is_some());
    assert!(harness.runner.calls().iter().any(|call| {
        call.args_lossy()
            .contains(&yp_core::consts::ARG_VERIFY.to_owned())
    }));
    assert!(
        cx.foreground_executor()
            .block_test(preview)
            .is_some_and(|script| script.contains(yp_core::consts::ARG_RUN))
    );
    let report = cx.foreground_executor().block_test(report).unwrap();
    assert_eq!(report.game_dir.as_deref(), Some(harness.game.root()));
    let result = cx.foreground_executor().block_test(clear);
    cx.run_until_parked();
    assert!(result.logs_deleted >= 1);
    assert!(result.detection_reset);
    assert!(!log_file.exists());
    assert_eq!(*cleared.borrow(), vec![result]);
}

#[gpui::test]
fn selection_check_validates_the_exe_before_accepting_it(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let check = |exe: std::path::PathBuf| {
        let task = stores
            .session
            .read_with(cx, |session, cx| session.check_selection(exe, cx));
        cx.foreground_executor().block_test(task)
    };

    assert_eq!(
        check(harness.root.path().join("absent.exe")),
        SelectionCheck::Missing
    );
    assert_eq!(check(harness.game.exe()), SelectionCheck::NotNierExe);

    let mut genuine = vec![0u8; 4096];
    genuine[0] = 0x4d;
    genuine[1] = 0x5a;
    genuine[2048..2055].copy_from_slice(yp_core::consts::GAME_SIGNATURE.as_bytes());
    std::fs::write(harness.game.exe(), genuine).unwrap();
    assert_eq!(check(harness.game.exe()), SelectionCheck::Accepted);
}
