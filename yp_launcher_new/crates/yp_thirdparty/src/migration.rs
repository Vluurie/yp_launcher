use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use yp_core::consts::{
    GAME_MODS_CONFIG_NAME, GAME_MODS_DIR_NAME, GRAPHIC_MODS_DIR_NAME, MIGOTO_DIR_NAME, RESHADE_DIR_NAME,
    THIRD_PARTY_DISABLED_DIR_NAME,
};
use yp_core::{AppPaths, GamePaths};

use crate::paths::{self, DIRECTX_PROXY_NAMES};
use crate::tree::{self, Entry};
use crate::{migoto, reshade};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MigrationResult {
    pub from: PathBuf,
    pub to: PathBuf,
    pub moved_files: usize,
    pub left_behind: Vec<PathBuf>,
}

impl MigrationResult {
    pub fn is_noop(&self) -> bool {
        self.moved_files == 0 && self.left_behind.is_empty()
    }
}

pub fn migrate_legacy(game: &GamePaths, app: &AppPaths, game_running: bool) -> MigrationResult {
    migrate_from(game, &app.launcher_thirdparty_dir(), game_running)
}

pub fn migrate_from(game: &GamePaths, legacy: &Path, game_running: bool) -> MigrationResult {
    if game.root().as_os_str().is_empty() || !legacy.is_dir() || game_running {
        return MigrationResult::default();
    }

    let before = tree::file_count(legacy);
    let mut left = Vec::new();

    let old_reshade = legacy.join(RESHADE_DIR_NAME);
    if old_reshade.is_dir() {
        left.extend(tree::move_merging(&old_reshade, &reshade::install_dir(game)));
    }

    let old_migoto = legacy.join(MIGOTO_DIR_NAME);
    if old_migoto.is_dir() {
        left.extend(tree::move_merging(&old_migoto, &migoto::install_dir(game)));
    }

    let old_game = legacy.join(GAME_MODS_DIR_NAME);
    if old_game.is_dir() {
        left.extend(move_game_mods(&old_game, &paths::game_mods(game)));
    }

    let old_graphic = legacy.join(GRAPHIC_MODS_DIR_NAME);
    if old_graphic.is_dir() {
        left.extend(move_graphic_mods(
            &old_graphic,
            &paths::graphic_mods(game),
            &paths::graphic_mods_disabled(game),
        ));
    }

    tree::remove_dir_if_empty(legacy);
    MigrationResult {
        from: legacy.to_path_buf(),
        to: paths::root(game),
        moved_files: before.saturating_sub(tree::file_count(legacy)),
        left_behind: left,
    }
}

fn is_dll(entry: &Entry) -> bool {
    entry.is_file() && entry.name().to_lowercase().ends_with(".dll")
}

fn move_game_mods(old: &Path, kind_dir: &Path) -> Vec<PathBuf> {
    let mut left = Vec::new();
    let mut rest = Vec::new();
    for entry in tree::list_dir(old) {
        if entry.is_file() && entry.name().to_lowercase() == GAME_MODS_CONFIG_NAME {
            left.extend(tree::move_merging(&entry.path, &kind_dir.join(GAME_MODS_CONFIG_NAME)));
        } else {
            rest.push(entry);
        }
    }

    let dlls: Vec<PathBuf> = rest.iter().filter(|entry| is_dll(entry)).map(|entry| entry.path.clone()).collect();
    let mut groups: IndexMap<PathBuf, Vec<PathBuf>> = dlls.iter().map(|dll| (dll.clone(), vec![dll.clone()])).collect();
    let mut unowned = Vec::new();
    for entry in rest.iter().filter(|entry| !is_dll(entry)) {
        match owner_dll(&entry.name(), &dlls) {
            Some(owner) => groups.entry(owner).or_default().push(entry.path.clone()),
            None => unowned.push(entry.path.clone()),
        }
    }
    if groups.len() == 1 {
        if let Some(files) = groups.values_mut().next() {
            files.extend(unowned);
        }
    } else {
        left.extend(unowned);
    }

    for (dll, files) in groups {
        let dll_name = tree::file_name(&dll);
        let stem = tree::file_stem(&dll);
        let dest = paths::mod_dir_with_dll(kind_dir, &[dll_name])
            .unwrap_or_else(|| kind_dir.join(paths::unique_dir_name(kind_dir, &paths::mod_dir_name(&stem, &stem))));
        for file in files {
            left.extend(tree::move_merging(&file, &dest.join(tree::file_name(&file))));
        }
    }

    tree::remove_dir_if_empty(old);
    left
}

