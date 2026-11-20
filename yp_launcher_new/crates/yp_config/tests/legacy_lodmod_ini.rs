mod support;

use support::GameDir;
use yp_config::legacy::lodmod_ini::parse_ini_value;
use yp_config::legacy::{detect_lodmod_ini, migrate_lodmod_ini, parse_lodmod_ini};

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
    assert_eq!(values["LODMultiplier"], toml::Value::Integer(0));
    assert_eq!(values["ShadowResolution"], toml::Value::Integer(8192));
    assert_eq!(values["ShadowDistanceMultiplier"], toml::Value::Float(2.5));
    assert_eq!(values["ShadowModelHQ"], toml::Value::Boolean(true));
    assert_eq!(values["DisableVignette"], toml::Value::Boolean(false));
    assert_eq!(values["ShadowFilterStrengthBias"], toml::Value::Float(-0.5));
}

#[test]
fn ini_values_follow_dart_number_rules() {
    assert_eq!(parse_ini_value("0x10"), toml::Value::Integer(16));
    assert_eq!(parse_ini_value("-7"), toml::Value::Integer(-7));
    assert_eq!(parse_ini_value("1.0"), toml::Value::Float(1.0));
    assert_eq!(parse_ini_value("1e3"), toml::Value::Float(1000.0));
    assert_eq!(parse_ini_value("high"), toml::Value::String("high".into()));
    assert_eq!(parse_ini_value("TRUE"), toml::Value::Boolean(true));
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
fn migration_enables_lodmod_and_copies_mapped_values() {
    let game = GameDir::empty();
    std::fs::write(game.root().join("LodMod.ini"), LODMOD_INI).unwrap();
    let values = detect_lodmod_ini(&game.game).unwrap().unwrap();
    assert!(
        !migrate_lodmod_ini(&game.game, &values).unwrap(),
        "no lodmod.toml yet"
    );

    game.ensure();
    assert!(migrate_lodmod_ini(&game.game, &values).unwrap());
    let raw = game.read_nams_file("lodmod.toml");
    let config = support::parse(&raw);
    assert_eq!(config["enabled"].as_bool(), Some(true));
    assert_eq!(config["lod_multiplier"].as_integer(), Some(0));
    assert_eq!(config["shadow_resolution"].as_integer(), Some(8192));
    assert_eq!(config["shadow_distance_multiplier"].as_float(), Some(2.5));
    assert_eq!(config["shadow_model_hq"].as_bool(), Some(true));
    assert_eq!(config["shadow_filter_strength_bias"].as_float(), Some(-0.5));
    assert!(
        raw.contains("# Shadow texture size."),
        "comments survive the import"
    );

    assert!(
        !migrate_lodmod_ini(&game.game, &values).unwrap(),
        "an enabled LodMod is left alone"
    );
}
