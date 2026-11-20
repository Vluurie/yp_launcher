use std::path::PathBuf;
use std::time::Instant;

use serde_json::{Map, Value as JsonValue};
use yp_core::GamePaths;

use crate::error::Result;
use crate::fields::overlay::workspace;
use crate::overlay::{
    self, CHEATS_KEY, DEFAULT_OUTFITS_KEY, IMPELLER_KEY, KEYBINDS_KEY, RANDOMIZER_KEY, Settings,
};
use crate::state::config_store::SELF_WRITE_QUIET_PERIOD;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OverlayData {
    pub settings: Settings,
    pub has_unsaved_changes: bool,
    pub settings_path: Option<PathBuf>,
}

fn object<'a>(settings: &'a Settings, key: &str) -> Option<&'a Map<String, JsonValue>> {
    settings.get(key).and_then(JsonValue::as_object)
}

impl OverlayData {
    pub fn keybinds(&self) -> Settings {
        object(&self.settings, KEYBINDS_KEY)
            .cloned()
            .unwrap_or_default()
    }

    pub fn impeller(&self) -> bool {
        self.settings
            .get(IMPELLER_KEY)
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
    }

    pub fn game_keybinds_global(&self) -> bool {
        workspace::GAME_KEYBINDS_GLOBAL.value_in_json(&self.settings)
    }

    pub fn loading_speedup_enabled(&self) -> bool {
        workspace::LOADING_SPEEDUP_ENABLED.value_in_json(&self.settings)
    }

    pub fn shaders_enabled(&self) -> bool {
        workspace::SHADERS_ENABLED.value_in_json(&self.settings)
    }

    pub fn sound_enabled(&self) -> bool {
        workspace::SOUND_ENABLED.value_in_json(&self.settings)
    }

    pub fn editor_mcp_enabled(&self) -> bool {
        workspace::EDITOR_MCP_ENABLED.value_in_json(&self.settings)
    }

    pub fn randomizer_config(&self) -> Settings {
        object(&self.settings, RANDOMIZER_KEY)
            .cloned()
            .unwrap_or_default()
    }

    pub fn default_outfits(&self) -> Settings {
        object(&self.settings, DEFAULT_OUTFITS_KEY)
            .cloned()
            .unwrap_or_default()
    }

    pub fn default_outfit_for(&self, character_type: &str) -> Option<String> {
        object(&self.settings, DEFAULT_OUTFITS_KEY)
            .and_then(|outfits| outfits.get(character_type))
            .and_then(JsonValue::as_str)
            .map(str::to_owned)
    }

    pub fn cheats_config(&self) -> Settings {
        object(&self.settings, CHEATS_KEY)
            .cloned()
            .unwrap_or_default()
    }
}

fn child_object<'a>(
    settings: &'a mut Settings,
    key: &str,
) -> Option<&'a mut Map<String, JsonValue>> {
    if !settings.get(key).is_some_and(JsonValue::is_object) {
        settings.insert(key.to_owned(), JsonValue::Object(Map::new()));
    }
    settings.get_mut(key).and_then(JsonValue::as_object_mut)
}

#[derive(Debug, Default)]
pub struct OverlayStore {
    game: Option<GamePaths>,
    data: OverlayData,
    last_self_write: Option<Instant>,
}

impl OverlayStore {
    pub fn new(game: Option<GamePaths>) -> Self {
        Self {
            game,
            data: OverlayData::default(),
            last_self_write: None,
        }
    }

    pub fn set_game(&mut self, game: Option<GamePaths>) {
        self.game = game;
    }

    pub fn data(&self) -> &OverlayData {
        &self.data
    }

    pub fn watch_path(&self) -> Option<PathBuf> {
        overlay::resolve_settings_path(self.game.as_ref())
    }

    pub fn load(&mut self) -> Result<()> {
        let settings = overlay::load_settings(self.game.as_ref())?;
        self.data = OverlayData {
            settings,
            has_unsaved_changes: false,
            settings_path: self.watch_path(),
        };
        Ok(())
    }

    pub fn should_reload_external(&self, now: Instant) -> bool {
        if self.data.has_unsaved_changes {
            return false;
        }
        self.last_self_write
            .is_none_or(|written| now.saturating_duration_since(written) >= SELF_WRITE_QUIET_PERIOD)
    }

    pub fn reload_external(&mut self, now: Instant) -> Result<bool> {
        if !self.should_reload_external(now) {
            return Ok(false);
        }
        self.data.settings = overlay::load_settings(self.game.as_ref())?;
        Ok(true)
    }

    pub fn reload_if_clean(&mut self) -> Result<bool> {
        if self.data.has_unsaved_changes {
            return Ok(false);
        }
        self.data.settings = overlay::load_settings(self.game.as_ref())?;
        Ok(true)
    }

    pub fn update_keybind(&mut self, category: &str, action: &str, key: &str) {
        if let Some(category) = child_object(&mut self.data.settings, KEYBINDS_KEY)
            .and_then(|keybinds| child_object(keybinds, category))
        {
            category.insert(action.to_owned(), JsonValue::String(key.to_owned()));
        }
        self.data.has_unsaved_changes = true;
    }

    pub fn update_toggle(&mut self, key: &str, value: bool) {
        self.data
            .settings
            .insert(key.to_owned(), JsonValue::Bool(value));
        self.data.has_unsaved_changes = true;
    }

    pub fn set_default_outfit(&mut self, character_type: &str, folder_name: Option<&str>) {
        let mut outfits = object(&self.data.settings, DEFAULT_OUTFITS_KEY)
            .cloned()
            .unwrap_or_default();
        match folder_name {
            Some(folder) => {
                outfits.insert(
                    character_type.to_owned(),
                    JsonValue::String(folder.to_owned()),
                );
            }
            None => {
                outfits.shift_remove(character_type);
            }
        }
        if outfits.is_empty() {
            self.data.settings.shift_remove(DEFAULT_OUTFITS_KEY);
        } else {
            self.data
                .settings
                .insert(DEFAULT_OUTFITS_KEY.to_owned(), JsonValue::Object(outfits));
        }
        self.data.has_unsaved_changes = true;
    }

    pub fn update_cheat(&mut self, key: &str, value: JsonValue) {
        if let Some(cheats) = child_object(&mut self.data.settings, CHEATS_KEY) {
            cheats.insert(key.to_owned(), value);
        }
        self.data.has_unsaved_changes = true;
    }

    pub fn update_randomizer(&mut self, key: &str, value: JsonValue) {
        if let Some(randomizer) = child_object(&mut self.data.settings, RANDOMIZER_KEY) {
            randomizer.insert(key.to_owned(), value);
        }
        self.data.has_unsaved_changes = true;
    }

    pub fn save(&mut self) -> Result<bool> {
        self.last_self_write = Some(Instant::now());
        let saved = overlay::save_settings(&self.data.settings, self.game.as_ref())?;
        if saved {
            self.data.has_unsaved_changes = false;
        }
        Ok(saved)
    }

    pub fn discard_changes(&mut self) -> Result<()> {
        self.load()
    }
}
