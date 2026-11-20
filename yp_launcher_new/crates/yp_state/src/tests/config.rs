use std::time::Duration;

use gpui::TestAppContext;
use yp_config::ConfigFile;
use yp_config::fields::{lodmod, nams};

use super::{Harness, settle};
use crate::config::{LODMOD_AUTOSAVE_DELAY, NAIOM_AUTOSAVE_DELAY, NAIOM_SECTION};
use crate::nams_settings::NamsSettingsEvent;
use crate::{ConfigEvent, Stores};

const SENSITIVITY_KEY: &str = "sensitivity";
const RELOAD_DELAY: Duration = Duration::from_millis(400);

fn read(harness: &Harness, file: ConfigFile) -> toml::Table {
    let raw = std::fs::read_to_string(file.path(&harness.game)).unwrap();
    yp_core::toml_io::parse_values(&raw).unwrap()
}

fn sensitivity(stores: &Stores, cx: &mut TestAppContext) -> Option<f64> {
    stores.config.read_with(cx, |config, _| {
        config
            .naiom()
            .get(SENSITIVITY_KEY)
            .and_then(toml::Value::as_float)
    })
}

#[gpui::test]
fn selecting_a_game_creates_and_loads_the_configs(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    for file in ConfigFile::ALL {
        assert!(file.path(&harness.game).is_file());
    }
    stores.config.read_with(cx, |config, _| {
        assert!(config.is_loaded());
        assert!(!config.is_loading());
        assert!(!config.is_dirty());
        assert!(config.failure().is_none());
        assert!(!config.nams().is_empty());
        assert!(!config.lodmod().is_empty());
        assert!(!config.raw(ConfigFile::Naiom).is_empty());
        assert_eq!(config.game(), Some(&harness.game));
    });
    assert_eq!(sensitivity(&stores, cx), Some(2.0));
}

#[gpui::test]
fn edits_mark_dirty_and_save_writes_them(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let key = nams::DISABLE_PLUGIN_LOADING.key();

    stores.config.update(cx, |config, cx| {
        config.update_nams(key, toml::Value::Boolean(true), None, cx);
    });
    stores.config.read_with(cx, |config, _| {
        assert!(config.is_dirty());
        assert!(nams::DISABLE_PLUGIN_LOADING.value_in(config.nams()));
    });
    assert!(!nams::DISABLE_PLUGIN_LOADING.value_in(&read(&harness, ConfigFile::Nams)));

    let saved = std::rc::Rc::new(std::cell::Cell::new(0));
    let seen = saved.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.config, move |_, event, _| {
            if matches!(event, ConfigEvent::Saved(_)) {
                seen.set(seen.get() + 1);
            }
        })
    });
    stores.config.update(cx, |config, cx| config.save(cx));
    cx.run_until_parked();

    assert_eq!(saved.get(), 1);
    assert!(!stores.config.read_with(cx, |config, _| config.is_dirty()));
    assert!(nams::DISABLE_PLUGIN_LOADING.value_in(&read(&harness, ConfigFile::Nams)));
}

#[gpui::test]
fn discard_restores_the_file_values(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let key = nams::DISABLE_PLUGIN_LOADING.key();

    stores.config.update(cx, |config, cx| {
        config.update_nams(key, toml::Value::Boolean(true), None, cx);
        config.discard(cx);
    });
    cx.run_until_parked();

    stores.config.read_with(cx, |config, _| {
        assert!(!config.is_dirty());
        assert!(!nams::DISABLE_PLUGIN_LOADING.value_in(config.nams()));
    });
}

#[gpui::test]
fn naiom_edits_save_themselves_after_the_delay(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    stores.config.update(cx, |config, cx| {
        config.update_naiom(SENSITIVITY_KEY, toml::Value::Float(3.5), NAIOM_SECTION, cx);
    });
    cx.run_until_parked();
    assert_eq!(sensitivity(&stores, cx), Some(3.5));
    assert!(!stores.config.read_with(cx, |config, _| config.is_dirty()));
    assert!(
        stores
            .config
            .read_with(cx, |config, _| config.has_pending_autosave())
    );
    assert_eq!(
        read(&harness, ConfigFile::Naiom)
            .get(SENSITIVITY_KEY)
            .and_then(toml::Value::as_float),
        Some(2.0)
    );

    settle(cx, NAIOM_AUTOSAVE_DELAY);

    assert_eq!(
        read(&harness, ConfigFile::Naiom)
            .get(SENSITIVITY_KEY)
            .and_then(toml::Value::as_float),
        Some(3.5)
    );
    assert!(
        !stores
            .config
            .read_with(cx, |config, _| config.has_pending_autosave())
    );
}

