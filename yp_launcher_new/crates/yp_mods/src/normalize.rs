use std::io;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};

use crate::detect::detect_packs;
use crate::fsx;
use crate::loose_paths::{data_dir_for_loose_file, has_vanilla_pair};
use crate::model::TexturePack;
use crate::probe::{is_cpk_file, misplaced_config_wrapper_dirs};
use crate::tables::is_data_dir_key;
use crate::text::utf16_cmp;

const TEXTURE_WRAPPERS_TO_DROP: [&str; 3] = ["SK_Res", "FAR_Res", "inject"];

pub(crate) fn loose_data_dirs(root: &Path) -> io::Result<()> {
    if !root.is_dir() || root.join("entities").is_dir() || root.join("wax").is_dir() {
        return Ok(());
    }
    let loose: Vec<fsx::Child> = fsx::child_dirs(root)
        .into_iter()
        .filter(|entry| is_data_dir_key(&entry.name.to_lowercase()))
        .collect();
    if loose.is_empty() {
        return Ok(());
    }
    let data_dir = root.join("data");
    std::fs::create_dir_all(&data_dir)?;
    for source in loose {
        std::fs::rename(&source.path, data_dir.join(&source.name))?;
    }
    Ok(())
}

pub(crate) fn loose_data_files(root: &Path) -> io::Result<Vec<String>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut moves: Vec<(fsx::Child, String)> = Vec::new();
    let mut exts_by_stem: IndexMap<String, IndexSet<String>> = IndexMap::new();
    for file in fsx::child_files(root) {
        let Some(target) = data_dir_for_loose_file(&file.name) else {
            continue;
        };
        let lower = file.name.to_lowercase();
        let split = lower.len() - 4;
        exts_by_stem
            .entry(lower[..split].to_owned())
            .or_default()
            .insert(lower[split + 1..].to_owned());
        moves.push((file, target));
    }
    if moves.is_empty() {
        return Ok(Vec::new());
    }
    for (file, sub) in &moves {
        let mut dest_dir = root.join("data");
        if !sub.is_empty() {
            for part in sub.split('/') {
                dest_dir.push(part);
            }
        }
        std::fs::create_dir_all(&dest_dir)?;
        std::fs::rename(&file.path, dest_dir.join(&file.name))?;
    }
    let mut warnings: Vec<String> = Vec::new();
    for (stem, exts) in &exts_by_stem {
        if !has_vanilla_pair(stem) {
            continue;
        }
        if !exts.contains("dat") {
            warnings.push(format!("{stem}.dat"));
        }
        if !exts.contains("dtt") {
            warnings.push(format!("{stem}.dtt"));
        }
    }
    warnings.sort_by(|a, b| utf16_cmp(a, b));
    Ok(warnings)
}

pub(crate) fn cpks(root: &Path) {
    if !root.is_dir() {
        return;
    }
    let loose: Vec<fsx::Child> = fsx::child_files(root)
        .into_iter()
        .filter(|file| is_cpk_file(&file.name))
        .collect();
    if loose.is_empty() {
        return;
    }
    let data_dir = root.join("data");
    if std::fs::create_dir_all(&data_dir).is_err() {
        return;
    }
    for file in loose {
        let dest = data_dir.join(&file.name);
        if dest.is_file() {
            continue;
        }
        if std::fs::rename(&file.path, &dest).is_err() && std::fs::copy(&file.path, &dest).is_ok() {
            let _ = std::fs::remove_file(&file.path);
        }
    }
}

pub(crate) fn misplaced_wax_config_dirs(root: &Path) {
    let wrappers = misplaced_config_wrapper_dirs(root);
    if wrappers.is_empty() {
        return;
    }
    let wax_mods = root.join("wax").join("mods");
    for wrapper in wrappers {
        if std::fs::create_dir_all(&wax_mods).is_err() {
            continue;
        }
        let dest_name = fsx::unique_child_name(&wax_mods, &fsx::file_name(&wrapper));
        let _ = fsx::move_dir(&wrapper, &wax_mods.join(dest_name));
    }
}

