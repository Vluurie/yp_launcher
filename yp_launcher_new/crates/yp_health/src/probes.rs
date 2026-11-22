use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

use indexmap::IndexMap;
use jiff::Timestamp;
use serde_json::Value;
use yp_core::hash::{sha1_file, to_hex};
use yp_core::text::strip_ansi;
use yp_core::{HostEnv, Os, ProcessRunner, ProcessSpec};

pub const RISKY_FILESYSTEMS: [&str; 6] = ["ntfs", "ntfs3", "fuseblk", "exfat", "vfat", "msdos"];
pub const TAIL_MAX_LINES: usize = 40;
pub const TAIL_MAX_BYTES: u64 = 64 * 1024;
pub const HEAD_MAX_LINES: usize = 60;
pub const HEAD_MAX_BYTES: u64 = 64 * 1024;
pub const SHORT_SHA1_CHARS: usize = 12;
pub const WINDOWS_11_FIRST_BUILD: u32 = 22_000;
pub const WINDOWS_10_FIRST_BUILD: u32 = 10_240;

pub const DISTRO_KEY: &str = "Distro";
pub const SESSION_KEY: &str = "Session";
pub const DESKTOP_KEY: &str = "Desktop";
pub const APPIMAGE_KEY: &str = "AppImage";
pub const FLATPAK_KEY: &str = "Flatpak";
pub const STEAM_DECK_KEY: &str = "Steam Deck";

const QUERY_TIMEOUT: Duration = Duration::from_secs(5);
const UNKNOWN: &str = "?";
const STEAM_WINDOWS_PROCESS: &str = "steam.exe";
const FINDMNT_COLUMNS: &str = "TARGET,FSTYPE,OPTIONS";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFacts {
    pub name: String,
    pub size: u64,
    pub modified: Option<Timestamp>,
    pub short_sha1: Option<String>,
}

