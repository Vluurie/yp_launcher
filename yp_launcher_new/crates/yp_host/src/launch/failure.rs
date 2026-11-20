use std::path::PathBuf;

use yp_core::LaunchUnavailable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamsReport {
    pub code: Option<i32>,
    pub panic: bool,
    pub headline: String,
    pub phase: Option<String>,
    pub os_error: Option<String>,
    pub cause: Option<String>,
    pub fix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureKind {
    FilesMissing { missing: Vec<&'static str>, launcher_dir: PathBuf },
    LauncherDirNotWritable { dir: PathBuf },
    ExeNotFound { game_dir: PathBuf, exe: PathBuf },
    GameDirNotWritable { game_dir: PathBuf, nams_dir: PathBuf },
    Unavailable(LaunchUnavailable),
    Reported(NamsReport),
    Exited { code: i32 },
    Internal { detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchFailure {
    pub kind: FailureKind,
    pub raw_output: String,
    pub captured_log: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureCategory {
    Panic,
    Unknown,
    NamsFailure,
    InstallNotFound,
    FolderCreate,
    RuntimePrep,
    HostFailure,
    SteamNotRunning,
    SteamNotOwned,
    SteamCheckFailed,
    InvalidArgs,
    ExitedUnexpectedly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureHint {
    ShareReport,
    RepickDirectory,
    VerifyFiles,
    WritableFolder,
    ProgramFiles,
    AntivirusExclusions,
    WritableCache,
    Reboot,
    PersistShare,
    StartSteam,
    SignInOwner,
    RestartSteam,
    PanicShare,
    PanicReboot,
    CheckLogViewer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintLine {
    FromNams(String),
    Known(FailureHint),
}

impl FailureCategory {
    pub fn for_code(code: i32) -> Self {
        match code {
            1 => FailureCategory::NamsFailure,
            2 => FailureCategory::InstallNotFound,
            3 => FailureCategory::FolderCreate,
            4 => FailureCategory::RuntimePrep,
            5..=9 => FailureCategory::HostFailure,
            20 => FailureCategory::SteamNotRunning,
            21 => FailureCategory::SteamNotOwned,
            22 => FailureCategory::SteamCheckFailed,
            64 => FailureCategory::InvalidArgs,
            _ => FailureCategory::ExitedUnexpectedly,
        }
    }

    pub fn hints(self) -> &'static [FailureHint] {
        use FailureHint::*;
        match self {
            FailureCategory::Panic => &[PanicShare, PanicReboot],
            FailureCategory::Unknown => &[],
            FailureCategory::NamsFailure => &[ShareReport],
            FailureCategory::InstallNotFound => &[RepickDirectory, VerifyFiles],
            FailureCategory::FolderCreate => &[WritableFolder, ProgramFiles],
            FailureCategory::RuntimePrep => &[AntivirusExclusions, WritableCache],
            FailureCategory::HostFailure => &[Reboot, AntivirusExclusions, PersistShare],
            FailureCategory::SteamNotRunning => &[StartSteam],
            FailureCategory::SteamNotOwned => &[SignInOwner],
            FailureCategory::SteamCheckFailed => &[RestartSteam, PersistShare],
            FailureCategory::InvalidArgs => &[PanicShare],
            FailureCategory::ExitedUnexpectedly => &[CheckLogViewer, ShareReport],
        }
    }
}

impl LaunchFailure {
    pub fn new(kind: FailureKind, raw_output: impl Into<String>) -> Self {
        Self {
            kind,
            raw_output: raw_output.into(),
            captured_log: None,
        }
    }

    pub fn code(&self) -> Option<i32> {
        match &self.kind {
            FailureKind::Reported(report) => report.code,
            FailureKind::Exited { code } => Some(*code),
            _ => None,
        }
    }

    pub fn is_panic(&self) -> bool {
        matches!(&self.kind, FailureKind::Reported(report) if report.panic)
    }

    pub fn reported_by_nams(&self) -> bool {
        self.code().is_some()
    }

    pub fn report(&self) -> Option<&NamsReport> {
        match &self.kind {
            FailureKind::Reported(report) => Some(report),
            _ => None,
        }
    }

    pub fn category(&self) -> FailureCategory {
        if self.is_panic() {
            return FailureCategory::Panic;
        }
        self.code().map_or(FailureCategory::Unknown, FailureCategory::for_code)
    }

    pub fn hints(&self) -> Vec<HintLine> {
        let category = self.category();
        let mut lines = Vec::new();
        if !matches!(category, FailureCategory::Panic | FailureCategory::Unknown)
            && let Some(fix) = self.report().and_then(|report| report.fix.clone()).filter(|fix| !fix.is_empty())
        {
            lines.push(HintLine::FromNams(fix));
        }
        lines.extend(category.hints().iter().copied().map(HintLine::Known));
        lines
    }
}

fn header(line: &str, tag: &str) -> Option<(Option<i32>, String)> {
    let rest = line.strip_prefix(tag)?.strip_prefix('[')?;
    let close = rest.find(']')?;
    let digits = &rest[..close];
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let after = &rest[close + 1..];
    if !after.starts_with(char::is_whitespace) {
        return None;
    }
    let headline = after.trim();
    if headline.is_empty() {
        return None;
    }
    Some((digits.parse().ok(), headline.to_owned()))
}

fn labelled(lines: &[&str], label: &str) -> Option<String> {
    lines.iter().find_map(|line| {
        let value = line.trim_start().strip_prefix(label)?.trim();
        (!value.is_empty()).then(|| value.to_owned())
    })
}

pub fn parse_nams_output(combined_output: &str) -> Option<NamsReport> {
    if combined_output.trim().is_empty() {
        return None;
    }
    let lines: Vec<&str> = combined_output.split('\n').map(|line| line.trim_end_matches('\r')).collect();
    let found = lines
        .iter()
        .find_map(|line| header(line, "PANIC").map(|(code, headline)| (code, headline, true)))
        .or_else(|| {
            lines
                .iter()
                .find_map(|line| header(line, "ERROR").map(|(code, headline)| (code, headline, false)))
        })?;
    let (code, headline, panic) = found;
    Some(NamsReport {
        code,
        panic,
        headline,
        phase: labelled(&lines, "Phase:"),
        os_error: labelled(&lines, "OS error:"),
        cause: labelled(&lines, "Likely cause:"),
        fix: labelled(&lines, "Try:"),
    })
}

pub fn parse(combined_output: &str, captured_log: Option<PathBuf>) -> Option<LaunchFailure> {
    let report = parse_nams_output(combined_output)?;
    Some(LaunchFailure {
        kind: FailureKind::Reported(report),
        raw_output: combined_output.to_owned(),
        captured_log,
    })
}
