mod common;

use std::path::{Path, PathBuf};

use common::{fresh_game, nams_flag, read_toml, temp, write, write_marked_dll, write_plain_dll, write_plain_dll_with, FreshGame};
use yp_core::consts::{GAME_MODS_DIR_NAME, THIRD_PARTY_DIR_NAME};
use yp_core::GamePaths;
use yp_thirdparty::paths;
use yp_thirdparty::{
    classify, GameModSettings, GameModsRuntime, GraphicModsRuntime, GraphicsRuntime, LoadingFlag, ThirdPartyKind,
    ThirdPartyRuntime,
};

const RUNTIME: GameModsRuntime = GameModsRuntime;

fn game() -> FreshGame {
    fresh_game("yp_gamemods_", "disable_game_mods_loading = true\n")
}

fn game_toml(game: &GamePaths) -> toml::Table {
    read_toml(&RUNTIME.config_path(game))
}

fn section<'a>(table: &'a toml::Table, key: &str) -> &'a toml::Table {
    table.get(key).and_then(toml::Value::as_table).unwrap()
}

fn mod_dir_names(kind_dir: &Path) -> Vec<String> {
    paths::mod_dirs(kind_dir).iter().map(|dir| dir.file_name().unwrap().to_string_lossy().into_owned()).collect()
}

fn installed() -> FreshGame {
    let fresh = game();
    let drop = temp("yp_gm_d_");
    write_plain_dll(&drop.path().join("celestial.dll"));
    RUNTIME.install(&fresh.game, &classify(drop.path())).unwrap();
    fresh
}

mod classification {
    use super::*;

    #[test]
    fn a_loose_game_offset_dll_is_a_game_mod() {
        let drop = temp("yp_gm_drop_");
        write_plain_dll(&drop.path().join("celestial.dll"));

        let c = classify(drop.path());

        assert_eq!(c.kind, ThirdPartyKind::GameMod);
        assert_eq!(common::file_names(&c.game_mod_dlls), ["celestial.dll"]);
    }

    #[test]
    fn a_reshade_dll_is_never_claimed_as_a_game_mod() {
        let drop = temp("yp_gm_rs_");
        write_marked_dll(&drop.path().join("dxgi.dll"), "reshade.me crosire");

        assert_eq!(classify(drop.path()).kind, ThirdPartyKind::ReShadeWholeInstall);
    }

    #[test]
    fn a_3dmigoto_dll_is_never_claimed_as_a_game_mod() {
        let drop = temp("yp_gm_mg_");
        write_marked_dll(&drop.path().join("d3d11.dll"), "HackerDevice 3DMigoto");

        assert_eq!(classify(drop.path()).kind, ThirdPartyKind::Migoto);
    }

    #[test]
    fn a_wax_dll_is_installed_as_a_game_mod_not_rejected() {
        let drop = temp("yp_gm_wax_");
        write_marked_dll(&drop.path().join("wax.dll"), "_wax loaded_");

        let c = classify(drop.path());

        assert_eq!(c.kind, ThirdPartyKind::GameMod);
        assert_eq!(common::file_names(&c.game_mod_dlls), ["wax.dll"]);
    }

    #[test]
    fn a_plain_dinput8_dll_is_a_game_mod_not_special_k() {
        let drop = temp("yp_gm_di8_");
        write_plain_dll(&drop.path().join("dinput8.dll"));

        assert_eq!(classify(drop.path()).kind, ThirdPartyKind::GameMod);
    }

    #[test]
    fn special_k_is_still_recognised_by_its_own_marker() {
        let drop = temp("yp_gm_sk_");
        write_marked_dll(&drop.path().join("dinput8.dll"), "SpecialK64");

        assert_eq!(classify(drop.path()).kind, ThirdPartyKind::SpecialK);
    }

    #[test]
    fn a_plain_direct3d_proxy_is_a_graphic_mod_with_its_files() {
        let drop = temp("yp_gm_gfx_");
        write_plain_dll(&drop.path().join("pkg").join("d3d11.dll"));
        write(&drop.path().join("pkg").join("d3d11.ini"), "x=1");

        let c = classify(drop.path());

        assert_eq!(c.kind, ThirdPartyKind::GraphicMod);
        assert_eq!(common::file_names(&c.graphic_mod_dlls), ["d3d11.dll"]);
        assert!(c.game_mod_dlls.is_empty());
    }

