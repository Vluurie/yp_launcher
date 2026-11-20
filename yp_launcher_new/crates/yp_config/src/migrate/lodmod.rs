use std::collections::BTreeSet;

use toml_edit::{DocumentMut, Item, TableLike};

use super::edit::{
    formatted_key, insert_value_first, key_prefix, prepend_to_following, value_keys,
};
use crate::fields::lodmod::{self, HIGH_GRIDS_SECTION};

fn grid_keys() -> BTreeSet<&'static str> {
    lodmod::ALL
        .iter()
        .filter(|field| field.meta().section == Some(HIGH_GRIDS_SECTION))
        .map(|field| field.meta().key)
        .collect()
}

fn root_keys() -> Vec<&'static str> {
    lodmod::ALL
        .iter()
        .filter(|field| field.meta().section.is_none())
        .map(|field| field.meta().key)
        .collect()
}

pub fn migrated_values(values: &toml::Table) -> Option<(toml::Table, BTreeSet<&'static str>)> {
    let existing_grids = values.get(HIGH_GRIDS_SECTION);
    if existing_grids.is_some_and(|grids| !grids.is_table()) {
        return None;
    }
    let mut migrated = values.clone();
    let mut grids = existing_grids
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let grid_keys = grid_keys();
    let misplaced: BTreeSet<&'static str> = root_keys()
        .into_iter()
        .filter(|key| !grid_keys.contains(key) && grids.contains_key(*key))
        .collect();
    for key in &misplaced {
        if let Some(value) = grids.remove(*key) {
            migrated.entry((*key).to_owned()).or_insert(value);
        }
    }
    for field in lodmod::ALL {
        let meta = field.meta();
        let target = if meta.section.is_none() {
            &mut migrated
        } else {
            &mut grids
        };
        target
            .entry(meta.key.to_owned())
            .or_insert_with(|| field.default_toml());
    }
    migrated.insert(HIGH_GRIDS_SECTION.to_owned(), toml::Value::Table(grids));
    Some((migrated, misplaced))
}

fn grids_table_mut(doc: &mut DocumentMut) -> Option<&mut dyn TableLike> {
    doc.as_table_mut()
        .get_mut(HIGH_GRIDS_SECTION)
        .and_then(Item::as_table_like_mut)
}

fn grids_is_standard_table(doc: &DocumentMut) -> bool {
    matches!(doc.as_table().get(HIGH_GRIDS_SECTION), Some(Item::Table(table)) if !table.is_dotted())
}

fn move_misplaced_to_root(
    doc: &mut DocumentMut,
    original: &toml::Table,
    misplaced: &BTreeSet<&'static str>,
) {
    let standard = grids_is_standard_table(doc);
    let order: Vec<String> = match doc.as_table().get(HIGH_GRIDS_SECTION) {
        Some(Item::Table(table)) => value_keys(table),
        Some(item) => item
            .as_table_like()
            .map(|table| table.iter().map(|(key, _)| key.to_owned()).collect())
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let mut recovered = Vec::new();
    for name in order
        .iter()
        .filter(|name| misplaced.contains(name.as_str()))
    {
        if standard {
            carry_grid_comment(doc, &order, name);
        }
        let Some(grids) = grids_table_mut(doc) else {
            return;
        };
        let Some(item) = grids.remove(name) else {
            continue;
        };
        if !original.contains_key(name.as_str()) {
            recovered.push((name.clone(), item));
        }
    }
    for (name, item) in recovered.into_iter().rev() {
        let mut item = item;
        if let Some(value) = item.as_value_mut() {
            value.decor_mut().set_prefix(" ");
            value.decor_mut().set_suffix("");
        }
        insert_value_first(doc.as_table_mut(), formatted_key(&name, ""), item);
    }
}

fn carry_grid_comment(doc: &mut DocumentMut, order: &[String], name: &str) {
    let Some(Item::Table(grids)) = doc.as_table().get(HIGH_GRIDS_SECTION) else {
        return;
    };
    let prefix = key_prefix(grids, name);
    if prefix.trim().is_empty() {
        return;
    }
    let index = order
        .iter()
        .position(|key| key == name)
        .unwrap_or(order.len());
    let next = order[index + 1..]
        .iter()
        .find(|key| grids.contains_key(key))
        .cloned();
    match next {
        Some(next) => {
            if let Some(Item::Table(grids)) = doc.as_table_mut().get_mut(HIGH_GRIDS_SECTION) {
                let existing = key_prefix(grids, &next);
                super::edit::set_key_prefix(grids, &next, &format!("{prefix}{existing}"));
            }
        }
        None => prepend_to_following(doc, &[HIGH_GRIDS_SECTION], &prefix),
    }
}

pub fn migrate_lodmod(raw: &str) -> String {
    let Ok(mut doc) = raw.parse::<DocumentMut>() else {
        return raw.to_owned();
    };
    let original = yp_core::toml_io::values(&doc);
    let Some((migrated, misplaced)) = migrated_values(&original) else {
        return raw.to_owned();
    };
    if yp_core::toml_io::same_values(&migrated, &original) {
        return raw.to_owned();
    }
    move_misplaced_to_root(&mut doc, &original, &misplaced);
    yp_core::toml_io::merge_values(&mut doc, &migrated);
    match yp_core::toml_io::to_string_verified(&doc, &migrated) {
        Ok(text) => text,
        Err(err) => {
            log::warn!(
                "lodmod.toml migration did not round-trip, leaving the file as it is: {err}"
            );
            raw.to_owned()
        }
    }
}