pub(crate) fn stage_sibling_texture_sets(
    drop_root: &Path,
    variant_root: &Path,
    target_dir: &Path,
    chosen: &[PathBuf],
) -> io::Result<Vec<PathBuf>> {
    let mut local = Vec::new();
    for sub in chosen {
        let absolute = fsx::normalize(&drop_root.join(sub));
        if fsx::same_path(&absolute, variant_root) || fsx::is_within(variant_root, &absolute) {
            if let Some(relative) = fsx::relative(&absolute, variant_root) {
                local.push(relative);
            }
            continue;
        }
        if !absolute.is_dir() {
            continue;
        }
        let dest_name = fsx::unique_child_name(target_dir, &fsx::file_name(&absolute));
        fsx::copy_dir(&absolute, &target_dir.join(&dest_name))?;
        local.push(PathBuf::from(dest_name));
    }
    Ok(local)
}

pub(crate) fn in_place_textures(mod_root: &Path, chosen: &[PathBuf]) {
    let packs = detect_packs(mod_root, true);
    if packs.is_empty() {
        return;
    }
    let textures_dir = mod_root.join("textures");

    let selected: Vec<&TexturePack> = if chosen.is_empty() {
        packs.iter().collect()
    } else {
        let targets: Vec<PathBuf> = chosen
            .iter()
            .map(|sub| fsx::normalize(&mod_root.join(sub)))
            .collect();
        let picked: Vec<&TexturePack> = packs
            .iter()
            .filter(|pack| {
                targets
                    .iter()
                    .any(|target| fsx::same_path(&pack.path, target))
            })
            .collect();
        if picked.is_empty() {
            packs.iter().collect()
        } else {
            picked
        }
    };

    for pack in selected {
        if !pack.path.is_dir() || fsx::same_path(&pack.path, &textures_dir) {
            continue;
        }
        for file in fsx::walk_files(&pack.path) {
            if !fsx::lower_path_ends_with(&file, ".dds") {
                continue;
            }
            let dest = textures_dir.join(fsx::file_name(&file));
            if dest.is_file() {
                continue;
            }
            if std::fs::create_dir_all(&textures_dir).is_ok()
                && std::fs::rename(&file, &dest).is_err()
            {
                let _ = std::fs::copy(&file, &dest);
            }
        }
    }

    for name in TEXTURE_WRAPPERS_TO_DROP {
        fsx::remove_dir_quiet(&mod_root.join(name));
    }
    for pack in &packs {
        if !pack.path.is_dir() || fsx::same_path(&pack.path, &textures_dir) {
            continue;
        }
        if pack
            .path
            .parent()
            .is_some_and(|parent| fsx::same_path(parent, mod_root))
        {
            fsx::remove_dir_quiet(&pack.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"x").unwrap();
    }

    #[test]
    fn moves_loose_dirs_and_files_under_data() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(&root.join("wp").join("wp0100.dat"));
        touch(&root.join("corehap.dat"));
        loose_data_dirs(root).unwrap();
        assert!(root.join("data").join("wp").join("wp0100.dat").is_file());
        let warnings = loose_data_files(root).unwrap();
        assert!(root.join("data").join("core").join("corehap.dat").is_file());
        assert!(warnings.is_empty() || warnings.iter().all(|w| w.starts_with("corehap")));
    }

    #[test]
    fn loose_dirs_are_left_alone_next_to_entities() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(&root.join("wp").join("wp0100.dat"));
        touch(&root.join("entities").join("a.toml"));
        loose_data_dirs(root).unwrap();
        assert!(root.join("wp").is_dir());
    }

    #[test]
    fn cpks_move_into_data_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        touch(&root.join("data100.cpk"));
        touch(&root.join("data201.CPK"));
        std::fs::create_dir_all(root.join("data")).unwrap();
        std::fs::write(root.join("data").join("data201.CPK"), b"old").unwrap();
        cpks(root);
        assert!(root.join("data").join("data100.cpk").is_file());
        assert!(root.join("data201.CPK").is_file());
        assert_eq!(
            std::fs::read(root.join("data").join("data201.CPK")).unwrap(),
            b"old"
        );
    }
}
