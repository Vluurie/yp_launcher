mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use common::{files_under, nams_flag, temp, write, write_bytes};
use tempfile::TempDir;
use yp_core::{GamePaths, Os};
use yp_thirdparty::dll_id::{self, GraphicsDll};
use yp_thirdparty::{GraphicsRuntime, LoadingFlag, ReShadeRuntime};

const RESHADE: ReShadeRuntime = ReShadeRuntime::for_os(Os::Windows);

struct Staged {
    _dir: TempDir,
    game: GamePaths,
}

fn real_game_dir() -> PathBuf {
    std::env::var_os("YP_NIER_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"F:\SteamLibrary\steamapps\common\NieRAutomata"))
}

fn has_real_reshade(game_dir: &Path) -> bool {
    game_dir.is_dir()
        && ["dxgi.dll", "d3d11.dll", "ReShade64.dll"]
            .iter()
            .any(|name| dll_id::identify_file(&game_dir.join(name)) == GraphicsDll::ReShade)
}

fn copy_tree(src: &Path, dst: &Path) {
    for file in files_under(src) {
        let target = dst.join(file.strip_prefix(src).unwrap());
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(&file, &target).unwrap();
    }
}

fn finish_stage(dir: TempDir) -> Staged {
    write_bytes(&dir.path().join("d3dcompiler_47.dll"), &[0x90; 64]);
    write(&dir.path().join("nams").join("nams.toml"), "disable_reshade_loading = true\n");
    let game = GamePaths::new(dir.path());
    Staged { _dir: dir, game }
}

fn stage_real_install() -> Option<Staged> {
    let real = real_game_dir();
    if !has_real_reshade(&real) {
        return None;
    }
    let dir = temp("yp_reshade_real_");
    for name in ["dxgi.dll", "ReShade.ini", "ReShadePreset.ini", "reshade-shaders"] {
        let src = real.join(name);
        if src.is_dir() {
            copy_tree(&src, &dir.path().join(name));
        } else if src.is_file() {
            std::fs::copy(&src, dir.path().join(name)).unwrap();
        }
    }
    Some(finish_stage(dir))
}

fn stage_synthetic_install() -> Staged {
    let dir = temp("yp_reshade_synth_");
    write_bytes(&dir.path().join("dxgi.dll"), b"MZ\x90\x00 reshade.me crosire ReShade 6.4.1\0............");
    write(
        &dir.path().join("ReShade.ini"),
        "[GENERAL]\r\n\
         EffectSearchPaths=C:\\Games\\NieR\\reshade-shaders\\Shaders\\**\r\n\
         TextureSearchPaths=C:\\Games\\NieR\\reshade-shaders\\Textures\\**\r\n\
         PresetPath=C:\\Games\\NieR\\ReShadePreset.ini\r\n\
         IntermediateCachePath=C:\\Users\\me\\AppData\\Local\\Temp\\ReShade\r\n\
         PreprocessorDefinitions=RESHADE_DEPTH_LINEARIZATION_FAR_PLANE=1000.0,RESHADE_DEPTH_INPUT_IS_REVERSED=0\r\n\
         \r\n\
         [SCREENSHOT]\r\n\
         SavePath=C:\\Users\\me\\Pictures\r\n",
    );
    write(&dir.path().join("ReShadePreset.ini"), "Techniques=SMAA@SMAA.fx\n\n[SMAA.fx]\nEdgeDetectionThreshold=0.1\n");
    write(&dir.path().join("reshade-shaders").join("Shaders").join("SMAA.fx"), "technique SMAA {}");
    write(&dir.path().join("reshade-shaders").join("Textures").join("AreaTex.png"), "png");
    finish_stage(dir)
}

fn stages() -> Vec<Staged> {
    let mut out = vec![stage_synthetic_install()];
    out.extend(stage_real_install());
    out
}

#[test]
fn relocates_dll_ini_and_shaders_into_thirdparty_reshade() {
    for staged in stages() {
        assert!(RESHADE.import_from_game_root(&staged.game).unwrap());

        let dir = RESHADE.install_dir(&staged.game);
        let dll = dir.join("ReShade64.dll");
        let addon_dll = dir.join("ReShade64_Addon.dll");
        assert!(dll.is_file() || addon_dll.is_file());
        let relocated = if dll.is_file() { dll } else { addon_dll };
        assert_eq!(dll_id::identify_file(&relocated), GraphicsDll::ReShade);
        assert!(dir.join("ReShade.ini").is_file());
        assert!(dir.join("reshade-shaders").join("Shaders").is_dir());
    }
}

#[test]
fn reshade_ini_search_paths_point_at_dirs_that_exist_after_relocate() {
    for staged in stages() {
        RESHADE.import_from_game_root(&staged.game).unwrap();
        let dir = RESHADE.install_dir(&staged.game);
        let ini = std::fs::read_to_string(dir.join("ReShade.ini")).unwrap();

        assert!(ini.contains(r".\reshade-shaders\Shaders"));
        let shaders = dir.join("reshade-shaders").join("Shaders");
        assert!(shaders.is_dir());
        let shader_count = files_under(&shaders)
            .iter()
            .filter(|file| file.to_string_lossy().to_lowercase().ends_with(".fx"))
            .count();
        assert!(shader_count > 0, "real shaders relocated, so EffectSearchPaths has targets");
    }
}

#[test]
fn keeps_the_real_nier_depth_defs_from_the_installer_ini() {
    for staged in stages() {
        RESHADE.import_from_game_root(&staged.game).unwrap();
        let ini = std::fs::read_to_string(RESHADE.install_dir(&staged.game).join("ReShade.ini")).unwrap();
        assert!(ini.contains("RESHADE_DEPTH_LINEARIZATION_FAR_PLANE=1000"));
    }
}

#[test]
fn relocates_d3dcompiler_47_so_nams_can_preload_it_under_wine() {
    for staged in stages() {
        RESHADE.import_from_game_root(&staged.game).unwrap();
        assert!(
            RESHADE.install_dir(&staged.game).join("d3dcompiler_47.dll").is_file(),
            "compiler moved into thirdparty/reshade for the preload"
        );
    }
}

#[test]
fn leaves_the_game_root_original_untouched() {
    for staged in stages() {
        let listing = |root: &Path| -> BTreeSet<String> {
            std::fs::read_dir(root)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_file())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        };
        let before = listing(staged.game.root());

        RESHADE.import_from_game_root(&staged.game).unwrap();

        let after = listing(staged.game.root());
        assert_eq!(after, before, "game-root files copied into thirdparty, not moved/renamed");
        assert!(!after.iter().any(|name| name.ends_with(".nams-backup")), "no backup files created");
    }
}

