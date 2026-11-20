use std::collections::BTreeSet;

use toml_edit::{DocumentMut, Item, Table, Value};

use super::edit::{
    append_table_last, carry_prefix, comment_block, ensure_blank_line_before_first_table,
    formatted_key, insert_table_first, insert_value_after, is_blank_line, is_comment_line,
    is_standard_table, key_prefix, prepend_to_following, raw_text, set_key_prefix, split_lines,
    standard_table_mut, table_locations, template_entries, value_keys,
};
use crate::fields::{bindings, content, nams};
use crate::naiom_keys;
use crate::templates;

pub const LEGACY_CONTENT_KEYS: [(&str, &[&str]); 9] = [
    (
        "content_items",
        &[
            "custom_items",
            "custom_weapons",
            "weapon_patches",
            "custom_outfits",
            "custom_hairsprays",
            "shop_items",
            "item_shortcut_menu",
        ],
    ),
    ("content_accessories", &["accessories"]),
    (
        "content_assemble_meshes",
        &["player_mesh_swaps", "mesh_visibility"],
    ),
    (
        "content_effect_areas",
        &["effect_areas", "effect_type_mute", "global_effects"],
    ),
    (
        "content_quest_integration",
        &[
            "custom_quests",
            "custom_mails",
            "custom_voice_lines",
            "custom_maps",
            "custom_music",
            "ambient_sounds",
            "custom_sounds",
            "script_groups",
            "lighting_filters",
            "outfit_selector_npc",
            "buddy_outfit_menu",
            "cutscene_replacements",
        ],
    ),
    ("content_effects_applier", &["effect_rules"]),
    ("content_equip_tracker", &[]),
    ("content_mcd", &["custom_texts"]),
    ("content_buddy_ruby_selector", &["buddy_outfit_menu"]),
];

pub const MOUSE_KEY_RENAMES: [(&str, &str); 3] = [
    ("fix_aim_acceleration", "aim_mode"),
    ("disable_pod_pet", "misc_disable_pod_pet"),
    ("debug_menu_key", "misc_open_debug_menu"),
];

pub const MOUSE_SECTION: &str = "mouse";
pub const CUTSCENE_SECTION: &str = "cutscene";
pub const LEGACY_EVADE_KEY: &str = "evade_key";
const LEGACY_EVADE_COMMENT: &str = "# Virtual key code for dedicated evade/dodge. 0 = disabled.";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NamsMigration {
    pub text: Option<String>,
    pub evade_key_name: Option<&'static str>,
}

pub fn migrate_nams(raw: &str) -> NamsMigration {
    let Ok(mut doc) = raw.parse::<DocumentMut>() else {
        return NamsMigration::default();
    };
    let mut modified = add_top_level_keys(&mut doc);
    modified |= migrate_content_section(&mut doc);
    modified |= rename_mouse_keys(&mut doc);
    let (evade_removed, evade_key_name) = take_legacy_evade_key(&mut doc);
    modified |= evade_removed;
    if let Some(name) = evade_key_name {
        modified |= bind_unbound_evade(&mut doc, name);
    }
    modified |= migrate_cutscene_section(&mut doc);

    let text = modified.then(|| doc.to_string()).filter(|text| text != raw);
    NamsMigration {
        text,
        evade_key_name,
    }
}

fn add_top_level_keys(doc: &mut DocumentMut) -> bool {
    let mut modified = false;
    for (key, item) in template_entries(templates::NAMS_ADDITIONS) {
        let name = key.get().to_owned();
        let root = doc.as_table_mut();
        if root.contains_key(&name) {
            continue;
        }
        let comments = comment_block(&raw_text(key.leaf_decor().prefix()));
        let anchor = if name == nams::VALIDATE_SCRIPTS.key() {
            Some(nams::VALIDATE_MODEL_DATA.key())
        } else if name == nams::DISABLE_CONTENT_FEATURES.key() {
            Some(nams::DISABLE_PLUGIN_LOADING.key())
        } else {
            None
        };
        let anchored = anchor.filter(|anchor| root.get(anchor).is_some_and(Item::is_value));
        match anchored {
            Some(anchor) => {
                let prefix = if comments.is_empty() {
                    String::new()
                } else {
                    format!("\n{comments}")
                };
                insert_value_after(root, anchor, formatted_key(&name, &prefix), item);
            }
            None if name == nams::VALIDATE_SCRIPTS.key() => continue,
            None => {
                root.insert_formatted(&formatted_key(&name, &format!("\n{comments}")), item);
                ensure_blank_line_before_first_table(doc);
            }
        }
        modified = true;
    }
    modified
}

fn legacy_off_features(root: &Table) -> BTreeSet<&'static str> {
    LEGACY_CONTENT_KEYS
        .iter()
        .filter(|(legacy, _)| root.get(legacy).and_then(Item::as_bool) == Some(false))
        .flat_map(|(_, features)| features.iter().copied())
        .collect()
}

