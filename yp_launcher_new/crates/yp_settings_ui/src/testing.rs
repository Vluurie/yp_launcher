use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use yp_core::testing::FakeRunner;
use yp_core::{
    AppPaths, CancelToken, GamePaths, HostEnv, LaunchUnavailable, NamsCommandRequest,
    NierInstallation, Os, PlatformAdapter, ProcessSpec, SelectionRejection, SettingsStore,
};
use yp_formats::testing::{cube_spec, dds_dxt1_solid, model_archives};
use yp_host::HostPlatform;
use yp_host::launch::command::{LaunchCommand, build_native};
use yp_config::testing::FakeNamsCli;
use yp_state::AppEnv;
use yp_state::inspector::ArchivePaths;

const TEXTURE_ID: u32 = 0xC0BE;
const TEXTURE_SIDE: u32 = 8;
const TEXTURE_COLOR: u16 = 0xF800;
const DDS_HEADER_LEN: usize = 128;

pub struct QuietPlatform {
    host: HostEnv,
}

impl QuietPlatform {
    pub fn new(host: HostEnv) -> Self {
        Self { host }
    }
}

impl PlatformAdapter for QuietPlatform {
    fn host(&self) -> &HostEnv {
        &self.host
    }

    fn can_launch_game(&self) -> bool {
        true
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

impl HostPlatform for QuietPlatform {
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

pub struct Sandbox {
    pub root: TempDir,
    pub game: GamePaths,
    pub env: AppEnv,
    pub cli: Arc<FakeNamsCli>,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Sandbox {
    pub fn new() -> Self {
        Self::with_runner(FakeRunner::new())
    }

    pub fn with_runner(runner: FakeRunner) -> Self {
        let root = tempfile::tempdir().expect("temp dir");
        let host = HostEnv::for_test(Os::Windows, &[], root.path());
        let paths = AppPaths::for_test(root.path());
        for dir in [paths.plugins_dir(), paths.logs_dir.clone(), host.temp_dir().to_path_buf()] {
            std::fs::create_dir_all(dir).expect("sandbox dir");
        }
        std::fs::write(paths.nams_exe(), b"nams").expect("fake nams");
        std::fs::write(paths.yorha_dll(), b"yorha").expect("fake plugin");
        let game = GamePaths::new(root.path().join("game"));
        std::fs::create_dir_all(game.root()).expect("game dir");
        std::fs::write(game.exe(), b"exe").expect("fake game exe");
        let platform = Arc::new(QuietPlatform::new(host.clone()));
        let env = AppEnv::new(
            host,
            paths,
            Arc::new(SettingsStore::in_memory()),
            Arc::new(runner),
        )
        .with_platform(platform)
        .with_launch_grace(Duration::ZERO)
        .with_file_watching(false)
        .with_gpu_registry(false);
        Self {
            root,
            game,
            env,
            cli: Arc::new(FakeNamsCli::default()),
        }
    }

    pub fn dds_bytes() -> Vec<u8> {
        dds_dxt1_solid(TEXTURE_SIDE, TEXTURE_SIDE, TEXTURE_COLOR)
    }

    pub fn add_texture_pack(&self, name: &str, files: &[&str]) -> PathBuf {
        let dir = self.game.inject_textures_dir().join(name);
        std::fs::create_dir_all(&dir).expect("pack dir");
        for file in files {
            std::fs::write(dir.join(file), Self::dds_bytes()).expect("pack file");
        }
        dir
    }

    pub fn write_drop_folder(&self, name: &str, files: &[&str]) -> PathBuf {
        let dir = self.root.path().join("drops").join(name);
        std::fs::create_dir_all(&dir).expect("drop dir");
        for file in files {
            let bytes = if file.to_lowercase().ends_with(".dds") {
                Self::dds_bytes()
            } else {
                vec![0u8; DDS_HEADER_LEN]
            };
            std::fs::write(dir.join(file), bytes).expect("drop file");
        }
        dir
    }

    pub fn write_model(&self, folder: &str, stem: &str, meshes: &[&str]) -> ArchivePaths {
        let dir = self.root.path().join("models").join(folder);
        std::fs::create_dir_all(&dir).expect("model dir");
        let archives = model_archives(
            stem,
            cube_spec(meshes).build(),
            &[(TEXTURE_ID, Self::dds_bytes())],
        );
        let dat = dir.join(format!("{stem}.dat"));
        let dtt = dir.join(format!("{stem}.dtt"));
        std::fs::write(&dat, archives.dat).expect("model dat");
        std::fs::write(&dtt, archives.dtt).expect("model dtt");
        ArchivePaths::new(Some(dat), Some(dtt))
    }
}
