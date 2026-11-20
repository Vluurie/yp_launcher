mod support;

use support::{GameDir, table};
use yp_config::fields::lodmod;
use yp_config::migrate::migrate_lodmod;
use yp_config::state::ConfigStore;

fn read_config(raw: &str) -> toml::Table {
    yp_core::toml_io::parse_values(raw).unwrap_or_else(|err| panic!("invalid TOML ({err}):\n{raw}"))
}

fn table_value(entries: &[(&str, i64)]) -> toml::Value {
    toml::Value::Table(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), toml::Value::Integer(*value)))
            .collect(),
    )
}

fn floats(values: &[f64]) -> toml::Value {
    toml::Value::Array(
        values
            .iter()
            .map(|value| toml::Value::Float(*value))
            .collect(),
    )
}

#[test]
fn adds_the_current_nams_fields_at_root_before_indented_commented_tables() {
    let raw = "enabled = true
lod_multiplier = 0.75
custom_root = 42
  [ high_grids ] # keep my rules
enabled = true
rings = 3
room_rings = [
  { room = 0x170, rings = 1 },
]
hidden_meshes = []
custom_grid = \"untouched\"
";
    let migrated = migrate_lodmod(raw);
    let config = read_config(&migrated);
    let grids = table(&config, "high_grids");

    assert_eq!(config["enabled"].as_bool(), Some(true));
    assert_eq!(config["lod_multiplier"].as_float(), Some(0.75));
    assert_eq!(config["custom_root"].as_integer(), Some(42));
    assert_eq!(config["render_scale"].as_float(), Some(1.0));
    assert_eq!(config["shadow_cascades"].as_integer(), Some(4));
    assert_eq!(config["shadow_cascade_range"].as_float(), Some(4.0));
    assert_eq!(config["shadow_blur_scale"], floats(&[1.0, 1.0, 1.0, 1.0]));
    assert_eq!(config["disable_hdr"].as_bool(), Some(false));
    assert_eq!(config["fxaa"].as_bool(), Some(false));
    assert_eq!(config["msaa_prepass_fix"].as_bool(), Some(true));
    assert_eq!(config["msaa_shadow_mask_fix"].as_bool(), Some(true));
    assert_eq!(config["msaa_per_pixel"].as_bool(), Some(false));
    assert_eq!(config["msaa_per_pixel_terrain"].as_bool(), Some(false));
    assert_eq!(config["ao_fade_fix"].as_bool(), Some(true));
    assert_eq!(config["ao_full_resolution"].as_bool(), Some(false));
    assert_eq!(config["constant_buffer_dedup"].as_bool(), Some(false));
    assert_eq!(
        config["constant_buffer_upload_on_bind"].as_bool(),
        Some(false)
    );
    assert_eq!(
        config["decrease_stutter_during_grid_loading"].as_bool(),
        Some(false)
    );
    assert_eq!(config["fps_cap_in_hacking"].as_bool(), Some(true));
    assert_eq!(config["fps_cap_in_events"].as_bool(), Some(true));
    assert_eq!(grids["enabled"].as_bool(), Some(true));
    assert_eq!(grids["rings"].as_integer(), Some(3));
    assert_eq!(
        grids["room_rings"],
        toml::Value::Array(vec![table_value(&[("room", 0x170), ("rings", 1)])])
    );
    assert_eq!(grids["hidden_meshes"].as_array().map(Vec::len), Some(0));
    assert_eq!(grids["custom_grid"].as_str(), Some("untouched"));
    assert!(!grids.contains_key("render_scale"));
    assert!(migrated.contains("# keep my rules"));
    assert_eq!(migrate_lodmod(&migrated), migrated);
}

#[test]
fn an_indented_table_header_at_eof_is_not_duplicated() {
    let migrated = migrate_lodmod("enabled = true\n  [high_grids]");
    let config = read_config(&migrated);
    let grids = table(&config, "high_grids");
    assert_eq!(config["enabled"].as_bool(), Some(true));
    assert_eq!(grids["enabled"].as_bool(), Some(false));
    assert_eq!(grids["rings"].as_integer(), Some(1));
    assert_eq!(
        grids["hidden_meshes"],
        toml::Value::Array(lodmod::HIGH_GRIDS_HIDDEN_MESHES.default_value())
    );
}

#[test]
fn repairs_misplaced_root_values_and_keeps_explicit_root_values_authoritative() {
    let raw = "enabled = false
fxaa = false
[high_grids]
enabled = true
rings = 2
fxaa = true
render_scale = 1.5
shadow_blur_scale = [
  0.5, 0.6, 0.7, 0.8,
]
constant_buffer_dedup = true
";
    let migrated = migrate_lodmod(raw);
    let config = read_config(&migrated);
    let grids = table(&config, "high_grids");
    assert_eq!(config["enabled"].as_bool(), Some(false));
    assert_eq!(config["fxaa"].as_bool(), Some(false));
    assert_eq!(config["render_scale"].as_float(), Some(1.5));
    assert_eq!(config["shadow_blur_scale"], floats(&[0.5, 0.6, 0.7, 0.8]));
    assert_eq!(config["constant_buffer_dedup"].as_bool(), Some(true));
    assert_eq!(grids["enabled"].as_bool(), Some(true));
    assert_eq!(grids["rings"].as_integer(), Some(2));
    for key in [
        "fxaa",
        "render_scale",
        "shadow_blur_scale",
        "constant_buffer_dedup",
    ] {
        assert!(!grids.contains_key(key), "{key} must leave [high_grids]");
    }
    assert_eq!(migrate_lodmod(&migrated), migrated);
}

