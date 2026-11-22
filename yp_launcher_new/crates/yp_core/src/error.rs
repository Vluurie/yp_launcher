use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IoOp {
    Read,
    Write,
    Create,
    Remove,
    Rename,
    Copy,
    List,
    Metadata,
}

impl fmt::Display for IoOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verb = match self {
            IoOp::Read => "read",
            IoOp::Write => "write",
            IoOp::Create => "create",
            IoOp::Remove => "remove",
            IoOp::Rename => "rename",
            IoOp::Copy => "copy",
            IoOp::List => "list",
            IoOp::Metadata => "stat",
        };
        f.write_str(verb)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessFailure {
    NotFound,
    Spawn(String),
    Exit(i32),
    Signal,
    TimedOut,
}

impl fmt::Display for ProcessFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessFailure::NotFound => f.write_str("program not found"),
            ProcessFailure::Spawn(reason) => write!(f, "failed to start: {reason}"),
            ProcessFailure::Exit(code) => write!(f, "exited with code {code}"),
            ProcessFailure::Signal => f.write_str("terminated by signal"),
            ProcessFailure::TimedOut => f.write_str("timed out"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveFailure {
    UnsupportedFormat,
    UnsupportedMethod,
    PasswordRequired,
    Corrupt,
    MissingVolume,
    DiskFull,
    NoFiles,
}

impl fmt::Display for ArchiveFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            ArchiveFailure::UnsupportedFormat => "not a zip, 7z or rar archive",
            ArchiveFailure::UnsupportedMethod => "unsupported compression method",
            ArchiveFailure::PasswordRequired => "password required",
            ArchiveFailure::Corrupt => "archive is damaged",
            ArchiveFailure::MissingVolume => "a volume of the archive is missing",
            ArchiveFailure::DiskFull => "not enough free disk space",
            ArchiveFailure::NoFiles => "archive holds no files",
        };
        f.write_str(reason)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{op} failed for {}: {source}", path.display())]
    Io {
        op: IoOp,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid TOML{}: {message}", describe(path))]
    TomlParse {
        path: Option<PathBuf>,
        message: String,
    },
    #[error("TOML round trip changed values{}", describe(path))]
    TomlRoundTrip { path: Option<PathBuf> },
    #[error("invalid JSON{}: {source}", describe(path))]
    Json {
        path: Option<PathBuf>,
        #[source]
        source: serde_json::Error,
    },
    #[error("{program}: {failure}")]
    Process {
        program: String,
        failure: ProcessFailure,
    },
    #[error("{failure}: {}", path.display())]
    Archive { path: PathBuf, failure: ArchiveFailure },
    #[error("cancelled")]
    Cancelled,
    #[error("invalid {what}: {detail}")]
    Invalid { what: &'static str, detail: String },
}

fn describe(path: &Option<PathBuf>) -> String {
    match path {
        Some(path) => format!(" in {}", path.display()),
        None => String::new(),
    }
}

impl Error {
    pub fn invalid(what: &'static str, detail: impl Into<String>) -> Self {
        Error::Invalid {
            what,
            detail: detail.into(),
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound)
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(self, Error::Cancelled)
    }

    pub fn archive(path: &Path, failure: ArchiveFailure) -> Self {
        Error::Archive {
            path: path.to_path_buf(),
            failure,
        }
    }

    pub fn archive_failure(&self) -> Option<ArchiveFailure> {
        match self {
            Error::Archive { failure, .. } => Some(*failure),
            _ => None,
        }
    }

    pub fn is_disk_full(&self) -> bool {
        match self {
            Error::Archive { failure, .. } => *failure == ArchiveFailure::DiskFull,
            Error::Io { source, .. } => is_disk_full_kind(source.kind()),
            _ => false,
        }
    }
}

pub fn is_disk_full_kind(kind: std::io::ErrorKind) -> bool {
    matches!(kind, std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded)
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait IoResultExt<T> {
    fn io(self, op: IoOp, path: &Path) -> Result<T>;
}

impl<T> IoResultExt<T> for std::io::Result<T> {
    fn io(self, op: IoOp, path: &Path) -> Result<T> {
        self.map_err(|source| Error::Io {
            op,
            path: path.to_path_buf(),
            source,
        })
    }
}
