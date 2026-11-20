use std::path::{Path, PathBuf};

use yp_core::{CancelToken, GamePaths, ProgressSink};

use crate::detect::detect_drop;
use crate::fsx;
use crate::install::{InstallRequest, install, install_variants};
use crate::model::{DropRejection, InstallFailure, InstallResult, Installed};
use crate::staging::DropStaging;
use crate::text::has_extension_in;
use crate::variants;

pub const ARCHIVE_EXTENSIONS: [&str; 3] = [".zip", ".7z", ".rar"];
pub const MAX_SCAN_DEPTH: usize = 2;
pub const FIRST_NAME_SUFFIX: u32 = 2;
pub const NAME_SUFFIX_LIMIT: u32 = 100;

pub fn find_archives(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return out;
    }
    collect_archives(root, 1, &mut out);
    out.sort_by_cached_key(|path| {
        path.to_string_lossy()
            .to_lowercase()
            .encode_utf16()
            .collect::<Vec<u16>>()
    });
    out
}

fn collect_archives(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    for child in fsx::children_no_follow(dir) {
        if child.is_dir {
            collect_archives(&child.path, depth + 1, out);
        } else if child.is_file && has_extension_in(&child.name, &ARCHIVE_EXTENSIONS) {
            out.push(child.path);
        }
    }
}

pub fn install_auto_named(
    staging: &DropStaging,
    game: &GamePaths,
    source: &Path,
    base_name: &str,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> InstallResult {
    let mut name = base_name.to_owned();
    let mut taken = String::new();
    for attempt in FIRST_NAME_SUFFIX..NAME_SUFFIX_LIMIT {
        let request = InstallRequest::named(name);
        match install(staging, game, source, &request, progress, cancel) {
            Err(InstallFailure::Exists { id }) => taken = id,
            other => return other,
        }
        name = format!("{base_name} {attempt}");
    }
    Err(InstallFailure::Exists { id: taken })
}

pub fn install_archive(
    staging: &DropStaging,
    game: &GamePaths,
    archive: &Path,
    requested_name: &str,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> InstallResult {
    let detected = detect_drop(staging, archive, progress, cancel).map_err(InstallFailure::extraction)?;
    if !detected.has_variants() {
        return install_auto_named(staging, game, archive, requested_name, progress, cancel);
    }
    let requests = variants::requests_without_textures(requested_name, &detected.variants);
    let mut first_failure = None;
    for result in install_variants(staging, game, archive, &requests, progress, cancel) {
        match result {
            Ok(installed) => {
                return Ok(Installed {
                    id: installed.id,
                    ..Installed::default()
                });
            }
            Err(failure) => {
                first_failure.get_or_insert(failure);
            }
        }
    }
    Err(first_failure.unwrap_or(InstallFailure::Rejected(DropRejection::UnknownDrop)))
}
