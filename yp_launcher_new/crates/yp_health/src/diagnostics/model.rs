use std::path::PathBuf;

use indexmap::IndexMap;
use jiff::Timestamp;
use yp_config::ConfigFile;
use yp_core::{HostEnv, Os};
use yp_cutscenes::{CutsceneDetection, CutsceneMod};
use yp_host::game_probe::{ExeBuildStatus, ExeVariant, SteamAppManifest, build_of_time_date_stamp};
use yp_host::launch::record::LaunchRecord;
use yp_host::nams_cli::NamsTexturePack;
use yp_host::nams_log::LogEntry;
use yp_host::wine::runtime::WineSource;
use yp_mods::InstalledMod;
use yp_thirdparty::ThirdPartyStatuses;

use crate::probes::{FileFacts, MountFacts, TextExcerpt};

pub const REPORT_FORMAT: u32 = 2;
pub const DISCORD_LIMIT: usize = 1900;
pub const VANILLA_DROP_CAP: usize = 200;
pub const RECENT_LOG_ISSUE_CAP: usize = 60;

pub const OS_KEY: &str = "OS";
pub const OS_VERSION_KEY: &str = "OS version";
pub const LOCALE_KEY: &str = "Locale";
pub const GAME_RUNNING_KEY: &str = "Game running";
pub const STEAM_RUNNING_KEY: &str = "Steam running";
pub const PREFER_DEDICATED_GPU_KEY: &str = "Prefer dedicated GPU";
pub const NVIDIA_DRIVER_KEY: &str = "NVIDIA driver";
pub const DRM_RENDER_NODES_KEY: &str = "DRM render nodes";
pub const DEDICATED_GPU_ENV_KEY: &str = "Dedicated GPU env";

pub const LAUNCHER_VERSION_KEY: &str = "Launcher version";
pub const LAUNCHER_DIR_KEY: &str = "Launcher dir";
pub const GAME_DIR_KEY: &str = "Game dir";
pub const GAME_DIR_NOT_SELECTED: &str = "<not selected>";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FindingLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticFinding {
    pub level: FindingLevel,
    pub message: String,
}

