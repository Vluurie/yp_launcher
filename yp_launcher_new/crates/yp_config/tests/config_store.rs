mod support;

use std::time::{Duration, Instant};

use support::{GameDir, parse, table};
use yp_config::fields::{bindings, lodmod, nams, texture_injection};
use yp_config::state::{AppliedChange, ConfigStore, CutsceneFlags};

fn loaded(game: &GameDir) -> ConfigStore {
    let mut store = ConfigStore::new(game.game.clone(), game.host.clone());
    store.load().unwrap();
    store
}

#[test]
fn load_merges_naiom_values_under_the_mouse_section() {
    let game = GameDir::empty();
    let store = loaded(&game);
    let data = store.data();
    assert!(data.nams_error.is_none());
    let mouse = table(&data.nams_values, "mouse");
    assert_eq!(mouse["sensitivity"].as_float(), Some(2.0));
    assert!(table(mouse, "bindings").contains_key(bindings::STANDARD_JUMP.key()));
    assert_eq!(nams::SENSITIVITY.value_in(&data.nams_values), 2.0);
    assert_eq!(bindings::STANDARD_JUMP.value_in(&data.nams_values), "");
    assert!(!data.has_unsaved_changes);
    assert_eq!(store.watch_paths(), vec![game.game.nams_dir()]);
    assert!(ConfigStore::is_watched_file("lodmod.toml"));
    assert!(!ConfigStore::is_watched_file("profiles.toml"));
}

#[test]
fn cutscene_flags_switch_h264_once_and_report_the_key() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    let changes = store
        .apply_cutscene_flags(CutsceneFlags { needs_h264: true })
        .unwrap();
    assert_eq!(
        changes,
        vec![AppliedChange::Updated {
            section: Some("cutscene"),
            key: "enable_h264"
        }]
    );
    assert_eq!(
        table(&game.nams(), "cutscene")["enable_h264"].as_bool(),
        Some(true)
    );
    assert_eq!(
        table(&store.data().nams_values, "cutscene")["enable_h264"].as_bool(),
        Some(true)
    );
    assert_eq!(store.data().nams_raw, game.raw_nams());
    assert!(
        store
            .apply_cutscene_flags(CutsceneFlags { needs_h264: true })
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        store
            .apply_cutscene_flags(CutsceneFlags::default())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        table(&game.nams(), "cutscene")["enable_h264"].as_bool(),
        Some(false)
    );
}

#[test]
fn update_nams_now_writes_immediately_and_keeps_comments() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    assert!(
        store
            .update_nams_now(
                nams::DISABLE_RESHADE_LOADING.key(),
                toml::Value::Boolean(true),
                None
            )
            .unwrap()
    );
    let raw = game.raw_nams();
    assert!(raw.contains("disable_reshade_loading = true"));
    assert!(raw.starts_with("# NAMS General Config."));
    assert!(!store.data().has_unsaved_changes);
    assert!(nams::DISABLE_RESHADE_LOADING.value_in(&store.data().nams_values));
    store
        .update_nams_now("custom_items", toml::Value::Boolean(false), Some("content"))
        .unwrap();
    assert_eq!(
        table(&game.nams(), "content")["custom_items"].as_bool(),
        Some(false)
    );
}

#[test]
fn naiom_live_updates_wait_for_a_flush() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    store.update_naiom_live(nams::SENSITIVITY.key(), toml::Value::Float(3.5), "mouse");
    store.update_naiom_live(
        bindings::STANDARD_JUMP.key(),
        toml::Value::String("SPACE".into()),
        "mouse.bindings",
    );
    assert!(store.has_pending_naiom());
    assert_eq!(game.naiom()["sensitivity"].as_float(), Some(2.0));
    assert!(store.flush_naiom().unwrap());
    assert!(!store.has_pending_naiom());
    let naiom = game.naiom();
    assert_eq!(naiom["sensitivity"].as_float(), Some(3.5));
    assert_eq!(
        table(&naiom, "bindings")["standard_jump"].as_str(),
        Some("SPACE")
    );
    assert!(
        game.raw_naiom()
            .contains("# Sensitivity multiplier for fix_camera_acceleration.")
    );
    assert!(!store.flush_naiom().unwrap());
}

#[test]
fn loading_flushes_a_pending_naiom_change_first() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    store.update_naiom_live(nams::AIM_MODE.key(), toml::Value::Boolean(true), "mouse");
    store.load().unwrap();
    assert_eq!(game.naiom()["aim_mode"].as_bool(), Some(true));
    assert!(nams::AIM_MODE.value_in(&store.data().nams_values));
}