fn is_legacy_content_key(name: &str) -> bool {
    LEGACY_CONTENT_KEYS
        .iter()
        .any(|(legacy, _)| *legacy == name)
}

enum RootLine {
    Text(String),
    Key(String),
}

fn remove_legacy_content_keys(doc: &mut DocumentMut) -> bool {
    let root = doc.as_table_mut();
    let keys = value_keys(root);
    if !keys.iter().any(|key| is_legacy_content_key(key)) {
        return false;
    }

    let mut lines: Vec<RootLine> = Vec::new();
    let mut indents: Vec<(String, String)> = Vec::new();
    for key in &keys {
        let prefix = key_prefix(root, key);
        let (prefix_lines, indent) = split_lines(&prefix);
        lines.extend(
            prefix_lines
                .into_iter()
                .map(|line| RootLine::Text(line.to_owned())),
        );
        lines.push(RootLine::Key(key.clone()));
        indents.push((key.clone(), indent.to_owned()));
    }

    let mut drop = vec![false; lines.len()];
    for index in 0..lines.len() {
        let RootLine::Key(key) = &lines[index] else {
            continue;
        };
        if !is_legacy_content_key(key) {
            continue;
        }
        drop[index] = true;
        let mut previous = index;
        while previous > 0 {
            previous -= 1;
            match &lines[previous] {
                RootLine::Text(text) if is_comment_line(text) => drop[previous] = true,
                _ => break,
            }
        }
    }

    let mut pending = String::new();
    let mut last_kept_blank = true;
    let mut after_drop = false;
    let mut kept_keys: Vec<(String, String)> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if drop[index] {
            after_drop = true;
            continue;
        }
        match line {
            RootLine::Text(text) => {
                let blank = is_blank_line(text);
                if blank && last_kept_blank && after_drop {
                    continue;
                }
                pending.push_str(text);
                last_kept_blank = blank;
            }
            RootLine::Key(key) => {
                let indent = indents
                    .iter()
                    .find(|(name, _)| name == key)
                    .map(|(_, indent)| indent.as_str())
                    .unwrap_or_default();
                kept_keys.push((key.clone(), format!("{pending}{indent}")));
                pending.clear();
                last_kept_blank = false;
            }
        }
        after_drop = false;
    }

    for key in keys.iter().filter(|key| is_legacy_content_key(key)) {
        root.remove(key);
    }
    for (key, prefix) in &kept_keys {
        set_key_prefix(root, key, prefix);
    }

    if after_drop && last_kept_blank {
        strip_leading_blank_lines_of_following(doc);
    }
    prepend_to_following(doc, &[], &pending);
    true
}

fn strip_leading_blank_lines_of_following(doc: &mut DocumentMut) {
    let Some(location) = table_locations(doc).into_iter().next() else {
        let trailing = raw_text(Some(doc.trailing()));
        let (lines, indent) = split_lines(&trailing);
        let kept: String = lines
            .into_iter()
            .skip_while(|line| is_blank_line(line))
            .collect();
        doc.set_trailing(format!("{kept}{indent}"));
        return;
    };
    let path: Vec<&str> = location.path.iter().map(String::as_str).collect();
    if location.array_index.is_none()
        && let Some(table) = standard_table_mut(doc, &path)
    {
        let prefix = raw_text(table.decor().prefix());
        let (lines, indent) = split_lines(&prefix);
        let kept: String = lines
            .into_iter()
            .skip_while(|line| is_blank_line(line))
            .collect();
        table.decor_mut().set_prefix(format!("{kept}{indent}"));
    }
}

fn template_content_table() -> Option<Table> {
    let doc = templates::NAMS.parse::<DocumentMut>().ok()?;
    match doc.as_table().get(content::SECTION)? {
        Item::Table(table) => Some(table.clone()),
        _ => None,
    }
}

fn migrate_content_section(doc: &mut DocumentMut) -> bool {
    let existing_kind = doc
        .as_table()
        .get(content::SECTION)
        .map(|item| is_standard_table(Some(item)));
    let off = match existing_kind {
        None => legacy_off_features(doc.as_table()),
        Some(_) => BTreeSet::new(),
    };
    let mut modified = remove_legacy_content_keys(doc);
    let Some(template) = template_content_table() else {
        return modified;
    };

    match existing_kind {
        None => {
            let mut table = template;
            for feature in content::fields() {
                if off.contains(feature.key())
                    && let Some(item) = table.get_mut(feature.key())
                    && let Some(value) = item.as_value_mut()
                {
                    let decor = value.decor().clone();
                    *value = Value::from(false);
                    *value.decor_mut() = decor;
                }
            }
            let header = comment_block(&raw_text(table.decor().prefix()));
            table.decor_mut().set_prefix(format!("\n{header}"));
            insert_table_first(doc, content::SECTION, table);
            ensure_blank_line_after_content(doc);
            modified = true;
        }
        Some(true) => {
            if let Some(target) = standard_table_mut(doc, &[content::SECTION]) {
                for feature in content::fields() {
                    if target.contains_key(feature.key()) {
                        continue;
                    }
                    let prefix = comment_block(&key_prefix(&template, feature.key()));
                    if let Some(item) = template.get(feature.key()) {
                        target
                            .insert_formatted(&formatted_key(feature.key(), &prefix), item.clone());
                        modified = true;
                    }
                }
            }
        }
        Some(false) => {}
    }
    modified
}