    #[test]
    fn several_dlls_in_one_drop_are_all_collected() {
        let drop = temp("yp_gm_multi_");
        write_plain_dll(&drop.path().join("b_mod.dll"));
        write_plain_dll(&drop.path().join("a_mod.dll"));

        let c = classify(drop.path());

        assert_eq!(c.kind, ThirdPartyKind::GameMod);
        assert_eq!(common::file_names(&c.game_mod_dlls), ["a_mod.dll", "b_mod.dll"]);
    }
}

mod install {
    use super::*;

    #[test]
    fn the_dll_lands_in_its_own_mod_folder_and_the_flag_is_switched_on() {
        let fresh = game();
        let drop = temp("yp_gm_i_");
        write_plain_dll(&drop.path().join("celestial.dll"));
        write(&drop.path().join("celestial.ini"), "x=1");

        let result = RUNTIME.install(&fresh.game, &classify(drop.path())).unwrap();

        assert!(result.is_ok());
        let mod_dir = paths::game_mods(&fresh.game).join("celestial");
        assert!(mod_dir.join("celestial.dll").is_file());
        assert!(mod_dir.join("celestial.ini").is_file(), "a mod folder holds every file of the mod");
        assert_eq!(nams_flag(&fresh.game, LoadingFlag::GameMods.key()), Some(false));
    }

    #[test]
    fn the_mod_folder_is_named_after_the_dropped_file() {
        let fresh = game();
        let drop = temp("yp_gm_n_");
        write_plain_dll(&drop.path().join("celestial.dll"));

        RUNTIME.install(&fresh.game, &classify(drop.path()).named("Celestial Timer")).unwrap();

        assert!(paths::game_mods(&fresh.game).join("Celestial Timer").join("celestial.dll").is_file());
        let status = RUNTIME.status(&fresh.game);
        let mods = &status.game_mods_info.as_ref().unwrap().mods;
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].mod_dir, "Celestial Timer");
        assert_eq!(mods[0].file_name, "celestial.dll");
    }

    #[test]
    fn an_update_goes_into_the_folder_that_already_holds_the_dll() {
        let fresh = game();
        let drop = temp("yp_gm_up_");
        write_plain_dll_with(&drop.path().join("celestial.dll"), "v1");
        RUNTIME.install(&fresh.game, &classify(drop.path()).named("first")).unwrap();

        write_plain_dll_with(&drop.path().join("celestial.dll"), "v2 is longer");
        RUNTIME.install(&fresh.game, &classify(drop.path()).named("second")).unwrap();

        let kind_dir = paths::game_mods(&fresh.game);
        assert_eq!(mod_dir_names(&kind_dir), ["first"]);
        let bytes = std::fs::read(kind_dir.join("first").join("celestial.dll")).unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("v2 is longer"));
    }

    #[test]
    fn installed_dlls_are_reported_by_status() {
        let fresh = game();
        let drop = temp("yp_gm_s_");
        write_plain_dll(&drop.path().join("celestial.dll"));

        RUNTIME.install(&fresh.game, &classify(drop.path())).unwrap();
        let status = RUNTIME.status(&fresh.game);

        assert!(status.installed);
        assert!(status.enabled);
        let mods = &status.game_mods_info.as_ref().unwrap().mods;
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].file_name, "celestial.dll");
        assert!(!mods[0].disabled);
        assert!(mods[0].size_bytes.is_some());
    }

    #[test]
    fn an_empty_status_is_reported_when_nothing_is_installed() {
        let fresh = game();

        let status = RUNTIME.status(&fresh.game);

        assert!(!status.installed);
        assert!(status.game_mods_info.unwrap().mods.is_empty());
    }

    #[test]
    fn re_installing_a_changed_dll_is_offered_as_an_update() {
        let fresh = game();
        let drop = temp("yp_gm_u_");
        write_plain_dll_with(&drop.path().join("celestial.dll"), "v1");
        RUNTIME.install(&fresh.game, &classify(drop.path())).unwrap();

        write_plain_dll_with(&drop.path().join("celestial.dll"), "v2 is longer");
        let update = RUNTIME.would_update(&fresh.game, &classify(drop.path()));

        assert_eq!(update.unwrap().runtime, ThirdPartyRuntime::GameMods);
    }

    #[test]
    fn an_identical_dll_is_not_offered_as_an_update() {
        let fresh = game();
        let drop = temp("yp_gm_nu_");
        write_plain_dll(&drop.path().join("celestial.dll"));
        let c = classify(drop.path());
        RUNTIME.install(&fresh.game, &c).unwrap();

        assert_eq!(RUNTIME.would_update(&fresh.game, &c), None);
    }

    #[test]
    fn a_classification_without_dlls_is_rejected() {
        let fresh = game();
        let c = yp_thirdparty::Classification::new(ThirdPartyKind::GameMod, PathBuf::from("nowhere"));

        let result = RUNTIME.install(&fresh.game, &c).unwrap();

        assert_eq!(result.failure(), Some(yp_thirdparty::InstallFailure::NoDll));
    }
}

