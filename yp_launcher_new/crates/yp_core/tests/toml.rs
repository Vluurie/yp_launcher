use yp_core::toml_io::{
    insert_with_comment, merge_values, parse, parse_or_empty, parse_values, remove_key, rename_key, same_values,
    set_value, to_string_verified, update_text, update_toml, values, Value,
};
use yp_core::Error;

fn table(text: &str) -> toml::Table {
    toml::from_str(text).unwrap()
}

fn get<'a>(values: &'a toml::Table, path: &[&str]) -> &'a toml::Value {
    let (last, parents) = path.split_last().unwrap();
    let mut current = values;
    for key in parents {
        current = current[*key].as_table().unwrap();
    }
    &current[*last]
}

#[test]
fn saving_after_a_commented_header_keeps_fields_in_their_table() {
    let raw = "enabled = true\n  [high_grids] # rules\nrings = 2\n";
    let output = update_toml(raw, &table("render_scale = 1.5\n[high_grids]\nrings = 3\n")).unwrap();
    let parsed = parse_values(&output).unwrap();
    assert_eq!(parsed["render_scale"].as_float(), Some(1.5));
    assert_eq!(get(&parsed, &["high_grids", "rings"]).as_integer(), Some(3));
    assert!(!parsed["high_grids"].as_table().unwrap().contains_key("render_scale"));
    assert!(output.contains("# rules"));
}

#[test]
fn updates_array_of_table_rules_without_duplicate_declarations() {
    let raw = "[high_grids] # keep rules\nrings = 2\n[[high_grids.room_rings]]\nroom = 0x170\nrings = 1\n";
    let output = update_toml(
        raw,
        &table("fxaa = true\n[high_grids]\nroom_rings = [{ room = 0x150, rings = 2 }]\n"),
    )
    .unwrap();
    let parsed = parse_values(&output).unwrap();
    assert_eq!(parsed["fxaa"].as_bool(), Some(true));
    assert_eq!(get(&parsed, &["high_grids", "rings"]).as_integer(), Some(2));
    let rules = get(&parsed, &["high_grids", "room_rings"]).as_array().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["room"].as_integer(), Some(0x150));
    assert_eq!(rules[0]["rings"].as_integer(), Some(2));
    assert!(output.contains("# keep rules"));
}

#[test]
fn unknown_multiline_values_and_escaped_strings_survive_a_save() {
    let raw = "enabled = true\nnote = \"\"\"\n[not_a_table]\ntext = 1\n\"\"\"\n[high_grids]\nrings = 2\n";
    let label = "quoted \"name\" with \\ and\na newline";
    let mut updates = toml::Table::new();
    updates.insert("label".into(), toml::Value::String(label.into()));
    let output = update_toml(raw, &updates).unwrap();
    let parsed = parse_values(&output).unwrap();
    assert_eq!(parsed["label"].as_str(), Some(label));
    assert_eq!(parsed["note"].as_str(), Some("[not_a_table]\ntext = 1\n"));
    assert_eq!(get(&parsed, &["high_grids", "rings"]).as_integer(), Some(2));
}

#[test]
fn refuses_to_rewrite_malformed_toml() {
    let err = update_toml("enabled = true\nenabled = false\n", &table("fxaa = true")).unwrap_err();
    assert!(matches!(err, Error::TomlParse { .. }));
}

const ORIGINAL: &str = "# NAMS General Config.

validate_scripts = false

[mouse]
# Remove the deadzone and acceleration curve.
fix_camera_acceleration = false
# Sensitivity multiplier.
sensitivity = 2.0

[cutscene]
hd_cutscenes = false
";

#[test]
fn writes_new_keys_into_an_existing_section() {
    let updated = update_toml(ORIGINAL, &table("[mouse]\nsensitivity = 3.5\nthird_person_mode = true\n")).unwrap();
    assert!(updated.contains("sensitivity = 3.5"));
    assert!(updated.contains("third_person_mode = true"));
    assert!(updated.find("third_person_mode = true").unwrap() < updated.find("[cutscene]").unwrap());
}

