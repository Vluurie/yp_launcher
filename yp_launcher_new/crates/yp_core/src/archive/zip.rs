use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::{Duration, SystemTime};

use encoding_rs::Encoding;
use zip::result::ZipError;
use zip::{ExtraField, System, ZipArchive};

use super::names::{Backslash, decode_legacy, entry_parts};
use super::output::{Output, filetime_to_system, read_failure};
use super::ArchiveEntry;
use crate::error::{ArchiveFailure, Error, IoOp, IoResultExt, Result};

const LAST_DOS_SECOND: u8 = 59;

type Reader = ZipArchive<BufReader<File>>;

pub(crate) fn archive_failure(error: &ZipError) -> Option<ArchiveFailure> {
    Some(match error {
        ZipError::Io(_) => return None,
        ZipError::UnsupportedArchive(ZipError::PASSWORD_REQUIRED) | ZipError::InvalidPassword => {
            ArchiveFailure::PasswordRequired
        }
        ZipError::UnsupportedArchive(_) | ZipError::CompressionMethodNotSupported(_) => {
            ArchiveFailure::UnsupportedMethod
        }
        _ => ArchiveFailure::Corrupt,
    })
}

fn failure(archive: &Path, error: ZipError) -> Error {
    match error {
        ZipError::Io(source) => read_failure(archive, source),
        other => Error::archive(archive, archive_failure(&other).unwrap_or(ArchiveFailure::Corrupt)),
    }
}

fn open(archive: &Path) -> Result<Reader> {
    let file = File::open(archive).io(IoOp::Read, archive)?;
    ZipArchive::new(BufReader::new(file)).map_err(|error| failure(archive, error))
}

struct Named {
    parts: Vec<String>,
    is_dir: bool,
    size: u64,
    encrypted: bool,
}

fn decode_name(raw: &[u8], cp437: impl FnOnce() -> Option<String>, oem: Option<&'static Encoding>) -> String {
    if let Ok(text) = std::str::from_utf8(raw) {
        return text.to_owned();
    }
    decode_legacy(raw, oem)
        .or_else(cp437)
        .unwrap_or_else(|| String::from_utf8_lossy(raw).into_owned())
}

fn backslash_for(system: System) -> Backslash {
    if system == System::Unix {
        Backslash::NameCharacter
    } else {
        Backslash::Separator
    }
}

fn named(reader: &Reader, index: usize, archive: &Path, oem: Option<&'static Encoding>) -> Result<Named> {
    let entry = reader.by_index_data(index).map_err(|error| failure(archive, error))?;
    let name = decode_name(entry.name_raw(), || entry.name().ok().map(|name| name.into_owned()), oem);
    let is_dir = entry.is_dir();
    Ok(Named {
        parts: entry_parts(&name, is_dir, backslash_for(entry.system())),
        is_dir,
        size: entry.size(),
        encrypted: entry.encrypted(),
    })
}

fn all_named(reader: &Reader, archive: &Path, oem: Option<&'static Encoding>) -> Result<Vec<Named>> {
    (0..reader.len()).map(|index| named(reader, index, archive, oem)).collect()
}

pub(crate) fn list(archive: &Path, oem: Option<&'static Encoding>) -> Result<Vec<ArchiveEntry>> {
    let reader = open(archive)?;
    Ok(all_named(&reader, archive, oem)?
        .iter()
        .filter_map(|entry| ArchiveEntry::from_parts(&entry.parts, entry.is_dir, entry.size))
        .collect())
}

fn dos_local_time(stamp: zip::DateTime) -> Option<SystemTime> {
    let civil = jiff::civil::DateTime::new(
        i16::try_from(stamp.year()).ok()?,
        i8::try_from(stamp.month()).ok()?,
        i8::try_from(stamp.day()).ok()?,
        i8::try_from(stamp.hour()).ok()?,
        i8::try_from(stamp.minute()).ok()?,
        i8::try_from(stamp.second().min(LAST_DOS_SECOND)).ok()?,
        0,
    )
    .ok()?;

    let zoned = civil
        .to_zoned_datetime(jiff::tz::TimeZone::system())
        .ok()?;

    SystemTime::try_from(zoned.timestamp()).ok()
}

