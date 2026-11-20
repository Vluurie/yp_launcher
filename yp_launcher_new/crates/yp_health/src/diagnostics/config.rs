use indexmap::IndexMap;
use yp_config::{ConfigFile, templates};
use yp_core::toml_io::{parse_values, same_value};

use super::model::{ConfigDelta, ConfigReport};
use super::text::truncated;

pub const DISPLAY_MAX_CHARS: usize = 160;
pub const SIGNIFICANT_DIGITS: usize = 7;
const RELATIVE_TOLERANCE: f64 = 1e-6;
const DECIMAL_NOTATION_FLOOR: f64 = 1e-6;
const DECIMAL_NOTATION_CEILING: f64 = 1e21;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyMatch {
    Exact(&'static str),
    Prefix(&'static str),
}

impl LegacyMatch {
    fn matches(self, key: &str) -> bool {
        match self {
            LegacyMatch::Exact(exact) => key == exact,
            LegacyMatch::Prefix(prefix) => key.starts_with(prefix),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegacyKey {
    pub file: ConfigFile,
    pub key: LegacyMatch,
    pub note: &'static str,
}

pub const LEGACY_KEYS: [LegacyKey; 6] = [
    LegacyKey {
        file: ConfigFile::Nams,
        key: LegacyMatch::Prefix("mouse."),
        note: "moved to naiom.toml, NAMS ignores it once naiom.toml exists",
    },
    LegacyKey {
        file: ConfigFile::Nams,
        key: LegacyMatch::Prefix("heap."),
        note: "removed, the heaps grow on their own",
    },
    LegacyKey {
        file: ConfigFile::Nams,
        key: LegacyMatch::Exact("cutscene.hd_cutscenes"),
        note: "removed",
    },
    LegacyKey {
        file: ConfigFile::Nams,
        key: LegacyMatch::Prefix("content_"),
        note: "old content switch, only read while [content] is missing",
    },
    LegacyKey {
        file: ConfigFile::LodMod,
        key: LegacyMatch::Exact("disable_fake_hdr"),
        note: "removed from NAMS",
    },
    LegacyKey {
        file: ConfigFile::LodMod,
        key: LegacyMatch::Exact("gi_min_light_extent"),
        note: "removed from NAMS",
    },
];

pub fn legacy_note(file: ConfigFile, key: &str) -> Option<&'static str> {
    LEGACY_KEYS
        .iter()
        .find(|legacy| legacy.file == file && legacy.key.matches(key))
        .map(|legacy| legacy.note)
}

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

pub fn collect_config(raws: &IndexMap<ConfigFile, String>) -> ConfigReport {
    let mut report = ConfigReport::default();
    for file in ConfigFile::ALL {
        let raw = raws.get(&file).map(String::as_str).unwrap_or_default();
        if raw.trim().is_empty() {
            report.missing_files.push(file);
            continue;
        }
        let parsed = match parse_values(raw) {
            Ok(parsed) => parsed,
            Err(error) => {
                report.parse_errors.insert(file, error.to_string());
                continue;
            }
        };
        let defaults = flatten(&parse_values(templates::template(file)).unwrap_or_default());
        for (key, value) in flatten(&parsed) {
            let default = defaults.get(&key);
            if default.is_some_and(|default| same_setting(&value, default)) {
                continue;
            }
            let note = if default.is_some() {
                None
            } else {
                legacy_note(file, &key)
            };
            let delta = ConfigDelta {
                file,
                value: display(&value),
                default_value: default.map(display).unwrap_or_default(),
                note,
                key,
            };
            if default.is_some() {
                report.deltas.push(delta);
            } else if note.is_some() {
                report.legacy_keys.push(delta);
            } else {
                report.unknown_keys.push(delta);
            }
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
    fn legacy_notes_match_prefixes_and_exact_keys_per_file() {
        assert!(
            legacy_note(ConfigFile::Nams, "mouse.sensitivity")
                .unwrap()
                .contains("naiom.toml")
        );
        assert_eq!(
            legacy_note(ConfigFile::Nams, "cutscene.hd_cutscenes"),
            Some("removed")
        );
        assert_eq!(
            legacy_note(ConfigFile::Nams, "cutscene.hd_cutscenes_extra"),
            None
        );
        assert!(legacy_note(ConfigFile::Nams, "content_weapons").is_some());
        assert_eq!(legacy_note(ConfigFile::LodMod, "mouse.sensitivity"), None);
        assert_eq!(
            legacy_note(ConfigFile::LodMod, "disable_fake_hdr"),
            Some("removed from NAMS")
        );
    }
}
