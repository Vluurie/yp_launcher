use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{Subscription, TestAppContext};
use serde_json::{Value as Json, json};
use yp_config::testing::FakeNamsCli;
use yp_config::{ResetTarget, cli};
use yp_formats::testing::usm_file;

use super::{Harness, settle};
use crate::config::{H264_ID, OVERLAY_LANGUAGE_ID, TEXTURE_LOAD_ORDER_ID, WRITE_DELAY};
use crate::{ConfigEvent, ConfigOp, Stores};

const SENSITIVITY_ID: &str = "naiom.sensitivity";
const PLUGIN_LOADING_ID: &str = "nams.disable_plugin_loading";
const LODMOD_ENABLED_ID: &str = "lodmod.enabled";
const SHADOW_RESOLUTION_ID: &str = "lodmod.shadow_resolution";
const RELOAD_DELAY: Duration = Duration::from_millis(400);
const H264_CODEC: u32 = 5;

type Events = Rc<RefCell<Vec<ConfigEvent>>>;

fn record(stores: &Stores, cx: &mut TestAppContext) -> (Events, Subscription) {
    let events: Events = Rc::default();
    let sink = events.clone();
    let subscription = cx.update(|cx| {
        cx.subscribe(&stores.config, move |_, event, _| {
            sink.borrow_mut().push(event.clone())
        })
    });
    (events, subscription)
}

fn written(events: &Events) -> usize {
    events
        .borrow()
        .iter()
        .filter(|event| matches!(event, ConfigEvent::Written(_)))
        .count()
}

fn sensitivity(stores: &Stores, cx: &mut TestAppContext) -> Option<f64> {
    stores
        .config
        .read_with(cx, |config, _| config.number(SENSITIVITY_ID))
}

fn edit_outside(harness: &Harness, id: &str, value: Json) {
    cli::set(harness.cli.as_ref(), &[(id.to_owned(), value)], false).unwrap();
}

#[gpui::test]
fn selecting_a_game_loads_the_schema_and_values(cx: &mut TestAppContext) {
    let harness = Harness::with_cli(FakeNamsCli::default().with(SENSITIVITY_ID, json!(3.5)));
    let stores = harness.start(cx);

    stores.config.read_with(cx, |config, _| {
        assert!(config.is_loaded());
        assert!(!config.is_loading());
        assert!(config.failure().is_none());
        assert!(!config.has_pending_writes());
        assert!(config.setting(SENSITIVITY_ID).is_some());
        assert!(!config.schema().categories.is_empty());
        assert_eq!(config.number(SENSITIVITY_ID), Some(3.5));
        assert!(config.is_changed(SENSITIVITY_ID));
        assert!(!config.bool(PLUGIN_LOADING_ID));
        assert_eq!(config.texts("lodmod.shadow_force_all_filters"), vec!["ENKEI"]);
        assert_eq!(config.text(OVERLAY_LANGUAGE_ID), "system");
        assert_eq!(config.value("missing.setting"), Json::Null);
        assert_eq!(config.game(), Some(&harness.game));
    });
    assert_eq!(harness.cli.count("migrate"), 1);
    assert_eq!(harness.cli.count("schema"), 1);
    assert_eq!(harness.cli.count("set"), 0);
}

#[gpui::test]
fn edits_apply_at_once_and_are_written_once_as_a_batch(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let (events, _subscription) = record(&stores, cx);

    for value in [2.5, 3.0, 4.0] {
        stores.config.update(cx, |config, cx| {
            config.set(SENSITIVITY_ID, json!(value), cx);
        });
        assert_eq!(sensitivity(&stores, cx), Some(value));
        settle(cx, WRITE_DELAY / 3);
    }
    stores.config.update(cx, |config, cx| {
        config.set(LODMOD_ENABLED_ID, json!(true), cx);
    });
    assert!(stores.config.read_with(cx, |config, _| config.has_pending_writes()));
    assert_eq!(harness.cli.count("set"), 0);

    settle(cx, WRITE_DELAY);

    assert_eq!(harness.cli.count("set"), 1);
    assert_eq!(written(&events), 1);
    assert_eq!(harness.cli.value(SENSITIVITY_ID), Some(json!(4.0)));
    assert_eq!(harness.cli.value(LODMOD_ENABLED_ID), Some(json!(true)));
    assert!(!stores.config.read_with(cx, |config, _| config.has_pending_writes()));
    let ConfigEvent::Written(report) = &events.borrow()[0] else {
        panic!("expected a write report");
    };
    let ids: Vec<&str> = report.changed.iter().map(|change| change.id.as_str()).collect();
    assert_eq!(ids, vec![SENSITIVITY_ID, LODMOD_ENABLED_ID]);
}

