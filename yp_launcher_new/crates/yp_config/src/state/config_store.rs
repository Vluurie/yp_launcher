use std::path::PathBuf;
use std::time::{Duration, Instant};

use yp_core::toml_io::TomlError;
use yp_core::{GamePaths, HostEnv};

use crate::ensure::ensure_configs;
use crate::error::Result;
use crate::fields::{lodmod, nams};
use crate::files::ConfigFile;
use crate::migrate::nams::CUTSCENE_SECTION;
use crate::values::{set_in, with_value};

pub const NAIOM_SECTION: &str = "mouse";
pub const ENABLE_H264_KEY: &str = "enable_h264";
pub const SELF_WRITE_QUIET_PERIOD: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CutsceneFlags {
    pub needs_h264: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigData {
    pub nams_values: toml::Table,
    pub lodmod_values: toml::Table,
    pub texture_injection_values: toml::Table,
    pub nams_raw: String,
    pub naiom_raw: String,
    pub lodmod_raw: String,
    pub texture_injection_raw: String,
    pub has_unsaved_changes: bool,
    pub nams_error: Option<TomlError>,
    pub naiom_error: Option<TomlError>,
    pub lodmod_error: Option<TomlError>,
    pub texture_injection_error: Option<TomlError>,
}

impl ConfigData {
    pub fn naiom_values(&self) -> toml::Table {
        self.nams_values
            .get(NAIOM_SECTION)
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default()
    }

    pub fn error(&self, file: ConfigFile) -> Option<&TomlError> {
        match file {
            ConfigFile::Nams => self.nams_error.as_ref(),
            ConfigFile::Naiom => self.naiom_error.as_ref(),
            ConfigFile::LodMod => self.lodmod_error.as_ref(),
            ConfigFile::TextureInjection => self.texture_injection_error.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SaveReport {
    pub default_outfits_changed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppliedChange {
    Updated {
        section: Option<&'static str>,
        key: &'static str,
    },
}

struct Parsed {
    values: toml::Table,
    error: Option<TomlError>,
}

fn parse_strict(raw: &str) -> Parsed {
    if raw.trim().is_empty() {
        return Parsed {
            values: toml::Table::new(),
            error: None,
        };
    }
    match yp_core::toml_io::parse_values(raw) {
        Ok(values) => Parsed {
            values,
            error: None,
        },
        Err(error) => Parsed {
            values: toml::Table::new(),
            error: Some(error),
        },
    }
}

fn read_raw(game: &GamePaths, file: ConfigFile) -> Result<String> {
    Ok(yp_core::fs::read_text_opt(&file.path(game))?.unwrap_or_default())
}

#[derive(Debug)]
pub struct ConfigStore {
    game: GamePaths,
    host: HostEnv,
    data: ConfigData,
    naiom_pending: bool,
    lodmod_pending: bool,
    last_self_write: Option<Instant>,
}

impl ConfigStore {
    pub fn new(game: GamePaths, host: HostEnv) -> Self {
        Self {
            game,
            host,
            data: ConfigData::default(),
            naiom_pending: false,
            lodmod_pending: false,
            last_self_write: None,
        }
    }

    pub fn game(&self) -> &GamePaths {
        &self.game
    }

    pub fn data(&self) -> &ConfigData {
        &self.data
    }

    pub fn watch_paths(&self) -> Vec<PathBuf> {
        vec![self.game.nams_dir()]
    }

    pub fn is_watched_file(name: &str) -> bool {
        ConfigFile::from_file_name(name).is_some()
    }

    pub fn has_pending_naiom(&self) -> bool {
        self.naiom_pending
    }

    pub fn has_pending_lodmod(&self) -> bool {
        self.lodmod_pending
    }

    fn write(&mut self, file: ConfigFile, text: &str) -> Result<()> {
        self.last_self_write = Some(Instant::now());
        yp_core::fs::write_text_atomic(&file.path(&self.game), text)?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        if self.naiom_pending {
            self.flush_naiom()?;
        }
        ensure_configs(&self.game, &self.host)?;

        let nams_raw = read_raw(&self.game, ConfigFile::Nams)?;
        let naiom_raw = read_raw(&self.game, ConfigFile::Naiom)?;
        let lodmod_raw = read_raw(&self.game, ConfigFile::LodMod)?;
        let texture_raw = read_raw(&self.game, ConfigFile::TextureInjection)?;

        let nams = parse_strict(&nams_raw);
        let naiom = parse_strict(&naiom_raw);
        let lodmod = parse_strict(&lodmod_raw);
        let texture = parse_strict(&texture_raw);

        let mut nams_values = nams.values;
        nams_values.insert(NAIOM_SECTION.to_owned(), toml::Value::Table(naiom.values));

        self.data = ConfigData {
            nams_values,
            lodmod_values: lodmod.values,
            texture_injection_values: texture.values,
            nams_raw,
            naiom_raw,
            lodmod_raw,
            texture_injection_raw: texture_raw,
            has_unsaved_changes: false,
            nams_error: nams.error,
            naiom_error: naiom.error,
            lodmod_error: lodmod.error,
            texture_injection_error: texture.error,
        };
        self.naiom_pending = false;
        self.lodmod_pending = false;
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
        self.load()?;
        Ok(true)
    }

    pub fn apply_cutscene_flags(&mut self, flags: CutsceneFlags) -> Result<Vec<AppliedChange>> {
        ensure_configs(&self.game, &self.host)?;
        let raw = read_raw(&self.game, ConfigFile::Nams)?;
        let parsed = parse_strict(&raw);
        if parsed.error.is_some() {
            return Ok(Vec::new());
        }
        let cutscene = parsed
            .values
            .get(CUTSCENE_SECTION)
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        if cutscene.get(ENABLE_H264_KEY).and_then(toml::Value::as_bool) == Some(flags.needs_h264) {
            return Ok(Vec::new());
        }
        let mut updates = toml::Table::new();
        set_in(
            &mut updates,
            Some(CUTSCENE_SECTION),
            ENABLE_H264_KEY,
            toml::Value::Boolean(flags.needs_h264),
        );
        let updated = yp_core::toml_io::update_toml(&raw, &updates)?;
        self.write(ConfigFile::Nams, &updated)?;
        set_in(
            &mut self.data.nams_values,
            Some(CUTSCENE_SECTION),
            ENABLE_H264_KEY,
            toml::Value::Boolean(flags.needs_h264),
        );
        self.data.nams_raw = updated;
        Ok(vec![AppliedChange::Updated {
            section: Some(CUTSCENE_SECTION),
            key: ENABLE_H264_KEY,
        }])
    }

    pub fn update_nams_now(
        &mut self,
        key: &str,
        value: toml::Value,
        section: Option<&str>,
    ) -> Result<bool> {
        let raw = read_raw(&self.game, ConfigFile::Nams)?;
        if parse_strict(&raw).error.is_some() {
            return Ok(false);
        }
        let mut updates = toml::Table::new();
        set_in(&mut updates, section, key, value.clone());
        let updated = yp_core::toml_io::update_toml(&raw, &updates)?;
        self.write(ConfigFile::Nams, &updated)?;
        set_in(&mut self.data.nams_values, section, key, value);
        self.data.nams_raw = updated;
        Ok(true)
    }

    pub fn update_naiom_live(&mut self, key: &str, value: toml::Value, section: &str) {
        set_in(&mut self.data.nams_values, Some(section), key, value);
        self.naiom_pending = true;
    }

    pub fn flush_naiom(&mut self) -> Result<bool> {
        self.naiom_pending = false;
        if self.data.naiom_error.is_some() {
            return Ok(false);
        }
        let values = self.data.naiom_values();
        let updated = yp_core::toml_io::update_toml(&self.data.naiom_raw, &values)?;
        if updated == self.data.naiom_raw {
            return Ok(false);
        }
        self.write(ConfigFile::Naiom, &updated)?;
        self.data.naiom_raw = updated;
        Ok(true)
    }

    pub fn update_nams(&mut self, key: &str, value: toml::Value, section: Option<&str>) {
        set_in(&mut self.data.nams_values, section, key, value);
        self.data.has_unsaved_changes = true;
    }

    pub fn update_lodmod(&mut self, key: &str, value: toml::Value, section: Option<&str>) {
        set_in(&mut self.data.lodmod_values, section, key, value);
        self.data.has_unsaved_changes = true;
    }

    pub fn update_lodmod_live(&mut self, key: &str, value: toml::Value, section: Option<&str>) {
        let mut updated = with_value(&self.data.lodmod_values, section, key, value.clone());
        let high_grids_toggle = section == lodmod::HIGH_GRIDS_ENABLED.section()
            && key == lodmod::HIGH_GRIDS_ENABLED.key();
        if high_grids_toggle && let Some(enabled) = value.as_bool() {
            set_in(
                &mut updated,
                None,
                lodmod::DISABLE_MANUAL_CULLING.key(),
                toml::Value::Boolean(enabled),
            );
            if enabled {
                set_in(
                    &mut updated,
                    None,
                    lodmod::DISABLE_CAMERA_CULLING.key(),
                    toml::Value::Boolean(true),
                );
            }
        }
        self.data.lodmod_values = updated;
        self.lodmod_pending = true;
    }

    pub fn apply_lodmod_bloom_2017_preset(&mut self) -> Result<bool> {
        self.lodmod_pending = false;
        for (key, value) in lodmod::bloom_2017_preset() {
            set_in(&mut self.data.lodmod_values, None, &key, value);
        }
        self.flush_lodmod()
    }

    pub fn reset_lodmod_to_defaults(&mut self) -> Result<bool> {
        self.lodmod_pending = false;
        let mut defaults = toml::Table::new();
        for field in lodmod::ALL {
            let meta = field.meta();
            set_in(&mut defaults, meta.section, meta.key, field.default_toml());
        }
        self.data.lodmod_values = defaults;
        self.flush_lodmod()
    }

    pub fn flush_lodmod(&mut self) -> Result<bool> {
        self.lodmod_pending = false;
        if self.data.lodmod_error.is_some() {
            return Ok(false);
        }
        let updated =
            yp_core::toml_io::update_toml(&self.data.lodmod_raw, &self.data.lodmod_values)?;
        if updated == self.data.lodmod_raw {
            return Ok(false);
        }
        self.write(ConfigFile::LodMod, &updated)?;
        self.data.lodmod_raw = updated;
        Ok(true)
    }

    pub fn update_texture_injection(&mut self, key: &str, value: toml::Value) {
        self.data
            .texture_injection_values
            .insert(key.to_owned(), value);
        self.data.has_unsaved_changes = true;
    }

    pub fn update_texture_injection_silent(&mut self, key: &str, value: toml::Value) {
        self.data
            .texture_injection_values
            .insert(key.to_owned(), value);
    }

    fn write_config(
        &mut self,
        file: ConfigFile,
        raw: &str,
        values: &toml::Table,
        error: bool,
    ) -> Result<String> {
        if error {
            return Ok(raw.to_owned());
        }
        let updated = yp_core::toml_io::update_toml(raw, values)?;
        if updated != raw || !file.path(&self.game).exists() {
            self.write(file, &updated)?;
        }
        Ok(updated)
    }

    pub fn save(&mut self) -> Result<SaveReport> {
        let default_outfits = nams::EXPERIMENTAL_DEFAULT_OUTFITS.key();
        let default_outfits_was = yp_core::toml_io::parse_values(&self.data.nams_raw)
            .ok()
            .and_then(|values| values.get(default_outfits).and_then(toml::Value::as_bool))
            == Some(true);

        let mut nams_values = self.data.nams_values.clone();
        nams_values.remove(NAIOM_SECTION);
        let naiom_values = self.data.naiom_values();

        let nams_raw = self.data.nams_raw.clone();
        let naiom_raw = self.data.naiom_raw.clone();
        let lodmod_raw = self.data.lodmod_raw.clone();
        let texture_raw = self.data.texture_injection_raw.clone();
        let lodmod_values = self.data.lodmod_values.clone();
        let texture_values = self.data.texture_injection_values.clone();

        let updated_nams = self.write_config(
            ConfigFile::Nams,
            &nams_raw,
            &nams_values,
            self.data.nams_error.is_some(),
        )?;
        let updated_naiom = self.write_config(
            ConfigFile::Naiom,
            &naiom_raw,
            &naiom_values,
            self.data.naiom_error.is_some(),
        )?;
        let updated_lodmod = self.write_config(
            ConfigFile::LodMod,
            &lodmod_raw,
            &lodmod_values,
            self.data.lodmod_error.is_some(),
        )?;
        let updated_texture = self.write_config(
            ConfigFile::TextureInjection,
            &texture_raw,
            &texture_values,
            self.data.texture_injection_error.is_some(),
        )?;

        self.data.nams_raw = updated_nams;
        self.data.naiom_raw = updated_naiom;
        self.data.lodmod_raw = updated_lodmod;
        self.data.texture_injection_raw = updated_texture;
        self.data.has_unsaved_changes = false;
        self.naiom_pending = false;
        self.lodmod_pending = false;

        let default_outfits_now = self
            .data
            .nams_values
            .get(default_outfits)
            .and_then(toml::Value::as_bool)
            == Some(true);
        Ok(SaveReport {
            default_outfits_changed: default_outfits_now != default_outfits_was,
        })
    }

    pub fn discard_changes(&mut self) -> Result<()> {
        self.load()
    }
}
