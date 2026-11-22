use std::path::{Path, PathBuf};

use serde_json::json;
use yp_core::GamePaths;

use crate::fsx;
use crate::pack_name::parse_pulled_out;
use crate::probe::is_hidden_name;
use crate::profiles::{active_name, profile_mods_dir};
use crate::texture_config::prune_file_quiet;

pub const SIDECAR_NAME: &str = ".bundled_textures.json";

pub fn sidecar_path(mod_dir: &Path) -> PathBuf {
    mod_dir.join(SIDECAR_NAME)
}

fn sidecar_values(sidecar: &Path) -> Option<Vec<serde_json::Value>> {
    let text = std::fs::read_to_string(sidecar).ok()?;
    let decoded: serde_json::Value = serde_json::from_str(&text).ok()?;
    decoded.get("packs")?.as_array().cloned()
}

pub fn read_packs_file(sidecar: &Path) -> Vec<String> {
    sidecar_values(sidecar)
        .unwrap_or_default()
        .iter()
        .filter_map(serde_json::Value::as_str)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn read_packs(mod_dir: &Path) -> Vec<String> {
    read_packs_file(&sidecar_path(mod_dir))
}

pub(crate) fn render_packs(packs: &[String]) -> String {
    json!({ "packs": packs }).to_string()
}

pub fn scan_cutscenes(mod_dir: &Path) -> Vec<String> {
    let dir = mod_dir.join("cutscenes");
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut out: Vec<String> = fsx::child_dirs(&dir)
        .into_iter()
        .filter(|entry| !is_hidden_name(&entry.name))
        .filter(|entry| {
            let movie = entry.path.join("movie");
            movie.is_dir()
                && fsx::child_files(&movie)
                    .iter()
                    .any(|file| fsx::lower_path_ends_with(&file.path, ".usm"))
        })
        .map(|entry| entry.name)
        .collect();
    out.sort_by(|a, b| crate::text::utf16_cmp(a, b));
    out
}

pub fn migrate_in_place(game: &GamePaths) -> usize {
    let inject_root = game.inject_textures_dir();
    if !inject_root.is_dir() {
        return 0;
    }
    let active = active_name(game);
    let mut migrated: Vec<String> = Vec::new();
    for entity in fsx::child_dirs(&inject_root) {
        let pack_name = entity.name.clone();
        let Some(parsed) = parse_pulled_out(&pack_name) else {
            continue;
        };
        let mod_root = if parsed.profile == active {
            game.mods_dir().join(parsed.mod_id)
        } else {
            profile_mods_dir(game, parsed.profile).join(parsed.mod_id)
        };
        if !mod_root.is_dir() {
            continue;
        }
        let dest_dir = mod_root.join("textures");
        let mut dest = dest_dir.join(parsed.base);
        if dest.is_dir() || dest.is_file() {
            dest = dest_dir.join(&pack_name);
            if dest.is_dir() {
                continue;
            }
        }
        if std::fs::create_dir_all(&dest_dir).is_err()
            || fsx::move_dir(&entity.path, &dest).is_err()
        {
            continue;
        }
        migrated.push(pack_name.clone());
        drop_pack_from_sidecar(&sidecar_path(&mod_root), &pack_name);
    }
    if migrated.is_empty() {
        return 0;
    }
    prune_file_quiet(&game.texture_injection_toml(), &migrated);
    migrated.len()
}

fn drop_pack_from_sidecar(sidecar: &Path, pack_name: &str) {
    if !sidecar.is_file() {
        return;
    }
    let Some(values) = sidecar_values(sidecar) else {
        return;
    };
    let remaining: Vec<String> = values
        .iter()
        .filter_map(serde_json::Value::as_str)
        .filter(|name| *name != pack_name)
        .map(str::to_owned)
        .collect();
    if remaining.is_empty() {
        let _ = std::fs::remove_file(sidecar);
    } else {
        let _ = std::fs::write(sidecar, render_packs(&remaining));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_round_trip_skips_non_strings_and_blanks() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(sidecar_path(dir.path()), r#"{"packs":["a","",3,"b"]}"#).unwrap();
        assert_eq!(read_packs(dir.path()), vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(render_packs(&["x".to_owned()]), r#"{"packs":["x"]}"#);
        std::fs::write(sidecar_path(dir.path()), "{broken").unwrap();
        assert!(read_packs(dir.path()).is_empty());
    }

    #[test]
    fn lists_cutscene_folders_with_usm_movies() {
        let dir = tempfile::tempdir().unwrap();
        for (name, file) in [
            ("b", "x.USM"),
            ("a", "y.usm"),
            ("_hidden", "z.usm"),
            ("c", "readme.txt"),
        ] {
            let movie = dir.path().join("cutscenes").join(name).join("movie");
            std::fs::create_dir_all(&movie).unwrap();
            std::fs::write(movie.join(file), b"").unwrap();
        }
        assert_eq!(
            scan_cutscenes(dir.path()),
            vec!["a".to_owned(), "b".to_owned()]
        );
    }

    #[test]
    fn migrates_pulled_out_packs_back_into_their_mod() {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path());
        let mod_root = game.mods_dir().join("kimono");
        std::fs::create_dir_all(&mod_root).unwrap();
        std::fs::write(
            sidecar_path(&mod_root),
            r#"{"packs":["HD (kimono@default)","Other"]}"#,
        )
        .unwrap();
        let pack = game.inject_textures_dir().join("HD (kimono@default)");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::write(pack.join("a.dds"), b"x").unwrap();
        std::fs::write(
            game.texture_injection_toml(),
            "load_order = [\"HD (kimono@default)\", \"Keep\"]\ndisabled_packs = []\n",
        )
        .unwrap();

        assert_eq!(migrate_in_place(&game), 1);
        assert!(mod_root.join("textures").join("HD").join("a.dds").is_file());
        assert!(!pack.exists());
        assert_eq!(read_packs(&mod_root), vec!["Other".to_owned()]);
        let toml = std::fs::read_to_string(game.texture_injection_toml()).unwrap();
        assert!(toml.contains("load_order = [\"Keep\"]"));
    }
}