#[gpui::test]
fn rapid_naiom_edits_are_written_once_with_the_last_value(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let writes = std::rc::Rc::new(std::cell::Cell::new(0));
    let seen = writes.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.config, move |_, event, _| {
            if matches!(event, ConfigEvent::Written(ConfigFile::Naiom)) {
                seen.set(seen.get() + 1);
            }
        })
    });

    for value in [2.5, 3.0, 4.0] {
        stores.config.update(cx, |config, cx| {
            config.update_naiom(
                SENSITIVITY_KEY,
                toml::Value::Float(value),
                NAIOM_SECTION,
                cx,
            );
        });
        settle(cx, NAIOM_AUTOSAVE_DELAY / 2);
    }
    settle(cx, NAIOM_AUTOSAVE_DELAY);

    assert_eq!(writes.get(), 1);
    assert_eq!(
        read(&harness, ConfigFile::Naiom)
            .get(SENSITIVITY_KEY)
            .and_then(toml::Value::as_float),
        Some(4.0)
    );
}

#[gpui::test]
fn lodmod_live_edits_save_themselves(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let key = lodmod::ENABLED.key();

    stores.config.update(cx, |config, cx| {
        config.update_lodmod_live(key, toml::Value::Boolean(true), None, cx);
    });
    cx.run_until_parked();
    assert!(!lodmod::ENABLED.value_in(&read(&harness, ConfigFile::LodMod)));

    settle(cx, LODMOD_AUTOSAVE_DELAY);

    assert!(lodmod::ENABLED.value_in(&read(&harness, ConfigFile::LodMod)));
    stores.config.read_with(cx, |config, _| {
        assert!(lodmod::ENABLED.value_in(config.lodmod()));
        assert!(!config.is_dirty());
    });
}

#[gpui::test]
fn pending_autosaves_can_be_flushed_at_once(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);

    let flush = stores.config.update(cx, |config, cx| {
        config.update_naiom(SENSITIVITY_KEY, toml::Value::Float(5.0), NAIOM_SECTION, cx);
        config.flush_pending(cx)
    });
    cx.run_until_parked();
    cx.foreground_executor().block_test(flush);

    assert_eq!(
        read(&harness, ConfigFile::Naiom)
            .get(SENSITIVITY_KEY)
            .and_then(toml::Value::as_float),
        Some(5.0)
    );
}

#[gpui::test]
fn reset_lodmod_writes_the_defaults(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let key = lodmod::ENABLED.key();

    stores.config.update(cx, |config, cx| {
        config.update_lodmod_live(key, toml::Value::Boolean(true), None, cx);
    });
    settle(cx, LODMOD_AUTOSAVE_DELAY);
    assert!(lodmod::ENABLED.value_in(&read(&harness, ConfigFile::LodMod)));

    stores
        .config
        .update(cx, |config, cx| config.reset_lodmod_defaults(cx));
    cx.run_until_parked();

    assert!(!lodmod::ENABLED.value_in(&read(&harness, ConfigFile::LodMod)));
    assert!(
        !stores
            .config
            .read_with(cx, |config, _| lodmod::ENABLED.value_in(config.lodmod()))
    );
}

#[gpui::test]
fn external_edits_are_reloaded_after_the_debounce(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let file = ConfigFile::Naiom.path(&harness.game);
    let edited = std::fs::read_to_string(&file)
        .unwrap()
        .replace("sensitivity = 2.0", "sensitivity = 7.0");
    std::fs::write(&file, edited).unwrap();

    stores.config.update(cx, |config, cx| {
        config.on_external_change(std::slice::from_ref(&file), cx);
    });
    cx.run_until_parked();
    assert_eq!(sensitivity(&stores, cx), Some(2.0));

    settle(cx, RELOAD_DELAY);
    assert_eq!(sensitivity(&stores, cx), Some(7.0));
}

