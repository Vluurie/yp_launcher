use serde_json::{Value, json};
use yp_config::migrate::with_impeller_off;
use yp_config::overlay::IMPELLER_KEY;

fn decode(raw: &str) -> serde_json::Map<String, Value> {
    match serde_json::from_str::<Value>(raw).unwrap() {
        Value::Object(map) => map,
        other => panic!("not an object: {other}"),
    }
}

#[test]
fn impeller_is_turned_off_and_everything_else_survives() {
    let raw = json!({
        IMPELLER_KEY: true,
        "firstPlaythrough": true,
        "keybinds": {"main": {"yorha_protocol": "F2"}},
    })
    .to_string();
    let result = decode(&with_impeller_off(&raw));
    assert_eq!(result[IMPELLER_KEY], Value::Bool(false));
    assert_eq!(result["firstPlaythrough"], Value::Bool(true));
    assert_eq!(result["keybinds"]["main"], json!({"yorha_protocol": "F2"}));
}

#[test]
fn impeller_off_output_uses_two_space_indentation_and_keeps_key_order() {
    let raw = "{\"b\": 1, \"impeller\": true, \"a\": [1, 2]}";
    assert_eq!(
        with_impeller_off(raw),
        "{\n  \"b\": 1,\n  \"impeller\": false,\n  \"a\": [\n    1,\n    2\n  ]\n}"
    );
}

#[test]
fn a_settings_file_that_already_has_it_off_is_left_alone() {
    let raw = "{\"impeller\": false}";
    assert_eq!(with_impeller_off(raw), raw);
}

#[test]
fn a_settings_file_without_the_key_is_left_alone() {
    let raw = "{\"firstPlaythrough\": true}";
    assert_eq!(with_impeller_off(raw), raw);
}

#[test]
fn unreadable_json_is_passed_through_instead_of_being_lost() {
    let raw = "{ this is not json";
    assert_eq!(with_impeller_off(raw), raw);
}

#[test]
fn a_json_array_is_passed_through() {
    let raw = "[true]";
    assert_eq!(with_impeller_off(raw), raw);
}