#[gpui::test]
fn setting_the_current_value_writes_nothing(cx: &mut TestAppContext) {
    let harness = Harness::with_cli(
        FakeNamsCli::default()
            .with(SENSITIVITY_ID, json!(2.0))
            .with(PLUGIN_LOADING_ID, json!(false)),
    );
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.set(SENSITIVITY_ID, json!(2.0), cx);
        config.set_now(PLUGIN_LOADING_ID, json!(false), cx);
    });
    settle(cx, WRITE_DELAY);

    assert_eq!(harness.cli.count("set"), 0);
}

#[gpui::test]
fn a_default_value_missing_from_the_file_is_written(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.set_now(PLUGIN_LOADING_ID, json!(false), cx);
    });
    cx.run_until_parked();

    assert_eq!(harness.cli.value(PLUGIN_LOADING_ID), Some(json!(false)));
    assert!(!stores.config.read_with(cx, |config, _| {
        config.values().not_in_file.contains(PLUGIN_LOADING_ID)
    }));
}

#[gpui::test]
fn set_now_and_flush_pending_write_without_the_delay(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.set_now(PLUGIN_LOADING_ID, json!(true), cx);
    });
    cx.run_until_parked();
    assert_eq!(harness.cli.value(PLUGIN_LOADING_ID), Some(json!(true)));

    let flush = stores.config.update(cx, |config, cx| {
        config.set(SENSITIVITY_ID, json!(5.0), cx);
        config.flush_pending(cx)
    });
    cx.run_until_parked();
    cx.foreground_executor().block_test(flush);

    assert_eq!(harness.cli.value(SENSITIVITY_ID), Some(json!(5.0)));
    assert_eq!(harness.cli.count("set"), 2);
}

#[gpui::test]
fn set_many_writes_every_assignment_in_one_call(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.set_many(
            vec![
                (LODMOD_ENABLED_ID.to_owned(), json!(true)),
                (SHADOW_RESOLUTION_ID.to_owned(), json!(4096)),
            ],
            cx,
        );
    });
    cx.run_until_parked();

    assert_eq!(harness.cli.count("set"), 1);
    assert_eq!(harness.cli.value(LODMOD_ENABLED_ID), Some(json!(true)));
    assert_eq!(harness.cli.value(SHADOW_RESOLUTION_ID), Some(json!(4096)));
}

#[gpui::test]
fn reset_writes_the_schema_defaults_of_a_category(cx: &mut TestAppContext) {
    let harness = Harness::with_cli(
        FakeNamsCli::default()
            .with(LODMOD_ENABLED_ID, json!(true))
            .with(SHADOW_RESOLUTION_ID, json!(4096))
            .with(SENSITIVITY_ID, json!(6.0)),
    );
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.reset(&ResetTarget::Category("graphics".to_owned()), cx);
    });
    cx.run_until_parked();

    assert_eq!(harness.cli.value(LODMOD_ENABLED_ID), Some(json!(false)));
    assert_eq!(harness.cli.value(SHADOW_RESOLUTION_ID), Some(json!(2048)));
    assert_eq!(harness.cli.value(SENSITIVITY_ID), Some(json!(6.0)));
    stores.config.read_with(cx, |config, _| {
        assert!(!config.bool(LODMOD_ENABLED_ID));
        assert_eq!(config.number(SENSITIVITY_ID), Some(6.0));
    });
}

