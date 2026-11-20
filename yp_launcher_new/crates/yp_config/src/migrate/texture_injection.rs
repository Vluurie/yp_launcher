use toml_edit::{DocumentMut, Item, Value};

use super::edit::{
    comment_block, formatted_key, is_blank_line, key_prefix, prepend_to_following, raw_text,
    set_key_prefix, split_lines, template_entries, value_keys,
};
use crate::fields::texture_injection;
use crate::templates;

pub const LEGACY_PRELOAD_KEYS: [&str; 2] = ["preload_max_dimension", "preload_all"];
pub const LEGACY_VRAM_BUDGET_MB: i64 = 4096;

fn is_legacy_preload(key: &str) -> bool {
    LEGACY_PRELOAD_KEYS
        .iter()
        .any(|legacy| key.starts_with(legacy))
}

fn drop_first_blank_line(prefix: &str) -> String {
    let (lines, indent) = split_lines(prefix);
    let mut lines = lines.into_iter().peekable();
    if lines.peek().is_some_and(|line| is_blank_line(line)) {
        lines.next();
    }
    let kept: String = lines.collect();
    format!("{kept}{indent}")
}

fn remove_legacy_preload(doc: &mut DocumentMut) -> bool {
    let root = doc.as_table_mut();
    let keys = value_keys(root);
    if !keys.iter().any(|key| is_legacy_preload(key)) {
        return false;
    }
    let mut carried = String::new();
    let mut skip_next_blank = false;
    for key in &keys {
        let prefix = key_prefix(root, key);
        if is_legacy_preload(key) {
            let (lines, _) = split_lines(&prefix);
            let mut lines: Vec<&str> = lines;
            if skip_next_blank && lines.first().is_some_and(|line| is_blank_line(line)) {
                lines.remove(0);
            }
            carried.extend(lines.into_iter().map(str::to_owned));
            root.remove(key);
            skip_next_blank = true;
            continue;
        }
        if skip_next_blank {
            let trimmed = drop_first_blank_line(&prefix);
            set_key_prefix(root, key, &format!("{carried}{trimmed}"));
            carried.clear();
            skip_next_blank = false;
        }
    }
    if skip_next_blank {
        let following = carried.clone();
        let tail = raw_text(Some(doc.trailing()));
        if crate::migrate::edit::table_locations(doc).is_empty() {
            doc.set_trailing(format!("{following}{}", drop_first_blank_line(&tail)));
        } else {
            prepend_to_following(doc, &[], &following);
        }
    }

    let vram = texture_injection::VRAM_BUDGET_MB.key();
    let root = doc.as_table_mut();
    if root.get(vram).and_then(Item::as_integer) == Some(LEGACY_VRAM_BUDGET_MB)
        && let Some(value) = root.get_mut(vram).and_then(Item::as_value_mut)
    {
        let decor = value.decor().clone();
        *value = Value::from(0);
        *value.decor_mut() = decor;
    }
    true
}

pub fn migrate_texture_injection(raw: &str) -> Option<String> {
    let mut doc = raw.parse::<DocumentMut>().ok()?;
    let mut modified = remove_legacy_preload(&mut doc);
    for (key, item) in template_entries(templates::TEXTURE_INJECTION_ADDITIONS) {
        let root = doc.as_table_mut();
        if root.contains_key(key.get()) {
            continue;
        }
        let comments = comment_block(&raw_text(key.leaf_decor().prefix()));
        root.insert_formatted(&formatted_key(key.get(), &format!("\n\n{comments}")), item);
        modified = true;
    }
    modified.then(|| doc.to_string()).filter(|text| text != raw)
}
