use std::path::{Path, PathBuf};

use indexmap::IndexSet;
use yp_core::{CancelToken, GamePaths, IoOp, ProgressSink};

use crate::detect::{detect_dir, sanitize_id};
use crate::fsx;
use crate::model::{
    DataCategory, DropRejection, InstallFailure, InstallResult, Installed, ModKind,
    VariantInstallRequest,
};
use crate::normalize;
use crate::probe::{has_cpk, has_nams_payload, is_texture_pack_root};
use crate::staging::{DropStaging, StagedDrop, source_base_name};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstallRequest {
    pub requested_name: Option<String>,
    pub variant_sub_path: Option<PathBuf>,
    pub texture_pack_sub_paths: Vec<PathBuf>,
}

impl InstallRequest {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            requested_name: Some(name.into()),
            ..Self::default()
        }
    }
}

fn stage_archive(
    staging: &DropStaging,
    game: &GamePaths,
    source: &Path,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> Result<StagedDrop, InstallFailure> {
    staging
        .extract(source, Some(game), progress, cancel)
        .map(StagedDrop::Temp)
        .map_err(InstallFailure::extraction)
}

pub fn install(
    staging: &DropStaging,
    game: &GamePaths,
    source: &Path,
    request: &InstallRequest,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> InstallResult {
    let staged = if DropStaging::is_archive(source) {
        stage_archive(staging, game, source, progress, cancel)?
    } else if source.is_file() {
        match staging.stage_loose_file(source, Some(game)) {
            Some(dir) => StagedDrop::Temp(dir),
            None => return Err(DropRejection::UnknownDrop.into()),
        }
    } else {
        StagedDrop::InPlace(source.to_path_buf())
    };
    if cancel.is_cancelled() {
        return Err(InstallFailure::Cancelled);
    }
    install_from_dir(game, staged.root(), &source_base_name(source), request)
}

fn io_failure(op: IoOp, path: &Path) -> impl FnOnce(std::io::Error) -> InstallFailure + '_ {
    move |err| InstallFailure::io(op, path, &err)
}

pub(crate) fn install_from_dir(
    game: &GamePaths,
    work_dir: &Path,
    source_folder_name: &str,
    request: &InstallRequest,
) -> InstallResult {
    let detected = detect_dir(work_dir, source_folder_name);
    let drop_root = detected.unwrapped_root.clone();
    let mut work_root = drop_root.clone();
    match request
        .variant_sub_path
        .as_deref()
        .filter(|sub| !sub.as_os_str().is_empty())
    {
        Some(sub) => {
            let resolved = fsx::normalize(&work_root.join(sub));
            if !resolved.is_dir() {
                return Err(InstallFailure::VariantMissing);
            }
            work_root = resolved;
        }
        None if detected.kind == ModKind::Unknown => {
            return Err(detected
                .rejection
                .unwrap_or(DropRejection::UnknownDrop)
                .into());
        }
        None => {}
    }

    normalize::loose_data_dirs(&work_root).map_err(io_failure(IoOp::Rename, &work_root))?;
    let unpaired_warnings =
        normalize::loose_data_files(&work_root).map_err(io_failure(IoOp::Rename, &work_root))?;
    normalize::cpks(&work_root);
    normalize::misplaced_wax_config_dirs(&work_root);

    let mods_root = game.mods_dir();
    let name_source = request
        .requested_name
        .as_deref()
        .filter(|name| !name.is_empty())
        .or_else(|| {
            detected
                .manifest
                .as_ref()
                .and_then(|manifest| manifest.id.as_deref())
        })
        .unwrap_or(source_folder_name);
    let target_id = sanitize_id(name_source);
    let target_dir = mods_root.join(&target_id);
    if target_dir.is_dir() {
        return Err(InstallFailure::Exists { id: target_id });
    }

    if !has_nams_payload(&work_root) && is_texture_pack_root(&work_root) {
        return Err(DropRejection::TextureOnly.into());
    }

    let rechecked = detect_dir(&work_root, source_folder_name);
    match rechecked.kind {
        ModKind::Unknown => {
            return Err(rechecked
                .rejection
                .unwrap_or(DropRejection::UnknownDrop)
                .into());
        }
        ModKind::Native => {
            if rechecked
                .native
                .as_ref()
                .map_or(0, |native| native.total_entity_files)
                == 0
            {
                return Err(InstallFailure::NativeEmpty);
            }
        }
        ModKind::Data => {
            let entries = rechecked
                .data
                .as_ref()
                .map(|data| data.entries.as_slice())
                .unwrap_or_default();
            let has_compat = rechecked
                .data
                .as_ref()
                .is_some_and(|data| data.has_compat_config);
            let has_recognised = entries
                .iter()
                .any(|entry| entry.category != DataCategory::Other);
            if !has_recognised && !has_compat && entries.is_empty() && !has_cpk(&work_root) {
                return Err(InstallFailure::DataEmpty);
            }
        }
        ModKind::Texture => {}
    }

    std::fs::create_dir_all(&mods_root).map_err(io_failure(IoOp::Create, &mods_root))?;
    fsx::move_dir(&work_root, &target_dir).map_err(|err| InstallFailure::MoveFailed {
        path: target_dir.clone(),
        kind: err.kind(),
    })?;

    let local_sub_paths = normalize::stage_sibling_texture_sets(
        &drop_root,
        &work_root,
        &target_dir,
        &request.texture_pack_sub_paths,
    )
    .map_err(io_failure(IoOp::Copy, &target_dir))?;
    normalize::in_place_textures(&target_dir, &local_sub_paths);

    Ok(Installed {
        id: target_id,
        side_installed_texture_packs: Vec::new(),
        unpaired_warnings,
    })
}

pub fn install_variants(
    staging: &DropStaging,
    game: &GamePaths,
    source: &Path,
    requests: &[VariantInstallRequest],
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> Vec<InstallResult> {
    let staged = if DropStaging::is_archive(source) {
        match stage_archive(staging, game, source, progress, cancel) {
            Ok(staged) => staged,
            Err(failure) => return requests.iter().map(|_| Err(failure.clone())).collect(),
        }
    } else {
        StagedDrop::InPlace(source.to_path_buf())
    };
    install_batch_from_dir(
        game,
        staged.root(),
        &source_base_name(source),
        requests,
        cancel,
    )
}

pub(crate) fn install_batch_from_dir(
    game: &GamePaths,
    work_dir: &Path,
    source_folder_name: &str,
    requests: &[VariantInstallRequest],
    cancel: &CancelToken,
) -> Vec<InstallResult> {
    let detected = detect_dir(work_dir, source_folder_name);
    let drop_root = detected.unwrapped_root;
    let mods_root = game.mods_dir();
    if let Err(err) = std::fs::create_dir_all(&mods_root) {
        let failure = InstallFailure::io(IoOp::Create, &mods_root, &err);
        return requests.iter().map(|_| Err(failure.clone())).collect();
    }

    let mut used_ids: IndexSet<String> = IndexSet::new();
    let mut results = Vec::with_capacity(requests.len());
    for request in requests {
        if cancel.is_cancelled() {
            results.push(Err(InstallFailure::Cancelled));
            continue;
        }
        let result = install_one_variant(&drop_root, &mods_root, request, &used_ids);
        if let Ok(installed) = &result {
            used_ids.insert(installed.id.clone());
        }
        results.push(result);
    }
    results
}

fn install_one_variant(
    drop_root: &Path,
    mods_root: &Path,
    request: &VariantInstallRequest,
    used_ids: &IndexSet<String>,
) -> InstallResult {
    let variant_root = fsx::normalize(&drop_root.join(&request.variant_sub_path));
    if !variant_root.is_dir() {
        return Err(InstallFailure::VariantMissing);
    }

    let mut target_id = sanitize_id(&request.requested_name);
    let mut attempt = 1u32;
    while used_ids.contains(&target_id) || mods_root.join(&target_id).is_dir() {
        attempt += 1;
        target_id = sanitize_id(&format!("{} {attempt}", request.requested_name));
    }
    let target_dir = mods_root.join(&target_id);

    fsx::copy_dir(&variant_root, &target_dir).map_err(|err| InstallFailure::MoveFailed {
        path: target_dir.clone(),
        kind: err.kind(),
    })?;

    normalize::loose_data_dirs(&target_dir).map_err(io_failure(IoOp::Rename, &target_dir))?;
    let unpaired_warnings =
        normalize::loose_data_files(&target_dir).map_err(io_failure(IoOp::Rename, &target_dir))?;
    normalize::cpks(&target_dir);
    normalize::misplaced_wax_config_dirs(&target_dir);

    let local_sub_paths = normalize::stage_sibling_texture_sets(
        drop_root,
        &variant_root,
        &target_dir,
        &request.texture_pack_sub_paths,
    )
    .map_err(io_failure(IoOp::Copy, &target_dir))?;
    normalize::in_place_textures(&target_dir, &local_sub_paths);

    Ok(Installed {
        id: target_id,
        side_installed_texture_packs: Vec::new(),
        unpaired_warnings,
    })
}
