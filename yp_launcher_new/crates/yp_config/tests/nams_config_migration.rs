mod support;

use std::collections::BTreeSet;

use support::{GameDir, index_of, parse, table};
use yp_config::fields::{AnyField, bindings, content, nams, texture_injection};
use yp_config::migrate::nams::LEGACY_CONTENT_KEYS;

fn legacy_keys() -> Vec<&'static str> {
    LEGACY_CONTENT_KEYS.iter().map(|(key, _)| *key).collect()
}

fn legacy_mapping(key: &str) -> BTreeSet<&'static str> {
    LEGACY_CONTENT_KEYS
        .iter()
        .find(|(legacy, _)| *legacy == key)
        .map(|(_, features)| features.iter().copied().collect())
        .unwrap()
}

fn legacy_config(overrides: &[(&str, bool)]) -> String {
    let lines: Vec<String> = legacy_keys()
        .into_iter()
        .map(|key| {
            let value = overrides
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| *value)
                .unwrap_or(true);
            format!("{key} = {value}")
        })
        .collect();
    format!(
        "# NAMS General Config. Restart game after changes.

validate_model_data = false
validate_scripts = false
loading_stall_hints = true
disable_plugin_loading = false
disable_content_features = false

# Per-feature content-layer toggles. All default to true. Useful for bisecting
# mod bugs without disabling the entire content layer.
{}

outfit_swap_visual_effects = true
experimental_default_outfits = false

# Mouse input fixes.
[mouse]
sensitivity = 2.0

[cutscene]
hd_cutscenes = true
",
        lines.join("\n")
    )
}

fn expect_content_off(cfg: &toml::Table, off: &BTreeSet<&str>) {
    let content_table = table(cfg, "content");
    for field in content::fields() {
        let expected = !off.contains(field.key());
        assert_eq!(
            content_table
                .get(field.key())
                .and_then(toml::Value::as_bool),
            Some(expected),
            "{} should be {}",
            field.key(),
            if expected { "on" } else { "off" }
        );
    }
}

#[test]
fn content_fields_list_the_35_nams_keys_once_each() {
    let keys: Vec<&str> = content::fields().map(|field| field.key()).collect();
    assert_eq!(keys.len(), 35);
    assert_eq!(keys.iter().collect::<BTreeSet<_>>().len(), 35);
    for field in content::fields() {
        assert_eq!(field.section(), Some("content"));
        assert!(field.default_value());
        assert!(field.meta.restart_required);
    }
}

#[test]
fn fresh_config_has_content_with_every_key_on_and_no_old_keys() {
    let game = GameDir::with_nams(None);
    game.ensure();
    let cfg = game.nams();
    let keys: BTreeSet<&str> = table(&cfg, "content").keys().map(String::as_str).collect();
    let expected: BTreeSet<&str> = content::fields().map(|field| field.key()).collect();
    assert_eq!(keys, expected);
    expect_content_off(&cfg, &BTreeSet::new());
    for key in legacy_keys() {
        assert!(
            !game.raw_nams().contains(key),
            "{key} must not be written any more"
        );
    }
}

#[test]
fn old_content_items_and_content_quest_integration_convert_to_content() {
    let game = GameDir::with_nams(Some(&legacy_config(&[
        ("content_items", false),
        ("content_quest_integration", false),
    ])));
    game.ensure();
    let raw = game.raw_nams();
    let cfg = parse(&raw);

    let mut off = legacy_mapping("content_items");
    off.extend(legacy_mapping("content_quest_integration"));
    expect_content_off(&cfg, &off);

    for key in legacy_keys() {
        assert!(!cfg.contains_key(key), "{key} must be removed");
        assert!(!raw.contains(key), "{key} line must be gone");
    }
    assert!(
        !raw.contains("Per-feature content-layer toggles"),
        "the old comment block must be removed too"
    );

    let content_at = index_of(&raw, "[content]");
    for (key, value) in &cfg {
        if value.is_table() {
            continue;
        }
        assert!(
            index_of(&raw, &format!("{key} =")) < content_at,
            "top-level {key} must stay above [content]"
        );
    }
    assert!(
        content_at < index_of(&raw, "# Mouse input fixes."),
        "[content] goes in front of [mouse] and its comment"
    );
    assert_eq!(table(&cfg, "mouse")["sensitivity"].as_float(), Some(2.0));
    assert_eq!(
        table(&cfg, "cutscene")["hd_cutscenes"].as_bool(),
        Some(true)
    );
}

