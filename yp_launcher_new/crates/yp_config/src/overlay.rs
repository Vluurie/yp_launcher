use std::path::{Path, PathBuf};

use serde_json::{Map, Value as JsonValue, json};
use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::error::{ConfigError, Result};
use crate::migrate::overlay::SETTINGS_FILE_NAME;

pub type Settings = Map<String, JsonValue>;

pub const IMPELLER_KEY: &str = "impeller";
pub const OVERLAY_LANGUAGE_KEY: &str = "overlayLanguage";
pub const KEYBINDS_KEY: &str = "keybinds";
pub const RANDOMIZER_KEY: &str = "randomizerConfig";
pub const DEFAULT_OUTFITS_KEY: &str = "defaultOutfits";
pub const CHEATS_KEY: &str = "cheatsConfig";

pub fn settings_path(game: &GamePaths) -> PathBuf {
    game.internal_cache_dir().join(SETTINGS_FILE_NAME)
}

pub fn resolve_settings_path(game: Option<&GamePaths>) -> Option<PathBuf> {
    game.filter(|game| !game.root().as_os_str().is_empty())
        .map(settings_path)
}

pub fn default_settings() -> Settings {
    let defaults = json!({
        "firstPlaythrough": true,
        "impeller": false,
        "keybinds": {
            "main": {"yorha_protocol": "F1"},
            "yorha_protocol": {
                "freeze_game": "F5",
                "max_speed": "F6",
                "free_cam": "F7",
                "phase_jump": "F8",
                "toggle_game_input": "F9",
                "advance_frame": "F10",
                "dev_mode": "Insert",
            },
        },
        "gameKeybindsGlobal": false,
        "loadingSpeedupEnabled": true,
        "shadersEnabled": true,
        "soundEnabled": true,
        "randomizerConfig": {
            "autoStartOnStartup": false,
            "randomizeGroundEnemies": true,
            "randomizeFlyingEnemies": true,
            "allowBigEnemies": true,
            "includeDlcEnemies": true,
            "excludedEnemies": [],
        },
    });
    match defaults {
        JsonValue::Object(map) => map,
        _ => Settings::new(),
    }
}

pub fn to_pretty_json(settings: &Settings) -> Option<String> {
    serde_json::to_string_pretty(settings).ok()
}

pub fn load_settings_at(path: Option<&Path>) -> Result<Settings> {
    let Some(path) = path else {
        return Ok(default_settings());
    };
    let Some(content) = yp_core::fs::read_text_opt(path)? else {
        return Ok(default_settings());
    };
    if content.trim().is_empty() {
        return Ok(default_settings());
    }
    match serde_json::from_str::<JsonValue>(&content) {
        Ok(JsonValue::Object(map)) => Ok(map),
        Ok(_) => Err(ConfigError::SettingsNotObject {
            path: path.to_path_buf(),
        }),
        Err(source) => Err(yp_core::Error::Json {
            path: Some(path.to_path_buf()),
            source,
        }
        .into()),
    }
}

pub fn load_settings(game: Option<&GamePaths>) -> Result<Settings> {
    load_settings_at(resolve_settings_path(game).as_deref())
}

pub fn save_settings_at(settings: &Settings, path: Option<&Path>) -> Result<bool> {
    let Some(path) = path else {
        return Ok(false);
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).io(IoOp::Create, parent)?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|source| yp_core::Error::Json {
        path: Some(path.to_path_buf()),
        source,
    })?;
    yp_core::fs::write_text_atomic(path, &text)?;
    Ok(true)
}

pub fn save_settings(settings: &Settings, game: Option<&GamePaths>) -> Result<bool> {
    save_settings_at(settings, resolve_settings_path(game).as_deref())
}

pub fn seed_overlay_language_quiet(game: Option<&GamePaths>, language_code: &str) -> bool {
    let attempt = || -> Result<bool> {
        let mut settings = load_settings(game)?;
        let existing = settings
            .get(OVERLAY_LANGUAGE_KEY)
            .and_then(JsonValue::as_str);
        if existing.is_some_and(|existing| !existing.is_empty()) {
            return Ok(false);
        }
        settings.insert(
            OVERLAY_LANGUAGE_KEY.to_owned(),
            JsonValue::String(language_code.to_owned()),
        );
        save_settings(&settings, game)
    };
    attempt().unwrap_or_else(|err| {
        log::warn!("could not seed the overlay language: {err}");
        false
    })
}

pub fn load_impeller_quiet(game: Option<&GamePaths>) -> bool {
    load_settings(game)
        .ok()
        .and_then(|settings| settings.get(IMPELLER_KEY).and_then(JsonValue::as_bool))
        .unwrap_or(false)
}

pub fn save_impeller_quiet(game: Option<&GamePaths>, enabled: bool) -> bool {
    let attempt = || -> Result<bool> {
        let mut settings = load_settings(game)?;
        settings.insert(IMPELLER_KEY.to_owned(), JsonValue::Bool(enabled));
        save_settings(&settings, game)
    };
    attempt().unwrap_or_else(|err| {
        log::warn!("could not save the impeller setting: {err}");
        false
    })
}
