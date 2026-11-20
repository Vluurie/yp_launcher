use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::fsx;
use crate::loose_paths::data_dir_for_loose_file;
use crate::model::{DataCategory, ModKind};
use crate::tables::{data_dir_category, is_data_dir_key};

pub(crate) const COMPAT_CONFIG_SUBDIRS: [&str; 9] = [
    "item_info",
    "outfits",
    "accessories",
    "hairsprays",
    "behaviors",
    "mcd",
    "shops",
    "inventory",
    "weapons",
];

const OUTFIT_CONFIG_SUBDIRS: [&str; 1] = ["outfits"];

const TEXTURE_WRAPPER_DIRS: [&str; 5] = [
    "sk_res",
    "far_res",
    "inject",
    "textures",
    "nierautomata.exe",
];

pub(crate) const MOD_PAYLOAD_DIRS: [&str; 4] = ["data", "entities", "wax", "cutscenes"];

const UNSUPPORTED_CPK_NAMES: [&str; 1] = ["sadfutago.cpk"];

pub(crate) fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.') || name.starts_with('_')
}

pub(crate) fn is_texture_wrapper(name: &str) -> bool {
    TEXTURE_WRAPPER_DIRS.contains(&name.to_lowercase().as_str())
}

pub(crate) fn is_mod_payload_dir(lower_name: &str) -> bool {
    MOD_PAYLOAD_DIRS.contains(&lower_name)
}

pub(crate) fn is_cpk_file(file_name: &str) -> bool {
    file_name.to_lowercase().ends_with(".cpk")
}

pub(crate) fn unwrap_single_child(root: &Path) -> PathBuf {
    let mut current = root.to_path_buf();
    for _ in 0..6 {
        if !current.is_dir() {
            return current;
        }
        let entries = fsx::children(&current);
        let dirs: Vec<&fsx::Child> = entries.iter().filter(|child| child.is_dir).collect();
        if dirs.len() != 1 || entries.len() != 1 {
            return current;
        }
        if current.join("entities").is_dir()
            || current.join("wax").is_dir()
            || current.join("data").is_dir()
            || current.join("mod.toml").is_file()
        {
            return current;
        }
        if is_data_dir_key(&dirs[0].name.to_lowercase()) {
            return current;
        }
        current = dirs[0].path.clone();
    }
    current
}

pub(crate) fn classify_kind(content_root: &Path) -> ModKind {
    let has_entities = entities_has_content(&content_root.join("entities"));
    let has_compat = has_compat_config(content_root);
    let has_data = data_has_subdirs(&content_root.join("data"));
    let has_loose_dir = has_loose_data_dir(content_root);
    let has_loose_files = has_loose_data_files(content_root);
    let has_cpk_files = has_cpk(content_root);

    if has_entities && has_compat {
        return ModKind::Unknown;
    }
    if has_entities {
        return ModKind::Native;
    }
    if has_data || has_compat || has_loose_dir || has_loose_files || has_cpk_files {
        return ModKind::Data;
    }
    ModKind::Unknown
}

pub(crate) fn has_nams_payload(root: &Path) -> bool {
    entities_has_content(&root.join("entities"))
        || root.join("wax").is_dir()
        || data_has_subdirs(&root.join("data"))
        || has_compat_config(root)
        || has_loose_data_dir(root)
        || has_loose_data_files(root)
        || has_cpk(root)
}

pub(crate) fn dir_has_dds(dir: &Path) -> bool {
    dir.is_dir() && fsx::any_walked_file(dir, |path| fsx::lower_path_ends_with(path, ".dds"))
}

pub(crate) fn dir_has_direct_dds(dir: &Path) -> bool {
    dir.is_dir()
        && fsx::child_files(dir)
            .iter()
            .any(|file| fsx::lower_path_ends_with(&file.path, ".dds"))
}

pub(crate) fn is_texture_pack_root(root: &Path) -> bool {
    if has_nams_payload(root) {
        return false;
    }
    for res_name in ["SK_Res", "FAR_Res"] {
        if root.join(res_name).join("inject").join("textures").is_dir() {
            return true;
        }
    }
    if root.join("inject").join("textures").is_dir() {
        return true;
    }
    let textures = root.join("textures");
    if textures.is_dir() && dir_has_dds(&textures) {
        return true;
    }
    if dir_has_direct_dds(root) {
        return true;
    }
    dir_has_dds(root)
}

pub(crate) fn has_loose_data_dir(content_root: &Path) -> bool {
    fsx::child_dirs(content_root)
        .iter()
        .any(|dir| is_data_dir_key(&dir.name.to_lowercase()))
}

pub(crate) fn has_loose_data_files(content_root: &Path) -> bool {
    fsx::child_files(content_root)
        .iter()
        .any(|file| data_dir_for_loose_file(&file.name).is_some())
}

pub(crate) fn has_cpk(content_root: &Path) -> bool {
    [content_root.to_path_buf(), content_root.join("data")]
        .iter()
        .filter(|base| base.is_dir())
        .any(|base| {
            fsx::child_files(base)
                .iter()
                .any(|file| is_cpk_file(&file.name))
        })
}