fn ensure_blank_line_after_content(doc: &mut DocumentMut) {
    let locations = table_locations(doc);
    let Some(content_position) = locations
        .iter()
        .find(|location| location.path == [content::SECTION])
        .map(|location| location.position)
    else {
        return;
    };
    if let Some(next) = locations
        .iter()
        .find(|location| location.position > content_position)
    {
        let path: Vec<&str> = next.path.iter().map(String::as_str).collect();
        if next.array_index.is_none()
            && let Some(table) = standard_table_mut(doc, &path)
        {
            let prefix = raw_text(table.decor().prefix());
            if !split_lines(&prefix)
                .0
                .first()
                .is_some_and(|line| is_blank_line(line))
            {
                table.decor_mut().set_prefix(format!("\n{prefix}"));
            }
        }
    }
}

fn rename_mouse_keys(doc: &mut DocumentMut) -> bool {
    let mut modified = false;
    for (from, to) in MOUSE_KEY_RENAMES {
        modified |= yp_core::toml_io::rename_key(doc, &[MOUSE_SECTION], from, to);
    }
    modified
}

fn take_legacy_evade_key(doc: &mut DocumentMut) -> (bool, Option<&'static str>) {
    let Some(mouse) = standard_table_mut(doc, &[MOUSE_SECTION]) else {
        return (false, None);
    };
    if !mouse.get(LEGACY_EVADE_KEY).is_some_and(Item::is_value) {
        return (false, None);
    }
    let name = mouse
        .get(LEGACY_EVADE_KEY)
        .and_then(Item::as_integer)
        .filter(|vk| *vk != 0)
        .and_then(naiom_keys::vk_to_key_name);
    let prefix = key_prefix(mouse, LEGACY_EVADE_KEY);
    let (lines, _) = split_lines(&prefix);
    let mut dropped_comment = false;
    let carried: String = lines
        .into_iter()
        .filter(|line| {
            if !dropped_comment && line.trim_end() == LEGACY_EVADE_COMMENT {
                dropped_comment = true;
                return false;
            }
            true
        })
        .collect();
    carry_prefix(doc, &[MOUSE_SECTION], LEGACY_EVADE_KEY, &carried);
    if let Some(mouse) = standard_table_mut(doc, &[MOUSE_SECTION]) {
        mouse.remove(LEGACY_EVADE_KEY);
    }
    (true, name)
}

fn bind_unbound_evade(doc: &mut DocumentMut, name: &str) -> bool {
    let evade = bindings::NON_STANDARD_EVADE.key();
    let unbound = standard_table_mut(doc, &[MOUSE_SECTION, "bindings"])
        .and_then(|table| table.get(evade))
        .and_then(Item::as_str)
        == Some("");
    if unbound {
        yp_core::toml_io::set_value(doc, &[MOUSE_SECTION, "bindings", evade], Value::from(name));
    }
    unbound
}

fn migrate_cutscene_section(doc: &mut DocumentMut) -> bool {
    let root = doc.as_table();
    if !root.contains_key(CUTSCENE_SECTION) {
        return append_default_cutscene_section(doc);
    }
    let Some(cutscene) = standard_table_mut(doc, &[CUTSCENE_SECTION]) else {
        return false;
    };
    let mut modified = false;
    for (key, item) in template_entries(templates::NAMS_CUTSCENE_KEYS) {
        if cutscene.contains_key(key.get()) {
            continue;
        }
        cutscene.insert_formatted(&key, item);
        modified = true;
    }
    modified
}

fn append_default_cutscene_section(doc: &mut DocumentMut) -> bool {
    let Ok(section) = templates::NAMS_CUTSCENE_SECTION.parse::<DocumentMut>() else {
        return false;
    };
    let Some(Item::Table(table)) = section.as_table().get(CUTSCENE_SECTION) else {
        return false;
    };
    let mut table = table.clone();
    let header = raw_text(table.decor().prefix());
    let trailing = raw_text(Some(doc.trailing()));
    table.decor_mut().set_prefix(format!("{trailing}{header}"));
    doc.set_trailing("");
    append_table_last(doc, CUTSCENE_SECTION, table);
    true
}
