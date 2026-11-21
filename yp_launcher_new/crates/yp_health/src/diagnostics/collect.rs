use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use indexmap::IndexMap;
use jiff::Timestamp;
use yp_config::cli;
use yp_core::consts::{APP_VERSION, NAMS_LOG_NAME};
use yp_core::runtime::RuntimeSetup;
use yp_core::wine_paths::{from_posix, posix, to_posix};
use yp_core::{
    AppPaths, CancelToken, GamePaths, HostEnv, Os, PlatformAdapter, ProcessRunner, SettingsStore,
};
use yp_cutscenes::{ScanDirs, ScanResult, Silent};
use yp_host::game_probe;
use yp_host::gpu::{self, GpuProbe};
use yp_host::launch::{nams_dll, CAPTURED_LOG_PREFIX};
use yp_host::nams_cli::{self, HostNamsCli};
use yp_host::nams_log::{LogEntry, LogLevel};
use yp_host::wine::WineContext;
use yp_host::wine::runtime::{self as wine_runtime, WineSource};
use yp_thirdparty::ThirdPartyStatuses;

use super::config::{collect_config, flattened_entries};
use super::model::ConfigReport;
use super::findings::build_findings;
use super::model::{
    CompatLayerReport, DEDICATED_GPU_ENV_KEY, DRM_RENDER_NODES_KEY, DiagnosticsReport, DlcSource,
    FilesystemRole, GAME_DIR_KEY, GAME_DIR_NOT_SELECTED, GAME_RUNNING_KEY, GameIdentity,
    LAUNCHER_DIR_KEY, LAUNCHER_VERSION_KEY, LOCALE_KEY, LogsReport, NVIDIA_DRIVER_KEY, NamsHealth,
    OS_KEY, OS_VERSION_KEY, PREFER_DEDICATED_GPU_KEY, RECENT_LOG_ISSUE_CAP, ReportHost,
    STEAM_RUNNING_KEY,
};
use super::scan::{hidden_mod_folders, scan_extras, scan_textures, scan_vanilla_drops};
use super::text::yes_no;
use crate::error::Result;
use crate::probes::{self, MountFacts, SystemFiles};

pub const NAMS_INFO_TIMEOUT: Duration = Duration::from_secs(10);
pub const NAMS_TEXTURES_TIMEOUT: Duration = Duration::from_secs(20);
pub const SHORT_ERROR_MAX_CHARS: usize = 200;
pub const HEARTBEAT_LOG_NAME: &str = "heartbeat.log";
pub const HEARTBEAT_TAIL_LINES: usize = 20;
pub const CRASH_DUMP_PREFIX: &str = "crashdump_";
pub const CRASH_DUMP_SUFFIX: &str = ".txt";
pub const CAPTURED_LOG_SUFFIX: &str = ".log";
pub const CAPTURED_LOGS_SHOWN: usize = 3;
pub const BUILD_INFO_FILE: &str = "Latest_BuildInfo.properties";
pub const COMPAT_EXE_NAME: &str = "NieRAutomataCompat.exe";
pub const DLC_ARCHIVE_NAME: &str = "data100.cpk";
pub const PROTON_VERSION_FILE: &str = "version";
pub const PROTON_PREFIX_DIR: &str = "pfx";

pub const NOTE_NO_GAME_FOLDER: &str = "no game folder selected";
pub const NOTE_WINDOWS_ONLY: &str = "NAMS texture query runs NAMS.exe natively (Windows only)";
pub const NOTE_QUERY_FAILED: &str = "NAMS texture query failed or timed out";
pub const NOTE_NAMS_MISSING: &str = "NAMS.exe missing";

pub fn short_error(error: &dyn Display) -> String {
    let text = error.to_string();
    let first_line = probes::split_lines(&text)
        .into_iter()
        .next()
        .unwrap_or_default();
    match super::text::truncated(first_line, SHORT_ERROR_MAX_CHARS) {
        Some(head) => format!("{head}..."),
        None => first_line.to_owned(),
    }
}

#[derive(Debug, Default)]
struct Sections {
    errors: IndexMap<String, String>,
}