#[test]
fn supports_quoted_headers_crlf_and_missing_trailing_newlines() {
    let raw = "enabled = true\r\n [\"high_grids\"] # grids\r\nrings = 4\r\nrender_scale = 2.0";
    let config = read_config(&migrate_lodmod(raw));
    let grids = table(&config, "high_grids");
    assert_eq!(config["render_scale"].as_float(), Some(2.0));
    assert_eq!(grids["rings"].as_integer(), Some(4));
    assert!(!grids.contains_key("render_scale"));
}

#[test]
fn migrates_alternate_table_syntax() {
    for source in [
        "high_grids.enabled = true\nhigh_grids.rings = 3\nhigh_grids.render_scale = 1.5\n",
        "high_grids = { enabled = true, rings = 3, render_scale = 1.5 }\n",
    ] {
        let migrated = migrate_lodmod(source);
        let config = read_config(&migrated);
        let grids = table(&config, "high_grids");
        assert_eq!(config["render_scale"].as_float(), Some(1.5), "{source}");
        assert_eq!(grids["enabled"].as_bool(), Some(true), "{source}");
        assert_eq!(grids["rings"].as_integer(), Some(3), "{source}");
        assert!(!grids.contains_key("render_scale"), "{source}");
        assert_eq!(migrate_lodmod(&migrated), migrated, "{source}");
    }
}

#[test]
fn keeps_array_of_table_rules_and_multiline_strings_intact() {
    let raw = "enabled = true
note = \"\"\"
[not_a_table]
render_scale = 9.0
\"\"\"
[high_grids] # grid rules
enabled = true
rings = 4
[[high_grids.room_rings]]
room = 0x170
rings = 1
[[high_grids.room_rings]]
room = 0x150
rings = 2
";
    let migrated = migrate_lodmod(raw);
    let config = read_config(&migrated);
    assert_eq!(
        config["note"].as_str(),
        Some("[not_a_table]\nrender_scale = 9.0\n")
    );
    assert_eq!(config["render_scale"].as_float(), Some(1.0));
    assert_eq!(
        table(&config, "high_grids")["room_rings"],
        toml::Value::Array(vec![
            table_value(&[("room", 0x170), ("rings", 1)]),
            table_value(&[("room", 0x150), ("rings", 2)]),
        ])
    );
    assert!(migrated.contains("# grid rules"));
}

#[test]
fn leaves_malformed_input_and_invalid_table_types_untouched() {
    for raw in [
        "enabled = true\nenabled = false\n",
        "[high_grids\nrings = 2\n",
        "high_grids = \"not a table\"\n",
    ] {
        assert_eq!(migrate_lodmod(raw), raw);
    }
}

#[test]
fn a_comment_above_a_misplaced_key_stays_in_its_table() {
    let migrated =
        migrate_lodmod("[high_grids]\nenabled = true\n# my note\nfxaa = true\nrings = 2\n");
    let config = read_config(&migrated);
    assert_eq!(config["fxaa"].as_bool(), Some(true));
    assert!(migrated.contains("# my note\nrings = 2"));
}

#[test]
fn ensure_configs_gui_save_and_reload_keep_migrated_fields_in_their_nams_scope() {
    let game = GameDir::empty();
    game.write_nams_file(
        "lodmod.toml",
        "enabled = true\n [high_grids] # my settings\nenabled = true\nrings = 3\nfxaa = true\nrender_scale = 1.25\n",
    );

    game.ensure();
    let first = game.read_nams_file("lodmod.toml");
    assert_eq!(read_config(&first)["render_scale"].as_float(), Some(1.25));
    game.ensure();
    assert_eq!(game.read_nams_file("lodmod.toml"), first);

    let mut store = ConfigStore::new(game.game.clone(), game.host.clone());
    store.load().unwrap();
    store.update_lodmod(lodmod::RENDER_SCALE.key(), toml::Value::Float(1.75), None);
    store.update_lodmod(
        lodmod::HIGH_GRIDS_RINGS.key(),
        toml::Value::Integer(4),
        Some(lodmod::HIGH_GRIDS_SECTION),
    );
    store.save().unwrap();
    store.load().unwrap();

    let config = read_config(&game.read_nams_file("lodmod.toml"));
    let grids = table(&config, "high_grids");
    assert_eq!(config["fxaa"].as_bool(), Some(true));
    assert_eq!(config["render_scale"].as_float(), Some(1.75));
    assert_eq!(grids["rings"].as_integer(), Some(4));
    assert!(!grids.contains_key("render_scale"));
    assert!(store.data().lodmod_error.is_none());
}

#[test]
fn every_lodmod_field_has_a_default_that_round_trips() {
    for field in lodmod::ALL {
        let meta = field.meta();
        let migrated = read_config(&migrate_lodmod(""));
        let scope = match meta.section {
            Some(section) => table(&migrated, section).clone(),
            None => migrated.clone(),
        };
        assert_eq!(
            scope.get(meta.key),
            Some(&field.default_toml()),
            "{}",
            meta.key
        );
    }
}