pub(crate) fn entities_has_content(dir: &Path) -> bool {
    dir.is_dir()
        && fsx::any_walked_file(dir, |path| {
            fsx::lower_path_ends_with(path, ".toml") || fsx::lower_path_ends_with(path, ".json")
        })
}

pub(crate) fn has_compat_config(content_root: &Path) -> bool {
    has_config_in(content_root, &COMPAT_CONFIG_SUBDIRS)
}

pub(crate) fn has_outfit_config(content_root: &Path) -> bool {
    has_config_in(content_root, &OUTFIT_CONFIG_SUBDIRS)
}

fn has_config_in(content_root: &Path, subdirs: &[&str]) -> bool {
    let wrapped_root = content_root.join("wax").join("mods");
    if wrapped_root.is_dir()
        && fsx::child_dirs(&wrapped_root)
            .iter()
            .any(|inner| has_config_subdir_at(&inner.path, subdirs))
    {
        return true;
    }
    if has_config_subdir_at(content_root, subdirs) {
        return true;
    }
    misplaced_config_wrapper_dirs(content_root)
        .iter()
        .any(|wrapper| has_config_subdir_at(wrapper, subdirs))
}

pub(crate) fn misplaced_config_wrapper_dirs(content_root: &Path) -> Vec<PathBuf> {
    fsx::child_dirs(content_root)
        .into_iter()
        .filter(|sub| {
            let lower = sub.name.to_lowercase();
            !is_hidden_name(&sub.name)
                && !is_mod_payload_dir(&lower)
                && !is_texture_wrapper(&lower)
                && !is_data_dir_key(&lower)
                && !COMPAT_CONFIG_SUBDIRS.contains(&lower.as_str())
                && is_misplaced_config_wrapper(&sub.path)
        })
        .map(|sub| sub.path)
        .collect()
}

fn is_misplaced_config_wrapper(root: &Path) -> bool {
    has_config_subdir_at(root, &COMPAT_CONFIG_SUBDIRS)
        && !root.join("wax").is_dir()
        && !root.join("entities").is_dir()
        && !data_has_subdirs(&root.join("data"))
        && !has_loose_data_dir(root)
        && !has_loose_data_files(root)
        && !has_cpk(root)
}

fn has_config_subdir_at(root: &Path, subdirs: &[&str]) -> bool {
    subdirs.iter().any(|sub| {
        let dir = root.join(sub);
        dir.is_dir()
            && fsx::child_files(&dir)
                .iter()
                .any(|file| fsx::lower_path_ends_with(&file.path, ".json"))
    })
}

pub(crate) fn data_has_subdirs(dir: &Path) -> bool {
    dir.is_dir()
        && fsx::child_dirs(dir)
            .iter()
            .any(|entry| is_data_dir_key(&entry.name.to_lowercase()))
}

pub(crate) fn has_unsupported_cpk(root: &Path) -> bool {
    root.is_dir()
        && fsx::any_walked_file(root, |path| {
            UNSUPPORTED_CPK_NAMES.contains(&fsx::file_name(path).to_lowercase().as_str())
        })
}

pub(crate) fn primary_data_category_at(content_root: &Path) -> Option<DataCategory> {
    let mut counts: IndexMap<DataCategory, usize> = IndexMap::new();
    let mut bump = |dir_key: &str, amount: usize| {
        if let Some(category) = data_dir_category(&dir_key.to_lowercase()) {
            *counts.entry(category).or_insert(0) += amount;
        }
    };
    for base in [content_root.to_path_buf(), content_root.join("data")] {
        if !base.is_dir() {
            continue;
        }
        for entity in fsx::child_dirs(&base) {
            let files = fsx::walk_files(&entity.path).len();
            bump(&entity.name, if files > 0 { files } else { 1 });
        }
        for file in fsx::child_files(&base) {
            if let Some(key) = data_dir_for_loose_file(&file.name) {
                bump(&key, 1);
            }
        }
    }
    let mut best: Option<(DataCategory, usize)> = None;
    for (category, count) in counts {
        if best.is_none_or(|(_, best_count)| count > best_count) {
            best = Some((category, count));
        }
    }
    best.map(|(category, _)| category)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"x").unwrap();
    }

    #[test]
    fn unwraps_single_wrappers_but_stops_at_data_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(
            &root
                .join("Wrapper")
                .join("Inner")
                .join("pl")
                .join("pl0000.dat"),
        );
        assert_eq!(
            unwrap_single_child(root),
            root.join("Wrapper").join("Inner")
        );
    }

    #[test]
    fn classifies_mixed_entities_and_compat_as_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(&root.join("entities").join("a.toml"));
        assert_eq!(classify_kind(root), ModKind::Native);
        touch(&root.join("outfits").join("o.json"));
        assert_eq!(classify_kind(root), ModKind::Unknown);
    }

    #[test]
    fn primary_category_prefers_largest_count_and_first_on_ties() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(&root.join("data").join("wp").join("a.dat"));
        touch(&root.join("data").join("wp").join("b.dat"));
        touch(&root.join("data").join("pl").join("a.dat"));
        assert_eq!(primary_data_category_at(root), Some(DataCategory::Weapon));
        touch(&root.join("data").join("pl").join("b.dat"));
        assert_eq!(primary_data_category_at(root), Some(DataCategory::Player));
    }
}
