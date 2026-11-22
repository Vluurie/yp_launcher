pub mod compat_tool;
pub mod crossover;
pub mod native_steam;
pub mod prefix_steam;
pub mod proton;
pub mod runtime;

use std::path::{Path, PathBuf};

use yp_core::wine_paths::{self, posix, SteamContext};
use yp_core::{HostEnv, NierInstallation, Os};

pub const WINE_COMMAND_VAR: &str = "YP_WINE_COMMAND";
pub const BOTTLE_ROOTS_VAR: &str = "YP_BOTTLE_ROOTS";
pub const PROTON_PATH_VAR: &str = "YP_PROTON_PATH";
pub const EXTRA_COMPAT_TOOLS_VAR: &str = "STEAM_EXTRA_COMPAT_TOOLS_PATHS";
pub const NIER_DIR_NAME: &str = "NieRAutomata";

pub const SYSTEM_COMPAT_TOOL_DIRS: [&str; 2] = [
    "/usr/share/steam/compatibilitytools.d",
    "/usr/local/share/steam/compatibilitytools.d",
];

pub const CROSSOVER_WINE_CANDIDATES: [&str; 8] = [
    "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine64",
    "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine",
    "/Applications/CrossOver Preview.app/Contents/SharedSupport/CrossOver/bin/wine64",
    "/Applications/CrossOver Preview.app/Contents/SharedSupport/CrossOver/bin/wine",
    "/opt/cxoffice/bin/wine64",
    "/opt/cxoffice/bin/wine",
    "/opt/crossover/bin/wine64",
    "/opt/crossover/bin/wine",
];

#[derive(Debug, Clone)]
pub struct WineContext {
    host: HostEnv,
    steam_roots: Option<Vec<String>>,
    bottle_roots: Option<Vec<String>>,
    wine_command: Option<String>,
    system_compat_tool_dirs: Vec<String>,
    crossover_wine_candidates: Vec<String>,
    search_path: bool,
}

impl WineContext {
    pub fn new(host: HostEnv) -> Self {
        let wine_command = host.var(WINE_COMMAND_VAR).map(str::to_owned);
        let bottle_roots = host
            .var(BOTTLE_ROOTS_VAR)
            .map(|value| value.split(':').map(str::to_owned).collect());
        Self {
            host,
            steam_roots: None,
            bottle_roots,
            wine_command,
            system_compat_tool_dirs: SYSTEM_COMPAT_TOOL_DIRS.iter().map(|dir| (*dir).to_owned()).collect(),
            crossover_wine_candidates: CROSSOVER_WINE_CANDIDATES.iter().map(|path| (*path).to_owned()).collect(),
            search_path: true,
        }
    }

    pub fn isolated(host: HostEnv) -> Self {
        Self {
            system_compat_tool_dirs: Vec::new(),
            crossover_wine_candidates: Vec::new(),
            search_path: false,
            steam_roots: Some(Vec::new()),
            bottle_roots: Some(Vec::new()),
            ..Self::new(host)
        }
    }

    pub fn with_steam_roots(mut self, roots: Vec<String>) -> Self {
        self.steam_roots = Some(roots);
        self
    }

    pub fn with_default_steam_roots(mut self) -> Self {
        self.steam_roots = None;
        self
    }

    pub fn with_bottle_roots(mut self, roots: Vec<String>) -> Self {
        self.bottle_roots = Some(roots);
        self
    }

    pub fn with_wine_command(mut self, command: Option<String>) -> Self {
        self.wine_command = command.filter(|command| !command.is_empty());
        self
    }

    pub fn with_system_compat_tool_dirs(mut self, dirs: Vec<String>) -> Self {
        self.system_compat_tool_dirs = dirs;
        self
    }

    pub fn with_crossover_wine_candidates(mut self, candidates: Vec<String>) -> Self {
        self.crossover_wine_candidates = candidates;
        self
    }

    pub fn with_path_search(mut self, enabled: bool) -> Self {
        self.search_path = enabled;
        self
    }

    pub fn host(&self) -> &HostEnv {
        &self.host
    }

    pub fn os(&self) -> Os {
        self.host.os()
    }

    pub fn var(&self, name: &str) -> Option<&str> {
        self.host.var(name)
    }

    pub fn home(&self) -> String {
        wine_paths::host_home(&self.host)
    }

    pub fn steam_roots(&self) -> Vec<String> {
        match &self.steam_roots {
            Some(roots) => roots.clone(),
            None => wine_paths::steam_root_candidates(&self.host),
        }
    }

    pub fn configured_bottle_roots(&self) -> Option<&[String]> {
        self.bottle_roots.as_deref()
    }

    pub fn wine_command(&self) -> Option<&str> {
        self.wine_command.as_deref()
    }

    pub fn system_compat_tool_dirs(&self) -> &[String] {
        &self.system_compat_tool_dirs
    }

    pub fn crossover_wine_candidates(&self) -> &[String] {
        &self.crossover_wine_candidates
    }

    pub fn infer_steam_context(&self, game_path: &str) -> Option<SteamContext> {
        wine_paths::infer_steam_context(game_path, &self.steam_roots())
    }

    pub fn find_on_path(&self, names: &[&str]) -> Option<String> {
        if !self.search_path {
            return None;
        }
        yp_core::process::find_on_path(&self.host, names).map(|path| wine_paths::to_posix(&path))
    }
}

pub(crate) fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

pub(crate) fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub(crate) fn subdir_names(path: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort_by_cached_key(|name| (name.to_lowercase(), name.clone()));
    names
}

pub(crate) fn installation(dir: &str) -> NierInstallation {
    let normalized = posix::normalize(dir);
    NierInstallation {
        has_data: is_dir(&posix::join(&normalized, "data")),
        path: PathBuf::from(normalized),
    }
}

pub(crate) fn push_unique(list: &mut Vec<String>, value: String) {
    if !list.contains(&value) {
        list.push(value);
    }
}
