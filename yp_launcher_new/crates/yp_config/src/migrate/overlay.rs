use std::path::{Path, PathBuf};

use serde_json::Value as JsonValue;
use yp_core::wine_paths::{self, to_posix};
use yp_core::{GamePaths, HostEnv, IoOp, IoResultExt, Os};

use crate::error::Result;
use crate::overlay::{IMPELLER_KEY, settings_path, to_pretty_json};

pub const LEGACY_SETTINGS_DIR: &str = "NAMS";
pub const SETTINGS_FILE_NAME: &str = "settings.json";

pub fn with_impeller_off(raw: &str) -> String {
    let Ok(JsonValue::Object(mut settings)) = serde_json::from_str::<JsonValue>(raw) else {
        return raw.to_owned();
    };
    if settings.get(IMPELLER_KEY) != Some(&JsonValue::Bool(true)) {
        return raw.to_owned();
    }
    settings.insert(IMPELLER_KEY.to_owned(), JsonValue::Bool(false));
    to_pretty_json(&settings).unwrap_or_else(|| raw.to_owned())
}

pub fn legacy_settings_roots(game_dir: &Path, host: &HostEnv) -> Vec<PathBuf> {
    let app_data = host
        .var("APPDATA")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    if host.os() == Os::Windows {
        return app_data.into_iter().collect();
    }
    let mut prefixes = Vec::new();
    if let Some(prefix) = wine_paths::infer_wine_prefix(&to_posix(game_dir)) {
        prefixes.push(prefix);
    }
    if let Some(steam) = wine_paths::infer_steam_context_for(host, game_dir) {
        prefixes.push(steam.prefix());
    }
    prefixes
        .iter()
        .map(|prefix| PathBuf::from(wine_paths::wine_roaming_dir(prefix, host)))
        .chain(app_data)
        .collect()
}

pub fn migrate_overlay_settings(game: &GamePaths, host: &HostEnv) -> Result<Option<PathBuf>> {
    let target = settings_path(game);
    if target.exists() {
        return Ok(None);
    }
    for root in legacy_settings_roots(game.root(), host) {
        let old = root.join(LEGACY_SETTINGS_DIR).join(SETTINGS_FILE_NAME);
        if !old.is_file() {
            continue;
        }
        let raw = std::fs::read_to_string(&old).io(IoOp::Read, &old)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).io(IoOp::Create, parent)?;
        }
        yp_core::fs::write_text_atomic(&target, &with_impeller_off(&raw))?;
        std::fs::remove_file(&old).io(IoOp::Remove, &old)?;
        return Ok(Some(old));
    }
    Ok(None)
}
