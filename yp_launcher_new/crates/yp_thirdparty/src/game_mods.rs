use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table};
use yp_core::consts::GAME_MODS_CONFIG_NAME;
use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::error::Result;
use crate::flags;
use crate::model::{
    Classification, DllSummary, GameModEntry, GameModSettings, GameModsInfo, InstallFailure, InstallOutcome,
    RuntimeStatus, ThirdPartyKind, ThirdPartyRuntime, ThirdPartyUpdateInfo,
};
use crate::paths;
use crate::runtime::GraphicsRuntime;
use crate::toml_ext;
use crate::tree::{self, ExistingFiles};

pub const DEFAULTS_SECTION: &str = "defaults";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameModsRuntime;

impl GameModsRuntime {
    pub fn config_path(&self, game: &GamePaths) -> PathBuf {
        paths::game_mods(game).join(GAME_MODS_CONFIG_NAME)
    }

    pub fn set_mod_settings(&self, game: &GamePaths, file_name: &str, settings: &GameModSettings) -> Result<bool> {
        let file = self.config_path(game);
        let raw = yp_core::fs::read_text_opt(&file)?.unwrap_or_default();
        let Some(updated) = with_settings(&raw, file_name, settings) else {
            return Ok(false);
        };
        let dir = paths::game_mods(game);
        std::fs::create_dir_all(&dir).io(IoOp::Create, &dir)?;
        std::fs::write(&file, updated).io(IoOp::Write, &file)?;
        Ok(true)
    }

    pub fn set_mod_disabled(&self, game: &GamePaths, file_name: &str, disabled: bool) -> Result<bool> {
        let config = read_config(&self.config_path(game));
        let settings = settings_for(&config, file_name).with_disabled(disabled);
        self.set_mod_settings(game, file_name, &settings)
    }

    pub fn path_of(&self, game: &GamePaths, mod_dir: &str) -> Option<PathBuf> {
        if !paths::is_plain_name(mod_dir) {
            return None;
        }
        Some(paths::game_mods(game).join(mod_dir)).filter(|dir| dir.is_dir())
    }

    pub fn remove_mod(&self, game: &GamePaths, mod_dir: &str) {
        if let Some(dir) = self.path_of(game, mod_dir) {
            yp_core::fs::remove_dir_quiet(&dir);
        }
    }

    fn collect_info(&self, dir: &Path) -> GameModsInfo {
        if !dir.is_dir() {
            return GameModsInfo::default();
        }
        let config_file = dir.join(GAME_MODS_CONFIG_NAME);
        let config = read_config(&config_file);
        let mods = paths::mod_dirs(dir)
            .into_iter()
            .flat_map(|mod_dir| {
                let mod_name = tree::file_name(&mod_dir);
                paths::dlls_in(&mod_dir).into_iter().map(move |dll| (mod_name.clone(), dll))
            })
            .map(|(mod_dir, dll)| {
                let file_name = tree::file_name(&dll);
                let settings = settings_for(&config, &file_name);
                GameModEntry {
                    size_bytes: tree::file_size(&dll),
                    disabled: settings.disabled,
                    settings: Some(settings),
                    file_name,
                    mod_dir,
                }
            })
            .collect();
        GameModsInfo {
            mods,
            has_config: config_file.is_file(),
        }
    }
}

impl GraphicsRuntime for GameModsRuntime {
    fn runtime(&self) -> ThirdPartyRuntime {
        ThirdPartyRuntime::GameMods
    }

    fn install_dir(&self, game: &GamePaths) -> PathBuf {
        paths::game_mods(game)
    }

    fn can_install(&self, classification: &Classification) -> bool {
        classification.kind == ThirdPartyKind::GameMod
    }

    fn install(&self, game: &GamePaths, classification: &Classification) -> Result<InstallOutcome> {
        let Some(first) = classification.game_mod_dlls.first() else {
            return Ok(InstallOutcome::Rejected(InstallFailure::NoDll));
        };
        let kind_dir = paths::game_mods(game);
        let names: Vec<String> = classification.game_mod_dlls.iter().map(|dll| tree::file_name(dll)).collect();
        let dest = paths::mod_dir_with_dll(&kind_dir, &names).unwrap_or_else(|| {
            let name = paths::mod_dir_name(classification.source_name.as_deref().unwrap_or_default(), &tree::file_stem(first));
            kind_dir.join(paths::unique_dir_name(&kind_dir, &name))
        });
        std::fs::create_dir_all(&dest).io(IoOp::Create, &dest)?;

        for folder in dll_folders(&classification.game_mod_dlls) {
            tree::merge_tree(&folder, &dest, ExistingFiles::Overwrite);
        }

        flags::set_disabled(game, self.loading_flag(), false)?;
        Ok(InstallOutcome::installed(ThirdPartyRuntime::GameMods))
    }

    fn would_update(&self, game: &GamePaths, classification: &Classification) -> Option<ThirdPartyUpdateInfo> {
        let kind_dir = paths::game_mods(game);
        classification.game_mod_dlls.iter().find_map(|incoming| {
            let installed = paths::find_dll(&kind_dir, &tree::file_name(incoming))?;
            tree::files_differ(incoming, &installed).then(|| ThirdPartyUpdateInfo {
                runtime: ThirdPartyRuntime::GameMods,
                installed: size_summary(&installed),
                incoming: size_summary(incoming),
            })
        })
    }

