use yp_core::GamePaths;

use crate::error::Result;
use crate::text::normalize_rel_path;
use crate::toml_doc::{parse_document, read_values, set_root_value, string_array, string_list};

pub const DEFAULT_FILE: &str = "# Decides which mod wins when two mods replace the same game file.
# Mods not listed here keep no defined order between each other.
# Restart the game after editing.
#
# Later entries override earlier ones, so the LAST entry wins a conflict.
#
# Paths are relative to the nams/ folder. Prefix matching is used.

load_order = []
";

pub fn normalize(rel_path: &str) -> String {
    normalize_rel_path(rel_path)
}

pub(crate) fn parse(raw: &str) -> Vec<String> {
    string_list(read_values(raw).get("load_order"))
        .unwrap_or_default()
        .iter()
        .map(|entry| normalize(entry))
        .filter(|entry| !entry.is_empty())
        .collect()
}

pub fn list(game: &GamePaths) -> Result<Vec<String>> {
    let file = game.mod_load_order_toml();
    if !file.is_file() {
        return Ok(Vec::new());
    }
    Ok(yp_core::fs::read_text_opt(&file)?
        .map(|raw| parse(&raw))
        .unwrap_or_default())
}

pub fn save(game: &GamePaths, order: &[String]) -> Result<()> {
    let order: Vec<String> = order
        .iter()
        .map(|entry| normalize(entry))
        .filter(|entry| !entry.is_empty())
        .collect();
    let file = game.mod_load_order_toml();
    let raw = if file.is_file() {
        yp_core::fs::read_text_opt(&file)?.unwrap_or_default()
    } else {
        DEFAULT_FILE.to_owned()
    };
    let mut doc = parse_document(&raw, &file)?;
    set_root_value(&mut doc, "load_order", string_array(order));
    yp_core::fs::write_text_atomic(&file, &doc.to_string())?;
    Ok(())
}
