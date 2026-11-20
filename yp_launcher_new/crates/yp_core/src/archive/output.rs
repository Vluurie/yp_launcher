use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::error::{ArchiveFailure, Error, IoOp, Result, is_disk_full_kind};
use crate::task::{CancelToken, Progress, ProgressSink};

const COPY_BUFFER: usize = 256 * 1024;
const PROGRESS_STEPS: u64 = 200;
const MIN_PROGRESS_STEP: u64 = 64 * 1024;
const MAX_PROGRESS_STEP: u64 = 8 * 1024 * 1024;
const FILETIME_UNIX_EPOCH: u64 = 116_444_736_000_000_000;
const FILETIME_TICKS_PER_SECOND: u64 = 10_000_000;
const NANOS_PER_FILETIME_TICK: u32 = 100;

pub(crate) fn read_failure(archive: &Path, source: std::io::Error) -> Error {
    let nested = source.get_ref().and_then(|inner| {
        inner
            .downcast_ref::<::zip::result::ZipError>()
            .and_then(super::zip::archive_failure)
            .or_else(|| {
                inner
                    .downcast_ref::<sevenz_rust2::Error>()
                    .and_then(super::sevenz::archive_failure)
            })
    });
    if let Some(failure) = nested {
        return Error::archive(archive, failure);
    }
    match source.kind() {
        ErrorKind::Unsupported => Error::archive(archive, ArchiveFailure::UnsupportedMethod),
        ErrorKind::InvalidData
        | ErrorKind::InvalidInput
        | ErrorKind::UnexpectedEof
        | ErrorKind::WriteZero
        | ErrorKind::Other => Error::archive(archive, ArchiveFailure::Corrupt),
        _ => Error::Io {
            op: IoOp::Read,
            path: archive.to_path_buf(),
            source,
        },
    }
}

pub(crate) fn write_failure(archive: &Path, op: IoOp, path: &Path, source: std::io::Error) -> Error {
    if is_disk_full_kind(source.kind()) {
        return Error::archive(archive, ArchiveFailure::DiskFull);
    }
    Error::Io {
        op,
        path: path.to_path_buf(),
        source,
    }
}

pub(crate) fn filetime_to_system(ticks: u64) -> Option<SystemTime> {
    let since_epoch = ticks.checked_sub(FILETIME_UNIX_EPOCH)?;
    let seconds = since_epoch / FILETIME_TICKS_PER_SECOND;
    let nanos = u32::try_from(since_epoch % FILETIME_TICKS_PER_SECOND).ok()? * NANOS_PER_FILETIME_TICK;
    SystemTime::UNIX_EPOCH.checked_add(Duration::new(seconds, nanos))
}

pub(crate) struct Output<'a> {
    archive: &'a Path,
    dest: &'a Path,
    progress: &'a dyn ProgressSink,
    cancel: &'a CancelToken,
    total: u64,
    done: u64,
    reported: Option<u64>,
    step: u64,
    written: Vec<PathBuf>,
    buffer: Vec<u8>,
}

impl<'a> Output<'a> {
    pub(crate) fn new(
        archive: &'a Path,
        dest: &'a Path,
        progress: &'a dyn ProgressSink,
        cancel: &'a CancelToken,
    ) -> Self {
        Self {
            archive,
            dest,
            progress,
            cancel,
            total: 0,
            done: 0,
            reported: None,
            step: MIN_PROGRESS_STEP,
            written: Vec::new(),
            buffer: vec![0; COPY_BUFFER],
        }
    }

    pub(crate) fn set_total(&mut self, total: u64) {
        self.total = total;
        self.step = (total / PROGRESS_STEPS).clamp(MIN_PROGRESS_STEP, MAX_PROGRESS_STEP);
    }

    pub(crate) fn check_cancel(&self) -> Result<()> {
        self.cancel.check()
    }

    fn path_of(&self, parts: &[String]) -> PathBuf {
        parts.iter().fold(self.dest.to_path_buf(), |path, part| path.join(part))
    }

    fn fraction(&self) -> Option<f32> {
        if self.total == 0 {
            None
        } else {
            Some((self.done as f64 / self.total as f64).min(1.0) as f32)
        }
    }

    fn report(&mut self, current: &str, upcoming: u64) {
        let due = self.reported.is_none_or(|at| self.done.saturating_sub(at) >= self.step);
        if !due && upcoming < self.step {
            return;
        }
        self.reported = Some(self.done);
        self.progress.report(Progress {
            fraction: self.fraction(),
            current: Some(current.to_owned()),
        });
    }