#[test]
fn each_legacy_key_false_turns_off_exactly_what_nams_turns_off() {
    for (legacy, features) in LEGACY_CONTENT_KEYS {
        let game = GameDir::with_nams(Some(&legacy_config(&[(legacy, false)])));
        game.ensure();
        let off: BTreeSet<&str> = features.iter().copied().collect();
        expect_content_off(&game.nams(), &off);
    }
}

#[test]
fn old_keys_that_are_not_literally_false_keep_the_feature_on() {
    let game = GameDir::with_nams(Some(
        "validate_model_data = false
content_items = \"false\"
content_mcd = 0

[mouse]
sensitivity = 2.0
",
    ));
    game.ensure();
    let cfg = game.nams();
    expect_content_off(&cfg, &BTreeSet::new());
    assert!(!cfg.contains_key("content_items"));
    assert!(!cfg.contains_key("content_mcd"));
}

#[test]
fn existing_content_keeps_user_values_and_ignores_leftover_old_keys() {
    let game = GameDir::with_nams(Some(
        "validate_model_data = false
content_items = false
content_mcd = false

# Content features.
[content]
custom_items = true
custom_texts = true
novels = false

# Mouse input fixes.
[mouse]
sensitivity = 2.0
",
    ));
    game.ensure();
    let raw = game.raw_nams();
    let cfg = parse(&raw);

    assert!(!cfg.contains_key("content_items"));
    assert!(!cfg.contains_key("content_mcd"));
    assert!(!raw.contains("content_items"));
    assert!(!raw.contains("content_mcd"));

    expect_content_off(&cfg, &BTreeSet::from(["novels"]));

    let content_at = index_of(&raw, "[content]");
    let mouse_at = index_of(&raw, "# Mouse input fixes.");
    for field in content::fields() {
        let at = index_of(&raw, &format!("\n{} =", field.key()));
        assert!(at > content_at, "{} below [content]", field.key());
        assert!(at < mouse_at, "{} above [mouse]", field.key());
    }
    assert!(
        raw.contains("# Content features.\n[content]"),
        "the user comment stays on its table"
    );
    assert_eq!(table(&cfg, "mouse")["sensitivity"].as_float(), Some(2.0));
}

#[test]
fn content_migration_is_idempotent() {
    let fixtures = [
        None,
        Some(legacy_config(&[
            ("content_items", false),
            ("content_mcd", false),
        ])),
        Some("content_items = false\n\n[content]\nnovels = false\n".to_owned()),
    ];
    for fixture in fixtures {
        let game = GameDir::with_nams(fixture.as_deref());
        game.ensure();
        let first = game.raw_nams();
        game.ensure();
        assert_eq!(game.raw_nams(), first);
    }
}

#[test]
fn saving_a_content_toggle_rewrites_only_that_line() {
    let game = GameDir::with_nams(None);
    game.ensure();
    let raw = game.raw_nams();
    let mut values = parse(&raw);
    let mut content_values = table(&values, "content").clone();
    content_values.insert(
        content::NOVELS.key().to_owned(),
        toml::Value::Boolean(false),
    );
    values.insert("content".to_owned(), toml::Value::Table(content_values));

    let updated = yp_core::toml_io::update_toml(&raw, &values).unwrap();
    assert_eq!(
        updated.trim_end(),
        raw.replacen("novels = true", "novels = false", 1)
            .trim_end()
    );
}

#[test]
fn fresh_config_has_every_top_level_key_with_nams_defaults() {
    let game = GameDir::with_nams(None);
    game.ensure();
    let cfg = game.nams();
    let expected = [
        &nams::VALIDATE_MODEL_DATA,
        &nams::VALIDATE_SCRIPTS,
        &nams::LOADING_STALL_HINTS,
        &nams::DISABLE_PLUGIN_LOADING,
        &nams::DISABLE_CONTENT_FEATURES,
        &nams::EXPERIMENTAL_DEFAULT_OUTFITS,
        &nams::OUTFIT_SWAP_VISUAL_EFFECTS,
        &nams::DISABLE_RESHADE_LOADING,
        &nams::DISABLE_3DMIGOTO_LOADING,
        &nams::DISABLE_GAME_MODS_LOADING,
        &nams::DISABLE_GRAPHIC_MODS_LOADING,
        &nams::DISABLE_TEXTURE_INJECTION,
        &nams::DISABLE_SPLASH_SCREEN,
        &nams::FIX_WIND_TIMER_BUG,
        &nams::DISABLE_DEBUG_HOTKEYS,
        &nams::DISABLE_INPUT_FEATURES,
    ];
    for field in expected {
        assert!(
            cfg.contains_key(field.key()),
            "{} missing from generated nams.toml",
            field.key()
        );
        assert_eq!(
            cfg[field.key()].as_bool(),
            Some(field.default_value()),
            "{} default does not match NAMS",
            field.key()
        );
    }
}

