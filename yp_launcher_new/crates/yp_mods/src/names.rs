use indexmap::IndexMap;
use yp_core::GamePaths;

use crate::text::dart_trim;

fn has_game_dir(game: &GamePaths) -> bool {
    !game.root().as_os_str().is_empty()
}

pub fn load(game: &GamePaths) -> IndexMap<String, String> {
    if !has_game_dir(game) {
        return IndexMap::new();
    }
    let Ok(text) = std::fs::read_to_string(game.mod_names_json()) else {
        return IndexMap::new();
    };
    let Ok(serde_json::Value::Object(decoded)) = serde_json::from_str::<serde_json::Value>(&text)
    else {
        return IndexMap::new();
    };
    decoded
        .into_iter()
        .filter_map(|(key, value)| match value {
            serde_json::Value::String(name) if !dart_trim(&name).is_empty() => Some((key, name)),
            _ => None,
        })
        .collect()
}

pub fn set_name(game: &GamePaths, mod_id: &str, name: Option<&str>) {
    if !has_game_dir(game) {
        return;
    }
    let mut names = load(game);
    match name.map(dart_trim).filter(|trimmed| !trimmed.is_empty()) {
        Some(trimmed) => {
            names.insert(mod_id.to_owned(), trimmed.to_owned());
        }
        None => {
            names.shift_remove(mod_id);
        }
    }
    let file = game.mod_names_json();
    if names.is_empty() {
        if file.is_file() {
            let _ = std::fs::remove_file(&file);
        }
        return;
    }
    let Ok(rendered) = serde_json::to_string(&names) else {
        return;
    };
    let _ = yp_core::fs::write_text_atomic(&file, &rendered);
}
