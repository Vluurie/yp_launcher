mod common;

use std::path::PathBuf;

use common::{temp, write};
use tempfile::TempDir;
use yp_core::{AppPaths, GamePaths};
use yp_thirdparty::migration::migrate_from;
use yp_thirdparty::{migoto, paths, reshade, GameModsRuntime, GraphicModsRuntime, GraphicsRuntime};

struct Setup {
    _root: TempDir,
    legacy: PathBuf,
    game: GamePaths,
}

fn setup() -> Setup {
    let root = temp("yp_tp_migrate_");
    let launcher = root.path().join("launcher");
    std::fs::create_dir_all(&launcher).unwrap();
    let game_dir = root.path().join("game");
    std::fs::create_dir_all(game_dir.join("nams")).unwrap();
    Setup {
        legacy: paths::legacy_root(&launcher),
        game: GamePaths::new(game_dir),
        _root: root,
    }
}

#[test]
fn nothing_to_do_without_the_old_folder() {
    let setup = setup();

    let result = migrate_from(&setup.game, &setup.legacy, false);

    assert_eq!(result.moved_files, 0);
    assert!(result.left_behind.is_empty());
}

#[test]
fn every_kind_lands_in_mod_folders_and_the_old_folder_is_gone() {
    let setup = setup();
    let legacy = &setup.legacy;
    write(&legacy.join("game").join("celestial.dll"), "celestial");
    write(&legacy.join("game").join("celestial.ini"), "x=1");
    write(&legacy.join("game").join("wax.dll"), "wax");
    write(&legacy.join("game").join("game.toml"), "[\"wax.dll\"]\ndisabled = true\n");
    write(&legacy.join("graphic").join("d3d11.dll"), "dlaa");
    write(&legacy.join("graphic").join("nvngx_dlss.dll"), "dlss");
    write(&legacy.join("graphic").join("NIER_DLAA.ini"), "x=1");
    write(&legacy.join("graphic").join("disabled").join("dxgi.dll"), "parked");
    write(&legacy.join("reshade").join("ReShade64.dll"), "reshade");
    write(&legacy.join("reshade").join("reshade-shaders").join("Shaders").join("a.fx"), "fx");
    write(&legacy.join("3dmigoto").join("3dmigoto.dll"), "migoto");
    write(&legacy.join("3dmigoto").join("nvapi64.dll"), "nvapi");

    let result = migrate_from(&setup.game, legacy, false);

    assert!(result.left_behind.is_empty(), "{:?}", result.left_behind);
    assert_eq!(result.moved_files, 12);
    assert!(!legacy.exists());

    let game_mods = paths::game_mods(&setup.game);
    assert!(game_mods.join("celestial").join("celestial.dll").is_file());
    assert!(game_mods.join("celestial").join("celestial.ini").is_file());
    assert!(game_mods.join("wax").join("wax.dll").is_file());
    assert!(game_mods.join("game.toml").is_file(), "game.toml stays beside the mod folders");
    let game_status = GameModsRuntime.status(&setup.game);
    let wax = game_status.game_mods_info.unwrap().mods.into_iter().find(|entry| entry.file_name == "wax.dll").unwrap();
    assert!(wax.disabled);

    let graphic = paths::graphic_mods(&setup.game).join("d3d11+dxgi");
    for name in ["d3d11.dll", "nvngx_dlss.dll", "NIER_DLAA.ini"] {
        assert!(graphic.join(name).is_file(), "{name}");
    }
    assert!(
        paths::graphic_mods_disabled(&setup.game).join("d3d11+dxgi").join("dxgi.dll").is_file(),
        "a turned-off DLL stays turned off"
    );
    let graphic_mods = GraphicModsRuntime.status(&setup.game).graphic_mods_info.unwrap().mods;
    let disabled: Vec<_> = graphic_mods.iter().filter(|entry| entry.disabled).map(|entry| entry.file_name.as_str()).collect();
    assert_eq!(disabled, ["dxgi.dll"]);

    let reshade_dir = reshade::install_dir(&setup.game);
    assert!(reshade_dir.join("ReShade64.dll").is_file());
    assert!(reshade_dir.join("reshade-shaders").join("Shaders").join("a.fx").is_file());

    let migoto_dir = migoto::install_dir(&setup.game);
    assert!(migoto_dir.join("3dmigoto.dll").is_file());
    assert!(migoto_dir.join("nvapi64.dll").is_file());
}

#[test]
fn a_file_whose_new_copy_differs_is_kept_in_the_old_folder() {
    let setup = setup();
    write(&setup.legacy.join("game").join("celestial.dll"), "old build");
    let newer = paths::game_mods(&setup.game).join("Celestial").join("celestial.dll");
    write(&newer, "new build");

    let result = migrate_from(&setup.game, &setup.legacy, false);

    let old = setup.legacy.join("game").join("celestial.dll");
    assert_eq!(result.left_behind, std::slice::from_ref(&old));
    assert!(yp_thirdparty::ThirdPartyNotice::from_migration(&result).is_empty());
    assert_eq!(std::fs::read_to_string(&old).unwrap(), "old build");
    assert_eq!(std::fs::read_to_string(&newer).unwrap(), "new build");
}

#[test]
fn an_identical_copy_in_the_new_folder_completes_the_move() {
    let setup = setup();
    write(&setup.legacy.join("game").join("celestial.dll"), "same");
    write(&paths::game_mods(&setup.game).join("celestial").join("celestial.dll"), "same");

    let result = migrate_from(&setup.game, &setup.legacy, false);

    assert!(result.left_behind.is_empty());
    assert!(!setup.legacy.exists());
}

#[test]
fn loose_files_stay_when_they_belong_to_none_of_several_mods() {
    let setup = setup();
    write(&setup.legacy.join("game").join("a.dll"), "a");
    write(&setup.legacy.join("game").join("b.dll"), "b");
    write(&setup.legacy.join("game").join("notes.txt"), "mine");

    let result = migrate_from(&setup.game, &setup.legacy, false);

    let notes = setup.legacy.join("game").join("notes.txt");
    assert_eq!(result.left_behind, std::slice::from_ref(&notes));
    assert!(notes.is_file());
    let game_mods = paths::game_mods(&setup.game);
    assert!(game_mods.join("a").join("a.dll").is_file());
    assert!(game_mods.join("b").join("b.dll").is_file());
}

#[test]
fn a_running_game_blocks_the_move() {
    let setup = setup();
    write(&setup.legacy.join("game").join("celestial.dll"), "celestial");

    let result = migrate_from(&setup.game, &setup.legacy, true);

    assert_eq!(result.moved_files, 0);
    assert!(setup.legacy.join("game").join("celestial.dll").is_file());
}

#[test]
fn the_legacy_folder_is_the_one_next_to_nams() {
    let root = temp("yp_tp_app_");
    let app = AppPaths::for_test(root.path());
    let game = GamePaths::new(root.path().join("game"));
    write(&app.launcher_thirdparty_dir().join("game").join("solo.dll"), "solo");

    let result = yp_thirdparty::migrate_legacy(&game, &app, false);

    assert_eq!(result.from, app.runtime_dir.join("thirdparty"));
    assert_eq!(result.to, game.thirdparty_dir());
    assert_eq!(result.moved_files, 1);
    assert!(paths::game_mods(&game).join("solo").join("solo.dll").is_file());
}
