mod names;
mod output;
mod rar;
mod sevenz;
mod zip;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use encoding_rs::Encoding;

use crate::error::{ArchiveFailure, Error, IoOp, IoResultExt, Result};
use crate::host::HostEnv;
use crate::paths::GamePaths;
use crate::task::{CancelToken, ProgressSink};

pub use names::BACKSLASH_IN_NAME;

use output::Output;

pub const ARCHIVE_TEMP_PREFIX: &str = "archive_";
pub const ARCHIVE_EXTENSIONS: [&str; 3] = [".zip", ".7z", ".rar"];

const SIGNATURE_LEN: usize = 8;
const ZIP_SIGNATURES: [[u8; 4]; 3] = [*b"PK\x03\x04", *b"PK\x05\x06", *b"PK\x07\x08"];
const SEVENZ_SIGNATURE: [u8; 6] = [0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c];
const RAR_SIGNATURE: [u8; 6] = *b"Rar!\x1a\x07";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveFormat {
    Zip,
    SevenZ,
    Rar,
}

impl ArchiveFormat {
    pub const ALL: [ArchiveFormat; 3] = [ArchiveFormat::Zip, ArchiveFormat::SevenZ, ArchiveFormat::Rar];

    pub const fn extension(self) -> &'static str {
        match self {
            ArchiveFormat::Zip => ARCHIVE_EXTENSIONS[0],
            ArchiveFormat::SevenZ => ARCHIVE_EXTENSIONS[1],
            ArchiveFormat::Rar => ARCHIVE_EXTENSIONS[2],
        }
    }

    pub fn from_signature(head: &[u8]) -> Option<Self> {
        if ZIP_SIGNATURES.iter().any(|signature| head.starts_with(signature)) {
            Some(ArchiveFormat::Zip)
        } else if head.starts_with(&SEVENZ_SIGNATURE) {
            Some(ArchiveFormat::SevenZ)
        } else if head.starts_with(&RAR_SIGNATURE) {
            Some(ArchiveFormat::Rar)
        } else {
            None
        }
    }

    pub fn from_extension(path: &Path) -> Option<Self> {
        let lower = path.to_string_lossy().to_lowercase();
        Self::ALL.into_iter().find(|format| lower.ends_with(format.extension()))
    }

    pub fn detect(path: &Path) -> Result<Self> {
        let mut head = [0u8; SIGNATURE_LEN];
        let mut filled = 0;
        let mut file = File::open(path).io(IoOp::Read, path)?;
        while filled < head.len() {
            match file.read(&mut head[filled..]) {
                Ok(0) => break,
                Ok(read) => filled += read,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => return Err(error).io(IoOp::Read, path),
            }
        }
        Self::from_signature(&head[..filled])
            .or_else(|| Self::from_extension(path))
            .ok_or_else(|| Error::archive(path, ArchiveFailure::UnsupportedFormat))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

impl ArchiveEntry {
    pub fn normalized_lower(&self) -> String {
        self.path.to_lowercase()
    }

    pub(crate) fn from_parts(parts: &[String], is_dir: bool, size: u64) -> Option<Self> {
        if parts.is_empty() {
            return None;
        }
        Some(Self {
            path: parts.join("/"),
            is_dir,
            size: if is_dir { 0 } else { size },
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Extractor {
    oem: Option<&'static Encoding>,
}

impl Extractor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_oem_code_page(code_page: Option<u32>) -> Self {
        Self {
            oem: code_page.and_then(names::oem_encoding),
        }
    }

    pub fn legacy_name_encoding(&self) -> Option<&'static str> {
        self.oem.map(Encoding::name)
    }

    pub fn list(&self, archive: &Path) -> Result<Vec<ArchiveEntry>> {
        let listed = match ArchiveFormat::detect(archive)? {
            ArchiveFormat::Zip => zip::list(archive, self.oem),
            ArchiveFormat::SevenZ => sevenz::list(archive),
            ArchiveFormat::Rar => rar::list(archive),
        };
        if let Err(error) = &listed {
            log::warn!("listing of {} failed: {error}", archive.display());
        }
        listed
    }

    pub fn contains_any(&self, archive: &Path, matches: &mut dyn FnMut(&str) -> bool) -> Result<bool> {
        Ok(self
            .list(archive)?
            .iter()
            .any(|entry| matches(&entry.normalized_lower())))
    }

    pub fn contains_extension(&self, archive: &Path, extensions: &[&str]) -> Result<bool> {
        let wanted: Vec<String> = extensions.iter().map(|ext| ext.to_lowercase()).collect();
        self.contains_any(archive, &mut |entry| wanted.iter().any(|ext| entry.ends_with(ext.as_str())))
    }

    pub fn extract(
        &self,
        archive: &Path,
        dest: &Path,
        progress: &dyn ProgressSink,
        cancel: &CancelToken,
    ) -> Result<()> {
        cancel.check()?;
        let format = ArchiveFormat::detect(archive)?;
        std::fs::create_dir_all(dest).io(IoOp::Create, dest)?;
        let mut output = Output::new(archive, dest, progress, cancel);
        let extracted = match format {
            ArchiveFormat::Zip => zip::extract(archive, self.oem, &mut output),
            ArchiveFormat::SevenZ => sevenz::extract(archive, &mut output),
            ArchiveFormat::Rar => rar::extract(archive, &mut output),
        };
        if let Err(error) = extracted {
            output.discard();
            if !error.is_cancelled() {
                log::warn!("extraction of {} failed: {error}", archive.display());
            }
            return Err(error);
        }
        output.finish();
        let has_files = std::fs::read_dir(dest).io(IoOp::List, dest)?.next().is_some();
        if !has_files {
            return Err(Error::archive(archive, ArchiveFailure::NoFiles));
        }
        Ok(())
    }

    pub fn extract_to_temp(
        &self,
        archive: &Path,
        game: Option<&GamePaths>,
        host: &HostEnv,
        progress: &dyn ProgressSink,
        cancel: &CancelToken,
    ) -> Result<tempfile::TempDir> {
        if !is_archive(archive) {
            return Err(Error::archive(archive, ArchiveFailure::UnsupportedFormat));
        }
        let dir = crate::fs::scoped_temp_dir(game, host, ARCHIVE_TEMP_PREFIX)?;
        self.extract(archive, dir.path(), progress, cancel)?;
        Ok(dir)
    }
}

pub fn is_archive(path: &Path) -> bool {
    ArchiveFormat::from_extension(path).is_some()
}

pub fn is_zip(path: &Path) -> bool {
    ArchiveFormat::from_extension(path) == Some(ArchiveFormat::Zip)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_extensions() {
        assert!(is_archive(Path::new("Mod.ZIP")));
        assert!(is_archive(Path::new("C:/Downloads/Mod-123.7z")));
        assert!(is_archive(Path::new("x.RAR")));
        assert!(!is_archive(Path::new("a.dat")));
        assert!(!is_archive(Path::new("x.cpk")));
        assert!(is_zip(Path::new("x.zip")));
        assert!(!is_zip(Path::new("x.7z")));
        for dropped in ["a.tar", "a.tar.gz", "a.gz", "a.bz2", "a.tgz"] {
            assert!(!is_archive(Path::new(dropped)), "{dropped}");
        }
    }

    #[test]
    fn signatures_name_the_format() {
        assert_eq!(ArchiveFormat::from_signature(b"PK\x03\x04rest"), Some(ArchiveFormat::Zip));
        assert_eq!(ArchiveFormat::from_signature(b"PK\x05\x06"), Some(ArchiveFormat::Zip));
        assert_eq!(
            ArchiveFormat::from_signature(&[0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c, 0, 4]),
            Some(ArchiveFormat::SevenZ)
        );
        assert_eq!(ArchiveFormat::from_signature(b"Rar!\x1a\x07\x00"), Some(ArchiveFormat::Rar));
        assert_eq!(ArchiveFormat::from_signature(b"Rar!\x1a\x07\x01\x00"), Some(ArchiveFormat::Rar));
        assert_eq!(ArchiveFormat::from_signature(b"PK"), None);
        assert_eq!(ArchiveFormat::from_signature(b"\x1f\x8b\x08\x00"), None);
        assert_eq!(ArchiveFormat::from_signature(b""), None);
    }

    #[test]
    fn extensions_are_the_fallback() {
        assert_eq!(ArchiveFormat::from_extension(Path::new("a/b.Zip")), Some(ArchiveFormat::Zip));
        assert_eq!(ArchiveFormat::from_extension(Path::new("b.7Z")), Some(ArchiveFormat::SevenZ));
        assert_eq!(ArchiveFormat::from_extension(Path::new("b.part1.rar")), Some(ArchiveFormat::Rar));
        assert_eq!(ArchiveFormat::from_extension(Path::new("b.tar.gz")), None);
        assert_eq!(ArchiveFormat::from_extension(Path::new("zip")), None);
    }
}
