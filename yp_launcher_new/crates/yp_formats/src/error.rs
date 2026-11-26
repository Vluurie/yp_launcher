use std::fmt;
use std::path::{Path, PathBuf};

use crate::weapon::WeaponClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileOp {
    Read,
    Write,
    CreateDir,
}

impl fmt::Display for FileOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FileOp::Read => "read",
            FileOp::Write => "write",
            FileOp::CreateDir => "create directory",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatFault {
    SmallerThanHeader,
    UnsupportedMagic,
    TooManyEntries,
    TableOverflow,
    RangeOutsideFile,
    InvalidNameWidth,
    EntryTooLarge,
}

impl fmt::Display for DatFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DatFault::SmallerThanHeader => "archive is smaller than its header",
            DatFault::UnsupportedMagic => "archive has an unsupported header",
            DatFault::TooManyEntries => "archive contains too many entries",
            DatFault::TableOverflow => "archive table size overflowed",
            DatFault::RangeOutsideFile => "archive range points outside the file",
            DatFault::InvalidNameWidth => "archive name width is invalid",
            DatFault::EntryTooLarge => "archive entry is too large",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelFault {
    NotWmb3,
    OutsideEntry,
    RangeOverflow,
    TooManyEntries,
    TooLarge,
    InvalidBatchIndex,
    InvalidVertexGroupIndex,
    UnterminatedString,
    UnsupportedVertexFlags(u32),
    NoRenderableBatches,
    OlderLayout(u32),
}

