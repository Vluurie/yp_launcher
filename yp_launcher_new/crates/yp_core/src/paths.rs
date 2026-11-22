use std::path::{Path, PathBuf};

use crate::consts;
use crate::error::{Error, Result};
use crate::host::{HostEnv, Os};

pub const ASSETS_DIR_VAR: &str = "YP_ASSETS_DIR";
pub const APP_ID: &str = "com.vluurie.yplauncher";
pub const WINDOWS_COMPANY_DIR: &str = "YoRHa Protocol";
pub const WINDOWS_PRODUCT_DIR: &str = "YoRHa Protocol Launcher";
pub const SETTINGS_FILE_NAME: &str = "settings.json";
pub const LEGACY_PREFS_FILE_NAME: &str = "shared_preferences.json";
pub const LAUNCH_RECORD_FILE_NAME: &str = "last_launch.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub bundled_bins_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub settings_file: PathBuf,
}

impl AppPaths {
    pub fn detect(host: &HostEnv) -> Result<Self> {
        let assets_dir = locate_assets_dir(host);
        let bundled_bins_dir = assets_dir.join("bins");
        let data_dir = data_dir(host)?;
        let runtime_dir = match host.os() {
            Os::Windows => bundled_bins_dir.clone(),
            Os::Linux => xdg_data_home(host)?.join("yp_launcher").join("bins"),
            Os::MacOs => mac_support_dir(host)?.join("bins"),
        };
        Ok(Self {
            logs_dir: runtime_dir.join(consts::LOGS_DIR_NAME),
            settings_file: data_dir.join(SETTINGS_FILE_NAME),
            data_dir,
            runtime_dir,
            bundled_bins_dir,
            assets_dir,
        })
    }

    pub fn for_test(root: &Path) -> Self {
        let assets_dir = root.join("assets");
        let runtime_dir = root.join("runtime");
        let data_dir = root.join("data");
        Self {
            bundled_bins_dir: assets_dir.join("bins"),
            logs_dir: runtime_dir.join(consts::LOGS_DIR_NAME),
            settings_file: data_dir.join(SETTINGS_FILE_NAME),
            assets_dir,
            runtime_dir,
            data_dir,
        }
    }

    pub fn legacy_prefs_file(&self, host: &HostEnv) -> Option<PathBuf> {
        match host.os() {
            Os::Windows => Some(self.data_dir.join(LEGACY_PREFS_FILE_NAME)),
            Os::Linux => xdg_data_home(host)
                .ok()
                .map(|root| root.join(APP_ID).join(LEGACY_PREFS_FILE_NAME)),
            Os::MacOs => None,
        }
    }

    pub fn nams_exe(&self) -> PathBuf {
        self.runtime_dir.join(consts::NAMS_EXE_NAME)
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.runtime_dir.join(consts::PLUGINS_DIR_NAME)
    }

    pub fn plugins_disabled_dir(&self) -> PathBuf {
        self.runtime_dir.join(consts::PLUGINS_DISABLED_DIR_NAME)
    }

    pub fn yorha_dll(&self) -> PathBuf {
        self.plugins_dir().join(consts::YORHA_DLL_NAME)
    }

    pub fn nams_log(&self) -> PathBuf {
        self.logs_dir.join(consts::NAMS_LOG_NAME)
    }

    pub fn runtime_stamp(&self) -> PathBuf {
        self.runtime_dir.join(".runtime_stamp")
    }

    pub fn launch_record(&self) -> PathBuf {
        self.logs_dir.join(LAUNCH_RECORD_FILE_NAME)
    }

    pub fn launcher_thirdparty_dir(&self) -> PathBuf {
        self.runtime_dir.join(consts::THIRD_PARTY_DIR_NAME)
    }

    pub fn docs_dir(&self) -> PathBuf {
        self.assets_dir.join("docs")
    }

    pub fn fonts_dir(&self) -> PathBuf {
        self.assets_dir.join("fonts")
    }

    pub fn images_dir(&self) -> PathBuf {
        self.assets_dir.join("images")
    }

    pub fn l10n_dir(&self) -> PathBuf {
        self.assets_dir.join("l10n")
    }

    pub fn needs_runtime_extraction(&self) -> bool {
        self.runtime_dir != self.bundled_bins_dir
    }
}

fn locate_assets_dir(host: &HostEnv) -> PathBuf {
    if let Some(dir) = host.var(ASSETS_DIR_VAR) {
        return PathBuf::from(dir);
    }
    let exe_dir = host.exe_dir();
    let mut candidates = vec![exe_dir.join("assets")];
    if host.os() == Os::MacOs {
        candidates.push(exe_dir.join("..").join("Resources").join("assets"));
    }
    candidates.extend(exe_dir.ancestors().skip(1).map(|dir| dir.join("assets")));
    candidates
        .into_iter()
        .find(|dir| dir.join("bins").is_dir())
        .unwrap_or_else(|| exe_dir.join("assets"))
}

