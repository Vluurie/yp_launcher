#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;
use yp_core::testing::FakeRunner;
use yp_core::{
    AppPaths, CancelToken, GamePaths, HostEnv, LaunchUnavailable, NamsCommandRequest,
    NierInstallation, Os, PlatformAdapter, ProcessSpec, SelectionRejection, SettingsStore,
};
use yp_formats::testing::usm_file;
use yp_host::HostPlatform;
use yp_host::launch::command::{LaunchCommand, build_native};
use yp_state::AppEnv;

pub const CODEC_MPEG2: u32 = 1;
pub const CODEC_H264: u32 = 5;

pub struct QuietPlatform {
    host: HostEnv,
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
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

fn write(path: PathBuf, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("fixture dir");
    }
    std::fs::write(path, bytes).expect("fixture file");
}

impl Sandbox {
    pub fn new() -> Self {
        let root = tempfile::tempdir().expect("temp dir");
        let host = HostEnv::for_test(Os::Windows, &[], root.path());
        let paths = AppPaths::for_test(root.path());
        for dir in [
            paths.plugins_dir(),
            paths.logs_dir.clone(),
            host.temp_dir().to_path_buf(),
        ] {
            std::fs::create_dir_all(dir).expect("sandbox dir");
        }
        std::fs::write(paths.nams_exe(), b"nams").expect("fake nams");
        std::fs::write(paths.yorha_dll(), b"yorha").expect("fake plugin");
        let game = GamePaths::new(root.path().join("game"));
        std::fs::create_dir_all(game.root()).expect("game dir");
        std::fs::write(game.exe(), b"exe").expect("fake game exe");
        let platform = Arc::new(QuietPlatform { host: host.clone() });
        let env = AppEnv::new(
            host,
            paths,
            Arc::new(SettingsStore::in_memory()),
            Arc::new(FakeRunner::new()),
        )
        .with_platform(platform)
        .with_launch_grace(Duration::ZERO)
        .with_file_watching(false)
        .with_gpu_registry(false);
        Self { root, game, env }
    }

    pub fn mod_dir(&self, id: &str) -> PathBuf {
        self.game.mods_dir().join(id)
    }

    pub fn drop_dir(&self, name: &str) -> PathBuf {
        self.root.path().join("drops").join(name)
    }

    pub fn pack_drop(&self, folder: &Path, archive: &str) -> PathBuf {
        let path = archive
            .split('/')
            .fold(self.root.path().join("drops"), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().expect("archive parent")).expect("archive dir");
        yp_core::testing::archives::write_archive_of_dir(folder, &path);
        path
    }