mod per_dll_disable_via_game_toml {
    use super::*;

    #[test]
    fn disabling_writes_a_per_dll_section_and_shows_up_in_status() {
        let fresh = installed();

        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", true).unwrap();

        let config = game_toml(&fresh.game);
        assert_eq!(section(&config, "celestial.dll")["disabled"].as_bool(), Some(true));
        let status = RUNTIME.status(&fresh.game);
        let info = status.game_mods_info.unwrap();
        assert!(info.mods[0].disabled);
        assert_eq!(info.enabled_count(), 0);
        assert!(
            paths::game_mods(&fresh.game).join("celestial").join("celestial.dll").is_file(),
            "disabling must not delete the DLL"
        );
    }

    #[test]
    fn re_enabling_flips_the_flag_back() {
        let fresh = installed();

        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", true).unwrap();
        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", false).unwrap();

        assert!(!RUNTIME.status(&fresh.game).game_mods_info.unwrap().mods[0].disabled);
    }

    #[test]
    fn an_existing_game_toml_keeps_its_other_entries_and_comments() {
        let fresh = installed();
        write(
            &RUNTIME.config_path(&fresh.game),
            "# keep me\n[defaults]\nsize_of_image = 26177536\n\n[\"other.dll\"]\ndisabled = true\n",
        );

        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", true).unwrap();

        let raw = std::fs::read_to_string(RUNTIME.config_path(&fresh.game)).unwrap();
        let config: toml::Table = toml::from_str(&raw).unwrap();
        assert!(raw.contains("# keep me"));
        assert_eq!(section(&config, "defaults")["size_of_image"].as_integer(), Some(26_177_536));
        assert_eq!(section(&config, "other.dll")["disabled"].as_bool(), Some(true));
        assert_eq!(section(&config, "celestial.dll")["disabled"].as_bool(), Some(true));
    }

    #[test]
    fn defaults_apply_to_a_dll_without_its_own_section_like_nams() {
        let fresh = installed();
        write(&RUNTIME.config_path(&fresh.game), "[defaults]\ndisabled = true\nsize_of_image = 0\n");

        let status = RUNTIME.status(&fresh.game);
        let entry = &status.game_mods_info.as_ref().unwrap().mods[0];

        assert!(entry.disabled, "NAMS for_dll falls back to [defaults] for every field");
        assert_eq!(entry.settings.unwrap().size_of_image, 0);
    }

    #[test]
    fn a_new_section_starts_from_defaults_so_nothing_else_changes() {
        let fresh = installed();
        write(&RUNTIME.config_path(&fresh.game), "[defaults]\nfake_exe_path = false\nsize_of_image = 0\n");

        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", true).unwrap();

        let config = game_toml(&fresh.game);
        let celestial = section(&config, "celestial.dll");
        assert_eq!(celestial["disabled"].as_bool(), Some(true));
        assert_eq!(celestial["fake_exe_path"].as_bool(), Some(false));
        assert_eq!(celestial["size_of_image"].as_integer(), Some(0));
        assert_eq!(celestial["fake_module_handle"].as_bool(), Some(true));
    }

    #[test]
    fn every_game_toml_field_round_trips_through_the_settings() {
        let fresh = installed();
        let settings = GameModSettings {
            fake_module_handle: false,
            fake_exe_path: false,
            size_of_image: GameModSettings::STEAM_2026_SIZE_OF_IMAGE,
            report_patches: true,
            wait_for_patch_phase: true,
            patch_phase_timeout_ms: 30_000,
            ..GameModSettings::default()
        };

        assert!(RUNTIME.set_mod_settings(&fresh.game, "celestial.dll", &settings).unwrap());

        let config = game_toml(&fresh.game);
        let expected: toml::Table = toml::from_str(
            "fake_module_handle = false\nfake_exe_path = false\nsize_of_image = 25944064\ndisabled = false\n\
             report_patches = true\nwait_for_patch_phase = true\npatch_phase_timeout_ms = 30000\n",
        )
        .unwrap();
        assert_eq!(section(&config, "celestial.dll"), &expected);
        let read = RUNTIME.status(&fresh.game).game_mods_info.unwrap().mods[0].settings.unwrap();
        assert_eq!(read.size_of_image, GameModSettings::STEAM_2026_SIZE_OF_IMAGE);
        assert!(read.report_patches);
        assert!(!read.fake_module_handle);
        assert!(read.wait_for_patch_phase);
        assert_eq!(read.patch_phase_timeout_ms, 30_000);
    }

