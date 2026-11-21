#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Value as Json, json};
use tempfile::TempDir;
use yp_config::testing::{FakeNamsCli, sample_schema};
use yp_core::consts::{GAME_EXE_NAME, NAMS_EXE_NAME, YORHA_DLL_NAME};
use yp_core::testing::FakeRunner;
use yp_core::{
    AppPaths, CancelToken, GamePaths, HostEnv, LaunchUnavailable, NamsCommandRequest, NamsInvocation,
    NierInstallation, Os, PlatformAdapter, ProcessSpec, SelectionRejection, SettingsStore,
};
use yp_health::diagnostics::Collector;
use yp_health::diagnostics::model::{DiagnosticsReport, GAME_DIR_KEY, ReportHost};
use yp_health::probes::SystemFiles;
use yp_host::game_probe::WINDOWS10_SIZE;
use yp_host::gpu::GpuProbe;
use yp_host::launch::command::invocation_args;
use yp_host::wine::WineContext;

pub const DLC_BYTES: u64 = 930 * 1024 * 1024;
pub const TEST_USER: &str = "tester";

pub struct FakePlatform {
    pub host: HostEnv,
    pub can_launch: bool,
    pub game_running: bool,
    pub runs_config: bool,
}

impl FakePlatform {
    pub fn new(host: HostEnv) -> Self {
        Self {
            host,
            can_launch: true,
            game_running: false,
            runs_config: false,
        }
    }

    pub fn running_config(host: HostEnv) -> Self {
        Self {
            runs_config: true,
            ..Self::new(host)
        }
    }

    pub fn unable_to_launch(host: HostEnv) -> Self {
        Self {
            can_launch: false,
            ..Self::new(host)
        }
    }
}

impl PlatformAdapter for FakePlatform {
    fn host(&self) -> &HostEnv {
        &self.host
    }

    fn can_launch_game(&self) -> bool {
        self.can_launch
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
        match request.invocation {
            NamsInvocation::Config(_) if self.runs_config => {
                let game = request.game_dir.to_string_lossy();
                Ok(ProcessSpec::new(request.nams_exe).args(invocation_args(request.invocation, &game)))
            }
            _ => Err(LaunchUnavailable::NoCompatLayer { os: self.host.os() }),
        }
    }

    fn resolve_nams_settings_path(&self, game_dir: Option<&Path>) -> Option<PathBuf> {
        yp_host::platform::nams_settings_path(game_dir)
    }

    fn is_game_running(&self) -> bool {
        self.game_running
    }