#[gpui::test]
fn external_edits_never_overwrite_unsaved_changes(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let file = ConfigFile::Naiom.path(&harness.game);
    stores.config.update(cx, |config, cx| {
        config.update_nams(
            nams::DISABLE_PLUGIN_LOADING.key(),
            toml::Value::Boolean(true),
            None,
            cx,
        );
    });
    let edited = std::fs::read_to_string(&file)
        .unwrap()
        .replace("sensitivity = 2.0", "sensitivity = 7.0");
    std::fs::write(&file, edited).unwrap();

    stores.config.update(cx, |config, cx| {
        config.on_external_change(std::slice::from_ref(&file), cx);
    });
    settle(cx, RELOAD_DELAY);

    assert_eq!(sensitivity(&stores, cx), Some(2.0));
    assert!(stores.config.read_with(cx, |config, _| config.is_dirty()));
}

#[gpui::test]
fn own_writes_do_not_trigger_a_reload(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let file = ConfigFile::Naiom.path(&harness.game);
    let reloads = std::rc::Rc::new(std::cell::Cell::new(0));
    let seen = reloads.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.config, move |_, event, _| {
            if matches!(event, ConfigEvent::ReloadedExternally) {
                seen.set(seen.get() + 1);
            }
        })
    });

    stores.config.update(cx, |config, cx| {
        config.update_naiom(SENSITIVITY_KEY, toml::Value::Float(3.0), NAIOM_SECTION, cx);
    });
    settle(cx, NAIOM_AUTOSAVE_DELAY);
    stores.config.update(cx, |config, cx| {
        config.on_external_change(std::slice::from_ref(&file), cx);
    });
    settle(cx, RELOAD_DELAY);

    assert_eq!(reloads.get(), 0);
    assert_eq!(sensitivity(&stores, cx), Some(3.0));
}

#[gpui::test]
fn unrelated_files_in_the_config_folder_are_ignored(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let file = ConfigFile::Naiom.path(&harness.game);
    let edited = std::fs::read_to_string(&file)
        .unwrap()
        .replace("sensitivity = 2.0", "sensitivity = 7.0");
    std::fs::write(&file, edited).unwrap();
    let other = harness.game.nams_dir().join("notes.txt");

    stores.config.update(cx, |config, cx| {
        config.on_external_change(std::slice::from_ref(&other), cx);
    });
    settle(cx, RELOAD_DELAY);

    assert_eq!(sensitivity(&stores, cx), Some(2.0));
}

#[gpui::test]
fn broken_toml_is_reported_per_file(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    std::fs::create_dir_all(harness.game.nams_dir()).unwrap();
    std::fs::write(ConfigFile::LodMod.path(&harness.game), "enabled = = true").unwrap();

    harness.select_game(&stores, cx);

    stores.config.read_with(cx, |config, _| {
        assert!(config.is_loaded());
        assert!(config.error(ConfigFile::LodMod).is_some());
        assert!(config.error(ConfigFile::Nams).is_none());
    });
}

#[gpui::test]
fn overlay_settings_edit_save_and_reload(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let saved = std::rc::Rc::new(std::cell::Cell::new(false));
    let seen = saved.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.nams_settings, move |_, event, _| {
            if matches!(event, NamsSettingsEvent::Saved { written: true }) {
                seen.set(true);
            }
        })
    });

    let path = stores
        .nams_settings
        .read_with(cx, |settings, _| {
            settings.path().map(|path| path.to_path_buf())
        })
        .unwrap();
    stores.nams_settings.update(cx, |settings, cx| {
        settings.update_toggle("impeller", true, cx);
        settings.update_keybind("overlay", "toggle", "F9", cx);
    });
    stores.nams_settings.read_with(cx, |settings, _| {
        assert!(settings.is_dirty());
        assert!(settings.impeller());
    });

    stores
        .nams_settings
        .update(cx, |settings, cx| settings.save(cx));
    cx.run_until_parked();

    assert!(saved.get());
    assert!(
        !stores
            .nams_settings
            .read_with(cx, |settings, _| settings.is_dirty())
    );
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(written["impeller"], serde_json::Value::Bool(true));
    assert_eq!(written["keybinds"]["overlay"]["toggle"], "F9");

    let mut edited = written.clone();
    edited["impeller"] = serde_json::Value::Bool(false);
    std::fs::write(&path, serde_json::to_string_pretty(&edited).unwrap()).unwrap();
    stores
        .nams_settings
        .update(cx, |settings, cx| settings.reload_if_clean(cx));
    cx.run_until_parked();
    assert!(
        !stores
            .nams_settings
            .read_with(cx, |settings, _| settings.impeller())
    );
}