fn data_dir(host: &HostEnv) -> Result<PathBuf> {
    match host.os() {
        Os::Windows => host
            .var("APPDATA")
            .map(PathBuf::from)
            .or_else(dirs::config_dir)
            .map(|root| root.join(WINDOWS_COMPANY_DIR).join(WINDOWS_PRODUCT_DIR))
            .ok_or_else(|| Error::invalid("data directory", "APPDATA is not set")),
        Os::Linux => Ok(xdg_data_home(host)?.join(APP_ID)),
        Os::MacOs => mac_support_dir(host),
    }
}

fn xdg_data_home(host: &HostEnv) -> Result<PathBuf> {
    if let Some(dir) = host.var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir));
    }
    host.home()
        .map(|home| home.join(".local").join("share"))
        .ok_or_else(|| Error::invalid("data directory", "HOME is not set"))
}

fn mac_support_dir(host: &HostEnv) -> Result<PathBuf> {
    host.home()
        .map(|home| home.join("Library").join("Application Support").join(APP_ID))
        .ok_or_else(|| Error::invalid("data directory", "HOME is not set"))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GamePaths {
    root: PathBuf,
}

impl GamePaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn exe(&self) -> PathBuf {
        self.root.join(consts::GAME_EXE_NAME)
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    pub fn movie_dir(&self) -> PathBuf {
        self.data_dir().join("movie")
    }

    pub fn nams_dir(&self) -> PathBuf {
        self.root.join("nams")
    }

    pub fn mods_dir(&self) -> PathBuf {
        self.nams_dir().join("mods")
    }

    pub fn nams_toml(&self) -> PathBuf {
        self.nams_dir().join("nams.toml")
    }

    pub fn naiom_toml(&self) -> PathBuf {
        self.nams_dir().join("naiom.toml")
    }

    pub fn lodmod_toml(&self) -> PathBuf {
        self.nams_dir().join("lodmod.toml")
    }

    pub fn texture_injection_toml(&self) -> PathBuf {
        self.nams_dir().join("texture_injection.toml")
    }

    pub fn disabled_mods_toml(&self) -> PathBuf {
        self.nams_dir().join("disabled_mods.toml")
    }

    pub fn default_mods_toml(&self) -> PathBuf {
        self.nams_dir().join("default_mods.toml")
    }

    pub fn mod_load_order_toml(&self) -> PathBuf {
        self.nams_dir().join("mod_load_order.toml")
    }

    pub fn profiles_toml(&self) -> PathBuf {
        self.nams_dir().join("profiles.toml")
    }

    pub fn mod_names_json(&self) -> PathBuf {
        self.nams_dir().join("mod_names.json")
    }

    pub fn internal_cache_dir(&self) -> PathBuf {
        self.nams_dir().join("_internal").join("cache")
    }

    pub fn inject_textures_dir(&self) -> PathBuf {
        self.nams_dir().join("inject").join("textures")
    }

    pub fn cutscenes_dir(&self) -> PathBuf {
        self.nams_dir().join("cutscenes")
    }

    pub fn thirdparty_dir(&self) -> PathBuf {
        self.nams_dir().join(consts::THIRD_PARTY_DIR_NAME)
    }

    pub fn scoped_temp_root(&self) -> PathBuf {
        self.nams_dir().join(".tmp")
    }

    pub fn wax_mods_dir(&self) -> PathBuf {
        self.root.join("wax").join("mods")
    }

    pub fn special_k_textures_dir(&self) -> PathBuf {
        self.root.join("SK_Res").join("inject").join("textures")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_runtime_is_bundled_bins() {
        let root = tempfile::tempdir().unwrap();
        let assets = root.path().join("assets");
        std::fs::create_dir_all(assets.join("bins")).unwrap();
        let host = HostEnv::for_test(
            Os::Windows,
            &[("APPDATA", "C:\\Users\\u\\AppData\\Roaming"), (ASSETS_DIR_VAR, assets.to_str().unwrap())],
            root.path(),
        );
        let paths = AppPaths::detect(&host).unwrap();
        assert_eq!(paths.runtime_dir, assets.join("bins"));
        assert!(!paths.needs_runtime_extraction());
        assert!(paths.settings_file.ends_with("YoRHa Protocol Launcher/settings.json")
            || paths.settings_file.ends_with("YoRHa Protocol Launcher\\settings.json"));
    }

    #[test]
    fn linux_runtime_lives_under_xdg_data_home() {
        let root = tempfile::tempdir().unwrap();
        let host = HostEnv::for_test(Os::Linux, &[("XDG_DATA_HOME", "/data"), ("HOME", "/home/u")], root.path());
        let paths = AppPaths::detect(&host).unwrap();
        assert_eq!(paths.runtime_dir, PathBuf::from("/data").join("yp_launcher").join("bins"));
        assert!(paths.needs_runtime_extraction());
        assert_eq!(
            paths.legacy_prefs_file(&host),
            Some(PathBuf::from("/data").join(APP_ID).join(LEGACY_PREFS_FILE_NAME))
        );
    }

    #[test]
    fn game_paths_layout() {
        let game = GamePaths::new("G");
        assert_eq!(game.mods_dir(), PathBuf::from("G").join("nams").join("mods"));
        assert_eq!(game.nams_toml(), PathBuf::from("G").join("nams").join("nams.toml"));
        assert_eq!(game.exe(), PathBuf::from("G").join("NieRAutomata.exe"));
    }
}