impl FileFacts {
    fn unreadable(name: String) -> Self {
        Self {
            name,
            size: 0,
            modified: None,
            short_sha1: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextExcerpt {
    pub name: String,
    pub size: u64,
    pub modified: Option<Timestamp>,
    pub lines: Vec<String>,
}

impl TextExcerpt {
    fn unreadable(name: String) -> Self {
        Self {
            name,
            size: 0,
            modified: None,
            lines: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountFacts {
    pub target: String,
    pub fs_type: String,
    pub options: String,
}

impl MountFacts {
    pub fn risky(&self) -> bool {
        RISKY_FILESYSTEMS.contains(&self.fs_type.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemFiles {
    pub os_release: PathBuf,
    pub flatpak_info: PathBuf,
    pub kernel_dir: PathBuf,
}

impl Default for SystemFiles {
    fn default() -> Self {
        Self {
            os_release: PathBuf::from("/etc/os-release"),
            flatpak_info: PathBuf::from("/.flatpak-info"),
            kernel_dir: PathBuf::from("/proc/sys/kernel"),
        }
    }
}

impl SystemFiles {
    pub fn under(root: &Path) -> Self {
        Self {
            os_release: root.join("etc").join("os-release"),
            flatpak_info: root.join(".flatpak-info"),
            kernel_dir: root.join("proc").join("sys").join("kernel"),
        }
    }
}

pub fn parse_findmnt(json: &str) -> Option<MountFacts> {
    let decoded: Value = serde_json::from_str(json).ok()?;
    let first = decoded
        .as_object()?
        .get("filesystems")?
        .as_array()?
        .first()?
        .as_object()?;
    let target = first.get("target")?.as_str()?;
    let fs_type = first.get("fstype")?.as_str()?;
    let options = first
        .get("options")
        .and_then(Value::as_str)
        .unwrap_or_default();
    Some(MountFacts {
        target: target.to_owned(),
        fs_type: fs_type.to_lowercase(),
        options: options.to_owned(),
    })
}

pub fn mount_of(host: &HostEnv, runner: &dyn ProcessRunner, path: &Path) -> Option<MountFacts> {
    if host.os() != Os::Linux || path.as_os_str().is_empty() {
        return None;
    }
    let spec = ProcessSpec::new("findmnt")
        .args(["-J", "-T"])
        .arg(path.as_os_str())
        .args(["-o", FINDMNT_COLUMNS]);
    let output = runner.output(&spec, Some(QUERY_TIMEOUT)).ok()?;
    if !output.is_success() {
        return None;
    }
    parse_findmnt(&output.stdout)
}

pub fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        match c {
            '\n' => {
                lines.push(&text[start..index]);
                start = index + 1;
            }
            '\r' => {
                lines.push(&text[start..index]);
                start = index + 1;
                if let Some(&(next, '\n')) = chars.peek() {
                    chars.next();
                    start = next + 1;
                }
            }
            _ => {}
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

fn unquote(value: &str) -> &str {
    ['"', '\'']
        .into_iter()
        .find_map(|quote| value.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(value)
}

pub fn parse_key_value_lines(content: &str) -> IndexMap<String, String> {
    let mut out = IndexMap::new();
    for raw in split_lines(content) {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        out.insert(key.trim().to_owned(), unquote(value.trim()).to_owned());
    }
    out
}

fn read_lossy(path: &Path) -> Option<String> {
    std::fs::read(path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

fn yes_no(flag: bool) -> String {
    if flag { "yes" } else { "no" }.to_owned()
}

pub fn linux_context(host: &HostEnv, files: &SystemFiles) -> IndexMap<String, String> {
    let mut out = IndexMap::new();
    if host.os() != Os::Linux {
        return out;
    }
    let release = read_lossy(&files.os_release)
        .map(|content| parse_key_value_lines(&content))
        .unwrap_or_default();
    let distro = release
        .get("PRETTY_NAME")
        .or_else(|| release.get("NAME"))
        .cloned()
        .unwrap_or_else(|| UNKNOWN.to_owned());
    let flatpak = host.var("FLATPAK_ID").is_some() || files.flatpak_info.is_file();
    let var_or_unknown = |name: &str| host.var(name).unwrap_or(UNKNOWN).to_owned();
    out.insert(DISTRO_KEY.to_owned(), distro);
    out.insert(SESSION_KEY.to_owned(), var_or_unknown("XDG_SESSION_TYPE"));
    out.insert(
        DESKTOP_KEY.to_owned(),
        var_or_unknown("XDG_CURRENT_DESKTOP"),
    );
    out.insert(
        APPIMAGE_KEY.to_owned(),
        yes_no(host.var("APPIMAGE").is_some()),
    );
    out.insert(FLATPAK_KEY.to_owned(), yes_no(flatpak));
    out.insert(
        STEAM_DECK_KEY.to_owned(),
        yes_no(host.var("SteamDeck") == Some("1")),
    );
    out
}

fn windows_version_label(build: Option<u32>) -> String {
    match build {
        Some(build) if build >= WINDOWS_11_FIRST_BUILD => format!("Windows 11 (Build {build})"),
        Some(build) if build >= WINDOWS_10_FIRST_BUILD => format!("Windows 10 (Build {build})"),
        Some(build) => format!("Windows (Build {build})"),
        None => "Windows".to_owned(),
    }
}

fn linux_version_label(files: &SystemFiles) -> String {
    let parts: Vec<String> = ["ostype", "osrelease", "version"]
        .iter()
        .filter_map(|name| first_line(&files.kernel_dir.join(name)))
        .collect();
    if parts.is_empty() {
        "Linux".to_owned()
    } else {
        parts.join(" ")
    }
}

fn mac_version_label(runner: &dyn ProcessRunner) -> String {
    let query = |flag: &str| {
        let spec = ProcessSpec::new("sw_vers").arg(flag);
        let output = runner.output(&spec, Some(QUERY_TIMEOUT)).ok()?;
        let value = output.stdout.trim().to_owned();
        (output.is_success() && !value.is_empty()).then_some(value)
    };
    match (query("-productVersion"), query("-buildVersion")) {
        (Some(version), Some(build)) => format!("Version {version} (Build {build})"),
        (Some(version), None) => format!("Version {version}"),
        _ => "macOS".to_owned(),
    }
}

pub fn os_version_label(host: &HostEnv, runner: &dyn ProcessRunner, files: &SystemFiles) -> String {
    match host.os() {
        Os::Windows => windows_version_label(host.windows_build()),
        Os::Linux => linux_version_label(files),
        Os::MacOs => mac_version_label(runner),
    }
}

#[cfg(windows)]
fn windows_process_running(name: &str) -> Option<bool> {
    Some(yp_host::platform::win32::is_process_running(name))
}

#[cfg(not(windows))]
fn windows_process_running(_name: &str) -> Option<bool> {
    None
}

pub fn is_steam_running(host: &HostEnv, runner: &dyn ProcessRunner) -> Option<bool> {
    let args = match host.os() {
        Os::Windows => return windows_process_running(STEAM_WINDOWS_PROCESS),
        Os::Linux => ["-x", "steam"],
        Os::MacOs => ["-f", STEAM_WINDOWS_PROCESS],
    };
    let output = runner
        .output(&ProcessSpec::new("pgrep").args(args), Some(QUERY_TIMEOUT))
        .ok()?;
    if output.timed_out {
        return None;
    }
    Some(output.exit_code == Some(0))
}

fn file_name(path: &Path) -> String {
    yp_core::fs::file_name_lossy(path)
}

fn modified(path: &Path) -> Option<Timestamp> {
    let stamp = std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()?;
    Timestamp::try_from(stamp).ok()
}

fn decode_lines(bytes: &[u8]) -> Vec<String> {
    let text = strip_ansi(&String::from_utf8_lossy(bytes));
    split_lines(&text)
        .into_iter()
        .map(|line| line.trim_end().to_owned())
        .collect()
}

fn read_tail(path: &Path, max_lines: usize, max_bytes: u64) -> std::io::Result<TextExcerpt> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let start = size.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.take(size - start).read_to_end(&mut bytes)?;
    let mut lines = decode_lines(&bytes);
    if size > max_bytes && !lines.is_empty() {
        lines.remove(0);
    }
    lines.retain(|line| !line.is_empty());
    if lines.len() > max_lines {
        lines.drain(..lines.len() - max_lines);
    }
    Ok(TextExcerpt {
        name: file_name(path),
        size,
        modified: modified(path),
        lines,
    })
}

pub fn tail(path: &Path, max_lines: usize, max_bytes: u64) -> Option<TextExcerpt> {
    if !path.is_file() {
        return None;
    }
    Some(
        read_tail(path, max_lines, max_bytes)
            .unwrap_or_else(|_| TextExcerpt::unreadable(file_name(path))),
    )
}

fn read_head(path: &Path, max_lines: usize) -> std::io::Result<TextExcerpt> {
    let file = File::open(path)?;
    let size = file.metadata()?.len();
    let mut bytes = Vec::new();
    file.take(HEAD_MAX_BYTES).read_to_end(&mut bytes)?;
    let mut lines = decode_lines(&bytes);
    lines.truncate(max_lines);
    Ok(TextExcerpt {
        name: file_name(path),
        size,
        modified: modified(path),
        lines,
    })
}

pub fn head(path: &Path, max_lines: usize) -> Option<TextExcerpt> {
    if !path.is_file() {
        return None;
    }
    Some(read_head(path, max_lines).unwrap_or_else(|_| TextExcerpt::unreadable(file_name(path))))
}

fn read_facts(path: &Path, hash: bool) -> Option<FileFacts> {
    let short_sha1 = if hash {
        let mut digest = to_hex(&sha1_file(path).ok()?);
        digest.truncate(SHORT_SHA1_CHARS);
        Some(digest)
    } else {
        None
    };
    Some(FileFacts {
        name: file_name(path),
        size: std::fs::metadata(path).ok()?.len(),
        modified: modified(path),
        short_sha1,
    })
}

pub fn facts(path: &Path, hash: bool) -> Option<FileFacts> {
    if !path.is_file() {
        return None;
    }
    Some(read_facts(path, hash).unwrap_or_else(|| FileFacts::unreadable(file_name(path))))
}

fn plain_files(dir: &Path, keep: &dyn Fn(&str) -> bool) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter(|entry| keep(&entry.file_name().to_string_lossy().to_lowercase()))
        .map(|entry| entry.path())
        .collect()
}

pub fn files_in(dir: &Path, keep: &dyn Fn(&str) -> bool, hash: bool) -> Vec<FileFacts> {
    let mut out: Vec<FileFacts> = plain_files(dir, keep)
        .iter()
        .filter_map(|path| facts(path, hash))
        .collect();
    out.sort_by_cached_key(|fact| (fact.name.to_lowercase(), fact.name.clone()));
    out
}

pub fn newest_first(dir: &Path, keep: &dyn Fn(&str) -> bool) -> Vec<PathBuf> {
    let mut stamped: Vec<(Option<Timestamp>, String, PathBuf)> = plain_files(dir, keep)
        .into_iter()
        .map(|path| (modified(&path), file_name(&path), path))
        .collect();
    stamped.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    stamped.into_iter().map(|(_, _, path)| path).collect()
}

pub fn first_line(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let content = read_lossy(path)?;
    split_lines(&content)
        .into_iter()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_split_on_every_terminator_like_dart() {
        assert_eq!(split_lines("a\nb\r\nc\rd"), vec!["a", "b", "c", "d"]);
        assert_eq!(split_lines("a\n"), vec!["a"]);
        assert_eq!(split_lines("a\n\nb"), vec!["a", "", "b"]);
        assert_eq!(split_lines("\n"), vec![""]);
        assert!(split_lines("").is_empty());
    }

    #[test]
    fn key_value_lines_skip_comments_and_strip_matching_quotes() {
        let parsed = parse_key_value_lines(
            "# comment\nNAME=\"Arch Linux\"\nPRETTY_NAME='Arch'\n=orphan\nnoequals\n  ID = arch \nQUOTE=\"\nMIXED=\"x'\nURL=a=b\n",
        );
        assert_eq!(parsed.get("NAME").map(String::as_str), Some("Arch Linux"));
        assert_eq!(parsed.get("PRETTY_NAME").map(String::as_str), Some("Arch"));
        assert_eq!(parsed.get("ID").map(String::as_str), Some("arch"));
        assert_eq!(parsed.get("QUOTE").map(String::as_str), Some("\""));
        assert_eq!(parsed.get("MIXED").map(String::as_str), Some("\"x'"));
        assert_eq!(parsed.get("URL").map(String::as_str), Some("a=b"));
        assert_eq!(parsed.len(), 6);
    }

    #[test]
    fn findmnt_rejects_shapes_without_a_mounted_filesystem() {
        assert_eq!(parse_findmnt("not json"), None);
        assert_eq!(parse_findmnt("[]"), None);
        assert_eq!(parse_findmnt("{\"filesystems\": []}"), None);
        assert_eq!(
            parse_findmnt("{\"filesystems\": [{\"target\": \"/\"}]}"),
            None
        );
        let lenient = parse_findmnt(
            "{\"filesystems\": [{\"target\": \"/\", \"fstype\": \"NTFS3\", \"options\": 3}]}",
        )
        .unwrap();
        assert_eq!(lenient.fs_type, "ntfs3");
        assert_eq!(lenient.options, "");
        assert!(lenient.risky());
    }

    #[test]
    fn windows_label_names_the_release_from_the_build() {
        assert_eq!(
            windows_version_label(Some(19_045)),
            "Windows 10 (Build 19045)"
        );
        assert_eq!(
            windows_version_label(Some(22_631)),
            "Windows 11 (Build 22631)"
        );
        assert_eq!(windows_version_label(Some(9_600)), "Windows (Build 9600)");
        assert_eq!(windows_version_label(None), "Windows");
    }
}
