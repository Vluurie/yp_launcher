use toml_edit::{Decor, DocumentMut, Item, Key, RawString, Table};

pub(crate) fn raw_text(raw: Option<&RawString>) -> String {
    raw.and_then(RawString::as_str)
        .unwrap_or_default()
        .to_owned()
}

pub(crate) fn split_lines(text: &str) -> (Vec<&str>, &str) {
    let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
    let indent = match lines.last() {
        Some(last) if !last.ends_with('\n') => lines.pop().unwrap_or_default(),
        _ => "",
    };
    (lines, indent)
}

pub(crate) fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

pub(crate) fn is_blank_line(line: &str) -> bool {
    line.trim().is_empty()
}

pub(crate) fn key_prefix(table: &Table, key: &str) -> String {
    table
        .key(key)
        .map(|key| raw_text(key.leaf_decor().prefix()))
        .unwrap_or_default()
}

pub(crate) fn set_key_prefix(table: &mut Table, key: &str, prefix: &str) {
    if let Some(mut key) = table.key_mut(key) {
        key.leaf_decor_mut().set_prefix(prefix.to_owned());
    }
}

pub(crate) fn comment_block(prefix: &str) -> String {
    let (lines, _) = split_lines(prefix);
    lines
        .into_iter()
        .skip_while(|line| is_blank_line(line))
        .collect()
}

pub(crate) fn formatted_key(name: &str, prefix: &str) -> Key {
    Key::new(name).with_leaf_decor(Decor::new(prefix.to_owned(), " "))
}

pub(crate) fn value_keys(table: &Table) -> Vec<String> {
    table
        .iter()
        .filter(|(_, item)| item.is_value())
        .map(|(key, _)| key.to_owned())
        .collect()
}

pub(crate) fn standard_table_mut<'a>(
    doc: &'a mut DocumentMut,
    path: &[&str],
) -> Option<&'a mut Table> {
    let mut current = doc.as_table_mut();
    for part in path {
        current = match current.get_mut(part)? {
            Item::Table(table) if !table.is_dotted() => table,
            _ => return None,
        };
    }
    Some(current)
}

pub(crate) fn is_standard_table(item: Option<&Item>) -> bool {
    matches!(item, Some(Item::Table(table)) if !table.is_dotted())
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableLocation {
    pub position: isize,
    pub path: Vec<String>,
    pub array_index: Option<usize>,
}

fn collect_locations(table: &Table, path: &mut Vec<String>, out: &mut Vec<TableLocation>) {
    for (key, item) in table.iter() {
        match item {
            Item::Table(child) => {
                path.push(key.to_owned());
                if !child.is_dotted()
                    && let Some(position) = child.position()
                {
                    out.push(TableLocation {
                        position,
                        path: path.clone(),
                        array_index: None,
                    });
                }
                collect_locations(child, path, out);
                path.pop();
            }
            Item::ArrayOfTables(tables) => {
                path.push(key.to_owned());
                for (index, child) in tables.iter().enumerate() {
                    if let Some(position) = child.position() {
                        out.push(TableLocation {
                            position,
                            path: path.clone(),
                            array_index: Some(index),
                        });
                    }
                    collect_locations(child, path, out);
                }
                path.pop();
            }
            _ => {}
        }
    }
}

pub(crate) fn table_locations(doc: &DocumentMut) -> Vec<TableLocation> {
    let mut out = Vec::new();
    collect_locations(doc.as_table(), &mut Vec::new(), &mut out);
    out.sort_by_key(|location| location.position);
    out
}

fn table_at_location_mut<'a>(
    doc: &'a mut DocumentMut,
    location: &TableLocation,
) -> Option<&'a mut Table> {
    let (last, parents) = location.path.split_last()?;
    let mut current = doc.as_table_mut();
    for part in parents {
        current = match current.get_mut(part)? {
            Item::Table(table) => table,
            Item::ArrayOfTables(tables) => {
                let newest = tables.len().checked_sub(1)?;
                tables.get_mut(newest)?
            }
            _ => return None,
        };
    }
    match (current.get_mut(last)?, location.array_index) {
        (Item::Table(table), None) => Some(table),
        (Item::ArrayOfTables(tables), Some(index)) => tables.get_mut(index),
        _ => None,
    }
}

fn position_of(doc: &DocumentMut, path: &[&str]) -> Option<isize> {
    let mut current = doc.as_table();

    for part in path {
        current = match current.get(part)? {
            Item::Table(table) => table,
            _ => return None,
        };
    }

    current.position()
}

