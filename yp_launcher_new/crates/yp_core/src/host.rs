use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const SIMULATE_OS_VAR: &str = "YP_SIMULATE_OS";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Windows,
    Linux,
    MacOs,
}

impl Os {
    pub const fn native() -> Self {
        if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOs
        } else {
            Os::Linux
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "windows" | "win" => Some(Os::Windows),
            "linux" => Some(Os::Linux),
            "macos" | "mac" | "darwin" => Some(Os::MacOs),
            _ => None,
        }
    }

    pub const fn exe_suffix(self) -> &'static str {
        match self {
            Os::Windows => ".exe",
            Os::Linux | Os::MacOs => "",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostEnv {
    os: Os,
    ui_os: Os,
    vars: BTreeMap<String, String>,
    home: Option<PathBuf>,
    exe_path: PathBuf,
    temp_dir: PathBuf,
    system_language: Option<String>,
    windows_build: Option<u32>,
}

impl HostEnv {
    pub fn current() -> Self {
        let os = Os::native();
        let vars = collect_vars(os, std::env::vars_os().map(|(k, v)| {
            (k.to_string_lossy().into_owned(), v.to_string_lossy().into_owned())
        }));
        let ui_os = simulated_os(&vars, os);
        let system_language = detect_system_language(os, &vars);
        Self {
            os,
            ui_os,
            home: dirs::home_dir(),
            exe_path: std::env::current_exe().unwrap_or_default(),
            temp_dir: std::env::temp_dir(),
            system_language,
            windows_build: detect_windows_build(os),
            vars,
        }
    }

    pub fn for_test(os: Os, vars: &[(&str, &str)], root: &Path) -> Self {
        let vars = collect_vars(
            os,
            vars.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
        );
        let ui_os = simulated_os(&vars, os);
        let system_language = language_from_vars(&vars);
        let home = vars
            .get(&normalize_key(os, if os == Os::Windows { "USERPROFILE" } else { "HOME" }))
            .map(PathBuf::from)
            .or_else(|| Some(root.join("home")));
        Self {
            os,
            ui_os,
            home,
            exe_path: root.join("app").join(format!("yp_launcher{}", os.exe_suffix())),
            temp_dir: root.join("tmp"),
            system_language,
            windows_build: None,
            vars,
        }
    }

    pub fn with_var(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(normalize_key(self.os, name), value.to_owned());
        self.ui_os = simulated_os(&self.vars, self.os);
        self
    }

    pub fn with_windows_build(mut self, build: Option<u32>) -> Self {
        self.windows_build = build;
        self
    }

    pub fn with_system_language(mut self, language: Option<&str>) -> Self {
        self.system_language = language.map(str::to_owned);
        self
    }

    pub fn os(&self) -> Os {
        self.os
    }

    pub fn ui_os(&self) -> Os {
        self.ui_os
    }

    pub fn ui_matches_host(&self) -> bool {
        self.os == self.ui_os
    }

    pub fn var(&self, name: &str) -> Option<&str> {
        self.vars
            .get(&normalize_key(self.os, name))
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    pub fn vars(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn home(&self) -> Option<PathBuf> {
        self.home.clone()
    }

    pub fn exe_path(&self) -> &Path {
        &self.exe_path
    }

    pub fn exe_dir(&self) -> &Path {
        self.exe_path.parent().unwrap_or(Path::new("."))
    }

    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    pub fn runs_game_through_wine(&self) -> bool {
        self.os != Os::Windows
    }

    pub fn system_language(&self) -> Option<String> {
        self.system_language.clone()
    }

    pub fn windows_build(&self) -> Option<u32> {
        self.windows_build
    }
}

fn normalize_key(os: Os, name: &str) -> String {
    match os {
        Os::Windows => name.to_ascii_uppercase(),
        Os::Linux | Os::MacOs => name.to_owned(),
    }
}

fn collect_vars(os: Os, vars: impl Iterator<Item = (String, String)>) -> BTreeMap<String, String> {
    vars.map(|(k, v)| (normalize_key(os, &k), v)).collect()
}

fn simulated_os(vars: &BTreeMap<String, String>, os: Os) -> Os {
    vars.get(&normalize_key(os, SIMULATE_OS_VAR))
        .and_then(|value| Os::from_name(value))
        .unwrap_or(os)
}

pub fn language_code(locale: &str) -> Option<String> {
    let code: String = locale
        .chars()
        .take_while(|c| !matches!(c, '_' | '-' | '.' | '@'))
        .collect::<String>()
        .to_ascii_lowercase();
    if code.is_empty() || code == "c" || code == "posix" {
        None
    } else {
        Some(code)
    }
}

fn language_from_vars(vars: &BTreeMap<String, String>) -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|key| vars.get(*key).or_else(|| vars.get(&key.to_ascii_uppercase())))
        .find_map(|value| language_code(value))
}

#[cfg(windows)]
fn detect_system_language(_os: Os, vars: &BTreeMap<String, String>) -> Option<String> {
    windows_registry::CURRENT_USER
        .open(r"Control Panel\International")
        .and_then(|key| key.get_string("LocaleName"))
        .ok()
        .and_then(|name| language_code(&name))
        .or_else(|| language_from_vars(vars))
}

#[cfg(target_os = "macos")]
fn detect_system_language(_os: Os, vars: &BTreeMap<String, String>) -> Option<String> {
    language_from_vars(vars).or_else(|| {
        std::process::Command::new("defaults")
            .args(["read", "-g", "AppleLocale"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| language_code(String::from_utf8_lossy(&output.stdout).trim()))
    })
}

#[cfg(not(any(windows, target_os = "macos")))]
fn detect_system_language(_os: Os, vars: &BTreeMap<String, String>) -> Option<String> {
    language_from_vars(vars)
}

#[cfg(windows)]
fn detect_windows_build(_os: Os) -> Option<u32> {
    windows_registry::LOCAL_MACHINE
        .open(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .and_then(|key| key.get_string("CurrentBuildNumber"))
        .ok()
        .and_then(|build| build.trim().parse().ok())
}

#[cfg(not(windows))]
fn detect_windows_build(_os: Os) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_code_strips_region_and_encoding() {
        assert_eq!(language_code("de_DE.UTF-8").as_deref(), Some("de"));
        assert_eq!(language_code("zh-Hans-CN").as_deref(), Some("zh"));
        assert_eq!(language_code("C.UTF-8"), None);
        assert_eq!(language_code("POSIX"), None);
        assert_eq!(language_code(""), None);
    }

    #[test]
    fn windows_vars_are_case_insensitive() {
        let root = Path::new("/r");
        let host = HostEnv::for_test(Os::Windows, &[("AppData", "C:\\x")], root);
        assert_eq!(host.var("APPDATA"), Some("C:\\x"));
        assert_eq!(host.var("appdata"), Some("C:\\x"));
    }

    #[test]
    fn simulated_os_only_changes_ui_os() {
        let host = HostEnv::for_test(Os::Windows, &[(SIMULATE_OS_VAR, "linux")], Path::new("/r"));
        assert_eq!(host.os(), Os::Windows);
        assert_eq!(host.ui_os(), Os::Linux);
        assert!(!host.ui_matches_host());
    }

    #[test]
    fn empty_vars_read_as_missing() {
        let host = HostEnv::for_test(Os::Linux, &[("XDG_DATA_HOME", "")], Path::new("/r"));
        assert_eq!(host.var("XDG_DATA_HOME"), None);
    }
}