#[test]
fn migration_adds_new_keys_to_an_old_config_at_top_level() {
    let game = GameDir::with_nams(Some(
        "# NAMS General Config.

validate_model_data = false
loading_stall_hints = true
disable_plugin_loading = false

[mouse]
fix_camera_acceleration = false
sensitivity = 2.0
",
    ));
    game.ensure();
    let cfg = game.nams();

    assert_eq!(
        cfg[nams::OUTFIT_SWAP_VISUAL_EFFECTS.key()].as_bool(),
        Some(true)
    );
    assert_eq!(
        cfg[nams::DISABLE_SPLASH_SCREEN.key()].as_bool(),
        Some(false)
    );
    assert_eq!(
        cfg[nams::DISABLE_CONTENT_FEATURES.key()].as_bool(),
        Some(false)
    );
    assert_eq!(
        table(&cfg, "content")[content::CUSTOM_ITEMS.key()].as_bool(),
        Some(true)
    );
    assert_eq!(
        cfg[nams::DISABLE_3DMIGOTO_LOADING.key()].as_bool(),
        Some(false)
    );
    assert_eq!(
        cfg[nams::DISABLE_RESHADE_LOADING.key()].as_bool(),
        Some(false)
    );
    assert_eq!(cfg[nams::DISABLE_DEBUG_HOTKEYS.key()].as_bool(), Some(true));
    assert_eq!(
        cfg[nams::DISABLE_INPUT_FEATURES.key()].as_bool(),
        Some(false)
    );
    assert_eq!(
        cfg[nams::DISABLE_GRAPHIC_MODS_LOADING.key()].as_bool(),
        Some(false)
    );
    assert_eq!(
        table(&cfg, "cutscene")[nams::SOLID_LETTERBOX_BARS.key()].as_bool(),
        Some(false),
        "solid_letterbox_bars migrated into [cutscene]"
    );

    let mouse = table(&cfg, "mouse");
    assert_eq!(mouse["sensitivity"].as_float(), Some(2.0));
    assert!(!cfg.contains_key("sensitivity"));
    for key in [
        nams::OUTFIT_SWAP_VISUAL_EFFECTS.key(),
        nams::DISABLE_SPLASH_SCREEN.key(),
    ] {
        assert!(
            !mouse.contains_key(key),
            "{key} must not land inside [mouse]"
        );
    }
}

#[test]
fn solid_letterbox_bars_lands_in_an_existing_cutscene_section() {
    let game = GameDir::with_nams(Some(
        "validate_model_data = false

[cutscene]
hd_cutscenes = true
enable_h264 = true
hide_subtitle_overlay = true
keep_subtitle_text = true
hide_subtitle_in_events = false

[heap]
global_heap_extra = 0
",
    ));
    game.ensure();
    let cfg = game.nams();
    let cutscene = table(&cfg, "cutscene");
    assert_eq!(
        cutscene[nams::SOLID_LETTERBOX_BARS.key()].as_bool(),
        Some(false)
    );
    assert_eq!(cutscene["hd_cutscenes"].as_bool(), Some(true));
    assert_eq!(cutscene["hide_subtitle_overlay"].as_bool(), Some(true));
    assert!(!cfg.contains_key(nams::SOLID_LETTERBOX_BARS.key()));
    assert_eq!(
        table(&cfg, "heap")["global_heap_extra"].as_integer(),
        Some(0)
    );
}

#[test]
fn migration_keeps_validate_scripts_off_matching_the_nams_default() {
    let game = GameDir::with_nams(Some(
        "validate_model_data = false\nloading_stall_hints = true\n",
    ));
    game.ensure();
    assert_eq!(
        game.nams()[nams::VALIDATE_SCRIPTS.key()].as_bool(),
        Some(false)
    );
}

