use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use yp_core::{CancelToken, ProgressSink};

use super::{TextureCopyProgress, TextureSource, install_textures};
use crate::detect::detect_texture_packs;
use crate::fsx;
use crate::model::{ModKind, ModVariant};
use crate::staging::{DropStaging, source_base_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeTarget {
    KeepNames,
    Cancel,
    Into(String),
}

pub trait TexturePackPrompts {
    fn choose_packs(&mut self, packs: &[ModVariant]) -> Option<Vec<ModVariant>>;

    fn pack_name(&mut self, fallback: &str) -> Option<String> {
        let _ = fallback;
        None
    }

    fn merge_target(&mut self) -> MergeTarget {
        MergeTarget::KeepNames
    }

    fn extract_failed(&mut self, archive: &Path, error: &yp_core::Error) {
        let _ = (archive, error);
    }

    fn copy_started(&mut self) {}

    fn copy_progress(&mut self, progress: &TextureCopyProgress) {
        let _ = progress;
    }
}

pub fn run(
    staging: &DropStaging,
    paths: &[PathBuf],
    texture_dir: &Path,
    prompts: &mut dyn TexturePackPrompts,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> yp_core::Result<Option<usize>> {
    let mut extracted_dirs = Vec::new();
    let mut effective: Vec<PathBuf> = paths
        .iter()
        .filter(|path| !DropStaging::is_archive(path))
        .cloned()
        .collect();
    let mut names: IndexMap<PathBuf, String> = IndexMap::new();

    for archive in paths.iter().filter(|path| DropStaging::is_archive(path)) {
        let extracted = match staging.extract(archive, None, progress, cancel) {
            Ok(dir) => dir,
            Err(err) if err.is_cancelled() => return Err(err),
            Err(error) => {
                prompts.extract_failed(archive, &error);
                continue;
            }
        };
        let root = extracted.path().to_path_buf();
        extracted_dirs.push(extracted);

        let packs = detect_texture_packs(&root);
        if cancel.is_cancelled() {
            return Ok(None);
        }
        let fallback = source_base_name(archive);

        if packs.len() <= 1 {
            let mut name = packs
                .first()
                .map_or_else(|| fallback.clone(), |pack| pack.label.clone());
            if name == fsx::file_name(&root) {
                name = prompts
                    .pack_name(&fallback)
                    .unwrap_or_else(|| fallback.clone());
            }
            if cancel.is_cancelled() {
                return Ok(None);
            }
            effective.push(root.clone());
            names.insert(root, name);
            continue;
        }

        let variants: Vec<ModVariant> = packs
            .iter()
            .map(|pack| ModVariant {
                sub_path: pack.path.clone(),
                label: pack.label.clone(),
                kind: ModKind::Texture,
                texture_only: true,
                category: None,
            })
            .collect();
        let Some(chosen) = prompts
            .choose_packs(&variants)
            .filter(|chosen| !chosen.is_empty())
        else {
            return Ok(None);
        };
        if cancel.is_cancelled() {
            return Ok(None);
        }
        let pack_name = if chosen.len() == 1 {
            chosen[0].label.clone()
        } else {
            fallback
        };
        for variant in chosen {
            names.insert(variant.sub_path.clone(), pack_name.clone());
            effective.push(variant.sub_path);
        }
    }

    if effective.is_empty() {
        return Ok(None);
    }

    if effective.len() == 1 {
        match prompts.merge_target() {
            MergeTarget::Cancel => return Ok(None),
            MergeTarget::Into(target) if !target.is_empty() => {
                for path in &effective {
                    names.insert(path.clone(), target.clone());
                }
            }
            MergeTarget::Into(_) | MergeTarget::KeepNames => {}
        }
    }

    prompts.copy_started();
    let sources: Vec<TextureSource> = effective
        .iter()
        .map(|path| TextureSource {
            path: path.clone(),
            folder: names.get(path).cloned(),
        })
        .collect();
    let count = install_textures(
        staging,
        &sources,
        texture_dir,
        &mut |copy_progress| prompts.copy_progress(&copy_progress),
        cancel,
    )?;
    drop(extracted_dirs);
    Ok(Some(count))
}
