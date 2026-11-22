use std::path::Path;

use yp_core::window_geometry::{
    WindowGeometry, DEFAULT_HEIGHT, DEFAULT_WIDTH, MAX_EXTENT, MIN_HEIGHT, MIN_WIDTH, WINDOW,
};
use yp_core::{OpenReport, SettingKey, SettingsStore};

const APP_THEME: SettingKey<String> = SettingKey::new("app_theme", || "dark".to_owned());

#[test]
fn a_failed_write_does_not_cache_the_value_or_prevent_retrying_it() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("settings.json");
    let (store, _) = SettingsStore::open(file.clone()).unwrap();
    std::fs::create_dir(&file).unwrap();
    assert!(store.set(&APP_THEME, &"nier".to_owned()).is_err());
    assert_eq!(store.get(&APP_THEME), "dark");
    std::fs::remove_dir(&file).unwrap();
    store.set(&APP_THEME, &"nier".to_owned()).unwrap();
    assert_eq!(SettingsStore::open(file).unwrap().0.get(&APP_THEME), "nier");
}

fn open(dir: &Path) -> (SettingsStore, OpenReport) {
    SettingsStore::open(dir.join("settings.json")).unwrap()
}

#[test]
fn recovers_from_every_corruption_and_keeps_a_backup() {
    let corruptions: [(&str, Vec<u8>); 3] = [
        ("null bytes after a killed write", vec![0u8; 512]),
        ("truncated json", b"{\"app_theme\":\"ni".to_vec()),
        ("invalid utf8", vec![0x7B, 0xC3, 0x28, 0xFF, 0xFE]),
    ];
    for (label, bytes) in corruptions {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        let backup = dir.path().join("settings.corrupt.json");
        std::fs::write(&file, &bytes).unwrap();

        let (store, report) = open(dir.path());

        assert_eq!(
            report,
            OpenReport::RecoveredCorrupt { backup: backup.clone() },
            "{label}"
        );
        assert!(!file.exists(), "{label}");
        assert_eq!(std::fs::read(&backup).unwrap(), bytes, "{label}");
        assert_eq!(store.get(&APP_THEME), "dark", "{label}");
        store.set(&APP_THEME, &"nier".to_owned()).unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        assert_eq!(saved["app_theme"], "nier", "{label}");
    }
}

#[test]
fn leaves_a_valid_file_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.json");
    let content = "{\"app_theme\":\"nier\",\"window\":{\"width\":1600.0,\"height\":900.0}}";
    std::fs::write(&file, content).unwrap();
    let (store, report) = open(dir.path());
    assert_eq!(report, OpenReport::Loaded);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), content);
    assert!(!dir.path().join("settings.corrupt.json").exists());
    assert_eq!(store.get(&APP_THEME), "nier");
    assert_eq!(store.get(&WINDOW).unwrap().width, 1600.0);
}

#[test]
fn does_nothing_on_first_launch_without_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let (store, report) = open(dir.path());
    assert_eq!(report, OpenReport::Created);
    assert!(!dir.path().join("settings.json").exists());
    assert!(!dir.path().join("settings.corrupt.json").exists());
    assert_eq!(store.get(&WINDOW), None);
}

#[test]
fn overwrites_an_older_backup() {
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("settings.corrupt.json");
    std::fs::write(&backup, "old").unwrap();
    std::fs::write(dir.path().join("settings.json"), "{\"x\":").unwrap();
    let _ = open(dir.path());
    assert_eq!(std::fs::read_to_string(&backup).unwrap(), "{\"x\":");
    assert!(!dir.path().join("settings.json").exists());
}

fn imported_window(legacy: &str) -> Option<WindowGeometry> {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("shared_preferences.json");
    std::fs::write(&file, legacy).unwrap();
    let store = SettingsStore::in_memory();
    store.import_flutter_prefs(&file).unwrap();
    store.get(&WINDOW)
}

fn saved_size(geometry: Option<WindowGeometry>) -> (f64, f64) {
    let geometry = geometry.unwrap_or_default().sanitize(&[]);
    (geometry.width, geometry.height)
}

#[test]
fn saved_size_falls_back_to_the_default_when_nothing_is_stored() {
    assert_eq!(saved_size(imported_window("{}")), (DEFAULT_WIDTH, DEFAULT_HEIGHT));
}

#[test]
fn saved_size_returns_what_was_stored() {
    let window = imported_window("{\"flutter.window_width\":1600.0,\"flutter.window_height\":900.0}");
    assert_eq!(saved_size(window), (1600.0, 900.0));
}

#[test]
fn saved_size_clamps_below_the_minimum() {
    let window = imported_window("{\"flutter.window_width\":120.0,\"flutter.window_height\":80.0}");
    assert_eq!(saved_size(window), (MIN_WIDTH, MIN_HEIGHT));
}

#[test]
fn saved_size_clamps_an_absurd_size() {
    let window = imported_window("{\"flutter.window_width\":999999.0,\"flutter.window_height\":999999.0}");
    let (width, height) = saved_size(window);
    assert!(width <= MAX_EXTENT && height <= MAX_EXTENT);
}

#[test]
fn saved_size_ignores_a_half_written_entry() {
    let window = imported_window("{\"flutter.window_width\":1600.0}");
    assert_eq!(saved_size(window), (DEFAULT_WIDTH, DEFAULT_HEIGHT));
}

#[test]
fn saved_position_is_none_when_nothing_is_stored_and_kept_otherwise() {
    let none = imported_window("{\"flutter.window_maximized\":true}").unwrap();
    assert_eq!((none.x, none.y), (None, None));
    let stored = imported_window("{\"flutter.window_x\":240.0,\"flutter.window_y\":120.0}")
        .unwrap()
        .sanitize(&[]);
    assert_eq!((stored.x, stored.y), (Some(240.0), Some(120.0)));
    let half = imported_window("{\"flutter.window_x\":240.0}").unwrap();
    assert_eq!((half.x, half.y), (None, None));
}

#[test]
fn maximized_and_fullscreen_default_false_and_are_independent() {
    let nothing = imported_window("{}").unwrap_or_default();
    assert!(!nothing.maximized && !nothing.fullscreen);
    assert!(
        imported_window("{\"flutter.window_maximized\":true}")
            .unwrap()
            .maximized
    );
    let full = imported_window("{\"flutter.window_fullscreen\":true,\"flutter.window_maximized\":false}").unwrap();
    assert!(full.fullscreen);
    assert!(!full.maximized);
}

#[test]
fn clear_removes_every_stored_key() {
    let store = SettingsStore::in_memory();
    store
        .set(
            &WINDOW,
            &Some(WindowGeometry {
                width: 1600.0,
                height: 900.0,
                x: Some(10.0),
                y: Some(20.0),
                maximized: true,
                fullscreen: true,
            }),
        )
        .unwrap();
    assert!(store.remove(&WINDOW).unwrap());
    let cleared = store.get(&WINDOW).unwrap_or_default();
    assert_eq!(cleared, WindowGeometry::default());
}