    #[test]
    fn patch_phase_wait_defaults_match_nams() {
        let defaults = GameModSettings::default();
        assert!(!defaults.wait_for_patch_phase);
        assert_eq!(defaults.patch_phase_timeout_ms, 10_000);
        let from_file = GameModSettings::from_toml(&toml::from_str("wait_for_patch_phase = true").unwrap());
        assert!(from_file.wait_for_patch_phase);
        assert_eq!(from_file.patch_phase_timeout_ms, 10_000);
    }

    #[test]
    fn keys_the_launcher_does_not_show_are_kept() {
        let fresh = installed();
        write(
            &RUNTIME.config_path(&fresh.game),
            "[\"celestial.dll\"]\nwait_for_patch_phase = true\nsome_future_key = 7\n",
        );

        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", true).unwrap();

        let config = game_toml(&fresh.game);
        let celestial = section(&config, "celestial.dll");
        assert_eq!(celestial["some_future_key"].as_integer(), Some(7));
        assert_eq!(celestial["wait_for_patch_phase"].as_bool(), Some(true));
        assert_eq!(celestial["disabled"].as_bool(), Some(true));
    }

    #[test]
    fn a_case_mismatched_section_is_updated_not_duplicated() {
        let fresh = installed();
        write(&RUNTIME.config_path(&fresh.game), "[\"CELESTIAL.DLL\"]\ndisabled = true\n");

        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", false).unwrap();

        let config = game_toml(&fresh.game);
        assert_eq!(config.keys().collect::<Vec<_>>(), ["CELESTIAL.DLL"]);
        assert_eq!(section(&config, "CELESTIAL.DLL")["disabled"].as_bool(), Some(false));
    }

    #[test]
    fn a_game_toml_that_does_not_parse_is_left_alone() {
        let fresh = installed();
        let broken = "[\"celestial.dll\"\ndisabled = = true\n";
        write(&RUNTIME.config_path(&fresh.game), broken);

        assert!(!RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", true).unwrap());
        assert_eq!(std::fs::read_to_string(RUNTIME.config_path(&fresh.game)).unwrap(), broken);
    }

    #[test]
    fn a_case_mismatched_section_still_matches_the_dll() {
        let fresh = installed();
        write(&RUNTIME.config_path(&fresh.game), "[\"CELESTIAL.DLL\"]\ndisabled = true\n");

        assert!(RUNTIME.status(&fresh.game).game_mods_info.unwrap().mods[0].disabled);
    }
}

mod removal {
    use super::*;

    #[test]
    fn removing_one_mod_folder_leaves_the_others_alone() {
        let fresh = game();
        let drop_a = temp("yp_gm_r_");
        let drop_b = temp("yp_gm_r_");
        write_plain_dll(&drop_a.path().join("celestial.dll"));
        write_plain_dll(&drop_b.path().join("other.dll"));
        RUNTIME.install(&fresh.game, &classify(drop_a.path())).unwrap();
        RUNTIME.install(&fresh.game, &classify(drop_b.path())).unwrap();

        RUNTIME.remove_mod(&fresh.game, "celestial");

        let status = RUNTIME.status(&fresh.game);
        let names: Vec<_> = status.game_mods_info.unwrap().mods.into_iter().map(|entry| entry.file_name).collect();
        assert_eq!(names, ["other.dll"]);
    }

    #[test]
    fn a_name_that_is_not_a_single_folder_removes_nothing() {
        let fresh = installed();

        RUNTIME.remove_mod(&fresh.game, "");
        RUNTIME.remove_mod(&fresh.game, "..");

        assert!(RUNTIME.status(&fresh.game).installed);
    }

    #[test]
    fn removing_the_runtime_clears_the_dlls_and_sets_the_disable_flag() {
        let fresh = installed();

        RUNTIME.remove(&fresh.game).unwrap();

        assert!(!RUNTIME.status(&fresh.game).installed);
        assert_eq!(nams_flag(&fresh.game, LoadingFlag::GameMods.key()), Some(true));
    }

    #[test]
    fn game_toml_survives_a_runtime_removal() {
        let fresh = installed();
        RUNTIME.set_mod_disabled(&fresh.game, "celestial.dll", true).unwrap();

        RUNTIME.remove(&fresh.game).unwrap();

        assert!(RUNTIME.config_path(&fresh.game).is_file(), "per-DLL settings should outlive the DLLs themselves");
    }
}

mod graphic_proxies {
    use super::*;

