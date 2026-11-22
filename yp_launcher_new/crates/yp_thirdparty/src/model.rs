use std::path::PathBuf;

use crate::flags::LoadingFlag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ThirdPartyRuntime {
    ReShade,
    Migoto,
    GameMods,
    GraphicMods,
}

impl ThirdPartyRuntime {
    pub const ALL: [ThirdPartyRuntime; 4] = [
        ThirdPartyRuntime::ReShade,
        ThirdPartyRuntime::Migoto,
        ThirdPartyRuntime::GameMods,
        ThirdPartyRuntime::GraphicMods,
    ];

    pub const fn loading_flag(self) -> LoadingFlag {
        match self {
            ThirdPartyRuntime::ReShade => LoadingFlag::ReShade,
            ThirdPartyRuntime::Migoto => LoadingFlag::Migoto,
            ThirdPartyRuntime::GameMods => LoadingFlag::GameMods,
            ThirdPartyRuntime::GraphicMods => LoadingFlag::GraphicMods,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThirdPartyKind {
    ReShadeWholeInstall,
    ReShadePreset,
    Migoto,
    GameMod,
    GraphicMod,
    LodMod,
    Textures,
    GameData,
    SpecialK,
    SkShaderEdit,
    BrokenLegacy,
    MultiBundle,
    Unknown,
}

impl ThirdPartyKind {
    pub const fn is_installable(self) -> bool {
        matches!(
            self,
            ThirdPartyKind::ReShadeWholeInstall
                | ThirdPartyKind::ReShadePreset
                | ThirdPartyKind::Migoto
                | ThirdPartyKind::GameMod
                | ThirdPartyKind::GraphicMod
        )
    }

    pub const fn drop_route(self) -> DropRoute {
        match self {
            ThirdPartyKind::ReShadeWholeInstall
            | ThirdPartyKind::ReShadePreset
            | ThirdPartyKind::Migoto
            | ThirdPartyKind::GameMod
            | ThirdPartyKind::GraphicMod => DropRoute::Install,
            ThirdPartyKind::GameData => DropRoute::ModsTab,
            ThirdPartyKind::Textures => DropRoute::TexturesTab,
            ThirdPartyKind::LodMod
            | ThirdPartyKind::SpecialK
            | ThirdPartyKind::SkShaderEdit
            | ThirdPartyKind::BrokenLegacy
            | ThirdPartyKind::MultiBundle
            | ThirdPartyKind::Unknown => DropRoute::Unsupported,
        }
    }

    pub const fn belongs_to_other_tab(self) -> bool {
        matches!(
            self,
            ThirdPartyKind::LodMod | ThirdPartyKind::GameData | ThirdPartyKind::Textures | ThirdPartyKind::MultiBundle
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropRoute {
    Install,
    ModsTab,
    TexturesTab,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimePresence {
    Installed,
    FoundInGameRoot,
    NotInstalled,
}

impl RuntimePresence {
    pub const fn of(installed: bool, found_in_game_root: bool) -> Self {
        if installed {
            RuntimePresence::Installed
        } else if found_in_game_root {
            RuntimePresence::FoundInGameRoot
        } else {
            RuntimePresence::NotInstalled
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    pub kind: ThirdPartyKind,
    pub source_root: PathBuf,
    pub source_name: Option<String>,
    pub inner_archives: Vec<PathBuf>,
    pub lodmod_ini: Option<PathBuf>,
    pub has_reshade_dll: bool,
    pub is_addon_build: bool,
    pub has_shaders: bool,
    pub preset_count: usize,
    pub game_mod_dlls: Vec<PathBuf>,
    pub graphic_mod_dlls: Vec<PathBuf>,
}

impl Classification {
    pub fn new(kind: ThirdPartyKind, source_root: PathBuf) -> Self {
        Self {
            kind,
            source_root,
            source_name: None,
            inner_archives: Vec::new(),
            lodmod_ini: None,
            has_reshade_dll: false,
            is_addon_build: false,
            has_shaders: false,
            preset_count: 0,
            game_mod_dlls: Vec::new(),
            graphic_mod_dlls: Vec::new(),
        }
    }

    pub fn with_source_name(mut self, name: Option<String>) -> Self {
        self.source_name = name;
        self
    }

    pub fn named(self, name: impl Into<String>) -> Self {
        self.with_source_name(Some(name.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameModSettings {
    pub fake_module_handle: bool,
    pub fake_exe_path: bool,
    pub size_of_image: u64,
    pub disabled: bool,
    pub report_patches: bool,
    pub wait_for_patch_phase: bool,
    pub patch_phase_timeout_ms: u64,
}

impl GameModSettings {
    pub const STEAM_2021_SIZE_OF_IMAGE: u64 = 26_177_536;
    pub const STEAM_2026_SIZE_OF_IMAGE: u64 = 25_944_064;
    pub const DEFAULT_PATCH_TIMEOUT_MS: u64 = 10_000;

    pub fn from_toml(table: &toml::Table) -> Self {
        let defaults = Self::default();
        let flag = |key: &str, fallback: bool| table.get(key).and_then(toml::Value::as_bool).unwrap_or(fallback);
        let non_negative = |key: &str, fallback: u64| {
            table
                .get(key)
                .and_then(toml::Value::as_integer)
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(fallback)
        };
        Self {
            fake_module_handle: flag("fake_module_handle", defaults.fake_module_handle),
            fake_exe_path: flag("fake_exe_path", defaults.fake_exe_path),
            size_of_image: non_negative("size_of_image", defaults.size_of_image),
            disabled: flag("disabled", defaults.disabled),
            report_patches: flag("report_patches", defaults.report_patches),
            wait_for_patch_phase: flag("wait_for_patch_phase", defaults.wait_for_patch_phase),
            patch_phase_timeout_ms: non_negative("patch_phase_timeout_ms", defaults.patch_phase_timeout_ms),
        }
    }

    pub fn toml_values(&self) -> [(&'static str, toml_edit::Value); 7] {
        let int = |value: u64| toml_edit::Value::from(i64::try_from(value).unwrap_or(i64::MAX));
        [
            ("fake_module_handle", toml_edit::Value::from(self.fake_module_handle)),
            ("fake_exe_path", toml_edit::Value::from(self.fake_exe_path)),
            ("size_of_image", int(self.size_of_image)),
            ("disabled", toml_edit::Value::from(self.disabled)),
            ("report_patches", toml_edit::Value::from(self.report_patches)),
            ("wait_for_patch_phase", toml_edit::Value::from(self.wait_for_patch_phase)),
            ("patch_phase_timeout_ms", int(self.patch_phase_timeout_ms)),
        ]
    }

    pub fn with_disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }

    pub fn with_fake_exe_path(self, fake_exe_path: bool) -> Self {
        Self {
            fake_exe_path,
            wait_for_patch_phase: self.wait_for_patch_phase && fake_exe_path,
            ..self
        }
    }

    pub fn with_wait_for_patch_phase(self, wait_for_patch_phase: bool) -> Self {
        Self {
            wait_for_patch_phase,
            fake_exe_path: self.fake_exe_path || wait_for_patch_phase,
            ..self
        }
    }
}

impl Default for GameModSettings {
    fn default() -> Self {
        Self {
            fake_module_handle: true,
            fake_exe_path: true,
            size_of_image: Self::STEAM_2021_SIZE_OF_IMAGE,
            disabled: false,
            report_patches: false,
            wait_for_patch_phase: false,
            patch_phase_timeout_ms: Self::DEFAULT_PATCH_TIMEOUT_MS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameModEntry {
    pub file_name: String,
    pub mod_dir: String,
    pub size_bytes: Option<u64>,
    pub disabled: bool,
    pub settings: Option<GameModSettings>,
}

impl GameModEntry {
    pub fn key(&self) -> String {
        if self.mod_dir.is_empty() {
            self.file_name.clone()
        } else {
            format!("{}/{}", self.mod_dir, self.file_name)
        }
    }

    pub fn shows_mod_dir(&self) -> bool {
        if self.mod_dir.is_empty() {
            return false;
        }
        let file = self.file_name.to_lowercase();
        let stem = file.strip_suffix(".dll").unwrap_or(&file);
        self.mod_dir.to_lowercase() != stem
    }

    pub fn label(&self) -> String {
        if self.shows_mod_dir() {
            format!("{} / {}", self.mod_dir, self.file_name)
        } else {
            self.file_name.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameModsInfo {
    pub mods: Vec<GameModEntry>,
    pub has_config: bool,
}

impl GameModsInfo {
    pub fn enabled_count(&self) -> usize {
        self.mods.iter().filter(|entry| !entry.disabled).count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReShadeInfo {
    pub presets: Vec<String>,
    pub shader_repos: Vec<String>,
    pub shader_count: usize,
    pub addons: Vec<String>,
    pub is_addon_build: bool,
    pub version: Option<String>,
    pub dll_name: Option<String>,
    pub d3d_compiler_missing: bool,
    pub config: ReShadeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReShadeConfig {
    pub performance_mode: bool,
    pub skip_loading_check: bool,
    pub active_preset: Option<String>,
    pub overlay_key: String,
    pub effect_toggle_key: String,
    pub screenshot_key: String,
    pub screenshot_path: Option<String>,
    pub screenshot_format: i64,
    pub show_fps: bool,
    pub show_clock: bool,
}

impl Default for ReShadeConfig {
    fn default() -> Self {
        Self {
            performance_mode: false,
            skip_loading_check: false,
            active_preset: None,
            overlay_key: String::new(),
            effect_toggle_key: String::new(),
            screenshot_key: String::new(),
            screenshot_path: None,
            screenshot_format: 1,
            show_fps: false,
            show_clock: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MigotoInfo {
    pub files: Vec<String>,
    pub shader_fix_count: usize,
    pub shader_fix_names: Vec<String>,
    pub loader_target: Option<String>,
    pub loader_target_ok: bool,
    pub config: MigotoConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigotoHunting {
    #[default]
    Off,
    On,
    NoMarking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigotoMarking {
    #[default]
    Skip,
    Original,
    Pink,
    Mono,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigotoConfig {
    pub hunting: MigotoHunting,
    pub marking_mode: MigotoMarking,
    pub verbose_overlay: bool,
    pub cache_shaders: bool,
    pub check_foreground_window: bool,
    pub reload_fixes_key: String,
    pub wipe_cache_key: String,
    pub toggle_hunt_key: String,
}

impl Default for MigotoConfig {
    fn default() -> Self {
        Self {
            hunting: MigotoHunting::Off,
            marking_mode: MigotoMarking::Skip,
            verbose_overlay: false,
            cache_shaders: true,
            check_foreground_window: false,
            reload_fixes_key: String::new(),
            wipe_cache_key: String::new(),
            toggle_hunt_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeStatus {
    pub installed: bool,
    pub enabled: bool,
    pub shaders_missing: bool,
    pub preset_count: usize,
    pub has_shader_fixes: bool,
    pub reshade_info: Option<ReShadeInfo>,
    pub migoto_info: Option<MigotoInfo>,
    pub game_mods_info: Option<GameModsInfo>,
    pub graphic_mods_info: Option<GameModsInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DllSummary {
    pub file_name: String,
    pub version: Option<String>,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThirdPartyUpdateInfo {
    pub runtime: ThirdPartyRuntime,
    pub installed: DllSummary,
    pub incoming: DllSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallReport {
    pub runtime: ThirdPartyRuntime,
    pub shaders_missing: bool,
    pub preset_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallFailure {
    NoDll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed(InstallReport),
    Rejected(InstallFailure),
}

impl InstallOutcome {
    pub fn installed(runtime: ThirdPartyRuntime) -> Self {
        InstallOutcome::Installed(InstallReport {
            runtime,
            shaders_missing: false,
            preset_count: 0,
        })
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, InstallOutcome::Installed(_))
    }

    pub fn report(&self) -> Option<&InstallReport> {
        match self {
            InstallOutcome::Installed(report) => Some(report),
            InstallOutcome::Rejected(_) => None,
        }
    }

    pub fn failure(&self) -> Option<InstallFailure> {
        match self {
            InstallOutcome::Installed(_) => None,
            InstallOutcome::Rejected(failure) => Some(*failure),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_for_the_patch_phase_needs_the_fake_exe_path() {
        let plain = GameModSettings {
            fake_exe_path: false,
            ..GameModSettings::default()
        };
        let waiting = plain.with_wait_for_patch_phase(true);
        assert!(waiting.wait_for_patch_phase);
        assert!(waiting.fake_exe_path);

        let without_path = waiting.with_fake_exe_path(false);
        assert!(!without_path.fake_exe_path);
        assert!(!without_path.wait_for_patch_phase);

        let stopped = waiting.with_wait_for_patch_phase(false);
        assert!(!stopped.wait_for_patch_phase);
        assert!(stopped.fake_exe_path);
        assert!(!plain.with_fake_exe_path(true).wait_for_patch_phase);
    }

    #[test]
    fn presence_prefers_the_installed_copy() {
        assert_eq!(RuntimePresence::of(true, true), RuntimePresence::Installed);
        assert_eq!(RuntimePresence::of(false, true), RuntimePresence::FoundInGameRoot);
        assert_eq!(RuntimePresence::of(false, false), RuntimePresence::NotInstalled);
    }
}
