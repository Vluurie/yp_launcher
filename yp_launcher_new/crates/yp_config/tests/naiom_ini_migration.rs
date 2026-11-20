mod support;

use support::{GameDir, table};
use yp_config::legacy::{SkippedNaiomSetting, detect_naiom_ini, migrate_naiom_ini};

const FULL_INI: &str = "
;===================================================================================================
; --- NAIOM - Configuration File ---
;===================================================================================================

; Type: float
ThirdPersonSensitivityX 1.500000

; Type: float
ThirdPersonSensitivityY -0.750000

; Type: float
AimSensitivityX 2.000000

; Type: float
AimSensitivityY 1.000000

; Type: bool
MovementDisableTapEvade 1

; Type: bool
MiscDisablePodPet 1

; Type: bool
MiscEnableCustomCursors 0

; Type: binding
configReload <unbound>

; Type: bool
soundsEnabled 1

; <items>;Disabled;Fix Disabled;RawInput;Raw Input;RawInputCharFollow;Raw Input Char Follow
; Type: string
ThirdPersonMode RawInputCharFollow

; <items>;Disabled;Fix Disabled;RawInputOnly;Raw Input Only;Crosshair;Raw Input Crosshair
; Type: string
AimMode Crosshair

; Type: binding
ThirdPersonModeToggle F5

; Type: binding
AimModeToggle <unbound>

; Type: binding
StandardMoveForward W

; Type: binding
StandardMoveBackward CAPSLOCK

; Type: binding
StandardJump SPACE

; Type: binding
StandardFire MOUSE1

; Type: binding
StandardWalk MOUSE2

; Type: binding
NonStandardEvade LSHIFT

; Type: binding
NonStandardUseItem lctrl+x

; Type: binding
NonStandardAutoFire CONTROLLER_A

; Type: binding
MiscOpenDebugMenu K
";

fn game_with_ini(ini: &str) -> GameDir {
    let game = GameDir::empty();
    std::fs::write(game.root().join("NAIOM.ini"), ini).unwrap();
    game
}

#[test]
fn parses_the_space_separated_ini_format() {
    let game = game_with_ini(FULL_INI);
    let parsed = detect_naiom_ini(&game.game).unwrap().unwrap();
    assert_eq!(parsed["thirdpersonsensitivityx"], "1.500000");
    assert_eq!(parsed["thirdpersonmode"], "RawInputCharFollow");
    assert_eq!(parsed["nonstandardevade"], "LSHIFT");
    assert!(parsed.contains_key("configreload"));
}

#[test]
fn imports_values_into_naiom_toml_with_correct_mappings() {
    let game = game_with_ini(FULL_INI);
    game.ensure();
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let result = migrate_naiom_ini(&game.game, &ini).unwrap();
    assert!(result.migrated);

    let mouse = game.naiom();
    let bindings = table(&mouse, "bindings");

    assert_eq!(mouse["third_person_sensitivity_x"].as_float(), Some(1.5));
    assert_eq!(mouse["third_person_sensitivity_y"].as_float(), Some(-0.75));
    assert_eq!(mouse["aim_sensitivity_x"].as_float(), Some(2.0));
    assert_eq!(mouse["movement_disable_tap_evade"].as_bool(), Some(true));
    assert_eq!(mouse["misc_disable_pod_pet"].as_bool(), Some(true));
    assert_eq!(
        mouse["third_person_mode"].as_bool(),
        Some(true),
        "RawInputCharFollow enables the camera fix"
    );
    assert_eq!(mouse["third_person_char_follow"].as_bool(), Some(true));
    assert_eq!(
        mouse["aim_mode"].as_bool(),
        Some(true),
        "Crosshair enables aim fix"
    );
    assert_eq!(mouse["aim_crosshair"].as_bool(), Some(true));
    assert_eq!(
        mouse["misc_disable_default_cursor"].as_bool(),
        Some(true),
        "MiscEnableCustomCursors 0 keeps the system cursor"
    );
    assert_eq!(
        mouse["misc_open_debug_menu"].as_integer(),
        Some(0x4B),
        "binding K becomes VK code 0x4B"
    );

    assert_eq!(bindings["standard_move_forward"].as_str(), Some("W"));
    assert_eq!(bindings["standard_jump"].as_str(), Some("SPACE"));
    assert_eq!(
        bindings["standard_fire"].as_str(),
        Some("MOUSE1"),
        "mouse buttons are allowed on standard_fire"
    );
    assert_eq!(bindings["non_standard_evade"].as_str(), Some("LSHIFT"));
    assert_eq!(
        bindings["non_standard_use_item"].as_str(),
        Some("LCTRL+X"),
        "combinations are normalized to uppercase with +"
    );
    assert_eq!(
        bindings["aim_mode_toggle"].as_str(),
        Some(""),
        "<unbound> stays unbound"
    );
}

