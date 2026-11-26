use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use jiff::Timestamp;
use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::bundled;
use crate::error::Result;
use crate::fsx;
use crate::manifest;
use crate::model::{
    DataArchivePair, DataCategory, DataDirEntry, DataSummary, InstalledMod, ModConflict,
    ModConflictKind, ModKind, NativeSummary, OutfitChoice, PlayerModelEntry,
};
use crate::normalize::misplaced_wax_config_dirs;
use crate::probe::{
    classify_kind, has_compat_config, has_outfit_config, is_cpk_file, is_hidden_name,
    misplaced_config_wrapper_dirs, unwrap_single_child,
};
use crate::tables::{data_dir_category, player_model_label};
use crate::text::{strip_ascii_suffix, utf16_cmp};

const OUTFIT_LISTS_BY_STEM: [(&str, [&str; 2]); 3] = [
    ("pl_2b_outfits", ["pl000d", "pl0000"]),
    ("pl_9s_outfits", ["pl020d", "pl0200"]),
    ("pl_a2_outfits", ["pl010d", "pl0100"]),
];

const CONDITIONALLY_MOUNTED_DIRS: [&str; 1] = ["pl"];

const ID_BUNDLE_KINDS: [&str; 14] = [
    "weapon_patch",
    "voice_bank",
    "shop_item",
    "ruby_patch",
    "char_name",
    "hairspray",
    "accessory",
    "outfit",
    "weapon",
    "reward",
    "quest",
    "item",
    "mail",
    "shop",
];

const FIXED_ENTITY_KINDS: [(&str, &str); 12] = [
    ("effects", "effect_area"),
    ("hairspray", "hairspray"),
    ("meshes", "mesh_visibility"),
    ("music", "music"),
    ("scenario", "scenario_variables"),
    ("phase", "phase"),
    ("graphics", "graphic_adjust"),
    ("hap_groups", "hap_group"),
    ("maps", "custom_map"),
    ("shops", "shop"),
    ("rubypatches", "ruby_patch"),
    ("camera", "player_camera_settings"),
];

pub fn list_installed(game: &GamePaths) -> Result<Vec<InstalledMod>> {
    let mods_root = game.mods_dir();
    if !mods_root.is_dir() {
        return Ok(Vec::new());
    }
    std::fs::read_dir(&mods_root).io(IoOp::List, &mods_root)?;
    let mut mods: Vec<InstalledMod> = fsx::children(&mods_root)
        .into_iter()
        .filter(|entry| entry.is_dir && !is_hidden_name(&entry.name))
        .map(|entry| scan_installed_mod(&entry.path, &entry.name))
        .collect();
    mods.sort_by_key(|installed| std::cmp::Reverse(installed.installed_at));
    Ok(compute_conflicts(mods))
}

fn scan_installed_mod(root_path: &Path, id: &str) -> InstalledMod {
    let content_root = unwrap_single_child(root_path);
    misplaced_wax_config_dirs(&content_root);
    let manifest = manifest::load(&content_root);
    let kind = classify_kind(&content_root);
    let native = (kind == ModKind::Native).then(|| scan_native(&content_root.join("entities")));
    let data = scan_data_if_present(&content_root);
    InstalledMod {
        id: id.to_owned(),
        display_name: manifest
            .as_ref()
            .and_then(|manifest| manifest.display_name.clone())
            .unwrap_or_else(|| id.to_owned()),
        root_path: root_path.to_path_buf(),
        kind,
        manifest,
        native,
        data,
        requires_missing: Vec::new(),
        conflicts: Vec::new(),
        installed_at: fsx::mtime(root_path).unwrap_or(Timestamp::UNIX_EPOCH),
        bundled_texture_packs: bundled::read_packs(root_path),
        bundled_cutscenes: bundled::scan_cutscenes(root_path),
    }
}

