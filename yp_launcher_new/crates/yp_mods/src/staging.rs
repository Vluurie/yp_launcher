use std::path::{Path, PathBuf};

use tempfile::TempDir;
use yp_core::archive::Extractor;
use yp_core::{ArchiveFailure, CancelToken, Error, GamePaths, HostEnv, ProgressSink};

use crate::detect::is_loose_data_file;
use crate::fsx;

pub const EXTRACT_TEMP_PREFIX: &str = "archive_";
pub const LOOSE_DROP_TEMP_PREFIX: &str = "yp_loose_drop_";

#[derive(Clone)]
pub struct DropStaging {
    extractor: Extractor,
    host: HostEnv,
}

impl DropStaging {
    pub fn new(extractor: Extractor, host: HostEnv) -> Self {
        Self { extractor, host }
    }

    pub fn extractor(&self) -> &Extractor {
        &self.extractor
    }

    pub fn host(&self) -> &HostEnv {
        &self.host
    }

    pub fn is_archive(path: &Path) -> bool {
        yp_core::archive::is_archive(path)
    }

    pub fn list(&self, archive: &Path) -> Result<Vec<String>, Error> {
        let entries = self.extractor.list(archive)?;
        Ok(entries.into_iter().map(|entry| entry.path).collect())
    }

    pub fn list_entries(&self, archive: &Path) -> Option<Vec<String>> {
        self.list(archive).ok()
    }

    pub fn extract(
        &self,
        archive: &Path,
        game: Option<&GamePaths>,
        progress: &dyn ProgressSink,
        cancel: &CancelToken,
    ) -> Result<TempDir, Error> {
        cancel.check()?;
        let extracted = self
            .extractor
            .extract_to_temp(archive, game, &self.host, progress, cancel)?;
        cancel.check()?;
        if fsx::children(extracted.path()).is_empty() {
            return Err(Error::archive(archive, ArchiveFailure::NoFiles));
        }
        Ok(extracted)
    }

    pub(crate) fn stage_loose_file(
        &self,
        source: &Path,
        game: Option<&GamePaths>,
    ) -> Option<TempDir> {
        let name = fsx::file_name(source);
        if !is_loose_data_file(&name) {
            return None;
        }
        let dir = yp_core::fs::scoped_temp_dir(game, &self.host, LOOSE_DROP_TEMP_PREFIX).ok()?;
        std::fs::copy(source, dir.path().join(&name)).ok()?;
        Some(dir)
    }
}

pub(crate) enum StagedDrop {
    InPlace(PathBuf),
    Temp(TempDir),
}

impl StagedDrop {
    pub(crate) fn root(&self) -> &Path {
        match self {
            StagedDrop::InPlace(path) => path,
            StagedDrop::Temp(dir) => dir.path(),
        }
    }
}

pub(crate) fn source_base_name(source: &Path) -> String {
    source
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}
