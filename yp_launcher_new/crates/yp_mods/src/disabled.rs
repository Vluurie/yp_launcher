use std::path::Path;

use yp_core::GamePaths;

use crate::error::Result;
use crate::fsx;
use crate::text::normalize_rel_path;
use crate::toml_doc::{parse_document, read_values, set_root_value, string_array, string_list};

pub const DEFAULT_FILE: &str = "# Mods listed here are completely ignored at boot.
# NAMS does not mount, scan, or load anything inside a disabled path.
# Restart the game after editing.
#
# Paths are relative to the nams/ folder. Prefix matching is used, so
# disabling a folder also disables everything inside it.

disabled = []
";

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

pub(crate) fn parse(raw: &str) -> Vec<String> {
    string_list(read_values(raw).get("disabled"))
        .unwrap_or_default()
        .iter()
        .map(|entry| normalize(entry))
        .filter(|entry| !entry.is_empty())
        .collect()
}

pub fn list(game: &GamePaths) -> Result<Vec<String>> {
    let file = game.disabled_mods_toml();
    if !file.is_file() {
        return Ok(Vec::new());
    }
    Ok(yp_core::fs::read_text_opt(&file)?
        .map(|raw| parse(&raw))
        .unwrap_or_default())
}

pub fn set_disabled(game: &GamePaths, rel_path: &str, disabled: bool) -> Result<()> {
    let rel = normalize(rel_path);
    let file = game.disabled_mods_toml();
    let raw = if file.is_file() {
        yp_core::fs::read_text_opt(&file)?.unwrap_or_default()
    } else {
        DEFAULT_FILE.to_owned()
    };
    let mut entries = parse(&raw);
    if disabled {
        if !entries.contains(&rel) {
            entries.push(rel);
        }
    } else if let Some(index) = entries.iter().position(|entry| *entry == rel) {
        entries.remove(index);
    }
    let mut doc = parse_document(&raw, &file)?;
    set_root_value(&mut doc, "disabled", string_array(entries));
    yp_core::fs::write_text_atomic(&file, &doc.to_string())?;
    Ok(())
}

pub fn matches(entries: &[String], rel_path: &str) -> bool {
    let rel = normalize(rel_path);
    entries.iter().any(|entry| {
        rel == *entry
            || rel
                .strip_prefix(entry.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_entries_and_keeps_header() {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path());
        assert!(list(&game).unwrap().is_empty());
        set_disabled(&game, "mods\\alpha", true).unwrap();
        set_disabled(&game, "mods/alpha", true).unwrap();
        set_disabled(&game, "mods/beta", true).unwrap();
        assert_eq!(
            list(&game).unwrap(),
            vec!["mods/alpha".to_owned(), "mods/beta".to_owned()]
        );
        set_disabled(&game, "mods/alpha", false).unwrap();
        assert_eq!(list(&game).unwrap(), vec!["mods/beta".to_owned()]);
        let text = std::fs::read_to_string(game.disabled_mods_toml()).unwrap();
        assert!(text.starts_with("# Mods listed here are completely ignored at boot."));
        assert!(text.contains("disabled = [\"mods/beta\"]"));
    }

    #[test]
    fn prefix_matching_respects_segment_boundaries() {
        let entries = vec!["mods/a".to_owned()];
        assert!(matches(&entries, "mods/a"));
        assert!(matches(&entries, "mods/a/pl"));
        assert!(!matches(&entries, "mods/ab"));
    }

    #[test]
    fn relative_paths_stay_inside_nams() {
        let game = GamePaths::new("G");
        assert_eq!(
            to_relative(&game, &game.mods_dir().join("x")).as_deref(),
            Some("mods/x")
        );
        assert_eq!(
            to_relative(&game, Path::new("G").join("data").as_path()),
            None
        );
    }

    #[test]
    fn broken_file_lists_nothing_and_refuses_to_write() {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path());
        std::fs::create_dir_all(game.nams_dir()).unwrap();
        std::fs::write(game.disabled_mods_toml(), "disabled = = [").unwrap();
        assert!(list(&game).unwrap().is_empty());
        assert!(set_disabled(&game, "mods/a", true).is_err());
    }
}