#[test]
fn migration_preserves_user_values_and_is_idempotent() {
    let game = GameDir::with_nams(Some(
        "validate_model_data = true\nloading_stall_hints = false\noutfit_swap_visual_effects = false\n",
    ));
    game.ensure();
    let first = game.raw_nams();
    game.ensure();
    assert_eq!(game.raw_nams(), first, "second migration must be a no-op");

    let cfg = game.nams();
    assert_eq!(cfg[nams::VALIDATE_MODEL_DATA.key()].as_bool(), Some(true));
    assert_eq!(cfg[nams::LOADING_STALL_HINTS.key()].as_bool(), Some(false));
    assert_eq!(
        cfg[nams::OUTFIT_SWAP_VISUAL_EFFECTS.key()].as_bool(),
        Some(false)
    );
}

#[test]
fn naiom_toml_carries_every_mouse_field() {
    let game = GameDir::with_nams(Some("validate_model_data = false\n"));
    game.ensure();
    let raw = game.raw_naiom();
    let mouse_fields: [&dyn AnyField; 19] = [
        &nams::FIX_CAMERA_ACCELERATION,
        &nams::SENSITIVITY,
        &nams::THIRD_PERSON_MODE,
        &nams::THIRD_PERSON_CHAR_FOLLOW,
        &nams::THIRD_PERSON_SENSITIVITY_X,
        &nams::THIRD_PERSON_SENSITIVITY_Y,
        &nams::AIM_MODE,
        &nams::AIM_CROSSHAIR,
        &nams::AIM_CROSSHAIR_ALWAYS,
        &nams::AIM_SENSITIVITY,
        &nams::AIM_SENSITIVITY_X,
        &nams::AIM_SENSITIVITY_Y,
        &nams::AIM_OUTPUT_MULTIPLIER,
        &nams::MOVEMENT_DISABLE_TAP_EVADE,
        &nams::MISC_DISABLE_POD_PET,
        &nams::MISC_OPEN_DEBUG_MENU,
        &nams::MISC_CUSTOM_CURSOR_MENU,
        &nams::MISC_CUSTOM_CURSOR_HACKING,
        &nams::MISC_DISABLE_DEFAULT_CURSOR,
    ];
    let naiom = parse(&raw);
    for field in mouse_fields {
        assert!(
            naiom.contains_key(field.meta().key),
            "{} missing from naiom.toml",
            field.meta().key
        );
    }
    let naiom_bindings = table(&naiom, "bindings");
    for field in bindings::ALL {
        assert!(
            naiom_bindings.contains_key(field.key()),
            "{} missing from naiom.toml [bindings]",
            field.key()
        );
    }
}

fn read_textures(game: &GameDir) -> toml::Table {
    parse(&game.read_nams_file("texture_injection.toml"))
}

#[test]
fn texture_injection_fresh_config_has_every_key_with_nams_defaults() {
    let game = GameDir::with_nams(None);
    game.ensure();
    let cfg = read_textures(&game);
    for field in texture_injection::ALL {
        let key = field.meta().key;
        assert!(
            cfg.contains_key(key),
            "{key} missing from generated texture_injection.toml"
        );
        assert_eq!(
            cfg[key],
            field.default_toml(),
            "{key} default does not match NAMS"
        );
    }
}

#[test]
fn texture_injection_migration_adds_pin_priority_textures_and_keeps_user_values() {
    let game = GameDir::with_nams(None);
    game.write_nams_file(
        "texture_injection.toml",
        "vram_budget_mb = 2048\nstreaming_enabled = false\nload_only_relevant = true\nload_order = [\"A\", \"B\"]\nhot_reload = false\n",
    );
    game.ensure();
    let cfg = read_textures(&game);
    assert_eq!(
        cfg[texture_injection::PIN_PRIORITY_TEXTURES.key()].as_bool(),
        Some(true)
    );
    assert_eq!(
        cfg[texture_injection::VRAM_BUDGET_MB.key()].as_integer(),
        Some(2048)
    );
    assert_eq!(
        cfg[texture_injection::STREAMING_ENABLED.key()].as_bool(),
        Some(false)
    );
    assert_eq!(
        cfg[texture_injection::LOAD_ONLY_RELEVANT.key()].as_bool(),
        Some(true)
    );
    assert_eq!(
        cfg[texture_injection::LOAD_ORDER.key()],
        toml::Value::Array(vec!["A".into(), "B".into()])
    );
    assert_eq!(
        cfg[texture_injection::HOT_RELOAD.key()].as_bool(),
        Some(false)
    );
}

