pub mod panel;

use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use yp_core::text::strip_ansi;
use yp_core::AppPaths;

pub const TAIL_BYTES: u64 = 512 * 1024;
pub const POLL_INTERVAL_MS: u64 = 750;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "ERROR" => Some(LogLevel::Error),
            "WARN" => Some(LogLevel::Warn),
            "INFO" => Some(LogLevel::Info),
            "DEBUG" => Some(LogLevel::Debug),
            "TRACE" => Some(LogLevel::Trace),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

fn digits(text: &str, count: usize) -> Option<&str> {
    let head = text.get(..count)?;
    head.bytes().all(|byte| byte.is_ascii_digit()).then(|| &text[count..])
}

fn literal(text: &str, expected: char) -> Option<&str> {
    text.strip_prefix(expected)
}

fn split_timestamp(line: &str) -> Option<(&str, &str)> {
    let mut rest = digits(line, 4)?;
    rest = digits(literal(rest, '-')?, 2)?;
    rest = digits(literal(rest, '-')?, 2)?;
    rest = digits(literal(rest, 'T')?, 2)?;
    rest = digits(literal(rest, ':')?, 2)?;
    rest = digits(literal(rest, ':')?, 2)?;
    rest = literal(rest, '.')?;
    let fraction = rest.bytes().take_while(u8::is_ascii_digit).count();
    if fraction == 0 {
        return None;
    }
    rest = literal(&rest[fraction..], 'Z')?;
    Some((&line[..line.len() - rest.len()], rest))
}

fn split_word(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_start_matches(char::is_whitespace);
    if trimmed.len() == text.len() {
        return None;
    }
    let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
    Some((&trimmed[..end], &trimmed[end..]))
}

fn split_module(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim_start_matches(char::is_whitespace);
    if trimmed.len() == text.len() {
        return None;
    }
    let token_end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
    let token = &trimmed[..token_end];
    let (module, after_colon) = match token.strip_suffix(':') {
        Some(module) if !module.is_empty() => (module, &trimmed[token_end..]),
        _ => {
            let colon = token.find(':').filter(|&index| index > 0)?;
            (&token[..colon], &trimmed[colon + 1..])
        }
    };
    Some((module, after_colon.trim_start_matches(char::is_whitespace)))
}

pub fn format_timestamp(iso: &str) -> String {
    let Some((_, time)) = iso.split_once('T') else {
        return iso.to_owned();
    };
    let time = time.strip_suffix('Z').unwrap_or(time);
    match time.split_once('.') {
        Some((whole, fraction)) if fraction.len() > 3 => format!("{whole}.{}", &fraction[..3]),
        _ => time.to_owned(),
    }
}

pub fn parse_line(raw: &str) -> Option<LogEntry> {
    let clean = strip_ansi(raw);
    let line = clean.trim_end_matches(['\r', '\n']);
    let (timestamp, rest) = split_timestamp(line)?;
    let (level, rest) = split_word(rest)?;
    let level = LogLevel::parse(level)?;
    let (module, message) =
        split_module(rest).unwrap_or(("", rest.trim_start_matches(char::is_whitespace)));
    Some(LogEntry {
        timestamp: format_timestamp(timestamp),
        level,
        module: module.to_owned(),
        message: message.to_owned(),
    })
}

pub fn parse_text(text: &str) -> Vec<LogEntry> {
    text.split('\n')
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_line)
        .collect()
}

pub fn log_path(paths: &AppPaths) -> PathBuf {
    paths.nams_log()
}

pub fn read_file(path: &Path) -> Vec<LogEntry> {
    match std::fs::read(path) {
        Ok(bytes) => parse_text(&String::from_utf8_lossy(&bytes)),
        Err(_) => Vec::new(),
    }
}

pub fn read(paths: &AppPaths) -> Vec<LogEntry> {
    read_file(&paths.nams_log())
}

pub fn clear_file(path: &Path) -> bool {
    path.is_file() && std::fs::write(path, b"").is_ok()
}

pub fn clear(paths: &AppPaths) -> bool {
    clear_file(&paths.nams_log())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TailUpdate {
    Unchanged,
    Reset,
    Appended(Vec<LogEntry>),
}

#[derive(Debug)]
pub struct LogTail {
    path: PathBuf,
    last_size: u64,
    leftover: Vec<u8>,
    first_read: bool,
}

impl LogTail {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            last_size: 0,
            leftover: Vec::new(),
            first_read: true,
        }
    }

    pub fn for_paths(paths: &AppPaths) -> Self {
        Self::new(paths.nams_log())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn poll(&mut self) -> TailUpdate {
        let size = match std::fs::metadata(&self.path) {
            Ok(meta) if meta.is_file() => meta.len(),
            _ => {
                if self.last_size != 0 || !self.leftover.is_empty() {
                    self.last_size = 0;
                    self.leftover.clear();
                    return TailUpdate::Reset;
                }
                return TailUpdate::Unchanged;
            }
        };
        if size < self.last_size {
            self.last_size = 0;
            self.leftover.clear();
        }
        if size == self.last_size {
            return TailUpdate::Unchanged;
        }
        let mut from = self.last_size;
        let mut started_mid_line = false;
        if self.first_read {
            self.first_read = false;
            if size > TAIL_BYTES {
                from = size - TAIL_BYTES;
                self.leftover.clear();
                started_mid_line = true;
            }
        }
        let Some(bytes) = read_range(&self.path, from, size) else {
            return TailUpdate::Unchanged;
        };
        self.last_size = size;
        let mut text = std::mem::take(&mut self.leftover);
        text.extend_from_slice(&bytes);
        if started_mid_line {
            text = match text.iter().position(|&byte| byte == b'\n') {
                Some(index) => text[index + 1..].to_vec(),
                None => Vec::new(),
            };
        }
        let Some(last_newline) = text.iter().rposition(|&byte| byte == b'\n') else {
            self.leftover = text;
            return TailUpdate::Unchanged;
        };
        self.leftover = text[last_newline + 1..].to_vec();
        let entries = parse_text(&String::from_utf8_lossy(&text[..last_newline]));
        if entries.is_empty() {
            TailUpdate::Unchanged
        } else {
            TailUpdate::Appended(entries)
        }
    }
}

fn read_range(path: &Path, from: u64, to: u64) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    file.seek(SeekFrom::Start(from)).ok()?;
    let mut bytes = Vec::with_capacity(usize::try_from(to - from).unwrap_or(0));
    file.take(to - from).read_to_end(&mut bytes).ok()?;
    Some(bytes)
}