    const GRAPHIC: GraphicModsRuntime = GraphicModsRuntime;

    fn installed_proxy() -> FreshGame {
        let fresh = game();
        let drop = temp("yp_gx_");
        write_plain_dll(&drop.path().join("d3d11.dll"));
        write(&drop.path().join("d3d11.ini"), "x=1");
        let result = GRAPHIC.install(&fresh.game, &classify(drop.path())).unwrap();
        assert!(result.is_ok());
        fresh
    }

    #[test]
    fn install_puts_the_proxy_and_its_files_in_one_mod_folder() {
        let fresh = installed_proxy();

        let dir = paths::graphic_mods(&fresh.game).join("d3d11");
        assert!(dir.join("d3d11.dll").is_file());
        assert!(dir.join("d3d11.ini").is_file());
        assert_eq!(paths::find_dll(&paths::game_mods(&fresh.game), "d3d11.dll"), None);
        assert_eq!(nams_flag(&fresh.game, LoadingFlag::GraphicMods.key()), Some(false));
        let status = GRAPHIC.status(&fresh.game);
        assert!(status.installed);
        let mods = status.graphic_mods_info.unwrap().mods;
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].file_name, "d3d11.dll");
        assert_eq!(mods[0].mod_dir, "d3d11");
    }

    #[test]
    fn disabling_moves_the_mod_folder_where_nams_does_not_look() {
        let fresh = installed_proxy();

        GRAPHIC.set_mod_disabled(&fresh.game, "d3d11", true);

        assert!(
            !paths::graphic_mods(&fresh.game).join("d3d11").exists(),
            "NAMS loads every mod folder in nams/thirdparty/graphic"
        );
        assert!(
            paths::graphic_mods_disabled(&fresh.game).join("d3d11").join("d3d11.ini").is_file(),
            "the whole mod folder moves, not only the DLL"
        );
        assert!(GRAPHIC.status(&fresh.game).graphic_mods_info.unwrap().mods[0].disabled);

        GRAPHIC.set_mod_disabled(&fresh.game, "d3d11", false);
        assert!(paths::graphic_mods(&fresh.game).join("d3d11").join("d3d11.dll").is_file());
    }

    #[test]
    fn remove_clears_enabled_and_parked_mods_and_sets_the_flag() {
        let fresh = installed_proxy();
        GRAPHIC.set_mod_disabled(&fresh.game, "d3d11", true);

        GRAPHIC.remove(&fresh.game).unwrap();

        assert!(!GRAPHIC.status(&fresh.game).installed);
        assert_eq!(nams_flag(&fresh.game, LoadingFlag::GraphicMods.key()), Some(true));
    }

    #[test]
    fn a_parked_mod_is_revived_by_a_reinstall() {
        let fresh = installed_proxy();
        GRAPHIC.set_mod_disabled(&fresh.game, "d3d11", true);
        let drop = temp("yp_gx_again_");
        write_plain_dll_with(&drop.path().join("d3d11.dll"), "newer build");

        GRAPHIC.install(&fresh.game, &classify(drop.path())).unwrap();

        let revived = paths::graphic_mods(&fresh.game).join("d3d11");
        assert!(revived.join("d3d11.ini").is_file());
        assert!(!paths::graphic_mods_disabled(&fresh.game).join("d3d11").exists());
    }

    #[test]
    fn the_graphic_dirs_match_what_nams_scans() {
        let game = GamePaths::new("C:/Games/NieRAutomata");
        let dir = paths::graphic_mods(&game);
        assert_eq!(dir.file_name().unwrap(), "graphic");
        assert_eq!(dir.parent().unwrap().file_name().unwrap(), THIRD_PARTY_DIR_NAME);
        assert_eq!(dir.parent().unwrap().parent().unwrap().file_name().unwrap(), "nams");
        assert!(
            !paths::graphic_mods_disabled(&game).starts_with(&dir),
            "a turned-off mod must stay out of the folder NAMS loads"
        );
    }
}

#[test]
fn the_install_dir_matches_what_nams_scans() {
    let game = GamePaths::new("C:/Games/NieRAutomata");
    let dir = paths::game_mods(&game);
    assert_eq!(dir.file_name().unwrap(), GAME_MODS_DIR_NAME);
    assert_eq!(dir.parent().unwrap().file_name().unwrap(), THIRD_PARTY_DIR_NAME);
    assert_eq!(dir.parent().unwrap().parent().unwrap(), Path::new("C:/Games/NieRAutomata").join("nams"));
}