pub(crate) fn scan_data_if_present(content_root: &Path) -> Option<DataSummary> {
    let data_dir = content_root.join("data");
    let has_compat = has_compat_config(content_root);
    let outfits = scan_outfit_choices(content_root);
    if !data_dir.is_dir() && !has_compat && outfits.is_empty() {
        return None;
    }
    Some(scan_data(
        &data_dir,
        content_root,
        has_compat,
        has_outfit_config(content_root),
        outfits,
    ))
}

pub(crate) fn scan_native(entities_dir: &Path) -> NativeSummary {
    if !entities_dir.is_dir() {
        return NativeSummary::default();
    }
    let mut counts: IndexMap<String, usize> = IndexMap::new();
    let mut total = 0usize;
    for file in fsx::walk_files(entities_dir) {
        let is_toml = fsx::lower_path_ends_with(&file, ".toml");
        let is_json = fsx::lower_path_ends_with(&file, ".json");
        if !is_toml && !is_json {
            continue;
        }
        total += 1;
        let bucket = peek_bundle_kind(&file, is_toml).unwrap_or_else(|| "unknown".to_owned());
        *counts.entry(bucket).or_insert(0) += 1;
    }
    NativeSummary {
        bundles_by_kind: counts,
        total_entity_files: total,
    }
}

fn peek_bundle_kind(file: &Path, is_toml: bool) -> Option<String> {
    if let Some(from_name) = bundle_kind_from_file_name(&fsx::file_name(file)) {
        return Some(from_name);
    }
    if is_toml {
        return None;
    }
    let text = std::fs::read_to_string(file).ok()?;
    let decoded: serde_json::Value = serde_json::from_str(&text).ok()?;
    match &decoded {
        serde_json::Value::Object(map) => map
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .map(normalize_kind),
        serde_json::Value::Array(items) => items
            .first()
            .and_then(serde_json::Value::as_object)
            .and_then(|first| first.get("kind"))
            .and_then(serde_json::Value::as_str)
            .map(normalize_kind),
        _ => None,
    }
}

fn normalize_kind(kind: &str) -> String {
    let lower = kind.to_lowercase();
    match lower.strip_suffix("_bundle") {
        Some(stripped) => stripped.to_owned(),
        None => lower,
    }
}

pub fn bundle_kind_from_file_name(file_name: &str) -> Option<String> {
    let lower = file_name.to_lowercase();
    let stem = strip_ascii_suffix(&lower, ".toml")
        .or_else(|| strip_ascii_suffix(&lower, ".json"))
        .unwrap_or(&lower);
    if stem.is_empty() {
        return None;
    }
    if let Some((_, fixed)) = FIXED_ENTITY_KINDS.iter().find(|(name, _)| *name == stem) {
        return Some((*fixed).to_owned());
    }
    for kind in ID_BUNDLE_KINDS {
        if stem == kind
            || stem
                .strip_prefix(kind)
                .is_some_and(|rest| rest.starts_with('_'))
        {
            return Some(kind.to_owned());
        }
    }
    let is_numbered_shop = stem
        .strip_prefix("shop")
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()));
    is_numbered_shop.then(|| "shop".to_owned())
}

fn count_asset_entries(dir: &Path) -> usize {
    let mut stems: IndexSet<String> = IndexSet::new();
    let mut other = 0usize;
    for file in fsx::walk_files(dir) {
        let base = fsx::file_name(&file);
        let stem = strip_ascii_suffix(&base, ".dat").or_else(|| strip_ascii_suffix(&base, ".dtt"));
        match stem {
            Some(stem) => {
                let parent = file.parent().map(Path::to_path_buf).unwrap_or_default();
                stems.insert(format!("{}/{stem}", parent.to_string_lossy()));
            }
            None => other += 1,
        }
    }
    stems.len() + other
}

struct ArchiveFiles {
    stem: String,
    dat: Option<PathBuf>,
    dtt: Option<PathBuf>,
}