impl Sections {
    fn guard<T: Default, E: Display>(
        &mut self,
        section: &str,
        result: std::result::Result<T, E>,
    ) -> T {
        match result {
            Ok(value) => value,
            Err(error) => {
                self.errors.insert(section.to_owned(), short_error(&error));
                T::default()
            }
        }
    }
}

pub fn os_name(os: Os) -> &'static str {
    match os {
        Os::Windows => "windows",
        Os::Linux => "linux",
        Os::MacOs => "macos",
    }
}

pub struct Collector {
    host: HostEnv,
    paths: AppPaths,
    settings: Arc<SettingsStore>,
    runner: Arc<dyn ProcessRunner>,
    platform: Arc<dyn PlatformAdapter>,
    wine: WineContext,
    system_files: SystemFiles,
    gpu_probe: Option<GpuProbe>,
    steam_running: Option<Option<bool>>,
}

impl Collector {
    fn collect_settings(&self, game: &GamePaths) -> std::result::Result<ConfigReport, yp_config::ConfigError> {
        let cli = HostNamsCli {
            platform: self.platform.clone(),
            runner: self.runner.clone(),
            paths: self.paths.clone(),
            game: game.clone(),
        };
        let schema = cli::schema(&cli, None)?;
        let values = cli::get(&cli)?;
        let validate = cli::validate(&cli).ok();
        Ok(collect_config(&schema, &values, validate.as_ref()))
    }

    pub fn new(
        host: HostEnv,
        paths: AppPaths,
        settings: Arc<SettingsStore>,
        runner: Arc<dyn ProcessRunner>,
        platform: Arc<dyn PlatformAdapter>,
    ) -> Self {
        let wine = WineContext::new(host.clone());
        Self {
            host,
            paths,
            settings,
            runner,
            platform,
            wine,
            system_files: SystemFiles::default(),
            gpu_probe: None,
            steam_running: None,
        }
    }

    pub fn with_wine_context(mut self, wine: WineContext) -> Self {
        self.wine = wine;
        self
    }

    pub fn with_system_files(mut self, files: SystemFiles) -> Self {
        self.system_files = files;
        self
    }

    pub fn with_gpu_probe(mut self, probe: GpuProbe) -> Self {
        self.gpu_probe = Some(probe);
        self
    }

    pub fn with_steam_running(mut self, running: Option<bool>) -> Self {
        self.steam_running = Some(running);
        self
    }

