use std::fs::File;
use std::io::Read;
use std::path::Path;

use sevenz_rust2::{ArchiveReader, EncoderMethod, EntryDecision, Password};

use super::names::{Backslash, entry_parts};
use super::output::{Output, filetime_to_system, read_failure};
use super::ArchiveEntry;
use crate::error::{ArchiveFailure, Error, Result};

type SevenZError = sevenz_rust2::Error;

pub(crate) fn archive_failure(error: &SevenZError) -> Option<ArchiveFailure> {
    Some(match error {
        SevenZError::Io(..) | SevenZError::FileOpen(..) => return None,
        SevenZError::PasswordRequired | SevenZError::MaybeBadPassword(_) => ArchiveFailure::PasswordRequired,
        SevenZError::UnsupportedCompressionMethod(method) if method == EncoderMethod::AES256_SHA256.name() => {
            ArchiveFailure::PasswordRequired
        }
        SevenZError::UnsupportedCompressionMethod(_)
        | SevenZError::Unsupported(_)
        | SevenZError::ExternalUnsupported
        | SevenZError::UnsupportedVersion { .. }
        | SevenZError::MaxMemLimited { .. } => ArchiveFailure::UnsupportedMethod,
        _ => ArchiveFailure::Corrupt,
    })
}

fn failure(archive: &Path, error: SevenZError) -> Error {
    match error {
        SevenZError::Io(source, _) | SevenZError::FileOpen(source, _) => read_failure(archive, source),
        other => Error::archive(archive, archive_failure(&other).unwrap_or(ArchiveFailure::Corrupt)),
    }
}

fn open(archive: &Path) -> Result<ArchiveReader<File>> {
    ArchiveReader::open(archive, Password::empty()).map_err(|error| failure(archive, error))
}

fn parts_of(entry: &sevenz_rust2::ArchiveEntry) -> Vec<String> {
    entry_parts(entry.name(), entry.is_directory(), Backslash::NameCharacter)
}

pub(crate) fn list(archive: &Path) -> Result<Vec<ArchiveEntry>> {
    let reader = open(archive)?;
    Ok(reader
        .archive()
        .files
        .iter()
        .filter(|entry| !entry.is_anti_item())
        .filter_map(|entry| ArchiveEntry::from_parts(&parts_of(entry), entry.is_directory(), entry.size()))
        .collect())
}

fn extract_entry(entry: &sevenz_rust2::ArchiveEntry, data: &mut dyn Read, output: &mut Output<'_>) -> Result<()> {
    output.check_cancel()?;
    if entry.is_anti_item() {
        return Ok(());
    }
    let parts = parts_of(entry);
    if parts.is_empty() {
        return Ok(());
    }
    if entry.is_directory() {
        return output.create_dir(&parts);
    }
    let modified = entry
        .has_last_modified_date
        .then(|| filetime_to_system(u64::from(entry.last_modified_date())))
        .flatten();
    output.write_file(&parts, data, modified)
}

pub(crate) fn extract(archive: &Path, output: &mut Output<'_>) -> Result<()> {
    let mut reader = open(archive)?;
    let files = &reader.archive().files;
    if (0..files.len()).any(|index| reader.archive().is_file_encrypted(index)) {
        return Err(Error::archive(archive, ArchiveFailure::PasswordRequired));
    }
    output.set_total(
        files
            .iter()
            .filter(|entry| !entry.is_directory() && !entry.is_anti_item())
            .map(|entry| entry.size())
            .sum(),
    );
    let mut stopped = None;
    let decoded = reader.for_each_entries_with_decision(|entry, data| {
        Ok(match extract_entry(entry, data, output) {
            Ok(()) => EntryDecision::Continue,
            Err(error) => {
                stopped = Some(error);
                EntryDecision::Stop
            }
        })
    });
    if let Some(error) = stopped {
        return Err(error);
    }
    decoded.map_err(|error| failure(archive, error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_failures_from_seven_z_errors() {
        let archive = Path::new("a.7z");
        let of = |error: SevenZError| failure(archive, error).archive_failure();
        assert_eq!(of(SevenZError::PasswordRequired), Some(ArchiveFailure::PasswordRequired));
        assert_eq!(
            of(SevenZError::UnsupportedCompressionMethod(EncoderMethod::AES256_SHA256.name().to_owned())),
            Some(ArchiveFailure::PasswordRequired)
        );
        assert_eq!(
            of(SevenZError::UnsupportedCompressionMethod(EncoderMethod::ZSTD.name().to_owned())),
            Some(ArchiveFailure::UnsupportedMethod)
        );
        assert_eq!(of(SevenZError::ExternalUnsupported), Some(ArchiveFailure::UnsupportedMethod));
        assert_eq!(of(SevenZError::ChecksumVerificationFailed), Some(ArchiveFailure::Corrupt));
        assert_eq!(of(SevenZError::NextHeaderCrcMismatch), Some(ArchiveFailure::Corrupt));
        assert_eq!(of(SevenZError::BadSignature([0; 6])), Some(ArchiveFailure::Corrupt));
        assert_eq!(
            of(SevenZError::Io(std::io::Error::from(std::io::ErrorKind::UnexpectedEof), "".into())),
            Some(ArchiveFailure::Corrupt)
        );
        assert_eq!(
            of(SevenZError::Io(std::io::Error::other(SevenZError::ChecksumVerificationFailed), "".into())),
            Some(ArchiveFailure::Corrupt)
        );
        assert!(matches!(
            failure(
                archive,
                SevenZError::FileOpen(std::io::Error::from(std::io::ErrorKind::NotFound), "a.7z".to_owned())
            ),
            Error::Io { .. }
        ));
    }
}