#[test]
fn skips_unsupported_keys_and_reports_them_by_name() {
    let game = game_with_ini(FULL_INI);
    game.ensure();
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let result = migrate_naiom_ini(&game.game, &ini).unwrap();
    let skipped: Vec<String> = result.skipped.iter().map(ToString::to_string).collect();

    assert!(
        skipped.iter().any(|entry| entry.contains("CAPSLOCK")),
        "CAPSLOCK is not a valid NAMS key name"
    );
    assert!(
        !skipped.iter().any(|entry| entry.contains("F5")),
        "function keys are valid NAMS key names"
    );
    assert!(
        !skipped.iter().any(|entry| entry.contains("CONTROLLER_A")),
        "controller buttons are valid on mouse-capable bindings"
    );
    assert!(
        skipped.iter().any(|entry| entry.contains("MOUSE2")),
        "standard_walk is keyboard-only, MOUSE2 would be a no-op"
    );
    assert!(skipped.contains(&"Standard Move Backward (CAPSLOCK)".to_owned()));
    assert!(result.skipped.iter().any(|entry| {
        entry.setting == SkippedNaiomSetting::Binding("standard_walk") && entry.value == "MOUSE2"
    }));

    let mouse = game.naiom();
    let bindings = table(&mouse, "bindings");
    assert_eq!(bindings["third_person_mode_toggle"].as_str(), Some("F5"));
    assert_eq!(bindings["standard_move_backward"].as_str(), Some(""));
    assert_eq!(
        bindings["non_standard_auto_fire"].as_str(),
        Some("CONTROLLER_A")
    );
    assert_eq!(bindings["standard_walk"].as_str(), Some(""));
}

#[test]
fn second_run_does_not_overwrite_existing_settings() {
    let game = game_with_ini(FULL_INI);
    game.ensure();
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    migrate_naiom_ini(&game.game, &ini).unwrap();

    let content =
        game.raw_naiom()
            .replacen("standard_jump = \"SPACE\"", "standard_jump = \"E\"", 1);
    game.write_nams_file("naiom.toml", &content);

    let second = migrate_naiom_ini(&game.game, &ini).unwrap();
    assert!(
        !second.migrated,
        "guard must refuse once bindings are configured"
    );
    assert_eq!(
        table(&game.naiom(), "bindings")["standard_jump"].as_str(),
        Some("E"),
        "user changes must survive"
    );
}

#[test]
fn returns_none_when_no_naiom_files_exist() {
    let game = GameDir::empty();
    assert!(detect_naiom_ini(&game.game).unwrap().is_none());
}

#[test]
fn falls_back_to_the_cfg_file_and_skips_empty_ini_files() {
    let game = GameDir::empty();
    std::fs::write(game.root().join("NAIOM.ini"), "; only comments\n").unwrap();
    std::fs::write(game.root().join("NAIOM.cfg"), "AimMode Disabled\r\n").unwrap();
    let parsed = detect_naiom_ini(&game.game).unwrap().unwrap();
    assert_eq!(parsed["aimmode"], "Disabled");
}

#[test]
fn an_unknown_debug_menu_key_is_reported() {
    let game = game_with_ini("MiscOpenDebugMenu PAUSE\nAimMode Disabled\n");
    game.ensure();
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let result = migrate_naiom_ini(&game.game, &ini).unwrap();
    assert!(result.migrated);
    assert_eq!(result.skipped.len(), 1);
    assert_eq!(result.skipped[0].setting, SkippedNaiomSetting::DebugMenuKey);
    assert_eq!(result.skipped[0].to_string(), "MiscOpenDebugMenu (PAUSE)");
    assert_eq!(game.naiom()["aim_mode"].as_bool(), Some(false));
}