    pub fn write_drop_file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = name
            .split('/')
            .fold(self.root.path().join("drops"), |path, part| path.join(part));
        write(path.clone(), bytes);
        path
    }

    pub fn write_locked_archive(&self, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let path = self.drop_dir(name);
        std::fs::create_dir_all(path.parent().expect("archive parent")).expect("archive dir");
        yp_core::testing::archives::write_encrypted_zip(&path, b"secret", files);
        path
    }

    pub fn add_native_mod(&self, id: &str, display_name: &str, version: &str) -> PathBuf {
        let dir = self.mod_dir(id);
        let manifest = format!(
            "[mod]\nid = \"{id}\"\ndisplay_name = \"{display_name}\"\nauthor = \"Vluurie\"\nversion = \"{version}\"\n"
        );
        write(dir.join("mod.toml"), manifest.as_bytes());
        write(
            dir.join("entities").join("item_bunker_key.toml"),
            b"[item]\nid = 9001\n",
        );
        write(
            dir.join("entities").join("quest_bunker.toml"),
            b"[quest]\nid = 77\n",
        );
        dir
    }

    pub fn add_camera_settings(&self, id: &str) {
        write(
            self.mod_dir(id).join("entities").join("camera.toml"),
            b"[player_camera_settings]\nside_offset = 0.5\nheight_offset = 0.0\ndistance_offset = 1.25\nfov_offset = 5\npitch_up_limit = 72\npitch_down_limit = 60\n",
        );
    }

    pub fn add_requirements(&self, id: &str, requires: &[&str], plugins: &[&str]) {
        let list = |items: &[&str]| {
            items
                .iter()
                .map(|item| format!("\"{item}\""))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let manifest = format!(
            "[mod]\nid = \"{id}\"\ndisplay_name = \"{id}\"\nrequires = [{}]\nrequires_plugins = [{}]\n",
            list(requires),
            list(plugins)
        );
        write(self.mod_dir(id).join("mod.toml"), manifest.as_bytes());
    }

    pub fn add_data_file(&self, id: &str, data_dir: &str, file: &str, size: usize) {
        write(
            self.mod_dir(id).join("data").join(data_dir).join(file),
            &vec![0u8; size],
        );
    }

    pub fn add_outfit_mod(&self, id: &str, stem: &str) -> PathBuf {
        self.add_data_file(id, "pl", &format!("{stem}.dat"), 32);
        self.add_data_file(id, "pl", &format!("{stem}.dtt"), 32);
        self.mod_dir(id)
    }

    pub fn add_wax_mod(&self, id: &str, stem: &str) -> PathBuf {
        let dir = self.add_outfit_mod(id, stem);
        write(
            dir.join("wax")
                .join("mods")
                .join(id)
                .join("outfits")
                .join("outfits.json"),
            br#"{"pl_2b_outfits":[{"outfit_id":20,"outfit_name":"Alice","item_id":900},{"outfit_id":21,"item_id":901}]}"#,
        );
        dir
    }

    pub fn add_bundled_textures(&self, id: &str, packs: &[&str]) {
        let list = packs
            .iter()
            .map(|pack| format!("\"{pack}\""))
            .collect::<Vec<_>>()
            .join(",");
        write(
            self.mod_dir(id).join(".bundled_textures.json"),
            format!("{{\"packs\":[{list}]}}").as_bytes(),
        );
    }

    pub fn add_bundled_cutscene(&self, id: &str, name: &str) {
        write(
            self.mod_dir(id)
                .join("cutscenes")
                .join(name)
                .join("movie")
                .join("ev0010.usm"),
            &usm_file(1920, 1080, CODEC_H264),
        );
    }

    pub fn add_cutscene_mod(&self, name: &str, files: usize, width: u32, height: u32, codec: u32) {
        for index in 0..files {
            write(
                self.game
                    .cutscenes_dir()
                    .join(name)
                    .join("movie")
                    .join(format!("ev{index:04}.usm")),
                &usm_file(width, height, codec),
            );
        }
    }

    pub fn write_cutscene_drop(&self, name: &str, files: usize) -> PathBuf {
        let dir = self.drop_dir(name);
        for index in 0..files {
            write(
                dir.join("movie").join(format!("ev{index:04}.usm")),
                &usm_file(2560, 1440, CODEC_MPEG2),
            );
        }
        dir
    }

    pub fn write_outfit_drop(&self, name: &str) -> PathBuf {
        let dir = self.drop_dir(name);
        write(dir.join("data").join("pl").join("pl000d.dat"), &[0u8; 32]);
        write(dir.join("data").join("pl").join("pl000d.dtt"), &[0u8; 32]);
        dir
    }

    pub fn write_native_drop(&self, name: &str, id: &str) -> PathBuf {
        let dir = self.drop_dir(name);
        write(
            dir.join("mod.toml"),
            format!("[mod]\nid = \"{id}\"\ndisplay_name = \"{name}\"\n").as_bytes(),
        );
        write(
            dir.join("entities").join("item_one.toml"),
            b"[item]\nid = 1\n",
        );
        dir
    }

    pub fn write_variant_drop(&self, name: &str, variants: &[&str]) -> PathBuf {
        let dir = self.drop_dir(name);
        for variant in variants {
            write(
                dir.join(variant).join("data").join("pl").join("pl000d.dat"),
                &[0u8; 32],
            );
            write(
                dir.join(variant).join("data").join("pl").join("pl000d.dtt"),
                &[0u8; 32],
            );
        }
        dir
    }

    pub fn write_texture_drop(&self, name: &str) -> PathBuf {
        let dir = self.drop_dir(name);
        write(dir.join("textures").join("a.dds"), &[0u8; 16]);
        write(dir.join("textures").join("b.dds"), &[0u8; 16]);
        dir
    }

    pub fn write_unknown_drop(&self, name: &str) -> PathBuf {
        let dir = self.drop_dir(name);
        write(dir.join("readme.txt"), b"hello");
        dir
    }

    pub fn thirdparty_dir(&self) -> PathBuf {
        self.game.thirdparty_dir()
    }

    pub fn install_reshade(&self) -> PathBuf {
        let dir = self.thirdparty_dir().join("reshade").join("ReShade");
        write(
            dir.join("ReShade.ini"),
            b"[GENERAL]\nPerformanceMode=1\nPresetPath=.\\reshade-presets\\Cinematic.ini\n[OVERLAY]\nShowFPS=0\nShowClock=1\n",
        );
        write(dir.join("ReShade64.dll"), b"MZ reshade.me ReShade 6.3.0");
        for preset in ["Cinematic.ini", "Vivid.ini"] {
            write(
                dir.join("reshade-presets").join(preset),
                b"Techniques=Bloom@Bloom.fx\n",
            );
        }
        for shader in [
            "qUINT/qUINT_bloom.fx",
            "qUINT/qUINT_mxao.fx",
            "SweetFX/LumaSharpen.fx",
        ] {
            write(
                dir.join("reshade-shaders").join("Shaders").join(shader),
                b"technique Bloom {}\n",
            );
        }
        dir
    }

    pub fn add_game_root_reshade(&self) -> PathBuf {
        let path = self.game.root().join("dxgi.dll");
        write(path.clone(), b"MZ reshade.me ReShade 6.3.0");
        path
    }

    pub fn write_nams_toml(&self, contents: &str) {
        write(self.game.nams_toml(), contents.as_bytes());
    }

    pub fn install_migoto(&self, loader_target: &str) -> PathBuf {
        let dir = self.thirdparty_dir().join("3dmigoto").join("3DMigoto");
        write(dir.join("d3d11.dll"), &[0u8; 2048]);
        write(
            dir.join("d3dx.ini"),
            format!(
                "[Loader]\ntarget = {loader_target}\n[Hunting]\nhunting = 1\nmarking_mode = pink\n[Rendering]\ncache_shaders = 1\n"
            )
            .as_bytes(),
        );
        write(
            dir.join("ShaderFixes")
                .join("0123456789abcdef-ps_replace.txt"),
            b"// fix\n",
        );
        dir
    }

    pub fn add_game_mod(&self, mod_dir: &str, dll: &str, size: usize) -> PathBuf {
        let path = self.thirdparty_dir().join("game").join(mod_dir).join(dll);
        write(path.clone(), &vec![0u8; size]);
        path
    }

    pub fn add_graphic_mod(&self, mod_dir: &str, dll: &str, size: usize) -> PathBuf {
        let path = self
            .thirdparty_dir()
            .join("graphic")
            .join(mod_dir)
            .join(dll);
        write(path.clone(), &vec![0u8; size]);
        path
    }

    pub fn write_game_mod_drop(&self, name: &str, dll: &str, size: usize) -> PathBuf {
        let dir = self.drop_dir(name);
        write(dir.join(dll), &vec![0u8; size]);
        dir
    }
}
