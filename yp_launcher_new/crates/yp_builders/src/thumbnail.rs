use std::path::{Path, PathBuf};

use yp_core::{GamePaths, IoOp, IoResultExt};
use yp_formats::files::{MiscTexResult, TexturePreview};

use crate::error::Result;
use crate::outfit::dotted_extension;

pub const DEFAULT_WIDTH: u32 = 256;
pub const DEFAULT_HEIGHT: u32 = 128;
pub const STORY_WIDTH: u32 = 800;
pub const STORY_HEIGHT: u32 = 560;

pub const ACCEPTED_EXTENSIONS: [&str; 6] = [".png", ".jpg", ".jpeg", ".webp", ".bmp", ".tga"];

const MISCTEX_EXTENSIONS: [&str; 2] = [".dat", ".dtt"];

pub fn is_accepted(path: &Path) -> bool {
    ACCEPTED_EXTENSIONS.contains(&dotted_extension(path).as_str())
}

pub fn texture_name_for(item_id: i64) -> String {
    format!("item_thumb_{item_id}")
}

pub fn misctex_name_for(item_id: i64) -> String {
    format!("misctex_{}", texture_name_for(item_id))
}

pub fn output_dir_for(game: &GamePaths, mod_id: &str) -> PathBuf {
    game.mods_dir().join(mod_id).join("data").join("misctex")
}

pub fn create(
    source_image: &Path,
    game: &GamePaths,
    mod_id: &str,
    item_id: i64,
    width: u32,
    height: u32,
) -> Result<MiscTexResult> {
    create_named(source_image, &output_dir_for(game, mod_id), &texture_name_for(item_id), width, height)
}

pub fn create_named(
    source_image: &Path,
    output_dir: &Path,
    texture_name: &str,
    width: u32,
    height: u32,
) -> Result<MiscTexResult> {
    Ok(yp_formats::files::create_item_thumbnail(source_image, output_dir, texture_name, width, height)?)
}

pub fn preview(dtt_path: &Path) -> Result<TexturePreview> {
    Ok(yp_formats::files::read_misctex_preview(dtt_path)?)
}

pub fn exists(game: &GamePaths, mod_id: &str, item_id: i64) -> bool {
    output_dir_for(game, mod_id).join(format!("{}.dat", misctex_name_for(item_id))).is_file()
}

pub fn remove(game: &GamePaths, mod_id: &str, item_id: i64) -> Result<()> {
    let dir = output_dir_for(game, mod_id);
    let stem = misctex_name_for(item_id);
    for extension in MISCTEX_EXTENSIONS {
        let file = dir.join(format!("{stem}{extension}"));
        if file.is_file() {
            std::fs::remove_file(&file).io(IoOp::Remove, &file)?;
        }
    }
    Ok(())
}