#[test]
fn sets_disable_reshade_loading_to_false() {
    for staged in stages() {
        RESHADE.import_from_game_root(&staged.game).unwrap();
        assert_eq!(nams_flag(&staged.game, LoadingFlag::ReShade.key()), Some(false));
    }
}

#[test]
fn status_reports_installed_enabled_and_shaders_present() {
    for staged in stages() {
        RESHADE.import_from_game_root(&staged.game).unwrap();
        let status = RESHADE.status(&staged.game);
        assert!(status.installed);
        assert!(status.enabled);
        assert!(!status.shaders_missing);
    }
}

#[test]
fn synthetic_import_strips_machine_paths_and_reads_the_install() {
    let staged = stage_synthetic_install();
    RESHADE.import_from_game_root(&staged.game).unwrap();
    let dir = RESHADE.install_dir(&staged.game);
    let ini = std::fs::read_to_string(dir.join("ReShade.ini")).unwrap();

    assert!(!ini.contains("C:\\"), "{ini}");
    assert!(ini.contains("PresetPath=.\\reshade-presets\\ReShadePreset.ini"));
    assert!(ini.contains("IntermediateCachePath=.\\reshade-cache"));
    assert!(ini.contains("SavePath=\r\n"));
    assert!(dir.join("reshade-presets").join("ReShadePreset.ini").is_file());

    let info = RESHADE.status(&staged.game).reshade_info.unwrap();
    assert_eq!(info.presets, ["ReShadePreset"]);
    assert_eq!(info.shader_count, 1);
    assert_eq!(info.version.as_deref(), Some("ReShade 6.4.1"));
    assert_eq!(info.dll_name.as_deref(), Some("ReShade64.dll"));
    assert_eq!(info.config.active_preset.as_deref(), Some("ReShadePreset"));
    assert!(!info.d3d_compiler_missing);
}

#[test]
fn a_disabled_reshade_is_not_imported_back_by_the_state_refresh() {
    let staged = stage_synthetic_install();
    let host = yp_core::HostEnv::for_test(Os::Windows, &[], staged.game.root());
    let mut state = yp_thirdparty::ThirdPartyState::with_legacy_root(&host, staged.game.root().join("no-legacy"));
    state.set_game(Some(staged.game.clone()));

    state.refresh(false).unwrap();
    assert!(!state.statuses().reshade.installed, "loading is off, so nothing is imported");

    write(&staged.game.nams_toml(), "disable_reshade_loading = false\n");
    state.refresh(false).unwrap();
    assert!(state.statuses().reshade.installed);
    assert!(state.statuses().reshade.enabled);

    state.remove(yp_thirdparty::ThirdPartyRuntime::ReShade).unwrap();
    state.refresh(false).unwrap();
    assert!(!state.statuses().reshade.installed, "removing turns loading off, so the game-root copy stays out");
}
