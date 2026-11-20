use std::path::PathBuf;

use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::error::Result;
use crate::toml_out::write_file;

pub const MOD_ID: &str = "my_first_mod";
pub const ITEM_FILE: &str = "item_9999.toml";
pub const MANIFEST_FILE: &str = "mod.toml";

pub const MANIFEST: &str = r#"id = "my_first_mod"
display_name = "My First Mod"
author = "Your Name"
version = "1.0.0"
"#;

pub const ITEM: &str = r#"name = "item_test_pebble"
id_type = 0
buy = -1
sell = -1
no_trash = true

[auto_give]
quantity = 1

[text.name]
_us = "Test Pebble"

[text.help]
_us = "A small stone. Proof your mod loaded."
"#;

pub fn mod_root(game: &GamePaths) -> PathBuf {
    game.mods_dir().join(MOD_ID)
}

pub fn manifest_path(game: &GamePaths) -> PathBuf {
    mod_root(game).join(MANIFEST_FILE)
}

pub fn entities_dir(game: &GamePaths) -> PathBuf {
    mod_root(game).join("entities")
}

pub fn item_path(game: &GamePaths) -> PathBuf {
    entities_dir(game).join(ITEM_FILE)
}

pub fn has_folder(game: &GamePaths) -> bool {
    mod_root(game).is_dir()
}

pub fn has_manifest(game: &GamePaths) -> bool {
    manifest_path(game).is_file()
}

pub fn has_item(game: &GamePaths) -> bool {
    item_path(game).is_file()
}

pub fn create_folder(game: &GamePaths) -> Result<()> {
    let dir = entities_dir(game);
    std::fs::create_dir_all(&dir).io(IoOp::Create, &dir)?;
    Ok(())
}

fn with_newline(text: &str) -> String {
    if text.ends_with('\n') { text.to_owned() } else { format!("{text}\n") }
}

pub fn write_manifest(game: &GamePaths, content: Option<&str>) -> Result<()> {
    write_file(&manifest_path(game), &with_newline(content.unwrap_or(MANIFEST)))?;
    Ok(())
}

pub fn write_item(game: &GamePaths, content: Option<&str>) -> Result<()> {
    write_file(&item_path(game), &with_newline(content.unwrap_or(ITEM)))?;
    Ok(())
}

pub fn read_manifest(game: &GamePaths) -> Result<String> {
    Ok(yp_core::fs::read_text_opt(&manifest_path(game))?.unwrap_or_else(|| MANIFEST.to_owned()))
}

pub fn read_item(game: &GamePaths) -> Result<String> {
    Ok(yp_core::fs::read_text_opt(&item_path(game))?.unwrap_or_else(|| ITEM.to_owned()))
}

pub fn remove_all(game: &GamePaths) -> Result<()> {
    let root = mod_root(game);
    if root.is_dir() {
        std::fs::remove_dir_all(&root).io(IoOp::Remove, &root)?;
    }
    Ok(())
}