#[gpui::test]
fn the_file_watcher_picks_up_outside_edits(cx: &mut TestAppContext) {
    cx.executor().allow_parking();
    let mut harness = Harness::new();
    harness.env = harness.env.clone().with_file_watching(true);
    let stores = harness.start(cx);
    let file = ConfigFile::Naiom.path(&harness.game);
    let edited = std::fs::read_to_string(&file)
        .unwrap()
        .replace("sensitivity = 2.0", "sensitivity = 7.0");

    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let mut written = false;
    while sensitivity(&stores, cx) != Some(7.0) {
        assert!(
            std::time::Instant::now() < deadline,
            "the watcher never reported the edit"
        );
        if !written {
            std::thread::sleep(Duration::from_millis(200));
            std::fs::write(&file, &edited).unwrap();
            written = true;
        }
        std::thread::sleep(Duration::from_millis(20));
        settle(cx, RELOAD_DELAY);
    }
}

#[gpui::test]
fn legacy_lodmod_settings_show_up_after_the_startup_migration(cx: &mut TestAppContext) {
    let harness = Harness::new();
    std::fs::write(
        harness.game.root().join("LodMod.ini"),
        "[LodMod]\nLODMultiplier=4\n",
    )
    .unwrap();
    let stores = harness.start(cx);

    assert!(stores.notifications.read_with(cx, |notifications, _| {
        notifications.contains("lodmod_migration")
    }));
    assert!(lodmod::ENABLED.value_in(&read(&harness, ConfigFile::LodMod)));
    stores.config.read_with(cx, |config, _| {
        assert!(lodmod::ENABLED.value_in(config.lodmod()));
        assert!(!config.is_dirty());
    });
    assert!(stores.detection.read_with(cx, |detection, _| {
        detection.snapshot().unwrap().lodmod_enabled
    }));
}

#[gpui::test]
fn third_party_switches_reach_the_config_values(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    assert!(!stores.config.read_with(cx, |config, _| {
        nams::DISABLE_3DMIGOTO_LOADING.value_in(config.nams())
    }));
    stores
        .thirdparty
        .update(cx, |thirdparty, cx| thirdparty.refresh(cx));
    cx.run_until_parked();

    stores.thirdparty.update(cx, |thirdparty, cx| {
        thirdparty.set_enabled(yp_thirdparty::ThirdPartyRuntime::Migoto, false, cx);
    });
    cx.run_until_parked();

    assert!(stores.config.read_with(cx, |config, _| {
        nams::DISABLE_3DMIGOTO_LOADING.value_in(config.nams())
    }));
}

fn texture_load_order(config: &crate::ConfigStore) -> Vec<String> {
    config
        .texture_injection()
        .get("load_order")
        .and_then(toml::Value::as_array)
        .map(|packs| {
            packs
                .iter()
                .filter_map(|pack| pack.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[gpui::test(iterations = 40)]
fn a_finished_load_shows_its_own_values_even_with_another_job_queued(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let file = ConfigFile::TextureInjection.path(&harness.game);
    let text = std::fs::read_to_string(&file).unwrap();
    let mut doc: toml::Table = text.parse().unwrap();
    doc.insert(
        "load_order".to_owned(),
        toml::Value::Array(vec!["zeta".into(), "alpha".into(), "mid".into()]),
    );
    std::fs::write(&file, toml::to_string(&doc).unwrap()).unwrap();

    let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let sink = seen.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.config, move |config, event, cx| {
            if matches!(event, ConfigEvent::Loaded) {
                sink.borrow_mut()
                    .push(texture_load_order(config.read(cx)));
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