fn unix_seconds(seconds: u32) -> Option<SystemTime> {
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(u64::from(seconds)))
}

fn modified<'a>(fields: impl Iterator<Item = &'a ExtraField>, dos: Option<zip::DateTime>) -> Option<SystemTime> {
    let mut ntfs = None;
    let mut unix = None;
    for field in fields {
        match field {
            ExtraField::Ntfs(times) => ntfs = ntfs.or_else(|| filetime_to_system(times.mtime())),
            ExtraField::ExtendedTimestamp(stamp) => unix = unix.or_else(|| stamp.mod_time().and_then(unix_seconds)),
            _ => {}
        }
    }
    ntfs.or(unix).or_else(|| dos.and_then(dos_local_time))
}

pub(crate) fn extract(archive: &Path, oem: Option<&'static Encoding>, output: &mut Output<'_>) -> Result<()> {
    let mut reader = open(archive)?;
    let entries = all_named(&reader, archive, oem)?;
    if entries.iter().any(|entry| entry.encrypted) {
        return Err(Error::archive(archive, ArchiveFailure::PasswordRequired));
    }
    output.set_total(entries.iter().filter(|entry| !entry.is_dir).map(|entry| entry.size).sum());
    for (index, entry) in entries.iter().enumerate() {
        output.check_cancel()?;
        if entry.parts.is_empty() {
            continue;
        }
        if entry.is_dir {
            output.create_dir(&entry.parts)?;
            continue;
        }
        let mut file = reader.by_index(index).map_err(|error| failure(archive, error))?;
        let stamp = modified(file.extra_data_fields(), file.last_modified());
        output.write_file(&entry.parts, &mut file, stamp)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_prefer_utf8_then_the_code_page_then_cp437() {
        let shift_jis = super::super::names::oem_encoding(932);
        let cp437 = || Some("cp437".to_owned());
        assert_eq!(decode_name("\u{dc}bung.txt".as_bytes(), cp437, shift_jis), "\u{dc}bung.txt");
        assert_eq!(
            decode_name(b"\x83\x65\x83\x58\x83\x67.txt", cp437, shift_jis),
            "\u{30c6}\u{30b9}\u{30c8}.txt"
        );
        assert_eq!(decode_name(b"\x9abung.txt", cp437, None), "cp437");
        assert_eq!(decode_name(b"bad\x83", cp437, shift_jis), "cp437");
        assert_eq!(decode_name(b"bad\x83", || None, None), "bad\u{fffd}");
    }

    #[test]
    fn only_unix_made_archives_keep_a_backslash_inside_a_name() {
        assert_eq!(backslash_for(System::Unix), Backslash::NameCharacter);
        assert_eq!(backslash_for(System::Dos), Backslash::Separator);
        assert_eq!(backslash_for(System::Unknown), Backslash::Separator);
    }

    #[test]
    fn typed_failures_from_zip_errors() {
        let archive = Path::new("a.zip");
        let of = |error: ZipError| failure(archive, error).archive_failure();
        assert_eq!(
            of(ZipError::UnsupportedArchive(ZipError::PASSWORD_REQUIRED)),
            Some(ArchiveFailure::PasswordRequired)
        );
        assert_eq!(of(ZipError::InvalidPassword), Some(ArchiveFailure::PasswordRequired));
        assert_eq!(of(ZipError::UnsupportedArchive("multi-disk")), Some(ArchiveFailure::UnsupportedMethod));
        assert_eq!(of(ZipError::CompressionMethodNotSupported(1)), Some(ArchiveFailure::UnsupportedMethod));
        assert_eq!(of(ZipError::InvalidArchive("no EOCD".into())), Some(ArchiveFailure::Corrupt));
        assert_eq!(of(ZipError::FileNotFound), Some(ArchiveFailure::Corrupt));
        assert_eq!(
            of(ZipError::Io(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))),
            Some(ArchiveFailure::Corrupt)
        );
        assert!(matches!(
            failure(archive, ZipError::Io(std::io::Error::from(std::io::ErrorKind::PermissionDenied))),
            Error::Io { .. }
        ));
    }
}