pub(crate) fn prepend_to_following(doc: &mut DocumentMut, table_path: &[&str], text: &str) {
    if text.is_empty() {
        return;
    }
    let after = if table_path.is_empty() {
        None
    } else {
        position_of(doc, table_path)
    };
    let following = table_locations(doc)
        .into_iter()
        .find(|location| after.is_none_or(|after| location.position > after));
    if let Some(location) = following
        && let Some(table) = table_at_location_mut(doc, &location)
    {
        let existing = raw_text(table.decor().prefix());
        table.decor_mut().set_prefix(format!("{text}{existing}"));
        return;
    }
    let trailing = raw_text(Some(doc.trailing()));
    doc.set_trailing(format!("{text}{trailing}"));
}

pub(crate) fn carry_prefix(
    doc: &mut DocumentMut,
    table_path: &[&str],
    removed_key: &str,
    carried: &str,
) {
    if carried.is_empty() {
        return;
    }
    let next_key = standard_table_mut(doc, table_path).and_then(|table| {
        let keys = value_keys(table);
        let index = keys.iter().position(|key| key == removed_key)?;
        keys.get(index + 1).cloned()
    });
    match next_key {
        Some(next) => {
            if let Some(table) = standard_table_mut(doc, table_path) {
                let existing = key_prefix(table, &next);
                set_key_prefix(table, &next, &format!("{carried}{existing}"));
            }
        }
        None => prepend_to_following(doc, table_path, carried),
    }
}

pub(crate) fn rebuild_order(table: &mut Table, order: &[String]) {
    let mut entries = Vec::with_capacity(order.len());
    for name in order {
        if let Some(entry) = table.remove_entry(name) {
            entries.push(entry);
        }
    }
    for (key, item) in entries {
        table.insert_formatted(&key, item);
    }
}

pub(crate) fn insert_value_after(table: &mut Table, anchor: &str, key: Key, item: Item) {
    let mut order: Vec<String> = table.iter().map(|(name, _)| name.to_owned()).collect();
    let name = key.get().to_owned();
    table.insert_formatted(&key, item);
    match order.iter().position(|existing| existing == anchor) {
        Some(index) => order.insert(index + 1, name),
        None => order.push(name),
    }
    rebuild_order(table, &order);
}

pub(crate) fn insert_value_first(table: &mut Table, key: Key, item: Item) {
    let mut order: Vec<String> = table.iter().map(|(name, _)| name.to_owned()).collect();
    let name = key.get().to_owned();
    table.insert_formatted(&key, item);
    order.insert(0, name);
    rebuild_order(table, &order);
}

pub(crate) fn ensure_blank_line_before_first_table(doc: &mut DocumentMut) {
    let Some(location) = table_locations(doc).into_iter().next() else {
        return;
    };
    if let Some(table) = table_at_location_mut(doc, &location) {
        let existing = raw_text(table.decor().prefix());
        let starts_blank = split_lines(&existing)
            .0
            .first()
            .is_some_and(|line| is_blank_line(line));
        if !starts_blank {
            table.decor_mut().set_prefix(format!("\n{existing}"));
        }
    }
}

fn shift_positions(table: &mut Table, from: isize) {
    for (_, item) in table.iter_mut() {
        match item {
            Item::Table(child) => {
                if let Some(position) = child.position()
                    && position >= from
                {
                    child.set_position(Some(position + 1));
                }

                shift_positions(child, from);
            }

            Item::ArrayOfTables(tables) => {
                for child in tables.iter_mut() {
                    if let Some(position) = child.position()
                        && position >= from
                    {
                        child.set_position(Some(position + 1));
                    }

                    shift_positions(child, from);
                }
            }

            _ => {}
        }
    }
}

pub(crate) fn insert_table_first(doc: &mut DocumentMut, name: &str, mut table: Table) {
    match table_locations(doc)
        .first()
        .map(|location| location.position)
    {
        Some(first) => {
            shift_positions(doc.as_table_mut(), first);
            table.set_position(Some(first));
        }

        None => {
            table.set_position(Some(1));
        }
    }

    doc.as_table_mut().insert(name, Item::Table(table));
}

pub(crate) fn append_table_last(doc: &mut DocumentMut, name: &str, mut table: Table) {
    let last = table_locations(doc)
        .last()
        .map(|location| location.position)
        .unwrap_or(0);

    table.set_position(Some(last + 1));

    doc.as_table_mut().insert(name, Item::Table(table));
}

pub(crate) fn template_entries(template: &str) -> Vec<(Key, Item)> {
    let Ok(doc) = template.parse::<DocumentMut>() else {
        return Vec::new();
    };
    doc.as_table()
        .iter()
        .filter_map(|(name, item)| {
            doc.as_table()
                .key(name)
                .map(|key| (key.clone(), item.clone()))
        })
        .collect()
}
