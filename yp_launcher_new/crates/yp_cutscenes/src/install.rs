use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use yp_core::archive::{self, ARCHIVE_EXTENSIONS, Extractor};
use yp_core::{CancelToken, Error, GamePaths, HostEnv, IoOp, IoResultExt, ProgressSink, Result};
use yp_formats::usm;

use crate::movie_dir::{dir_contains_usm, find_movie_dir, usm_files_below};
use crate::observer::Observer;

pub const MAX_MOD_NAME_LENGTH: usize = 27;
const MOVIE_DIR: &str = "movie";
const COPY_CHUNK: usize = 1024 * 1024;
const REPORT_EVERY_BYTES: u64 = 5 * 1024 * 1024;
const USM_EXTENSION: &str = ".usm";
const RESERVED_NAME_CHARACTERS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Archive,
    UsmFile,
    Directory,
    Unsupported,
}

pub fn classify_source(path: &Path) -> SourceKind {
    if archive::is_archive(path) {
        SourceKind::Archive
    } else if path.is_file() {
        if usm::is_usm_path(path) {
            SourceKind::UsmFile
        } else {
            SourceKind::Unsupported
        }
    } else if path.is_dir() {
        SourceKind::Directory
    } else {
        SourceKind::Unsupported
    }
}

pub fn source_contains_usm(path: &Path, extractor: &Extractor) -> Result<bool> {
    Ok(match classify_source(path) {
        SourceKind::Archive => extractor.contains_extension(path, &[USM_EXTENSION])?,
        SourceKind::UsmFile => true,
        SourceKind::Directory => dir_contains_usm(path),
        SourceKind::Unsupported => false,
    })
}

pub fn default_mod_name(source: &Path) -> String {
    let name = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    for extension in ARCHIVE_EXTENSIONS {
        let Some(cut) = name.len().checked_sub(extension.len()) else {
            continue;
        };
        if name.is_char_boundary(cut) && name[cut..].eq_ignore_ascii_case(extension) {
            return name[..cut].to_string();
        }
    }
    name
}

fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

