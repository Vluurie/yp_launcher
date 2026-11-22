use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ignore::{WalkBuilder, WalkState};
use yp_core::consts::GAME_EXE_NAME;
use yp_core::{CancelToken, HostEnv, NierInstallation};

use crate::vdf;

pub const UNINSTALL_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 524220";
pub const INSTALL_LOCATION_VALUE: &str = "InstallLocation";
pub const STEAM_KEY: &str = r"SOFTWARE\Valve\Steam";
pub const STEAM_PATH_VALUE: &str = "SteamPath";
pub const DRIVE_SCAN_TIMEOUT: Duration = Duration::from_secs(30);
pub const SKIPPED_DIR_NAMES: [&str; 6] = [
    "$recycle.bin",
    "windows",
    "system volume information",
    "recovery",
    "programdata",
    "appdata",
];

const NIER_DIR_NAME: &str = "NieRAutomata";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryHints {
    pub install_location: Option<String>,
    pub steam_path: Option<String>,
}

#[cfg(windows)]
pub fn registry_hints() -> RegistryHints {
    let read = |root: &windows_registry::Key, key: &str, value: &str| {
        root.open(key)
            .and_then(|key| key.get_string(value))
            .ok()
            .map(|text| text.trim().to_owned())
            .filter(|text| !text.is_empty())
    };
    RegistryHints {
        install_location: read(windows_registry::LOCAL_MACHINE, UNINSTALL_KEY, INSTALL_LOCATION_VALUE),
        steam_path: read(windows_registry::CURRENT_USER, STEAM_KEY, STEAM_PATH_VALUE),
    }
}

#[cfg(not(windows))]
pub fn registry_hints() -> RegistryHints {
    RegistryHints::default()
}

fn normalized(path: &str) -> PathBuf {
    let trimmed = path.trim_end_matches(['\\', '/']);
    PathBuf::from(if trimmed.ends_with(':') { path } else { trimmed })
}

fn installation_at(dir: PathBuf) -> NierInstallation {
    NierInstallation {
        has_data: dir.join("data").is_dir(),
        path: dir,
    }
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\").to_lowercase()
}

pub fn installs_near_launcher(launcher_dir: &Path) -> Vec<NierInstallation> {
    launcher_dir
        .ancestors()
        .find(|dir| dir.join(GAME_EXE_NAME).is_file())
        .map(|dir| installation_at(dir.to_path_buf()))
        .into_iter()
        .collect()
}

pub fn installs_from_hints(hints: &RegistryHints) -> Vec<NierInstallation> {
    let mut candidates = Vec::new();
    if let Some(location) = &hints.install_location {
        candidates.push(normalized(location));
    }
    if let Some(steam) = &hints.steam_path {
        candidates.extend(steam_candidates(&normalized(steam)));
    }
    installations(candidates)
}

pub fn installs_from_windows(host: &HostEnv, hints: &RegistryHints) -> Vec<NierInstallation> {
    let mut candidates: Vec<_> = installs_near_launcher(host.exe_dir())
        .into_iter()
        .chain(installs_from_hints(hints))
        .map(|install| install.path)
        .collect();
    let mut seen = HashSet::new();
    for variable in ["ProgramFiles(x86)", "ProgramW6432", "ProgramFiles"] {
        if let Some(root) = host.var(variable) {
            let steam = normalized(root).join("Steam");
            if seen.insert(path_key(&steam)) {
                candidates.extend(steam_candidates(&steam));
            }
        }
    }
    installations(candidates)
}

fn steam_candidates(steam: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![steam.to_path_buf()];
    for folder in ["steamapps", "config"] {
        if let Ok(text) = std::fs::read_to_string(steam.join(folder).join("libraryfolders.vdf")) {
            libraries.extend(vdf::path_entries(&text).iter().map(|path| normalized(path)));
        }
    }
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    for library in libraries {
        if !seen.insert(path_key(&library)) {
            continue;
        }
        let steamapps = library.join("steamapps");
        let common = steamapps.join("common");
        let manifest = steamapps.join(format!("appmanifest_{}.acf", crate::game_probe::STEAM_APP_ID));
        if let Ok(text) = std::fs::read_to_string(manifest)
            && let Ok(document) = vdf::parse(&text)
            && let Some(state) = document.block("AppState")
            && let Some(directory) = state.string("installdir")
            && !directory.is_empty()
            && directory != "."
            && directory != ".."
            && !directory.contains(['/', '\\', ':'])
        {
            candidates.push(common.join(directory));
        }
        candidates.push(common.join(NIER_DIR_NAME));
    }
    candidates
}

fn installations(candidates: Vec<PathBuf>) -> Vec<NierInstallation> {
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|dir| dir.join(GAME_EXE_NAME).is_file())
        .filter(|dir| seen.insert(path_key(dir)))
        .map(installation_at)
        .collect()
}

pub fn find_via_registry_and_steam() -> Vec<NierInstallation> {
    installs_from_hints(&registry_hints())
}

pub fn drive_roots() -> Vec<PathBuf> {
    ('C'..='Z')
        .map(|letter| PathBuf::from(format!("{letter}:\\")))
        .filter(|root| root.is_dir())
        .collect()
}

pub fn drive_scan(cancel: &CancelToken) -> Vec<NierInstallation> {
    scan_roots(&drive_roots(), cancel, DRIVE_SCAN_TIMEOUT)
}

pub fn scan_roots(roots: &[PathBuf], cancel: &CancelToken, timeout: Duration) -> Vec<NierInstallation> {
    let Some(root) = roots.first() else {
        return Vec::new();
    };
    if cancel.is_cancelled() || timeout.is_zero() {
        return Vec::new();
    }
    let deadline = Instant::now() + timeout;
    let visited = Mutex::new(HashSet::new());
    let found = Mutex::new(Vec::new());
    let mut builder = WalkBuilder::new(root);
    for root in &roots[1..] {
        builder.add(root);
    }
    builder.standard_filters(false).follow_links(true).threads(2);
    builder.build_parallel().run(|| {
        Box::new(|entry| {
            if cancel.is_cancelled() || Instant::now() >= deadline {
                return WalkState::Quit;
            }
            let Ok(entry) = entry else {
                return WalkState::Continue;
            };
            let Some(kind) = entry.file_type() else {
                return WalkState::Continue;
            };
            let path = entry.path();
            if kind.is_dir() {
                if entry.depth() > 0 && is_skipped(&entry.file_name().to_string_lossy()) {
                    return WalkState::Skip;
                }
                let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
                if !visited.lock().unwrap_or_else(|err| err.into_inner()).insert(key) {
                    return WalkState::Skip;
                }
                if path.join(GAME_EXE_NAME).is_file() {
                    found
                        .lock()
                        .unwrap_or_else(|err| err.into_inner())
                        .push(installation_at(path.to_path_buf()));
                    return WalkState::Skip;
                }
            } else if kind.is_file()
                && entry.file_name().to_string_lossy().eq_ignore_ascii_case(GAME_EXE_NAME)
                && let Some(parent) = path.parent()
            {
                found
                    .lock()
                    .unwrap_or_else(|err| err.into_inner())
                    .push(installation_at(parent.to_path_buf()));
            }
            WalkState::Continue
        })
    });
    let mut found = found.into_inner().unwrap_or_else(|err| err.into_inner());
    found.sort_by(|left, right| left.path.cmp(&right.path));
    found
}

fn is_skipped(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with('.') || SKIPPED_DIR_NAMES.contains(&lower.as_str())
}
