mod controller;
mod directory;
mod onboarding;
mod shell;
mod window;

use yp_config::cli::NamsCli;
use yp_config::testing::FakeNamsCli;
use yp_state::CliFactory;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use gpui::{Entity, TestAppContext, VisualTestContext};
use tempfile::TempDir;
use yp_core::testing::FakeRunner;
use yp_core::{
    AppPaths, CancelToken, GamePaths, HostEnv, LaunchUnavailable, NamsCommandRequest,
    NierInstallation, Os, PlatformAdapter, ProcessSpec, SelectionRejection, SettingsStore,
};
use yp_host::HostPlatform;
use yp_host::launch::command::{LaunchCommand, build_native};
use yp_state::{AppEnv, StateExt, Stores};
use yp_ui::{Root, ToastId, WindowExt};

use crate::{ShellOptions, ShellView};

pub(crate) struct TestPlatform {
    host: HostEnv,
    pub can_launch: AtomicBool,
    pub fast_installs: Mutex<Vec<NierInstallation>>,
    pub deep_installs: Mutex<Vec<NierInstallation>>,
    pub deep_searches: AtomicUsize,
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
        false
    }

    fn terminate_game(&self) -> bool {
        true
    }

    fn discover_fast(&self) -> Vec<NierInstallation> {
        self.fast_installs.lock().unwrap().clone()
    }

    fn discover_deep(&self, _cancel: &CancelToken) -> Vec<NierInstallation> {
        self.deep_searches.fetch_add(1, Ordering::SeqCst);
        self.deep_installs.lock().unwrap().clone()
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
    _root: TempDir,
    pub game: GamePaths,
    pub env: AppEnv,
    pub platform: Arc<TestPlatform>,
    pub cli: Arc<FakeNamsCli>,
}

impl Harness {
    pub fn new() -> Self {
        Self::build(true)
    }

    pub fn without_runtime() -> Self {
        Self::build(false)
    }

    fn build(with_runtime: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        let host = HostEnv::for_test(Os::Windows, &[], root.path());
        let paths = AppPaths::for_test(root.path());
        std::fs::create_dir_all(paths.plugins_dir()).unwrap();
        std::fs::create_dir_all(&paths.logs_dir).unwrap();
        std::fs::create_dir_all(host.temp_dir()).unwrap();
        if with_runtime {
            std::fs::write(paths.nams_exe(), b"nams").unwrap();
            std::fs::write(paths.yorha_dll(), b"yorha").unwrap();
        }
        let game = GamePaths::new(root.path().join("game"));
        std::fs::create_dir_all(game.root()).unwrap();
        std::fs::write(game.exe(), b"exe").unwrap();
        let platform = Arc::new(TestPlatform {
            host: host.clone(),
            can_launch: AtomicBool::new(true),
            fast_installs: Mutex::new(Vec::new()),
            deep_installs: Mutex::new(Vec::new()),
            deep_searches: AtomicUsize::new(0),
        });
        let env = AppEnv::new(
            host,
            paths,
            Arc::new(SettingsStore::in_memory()),
            Arc::new(FakeRunner::new()),
        )
        .with_platform(platform.clone())
        .with_launch_grace(Duration::ZERO)
        .with_file_watching(false)
        .with_gpu_registry(false);
        Self {
            _root: root,
            game,
            env,
            platform,
            cli: Arc::new(FakeNamsCli::default()),
        }
    }

    pub fn init(&self, cx: &mut TestAppContext) -> Stores {
        let env = self.env.clone();
        cx.update(|cx| {
            yp_ui::init_for_tests(cx);
            yp_state::init_with(env, cx);
            let cli = self.cli.clone();
            let factory: CliFactory = Arc::new(move |_: &GamePaths| cli.clone() as Arc<dyn NamsCli>);
            let config = cx.stores().config.clone();
            config.update(cx, |config, cx| config.set_cli_factory(factory, cx));
            crate::init_with(
                ShellOptions {
                    auto_search: false,
                    ..ShellOptions::default()
                },
                cx,
            );
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
}

pub(crate) struct Opened<'a> {
    pub shell: Entity<ShellView>,
    pub root: Entity<Root>,
    pub cx: &'a mut VisualTestContext,
}

pub(crate) fn open_shell(cx: &mut TestAppContext) -> Opened<'_> {
    let mut created = None;
    let (root, cx) = cx.add_window_view(|window, cx| {
        let shell = gpui::AppContext::new(cx, |cx| ShellView::new(window, cx));
        created = Some(shell.clone());
        Root::new(shell, window, cx)
    });
    cx.run_until_parked();
    Opened {
        shell: created.unwrap(),
        root,
        cx,
    }
}

pub(crate) fn open_main(cx: &mut TestAppContext) -> Opened<'_> {
    cx.update(|cx| cx.prefs().set_onboarding_complete(true));
    let opened = open_shell(cx);
    let shell = opened.shell.clone();
    opened.cx.update(|window, cx| {
        shell.update(cx, |shell, cx| {
            shell.finish_startup(Ok(yp_core::runtime::RuntimeStatus::Bundled), window, cx);
        });
    });
    opened.cx.run_until_parked();
    opened
}

pub(crate) fn has_modal(root: &Entity<Root>, cx: &mut VisualTestContext) -> bool {
    root.read_with(cx, |root, _| root.has_modal())
}

pub(crate) fn has_toast(id: &'static str, cx: &mut VisualTestContext) -> bool {
    cx.update(|window, cx| {
        window
            .toast_stack(cx)
            .is_some_and(|stack| stack.read(cx).contains(&ToastId::from(id)))
    })
}
