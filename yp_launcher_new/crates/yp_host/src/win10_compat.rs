use std::collections::BTreeSet;
use std::path::Path;
use std::time::Duration;

use yp_core::{HostEnv, Os, ProcessRunner, ProcessSpec, SettingKey, SettingsStore};

pub const WIN10_SETUP_DONE: SettingKey<BTreeSet<String>> = SettingKey::new("win10setup_done", BTreeSet::new);
pub const SETUP_BATCH_NAME: &str = "win10setup.bat";
pub const WINDOWS_11_FIRST_BUILD: u32 = 22000;
const SETUP_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Win10SetupOutcome {
    NotWindows,
    NotWindows11,
    NoBatchFile,
    AlreadyDone,
    Ran,
}

pub fn normalize_key(game_dir: &str) -> String {
    game_dir
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_lowercase() || c.is_ascii_digit() { c } else { '_' })
        .collect()
}

pub fn is_windows_11_or_newer(host: &HostEnv) -> bool {
    host.windows_build().is_some_and(|build| build >= WINDOWS_11_FIRST_BUILD)
}

pub fn ensure_ran_for_directory(
    host: &HostEnv,
    runner: &dyn ProcessRunner,
    settings: &SettingsStore,
    game_dir: &Path,
) -> Win10SetupOutcome {
    if host.os() != Os::Windows {
        return Win10SetupOutcome::NotWindows;
    }
    if !is_windows_11_or_newer(host) {
        return Win10SetupOutcome::NotWindows11;
    }
    if !game_dir.join(SETUP_BATCH_NAME).is_file() {
        return Win10SetupOutcome::NoBatchFile;
    }
    let key = normalize_key(&game_dir.to_string_lossy());
    if settings.get(&WIN10_SETUP_DONE).contains(&key) {
        return Win10SetupOutcome::AlreadyDone;
    }
    let spec = ProcessSpec::new("cmd.exe")
        .args(["/c", SETUP_BATCH_NAME])
        .cwd(game_dir)
        .hide_console();
    if let Err(err) = runner.output(&spec, Some(SETUP_TIMEOUT)) {
        log::warn!("win10setup.bat failed: {err}");
    }
    if let Err(err) = settings.update(&WIN10_SETUP_DONE, |done| {
        done.insert(key);
    }) {
        log::warn!("could not remember win10setup.bat: {err}");
    }
    Win10SetupOutcome::Ran
}
