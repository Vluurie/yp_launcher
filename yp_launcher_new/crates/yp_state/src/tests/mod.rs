mod cell;
mod config;
mod mods;
mod navigation;
mod session;
mod stores;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use gpui::TestAppContext;
use tempfile::TempDir;
use yp_core::testing::FakeRunner;
use yp_core::{
    AppPaths, CancelToken, GamePaths, HostEnv, LaunchUnavailable, NamsCommandRequest,
    NierInstallation, Os, PlatformAdapter, ProcessSpec, SelectionRejection, SettingsStore,
};
use yp_host::HostPlatform;
use yp_host::launch::command::{LaunchCommand, build_native};

use crate::{AppEnv, StateExt, Stores};

pub(crate) struct TestPlatform {
    host: HostEnv,
    pub running: AtomicBool,
    pub can_launch: AtomicBool,
    pub terminate_succeeds: AtomicBool,
    pub terminations: AtomicUsize,
}

impl TestPlatform {
    fn new(host: HostEnv) -> Self {
        Self {
            host,
            running: AtomicBool::new(false),
            can_launch: AtomicBool::new(true),
            terminate_succeeds: AtomicBool::new(true),
            terminations: AtomicUsize::new(0),
        }
    }
}

impl PlatformAdapter for TestPlatform {
    fn host(&self) -> &HostEnv {
        &self.host
    }

    fn can_launch_game(&self) -> bool {
        self.can_launch.load(Ordering::SeqCst)
    }

    fn is_experimental(&self) -> bool {
        false
    }

    fn uses_native_title_bar(&self) -> bool {
        false
    }

    fn supports_desktop_shortcut(&self) -> bool {
        false
    }

    fn build_nams_command(
        &self,
        request: &NamsCommandRequest<'_>,
    ) -> Result<ProcessSpec, LaunchUnavailable> {
        self.build_launch_command(request)
            .map(|command| command.to_process_spec())
    }

    fn resolve_nams_settings_path(&self, game_dir: Option<&Path>) -> Option<PathBuf> {
        yp_host::platform::nams_settings_path(game_dir)
    }

    fn is_game_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn terminate_game(&self) -> bool {
        self.terminations.fetch_add(1, Ordering::SeqCst);
        let stopped = self.terminate_succeeds.load(Ordering::SeqCst);
        if stopped {
            self.running.store(false, Ordering::SeqCst);
        }
        stopped
    }

    fn discover_fast(&self) -> Vec<NierInstallation> {
        Vec::new()
    }

    fn discover_deep(&self, _cancel: &CancelToken) -> Vec<NierInstallation> {
        Vec::new()
    }

    fn reject_game_selection(&self, _exe: &Path) -> Option<SelectionRejection> {
        None
    }

    fn reveal(&self, _target: &Path, _is_file: bool) -> yp_core::Result<()> {
        Ok(())
    }
}

impl HostPlatform for TestPlatform {
    fn build_launch_command(
        &self,
        request: &NamsCommandRequest<'_>,
    ) -> Result<LaunchCommand, LaunchUnavailable> {
        Ok(build_native(
            &request.nams_exe.to_string_lossy(),
            &request.game_dir.to_string_lossy(),
            &request.launcher_dir.to_string_lossy(),
            request.invocation,
        ))
    }
}

pub(crate) struct Harness {
    pub root: TempDir,
    pub game: GamePaths,
    pub env: AppEnv,
    pub runner: FakeRunner,
    pub platform: Arc<TestPlatform>,
}

impl Harness {
    pub fn new() -> Self {
        Self::with_runner(FakeRunner::new())
    }

    pub fn with_runner(runner: FakeRunner) -> Self {
        let root = tempfile::tempdir().unwrap();
        let host = HostEnv::for_test(Os::Windows, &[], root.path());
        let paths = AppPaths::for_test(root.path());
        std::fs::create_dir_all(paths.plugins_dir()).unwrap();
        std::fs::create_dir_all(&paths.logs_dir).unwrap();
        std::fs::create_dir_all(host.temp_dir()).unwrap();
        std::fs::write(paths.nams_exe(), b"nams").unwrap();
        std::fs::write(paths.yorha_dll(), b"yorha").unwrap();
        let game = GamePaths::new(root.path().join("game"));
        std::fs::create_dir_all(game.root()).unwrap();
        std::fs::write(game.exe(), b"exe").unwrap();
        let platform = Arc::new(TestPlatform::new(host.clone()));
        let env = AppEnv::new(
            host,
            paths,
            Arc::new(SettingsStore::in_memory()),
            Arc::new(runner.clone()),
        )
        .with_platform(platform.clone())
        .with_launch_grace(Duration::ZERO)
        .with_file_watching(false)
        .with_gpu_registry(false);
        Self {
            root,
            game,
            env,
            runner,
            platform,
        }
    }

    pub fn init(&self, cx: &mut TestAppContext) -> Stores {
        let env = self.env.clone();
        cx.update(|cx| {
            crate::init_with(env, cx);
            cx.stores().clone()
        })
    }

    pub fn start(&self, cx: &mut TestAppContext) -> Stores {
        let stores = self.init(cx);
        self.select_game(&stores, cx);
        stores
    }

    pub fn select_game(&self, stores: &Stores, cx: &mut TestAppContext) {
        let dir = self.game.root().to_path_buf();
        stores
            .session
            .update(cx, |session, cx| session.set_directory(dir, cx));
        cx.run_until_parked();
    }

    pub fn write_data_mod(&self, name: &str) -> PathBuf {
        let drop = self.root.path().join("drops").join(name);
        let model_dir = drop.join("data").join("pl");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("pl000d.dat"), [0u8; 32]).unwrap();
        std::fs::write(model_dir.join("pl000d.dtt"), [0u8; 32]).unwrap();
        drop
    }
}

pub(crate) fn settle(cx: &mut TestAppContext, delay: Duration) {
    cx.executor().advance_clock(delay);
    cx.run_until_parked();
}