pub(crate) fn scan_data(
    data_dir: &Path,
    config_root: &Path,
    has_compat_config: bool,
    has_outfit_config: bool,
    outfits_by_stem: IndexMap<String, Vec<OutfitChoice>>,
) -> DataSummary {
    let mut entries: Vec<DataDirEntry> = Vec::new();
    let mut players: Vec<PlayerModelEntry> = Vec::new();
    let mut archives: Vec<DataArchivePair> = Vec::new();

    if data_dir.is_dir() {
        let cpk_count = fsx::child_files(data_dir)
            .iter()
            .filter(|file| is_cpk_file(&file.name))
            .count();
        if cpk_count > 0 {
            entries.push(DataDirEntry {
                dir_name: "cpk".to_owned(),
                category: DataCategory::Archive,
                file_count: cpk_count,
            });
        }

        for sub in fsx::child_dirs(data_dir) {
            let category =
                data_dir_category(&sub.name.to_lowercase()).unwrap_or(DataCategory::Other);
            entries.push(DataDirEntry {
                dir_name: sub.name.clone(),
                category,
                file_count: count_asset_entries(&sub.path),
            });
            if category == DataCategory::Player {
                for file in fsx::child_files(&sub.path) {
                    let Some(stem) = strip_ascii_suffix(&file.name, ".dat") else {
                        continue;
                    };
                    let label = player_model_label(&stem.to_lowercase())
                        .map_or_else(|| file.name.clone(), str::to_owned);
                    players.push(PlayerModelEntry {
                        file_name: file.name.clone(),
                        label,
                    });
                }
            }
        }

        let mut archive_files: IndexMap<String, ArchiveFiles> = IndexMap::new();
        for file in fsx::walk_files(data_dir) {
            let base = fsx::file_name(&file);
            let (stem, is_dat) = match strip_ascii_suffix(&base, ".dat") {
                Some(stem) => (stem, true),
                None => match strip_ascii_suffix(&base, ".dtt") {
                    Some(stem) => (stem, false),
                    None => continue,
                },
            };
            let parent = file.parent().map(Path::to_path_buf).unwrap_or_default();
            let key = format!(
                "{}/{}",
                parent.to_string_lossy().to_lowercase(),
                stem.to_lowercase()
            );
            let current = archive_files.entry(key).or_insert_with(|| ArchiveFiles {
                stem: stem.to_owned(),
                dat: None,
                dtt: None,
            });
            if is_dat {
                current.dat = Some(file.clone());
            } else {
                current.dtt = Some(file.clone());
            }
        }
        for pair in archive_files.into_values() {
            let label = player_model_label(&pair.stem.to_lowercase())
                .map_or_else(|| pair.stem.clone(), str::to_owned);
            archives.push(DataArchivePair {
                stem: pair.stem,
                label,
                config_root: config_root.to_path_buf(),
                dat_path: pair.dat,
                dtt_path: pair.dtt,
            });
        }
    }

    entries.sort_by(|a, b| utf16_cmp(&a.dir_name, &b.dir_name));
    players.sort_by(|a, b| utf16_cmp(&a.file_name, &b.file_name));
    archives.sort_by(|a, b| utf16_cmp(&a.label, &b.label));
    DataSummary {
        entries,
        players,
        archives,
        has_compat_config,
        has_outfit_config,
        outfits_by_stem,
    }
}

