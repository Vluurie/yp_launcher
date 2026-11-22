use std::path::{Path, PathBuf};

use tempfile::TempDir;
use yp_core::{CancelToken, IoOp, IoResultExt, Progress, ProgressSink};

use crate::detect::is_loose_data_file;
use crate::fsx;
use crate::staging::{DropStaging, LOOSE_DROP_TEMP_PREFIX, source_base_name};
use crate::text::has_extension_in;

pub const STAGE_TEMP_PREFIX: &str = "yp_loose_stage_";
pub const PROGRESS_INTERVAL: usize = 100;
pub const LOOSE_EXTENSIONS: [&str; 2] = [".dat", ".dtt"];

fn loose_files(root: &Path) -> Vec<fsx::Child> {
    if !root.is_dir() {
        return Vec::new();
    }
    fsx::children_no_follow(root)
        .into_iter()
        .filter(|child| child.is_file && has_extension_in(&child.name, &LOOSE_EXTENSIONS))
        .collect()
}

pub fn count_files(root: &Path) -> usize {
    loose_files(root).len()
}

pub fn stage_folder(
    staging: &DropStaging,
    root: &Path,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> yp_core::Result<Option<TempDir>> {
    let loose = loose_files(root);
    if loose.is_empty() {
        return Ok(None);
    }
    let staged = yp_core::fs::scoped_temp_dir(None, staging.host(), STAGE_TEMP_PREFIX)?;
    let total = loose.len();
    for (index, file) in loose.iter().enumerate() {
        cancel.check()?;
        let target = staged.path().join(&file.name);
        std::fs::copy(&file.path, &target).io(IoOp::Copy, &target)?;
        let done = index + 1;
        if done % PROGRESS_INTERVAL == 0 || done == total {
            progress.report(Progress::step(done, total, done.to_string()));
        }
    }
    Ok(Some(staged))
}

#[derive(Debug)]
pub struct StagedLooseFiles {
    root: TempDir,
    source: PathBuf,
    copied: usize,
}

impl StagedLooseFiles {
    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn copied(&self) -> usize {
        self.copied
    }
}

pub fn stage_files(
    staging: &DropStaging,
    files: &[PathBuf],
) -> yp_core::Result<Option<StagedLooseFiles>> {
    let Some(first) = files.first() else {
        return Ok(None);
    };
    let root = yp_core::fs::scoped_temp_dir(None, staging.host(), LOOSE_DROP_TEMP_PREFIX)?;
    let source = root.path().join(source_base_name(first));
    std::fs::create_dir(&source).io(IoOp::Create, &source)?;
    let copied = files
        .iter()
        .filter(|file| std::fs::copy(file, source.join(fsx::file_name(file))).is_ok())
        .count();
    if copied == 0 {
        return Ok(None);
    }
    Ok(Some(StagedLooseFiles {
        root,
        source,
        copied,
    }))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DropSplit {
    pub loose_files: Vec<PathBuf>,
    pub rest: Vec<PathBuf>,
}

pub fn split_drop(paths: &[PathBuf]) -> DropSplit {
    let mut split = DropSplit::default();
    for path in paths {
        let is_loose = path.is_file()
            && !DropStaging::is_archive(path)
            && is_loose_data_file(&fsx::file_name(path));
        if is_loose {
            split.loose_files.push(path.clone());
        } else {
            split.rest.push(path.clone());
        }
    }
    if split.loose_files.len() == 1 {
        split.rest.splice(0..0, split.loose_files.drain(..));
    }
    split
}
