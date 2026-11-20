use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};

use super::is_texture;
use crate::model::InstalledMod;
use crate::probe::dir_has_dds;
use crate::staging::DropStaging;

const TEXTURE_SUFFIX: &str = ".dds";

pub fn dir_contains_dds(dir: &Path) -> bool {
    dir_has_dds(dir)
}

fn archive_contains_dds(staging: &DropStaging, archive: &Path) -> bool {
    staging.list_entries(archive).is_some_and(|entries| {
        entries
            .iter()
            .any(|entry| entry.to_lowercase().ends_with(TEXTURE_SUFFIX))
    })
}

pub fn drop_contains_dds(staging: &DropStaging, paths: &[PathBuf]) -> bool {
    paths.iter().any(|path| {
        if DropStaging::is_archive(path) {
            archive_contains_dds(staging, path)
        } else if path.is_file() {
            is_texture(path)
        } else {
            dir_contains_dds(path)
        }
    })
}

pub fn reconciled_load_order(current: &[String], folders: &[String]) -> Option<Vec<String>> {
    if folders.is_empty() {
        return None;
    }
    let deduped: IndexSet<&String> = current.iter().collect();
    let missing: Vec<&String> = folders
        .iter()
        .filter(|folder| !deduped.contains(folder))
        .collect();
    if missing.is_empty() && deduped.len() == current.len() {
        return None;
    }
    Some(deduped.into_iter().chain(missing).cloned().collect())
}

pub fn bundled_origin_by_pack(mods: &[InstalledMod]) -> IndexMap<String, String> {
    let mut origins = IndexMap::new();
    for installed in mods {
        for pack in &installed.bundled_texture_packs {
            origins.insert(pack.clone(), installed.id.clone());
        }
    }
    origins
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModKind;

    fn names(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn load_order_gains_missing_folders_and_loses_duplicates() {
        assert_eq!(
            reconciled_load_order(&names(&["b", "a"]), &names(&["a", "c", "b", "d"])),
            Some(names(&["b", "a", "c", "d"]))
        );
        assert_eq!(
            reconciled_load_order(&names(&["a", "b", "a"]), &names(&["a", "b"])),
            Some(names(&["a", "b"]))
        );
        assert_eq!(
            reconciled_load_order(&names(&["gone", "a"]), &names(&["a"])),
            None
        );
        assert_eq!(reconciled_load_order(&names(&["a", "a"]), &[]), None);
        assert_eq!(
            reconciled_load_order(&[], &names(&["x"])),
            Some(names(&["x"]))
        );
    }

    #[test]
    fn bundled_packs_point_at_the_last_mod_that_lists_them() {
        let mut first = InstalledMod::new("first", "mods/first", ModKind::Data);
        first.bundled_texture_packs = names(&["Shared", "Only First"]);
        let mut second = InstalledMod::new("second", "mods/second", ModKind::Data);
        second.bundled_texture_packs = names(&["Shared"]);
        let origins = bundled_origin_by_pack(&[first, second]);
        assert_eq!(
            origins.into_iter().collect::<Vec<_>>(),
            vec![
                ("Shared".to_owned(), "second".to_owned()),
                ("Only First".to_owned(), "first".to_owned()),
            ]
        );
    }
}