#[gpui::test]
fn a_rejected_write_reports_the_failure_and_restores_the_file_value(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let (events, _subscription) = record(&stores, cx);
    harness.cli.reject(SENSITIVITY_ID);

    stores.config.update(cx, |config, cx| {
        config.set_now(SENSITIVITY_ID, json!(9.0), cx);
    });
    cx.run_until_parked();

    stores.config.read_with(cx, |config, _| {
        assert_eq!(config.failure().map(|failure| failure.op), Some(ConfigOp::Write));
        assert_eq!(config.number(SENSITIVITY_ID), Some(2.0));
        assert!(!config.has_pending_writes());
    });
    assert!(events.borrow().iter().any(|event| matches!(
        event,
        ConfigEvent::Failed(failure) if failure.op == ConfigOp::Write
    )));
    assert_eq!(written(&events), 0);
}

#[gpui::test]
fn external_edits_are_reloaded_after_the_debounce(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let (events, _subscription) = record(&stores, cx);
    edit_outside(&harness, SENSITIVITY_ID, json!(7.0));

    stores.config.update(cx, |config, cx| {
        config.on_external_change(&[harness.game.naiom_toml()], cx);
    });
    cx.run_until_parked();
    assert_eq!(sensitivity(&stores, cx), Some(2.0));

    settle(cx, RELOAD_DELAY);
    assert_eq!(sensitivity(&stores, cx), Some(7.0));
    assert!(events
        .borrow()
        .iter()
        .any(|event| matches!(event, ConfigEvent::ReloadedExternally)));
}

#[gpui::test]
fn reloads_keep_edits_that_are_not_written_yet(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    stores.config.update(cx, |config, cx| {
        config.set(SENSITIVITY_ID, json!(3.0), cx);
    });
    edit_outside(&harness, LODMOD_ENABLED_ID, json!(true));
    edit_outside(&harness, SENSITIVITY_ID, json!(7.0));
    let gets = harness.cli.count("get");

    stores
        .config
        .update(cx, |config, cx| config.reload_if_clean(cx));
    cx.run_until_parked();
    assert_eq!(harness.cli.count("get"), gets);
    assert_eq!(sensitivity(&stores, cx), Some(3.0));

    stores.config.update(cx, |config, cx| config.load(cx));
    cx.run_until_parked();
    stores.config.read_with(cx, |config, _| {
        assert!(config.bool(LODMOD_ENABLED_ID));
        assert_eq!(config.number(SENSITIVITY_ID), Some(3.0));
    });

    settle(cx, WRITE_DELAY);
    assert_eq!(harness.cli.value(SENSITIVITY_ID), Some(json!(3.0)));
}

#[gpui::test]
fn own_writes_do_not_count_as_external_changes(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let (events, _subscription) = record(&stores, cx);

    stores.config.update(cx, |config, cx| {
        config.set(SENSITIVITY_ID, json!(3.0), cx);
    });
    settle(cx, WRITE_DELAY);
    stores.config.update(cx, |config, cx| {
        config.on_external_change(&[harness.game.naiom_toml()], cx);
    });
    settle(cx, RELOAD_DELAY);

    assert!(!events
        .borrow()
        .iter()
        .any(|event| matches!(event, ConfigEvent::ReloadedExternally)));
    assert_eq!(sensitivity(&stores, cx), Some(3.0));
}

#[gpui::test]
fn unrelated_files_in_the_config_folder_are_ignored(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    edit_outside(&harness, SENSITIVITY_ID, json!(7.0));
    let other = harness.game.nams_dir().join("notes.txt");

    stores.config.update(cx, |config, cx| {
        config.on_external_change(std::slice::from_ref(&other), cx);
    });
    settle(cx, RELOAD_DELAY);

    assert_eq!(sensitivity(&stores, cx), Some(2.0));
}

#[gpui::test]
fn the_file_watcher_picks_up_outside_edits(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let mut harness = Harness::new();
    harness.env = harness.env.clone().with_file_watching(true);
    std::fs::create_dir_all(harness.game.nams_dir()).unwrap();
    let stores = harness.start(cx);
    edit_outside(&harness, SENSITIVITY_ID, json!(7.0));

    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let mut touched = false;
    while sensitivity(&stores, cx) != Some(7.0) {
        assert!(
            std::time::Instant::now() < deadline,
            "the watcher never reported the edit"
        );
        if !touched {
            std::thread::sleep(Duration::from_millis(200));
            std::fs::write(harness.game.naiom_toml(), "sensitivity = 7.0\n").unwrap();
            touched = true;
        }
        std::thread::sleep(Duration::from_millis(20));
        settle(cx, RELOAD_DELAY);
    }
}

