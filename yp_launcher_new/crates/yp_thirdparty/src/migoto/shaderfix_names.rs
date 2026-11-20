use std::collections::HashSet;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

pub const FILE_NAME: &str = "nams.shaderfix_names.json";

fn file(migoto_dir: &Path) -> PathBuf {
    migoto_dir.join(FILE_NAME)
}

pub fn read(migoto_dir: &Path) -> IndexMap<String, String> {
    let Ok(text) = std::fs::read_to_string(file(migoto_dir)) else {
        return IndexMap::new();
    };
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(serde_json::Value::Object(map)) => map
            .into_iter()
            .map(|(key, value)| {
                let value = match value {
                    serde_json::Value::String(text) => text,
                    other => other.to_string(),
                };
                (key, value)
            })
            .collect(),
        _ => IndexMap::new(),
    }
}

pub fn assign<S: AsRef<str>>(migoto_dir: &Path, shader_fix_file_names: &[S], mod_name: &str) {
    let mut map = read(migoto_dir);
    let mut changed = false;
    for name in shader_fix_file_names {
        let name = name.as_ref();
        if map.contains_key(name) {
            continue;
        }
        map.insert(name.to_owned(), mod_name.to_owned());
        changed = true;
    }
    if changed {
        write(migoto_dir, &map);
    }
}

pub fn prune(migoto_dir: &Path, existing_file_names: &HashSet<String>) {
    let mut map = read(migoto_dir);
    let before = map.len();
    map.retain(|name, _| existing_file_names.contains(name));
    if map.len() != before {
        write(migoto_dir, &map);
    }
}

fn write(migoto_dir: &Path, map: &IndexMap<String, String>) {
    if let Ok(text) = serde_json::to_string_pretty(map) {
        let _ = std::fs::write(file(migoto_dir), text);
    }
}
