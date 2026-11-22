use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::{IoOp, IoResultExt, Result};

pub const LAUNCHER_LOG_NAME: &str = "launcher.log";
const ROTATED_LOG_NAME: &str = "launcher.old.log";
const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;

struct FileLogger {
    level: log::LevelFilter,
    path: PathBuf,
    file: Mutex<File>,
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} {:<5} {}: {}\n",
            jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%S%.3fZ"),
            record.level(),
            record.target(),
            record.args()
        );
        if cfg!(debug_assertions) {
            eprint!("{line}");
        }
        let mut file = self.file.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = file.write_all(line.as_bytes());
        if file.metadata().map(|meta| meta.len() > MAX_LOG_BYTES).unwrap_or(false)
            && let Ok(rotated) = rotate(&self.path)
        {
            *file = rotated;
        }
    }

    fn flush(&self) {
        let mut file = self.file.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = file.flush();
    }
}

fn open_append(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

fn rotate(path: &Path) -> std::io::Result<File> {
    let rotated = path.with_file_name(ROTATED_LOG_NAME);
    let _ = std::fs::remove_file(&rotated);
    std::fs::rename(path, &rotated)?;
    open_append(path)
}

pub fn init_logging(logs_dir: &Path, level: log::LevelFilter) -> Result<()> {
    std::fs::create_dir_all(logs_dir).io(IoOp::Create, logs_dir)?;
    let path = logs_dir.join(LAUNCHER_LOG_NAME);
    let mut file = open_append(&path).io(IoOp::Write, &path)?;
    if file.metadata().map(|meta| meta.len() > MAX_LOG_BYTES).unwrap_or(false) {
        file = rotate(&path).io(IoOp::Rename, &path)?;
    }
    let logger = FileLogger {
        level,
        path,
        file: Mutex::new(file),
    };
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(level);
    }
    Ok(())
}
