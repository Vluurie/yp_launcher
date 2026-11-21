use indexmap::IndexMap;
use serde_json::Value as Json;
use yp_config::{Schema, ValidateReport, Values, values_equal};
use yp_core::toml_io::{parse_values, same_value};

use super::model::{ConfigDelta, ConfigReport};
use super::text::truncated;

pub const DISPLAY_MAX_CHARS: usize = 160;
pub const SIGNIFICANT_DIGITS: usize = 7;
pub const UNKNOWN_KEY_CODE: &str = "unknown_key";
pub const DEPRECATED_KEY_CODE: &str = "deprecated_key";
const RELATIVE_TOLERANCE: f64 = 1e-6;
const DECIMAL_NOTATION_FLOOR: f64 = 1e-6;
const DECIMAL_NOTATION_CEILING: f64 = 1e21;

pub fn flatten(table: &toml::Table) -> IndexMap<String, toml::Value> {
    fn walk(prefix: &str, node: &toml::Table, out: &mut IndexMap<String, toml::Value>) {
        for (name, value) in node {
            let key = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };
            match value {
                toml::Value::Table(inner) => walk(&key, inner, out),
                leaf => {
                    out.insert(key, leaf.clone());
                }
            }
        }
    }
    let mut out = IndexMap::new();
    walk("", table, &mut out);
    out
}

fn number(value: &toml::Value) -> Option<f64> {
    match value {
        toml::Value::Integer(number) => Some(*number as f64),
        toml::Value::Float(number) => Some(*number),
        _ => None,
    }
}

pub fn same_setting(a: &toml::Value, b: &toml::Value) -> bool {
    if let (Some(x), Some(y)) = (number(a), number(b)) {
        if x.is_nan() && y.is_nan() {
            return true;
        }
        let scale = 1.0_f64.max(x.abs()).max(y.abs());
        return (x - y).abs() <= RELATIVE_TOLERANCE * scale;
    }
    match (a, b) {
        (toml::Value::Array(a), toml::Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_setting(a, b))
        }
        (toml::Value::Table(a), toml::Value::Table(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, value)| b.get(key).is_some_and(|other| same_setting(value, other)))
        }
        _ => same_value(a, b),
    }
}

fn rounded_to_significant_digits(value: f64) -> f64 {
    format!("{value:.precision$e}", precision = SIGNIFICANT_DIGITS - 1)
        .parse()
        .unwrap_or(value)
}

pub fn float_text(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    let rounded = rounded_to_significant_digits(value);
    let magnitude = rounded.abs();
    if magnitude >= DECIMAL_NOTATION_CEILING {
        return format!("{rounded:e}").replace('e', "e+");
    }
    if magnitude != 0.0 && magnitude < DECIMAL_NOTATION_FLOOR {
        return format!("{rounded:e}");
    }
    let text = format!("{rounded}");
    if text.contains('.') {
        text
    } else {
        format!("{text}.0")
    }
}