    fn repair(&self, _game: &GamePaths) -> Result<()> {
        Ok(())
    }

    fn import_from_game_root(&self, _game: &GamePaths) -> Result<bool> {
        Ok(false)
    }

    fn status(&self, game: &GamePaths) -> RuntimeStatus {
        let info = self.collect_info(&paths::game_mods(game));
        RuntimeStatus {
            installed: !info.mods.is_empty(),
            enabled: self.is_enabled(game),
            game_mods_info: Some(info),
            ..RuntimeStatus::default()
        }
    }

    fn remove(&self, game: &GamePaths) -> Result<()> {
        for dir in paths::mod_dirs(&paths::game_mods(game)) {
            yp_core::fs::remove_dir_quiet(&dir);
        }
        flags::set_disabled(game, self.loading_flag(), true)
    }
}

pub(crate) fn dll_folders(dlls: &[PathBuf]) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    for folder in dlls.iter().filter_map(|dll| dll.parent()) {
        if !folders.iter().any(|known| known == folder) {
            folders.push(folder.to_path_buf());
        }
    }
    folders
}

pub(crate) fn size_summary(dll: &Path) -> DllSummary {
    DllSummary {
        file_name: tree::file_name(dll),
        version: None,
        size_bytes: tree::file_size(dll),
    }
}

fn read_config(file: &Path) -> toml::Table {
    match yp_core::fs::read_text_opt(file) {
        Ok(Some(text)) => toml_ext::lenient_values(&text),
        _ => toml::Table::new(),
    }
}

fn section_key<'a>(keys: impl Iterator<Item = (&'a str, bool)>, file_name: &str) -> Option<String> {
    let wanted = file_name.to_lowercase();
    keys.filter(|(key, is_table)| *is_table && *key != DEFAULTS_SECTION)
        .find(|(key, _)| key.to_lowercase() == wanted)
        .map(|(key, _)| key.to_owned())
}

pub fn settings_for(config: &toml::Table, file_name: &str) -> GameModSettings {
    let key = section_key(config.iter().map(|(key, value)| (key.as_str(), value.is_table())), file_name);
    match config.get(key.as_deref().unwrap_or(DEFAULTS_SECTION)) {
        Some(toml::Value::Table(table)) => GameModSettings::from_toml(table),
        _ => GameModSettings::default(),
    }
}

pub fn with_settings(raw: &str, file_name: &str, settings: &GameModSettings) -> Option<String> {
    let mut doc: DocumentMut = yp_core::toml_io::parse(raw).ok()?;
    let key = section_key(doc.iter().map(|(key, item)| (key, item.is_table_like())), file_name)
        .unwrap_or_else(|| file_name.to_owned());
    let root = doc.as_table_mut();
    if !root.get(&key).is_some_and(Item::is_table_like) {
        let mut table = Table::new();
        if !root.is_empty() {
            table.decor_mut().set_prefix("\n");
        }
        root.insert(&key, Item::Table(table));
    }
    let table = root.get_mut(&key).and_then(Item::as_table_like_mut)?;
    for (field, value) in settings.toml_values() {
        toml_ext::set_keeping_decor(table, field, value);
    }
    Some(doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_section_is_appended_after_a_blank_line() {
        let settings = GameModSettings::default().with_disabled(true);
        let raw = "# keep me\n[defaults]\nsize_of_image = 1 # trailing\n";
        let out = with_settings(raw, "celestial.dll", &settings).unwrap();
        assert_eq!(
            out,
            "# keep me\n[defaults]\nsize_of_image = 1 # trailing\n\n[\"celestial.dll\"]\nfake_module_handle = true\n\
             fake_exe_path = true\nsize_of_image = 26177536\ndisabled = true\nreport_patches = false\n\
             wait_for_patch_phase = false\npatch_phase_timeout_ms = 10000\n"
        );
    }

    #[test]
    fn an_existing_value_keeps_its_spacing_and_comment() {
        let settings = GameModSettings::default().with_disabled(true);
        let out = with_settings("[\"celestial.dll\"]\ndisabled   =   false # mine\n", "celestial.dll", &settings).unwrap();
        assert!(out.starts_with("[\"celestial.dll\"]\ndisabled   =   true # mine\n"));
    }

    #[test]
    fn an_inline_section_is_updated_in_place() {
        let settings = GameModSettings::default().with_disabled(true);
        let out = with_settings("\"celestial.dll\" = { disabled = false }\n", "celestial.dll", &settings).unwrap();
        let values: toml::Table = toml::from_str(&out).unwrap();
        assert_eq!(values["celestial.dll"]["disabled"].as_bool(), Some(true));
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn settings_fall_back_to_defaults_only_without_an_own_section() {
        let config: toml::Table =
            toml::from_str("[defaults]\ndisabled = true\n[\"A.dll\"]\nreport_patches = true\n").unwrap();
        assert!(settings_for(&config, "b.dll").disabled);
        let own = settings_for(&config, "a.dll");
        assert!(own.report_patches);
        assert!(!own.disabled);
    }
}