    pub(crate) fn create_dir(&mut self, parts: &[String]) -> Result<()> {
        let path = self.path_of(parts);
        std::fs::create_dir_all(&path).map_err(|source| write_failure(self.archive, IoOp::Create, &path, source))
    }

    pub(crate) fn begin_file(&mut self, parts: &[String], unreported_size: u64) -> Result<PathBuf> {
        self.cancel.check()?;
        let path = self.path_of(parts);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|source| write_failure(self.archive, IoOp::Create, parent, source))?;
        }
        self.report(&parts.join("/"), unreported_size);
        self.cancel.check()?;
        self.written.push(path.clone());
        Ok(path)
    }

    pub(crate) fn advance(&mut self, bytes: u64) {
        self.done = self.done.saturating_add(bytes);
    }

    pub(crate) fn write_file(
        &mut self,
        parts: &[String],
        reader: &mut dyn Read,
        modified: Option<SystemTime>,
    ) -> Result<()> {
        let path = self.begin_file(parts, 0)?;
        let name = parts.join("/");
        let mut file =
            File::create(&path).map_err(|source| write_failure(self.archive, IoOp::Create, &path, source))?;
        loop {
            self.cancel.check()?;
            let read = match reader.read(&mut self.buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(read_failure(self.archive, error)),
            };
            file.write_all(&self.buffer[..read])
                .map_err(|source| write_failure(self.archive, IoOp::Write, &path, source))?;
            self.advance(read as u64);
            self.report(&name, 0);
        }
        if let Some(modified) = modified {
            let _ = file.set_modified(modified);
        }
        Ok(())
    }

    pub(crate) fn finish(self) {
        self.progress.report(Progress::fraction(1.0));
    }

    pub(crate) fn discard(self) {
        for path in self.written.iter().rev() {
            if let Err(error) = std::fs::remove_file(path)
                && error.kind() != ErrorKind::NotFound
            {
                log::warn!("could not remove {} after a failed extraction: {error}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filetimes_convert_to_system_time() {
        assert_eq!(filetime_to_system(FILETIME_UNIX_EPOCH), Some(SystemTime::UNIX_EPOCH));
        assert_eq!(
            filetime_to_system(FILETIME_UNIX_EPOCH + 15_000_000),
            SystemTime::UNIX_EPOCH.checked_add(Duration::from_millis(1500))
        );
        assert_eq!(filetime_to_system(0), None);
    }

    #[test]
    fn decoder_errors_are_corruption_and_os_errors_stay_io() {
        let archive = Path::new("a.zip");
        for kind in [ErrorKind::InvalidData, ErrorKind::UnexpectedEof, ErrorKind::Other, ErrorKind::InvalidInput] {
            let failure = read_failure(archive, std::io::Error::from(kind)).archive_failure();
            assert_eq!(failure, Some(ArchiveFailure::Corrupt), "{kind:?}");
        }
        assert_eq!(
            read_failure(archive, std::io::Error::from(ErrorKind::Unsupported)).archive_failure(),
            Some(ArchiveFailure::UnsupportedMethod)
        );
        assert!(matches!(
            read_failure(archive, std::io::Error::from(ErrorKind::PermissionDenied)),
            Error::Io { op: IoOp::Read, .. }
        ));
        let nested = std::io::Error::other(sevenz_rust2::Error::PasswordRequired);
        assert_eq!(read_failure(archive, nested).archive_failure(), Some(ArchiveFailure::PasswordRequired));
        let nested = std::io::Error::from(::zip::result::ZipError::CompressionMethodNotSupported(98));
        assert_eq!(read_failure(archive, nested).archive_failure(), Some(ArchiveFailure::UnsupportedMethod));
    }

    #[test]
    fn a_full_disk_is_a_typed_failure() {
        let archive = Path::new("a.zip");
        let out = Path::new("out/file.bin");
        let full = write_failure(archive, IoOp::Write, out, std::io::Error::from(ErrorKind::StorageFull));
        assert_eq!(full.archive_failure(), Some(ArchiveFailure::DiskFull));
        assert!(full.is_disk_full());
        let quota = write_failure(archive, IoOp::Write, out, std::io::Error::from(ErrorKind::QuotaExceeded));
        assert!(quota.is_disk_full());
        let denied = write_failure(archive, IoOp::Create, out, std::io::Error::from(ErrorKind::PermissionDenied));
        assert!(matches!(denied, Error::Io { op: IoOp::Create, ref path, .. } if path == out));
        assert!(!denied.is_disk_full());
    }
}