#[test]
fn creates_a_missing_nested_section_with_its_keys() {
    let updated = update_toml(
        ORIGINAL,
        &table("[mouse.bindings]\nstandard_jump = \"SPACE\"\nnon_standard_evade = \"\"\n"),
    )
    .unwrap();
    assert!(updated.contains("[mouse.bindings]"));
    assert!(updated.contains("standard_jump = \"SPACE\""));
    assert!(updated.contains("non_standard_evade = \"\""));
    assert!(!updated.contains("bindings = "));
}

#[test]
fn preserves_comments() {
    let updated = update_toml(ORIGINAL, &table("validate_scripts = true\n[mouse]\nsensitivity = 3.5\n")).unwrap();
    assert!(updated.contains("# NAMS General Config."));
    assert!(updated.contains("# Remove the deadzone and acceleration curve."));
    assert!(updated.contains("# Sensitivity multiplier."));
}

#[test]
fn round_trips_through_parse() {
    let updated = update_toml(
        ORIGINAL,
        &table("[mouse]\nsensitivity = 3.5\n[mouse.bindings]\nstandard_jump = \"SPACE\"\n"),
    )
    .unwrap();
    let parsed = values(&parse_or_empty(&updated));
    assert_eq!(get(&parsed, &["mouse", "sensitivity"]).as_float(), Some(3.5));
    assert_eq!(get(&parsed, &["mouse", "bindings", "standard_jump"]).as_str(), Some("SPACE"));
    assert_eq!(get(&parsed, &["cutscene", "hd_cutscenes"]).as_bool(), Some(false));
}

#[test]
fn write_back_of_a_parsed_document_is_a_no_op() {
    let updated = update_toml(ORIGINAL, &table("[mouse.bindings]\nstandard_jump = \"SPACE\"\n")).unwrap();
    let twice = update_toml(&updated, &values(&parse_or_empty(&updated))).unwrap();
    assert_eq!(twice, updated);
}

#[test]
fn updates_keys_inside_an_existing_dotted_section_in_place() {
    let with_nested = "
[mouse]
sensitivity = 2.0

[mouse.bindings]
standard_jump = \"SPACE\"

[cutscene]
hd_cutscenes = true
";
    let updated = update_toml(
        with_nested,
        &table("[mouse]\nsensitivity = 4.0\n[mouse.bindings]\nstandard_jump = \"E\"\nstandard_walk = \"LSHIFT\"\n"),
    )
    .unwrap();
    assert!(updated.contains("standard_jump = \"E\""));
    let walk = updated.find("standard_walk = \"LSHIFT\"").unwrap();
    assert!(walk > updated.find("[mouse.bindings]").unwrap());
    assert!(walk < updated.find("[cutscene]").unwrap());
    assert!(!updated.split("[mouse.bindings]").next().unwrap().contains("standard_jump"));
    assert!(updated.contains("hd_cutscenes = true"));
}

#[test]
fn strict_accepts_a_valid_manifest() {
    let parsed = parse_values("id = \"my_first_mod\"\ndisplay_name = \"My First Mod\"\nversion = \"1.0.0\"\n").unwrap();
    assert_eq!(parsed["id"].as_str(), Some("my_first_mod"));
}

#[test]
fn strict_rejects_broken_documents() {
    for broken in [
        "id = \"my_first_mod\nversion = \"1.0.0\"\n",
        "id = \"my_first_mod\"\ndisplay_name = \"My First Mod\n",
        "id = my_first_mod\n",
        "name = \"x\"\n[text.name\n_us = \"y\"\n",
        "id = \"a\"\nid = \"b\"\n",
    ] {
        let err = parse(broken).unwrap_err();
        assert!(!err.message.is_empty(), "{broken:?}");
        assert!(err.line.is_some(), "{broken:?}");
    }
}

