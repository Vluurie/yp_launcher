use std::sync::Arc;

use yp_formats::FormatError;

use crate::session::{ArchiveSide, ProbeWarning};

#[derive(Debug, Clone, thiserror::Error)]
pub enum InspectorError {
    #[error("no DAT or DTT path was provided")]
    NoArchives,
    #[error("none of the archives could be read")]
    NoReadableArchive { warnings: Vec<ProbeWarning> },
    #[error("the {0:?} archive path is missing")]
    MissingArchive(ArchiveSide),
    #[error("entry {index} no longer exists in the {side:?} archive")]
    EntryMissing { side: ArchiveSide, index: u32 },
    #[error("mesh {0} does not exist")]
    MeshMissing(u32),
    #[error("LOD {0} does not exist")]
    LodMissing(u32),
    #[error("render size {width}x{height} is too small")]
    RenderTooSmall { width: u32, height: u32 },
    #[error("render size {width}x{height} is too large")]
    RenderTooLarge { width: u32, height: u32 },
    #[error(transparent)]
    Format(Arc<FormatError>),
}

impl From<FormatError> for InspectorError {
    fn from(error: FormatError) -> Self {
        InspectorError::Format(Arc::new(error))
    }
}

pub type Result<T, E = InspectorError> = std::result::Result<T, E>;
