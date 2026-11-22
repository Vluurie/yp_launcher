use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

use unrar_ng::error::{Code, UnrarError, When};
use unrar_ng::{Archive, FileHeader, VolumeInfo};

use super::names::{Backslash, entry_parts};
use super::output::Output;
use super::ArchiveEntry;
use crate::error::{ArchiveFailure, Error, IoOp, Result};

static UNRAR: Mutex<()> = Mutex::new(());

fn exclusive() -> MutexGuard<'static, ()> {
    UNRAR.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) fn archive_failure(error: &UnrarError) -> Option<ArchiveFailure> {
    Some(match (error.code, error.when) {
        (Code::MissingPassword | Code::BadPassword, _) => ArchiveFailure::PasswordRequired,
        (Code::EOpen, When::Open) => return None,
        (Code::EOpen, _) => ArchiveFailure::MissingVolume,
        (Code::EWrite, _) => ArchiveFailure::DiskFull,
        (Code::UnknownFormat | Code::LargeDict, _) => ArchiveFailure::UnsupportedMethod,
        (Code::ERead | Code::ECreate | Code::EClose | Code::NoMemory, _) => return None,
        _ => ArchiveFailure::Corrupt,
    })
}

fn io_failure(op: IoOp, path: &Path, error: UnrarError) -> Error {
    let kind = match error.code {
        Code::NoMemory => std::io::ErrorKind::OutOfMemory,
        _ => std::io::ErrorKind::Other,
    };
    Error::Io {
        op,
        path: path.to_path_buf(),
        source: std::io::Error::new(kind, error),
    }
}

fn failure(archive: &Path, error: UnrarError) -> Error {
    match archive_failure(&error) {
        Some(failure) => Error::archive(archive, failure),
        None => io_failure(IoOp::Read, archive, error),
    }
}

fn output_failure(archive: &Path, target: &Path, error: UnrarError) -> Error {
    match error.code {
        Code::ECreate => io_failure(IoOp::Create, target, error),
        Code::EClose => io_failure(IoOp::Write, target, error),
        _ => failure(archive, error),
    }
}

struct Listed {
    parts: Vec<String>,
    is_dir: bool,
    size: u64,
    encrypted: bool,
}

fn parts_of(header: &FileHeader) -> Vec<String> {
    entry_parts(&header.filename.to_string_lossy(), header.is_directory(), Backslash::Separator)
}

fn headers(archive: &Path) -> Result<Vec<Listed>> {
    let listing = Archive::new(archive)
        .open_for_listing()
        .map_err(|error| failure(archive, error))?;
    if listing.volume_info() == VolumeInfo::Subsequent {
        return Err(Error::archive(archive, ArchiveFailure::MissingVolume));
    }
    let mut listed = Vec::new();
    for header in listing {
        let header = header.map_err(|error| failure(archive, error))?;
        listed.push(Listed {
            parts: parts_of(&header),
            is_dir: header.is_directory(),
            size: header.unpacked_size,
            encrypted: header.is_encrypted(),
        });
    }
    Ok(listed)
}

pub(crate) fn list(archive: &Path) -> Result<Vec<ArchiveEntry>> {
    let _unrar = exclusive();
    Ok(headers(archive)?
        .iter()
        .filter_map(|entry| ArchiveEntry::from_parts(&entry.parts, entry.is_dir, entry.size))
        .collect())
}

pub(crate) fn extract(archive: &Path, output: &mut Output<'_>) -> Result<()> {
    let _unrar = exclusive();
    let listed = headers(archive)?;
    if listed.iter().any(|entry| entry.encrypted) {
        return Err(Error::archive(archive, ArchiveFailure::PasswordRequired));
    }
    output.set_total(listed.iter().filter(|entry| !entry.is_dir).map(|entry| entry.size).sum());
    let mut cursor = Archive::new(archive)
        .open_for_processing()
        .map_err(|error| failure(archive, error))?;
    loop {
        output.check_cancel()?;
        let Some(entry) = cursor.read_header().map_err(|error| failure(archive, error))? else {
            return Ok(());
        };
        let header = entry.entry();
        let parts = parts_of(header);
        let size = header.unpacked_size;
        cursor = if parts.is_empty() {
            entry.skip().map_err(|error| failure(archive, error))?
        } else if header.is_directory() {
            output.create_dir(&parts)?;
            entry.skip().map_err(|error| failure(archive, error))?
        } else {
            let target = output.begin_file(&parts, size)?;
            let next = entry
                .extract_to(&target)
                .map_err(|error| output_failure(archive, &target, error))?;
            output.advance(size);
            next
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn of(code: Code, when: When) -> Error {
        failure(Path::new("a.rar"), UnrarError::from(code, when))
    }

    #[test]
    fn typed_failures_from_unrar_codes() {
        assert_eq!(of(Code::MissingPassword, When::Read).archive_failure(), Some(ArchiveFailure::PasswordRequired));
        assert_eq!(of(Code::BadPassword, When::Process).archive_failure(), Some(ArchiveFailure::PasswordRequired));
        assert_eq!(of(Code::EOpen, When::Process).archive_failure(), Some(ArchiveFailure::MissingVolume));
        assert_eq!(of(Code::EOpen, When::Read).archive_failure(), Some(ArchiveFailure::MissingVolume));
        assert_eq!(of(Code::EWrite, When::Process).archive_failure(), Some(ArchiveFailure::DiskFull));
        assert_eq!(of(Code::UnknownFormat, When::Process).archive_failure(), Some(ArchiveFailure::UnsupportedMethod));
        assert_eq!(of(Code::LargeDict, When::Process).archive_failure(), Some(ArchiveFailure::UnsupportedMethod));
        assert_eq!(of(Code::BadData, When::Process).archive_failure(), Some(ArchiveFailure::Corrupt));
        assert_eq!(of(Code::BadArchive, When::Open).archive_failure(), Some(ArchiveFailure::Corrupt));
        assert_eq!(of(Code::Unmapped(99), When::Read).archive_failure(), Some(ArchiveFailure::Corrupt));
        assert!(matches!(of(Code::EOpen, When::Open), Error::Io { op: IoOp::Read, .. }));
        assert!(matches!(of(Code::ERead, When::Process), Error::Io { op: IoOp::Read, .. }));
    }

    #[test]
    fn create_failures_name_the_output_file() {
        let target = Path::new("out/file.bin");
        let create = output_failure(Path::new("a.rar"), target, UnrarError::from(Code::ECreate, When::Process));
        assert!(matches!(create, Error::Io { op: IoOp::Create, ref path, .. } if path == target));
        let full = output_failure(Path::new("a.rar"), target, UnrarError::from(Code::EWrite, When::Process));
        assert!(full.is_disk_full());
    }
}