#[test]
fn strict_empty_input_is_not_an_error() {
    assert!(parse_values("   \n").unwrap().is_empty());
}

#[test]
fn strict_accepts_the_item_example_with_its_tables() {
    let parsed = parse_values(
        "name = \"item_test_pebble\"\nid_type = 0\nno_trash = true\n\n[auto_give]\nquantity = 1\n\n[text.name]\n_us = \"Test Pebble\"\n",
    )
    .unwrap();
    assert_eq!(get(&parsed, &["auto_give", "quantity"]).as_integer(), Some(1));
    assert_eq!(get(&parsed, &["text", "name", "_us"]).as_str(), Some("Test Pebble"));
}

#[test]
fn lenient_parse_swallows_what_strict_rejects() {
    let broken = "id = \"my_first_mod\n";
    assert!(parse(broken).is_err());
    assert!(values(&parse_or_empty(broken)).is_empty());
}

#[test]
fn replaces_a_multiline_array_without_leaking_its_old_lines() {
    let raw = "
[high_grids]
rings = 1
room_rings = [
    { room = 0x170, rings = 1 },
]
far_grid_load_interval = 3
";
    let mut current = parse_values(raw).unwrap();
    let grids = current.get_mut("high_grids").unwrap().as_table_mut().unwrap();
    grids.insert(
        "room_rings".into(),
        toml::Value::Array(vec![
            toml::Value::Table(table("room = 0x170\nrings = 1")),
            toml::Value::Table(table("room = 0x150\nrings = 2")),
        ]),
    );
    let out = update_toml(raw, &current).unwrap();
    assert!(!out.contains("0x170"));
    assert!(out.matches('\n').count() < 8);
    let reparsed = parse_values(&out).unwrap();
    let rules = get(&reparsed, &["high_grids", "room_rings"]).as_array().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[1]["room"].as_integer(), Some(0x150));
    assert_eq!(get(&reparsed, &["high_grids", "far_grid_load_interval"]).as_integer(), Some(3));
}

#[test]
fn keeps_a_multiline_array_untouched_when_the_value_is_unchanged() {
    let raw = "
[high_grids]
room_rings = [
    { room = 0x170, rings = 1 },
]
";
    let out = update_toml(raw, &parse_values(raw).unwrap()).unwrap();
    assert_eq!(out, raw);
    let rules = get(&parse_values(&out).unwrap(), &["high_grids", "room_rings"]).as_array().unwrap().clone();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["room"].as_integer(), Some(0x170));
}

#[test]
fn ignores_brackets_inside_strings_and_comments() {
    let raw = "name = \"a [ b\"  # trailing ] comment\nother = 1\n";
    let mut current = parse_values(raw).unwrap();
    current.insert("other".into(), toml::Value::Integer(2));
    let out = update_toml(raw, &current).unwrap();
    let reparsed = parse_values(&out).unwrap();
    assert_eq!(reparsed["name"].as_str(), Some("a [ b"));
    assert_eq!(reparsed["other"].as_integer(), Some(2));
    assert!(out.contains("# trailing ] comment"));
}

#[test]
fn set_value_keeps_trailing_comments_and_creates_tables() {
    let mut doc = parse("[core]\nlevel = 1 # keep me\n").unwrap();
    set_value(&mut doc, &["core", "level"], Value::from(5));
    set_value(&mut doc, &["extra", "deep", "flag"], Value::from(true));
    let text = doc.to_string();
    assert!(text.contains("level = 5 # keep me"));
    let parsed = parse_values(&text).unwrap();
    assert_eq!(get(&parsed, &["extra", "deep", "flag"]).as_bool(), Some(true));
    assert!(!text.contains("[extra]\n"));
}