fn move_graphic_mods(old: &Path, active_root: &Path, disabled_root: &Path) -> Vec<PathBuf> {
    let mut left = Vec::new();
    let entries = tree::list_dir(old);
    let parked_dir = entries
        .iter()
        .find(|entry| entry.is_dir() && entry.name().to_lowercase() == THIRD_PARTY_DISABLED_DIR_NAME)
        .map(|entry| entry.path.clone());
    let active: Vec<Entry> = entries
        .into_iter()
        .filter(|entry| Some(&entry.path) != parked_dir.as_ref())
        .collect();
    let parked = parked_dir.as_deref().map(tree::list_dir).unwrap_or_default();

    if !active.is_empty() || !parked.is_empty() {
        let active_dlls: Vec<String> = active.iter().filter(|entry| is_dll(entry)).map(Entry::name).collect();
        let mut all_dlls: Vec<String> = active_dlls
            .iter()
            .cloned()
            .chain(parked.iter().filter(|entry| is_dll(entry)).map(Entry::name))
            .collect();
        all_dlls.sort_by_cached_key(|name| (name.to_lowercase(), name.clone()));
        let mut proxy_stems: Vec<String> = all_dlls
            .iter()
            .filter(|name| DIRECTX_PROXY_NAMES.contains(&name.to_lowercase().as_str()))
            .map(|name| tree::file_stem(Path::new(name)).to_lowercase())
            .collect();
        proxy_stems.sort();
        proxy_stems.dedup();
        let raw_name = if !proxy_stems.is_empty() {
            proxy_stems.join("+")
        } else if let Some(first) = all_dlls.first() {
            tree::file_stem(Path::new(first))
        } else {
            GRAPHIC_MODS_DIR_NAME.to_owned()
        };
        let name = paths::mod_dir_name(&raw_name, GRAPHIC_MODS_DIR_NAME);
        let active_dest = paths::mod_dir_with_dll(active_root, &active_dlls)
            .unwrap_or_else(|| active_root.join(paths::unique_dir_name(active_root, &name)));
        let parked_dest = disabled_root.join(tree::file_name(&active_dest));

        for entry in &active {
            left.extend(tree::move_merging(&entry.path, &active_dest.join(entry.name())));
        }
        for entry in &parked {
            left.extend(tree::move_merging(&entry.path, &parked_dest.join(entry.name())));
        }
    }

    if let Some(parked_dir) = parked_dir {
        tree::remove_dir_if_empty(&parked_dir);
    }
    tree::remove_dir_if_empty(old);
    left
}

pub fn owner_dll(file_name: &str, dlls: &[PathBuf]) -> Option<PathBuf> {
    let name = file_name.to_lowercase();
    let mut owner = None;
    let mut best = None;
    for dll in dlls {
        let stem = tree::file_stem(dll).to_lowercase();
        if best.is_some_and(|best| stem.len() <= best) || !name.starts_with(&stem) {
            continue;
        }
        if name.as_bytes().get(stem.len()).is_some_and(|next| next.is_ascii_lowercase() || next.is_ascii_digit()) {
            continue;
        }
        owner = Some(dll.clone());
        best = Some(stem.len());
    }
    owner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_is_the_longest_stem_followed_by_a_separator() {
        let dlls = vec![PathBuf::from("celestial.dll"), PathBuf::from("celestial_extra.dll")];
        assert_eq!(owner_dll("celestial.ini", &dlls), Some(PathBuf::from("celestial.dll")));
        assert_eq!(owner_dll("celestial_log.txt", &dlls), Some(PathBuf::from("celestial.dll")));
        assert_eq!(owner_dll("celestial_extra.ini", &dlls), Some(PathBuf::from("celestial_extra.dll")));
        assert_eq!(owner_dll("celestialx.ini", &dlls), None);
        assert_eq!(owner_dll("notes.txt", &dlls), None);
        assert_eq!(owner_dll("CELESTIAL.INI", &dlls), Some(PathBuf::from("celestial.dll")));
    }
}