    fn terminate_game(&self) -> bool {
        false
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

pub struct Rig {
    pub root: TempDir,
    pub host: HostEnv,
    pub paths: AppPaths,
    pub settings: Arc<SettingsStore>,
    pub runner: FakeRunner,
}

impl Rig {
    pub fn new(os: Os) -> Self {
        Self::with_runner(os, FakeRunner::new())
    }

    pub fn with_runner(os: Os, runner: FakeRunner) -> Self {
        let root = tempfile::Builder::new()
            .prefix("yp_diag_launcher_")
            .tempdir()
            .unwrap();
        let home = root.path().join("home");
        let home_text = home.to_string_lossy().into_owned();
        let vars: Vec<(&str, &str)> = match os {
            Os::Windows => vec![("USERPROFILE", home_text.as_str()), ("USERNAME", TEST_USER)],
            Os::Linux | Os::MacOs => vec![("HOME", home_text.as_str()), ("USER", TEST_USER)],
        };
        let host = HostEnv::for_test(os, &vars, root.path());
        let paths = AppPaths::for_test(root.path());
        Self {
            root,
            host,
            paths,
            settings: Arc::new(SettingsStore::in_memory()),
            runner,
        }
    }

    pub fn native() -> Self {
        Self::new(Os::native())
    }

    pub fn windows() -> Self {
        Self::new(Os::Windows)
    }

    pub fn system_root(&self) -> PathBuf {
        self.root.path().join("system")
    }

    pub fn platform(&self) -> FakePlatform {
        FakePlatform::new(self.host.clone())
    }

    pub fn collector(&self) -> Collector {
        Collector::new(
            self.host.clone(),
            self.paths.clone(),
            self.settings.clone(),
            Arc::new(self.runner.clone()),
            Arc::new(self.platform()),
        )
        .with_wine_context(WineContext::isolated(self.host.clone()))
        .with_system_files(SystemFiles::under(&self.system_root()))
        .with_gpu_probe(GpuProbe::default())
        .with_steam_running(Some(true))
    }

    pub fn collect(&self, game: Option<&Path>) -> DiagnosticsReport {
        self.collect_with_preview(game, None)
    }

    pub fn collect_with_preview(
        &self,
        game: Option<&Path>,
        preview: Option<String>,
    ) -> DiagnosticsReport {
        let game = game.map(GamePaths::new);
        self.collector()
            .collect(game.as_ref(), preview, &CancelToken::new())
            .unwrap()
    }

    pub fn install_runtime_files(&self) {
        std::fs::create_dir_all(self.paths.plugins_dir()).unwrap();
        std::fs::write(self.paths.nams_exe(), b"MZ nams").unwrap();
        std::fs::write(self.paths.yorha_dll(), b"MZ yorha").unwrap();
    }

    pub fn write_log(&self, name: &str, content: &str) -> PathBuf {
        std::fs::create_dir_all(&self.paths.logs_dir).unwrap();
        let file = self.paths.logs_dir.join(name);
        std::fs::write(&file, content).unwrap();
        file
    }
}

pub fn write(path: &Path, bytes: impl AsRef<[u8]>) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

pub fn make_sized_file(path: &Path, size: u64) {
    let file = std::fs::File::create(path).unwrap();
    file.set_len(size).unwrap();
}

pub struct FakeGame {
    pub dir: TempDir,
    root: PathBuf,
}

impl FakeGame {
    pub fn create() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("yp_diag_")
            .tempdir()
            .unwrap();
        let root = dir.path().to_path_buf();
        Self::populate(&root);
        Self { dir, root }
    }

    pub fn create_at(relative: &[&str]) -> Self {
        let dir = tempfile::Builder::new()
            .prefix("yp_diag_lib_")
            .tempdir()
            .unwrap();
        let root = relative
            .iter()
            .fold(dir.path().to_path_buf(), |path, part| path.join(part));
        std::fs::create_dir_all(&root).unwrap();
        Self { dir, root }
    }

    fn populate(root: &Path) {
        let data = root.join("data");
        write(&data.join("data000.cpk"), [0u8; 16]);
        write(&data.join("hack.dll"), [0u8; 8]);
        write(&data.join("enemy.dtt"), [0u8; 8]);
        write(&data.join("sound").join("bgm.wsp"), [0u8; 8]);
        make_sized_file(&root.join(GAME_EXE_NAME), WINDOWS10_SIZE);
        write(&root.join("steam_api64.dll"), [0u8; 8]);
        let nams = root.join("nams");
        write(&nams.join("nams.toml"), "disable_splash_screen = true\n");
        write(&nams.join("lodmod.toml"), "");
        write(&nams.join("texture_injection.toml"), "");
    }

    pub fn with_dlc_archive(self) -> Self {
        make_sized_file(&self.root.join("data").join("data100.cpk"), DLC_BYTES);
        self
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn paths(&self) -> GamePaths {
        GamePaths::new(&self.root)
    }

    pub fn exe(&self) -> PathBuf {
        self.root.join(GAME_EXE_NAME)
    }

    pub fn nams(&self) -> PathBuf {
        self.root.join("nams")
    }

    pub fn text(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }
}

pub fn pe_header(stamp: u32, nt: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; 0x200];
    bytes[0] = 0x4d;
    bytes[1] = 0x5a;
    bytes[0x3c..0x40].copy_from_slice(&(nt as u32).to_le_bytes());
    bytes[nt..nt + 4].copy_from_slice(&0x0000_4550u32.to_le_bytes());
    bytes[nt + 8..nt + 12].copy_from_slice(&stamp.to_le_bytes());
    bytes
}

pub fn manifest(target: &str, disabled_dlc: &str) -> String {
    format!(
        "\"AppState\"\n{{\n\t\"appid\"\t\t\"524220\"\n\t\"buildid\"\t\t\"25901062\"\n\t\"TargetBuildID\"\t\t\"{target}\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"LastUpdated\"\t\t\"1793892324\"\n\t\"InstalledDepots\"\n\t{{\n\t\t\"524221\"\n\t\t{{\n\t\t\t\"manifest\"\t\t\"2909777361921225087\"\n\t\t\t\"size\"\t\t\"43732108109\"\n\t\t}}\n\t\t\"580600\"\n\t\t{{\n\t\t\t\"manifest\"\t\t\"1599661617359645076\"\n\t\t\t\"size\"\t\t\"43732108109\"\n\t\t\t\"dlcappid\"\t\t\"580600\"\n\t\t}}\n\t}}\n\t\"UserConfig\"\n\t{{\n\t\t\"language\"\t\t\"english\"\n\t\t\"DisabledDLC\"\t\t\"{disabled_dlc}\"\n\t}}\n}}\n"
    )
}

pub fn bare_report(game_dir: &str) -> DiagnosticsReport {
    let mut report = DiagnosticsReport::empty(
        "2026-11-06T00:00:00Z".parse().unwrap(),
        ReportHost::anonymous(Os::native()),
    );
    if !game_dir.is_empty() {
        report.game_dir = Some(PathBuf::from(game_dir));
    }
    report
        .launcher_info
        .insert(GAME_DIR_KEY.to_owned(), game_dir.to_owned());
    report
}

pub fn messages(report: &DiagnosticsReport) -> String {
    report
        .findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn runtime_file_names() -> [&'static str; 2] {
    [NAMS_EXE_NAME, YORHA_DLL_NAME]
}

pub fn naiom_schema() -> Json {
    let mut schema = sample_schema();
    let extra = [
        json!({ "id": "naiom.aim_mode", "category": "input/mouse", "file": "naiom.toml", "key": "aim_mode", "type": "bool", "default": false }),
        json!({ "id": "naiom.aim_crosshair", "category": "input/mouse", "file": "naiom.toml", "key": "aim_crosshair", "type": "bool", "default": true }),
        json!({ "id": "naiom.third_person_sensitivity_x", "category": "input/mouse", "file": "naiom.toml", "key": "third_person_sensitivity_x", "type": "float", "default": 1.0 }),
    ];
    if let Some(settings) = schema["settings"].as_array_mut() {
        settings.extend(extra);
    }
    schema
}

pub fn fake_cli() -> FakeNamsCli {
    FakeNamsCli::new(naiom_schema())
}