#[test]
fn enabling_high_grids_live_turns_on_both_culling_fixes() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    store.update_lodmod_live(
        lodmod::HIGH_GRIDS_ENABLED.key(),
        toml::Value::Boolean(true),
        Some(lodmod::HIGH_GRIDS_SECTION),
    );
    let values = &store.data().lodmod_values;
    assert!(lodmod::HIGH_GRIDS_ENABLED.value_in(values));
    assert!(lodmod::DISABLE_MANUAL_CULLING.value_in(values));
    assert!(lodmod::DISABLE_CAMERA_CULLING.value_in(values));
    assert!(store.has_pending_lodmod());
    assert!(store.flush_lodmod().unwrap());
    let written = parse(&game.read_nams_file("lodmod.toml"));
    assert_eq!(written["disable_manual_culling"].as_bool(), Some(true));
    assert_eq!(
        table(&written, "high_grids")["enabled"].as_bool(),
        Some(true)
    );

    store.update_lodmod_live(
        lodmod::HIGH_GRIDS_ENABLED.key(),
        toml::Value::Boolean(false),
        Some(lodmod::HIGH_GRIDS_SECTION),
    );
    let values = &store.data().lodmod_values;
    assert!(!lodmod::DISABLE_MANUAL_CULLING.value_in(values));
    assert!(
        lodmod::DISABLE_CAMERA_CULLING.value_in(values),
        "camera culling stays on"
    );
}

#[test]
fn bloom_preset_and_reset_write_through() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    assert!(store.apply_lodmod_bloom_2017_preset().unwrap());
    let written = parse(&game.read_nams_file("lodmod.toml"));
    assert_eq!(
        written["bloom_reference_height"].as_integer(),
        Some(lodmod::BLOOM_2017_REFERENCE_HEIGHT)
    );

    store.update_lodmod(lodmod::FXAA.key(), toml::Value::Boolean(true), None);
    assert!(store.reset_lodmod_to_defaults().unwrap());
    let written = parse(&game.read_nams_file("lodmod.toml"));
    for field in lodmod::ALL {
        let meta = field.meta();
        let scope = match meta.section {
            Some(section) => table(&written, section),
            None => &written,
        };
        assert_eq!(
            scope.get(meta.key),
            Some(&field.default_toml()),
            "{}",
            meta.key
        );
    }
}

#[test]
fn save_reports_a_default_outfits_switch() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    store.update_texture_injection(
        texture_injection::HOT_RELOAD.key(),
        toml::Value::Boolean(false),
    );
    assert!(store.data().has_unsaved_changes);
    let report = store.save().unwrap();
    assert!(!report.default_outfits_changed);
    assert!(!store.data().has_unsaved_changes);
    assert_eq!(
        parse(&game.read_nams_file("texture_injection.toml"))["hot_reload"].as_bool(),
        Some(false)
    );

    store.update_nams(
        nams::EXPERIMENTAL_DEFAULT_OUTFITS.key(),
        toml::Value::Boolean(false),
        None,
    );
    let report = store.save().unwrap();
    assert!(report.default_outfits_changed);
    assert_eq!(
        game.nams()["experimental_default_outfits"].as_bool(),
        Some(false)
    );
    assert!(!store.save().unwrap().default_outfits_changed);
}

#[test]
fn naiom_values_saved_through_the_mouse_section_go_to_naiom_toml() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    store.update_nams(
        nams::FIX_CAMERA_ACCELERATION.key(),
        toml::Value::Boolean(true),
        Some("mouse"),
    );
    store.save().unwrap();
    assert_eq!(
        game.naiom()["fix_camera_acceleration"].as_bool(),
        Some(true)
    );
    assert!(!game.nams().contains_key("mouse"));
}

#[test]
fn silent_texture_updates_do_not_mark_changes() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    store.update_texture_injection_silent(
        texture_injection::LOAD_ORDER.key(),
        toml::Value::Array(vec!["A".into()]),
    );
    assert!(!store.data().has_unsaved_changes);
    assert_eq!(
        texture_injection::LOAD_ORDER.value_in(&store.data().texture_injection_values),
        vec!["A".to_owned()]
    );
}

#[test]
fn external_reloads_wait_for_clean_state_and_a_quiet_period() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    let now = Instant::now();
    assert!(store.should_reload_external(now));

    store.update_nams(
        nams::FIX_WIND_TIMER_BUG.key(),
        toml::Value::Boolean(false),
        None,
    );
    assert!(!store.should_reload_external(now));
    assert!(!store.reload_external(now).unwrap());
    store.save().unwrap();

    let after_write = Instant::now();
    assert!(!store.should_reload_external(after_write));
    assert!(store.should_reload_external(after_write + Duration::from_secs(3)));

    game.write_nams_file("naiom.toml", "sensitivity = 7.0\n");
    assert!(
        store
            .reload_external(after_write + Duration::from_secs(3))
            .unwrap()
    );
    assert_eq!(nams::SENSITIVITY.value_in(&store.data().nams_values), 7.0);
}

#[test]
fn discard_changes_reloads_from_disk() {
    let game = GameDir::empty();
    let mut store = loaded(&game);
    store.update_lodmod(lodmod::FXAA.key(), toml::Value::Boolean(true), None);
    store.discard_changes().unwrap();
    assert!(!store.data().has_unsaved_changes);
    assert!(!lodmod::FXAA.value_in(&store.data().lodmod_values));
}