impl fmt::Display for ModelFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelFault::NotWmb3 => f.write_str("entry is not a WMB3 model"),
            ModelFault::OutsideEntry => f.write_str("model data points outside the entry"),
            ModelFault::RangeOverflow => f.write_str("model data range overflowed"),
            ModelFault::TooManyEntries => f.write_str("model table contains too many entries"),
            ModelFault::TooLarge => f.write_str("model is too large"),
            ModelFault::InvalidBatchIndex => f.write_str("model LOD batch index is invalid"),
            ModelFault::InvalidVertexGroupIndex => {
                f.write_str("model vertex group index is invalid")
            }
            ModelFault::UnterminatedString => f.write_str("model string is not terminated"),
            ModelFault::UnsupportedVertexFlags(flags) => {
                write!(f, "model vertex flags {flags} are unsupported")
            }
            ModelFault::NoRenderableBatches => f.write_str("model contains no renderable batches"),
            ModelFault::OlderLayout(version) => write!(
                f,
                "model layout {version:#010x} is not the one the game reads"
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsmFault {
    NotUsm,
    Truncated,
    InvalidChunk,
    InvalidTable,
    UnknownColumnType(u8),
    EmptyTable,
    NoVideoHeader,
}

impl fmt::Display for UsmFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsmFault::NotUsm => f.write_str("file does not start with a CRID chunk"),
            UsmFault::Truncated => f.write_str("file ends inside a chunk"),
            UsmFault::InvalidChunk => f.write_str("chunk header is invalid"),
            UsmFault::InvalidTable => f.write_str("@UTF table is invalid"),
            UsmFault::UnknownColumnType(kind) => write!(f, "@UTF column type {kind:#x} is unknown"),
            UsmFault::EmptyTable => f.write_str("@UTF table has no rows"),
            UsmFault::NoVideoHeader => f.write_str("no @SFV header chunk was found"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("image could not be decoded: {detail}")]
    ImageUnreadable { detail: String },
    #[error("image size {width}x{height} is outside 4..=4096")]
    ImageSizeOutOfRange { width: u32, height: u32 },
    #[error("image size {width}x{height} is not a multiple of four")]
    ImageSizeNotMultipleOfFour { width: u32, height: u32 },
    #[error("nothing to pack")]
    ArchiveEmpty,
    #[error("extension of {name} is longer than three characters")]
    ArchiveExtensionTooLong { name: String },
    #[error("archive has too many entries to pack: {count}")]
    ArchiveTooManyEntries { count: usize },
    #[error("{0}")]
    Dat(DatFault),
    #[error("weapon stem {stem:?} is not a weapon")]
    WeaponStemUnreadable { stem: String },
    #[error("weapon stems differ in length: {from} -> {to}")]
    WeaponStemLengthMismatch { from: String, to: String },
    #[error("weapon class differs: {from:?} -> {to:?}")]
    WeaponClassMismatch { from: WeaponClass, to: WeaponClass },
    #[error("weapon is already {stem}")]
    WeaponSameStem { stem: String },
    #[error("no entry of {stem} to rename")]
    WeaponNothingToRename { stem: String },
    #[error("archive has no .wtp texture")]
    TextureNoWtp,
    #[error("texture is not a DDS file")]
    TextureNotDds,
    #[error("texture size {width}x{height} is invalid")]
    TextureBadSize { width: u32, height: u32 },
    #[error("texture format {} is unsupported", String::from_utf8_lossy(fourcc))]
    TextureUnsupportedFormat { fourcc: [u8; 4] },
    #[error("texture data is truncated: {available} < {expected}")]
    TextureTruncated { available: usize, expected: usize },
    #[error("texture could not be encoded: {detail}")]
    TextureEncodeFailed { detail: String },
    #[error("{0}")]
    Model(ModelFault),
    #[error("{0}")]
    Usm(UsmFault),
    #[error("{name} is not in the game archives")]
    GameFileMissing { name: String },
    #[error("{name} could not be read: {detail}")]
    GameFileDamaged { name: String, detail: &'static str },
    #[error("the font cache is damaged")]
    FontCacheDamaged,
    #[error("{op} failed for {}: {source}", path.display())]
    Io {
        op: FileOp,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl FormatError {
    pub fn io(op: FileOp, path: &Path, source: std::io::Error) -> Self {
        FormatError::Io {
            op,
            path: path.to_path_buf(),
            source,
        }
    }

    pub fn code(&self) -> u32 {
        match self {
            FormatError::ImageUnreadable { .. } => 1100,
            FormatError::ImageSizeOutOfRange { .. } => 1101,
            FormatError::ImageSizeNotMultipleOfFour { .. } => 1102,
            FormatError::ArchiveEmpty => 1200,
            FormatError::ArchiveExtensionTooLong { .. } => 1201,
            FormatError::WeaponStemUnreadable { .. } => 1202,
            FormatError::WeaponStemLengthMismatch { .. } => 1203,
            FormatError::WeaponSameStem { .. } => 1204,
            FormatError::WeaponNothingToRename { .. } => 1205,
            FormatError::WeaponClassMismatch { .. } => 1206,
            FormatError::ArchiveTooManyEntries { .. } => 1207,
            FormatError::TextureNoWtp => 1210,
            FormatError::TextureNotDds => 1211,
            FormatError::TextureBadSize { .. } => 1212,
            FormatError::TextureUnsupportedFormat { .. } => 1213,
            FormatError::TextureTruncated { .. } => 1214,
            FormatError::TextureEncodeFailed { .. } => 1215,
            FormatError::Dat(_) => 1220,
            FormatError::Model(_) => 1230,
            FormatError::Usm(_) => 1240,
            FormatError::GameFileMissing { .. } => 1250,
            FormatError::GameFileDamaged { .. } => 1251,
            FormatError::FontCacheDamaged => 1252,
            FormatError::Io {
                op: FileOp::Read, ..
            } => 1300,
            FormatError::Io {
                op: FileOp::Write, ..
            } => 1301,
            FormatError::Io {
                op: FileOp::CreateDir,
                ..
            } => 1302,
        }
    }

    pub fn is_texture_unreadable(&self) -> bool {
        matches!(
            self,
            FormatError::TextureNoWtp
                | FormatError::TextureNotDds
                | FormatError::TextureBadSize { .. }
                | FormatError::TextureUnsupportedFormat { .. }
                | FormatError::TextureTruncated { .. }
                | FormatError::TextureEncodeFailed { .. }
                | FormatError::Dat(_)
        )
    }
}

impl From<DatFault> for FormatError {
    fn from(fault: DatFault) -> Self {
        FormatError::Dat(fault)
    }
}

impl From<ModelFault> for FormatError {
    fn from(fault: ModelFault) -> Self {
        FormatError::Model(fault)
    }
}

impl From<UsmFault> for FormatError {
    fn from(fault: UsmFault) -> Self {
        FormatError::Usm(fault)
    }
}

pub type Result<T, E = FormatError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_unique() {
        let errors = vec![
            FormatError::ImageUnreadable {
                detail: String::new(),
            },
            FormatError::ImageSizeOutOfRange {
                width: 0,
                height: 0,
            },
            FormatError::ImageSizeNotMultipleOfFour {
                width: 0,
                height: 0,
            },
            FormatError::ArchiveEmpty,
            FormatError::ArchiveExtensionTooLong {
                name: String::new(),
            },
            FormatError::ArchiveTooManyEntries { count: 0 },
            FormatError::Dat(DatFault::UnsupportedMagic),
            FormatError::WeaponStemUnreadable {
                stem: String::new(),
            },
            FormatError::WeaponStemLengthMismatch {
                from: String::new(),
                to: String::new(),
            },
            FormatError::WeaponClassMismatch {
                from: WeaponClass::Spear,
                to: WeaponClass::SmallSword,
            },
            FormatError::WeaponSameStem {
                stem: String::new(),
            },
            FormatError::WeaponNothingToRename {
                stem: String::new(),
            },
            FormatError::TextureNoWtp,
            FormatError::TextureNotDds,
            FormatError::TextureBadSize {
                width: 0,
                height: 0,
            },
            FormatError::TextureUnsupportedFormat { fourcc: *b"ATI2" },
            FormatError::TextureTruncated {
                available: 0,
                expected: 1,
            },
            FormatError::TextureEncodeFailed {
                detail: String::new(),
            },
            FormatError::Model(ModelFault::NotWmb3),
            FormatError::Usm(UsmFault::NotUsm),
            FormatError::io(
                FileOp::Read,
                Path::new("a"),
                std::io::ErrorKind::NotFound.into(),
            ),
            FormatError::io(
                FileOp::Write,
                Path::new("a"),
                std::io::ErrorKind::NotFound.into(),
            ),
            FormatError::io(
                FileOp::CreateDir,
                Path::new("a"),
                std::io::ErrorKind::NotFound.into(),
            ),
        ];
        let mut codes: Vec<u32> = errors.iter().map(FormatError::code).collect();
        let total = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), total);
    }

    #[test]
    fn weapon_codes_match_the_shipped_rust_crate() {
        assert_eq!(
            FormatError::WeaponStemUnreadable {
                stem: String::new()
            }
            .code(),
            1202
        );
        assert_eq!(
            FormatError::WeaponStemLengthMismatch {
                from: String::new(),
                to: String::new()
            }
            .code(),
            1203
        );
        assert_eq!(
            FormatError::WeaponSameStem {
                stem: String::new()
            }
            .code(),
            1204
        );
        assert_eq!(
            FormatError::WeaponNothingToRename {
                stem: String::new()
            }
            .code(),
            1205
        );
    }

    #[test]
    fn texture_family_is_grouped() {
        assert!(FormatError::TextureNotDds.is_texture_unreadable());
        assert!(!FormatError::ArchiveEmpty.is_texture_unreadable());
    }
}