pub(crate) fn scan_outfit_choices(content_root: &Path) -> IndexMap<String, Vec<OutfitChoice>> {
    let mut by_stem: IndexMap<String, Vec<OutfitChoice>> = IndexMap::new();
    let mut read_dir = |root: &Path| {
        let entities = root.join("entities");
        if entities.is_dir() {
            let mut files = fsx::walk_files(&entities);
            files.sort();
            for file in files {
                if !fsx::file_name(&file)
                    .to_ascii_lowercase()
                    .starts_with("outfit_")
                    || !fsx::lower_path_ends_with(&file, ".toml")
                {
                    continue;
                }
                let Some(text) = yp_core::fs::read_text_opt(&file).ok().flatten() else {
                    continue;
                };
                let values = crate::toml_doc::read_values(&text);
                let Some(id) = values.get("outfit_id").and_then(toml::Value::as_integer) else {
                    continue;
                };
                let stems = match values.get("character").and_then(toml::Value::as_str) {
                    Some("2B") => OUTFIT_LISTS_BY_STEM[0].1,
                    Some("9S") => OUTFIT_LISTS_BY_STEM[1].1,
                    Some("A2") => OUTFIT_LISTS_BY_STEM[2].1,
                    _ => continue,
                };
                let choice = OutfitChoice {
                    outfit_id: id,
                    name: values
                        .get("name")
                        .and_then(toml::Value::as_str)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned),
                    needs_item: values
                        .get("item_id")
                        .and_then(toml::Value::as_integer)
                        .is_some(),
                };
                for stem in stems {
                    by_stem
                        .entry(stem.to_owned())
                        .or_default()
                        .push(choice.clone());
                }
            }
        }
        let dir = root.join("outfits");
        if !dir.is_dir() {
            return;
        }
        let mut files: Vec<PathBuf> = fsx::walk_files(&dir)
            .into_iter()
            .filter(|file| fsx::lower_path_ends_with(file, ".json"))
            .collect();
        files.sort_by(|a, b| utf16_cmp(&a.to_string_lossy(), &b.to_string_lossy()));
        for file in files {
            let Some(serde_json::Value::Object(raw)) = std::fs::read_to_string(&file)
                .ok()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            else {
                continue;
            };
            for (list_key, stems) in OUTFIT_LISTS_BY_STEM {
                let Some(list) = raw.get(list_key).and_then(serde_json::Value::as_array) else {
                    continue;
                };
                let choices: Vec<OutfitChoice> = list
                    .iter()
                    .filter_map(serde_json::Value::as_object)
                    .filter_map(|outfit| {
                        let id = outfit
                            .get("outfit_id")
                            .and_then(serde_json::Value::as_i64)?;
                        let name = outfit
                            .get("outfit_name")
                            .and_then(serde_json::Value::as_str)
                            .filter(|name| !name.is_empty())
                            .map(str::to_owned);
                        Some(OutfitChoice {
                            outfit_id: id,
                            name,
                            needs_item: outfit.get("item_id").is_some_and(|item| !item.is_null()),
                        })
                    })
                    .collect();
                if choices.is_empty() {
                    continue;
                }
                for stem in stems {
                    by_stem
                        .entry(stem.to_owned())
                        .or_default()
                        .extend(choices.iter().cloned());
                }
            }
        }
    };

    let wrapped_root = content_root.join("wax").join("mods");
    if wrapped_root.is_dir() {
        for inner in fsx::child_dirs(&wrapped_root) {
            read_dir(&inner.path);
        }
    }
    read_dir(content_root);
    for wrapper in misplaced_config_wrapper_dirs(content_root) {
        read_dir(&wrapper);
    }

    for list in by_stem.values_mut() {
        list.sort_by_key(|choice| choice.outfit_id);
        list.dedup();
    }
    by_stem
}

fn collect_data_file_stamps(root_path: &Path) -> IndexMap<String, i64> {
    let content_root = unwrap_single_child(root_path);
    let data_dir = content_root.join("data");
    let mut files = IndexMap::new();
    if !data_dir.is_dir() {
        return files;
    }
    for file in fsx::walk_files(&data_dir) {
        let Ok(relative) = file.strip_prefix(&data_dir) else {
            continue;
        };
        let parts: Vec<String> = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect();
        let rel = parts.join("/");
        let first_segment = parts
            .first()
            .map(|part| part.to_lowercase())
            .unwrap_or_default();
        if CONDITIONALLY_MOUNTED_DIRS.contains(&first_segment.as_str()) {
            continue;
        }
        let size = std::fs::metadata(&file)
            .ok()
            .and_then(|meta| i64::try_from(meta.len()).ok())
            .unwrap_or(-1);
        files.insert(format!("data/{rel}"), size);
    }
    files
}

