use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gpui::Global;
use yp_core::archive::Extractor;
use yp_core::runtime::{RuntimeSetup, RuntimeStatus};
use yp_core::{
    AppPaths, GamePaths, HostEnv, OpenReport, PlatformAdapter, ProcessRunner, SettingsStore,
    SystemRunner,
};
use yp_host::HostPlatform;
use yp_host::launch::{GRACE_PERIOD, Launcher};
use yp_mods::DropStaging;

#[derive(Clone)]
pub struct AppEnv {
    pub host: HostEnv,
    pub paths: AppPaths,
    pub settings: Arc<SettingsStore>,
    pub runner: Arc<dyn ProcessRunner>,
    pub platform: Arc<dyn HostPlatform>,
    pub extractor: Extractor,
    pub launch_grace: Duration,
    pub watch_files: bool,
    pub write_gpu_registry: bool,
    pub settings_report: Option<OpenReport>,
}

impl Global for AppEnv {}

impl AppEnv {
    pub fn new(
        host: HostEnv,
        paths: AppPaths,
        settings: Arc<SettingsStore>,
        runner: Arc<dyn ProcessRunner>,
    ) -> Self {
        let platform = yp_host::adapter_for(host.clone(), runner.clone());
        Self {
            host,
            paths,
            settings,
            runner,
            platform,
            extractor: Extractor::new(),
            launch_grace: GRACE_PERIOD,
            watch_files: true,
            write_gpu_registry: true,
            settings_report: None,
        }
    }

    pub fn detect() -> yp_core::Result<Self> {
        Self::detect_for(HostEnv::current())
    }

    pub fn detect_for(host: HostEnv) -> yp_core::Result<Self> {
        let paths = AppPaths::detect(&host)?;
        let (settings, report) = SettingsStore::open(paths.settings_file.clone())?;
        let mut env = Self::new(host, paths, Arc::new(settings), Arc::new(SystemRunner)).with_system_extractor();
        env.settings_report = Some(report);
        Ok(env)
    }

    pub fn fallback() -> Self {
        let host = HostEnv::current();
        let paths = AppPaths::for_test(host.temp_dir());
        Self::new(
            host,
            paths,
            Arc::new(SettingsStore::in_memory()),
            Arc::new(SystemRunner),
        )
        .with_system_extractor()
    }

    pub fn with_system_extractor(mut self) -> Self {
        self.extractor = Extractor::with_oem_code_page(yp_host::platform::oem_code_page());
        self
    }

    pub fn with_platform(mut self, platform: Arc<dyn HostPlatform>) -> Self {
        self.platform = platform;
        self
    }

    pub fn with_launch_grace(mut self, grace: Duration) -> Self {
        self.launch_grace = grace;
        self
    }

    pub fn with_file_watching(mut self, enabled: bool) -> Self {
        self.watch_files = enabled;
        self
    }

    pub fn with_gpu_registry(mut self, enabled: bool) -> Self {
        self.write_gpu_registry = enabled;
        self
    }

    pub fn game(&self, dir: impl AsRef<Path>) -> GamePaths {
        GamePaths::new(dir.as_ref())
    }

    pub fn launcher(&self) -> Launcher {
        let launcher = Launcher::new(
            self.platform.clone(),
            self.runner.clone(),
            self.paths.clone(),
            self.settings.clone(),
        )
        .with_grace(self.launch_grace);
        if self.write_gpu_registry {
            launcher
        } else {
            launcher.with_gpu_registry(false)
        }
    }

    pub fn platform_adapter(&self) -> Arc<dyn PlatformAdapter> {
        self.platform.clone()
    }

    pub fn extractor(&self) -> Extractor {
        self.extractor
    }

    pub fn staging(&self) -> DropStaging {
        DropStaging::new(self.extractor, self.host.clone())
    }

    pub fn runtime_setup(&self) -> RuntimeSetup<'_> {
        RuntimeSetup::new(&self.paths, &self.host)
    }

    pub fn ensure_runtime(&self) -> yp_core::Result<RuntimeStatus> {
        self.runtime_setup().ensure_ready()
    }

    pub fn missing_runtime_files(&self) -> Vec<&'static str> {
        self.runtime_setup().missing_files()
    }

    pub fn can_launch_game(&self) -> bool {
        self.platform.can_launch_game()
    }
}
