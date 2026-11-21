mod support;

use serde_json::json;
use support::{FakeCli, GameDir};
use yp_config::legacy::lodmod_ini::parse_ini_value;
use yp_config::legacy::{detect_lodmod_ini, import_lodmod_ini, parse_lodmod_ini};

const LODMOD_INI: &str = "[LodMod]
; Automata-LodMod settings
LODMultiplier = 0 ; best quality
ShadowResolution=4096
ShadowDistanceMultiplier = 2.5
ShadowModelHQ = true
DisableVignette = FALSE
UnrelatedSetting = 7
#ShadowDistancePSS = 1500

[Extra]
ShadowFilterStrengthBias = -0.5
ShadowResolution = 8192
";

#[test]
fn only_known_keys_are_parsed_and_typed() {
    let values = parse_lodmod_ini(LODMOD_INI);
    let keys: Vec<&str> = values.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        vec![
            "LODMultiplier",
            "ShadowResolution",
            "ShadowDistanceMultiplier",
            "ShadowModelHQ",
            "DisableVignette",
            "ShadowFilterStrengthBias",
        ]
    );
    assert_eq!(values["LODMultiplier"], json!(0));
    assert_eq!(values["ShadowResolution"], json!(8192));
    assert_eq!(values["ShadowDistanceMultiplier"], json!(2.5));
    assert_eq!(values["ShadowModelHQ"], json!(true));
    assert_eq!(values["DisableVignette"], json!(false));
    assert_eq!(values["ShadowFilterStrengthBias"], json!(-0.5));
}

#[test]
fn ini_values_follow_dart_number_rules() {
    assert_eq!(parse_ini_value("0x10"), json!(16));
    assert_eq!(parse_ini_value("-7"), json!(-7));
    assert_eq!(parse_ini_value("1.0"), json!(1.0));
    assert_eq!(parse_ini_value("1e3"), json!(1000.0));
    assert_eq!(parse_ini_value("high"), json!("high"));
    assert_eq!(parse_ini_value("TRUE"), json!(true));
}

#[test]
fn detection_needs_a_file_with_known_keys() {
    let game = GameDir::empty();
    assert!(detect_lodmod_ini(&game.game).unwrap().is_none());
    std::fs::write(game.root().join("LodMod.ini"), "[LodMod]\nUnknown = 1\n").unwrap();
    assert!(detect_lodmod_ini(&game.game).unwrap().is_none());
    std::fs::write(game.root().join("LodMod.ini"), LODMOD_INI).unwrap();
    assert_eq!(detect_lodmod_ini(&game.game).unwrap().unwrap().len(), 6);
}

#[test]
fn import_enables_lodmod_and_sets_the_mapped_settings() {
    let game = GameDir::empty();
    std::fs::write(game.root().join("LodMod.ini"), LODMOD_INI).unwrap();
    let values = detect_lodmod_ini(&game.game).unwrap().unwrap();
    let cli = FakeCli::with(&[("lodmod.enabled", json!(false))]);

    assert!(import_lodmod_ini(&cli, &values).unwrap());
    assert_eq!(cli.value("lodmod.enabled"), Some(json!(true)));
    assert_eq!(cli.value("lodmod.lod_multiplier"), Some(json!(0)));
    assert_eq!(cli.value("lodmod.shadow_resolution"), Some(json!(8192)));
    assert_eq!(cli.value("lodmod.shadow_distance_multiplier"), Some(json!(2.5)));
    assert_eq!(cli.value("lodmod.shadow_model_hq"), Some(json!(true)));
    assert_eq!(cli.value("lodmod.shadow_filter_strength_bias"), Some(json!(-0.5)));

    assert!(!import_lodmod_ini(&cli, &values).unwrap(), "an enabled LodMod is left alone");
    assert_eq!(cli.set_calls(), 1);
}

#[test]
fn a_value_nams_rejects_is_left_out_and_the_rest_is_imported() {
    let values = parse_lodmod_ini(LODMOD_INI);
    let cli = FakeCli::default().rejecting(&["lodmod.shadow_resolution"]);
    assert!(import_lodmod_ini(&cli, &values).unwrap());
    assert_eq!(cli.value("lodmod.shadow_resolution"), None);
    assert_eq!(cli.value("lodmod.shadow_model_hq"), Some(json!(true)));
}