#[test]
fn remove_and_rename_keep_order_and_comments() {
    let mut doc = parse("[mouse]\n# first\nalpha = 1\n# old name\nbeta = 2\ngamma = 3\n").unwrap();
    assert!(rename_key(&mut doc, &["mouse"], "beta", "delta"));
    assert!(!rename_key(&mut doc, &["mouse"], "missing", "x"));
    assert!(!rename_key(&mut doc, &["mouse"], "alpha", "gamma"));
    let text = doc.to_string();
    assert_eq!(text, "[mouse]\n# first\nalpha = 1\n# old name\ndelta = 2\ngamma = 3\n");
    assert!(remove_key(&mut doc, &["mouse", "alpha"]).is_some());
    assert!(remove_key(&mut doc, &["mouse", "alpha"]).is_none());
    assert!(remove_key(&mut doc, &["nope", "alpha"]).is_none());
    assert!(!doc.to_string().contains("alpha"));
}

#[test]
fn insert_with_comment_writes_the_comment_above_the_key() {
    let mut doc = parse("[mouse]\nalpha = 1\n").unwrap();
    insert_with_comment(&mut doc, &["mouse"], "beta", Value::from(2.5), &["Second line", "# already marked"]);
    insert_with_comment(&mut doc, &["mouse"], "alpha", Value::from(7), &["ignored"]);
    let text = doc.to_string();
    assert_eq!(text, "[mouse]\nalpha = 7\n# Second line\n# already marked\nbeta = 2.5\n");
}

#[test]
fn merge_values_and_verified_output() {
    let mut doc = parse("a = 1\n").unwrap();
    merge_values(&mut doc, &table("b = \"x\"\n[t]\nc = [1, 2]\n"));
    let expected = table("a = 1\nb = \"x\"\n[t]\nc = [1, 2]\n");
    let text = to_string_verified(&doc, &expected).unwrap();
    assert!(same_values(&parse_values(&text).unwrap(), &expected));
    let wrong = table("a = 2");
    assert!(matches!(to_string_verified(&doc, &wrong), Err(Error::TomlRoundTrip { .. })));
}

#[test]
fn update_text_edits_without_losing_comments_and_never_accepts_broken_input() {
    let src = "# header\n[core]\nlevel = 1 # inline\n";
    let out = update_text(src, &[(&["core", "level"], Value::from(2)), (&["core", "name"], Value::from("n"))]).unwrap();
    assert_eq!(out, "# header\n[core]\nlevel = 2 # inline\nname = \"n\"\n");
    assert!(update_text("a = \"", &[(&["a"], Value::from(1))]).is_err());
}

#[test]
fn same_values_treats_nan_as_equal_and_types_strictly() {
    let a = table("x = nan\ny = [1, 2]");
    let b = table("x = nan\ny = [1, 2]");
    assert!(same_values(&a, &b));
    assert!(!same_values(&table("x = 1"), &table("x = 1.0")));
    assert!(!same_values(&table("x = 1"), &table("x = 1\ny = 2")));
}

#[test]
fn crlf_documents_round_trip_unchanged() {
    let src = "# c\r\n[core]\r\nlevel = 1\r\n";
    let out = update_toml(src, &parse_values(src).unwrap()).unwrap();
    assert_eq!(out, src);
    let edited = update_text(src, &[(&["core", "level"], Value::from(3))]).unwrap();
    assert_eq!(edited, "# c\r\n[core]\r\nlevel = 3\r\n");
}

#[test]
fn empty_tables_survive_as_explicit_headers() {
    let mut updates = toml::Table::new();
    updates.insert("empty".into(), toml::Value::Table(toml::Table::new()));
    updates.insert("outer".into(), toml::Value::Table(table("[inner]\nflag = true")));
    let out = update_toml("a = 1\n", &updates).unwrap();
    assert!(out.contains("[empty]"));
    assert!(out.contains("[outer.inner]"));
    assert!(!out.contains("[outer]\n"));
    assert!(parse_values(&out).unwrap()["empty"].as_table().unwrap().is_empty());
}