const OLD_MOUSE_CONFIG: &str = "# NAMS General Config. Restart game after changes.

validate_model_data = false
validate_scripts = false
loading_stall_hints = true
disable_plugin_loading = false
disable_content_features = false

content_items = true
content_accessories = true
content_assemble_meshes = true
content_quest_integration = true
content_effects_applier = true
content_equip_tracker = true
content_mcd = true
content_buddy_ruby_selector = true

outfit_swap_visual_effects = true
disable_reshade_loading = false
disable_texture_injection = false
disable_splash_screen = false
fix_wind_timer_bug = true

# Mouse input fixes.
[mouse]
# Remove the deadzone and acceleration curve from camera rotation.
fix_camera_acceleration = true
# Sensitivity multiplier. Higher = faster camera rotation.
sensitivity = 3.75
# Disable the pod petting animation that triggers when moving the mouse.
disable_pod_pet = true
# Remove the clamp and deadzone from pod/mech aiming.
fix_aim_acceleration = true
# Aim sensitivity for top-down/side-scroll.
aim_sensitivity = 0.002
# Raw multiplier applied to aim output after normalization.
aim_output_multiplier = 12.0
# Virtual key code for opening the debug menu. 0 = disabled.
debug_menu_key = 112
# Virtual key code for dedicated evade/dodge. 0 = disabled.
evade_key = 82

[cutscene]
hd_cutscenes = true
enable_h264 = false

[heap]
global_heap_extra = 0
";

#[test]
fn old_mouse_keys_are_renamed_with_values_preserved() {
    let game = GameDir::with_nams(Some(OLD_MOUSE_CONFIG));
    game.ensure();
    let cfg = game.nams();
    let mouse = table(&cfg, "mouse");

    assert_eq!(mouse["aim_mode"].as_bool(), Some(true));
    assert_eq!(mouse["misc_disable_pod_pet"].as_bool(), Some(true));
    assert_eq!(mouse["misc_open_debug_menu"].as_integer(), Some(112));
    assert!(!mouse.contains_key("fix_aim_acceleration"));
    assert!(!mouse.contains_key("disable_pod_pet"));
    assert!(!mouse.contains_key("debug_menu_key"));
    assert!(
        !mouse.contains_key("evade_key"),
        "evade_key was never read by NAMS and must be removed"
    );

    assert_eq!(mouse["sensitivity"].as_float(), Some(3.75));
    assert_eq!(mouse["aim_sensitivity"].as_float(), Some(0.002));
    assert_eq!(mouse["aim_output_multiplier"].as_float(), Some(12.0));
    assert_eq!(mouse["fix_camera_acceleration"].as_bool(), Some(true));
    assert!(!game.raw_nams().contains("dedicated evade/dodge"));
}

#[test]
fn evade_key_vk_code_becomes_non_standard_evade_key_name() {
    let game = GameDir::with_nams(Some(OLD_MOUSE_CONFIG));
    game.ensure();
    let naiom = game.naiom();
    assert_eq!(
        table(&naiom, "bindings")["non_standard_evade"].as_str(),
        Some("R")
    );
}

#[test]
fn naiom_toml_gets_the_old_mouse_values_and_every_new_key() {
    let game = GameDir::with_nams(Some(OLD_MOUSE_CONFIG));
    game.ensure();
    let raw = game.raw_naiom();
    let mouse = parse(&raw);
    let naiom_bindings = table(&mouse, "bindings");

    for key in [
        "third_person_mode",
        "third_person_char_follow",
        "third_person_smoothing",
        "third_person_sensitivity_x",
        "third_person_sensitivity_y",
        "aim_crosshair",
        "aim_crosshair_always",
        "aim_sensitivity_x",
        "aim_sensitivity_y",
        "movement_disable_tap_evade",
        "misc_custom_cursor_menu",
        "misc_custom_cursor_hacking",
        "misc_disable_default_cursor",
    ] {
        assert!(mouse.contains_key(key), "{key} must be in naiom.toml");
    }
    for field in bindings::ALL {
        assert!(
            naiom_bindings.contains_key(field.key()),
            "{} must exist in [bindings]",
            field.key()
        );
    }
    assert!(
        index_of(&raw, "third_person_mode = false") < index_of(&raw, "[bindings]"),
        "mouse keys must stay top-level, not inside [bindings]"
    );
    assert!(raw.contains("# Remove the deadzone and acceleration curve"));
    assert_eq!(mouse["sensitivity"].as_float(), Some(3.75));
    assert_eq!(mouse["aim_mode"].as_bool(), Some(true));
    assert_eq!(mouse["misc_disable_pod_pet"].as_bool(), Some(true));
    assert_eq!(mouse["misc_open_debug_menu"].as_integer(), Some(112));
    assert_eq!(mouse["aim_sensitivity"].as_float(), Some(0.002));
    assert_eq!(mouse["aim_output_multiplier"].as_float(), Some(12.0));
    assert_eq!(mouse["fix_camera_acceleration"].as_bool(), Some(true));
    assert!(!mouse.contains_key("evade_key"));
    assert_eq!(
        table(&game.nams(), "cutscene")["hd_cutscenes"].as_bool(),
        Some(true)
    );
}

