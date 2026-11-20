use std::time::{Duration, Instant};

use serde_json::{Value, json};
use yp_config::ConfigError;
use yp_config::overlay::{
    self, default_settings, load_impeller_quiet, load_settings, resolve_settings_path,
    save_impeller_quiet, save_settings, seed_overlay_language_quiet,
};
use yp_config::state::OverlayStore;
use yp_core::GamePaths;

fn game() -> (tempfile::TempDir, GamePaths) {
    let dir = tempfile::tempdir().unwrap();
    let game = GamePaths::new(dir.path().join("game"));
    (dir, game)
}

fn read_json(game: &GamePaths) -> Value {
    let path = resolve_settings_path(Some(game)).unwrap();
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn the_settings_live_in_the_nams_internal_cache() {
    let (_dir, game) = game();
    assert_eq!(
        resolve_settings_path(Some(&game)),
        Some(
            game.root()
                .join("nams")
                .join("_internal")
                .join("cache")
                .join("settings.json")
        )
    );
    assert_eq!(resolve_settings_path(None), None);
    assert_eq!(resolve_settings_path(Some(&GamePaths::new(""))), None);
}

#[test]
fn missing_or_blank_files_load_the_overlay_defaults() {
    let (_dir, game) = game();
    assert_eq!(load_settings(Some(&game)).unwrap(), default_settings());
    assert_eq!(load_settings(None).unwrap(), default_settings());
    let path = resolve_settings_path(Some(&game)).unwrap();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "  \n").unwrap();
    assert_eq!(load_settings(Some(&game)).unwrap(), default_settings());
    let defaults = Value::Object(default_settings());
    assert_eq!(defaults["keybinds"]["yorha_protocol"]["dev_mode"], "Insert");
    assert_eq!(defaults["randomizerConfig"]["excludedEnemies"], json!([]));
    assert_eq!(defaults["loadingSpeedupEnabled"], Value::Bool(true));
}

#[test]
fn invalid_or_non_object_json_is_an_error() {
    let (_dir, game) = game();
    let path = resolve_settings_path(Some(&game)).unwrap();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{ nope").unwrap();
    assert!(load_settings(Some(&game)).is_err());
    std::fs::write(&path, "[1, 2]").unwrap();
    assert!(matches!(
        load_settings(Some(&game)),
        Err(ConfigError::SettingsNotObject { .. })
    ));
    assert!(!load_impeller_quiet(Some(&game)));
    assert!(!save_impeller_quiet(Some(&game), true));
}

#[test]
fn saving_creates_the_folder_and_writes_two_space_json() {
    let (_dir, game) = game();
    let mut settings = default_settings();
    settings.insert("soundEnabled".into(), Value::Bool(false));
    assert!(save_settings(&settings, Some(&game)).unwrap());
    let raw = std::fs::read_to_string(resolve_settings_path(Some(&game)).unwrap()).unwrap();
    assert!(raw.starts_with("{\n  \"firstPlaythrough\": true,\n  \"impeller\": false,"));
    assert_eq!(read_json(&game)["soundEnabled"], Value::Bool(false));
    assert!(!save_settings(&settings, None).unwrap());
}

#[test]
fn the_overlay_language_is_only_seeded_once() {
    let (_dir, game) = game();
    assert!(seed_overlay_language_quiet(Some(&game), "de"));
    assert_eq!(read_json(&game)[overlay::OVERLAY_LANGUAGE_KEY], "de");
    assert!(!seed_overlay_language_quiet(Some(&game), "zh"));
    assert_eq!(read_json(&game)[overlay::OVERLAY_LANGUAGE_KEY], "de");
    assert!(!seed_overlay_language_quiet(None, "zh"));
}

#[test]
fn impeller_round_trips_quietly() {
    let (_dir, game) = game();
    assert!(!load_impeller_quiet(Some(&game)));
    assert!(save_impeller_quiet(Some(&game), true));
    assert!(load_impeller_quiet(Some(&game)));
    assert!(!save_impeller_quiet(None, true));
}

#[test]
fn the_store_edits_nested_settings_and_saves_them() {
    let (_dir, game) = game();
    let mut store = OverlayStore::new(Some(game.clone()));
    store.load().unwrap();
    assert_eq!(
        store.data().settings_path,
        resolve_settings_path(Some(&game))
    );
    assert!(!store.data().impeller());

    store.update_toggle("shadersEnabled", false);
    store.update_cheat("damageMultiplier", json!(2.5));
    store.update_randomizer("allowBigEnemies", json!(false));
    store.set_default_outfit("2B", Some("Fancy"));
    store.set_default_outfit("9S", Some("Shorts"));
    store.set_default_outfit("9S", None);
    assert!(store.data().has_unsaved_changes);
    assert_eq!(
        store.data().default_outfit_for("2B").as_deref(),
        Some("Fancy")
    );
    assert_eq!(store.data().default_outfit_for("9S"), None);

    assert!(store.save().unwrap());
    assert!(!store.data().has_unsaved_changes);
    let saved = read_json(&game);
    assert_eq!(saved["shadersEnabled"], Value::Bool(false));
    assert_eq!(saved["cheatsConfig"]["damageMultiplier"], json!(2.5));
    assert_eq!(
        saved["randomizerConfig"]["allowBigEnemies"],
        Value::Bool(false)
    );
    assert_eq!(
        saved["randomizerConfig"]["includeDlcEnemies"],
        Value::Bool(true)
    );
    assert_eq!(saved["defaultOutfits"], json!({"2B": "Fancy"}));

    store.set_default_outfit("2B", None);
    assert!(!store.data().settings.contains_key("defaultOutfits"));
}

#[test]
fn the_store_without_a_game_cannot_save() {
    let mut store = OverlayStore::new(None);
    store.load().unwrap();
    store.update_toggle("soundEnabled", false);
    assert!(!store.save().unwrap());
    assert!(store.data().has_unsaved_changes);
    assert_eq!(store.watch_path(), None);
}

#[test]
fn external_reloads_respect_unsaved_changes_and_self_writes() {
    let (_dir, game) = game();
    let mut store = OverlayStore::new(Some(game.clone()));
    store.load().unwrap();
    store.update_toggle("soundEnabled", false);
    assert!(!store.reload_external(Instant::now()).unwrap());
    assert!(!store.reload_if_clean().unwrap());
    store.save().unwrap();
    let written = Instant::now();
    assert!(!store.should_reload_external(written));

    let path = resolve_settings_path(Some(&game)).unwrap();
    std::fs::write(&path, "{\"soundEnabled\": true, \"impeller\": true}").unwrap();
    assert!(
        store
            .reload_external(written + Duration::from_secs(5))
            .unwrap()
    );
    assert!(store.data().impeller());
    assert!(store.reload_if_clean().unwrap());
    store.update_keybind("main", "yorha_protocol", "F4");
    store.discard_changes().unwrap();
    assert!(!store.data().has_unsaved_changes);
    assert_eq!(store.data().keybinds(), serde_json::Map::new());
}