pub fn plain(value: &toml::Value) -> String {
    match value {
        toml::Value::String(text) => format!("\"{text}\""),
        toml::Value::Integer(number) => number.to_string(),
        toml::Value::Float(number) => float_text(*number),
        toml::Value::Boolean(flag) => flag.to_string(),
        toml::Value::Datetime(stamp) => stamp.to_string(),
        toml::Value::Array(items) => {
            let items: Vec<String> = items.iter().map(plain).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::Table(table) => {
            let entries: Vec<String> = table
                .iter()
                .map(|(key, value)| format!("{key} = {}", plain(value)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
    }
}

pub fn display(value: &toml::Value) -> String {
    let text = plain(value);
    match truncated(&text, DISPLAY_MAX_CHARS) {
        Some(head) => format!("{head}..."),
        None => text,
    }
}

pub fn flattened_entries(raw: &str) -> Vec<String> {
    let values = parse_values(raw).unwrap_or_default();
    flatten(&values)
        .iter()
        .map(|(key, value)| format!("{key} = {}", display(value)))
        .collect()
}

pub fn json_plain(value: &Json) -> String {
    match value {
        Json::Null => "(unset)".to_owned(),
        Json::String(text) => format!("\"{text}\""),
        Json::Number(number) => match number.as_i64() {
            Some(int) if !number.is_f64() => int.to_string(),
            _ => float_text(number.as_f64().unwrap_or_default()),
        },
        Json::Bool(flag) => flag.to_string(),
        Json::Array(items) => {
            let items: Vec<String> = items.iter().map(json_plain).collect();
            format!("[{}]", items.join(", "))
        }
        Json::Object(table) => {
            let entries: Vec<String> = table
                .iter()
                .map(|(key, value)| format!("{key} = {}", json_plain(value)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
    }
}

pub fn json_display(value: &Json) -> String {
    let text = json_plain(value);
    match truncated(&text, DISPLAY_MAX_CHARS) {
        Some(head) => format!("{head}..."),
        None => text,
    }
}

pub fn collect_config(schema: &Schema, values: &Values, validate: Option<&ValidateReport>) -> ConfigReport {
    let mut report = ConfigReport::default();
    for (file, state) in &values.files {
        match &state.error {
            Some(error) => {
                report.parse_errors.insert(file.clone(), error.clone());
            }
            None if !state.exists => report.missing_files.push(file.clone()),
            None => {}
        }
    }
    for setting in &schema.settings {
        if report.parse_errors.contains_key(&setting.file) || values.not_in_file.contains(&setting.id) {
            continue;
        }
        let Some(value) = values.get(&setting.id) else {
            continue;
        };
        if values_equal(value, &setting.default) {
            continue;
        }
        report.deltas.push(ConfigDelta {
            file: setting.file.clone(),
            key: setting.key.clone(),
            value: json_display(value),
            default_value: json_display(&setting.default),
            note: None,
        });
    }
    for problem in validate.map(|report| report.problems.as_slice()).unwrap_or_default() {
        let delta = ConfigDelta {
            file: problem.file.clone().unwrap_or_default(),
            key: problem.key.clone().or_else(|| problem.id.clone()).unwrap_or_default(),
            value: String::new(),
            default_value: String::new(),
            note: Some(problem.message.clone()),
        };
        match problem.code.as_str() {
            DEPRECATED_KEY_CODE => report.legacy_keys.push(delta),
            UNKNOWN_KEY_CODE => report.unknown_keys.push(delta),
            _ => {}
        }
    }
    report
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_print_like_the_flutter_report() {
        assert_eq!(float_text(1.700_000_000_000_000_2), "1.7");
        assert_eq!(float_text(3.0), "3.0");
        assert_eq!(float_text(0.001), "0.001");
        assert_eq!(float_text(15.0), "15.0");
        assert_eq!(float_text(-0.25), "-0.25");
        assert_eq!(float_text(0.0), "0.0");
        assert_eq!(float_text(1.0 / 3.0), "0.3333333");
        assert_eq!(float_text(1e-7), "1e-7");
        assert_eq!(float_text(1.5e22), "1.5e+22");
        assert_eq!(float_text(f64::NAN), "NaN");
        assert_eq!(float_text(f64::NEG_INFINITY), "-Infinity");
    }

    #[test]
    fn values_print_in_toml_like_form() {
        let table: toml::Table = "a = \"x\"\nb = [1, 2.5, true]\nc = { d = \"e\", f = 2 }\n"
            .parse()
            .unwrap();
        assert_eq!(plain(&table["a"]), "\"x\"");
        assert_eq!(plain(&table["b"]), "[1, 2.5, true]");
        assert_eq!(plain(&table["c"]), "{d = \"e\", f = 2}");
        let long = toml::Value::String("x".repeat(200));
        let shown = display(&long);
        assert_eq!(shown.chars().count(), DISPLAY_MAX_CHARS + 3);
        assert!(shown.ends_with("..."));
    }

    #[test]
    fn numbers_compare_with_a_relative_tolerance() {
        let int = toml::Value::Integer(2);
        let float = toml::Value::Float(2.0);
        assert!(same_setting(&int, &float));
        assert!(same_setting(
            &toml::Value::Float(0.1 + 0.2),
            &toml::Value::Float(0.3)
        ));
        assert!(!same_setting(
            &toml::Value::Float(1.7),
            &toml::Value::Float(1.0)
        ));
        assert!(!same_setting(
            &toml::Value::Boolean(true),
            &toml::Value::Integer(1)
        ));
        let a = toml::Value::Array(vec![int.clone(), toml::Value::String("x".into())]);
        let b = toml::Value::Array(vec![float.clone(), toml::Value::String("x".into())]);
        assert!(same_setting(&a, &b));
    }

    #[test]
    fn nested_tables_flatten_to_dotted_keys_in_file_order() {
        let table: toml::Table = "z = 1\n[b]\ny = 2\n[b.c]\nx = [1]\n[a]\nw = 3\n"
            .parse()
            .unwrap();
        let keys: Vec<String> = flatten(&table).keys().cloned().collect();
        assert_eq!(keys, ["z", "b.y", "b.c.x", "a.w"]);
    }

    #[test]
    fn json_values_print_in_toml_like_form() {
        let value: Json = serde_json::json!({ "a": "x", "b": [1, 2.5, true], "n": null });
        assert_eq!(json_plain(&value["a"]), "\"x\"");
        assert_eq!(json_plain(&value["b"]), "[1, 2.5, true]");
        assert_eq!(json_plain(&value["n"]), "(unset)");
        assert_eq!(json_plain(&serde_json::json!(3.0)), "3.0");
        assert_eq!(json_display(&Json::String("x".repeat(200))).chars().count(), DISPLAY_MAX_CHARS + 3);
    }
}
