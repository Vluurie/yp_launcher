use std::path::{Path, PathBuf};

use tempfile::TempDir;
use yp_core::archive::{self, Extractor};
use yp_core::{CancelToken, GamePaths, HostEnv, ProgressSink};

use crate::classify;
use crate::model::Classification;

pub const LOOSE_FILE_TEMP_PREFIX: &str = "yp_tp_file_";
pub const PICKER_EXTENSIONS: [&str; 6] = ["zip", "7z", "rar", "ini", "exe", "dll"];

const EXECUTABLE_EXTENSION: &str = ".exe";
const LOOSE_FILE_EXTENSIONS: [&str; 2] = [".ini", ".dll"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Archive,
    Executable,
    LooseFile,
    Folder,
    Unsupported,
}

pub fn source_kind(source: &Path) -> SourceKind {
    let lower = source.to_string_lossy().to_lowercase();
    if archive::is_archive(source) {
        SourceKind::Archive
    } else if lower.ends_with(EXECUTABLE_EXTENSION) {
        SourceKind::Executable
    } else if LOOSE_FILE_EXTENSIONS.iter().any(|extension| lower.ends_with(extension)) {
        SourceKind::LooseFile
    } else if source.is_dir() {
        SourceKind::Folder
    } else {
        SourceKind::Unsupported
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StageFailure {
    #[error("archive could not be extracted")]
    ExtractFailed(#[source] yp_core::Error),
    #[error("executables are not unpacked")]
    ExecutableNotSupported,
    #[error("file type is not supported")]
    UnsupportedFile,
    #[error("cancelled")]
    Cancelled,
    #[error("dropped file could not be staged")]
    Io(#[source] yp_core::Error),
}

#[derive(Debug)]
pub struct StagedSource {
    root: PathBuf,
    source: PathBuf,
    temp: Option<TempDir>,
}

impl StagedSource {
    fn in_place(source: &Path) -> Self {
        Self {
            root: source.to_path_buf(),
            source: source.to_path_buf(),
            temp: None,
        }
    }

    fn temporary(source: &Path, temp: TempDir) -> Self {
        Self {
            root: temp.path().to_path_buf(),
            source: source.to_path_buf(),
            temp: Some(temp),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn is_temporary(&self) -> bool {
        self.temp.is_some()
    }

    pub fn classify(&self) -> Classification {
        classify::classify(&self.root).named(classify::source_name_from_path(&self.source))
    }
}

pub fn stage_source(
    extractor: &Extractor,
    host: &HostEnv,
    game: Option<&GamePaths>,
    source: &Path,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> Result<StagedSource, StageFailure> {
    if cancel.is_cancelled() {
        return Err(StageFailure::Cancelled);
    }
    match source_kind(source) {
        SourceKind::Archive => {
            let temp = extractor.extract_to_temp(source, game, host, progress, cancel).map_err(extract_failure)?;
            Ok(StagedSource::temporary(source, temp))
        }
        SourceKind::Executable => Err(StageFailure::ExecutableNotSupported),
        SourceKind::LooseFile => {
            let temp = yp_core::fs::scoped_temp_dir(game, host, LOOSE_FILE_TEMP_PREFIX).map_err(StageFailure::Io)?;
            if source.is_file() {
                yp_core::fs::copy_file_into(source, temp.path()).map_err(StageFailure::Io)?;
            }
            Ok(StagedSource::temporary(source, temp))
        }
        SourceKind::Folder => Ok(StagedSource::in_place(source)),
        SourceKind::Unsupported => Err(StageFailure::UnsupportedFile),
    }
}

fn extract_failure(error: yp_core::Error) -> StageFailure {
    match error {
        yp_core::Error::Cancelled => StageFailure::Cancelled,
        other => StageFailure::ExtractFailed(other),
    }
}