impl DiagnosticFinding {
    pub fn new(level: FindingLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
        }
    }

    pub fn tag(&self) -> &'static str {
        match self.level {
            FindingLevel::Error => "[X]",
            FindingLevel::Warning => "[!]",
            FindingLevel::Info => "[i]",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportHost {
    pub os: Os,
    pub user_home: String,
    pub user_name: String,
}

impl ReportHost {
    pub fn of(host: &HostEnv) -> Self {
        Self {
            os: host.os(),
            user_home: host
                .home()
                .map(|home| home.to_string_lossy().into_owned())
                .unwrap_or_default(),
            user_name: host
                .var("USERNAME")
                .or_else(|| host.var("USER"))
                .unwrap_or_default()
                .to_owned(),
        }
    }

    pub fn anonymous(os: Os) -> Self {
        Self {
            os,
            user_home: String::new(),
            user_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DlcSource {
    SteamManifest,
    #[default]
    SizeCheck,
}

impl DlcSource {
    pub fn label(self) -> &'static str {
        match self {
            DlcSource::SteamManifest => "Steam manifest",
            DlcSource::SizeCheck => "size check",
        }
    }
}

pub fn exe_variant_name(variant: ExeVariant) -> &'static str {
    match variant {
        ExeVariant::Missing => "missing",
        ExeVariant::Original => "original",
        ExeVariant::WolfLimitBreak => "wolfLimitBreak",
        ExeVariant::LegacyWindows7 => "legacyWindows7",
        ExeVariant::Unknown => "unknown",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameIdentity {
    pub exe_variant: ExeVariant,
    pub has_dlc: bool,
    pub dlc_source: DlcSource,
    pub exe_size: Option<u64>,
    pub exe_sha1: Option<String>,
    pub exe_time_date_stamp: Option<u32>,
    pub build_status: ExeBuildStatus,
    pub build_label: Option<&'static str>,
    pub stamp_build_label: Option<&'static str>,
    pub steam_manifest: Option<SteamAppManifest>,
    pub build_info: IndexMap<String, String>,
    pub compat_exe_present: bool,
    pub data100_size: Option<u64>,
    pub game_bin: Option<FileFacts>,
    pub game_bin_time_date_stamp: Option<u32>,
}

impl Default for GameIdentity {
    fn default() -> Self {
        Self {
            exe_variant: ExeVariant::Unknown,
            has_dlc: false,
            dlc_source: DlcSource::SizeCheck,
            exe_size: None,
            exe_sha1: None,
            exe_time_date_stamp: None,
            build_status: ExeBuildStatus::Missing,
            build_label: None,
            stamp_build_label: None,
            steam_manifest: None,
            build_info: IndexMap::new(),
            compat_exe_present: false,
            data100_size: None,
            game_bin: None,
            game_bin_time_date_stamp: None,
        }
    }
}

impl GameIdentity {
    pub fn exe_variant_supported(&self) -> bool {
        self.build_status == ExeBuildStatus::Known
    }

    pub fn game_bin_matches_exe(&self) -> bool {
        self.game_bin_time_date_stamp.is_some()
            && self.game_bin_time_date_stamp == self.exe_time_date_stamp
    }

    pub fn game_bin_build_label(&self) -> Option<&'static str> {
        build_of_time_date_stamp(self.game_bin_time_date_stamp).map(|build| build.label)
    }

    pub fn wolf_limit_break_suffix(&self) -> &'static str {
        if self.exe_variant == ExeVariant::WolfLimitBreak {
            " (Wolf Limit Break)"
        } else {
            ""
        }
    }

    pub fn build_display(&self) -> String {
        match self.build_status {
            ExeBuildStatus::Known => self.build_label.unwrap_or("?").to_owned(),
            ExeBuildStatus::Missing => "missing".to_owned(),
            ExeBuildStatus::LegacyWindows7 => "Windows 7/8 legacy build".to_owned(),
            ExeBuildStatus::Modified => format!(
                "changed or modified copy of {}{}",
                self.stamp_build_label.unwrap_or("?"),
                self.wolf_limit_break_suffix()
            ),
            ExeBuildStatus::Newer => "newer than every build NAMS knows".to_owned(),
            ExeBuildStatus::Unknown => "unknown build".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VanillaBucket {
    DataRootLoose,
}

impl VanillaBucket {
    pub fn label(self) -> &'static str {
        match self {
            VanillaBucket::DataRootLoose => "data-root-loose",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanillaDirFinding {
    pub path: String,
    pub size_bytes: u64,
    pub bucket: VanillaBucket,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamsHealth {
    pub missing_files: Vec<&'static str>,
    pub nams_exe_present: bool,
    pub launch_command_preview: Option<String>,
    pub nams_exe: Option<FileFacts>,
    pub plugins: Vec<FileFacts>,
    pub nams_version: Option<String>,
    pub nier_install_error: Option<String>,
    pub steam_running: Option<bool>,
}

pub fn wine_source_name(source: WineSource) -> &'static str {
    match source {
        WineSource::CrossOver => "crossOver",
        WineSource::Proton => "proton",
        WineSource::Wine => "wine",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatLayerReport {
    pub source: WineSource,
    pub label: String,
    pub binary: String,
    pub prefix: Option<String>,
    pub prefix_exists: bool,
    pub tool_version: Option<String>,
    pub prefix_version: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogsReport {
    pub nams_log: Option<TextExcerpt>,
    pub crash_dump_count: usize,
    pub newest_crash_dump: Option<TextExcerpt>,
    pub heartbeat: Option<TextExcerpt>,
    pub captured_launch_logs: Vec<FileFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDelta {
    pub file: ConfigFile,
    pub key: String,
    pub value: String,
    pub default_value: String,
    pub note: Option<&'static str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigReport {
    pub deltas: Vec<ConfigDelta>,
    pub unknown_keys: Vec<ConfigDelta>,
    pub legacy_keys: Vec<ConfigDelta>,
    pub parse_errors: IndexMap<ConfigFile, String>,
    pub missing_files: Vec<ConfigFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilesystemRole {
    GameFolder,
    LauncherFolder,
    WinePrefix,
}

impl FilesystemRole {
    pub fn label(self) -> &'static str {
        match self {
            FilesystemRole::GameFolder => "game folder",
            FilesystemRole::LauncherFolder => "launcher folder",
            FilesystemRole::WinePrefix => "wine prefix",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackSwitch {
    TextureInjectionToml,
    DisabledModsToml,
}

impl PackSwitch {
    pub fn file_name(self) -> &'static str {
        match self {
            PackSwitch::TextureInjectionToml => "texture_injection.toml",
            PackSwitch::DisabledModsToml => "disabled_mods.toml",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticsReport {
    pub generated_at: Timestamp,
    pub host: ReportHost,
    pub game_dir: Option<PathBuf>,
    pub system_info: IndexMap<String, String>,
    pub launcher_info: IndexMap<String, String>,
    pub mods: Vec<InstalledMod>,
    pub hidden_mod_folders: Vec<String>,
    pub disabled_mods_entries: Vec<String>,
    pub active_profile: Option<String>,
    pub default_mods_entries: Vec<String>,
    pub cutscene_mods: Vec<CutsceneMod>,
    pub direct_overrides: Vec<String>,
    pub cutscene_detection: CutsceneDetection,
    pub nams_textures: Vec<String>,
    pub sk_res_textures: Vec<String>,
    pub wax_textures: Vec<String>,
    pub data_dir_contents: IndexMap<String, usize>,
    pub wax_mods_root: Vec<String>,
    pub sk_res_packs: Vec<String>,
    pub reshade_contents: Vec<String>,
    pub game_root_extras: Vec<String>,
    pub config_raw: IndexMap<ConfigFile, String>,
    pub game_identity: GameIdentity,
    pub vanilla_dropped: Vec<VanillaDirFinding>,
    pub third_party: ThirdPartyStatuses,
    pub nams_health: NamsHealth,
    pub texture_packs: Vec<NamsTexturePack>,
    pub texture_packs_available: bool,
    pub texture_packs_note: Option<&'static str>,
    pub recent_log_issues: Vec<LogEntry>,
    pub config: ConfigReport,
    pub game_running: bool,
    pub prefer_dedicated_gpu: bool,
    pub filesystems: IndexMap<FilesystemRole, Option<MountFacts>>,
    pub compat_layer: Option<CompatLayerReport>,
    pub last_launch: Option<LaunchRecord>,
    pub logs: LogsReport,
    pub collection_errors: IndexMap<String, String>,
    pub findings: Vec<DiagnosticFinding>,
}

impl DiagnosticsReport {
    pub fn empty(generated_at: Timestamp, host: ReportHost) -> Self {
        Self {
            generated_at,
            host,
            game_dir: None,
            system_info: IndexMap::new(),
            launcher_info: IndexMap::new(),
            mods: Vec::new(),
            hidden_mod_folders: Vec::new(),
            disabled_mods_entries: Vec::new(),
            active_profile: None,
            default_mods_entries: Vec::new(),
            cutscene_mods: Vec::new(),
            direct_overrides: Vec::new(),
            cutscene_detection: CutsceneDetection::default(),
            nams_textures: Vec::new(),
            sk_res_textures: Vec::new(),
            wax_textures: Vec::new(),
            data_dir_contents: IndexMap::new(),
            wax_mods_root: Vec::new(),
            sk_res_packs: Vec::new(),
            reshade_contents: Vec::new(),
            game_root_extras: Vec::new(),
            config_raw: IndexMap::new(),
            game_identity: GameIdentity::default(),
            vanilla_dropped: Vec::new(),
            third_party: ThirdPartyStatuses::default(),
            nams_health: NamsHealth::default(),
            texture_packs: Vec::new(),
            texture_packs_available: false,
            texture_packs_note: None,
            recent_log_issues: Vec::new(),
            config: ConfigReport::default(),
            game_running: false,
            prefer_dedicated_gpu: false,
            filesystems: IndexMap::new(),
            compat_layer: None,
            last_launch: None,
            logs: LogsReport::default(),
            collection_errors: IndexMap::new(),
            findings: Vec::new(),
        }
    }

    pub fn has_game(&self) -> bool {
        self.game_dir.is_some()
    }

    pub fn config_deltas(&self) -> &[ConfigDelta] {
        &self.config.deltas
    }

    pub fn system_value(&self, key: &str) -> Option<&str> {
        self.system_info.get(key).map(String::as_str)
    }

    pub fn launcher_value(&self, key: &str) -> Option<&str> {
        self.launcher_info.get(key).map(String::as_str)
    }
}