#[test]
fn rename_migration_is_idempotent() {
    let game = GameDir::with_nams(Some(OLD_MOUSE_CONFIG));
    game.ensure();
    let first = game.raw_nams();
    game.ensure();
    assert_eq!(game.raw_nams(), first);
}

#[test]
fn subtitle_keys_are_added_to_an_existing_cutscene_section() {
    let game = GameDir::with_nams(Some(OLD_MOUSE_CONFIG));
    game.ensure();
    let raw = game.raw_nams();
    let cfg = parse(&raw);
    let cutscene = table(&cfg, "cutscene");
    for field in [
        &nams::HIDE_SUBTITLE_OVERLAY,
        &nams::KEEP_SUBTITLE_TEXT,
        &nams::HIDE_SUBTITLE_IN_EVENTS,
    ] {
        assert!(
            cutscene.contains_key(field.key()),
            "{} must be added to [cutscene]",
            field.key()
        );
        assert_eq!(cutscene[field.key()].as_bool(), Some(field.default_value()));
    }
    assert_eq!(cutscene["hd_cutscenes"].as_bool(), Some(true));
    assert!(
        index_of(&raw, "hide_subtitle_overlay") < index_of(&raw, "[heap]"),
        "new keys must land inside [cutscene], not at EOF"
    );
}

#[test]
fn a_config_with_no_cutscene_section_gains_the_full_section() {
    let game = GameDir::with_nams(Some("validate_model_data = false\n"));
    game.ensure();
    let cfg = game.nams();
    let cutscene = table(&cfg, "cutscene");
    assert_eq!(cutscene["hide_subtitle_overlay"].as_bool(), Some(false));
    assert_eq!(cutscene["keep_subtitle_text"].as_bool(), Some(false));
    assert_eq!(cutscene["hide_subtitle_in_events"].as_bool(), Some(false));
}

#[test]
fn game_mods_flag_is_added_to_an_old_config() {
    let game = GameDir::with_nams(Some(
        "validate_model_data = false\ndisable_reshade_loading = false\ndisable_3dmigoto_loading = false\n\n[mouse]\nsensitivity = 2.0\n",
    ));
    game.ensure();
    let raw = game.raw_nams();
    let cfg = parse(&raw);
    assert_eq!(
        cfg[nams::DISABLE_GAME_MODS_LOADING.key()].as_bool(),
        Some(false)
    );
    assert!(
        index_of(&raw, "disable_game_mods_loading") < index_of(&raw, "[mouse]"),
        "must be inserted as a top-level key, not into [mouse]"
    );
}

#[test]
fn cutscene_migration_is_idempotent() {
    let game = GameDir::with_nams(Some(OLD_MOUSE_CONFIG));
    game.ensure();
    let first = game.raw_nams();
    game.ensure();
    assert_eq!(game.raw_nams(), first);
}

#[test]
fn unbindable_evade_key_vk_is_dropped_and_the_binding_stays_empty() {
    let game = GameDir::with_nams(Some("[mouse]\nevade_key = 19\n"));
    game.ensure();
    assert_eq!(
        table(&game.naiom(), "bindings")["non_standard_evade"].as_str(),
        Some(""),
        "VK 19 (Pause) has no NAMS key name and must not be migrated"
    );
}

#[test]
fn a_function_key_evade_key_vk_is_migrated_by_name() {
    let game = GameDir::with_nams(Some("[mouse]\nevade_key = 112\n"));
    game.ensure();
    assert_eq!(
        table(&game.naiom(), "bindings")["non_standard_evade"].as_str(),
        Some("F1")
    );
}
