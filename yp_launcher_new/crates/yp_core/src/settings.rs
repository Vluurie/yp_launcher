use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::error::{Error, IoOp, IoResultExt, Result};

const LEGACY_PREFIX: &str = "flutter.";
const IMPORTED_MARKER: &str = "flutter_prefs_imported";
const SIDEBAR_PREFIX: &str = "sidebar_section_collapsed_";
const WIN10_SETUP_PREFIX: &str = "win10setup_done_";
const WINDOW_PREFIX: &str = "window_";
const WINDOW_FIELDS: [&str; 6] = ["width", "height", "x", "y", "maximized", "fullscreen"];

pub struct SettingKey<T: 'static> {
    name: &'static str,
    default: fn() -> T,
    marker: PhantomData<fn() -> T>,
}

impl<T> SettingKey<T> {
    pub const fn new(name: &'static str, default: fn() -> T) -> Self {
        Self {
            name,
            default,
            marker: PhantomData,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub fn default_value(&self) -> T {
        (self.default)()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenReport {
    Loaded,
    Created,
    RecoveredCorrupt { backup: PathBuf },
}

#[derive(Debug)]
pub struct SettingsStore {
    file: Option<PathBuf>,
    values: Mutex<Map<String, Value>>,
}

impl SettingsStore {
    pub fn open(file: PathBuf) -> Result<(Self, OpenReport)> {
        let (values, report) = match std::fs::read(&file) {
            Ok(bytes) => match parse_object(&bytes) {
                Some(values) => (values, OpenReport::Loaded),
                None => {
                    let backup = backup_path(&file);
                    std::fs::copy(&file, &backup).io(IoOp::Copy, &backup)?;
                    std::fs::remove_file(&file).io(IoOp::Remove, &file)?;
                    log::warn!("settings file {} was corrupt, moved to {}", file.display(), backup.display());
                    (Map::new(), OpenReport::RecoveredCorrupt { backup })
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => (Map::new(), OpenReport::Created),
            Err(err) => return Err(err).io(IoOp::Read, &file),
        };
        Ok((
            Self {
                file: Some(file),
                values: Mutex::new(values),
            },
            report,
        ))
    }

    pub fn in_memory() -> Self {
        Self {
            file: None,
            values: Mutex::new(Map::new()),
        }
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    pub fn get<T: DeserializeOwned>(&self, key: &SettingKey<T>) -> T {
        let values = self.lock();
        values
            .get(key.name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or_else(|| key.default_value())
    }

    pub fn contains<T>(&self, key: &SettingKey<T>) -> bool {
        self.lock().contains_key(key.name)
    }

    pub fn set<T: Serialize>(&self, key: &SettingKey<T>, value: &T) -> Result<()> {
        let value = serde_json::to_value(value).map_err(|source| Error::Json { path: None, source })?;
        let mut values = self.lock();
        if values.get(key.name) == Some(&value) {
            return Ok(());
        }
        let previous = values.insert(key.name.to_owned(), value);
        if let Err(error) = self.persist(&values) {
            match previous {
                Some(previous) => {
                    values.insert(key.name.to_owned(), previous);
                }
                None => {
                    values.shift_remove(key.name);
                }
            }
            return Err(error);
        }
        Ok(())
    }

    pub fn update<T: Serialize + DeserializeOwned>(&self, key: &SettingKey<T>, f: impl FnOnce(&mut T)) -> Result<()> {
        let mut current = self.get(key);
        f(&mut current);
        self.set(key, &current)
    }

    pub fn remove<T>(&self, key: &SettingKey<T>) -> Result<bool> {
        let mut values = self.lock();
        if values.shift_remove(key.name).is_none() {
            return Ok(false);
        }
        self.persist(&values)?;
        Ok(true)
    }

    pub fn import_flutter_prefs(&self, legacy_file: &Path) -> Result<usize> {
        let mut values = self.lock();
        if values.get(IMPORTED_MARKER) == Some(&Value::Bool(true)) {
            return Ok(0);
        }
        let legacy = match std::fs::read(legacy_file) {
            Ok(bytes) => parse_object(&bytes).unwrap_or_default(),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Map::new(),
            Err(err) => return Err(err).io(IoOp::Read, legacy_file),
        };
        let imported = fold_legacy(&legacy);
        let mut count = 0;
        for (key, value) in imported {
            if !values.contains_key(&key) {
                values.insert(key, value);
                count += 1;
            }
        }
        values.insert(IMPORTED_MARKER.to_owned(), Value::Bool(true));
        self.persist(&values)?;
        Ok(count)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Map<String, Value>> {
        self.values.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn persist(&self, values: &Map<String, Value>) -> Result<()> {
        let Some(file) = &self.file else {
            return Ok(());
        };
        let bytes = serde_json::to_vec_pretty(values).map_err(|source| Error::Json {
            path: Some(file.clone()),
            source,
        })?;
        crate::fs::write_atomic(file, &bytes)
    }
}

fn parse_object(bytes: &[u8]) -> Option<Map<String, Value>> {
    let text = std::str::from_utf8(bytes).ok()?;
    if text.trim().is_empty() {
        return Some(Map::new());
    }
    match serde_json::from_str::<Value>(text).ok()? {
        Value::Object(map) => Some(map),
        _ => None,
    }
}

fn backup_path(file: &Path) -> PathBuf {
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("settings");
    file.with_file_name(format!("{stem}.corrupt.json"))
}

fn fold_legacy(legacy: &Map<String, Value>) -> Vec<(String, Value)> {
    let mut plain = Vec::new();
    let mut sidebar = Map::new();
    let mut win10 = Vec::new();
    let mut window = Map::new();
    for (raw_key, value) in legacy {
        let Some(key) = raw_key.strip_prefix(LEGACY_PREFIX) else {
            continue;
        };
        if let Some(section) = key.strip_prefix(SIDEBAR_PREFIX) {
            sidebar.insert(section.to_owned(), value.clone());
        } else if let Some(game_key) = key.strip_prefix(WIN10_SETUP_PREFIX) {
            if value == &Value::Bool(true) {
                win10.push(Value::String(game_key.to_owned()));
            }
        } else if let Some(field) = key.strip_prefix(WINDOW_PREFIX).filter(|f| WINDOW_FIELDS.contains(f)) {
            window.insert(field.to_owned(), value.clone());
        } else {
            plain.push((key.to_owned(), value.clone()));
        }
    }
    if !sidebar.is_empty() {
        plain.push(("sidebar_section_collapsed".to_owned(), Value::Object(sidebar)));
    }
    if !win10.is_empty() {
        plain.push(("win10setup_done".to_owned(), Value::Array(win10)));
    }
    if !window.is_empty() {
        plain.push(("window".to_owned(), Value::Object(complete_window(window))));
    }
    plain
}

fn complete_window(mut window: Map<String, Value>) -> Map<String, Value> {
    let has_size = window.get("width").is_some_and(Value::is_number) && window.get("height").is_some_and(Value::is_number);
    if !has_size {
        window.insert("width".to_owned(), Value::from(crate::window_geometry::DEFAULT_WIDTH));
        window.insert("height".to_owned(), Value::from(crate::window_geometry::DEFAULT_HEIGHT));
    }
    let has_position = window.get("x").is_some_and(Value::is_number) && window.get("y").is_some_and(Value::is_number);
    if !has_position {
        window.remove("x");
        window.remove("y");
    }
    window
}

#[cfg(test)]
mod tests {
    use super::*;

    const THEME: SettingKey<String> = SettingKey::new("app_theme", || "dark".to_owned());
    const WIDTH: SettingKey<f64> = SettingKey::new("width", || 1300.0);
    const COLLAPSED: SettingKey<std::collections::BTreeMap<String, bool>> =
        SettingKey::new("sidebar_section_collapsed", Default::default);

    #[test]
    fn missing_file_starts_empty_and_writes_on_set() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        let (store, report) = SettingsStore::open(file.clone()).unwrap();
        assert_eq!(report, OpenReport::Created);
        assert_eq!(store.get(&THEME), "dark");
        assert!(!file.exists());
        store.set(&THEME, &"nier".to_owned()).unwrap();
        let (reopened, report) = SettingsStore::open(file).unwrap();
        assert_eq!(report, OpenReport::Loaded);
        assert_eq!(reopened.get(&THEME), "nier");
    }

    #[test]
    fn null_bytes_are_recovered_into_backup() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        std::fs::write(&file, vec![0u8; 657]).unwrap();
        let (store, report) = SettingsStore::open(file.clone()).unwrap();
        let backup = dir.path().join("settings.corrupt.json");
        assert_eq!(report, OpenReport::RecoveredCorrupt { backup: backup.clone() });
        assert_eq!(std::fs::read(&backup).unwrap(), vec![0u8; 657]);
        assert!(!file.exists());
        store.set(&WIDTH, &1600.0).unwrap();
        assert_eq!(SettingsStore::open(file).unwrap().0.get(&WIDTH), 1600.0);
    }

    #[test]
    fn truncated_json_and_invalid_utf8_are_recovered() {
        for bytes in [b"{\"app_theme\":\"ni".to_vec(), vec![0x7B, 0xC3, 0x28, 0xFF]] {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join("settings.json");
            std::fs::write(&file, &bytes).unwrap();
            let (_, report) = SettingsStore::open(file).unwrap();
            assert!(matches!(report, OpenReport::RecoveredCorrupt { .. }));
        }
    }

    #[test]
    fn wrong_type_falls_back_to_default() {
        let store = SettingsStore::in_memory();
        store.set(&SettingKey::<i32>::new("app_theme", || 0), &5).unwrap();
        assert_eq!(store.get(&THEME), "dark");
    }

    #[test]
    fn update_and_remove() {
        let store = SettingsStore::in_memory();
        store.update(&COLLAPSED, |map| {
            map.insert("Mods".into(), true);
        })
        .unwrap();
        assert_eq!(store.get(&COLLAPSED).get("Mods"), Some(&true));
        assert!(store.remove(&COLLAPSED).unwrap());
        assert!(!store.remove(&COLLAPSED).unwrap());
    }

    #[test]
    fn flutter_prefs_import_folds_prefixed_keys_once() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("shared_preferences.json");
        std::fs::write(
            &legacy,
            r#"{"flutter.app_theme":"nier","flutter.sidebar_section_collapsed_Mods":true,
               "flutter.win10setup_done_c:/games/nier":true,"flutter.window_width":1600.0,
               "flutter.window_height":900.0,"flutter.window_maximized":false,"other":1}"#,
        )
        .unwrap();
        let (store, _) = SettingsStore::open(dir.path().join("settings.json")).unwrap();
        assert_eq!(store.import_flutter_prefs(&legacy).unwrap(), 4);
        assert_eq!(store.get(&THEME), "nier");
        assert_eq!(store.get(&COLLAPSED).get("Mods"), Some(&true));
        let win10: Vec<String> = store.get(&SettingKey::new("win10setup_done", Vec::new));
        assert_eq!(win10, vec!["c:/games/nier".to_owned()]);
        let window: Map<String, Value> = store.get(&SettingKey::new("window", Map::new));
        assert_eq!(window.get("width"), Some(&Value::from(1600.0)));
        assert_eq!(store.import_flutter_prefs(&legacy).unwrap(), 0);
    }

    #[test]
    fn import_never_overwrites_existing_values() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("legacy.json");
        std::fs::write(&legacy, r#"{"flutter.app_theme":"nier"}"#).unwrap();
        let store = SettingsStore::in_memory();
        store.set(&THEME, &"dark".to_owned()).unwrap();
        assert_eq!(store.import_flutter_prefs(&legacy).unwrap(), 0);
        assert_eq!(store.get(&THEME), "dark");
    }
}
