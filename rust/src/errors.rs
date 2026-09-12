#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodedError {
    pub code: u32,
    pub detail: String,
}

impl CodedError {
    pub fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code: code as u32,
            detail: detail.into(),
        }
    }

    pub fn bare(code: ErrorCode) -> Self {
        Self {
            code: code as u32,
            detail: String::new(),
        }
    }
}

impl std::fmt::Display for CodedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.detail.is_empty() {
            write!(f, "{}", self.code)
        } else {
            write!(f, "{}: {}", self.code, self.detail)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCode {
    Unknown = 1000,

    ImageUnreadable = 1100,
    ImageSizeOutOfRange = 1101,
    ImageSizeNotMultipleOfFour = 1102,

    ArchiveEmpty = 1200,
    ArchiveExtensionTooLong = 1201,
    WeaponStemUnreadable = 1202,
    WeaponStemLengthMismatch = 1203,
    WeaponSameStem = 1204,
    WeaponNothingToRename = 1205,

    TextureNoWtp = 1210,
    TextureNotDds = 1211,
    TextureBadSize = 1212,
    TextureUnsupportedFormat = 1213,
    TextureTruncated = 1214,
    TextureEncodeFailed = 1215,

    FileReadFailed = 1300,
    FileWriteFailed = 1301,
    DirectoryCreateFailed = 1302,
}
