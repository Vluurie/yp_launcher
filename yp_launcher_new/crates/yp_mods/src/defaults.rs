use std::path::Path;

use toml_edit::Value;
use yp_core::GamePaths;

use crate::error::Result;
use crate::fsx;
use crate::model::OutfitChoice;
use crate::text::normalize_rel_path;
use crate::toml_doc::{inline_table, parse_document, read_values, set_root_value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefaultKind {
    OutfitBare,
    OutfitConfig,
    OutfitAnimation,
}

impl DefaultKind {
    pub const ALL: [DefaultKind; 3] = [
        DefaultKind::OutfitBare,
        DefaultKind::OutfitConfig,
        DefaultKind::OutfitAnimation,
    ];

    pub fn key(self) -> &'static str {
        match self {
            DefaultKind::OutfitBare => "outfit_bare",
            DefaultKind::OutfitConfig => "outfit_config",
            DefaultKind::OutfitAnimation => "outfit_animation",
        }
    }

    pub fn exclusive_group(self) -> Option<&'static str> {
        match self {
            DefaultKind::OutfitBare | DefaultKind::OutfitConfig => Some("outfit"),
            DefaultKind::OutfitAnimation => None,
        }
    }

    pub fn from_key(key: &str) -> Option<DefaultKind> {
        DefaultKind::ALL.into_iter().find(|kind| kind.key() == key)
    }

    pub fn conflicts_with(self, other: DefaultKind) -> bool {
        self.exclusive_group().is_some() && self.exclusive_group() == other.exclusive_group()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultEntry {
    pub path: String,
    pub kind: DefaultKind,
    pub outfit_id: Option<i64>,
}

impl DefaultEntry {
    pub fn new(path: impl Into<String>, kind: DefaultKind) -> Self {
        Self {
            path: path.into(),
            kind,
            outfit_id: None,
        }
    }
}

pub const FEATURE_KEY: &str = "experimental_default_outfits";

pub fn is_feature_enabled(game: &GamePaths) -> bool {
    let Ok(Some(text)) = yp_core::fs::read_text_opt(&game.nams_toml()) else {
        return false;
    };
    read_values(&text)
        .get(FEATURE_KEY)
        .and_then(toml::Value::as_bool)
        == Some(true)
}

pub fn default_file() -> String {
    let mut out = String::from(
        "# Mods listed here are active from the moment the game starts,\n\
         # exactly as if their files were placed in NieRAutomata/data.\n\
         # Restart the game after editing.\n\
         #\n\
         # Paths are relative to the nams/ folder. Prefix matching is used.\n\
         \n",
    );
    for kind in DefaultKind::ALL {
        out.push_str(kind.key());
        out.push_str(" = []\n");
    }
    out
}

pub fn normalize(rel_path: &str) -> String {
    normalize_rel_path(rel_path)
}

pub fn to_relative(game: &GamePaths, absolute_path: &Path) -> Option<String> {
    let relative = fsx::relative(absolute_path, &game.nams_dir())?;
    let text = relative.to_string_lossy();
    if text.starts_with("..") {
        return None;
    }
    Some(normalize(&text))
}

pub(crate) fn parse(raw: &str) -> Vec<DefaultEntry> {
    let values = read_values(raw);
    let mut out = Vec::new();
    for kind in DefaultKind::ALL {
        let Some(items) = values.get(kind.key()).and_then(toml::Value::as_array) else {
            continue;
        };
        for item in items {
            let (raw_path, outfit_id) = match item {
                toml::Value::String(path) => (Some(path.as_str()), None),
                toml::Value::Table(table) => (
                    table.get("path").and_then(toml::Value::as_str),
                    table.get("outfit_id").and_then(toml::Value::as_integer),
                ),
                _ => (None, None),
            };
            let Some(raw_path) = raw_path else {
                continue;
            };
            let path = normalize(raw_path);
            if path.is_empty() {
                continue;
            }
            out.push(DefaultEntry {
                path,
                kind,
                outfit_id,
            });
        }
    }
    out
}

pub fn list(game: &GamePaths) -> Result<Vec<DefaultEntry>> {
    let file = game.default_mods_toml();
    if !file.is_file() {
        return Ok(Vec::new());
    }
    Ok(yp_core::fs::read_text_opt(&file)?
        .map(|raw| parse(&raw))
        .unwrap_or_default())
}

pub fn set_default(
    game: &GamePaths,
    rel_path: &str,
    kind: Option<DefaultKind>,
    outfit_id: Option<i64>,
) -> Result<()> {
    let rel = normalize(rel_path);
    let file = game.default_mods_toml();
    let raw = if file.is_file() {
        yp_core::fs::read_text_opt(&file)?.unwrap_or_default()
    } else {
        default_file()
    };

    let mut entries = parse(&raw);
    entries.retain(|entry| entry.path != rel);
    if let Some(kind) = kind {
        if let Some(stem) = stem_of(&rel) {
            entries.retain(|entry| {
                !(kind.conflicts_with(entry.kind) && stem_of(&entry.path).as_deref() == Some(&stem))
            });
        }
        entries.push(DefaultEntry {
            path: rel,
            kind,
            outfit_id,
        });
    }

    let mut doc = parse_document(&raw, &file)?;
    for list_kind in DefaultKind::ALL {
        let items = entries
            .iter()
            .filter(|entry| entry.kind == list_kind)
            .map(|entry| match entry.outfit_id {
                None => Value::from(entry.path.as_str()),
                Some(id) => inline_table(vec![
                    ("path", Value::from(entry.path.as_str())),
                    ("outfit_id", Value::from(id)),
                ]),
            });
        set_root_value(&mut doc, list_kind.key(), Value::Array(items.collect()));
    }
    yp_core::fs::write_text_atomic(&file, &doc.to_string())?;
    Ok(())
}

pub fn entry_for<'a>(entries: &'a [DefaultEntry], rel_path: &str) -> Option<&'a DefaultEntry> {
    let rel = normalize(rel_path);
    entries.iter().find(|entry| {
        rel == entry.path
            || rel
                .strip_prefix(entry.path.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

pub fn matches(entries: &[DefaultEntry], rel_path: &str) -> bool {
    entry_for(entries, rel_path).is_some()
}

pub fn stem_of(entry: &str) -> Option<String> {
    let normalized = normalize(entry);
    let parts: Vec<&str> = normalized.split('/').collect();
    if parts.len() < 3 {
        return None;
    }
    parts.last().map(|last| (*last).to_owned())
}

pub fn is_outfit_stem(stem: &str) -> bool {
    let lower = stem.to_lowercase();
    if !lower.starts_with("pl") || crate::text::utf16_len(&lower) < 3 {
        return false;
    }
    !lower.ends_with('f')
}

pub fn kind_for(stem: &str, has_outfit_config: bool) -> DefaultKind {
    if !is_outfit_stem(stem) {
        return DefaultKind::OutfitAnimation;
    }
    if has_outfit_config {
        DefaultKind::OutfitConfig
    } else {
        DefaultKind::OutfitBare
    }
}

pub fn initial_outfit_id(kind: DefaultKind, choices: &[OutfitChoice]) -> Option<i64> {
    if kind != DefaultKind::OutfitConfig {
        return None;
    }
    if choices.iter().any(|choice| !choice.needs_item) {
        return None;
    }
    let mut item_outfits = choices.iter().filter(|choice| choice.needs_item);
    let first = item_outfits.next()?;
    if item_outfits.next().is_some() {
        return None;
    }
    Some(first.outfit_id)
}