#[gpui::test]
fn h264_cutscene_mods_switch_the_decoder_on_after_loading(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let video = harness
        .game
        .cutscenes_dir()
        .join("Pack HD")
        .join("movie")
        .join("ev0020.usm");
    std::fs::create_dir_all(video.parent().unwrap()).unwrap();
    std::fs::write(&video, usm_file(1920, 1080, H264_CODEC)).unwrap();
    let stores = harness.init(cx);
    let (events, _subscription) = record(&stores, cx);

    harness.select_game(&stores, cx);

    assert_eq!(harness.cli.value(H264_ID), Some(json!(true)));
    assert!(stores.config.read_with(cx, |config, _| config.bool(H264_ID)));
    let applied: Vec<_> = events
        .borrow()
        .iter()
        .filter_map(|event| match event {
            ConfigEvent::CutsceneFlagsApplied(changes) => Some(changes.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0][0].id, H264_ID);
    assert_eq!(applied[0][0].new, json!(true));
}

#[gpui::test]
fn cutscene_detection_leaves_a_matching_flag_alone(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    stores
        .config
        .update(cx, |config, cx| config.auto_detect_cutscenes(cx));
    cx.run_until_parked();

    assert_eq!(harness.cli.count("set"), 0);
    assert!(!stores.config.read_with(cx, |config, _| config.bool(H264_ID)));
}

#[gpui::test]
fn the_overlay_language_is_seeded_only_while_the_file_has_none(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.seed_overlay_language("xx", cx);
    });
    cx.run_until_parked();
    assert_eq!(harness.cli.count("set"), 0);

    stores.config.update(cx, |config, cx| {
        config.seed_overlay_language("de", cx);
    });
    cx.run_until_parked();
    assert_eq!(harness.cli.value(OVERLAY_LANGUAGE_ID), Some(json!("de")));
    assert_eq!(
        stores
            .config
            .read_with(cx, |config, _| config.text(OVERLAY_LANGUAGE_ID)),
        "de"
    );
}

#[gpui::test]
fn a_chosen_overlay_language_is_never_overwritten(cx: &mut TestAppContext) {
    let harness = Harness::with_cli(FakeNamsCli::default().with(OVERLAY_LANGUAGE_ID, json!("en")));
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.seed_overlay_language("de", cx);
    });
    cx.run_until_parked();

    assert_eq!(harness.cli.count("set"), 0);
    assert_eq!(harness.cli.value(OVERLAY_LANGUAGE_ID), Some(json!("en")));
}

#[gpui::test]
fn third_party_switches_reload_the_config_values(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    stores
        .thirdparty
        .update(cx, |thirdparty, cx| thirdparty.refresh(cx));
    cx.run_until_parked();
    let gets = harness.cli.count("get");

    stores.thirdparty.update(cx, |thirdparty, cx| {
        thirdparty.set_enabled(yp_thirdparty::ThirdPartyRuntime::Migoto, false, cx);
    });
    cx.run_until_parked();

    assert!(harness.cli.count("get") > gets);
}

#[gpui::test(iterations = 40)]
fn a_finished_load_shows_its_own_values_even_with_another_job_queued(cx: &mut TestAppContext) {
    let harness = Harness::with_cli(
        FakeNamsCli::default().with(TEXTURE_LOAD_ORDER_ID, json!(["zeta", "alpha", "mid"])),
    );
    let stores = harness.start(cx);

    let seen = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.config, move |config, event, cx| {
            if matches!(event, ConfigEvent::Loaded) {
                sink.borrow_mut()
                    .push(config.read(cx).texts(TEXTURE_LOAD_ORDER_ID));
            }
        })
    });
    stores.config.update(cx, |config, cx| {
        config.load(cx);
        config.reload_if_clean(cx);
        config.reload_if_clean(cx);
    });
    cx.run_until_parked();

    let seen = seen.borrow();
    assert!(!seen.is_empty());
    for order in seen.iter() {
        assert_eq!(order, &["zeta", "alpha", "mid"]);
    }
}
