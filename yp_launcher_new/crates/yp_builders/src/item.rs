use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use toml_edit::Value;
use walkdir::WalkDir;
use yp_core::GamePaths;

use crate::error::Result;
use crate::toml_out::{basic_string, slug, text_block, write_file, TomlOut};

pub const FIRST_FREE_ID: i64 = 8000;
pub const ID_CEILING: i64 = 60000;
pub const UNNAMED: &str = "item_unnamed";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDraft {
    pub display_name: String,
    pub description: String,
    pub id: i64,
    pub max_carry: i64,
    pub sellable: bool,
    pub sell_price: i64,
    pub auto_give: bool,
    pub misctex_name: Option<String>,
    pub buy_price: Option<i64>,
    pub category: i64,
}

impl ItemDraft {
    pub fn new(display_name: impl Into<String>, description: impl Into<String>, id: i64) -> Self {
        Self {
            display_name: display_name.into(),
            description: description.into(),
            id,
            max_carry: 1,
            sellable: false,
            sell_price: 100,
            auto_give: true,
            misctex_name: None,
            buy_price: None,
            category: 0,
        }
    }

    pub fn internal_name(&self) -> String {
        internal_name_for(&self.display_name)
    }
}

pub fn internal_name_for(display_name: &str) -> String {
    let name = slug(display_name);
    if name.is_empty() { UNNAMED.to_owned() } else { format!("item_{name}") }
}

pub fn to_toml(draft: &ItemDraft) -> String {
    let mut out = TomlOut::new();
    out.set("name", basic_string(&draft.internal_name()));
    out.set("id_type", Value::from(0));
    if draft.category != 0 {
        out.set("category", Value::from(draft.category));
    }
    if draft.max_carry != 1 {
        out.set("max_num", Value::from(draft.max_carry));
    }
    if draft.sellable {
        out.set("sell", Value::from(draft.sell_price));
    } else {
        out.set("no_trash", Value::from(true));
    }
    if let Some(buy) = draft.buy_price {
        out.set("buy", Value::from(buy));
    }
    if draft.auto_give {
        out.header(&["auto_give"], vec![("quantity", Value::from(1))]);
    }
    if let Some(misctex) = draft.misctex_name.as_deref().filter(|name| !name.is_empty()) {
        out.header(&["icon"], vec![("misctex_name", basic_string(misctex))]);
    }
    out.header(&["text", "name"], vec![("_us", basic_string(&draft.display_name))]);
    out.header(&["text", "help"], vec![("_us", text_block(&draft.description))]);
    out.finish()
}

pub fn file_name_for(draft: &ItemDraft) -> String {
    format!("item_{}.toml", draft.id)
}

pub fn entities_dir(game: &GamePaths, mod_id: &str) -> PathBuf {
    game.mods_dir().join(mod_id).join("entities")
}

pub fn path_for(game: &GamePaths, mod_id: &str, draft: &ItemDraft) -> PathBuf {
    entities_dir(game, mod_id).join(file_name_for(draft))
}

pub fn next_free_id(game: &GamePaths) -> i64 {
    let taken = taken_ids(game);
    let mut id = FIRST_FREE_ID;
    while taken.contains(&id) && id < ID_CEILING {
        id += 1;
    }
    id
}

pub fn taken_ids(game: &GamePaths) -> BTreeSet<i64> {
    mod_file_names(&game.mods_dir()).iter().filter_map(|name| id_from_file_name(name)).collect()
}

pub fn id_from_file_name(file_name: &str) -> Option<i64> {
    numbered_id(file_name, "item_")
}

pub fn write(game: &GamePaths, mod_id: &str, draft: &ItemDraft) -> Result<PathBuf> {
    Ok(write_file(&path_for(game, mod_id, draft), &to_toml(draft))?)
}

pub(crate) fn numbered_id(file_name: &str, prefix: &str) -> Option<i64> {
    file_name.strip_prefix(prefix)?.strip_suffix(".toml")?.parse().ok()
}

pub(crate) fn mod_file_names(root: &Path) -> Vec<String> {
    mod_files(root).iter().map(|path| yp_core::fs::file_name_lossy(path)).collect()
}

pub(crate) fn mod_files(root: &Path) -> Vec<PathBuf> {
    if !root.is_dir() {
        return Vec::new();
    }
    WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect()
}
