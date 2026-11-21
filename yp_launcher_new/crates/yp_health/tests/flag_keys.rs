use yp_thirdparty::{LoadingFlag, ThirdPartyRuntime};

#[test]
fn flags_map_to_the_nams_toml_keys_the_settings_screen_edits() {
    let expected = [
        (LoadingFlag::ReShade, "disable_reshade_loading"),
        (LoadingFlag::Migoto, "disable_3dmigoto_loading"),
        (LoadingFlag::GameMods, "disable_game_mods_loading"),
        (LoadingFlag::GraphicMods, "disable_graphic_mods_loading"),
    ];
    assert_eq!(expected.len(), LoadingFlag::ALL.len());
    for (flag, key) in expected {
        assert_eq!(flag.key(), key);
    }
    for runtime in ThirdPartyRuntime::ALL {
        assert!(LoadingFlag::ALL.contains(&runtime.loading_flag()));
    }
}