fn compute_conflicts(mods: Vec<InstalledMod>) -> Vec<InstalledMod> {
    let stamps: Vec<IndexMap<String, i64>> = mods
        .iter()
        .map(|installed| collect_data_file_stamps(&installed.root_path))
        .collect();
    let installed_ids: IndexSet<&str> =
        mods.iter().map(|installed| installed.id.as_str()).collect();

    let mut conflicts_by_mod: Vec<Vec<ModConflict>> = Vec::with_capacity(mods.len());
    let mut missing_by_mod: Vec<Vec<String>> = Vec::with_capacity(mods.len());
    for (index, installed) in mods.iter().enumerate() {
        let mut conflicts = Vec::new();
        for (other_index, other) in mods.iter().enumerate() {
            if other.id == installed.id {
                continue;
            }
            for (path, size) in &stamps[index] {
                let Some(their_size) = stamps[other_index].get(path) else {
                    continue;
                };
                if their_size == size && *their_size >= 0 {
                    continue;
                }
                conflicts.push(ModConflict {
                    other_mod_id: other.id.clone(),
                    kind: ModConflictKind::OverlappingDataFile,
                    detail: path.clone(),
                });
            }
        }
        conflicts_by_mod.push(conflicts);
        missing_by_mod.push(
            installed
                .manifest
                .as_ref()
                .map(|manifest| manifest.requires.as_slice())
                .unwrap_or_default()
                .iter()
                .filter(|id| !installed_ids.contains(id.as_str()))
                .cloned()
                .collect(),
        );
    }

    mods.into_iter()
        .zip(conflicts_by_mod.into_iter().zip(missing_by_mod))
        .map(|(installed, (conflicts, requires_missing))| InstalledMod {
            conflicts,
            requires_missing,
            ..installed
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_kinds_from_file_names() {
        assert_eq!(
            bundle_kind_from_file_name("Effects.toml").as_deref(),
            Some("effect_area")
        );
        assert_eq!(
            bundle_kind_from_file_name("weapon_patch_01.toml").as_deref(),
            Some("weapon_patch")
        );
        assert_eq!(
            bundle_kind_from_file_name("weapon_sword.json").as_deref(),
            Some("weapon")
        );
        assert_eq!(
            bundle_kind_from_file_name("shop12.toml").as_deref(),
            Some("shop")
        );
        assert_eq!(bundle_kind_from_file_name("shopx.toml"), None);
        assert_eq!(bundle_kind_from_file_name("readme.txt"), None);
        assert_eq!(bundle_kind_from_file_name(".toml"), None);
    }

    #[test]
    fn json_kind_is_peeked_when_the_name_is_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let entities = dir.path().join("entities");
        std::fs::create_dir_all(entities.join("sub")).unwrap();
        std::fs::write(entities.join("a.json"), r#"{"kind":"Quest_Bundle"}"#).unwrap();
        std::fs::write(entities.join("sub").join("b.json"), r#"[{"kind":"mail"}]"#).unwrap();
        std::fs::write(entities.join("c.toml"), "kind = \"x\"").unwrap();
        std::fs::write(entities.join("d.txt"), "").unwrap();
        let summary = scan_native(&entities);
        assert_eq!(summary.total_entity_files, 3);
        assert_eq!(summary.bundles_by_kind.get("quest"), Some(&1));
        assert_eq!(summary.bundles_by_kind.get("mail"), Some(&1));
        assert_eq!(summary.bundles_by_kind.get("unknown"), Some(&1));
    }
}
