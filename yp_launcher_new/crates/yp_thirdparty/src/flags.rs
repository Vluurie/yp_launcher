use yp_core::GamePaths;

use crate::error::Result;
use crate::toml_ext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoadingFlag {
    ReShade,
    Migoto,
    GameMods,
    GraphicMods,
}

impl LoadingFlag {
    pub const ALL: [LoadingFlag; 4] = [
        LoadingFlag::ReShade,
        LoadingFlag::Migoto,
        LoadingFlag::GameMods,
        LoadingFlag::GraphicMods,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            LoadingFlag::ReShade => "disable_reshade_loading",
            LoadingFlag::Migoto => "disable_3dmigoto_loading",
            LoadingFlag::GameMods => "disable_game_mods_loading",
            LoadingFlag::GraphicMods => "disable_graphic_mods_loading",
        }
    }
}

pub fn read(game: &GamePaths) -> toml::Table {
    match yp_core::fs::read_text_opt(&game.nams_toml()) {
        Ok(Some(raw)) if !raw.is_empty() => toml_ext::lenient_values(&raw),
        _ => toml::Table::new(),
    }
}

pub fn set(game: &GamePaths, key: &str, value: bool) -> Result<()> {
    let path = game.nams_toml();
    let Some(raw) = yp_core::fs::read_text_opt(&path)? else {
        return Ok(());
    };
    if raw.is_empty() {
        return Ok(());
    }
    if toml_ext::lenient_values(&raw).get(key).and_then(toml::Value::as_bool) == Some(value) {
        return Ok(());
    }
    let mut doc = yp_core::toml_io::parse(&raw).map_err(|err| yp_core::Error::TomlParse {
        path: Some(path.clone()),
        message: err.to_string(),
    })?;
    toml_ext::set_keeping_decor(doc.as_table_mut(), key, toml_edit::Value::from(value));
    yp_core::fs::write_text_atomic(&path, &doc.to_string())?;
    Ok(())
}

pub fn set_disabled(game: &GamePaths, flag: LoadingFlag, disabled: bool) -> Result<()> {
    set(game, flag.key(), disabled)
}

pub fn is_disabled(game: &GamePaths, flag: LoadingFlag) -> bool {
    read(game).get(flag.key()).and_then(toml::Value::as_bool) == Some(true)
}
