use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use yp_i18n::Locale;

pub fn arb_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/l10n")
}

fn read_arb_file(path: &Path) -> Map<String, Value> {
    let text = std::fs::read_to_string(path).unwrap();
    match serde_json::from_str::<Value>(&text).unwrap() {
        Value::Object(map) => map,
        _ => panic!("{} is not an object", path.display()),
    }
}

pub fn extra_arb_dir() -> PathBuf {
    arb_dir().join("gpui")
}

pub fn extra_set_names() -> BTreeSet<String> {
    let mut sets = BTreeSet::new();
    let Ok(listing) = std::fs::read_dir(extra_arb_dir()) else {
        return sets;
    };
    for entry in listing.flatten() {
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if let Some(set) = stem.strip_suffix("_en") {
            sets.insert(set.to_owned());
        }
    }
    sets
}

pub fn read_extra_set(set: &str, locale: Locale) -> Map<String, Value> {
    let template = read_arb_file(&extra_arb_dir().join(format!("{set}_en.arb")));
    let path = extra_arb_dir().join(format!("{set}_{}.arb", locale.code()));
    if !path.is_file() {
        return template;
    }
    let mut document = read_arb_file(&path);
    for (key, value) in template {
        if !document.contains_key(&key) {
            document.insert(key, value);
        }
    }
    document
}

pub fn read_arb(locale: Locale) -> Map<String, Value> {
    let path = arb_dir().join(format!("app_{}.arb", locale.code()));
    let mut document = read_arb_file(&path);
    for set in extra_set_names() {
        for (key, value) in read_extra_set(&set, Locale::En) {
            let localized = read_extra_set(&set, locale)
                .get(&key)
                .cloned()
                .unwrap_or(value);
            document.insert(key, localized);
        }
    }
    document
}

pub fn message_keys(arb: &Map<String, Value>) -> Vec<String> {
    arb.keys()
        .filter(|key| !key.starts_with('@'))
        .cloned()
        .collect()
}

pub fn declared_placeholders(template: &Map<String, Value>, key: &str) -> Vec<(String, String)> {
    template
        .get(&format!("@{key}"))
        .and_then(|meta| meta.get("placeholders"))
        .and_then(Value::as_object)
        .map(|placeholders| {
            placeholders
                .iter()
                .map(|(name, spec)| {
                    let kind = spec
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or("String")
                        .to_owned();
                    (name.clone(), kind)
                })
                .collect()
        })
        .unwrap_or_default()
}