    pub fn host(&self) -> &HostEnv {
        &self.host
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn collect(
        &self,
        game: Option<&GamePaths>,
        launch_command_preview: Option<String>,
        cancel: &CancelToken,
    ) -> Result<DiagnosticsReport> {
        let game = game.filter(|game| !game.root().as_os_str().is_empty());
        let mut sections = Sections::default();
        let mut report = DiagnosticsReport::empty(Timestamp::now(), ReportHost::of(&self.host));
        report.game_dir = game.map(|game| game.root().to_path_buf());

        if let Some(game) = game {
            report.mods = sections.guard("mods", yp_mods::scan::list_installed(game));
            cancel.check()?;
            report.hidden_mod_folders =
                sections.guard("hidden mod folders", hidden_mod_folders(game));
            report.disabled_mods_entries =
                sections.guard("disabled mods", yp_mods::disabled::list(game));
            let cutscenes: ScanResult = sections.guard(
                "cutscene mods",
                yp_cutscenes::scan(&ScanDirs::for_game(game), &Silent, cancel),
            );
            cancel.check()?;
            report.cutscene_detection = cutscenes.detection();
            report.cutscene_mods = cutscenes.mods;
            report.direct_overrides = cutscenes.direct_overrides;

            let textures = sections.guard("texture folders", scan_textures(game));
            report.nams_textures = textures.nams;
            report.sk_res_textures = textures.special_k;
            report.wax_textures = textures.wax;

            let extras = sections.guard("game folder scan", scan_extras(game));
            cancel.check()?;
            report.data_dir_contents = extras.data_dir_contents;
            report.wax_mods_root = extras.wax_mods_root;
            report.sk_res_packs = extras.sk_res_packs;
            report.reshade_contents = extras.reshade_contents;
            report.game_root_extras = extras.game_root_extras;

            report.config = sections.guard("settings", self.collect_settings(game));
            for file in report.config.parse_errors.keys() {
                let raw = yp_core::fs::read_text_opt(&game.nams_dir().join(file)).map(Option::unwrap_or_default);
                report.config_raw.insert(file.clone(), sections.guard(file, raw));
            }

            report.game_identity = self.collect_game_identity(game);
            cancel.check()?;
            report.vanilla_dropped = sections.guard("vanilla data drops", scan_vanilla_drops(game));
            report.third_party = self.collect_third_party(game);
            report.active_profile = Some(yp_mods::profiles::active_name(game));
            report.default_mods_entries = sections.guard(
                "profiles",
                yp_core::fs::read_text_opt(&game.default_mods_toml())
                    .map(|raw| flattened_entries(&raw.unwrap_or_default())),
            );
            cancel.check()?;
        }

        report.nams_health = self.collect_nams_health(game, launch_command_preview);
        cancel.check()?;
        self.collect_texture_packs(game, &mut report);
        cancel.check()?;

        let nams_logs = nams_dll::logs_dir(&self.paths, &self.settings, game.map(GamePaths::root));
        report.recent_log_issues = self.collect_recent_issues(&nams_logs);
        report.game_running = self.platform.is_game_running();
        report.prefer_dedicated_gpu = gpu::prefer_dedicated(&self.settings);
        report.nams_dll_picked = nams_dll::picked(&self.settings).is_some();
        let gpu_probe = self
            .gpu_probe
            .unwrap_or_else(|| GpuProbe::detect(&self.host));

        if let Some(game) = game
            && self.host.os() != Os::Windows
        {
            report.compat_layer = self.collect_compat_layer(game);
        }
        self.collect_filesystems(game, &mut report);
        cancel.check()?;

        report.last_launch = yp_host::launch::record::read(&self.paths);
        report.logs = self.collect_logs(&nams_logs);

        report.system_info = self.system_info(&report, &gpu_probe);
        report.launcher_info = self.launcher_info(game);
        report.collection_errors = sections.errors;
        report.generated_at = Timestamp::now();
        report.findings = build_findings(&report);
        Ok(report)
    }

    fn collect_game_identity(&self, game: &GamePaths) -> GameIdentity {
        let root = game.root();
        let probe = game_probe::probe_exe(root);
        let manifest = game_probe::read_steam_manifest(root);
        let build_info = std::fs::read(root.join(BUILD_INFO_FILE))
            .map(|bytes| probes::parse_key_value_lines(&String::from_utf8_lossy(&bytes)))
            .unwrap_or_default();
        let game_bin = self.paths.runtime_dir.join("runtime").join("game.bin");
        GameIdentity {
            exe_variant: game_probe::detect_exe_variant(root),
            has_dlc: game_probe::has_dlc(root),
            dlc_source: if manifest.is_some() {
                DlcSource::SteamManifest
            } else {
                DlcSource::SizeCheck
            },
            exe_size: probe.size,
            exe_time_date_stamp: probe.time_date_stamp,
            build_status: probe.status(),
            build_label: probe.build().map(|build| build.label),
            stamp_build_label: probe.stamp_build().map(|build| build.label),
            exe_sha1: probe.sha1,
            steam_manifest: manifest,
            build_info,
            compat_exe_present: root.join(COMPAT_EXE_NAME).is_file(),
            data100_size: std::fs::metadata(game.data_dir().join(DLC_ARCHIVE_NAME))
                .ok()
                .filter(|meta| meta.is_file())
                .map(|meta| meta.len()),
            game_bin: probes::facts(&game_bin, false),
            game_bin_time_date_stamp: game_probe::file_time_date_stamp(&game_bin),
        }
    }

    fn collect_third_party(&self, game: &GamePaths) -> ThirdPartyStatuses {
        yp_thirdparty::status_all(game, &self.host)
    }

    fn collect_nams_health(
        &self,
        game: Option<&GamePaths>,
        launch_command_preview: Option<String>,
    ) -> NamsHealth {
        let nams_exe = probes::facts(&self.paths.nams_exe(), true);
        let plugins = probes::files_in(
            &self.paths.plugins_dir(),
            &|name| name.ends_with(".dll"),
            true,
        );
        let info = match game {
            Some(game) if self.host.os() == Os::Windows && nams_exe.is_some() => nams_cli::info(
                self.runner.as_ref(),
                &self.paths,
                game.root(),
                Some(NAMS_INFO_TIMEOUT),
            ),
            _ => None,
        };
        let (nams_version, nier_install_error) = info
            .map(|info| (info.version, info.nier_install_error))
            .unwrap_or_default();
        NamsHealth {
            missing_files: RuntimeSetup::new(&self.paths, &self.host).missing_files(),
            nams_exe_present: nams_exe.is_some(),
            launch_command_preview,
            nams_exe,
            plugins,
            nams_version,
            nier_install_error,
            steam_running: self
                .steam_running
                .unwrap_or_else(|| probes::is_steam_running(&self.host, self.runner.as_ref())),
        }
    }

    fn collect_texture_packs(&self, game: Option<&GamePaths>, report: &mut DiagnosticsReport) {
        let Some(game) = game else {
            report.texture_packs_note = Some(NOTE_NO_GAME_FOLDER);
            return;
        };
        if self.host.os() != Os::Windows {
            report.texture_packs_note = Some(NOTE_WINDOWS_ONLY);
            return;
        }
        let listed = nams_cli::textures_list(
            self.runner.as_ref(),
            &self.paths,
            game.root(),
            Some(NAMS_TEXTURES_TIMEOUT),
        );
        match listed {
            Some(packs) => {
                report.texture_packs = packs;
                report.texture_packs_available = true;
            }
            None => {
                report.texture_packs_note = Some(if report.nams_health.nams_exe_present {
                    NOTE_QUERY_FAILED
                } else {
                    NOTE_NAMS_MISSING
                });
            }
        }
    }

    fn collect_recent_issues(&self, nams_logs: &Path) -> Vec<LogEntry> {
        let mut issues: Vec<LogEntry> = yp_host::nams_log::read_file(&nams_logs.join(NAMS_LOG_NAME))
            .into_iter()
            .filter(|entry| matches!(entry.level, LogLevel::Warn | LogLevel::Error))
            .collect();
        if issues.len() > RECENT_LOG_ISSUE_CAP {
            issues.drain(..issues.len() - RECENT_LOG_ISSUE_CAP);
        }
        issues
    }

    fn collect_compat_layer(&self, game: &GamePaths) -> Option<CompatLayerReport> {
        let game_exe = to_posix(&game.exe());
        let runtime = wine_runtime::detect(&self.wine, Some(&game_exe))?;
        let version_beside = |path: &str| {
            probes::first_line(&from_posix(&posix::join(
                &posix::dirname(path),
                PROTON_VERSION_FILE,
            )))
        };
        let is_proton = runtime.source == WineSource::Proton;
        let tool_version = if is_proton {
            version_beside(&runtime.binary)
        } else {
            None
        };
        let prefix_version = runtime
            .prefix
            .as_deref()
            .filter(|prefix| is_proton && posix::basename(prefix) == PROTON_PREFIX_DIR)
            .and_then(version_beside);
        Some(CompatLayerReport {
            source: runtime.source,
            label: runtime.label(),
            prefix_exists: runtime
                .prefix
                .as_deref()
                .is_some_and(|prefix| from_posix(prefix).is_dir()),
            binary: runtime.binary,
            prefix: runtime.prefix,
            tool_version,
            prefix_version,
        })
    }

    fn mount_of(&self, path: &std::path::Path) -> Option<MountFacts> {
        probes::mount_of(&self.host, self.runner.as_ref(), path)
    }

    fn collect_filesystems(&self, game: Option<&GamePaths>, report: &mut DiagnosticsReport) {
        if self.host.os() != Os::Linux {
            return;
        }
        if let Some(game) = game {
            report
                .filesystems
                .insert(FilesystemRole::GameFolder, self.mount_of(game.root()));
        }
        report.filesystems.insert(
            FilesystemRole::LauncherFolder,
            self.mount_of(&self.paths.runtime_dir),
        );
        let prefix = report
            .compat_layer
            .as_ref()
            .and_then(|compat| compat.prefix.as_deref())
            .map(from_posix)
            .filter(|prefix| prefix.is_dir());
        if let Some(prefix) = prefix {
            report
                .filesystems
                .insert(FilesystemRole::WinePrefix, self.mount_of(&prefix));
        }
    }

    fn collect_logs(&self, nams_logs: &Path) -> LogsReport {
        let dir = nams_logs;
        let crash_dumps = probes::newest_first(dir, &|name| {
            name.starts_with(CRASH_DUMP_PREFIX) && name.ends_with(CRASH_DUMP_SUFFIX)
        });
        let captured = probes::newest_first(&self.paths.logs_dir, &|name| {
            name.starts_with(CAPTURED_LOG_PREFIX) && name.ends_with(CAPTURED_LOG_SUFFIX)
        });
        LogsReport {
            nams_log: probes::tail(
                &dir.join(NAMS_LOG_NAME),
                probes::TAIL_MAX_LINES,
                probes::TAIL_MAX_BYTES,
            ),
            crash_dump_count: crash_dumps.len(),
            newest_crash_dump: crash_dumps
                .first()
                .and_then(|newest| probes::head(newest, probes::HEAD_MAX_LINES)),
            heartbeat: probes::tail(
                &dir.join(HEARTBEAT_LOG_NAME),
                HEARTBEAT_TAIL_LINES,
                probes::TAIL_MAX_BYTES,
            ),
            captured_launch_logs: captured
                .iter()
                .take(CAPTURED_LOGS_SHOWN)
                .filter_map(|path| probes::facts(path, false))
                .collect(),
        }
    }

    fn system_info(
        &self,
        report: &DiagnosticsReport,
        gpu_probe: &GpuProbe,
    ) -> IndexMap<String, String> {
        let mut info = IndexMap::new();
        let mut put = |key: &str, value: String| {
            info.insert(key.to_owned(), value);
        };
        put(OS_KEY, os_name(self.host.os()).to_owned());
        put(
            OS_VERSION_KEY,
            probes::os_version_label(&self.host, self.runner.as_ref(), &self.system_files),
        );
        put(
            LOCALE_KEY,
            self.host
                .system_language()
                .unwrap_or_else(|| "?".to_owned()),
        );
        put(GAME_RUNNING_KEY, yes_no(report.game_running).to_owned());
        put(
            STEAM_RUNNING_KEY,
            match report.nams_health.steam_running {
                Some(running) => yes_no(running),
                None => "unknown",
            }
            .to_owned(),
        );
        put(
            PREFER_DEDICATED_GPU_KEY,
            yes_no(report.prefer_dedicated_gpu).to_owned(),
        );
        for (key, value) in probes::linux_context(&self.host, &self.system_files) {
            put(&key, value);
        }
        if self.host.os() == Os::Linux {
            let gpu_env: &[(&str, &str)] = if report.prefer_dedicated_gpu {
                gpu::linux_gpu_env(gpu_probe)
            } else {
                &[]
            };
            let gpu_env_label = if gpu_env.is_empty() {
                "none".to_owned()
            } else {
                gpu_env
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            put(
                NVIDIA_DRIVER_KEY,
                yes_no(gpu_probe.nvidia_driver_loaded).to_owned(),
            );
            put(DRM_RENDER_NODES_KEY, gpu_probe.render_nodes.to_string());
            put(DEDICATED_GPU_ENV_KEY, gpu_env_label);
        }
        info
    }

    fn launcher_info(&self, game: Option<&GamePaths>) -> IndexMap<String, String> {
        let game_dir = match game {
            Some(game) => game.root().to_string_lossy().into_owned(),
            None => GAME_DIR_NOT_SELECTED.to_owned(),
        };
        IndexMap::from([
            (LAUNCHER_VERSION_KEY.to_owned(), APP_VERSION.to_owned()),
            (
                LAUNCHER_DIR_KEY.to_owned(),
                self.paths.runtime_dir.to_string_lossy().into_owned(),
            ),
            (GAME_DIR_KEY.to_owned(), game_dir),
        ])
    }
}