pub fn truncate_name(name: &str) -> String {
    let mut out = String::new();
    let mut units = 0;
    for character in name.chars() {
        units += character.len_utf16();
        if units > MAX_MOD_NAME_LENGTH {
            break;
        }
        out.push(character);
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameRejection {
    Empty,
    TooLong { max: usize },
    Invalid,
}

fn is_valid_folder_name(name: &str) -> bool {
    name != "."
        && name != ".."
        && !name.ends_with('.')
        && !name.chars().any(|character| {
            character.is_control() || RESERVED_NAME_CHARACTERS.contains(&character)
        })
}

pub fn validate_mod_name(raw: &str) -> std::result::Result<String, NameRejection> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(NameRejection::Empty);
    }
    if utf16_len(trimmed) > MAX_MOD_NAME_LENGTH {
        return Err(NameRejection::TooLong {
            max: MAX_MOD_NAME_LENGTH,
        });
    }
    if !is_valid_folder_name(trimmed) {
        return Err(NameRejection::Invalid);
    }
    Ok(trimmed.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameChoice {
    MergeInto(String),
    New(String),
}

impl NameChoice {
    pub fn resolve(&self) -> std::result::Result<String, NameRejection> {
        match self {
            NameChoice::New(raw) => validate_mod_name(raw),
            NameChoice::MergeInto(existing) => {
                let trimmed = existing.trim();
                if trimmed.is_empty() {
                    Err(NameRejection::Empty)
                } else if !is_valid_folder_name(trimmed) {
                    Err(NameRejection::Invalid)
                } else {
                    Ok(trimmed.to_string())
                }
            }
        }
    }
}

pub fn target_dir(cutscenes_dir: &Path, name: &str) -> PathBuf {
    cutscenes_dir.join(name)
}

#[derive(Debug, thiserror::Error)]
pub enum PrepareFailure {
    #[error("the dropped item contains no .usm files")]
    NoUsm,
    #[error("extraction failed (disk full: {disk_full}): {error}")]
    Extract { disk_full: bool, error: Error },
    #[error("no folder with .usm files was found")]
    NoMovieFolder,
    #[error(transparent)]
    Failed(#[from] Error),
}

impl PrepareFailure {
    pub fn is_cancelled(&self) -> bool {
        matches!(
            self,
            PrepareFailure::Failed(Error::Cancelled)
                | PrepareFailure::Extract {
                    error: Error::Cancelled,
                    ..
                }
        )
    }
}

pub fn is_disk_full(error: &Error) -> bool {
    error.is_disk_full()
}

fn extract_failure(error: Error) -> PrepareFailure {
    PrepareFailure::Extract {
        disk_full: error.is_disk_full(),
        error,
    }
}

#[derive(Debug)]
pub struct PreparedSource {
    source: PathBuf,
    movie_dir: PathBuf,
    extracted: Option<tempfile::TempDir>,
}

impl PreparedSource {
    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn movie_dir(&self) -> &Path {
        &self.movie_dir
    }

    pub fn was_extracted(&self) -> bool {
        self.extracted.is_some()
    }

    pub fn default_name(&self) -> String {
        default_mod_name(&self.source)
    }

    pub fn suggested_name(&self) -> String {
        truncate_name(&self.default_name())
    }

    pub fn usm_files(&self) -> Vec<PathBuf> {
        usm_files_below(&self.movie_dir)
    }
}

pub struct InstallContext<'a> {
    pub game: &'a GamePaths,
    pub host: &'a HostEnv,
    pub extractor: &'a Extractor,
}

pub fn prepare(
    source: &Path,
    context: &InstallContext<'_>,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> std::result::Result<PreparedSource, PrepareFailure> {
    let kind = classify_source(source);
    if !source_contains_usm(source, context.extractor).map_err(extract_failure)? {
        return Err(PrepareFailure::NoUsm);
    }
    cancel.check()?;
    let extracted = match kind {
        SourceKind::Archive => Some(
            context
                .extractor
                .extract_to_temp(source, Some(context.game), context.host, progress, cancel)
                .map_err(extract_failure)?,
        ),
        SourceKind::UsmFile | SourceKind::Directory | SourceKind::Unsupported => None,
    };
    let root = extracted.as_ref().map_or(source, |dir| dir.path());
    let movie_dir = find_movie_dir(root).ok_or(PrepareFailure::NoMovieFolder)?;
    Ok(PreparedSource {
        source: source.to_path_buf(),
        movie_dir,
        extracted,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyProgress {
    pub file_index: usize,
    pub file_count: usize,
    pub file_name: String,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

impl CopyProgress {
    pub fn fraction(&self) -> f32 {
        if self.bytes_total == 0 {
            0.0
        } else {
            (self.bytes_done as f64 / self.bytes_total as f64) as f32
        }
    }

    pub fn megabytes_done(&self) -> u64 {
        self.bytes_done / (1024 * 1024)
    }

    pub fn megabytes_total(&self) -> u64 {
        self.bytes_total / (1024 * 1024)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed {
        name: String,
        target: PathBuf,
        files: usize,
    },
    NoUsmFiles,
}

pub fn install(
    prepared: &PreparedSource,
    cutscenes_dir: &Path,
    name: &str,
    progress: &dyn Observer<CopyProgress>,
    cancel: &CancelToken,
) -> Result<InstallOutcome> {
    let files = prepared.usm_files();
    if files.is_empty() {
        return Ok(InstallOutcome::NoUsmFiles);
    }
    let target = target_dir(cutscenes_dir, name);
    let target_movie = target.join(MOVIE_DIR);
    std::fs::create_dir_all(&target_movie).io(IoOp::Create, &target_movie)?;

    let mut bytes_total = 0;
    for file in &files {
        bytes_total += std::fs::metadata(file).io(IoOp::Metadata, file)?.len();
    }
    let mut state = CopyProgress {
        file_index: 0,
        file_count: files.len(),
        file_name: String::new(),
        bytes_done: 0,
        bytes_total,
    };
    for (index, file) in files.iter().enumerate() {
        cancel.check()?;
        let relative = file
            .strip_prefix(prepared.movie_dir())
            .unwrap_or(file.as_path());
        let destination = target_movie.join(relative);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).io(IoOp::Create, parent)?;
        }
        state.file_index = index + 1;
        state.file_name = file
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        progress.observe(&state);
        if let Err(error) = copy_with_progress(file, &destination, &mut state, progress, cancel) {
            let _ = std::fs::remove_file(&destination);
            return Err(error);
        }
    }
    Ok(InstallOutcome::Installed {
        name: name.to_string(),
        target,
        files: files.len(),
    })
}

fn copy_with_progress(
    source: &Path,
    destination: &Path,
    state: &mut CopyProgress,
    progress: &dyn Observer<CopyProgress>,
    cancel: &CancelToken,
) -> Result<()> {
    let mut input = File::open(source).io(IoOp::Read, source)?;
    let mut output = File::create(destination).io(IoOp::Write, destination)?;
    let mut buffer = vec![0u8; COPY_CHUNK];
    let mut last_report = state.bytes_done;
    loop {
        cancel.check()?;
        let read = input.read(&mut buffer).io(IoOp::Read, source)?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .io(IoOp::Write, destination)?;
        state.bytes_done += read as u64;
        if state.bytes_done - last_report > REPORT_EVERY_BYTES {
            last_report = state.bytes_done;
            progress.observe(state);
        }
    }
    output.flush().io(IoOp::Write, destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_names_drop_one_archive_extension() {
        assert_eq!(
            default_mod_name(Path::new("C:/dl/HD Cutscenes.ZIP")),
            "HD Cutscenes"
        );
        assert_eq!(default_mod_name(Path::new("/dl/pack.tar.gz")), "pack.tar.gz");
        assert_eq!(default_mod_name(Path::new("/dl/pack.part1.rar")), "pack.part1");
        assert_eq!(default_mod_name(Path::new("/dl/My Folder")), "My Folder");
        assert_eq!(default_mod_name(Path::new("/dl/folder.7z/")), "folder");
        assert_eq!(default_mod_name(Path::new("/dl/.zip")), "");
        assert_eq!(default_mod_name(Path::new("/dl/notes.zipx")), "notes.zipx");
    }

    #[test]
    fn suggested_names_are_cut_to_the_limit() {
        let long = "A".repeat(40);
        assert_eq!(truncate_name(&long).len(), MAX_MOD_NAME_LENGTH);
        assert_eq!(truncate_name("short"), "short");
        let emoji = format!("{}\u{1F600}", "a".repeat(26));
        assert_eq!(truncate_name(&emoji), "a".repeat(26));
    }

    #[test]
    fn name_validation() {
        assert_eq!(validate_mod_name("  HD pack  "), Ok("HD pack".to_string()));
        assert_eq!(validate_mod_name("   "), Err(NameRejection::Empty));
        assert_eq!(
            validate_mod_name(&"x".repeat(28)),
            Err(NameRejection::TooLong {
                max: MAX_MOD_NAME_LENGTH
            })
        );
        assert_eq!(validate_mod_name(&"x".repeat(27)), Ok("x".repeat(27)));
        for bad in ["..", ".", "a/b", "a\\b", "what?", "trailing.", "tab\there"] {
            assert_eq!(validate_mod_name(bad), Err(NameRejection::Invalid), "{bad}");
        }
        assert_eq!(
            NameChoice::MergeInto("An existing folder name that is long".to_string()).resolve(),
            Ok("An existing folder name that is long".to_string())
        );
        assert_eq!(
            NameChoice::MergeInto("../x".to_string()).resolve(),
            Err(NameRejection::Invalid)
        );
        assert_eq!(
            NameChoice::New(String::new()).resolve(),
            Err(NameRejection::Empty)
        );
    }

    #[test]
    fn disk_full_detection() {
        let archive = |failure| Error::archive(Path::new("pack.7z"), failure);
        assert!(is_disk_full(&archive(yp_core::ArchiveFailure::DiskFull)));
        assert!(!is_disk_full(&archive(yp_core::ArchiveFailure::Corrupt)));
        assert!(!is_disk_full(&archive(yp_core::ArchiveFailure::PasswordRequired)));
        let io = Error::Io {
            op: IoOp::Write,
            path: PathBuf::from("x"),
            source: std::io::ErrorKind::StorageFull.into(),
        };
        assert!(is_disk_full(&io));
        assert!(!is_disk_full(&Error::Cancelled));
    }

    #[test]
    fn copy_progress_math() {
        let progress = CopyProgress {
            file_index: 1,
            file_count: 2,
            file_name: "a.usm".to_string(),
            bytes_done: 3 * 1024 * 1024,
            bytes_total: 12 * 1024 * 1024,
        };
        assert_eq!(progress.fraction(), 0.25);
        assert_eq!(
            (progress.megabytes_done(), progress.megabytes_total()),
            (3, 12)
        );
        let empty = CopyProgress {
            bytes_total: 0,
            ..progress
        };
        assert_eq!(empty.fraction(), 0.0);
    }
}
