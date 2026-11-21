mod support;

use serde_json::json;
use support::{FakeCli, GameDir};
use yp_config::legacy::{SkippedNaiomSetting, detect_naiom_ini, import_naiom_ini};

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
fn imports_values_into_naiom_settings_with_correct_mappings() {
    let game = game_with_ini(FULL_INI);
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let cli = FakeCli::default();
    let result = import_naiom_ini(&cli, &ini).unwrap();
    assert!(result.migrated);

    assert_eq!(cli.value("naiom.third_person_sensitivity_x"), Some(json!(1.5)));
    assert_eq!(cli.value("naiom.third_person_sensitivity_y"), Some(json!(-0.75)));
    assert_eq!(cli.value("naiom.aim_sensitivity_x"), Some(json!(2.0)));
    assert_eq!(cli.value("naiom.movement_disable_tap_evade"), Some(json!(true)));
    assert_eq!(cli.value("naiom.misc_disable_pod_pet"), Some(json!(true)));
    assert_eq!(
        cli.value("naiom.third_person_mode"),
        Some(json!(true)),
        "RawInputCharFollow enables the camera fix"
    );
    assert_eq!(cli.value("naiom.third_person_char_follow"), Some(json!(true)));
    assert_eq!(cli.value("naiom.aim_mode"), Some(json!(true)), "Crosshair enables aim fix");
    assert_eq!(cli.value("naiom.aim_crosshair"), Some(json!(true)));
    assert_eq!(
        cli.value("naiom.misc_disable_default_cursor"),
        Some(json!(true)),
        "MiscEnableCustomCursors 0 keeps the system cursor"
    );
    assert_eq!(
        cli.value("naiom.misc_open_debug_menu"),
        Some(json!(0x4B)),
        "binding K becomes VK code 0x4B"
    );

    assert_eq!(cli.value("naiom.bindings.standard_move_forward"), Some(json!("W")));
    assert_eq!(cli.value("naiom.bindings.standard_jump"), Some(json!("SPACE")));
    assert_eq!(
        cli.value("naiom.bindings.standard_fire"),
        Some(json!("MOUSE1")),
        "mouse buttons are allowed on standard_fire"
    );
    assert_eq!(cli.value("naiom.bindings.non_standard_evade"), Some(json!("LSHIFT")));
    assert_eq!(
        cli.value("naiom.bindings.non_standard_use_item"),
        Some(json!("LCTRL+X")),
        "combinations are normalized to uppercase with +"
    );
    assert_eq!(cli.value("naiom.bindings.aim_mode_toggle"), None, "<unbound> is not written");
}

#[test]
fn skips_unsupported_keys_and_reports_them_by_name() {
    let game = game_with_ini(FULL_INI);
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let cli = FakeCli::default();
    let result = import_naiom_ini(&cli, &ini).unwrap();
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

    assert_eq!(cli.value("naiom.bindings.third_person_mode_toggle"), Some(json!("F5")));
    assert_eq!(cli.value("naiom.bindings.standard_move_backward"), None);
    assert_eq!(cli.value("naiom.bindings.non_standard_auto_fire"), Some(json!("CONTROLLER_A")));
    assert_eq!(cli.value("naiom.bindings.standard_walk"), None);
}

#[test]
fn a_binding_nams_rejects_is_reported_and_the_rest_is_imported() {
    let game = game_with_ini(FULL_INI);
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let cli = FakeCli::default().rejecting(&["naiom.bindings.standard_jump"]);
    let result = import_naiom_ini(&cli, &ini).unwrap();
    assert!(result.migrated);
    assert_eq!(cli.value("naiom.bindings.standard_jump"), None);
    assert_eq!(cli.value("naiom.bindings.standard_move_forward"), Some(json!("W")));
    assert!(result.skipped.iter().any(|entry| {
        entry.setting == SkippedNaiomSetting::Binding("standard_jump") && entry.value == "SPACE"
    }));
}

#[test]
fn second_run_does_not_overwrite_existing_settings() {
    let game = game_with_ini(FULL_INI);
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let cli = FakeCli::with(&[("naiom.bindings.standard_jump", json!("E"))]);
    let second = import_naiom_ini(&cli, &ini).unwrap();
    assert!(!second.migrated, "guard must refuse once bindings are configured");
    assert_eq!(
        cli.value("naiom.bindings.standard_jump"),
        Some(json!("E")),
        "user changes must survive"
    );
    assert_eq!(cli.set_calls(), 0);
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
    let game = game_with_ini("MiscOpenDebugMenu PAUSE
AimMode Disabled
");
    let ini = detect_naiom_ini(&game.game).unwrap().unwrap();
    let cli = FakeCli::default();
    let result = import_naiom_ini(&cli, &ini).unwrap();
    assert!(result.migrated);
    assert_eq!(result.skipped.len(), 1);
    assert_eq!(result.skipped[0].setting, SkippedNaiomSetting::DebugMenuKey);
    assert_eq!(result.skipped[0].to_string(), "MiscOpenDebugMenu (PAUSE)");
    assert_eq!(cli.value("naiom.aim_mode"), Some(json!(false)));
}
