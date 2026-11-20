use std::path::Path;

pub use toml_edit::{DocumentMut, Item, Table, Value};
use toml_edit::{ArrayOfTables, Decor, Key, TableLike};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl std::fmt::Display for TomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => write!(f, "line {line}, column {column}: {}", self.message),
            _ => f.write_str(&self.message),
        }
    }
}

impl TomlError {
    pub fn into_error(self, path: Option<&Path>) -> Error {
        Error::TomlParse {
            path: path.map(Path::to_path_buf),
            message: self.to_string(),
        }
    }
}

pub fn parse(text: &str) -> std::result::Result<DocumentMut, TomlError> {
    text.parse::<DocumentMut>().map_err(|err| {
        let (line, column) = err
            .span()
            .map(|span| line_column(text, span.start))
            .map(|(line, column)| (Some(line), Some(column)))
            .unwrap_or((None, None));
        TomlError {
            message: err.message().to_owned(),
            line,
            column,
        }
    })
}

pub fn parse_or_empty(text: &str) -> DocumentMut {
    parse(text).unwrap_or_default()
}

pub fn parse_values(text: &str) -> std::result::Result<toml::Table, TomlError> {
    parse(text).map(|doc| values(&doc))
}

pub fn read_file(path: &Path) -> Result<Option<DocumentMut>> {
    match crate::fs::read_text_opt(path)? {
        None => Ok(None),
        Some(text) => parse(&text).map(Some).map_err(|err| err.into_error(Some(path))),
    }
}

pub fn values(doc: &DocumentMut) -> toml::Table {
    table_like_values(doc.as_table())
}

fn line_column(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map(|tail| tail.chars().count() + 1).unwrap_or(1);
    (line, column)
}

pub fn item_value(item: &Item) -> Option<toml::Value> {
    match item {
        Item::None => None,
        Item::Value(value) => Some(edit_value_to_value(value)),
        Item::Table(table) => Some(toml::Value::Table(table_like_values(table))),
        Item::ArrayOfTables(tables) => Some(toml::Value::Array(
            tables.iter().map(|table| toml::Value::Table(table_like_values(table))).collect(),
        )),
    }
}

fn table_like_values(table: &dyn TableLike) -> toml::Table {
    table
        .iter()
        .filter_map(|(key, item)| item_value(item).map(|value| (key.to_owned(), value)))
        .collect()
}

fn edit_value_to_value(value: &Value) -> toml::Value {
    match value {
        Value::String(text) => toml::Value::String(text.value().clone()),
        Value::Integer(number) => toml::Value::Integer(*number.value()),
        Value::Float(number) => toml::Value::Float(*number.value()),
        Value::Boolean(flag) => toml::Value::Boolean(*flag.value()),
        Value::Datetime(stamp) => toml::Value::Datetime(*stamp.value()),
        Value::Array(items) => toml::Value::Array(items.iter().map(edit_value_to_value).collect()),
        Value::InlineTable(table) => toml::Value::Table(table_like_values(table)),
    }
}

pub fn to_edit_value(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(text) => Value::from(text.as_str()),
        toml::Value::Integer(number) => Value::from(*number),
        toml::Value::Float(number) => Value::from(*number),
        toml::Value::Boolean(flag) => Value::from(*flag),
        toml::Value::Datetime(stamp) => Value::from(*stamp),
        toml::Value::Array(items) => Value::Array(items.iter().map(to_edit_value).collect()),
        toml::Value::Table(table) => {
            Value::InlineTable(table.iter().map(|(key, value)| (key.as_str(), to_edit_value(value))).collect())
        }
    }
}

fn to_edit_table(values: &toml::Table) -> Table {
    let mut table = Table::new();
    for (key, value) in values {
        table.insert(key, to_edit_item(value));
    }
    table
}

fn to_edit_item(value: &toml::Value) -> Item {
    match value {
        toml::Value::Table(table) => {
            let mut table = to_edit_table(table);
            table.set_implicit(!table.is_empty());
            Item::Table(table)
        }
        other => Item::Value(to_edit_value(other)),
    }
}

fn array_of_tables(value: &toml::Value) -> Option<ArrayOfTables> {
    let toml::Value::Array(items) = value else {
        return None;
    };
    if items.is_empty() {
        return None;
    }
    items
        .iter()
        .map(|item| match item {
            toml::Value::Table(table) => Some(to_edit_table(table)),
            _ => None,
        })
        .collect()
}

pub fn same_value(a: &toml::Value, b: &toml::Value) -> bool {
    match (a, b) {
        (toml::Value::Table(a), toml::Value::Table(b)) => same_values(a, b),
        (toml::Value::Array(a), toml::Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_value(a, b))
        }
        (toml::Value::Float(a), toml::Value::Float(b)) if a.is_nan() && b.is_nan() => true,
        _ => a == b,
    }
}

pub fn same_values(a: &toml::Table, b: &toml::Table) -> bool {
    a.len() == b.len() && a.iter().all(|(key, value)| b.get(key).is_some_and(|other| same_value(value, other)))
}

pub fn merged_values(original: &toml::Table, updates: &toml::Table) -> toml::Table {
    let mut merged = original.clone();
    for (key, update) in updates {
        let combined = match (merged.get(key), update) {
            (Some(toml::Value::Table(previous)), toml::Value::Table(nested)) => {
                toml::Value::Table(merged_values(previous, nested))
            }
            _ => update.clone(),
        };
        merged.insert(key.clone(), combined);
    }
    merged
}

fn table_at<'a>(doc: &'a mut DocumentMut, path: &[&str], create: bool) -> Option<&'a mut dyn TableLike> {
    let mut current: &mut dyn TableLike = doc.as_table_mut();
    for key in path {
        if create && !current.get(key).is_some_and(Item::is_table_like) {
            let mut table = Table::new();
            table.set_implicit(true);
            current.insert(key, Item::Table(table));
        }
        current = current.get_mut(key)?.as_table_like_mut()?;
    }
    Some(current)
}

fn replace_keeping_decor(existing: &mut Item, mut value: Value) {
    if let Item::Value(old) = existing {
        *value.decor_mut() = old.decor().clone();
    }
    *existing = Item::Value(value);
}

pub fn set_value(doc: &mut DocumentMut, path: &[&str], value: Value) {
    let Some((key, parents)) = path.split_last() else {
        return;
    };
    let Some(table) = table_at(doc, parents, true) else {
        return;
    };
    match table.get_mut(key) {
        Some(existing) if !existing.is_none() => replace_keeping_decor(existing, value),
        _ => {
            table.insert(key, Item::Value(value));
        }
    }
}

pub fn remove_key(doc: &mut DocumentMut, path: &[&str]) -> Option<Item> {
    let (key, parents) = path.split_last()?;
    table_at(doc, parents, false)?.remove(key)
}

pub fn rename_key(doc: &mut DocumentMut, table: &[&str], from: &str, to: &str) -> bool {
    let Some(table) = table_at(doc, table, false) else {
        return false;
    };
    if from == to || !table.contains_key(from) || table.contains_key(to) {
        return false;
    }
    let order: Vec<String> = table.iter().map(|(key, _)| key.to_owned()).collect();
    let mut entries = Vec::with_capacity(order.len());
    for name in &order {
        let key = table.key(name).cloned();
        if let (Some(key), Some(item)) = (key, table.remove(name)) {
            entries.push((key, item));
        }
    }
    for (key, item) in entries {
        let key = if key.get() == from {
            Key::new(to).with_leaf_decor(key.leaf_decor().clone())
        } else {
            key
        };
        if let toml_edit::Entry::Vacant(slot) = table.entry_format(&key) {
            slot.insert(item);
        }
    }
    true
}

pub fn comment_prefix(comment_lines: &[&str]) -> String {
    comment_lines
        .iter()
        .map(|line| {
            let trimmed = line.trim_end();
            if trimmed.starts_with('#') {
                format!("{trimmed}\n")
            } else if trimmed.is_empty() {
                "#\n".to_owned()
            } else {
                format!("# {trimmed}\n")
            }
        })
        .collect()
}

pub fn insert_with_comment(doc: &mut DocumentMut, table: &[&str], key: &str, value: Value, comment_lines: &[&str]) {
    let Some(target) = table_at(doc, table, true) else {
        return;
    };
    if let Some(existing) = target.get_mut(key).filter(|item| !item.is_none()) {
        replace_keeping_decor(existing, value);
        return;
    }
    let formatted = Key::new(key).with_leaf_decor(Decor::new(comment_prefix(comment_lines), " "));
    if let toml_edit::Entry::Vacant(slot) = target.entry_format(&formatted) {
        slot.insert(Item::Value(value));
    }
}

pub fn merge_values(doc: &mut DocumentMut, values: &toml::Table) {
    merge_into(doc.as_table_mut(), values);
}

fn merge_into(table: &mut dyn TableLike, updates: &toml::Table) {
    for (key, update) in updates {
        let nested = match update {
            toml::Value::Table(nested) => Some(nested),
            _ => None,
        };
        match (table.get_mut(key), nested) {
            (Some(existing), Some(nested)) if existing.is_table_like() => {
                if let Some(child) = existing.as_table_like_mut() {
                    merge_into(child, nested);
                }
            }
            (Some(existing), _) if !existing.is_none() => {
                if item_value(existing).is_some_and(|current| same_value(&current, update)) {
                    continue;
                }
                replace_item(existing, update);
            }
            (_, Some(nested)) => {
                let mut fresh = Table::new();
                fresh.set_implicit(!nested.is_empty());
                table.insert(key, Item::Table(fresh));
                if let Some(child) = table.get_mut(key).and_then(Item::as_table_like_mut) {
                    merge_into(child, nested);
                }
            }
            (_, None) => {
                table.insert(key, Item::Value(to_edit_value(update)));
            }
        }
    }
}

fn replace_item(existing: &mut Item, update: &toml::Value) {
    if existing.is_array_of_tables()
        && let Some(tables) = array_of_tables(update)
    {
        *existing = Item::ArrayOfTables(tables);
        return;
    }
    match update {
        toml::Value::Table(_) => *existing = to_edit_item(update),
        other => replace_keeping_decor(existing, to_edit_value(other)),
    }
}

pub fn to_string_verified(doc: &DocumentMut, expected: &toml::Table) -> Result<String> {
    verify_text(doc.to_string(), expected)
}

fn uses_crlf(src: &str) -> bool {
    src.find('\n').is_some_and(|index| src[..index].ends_with('\r'))
}

fn with_crlf(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 16);
    let mut previous = '\0';
    for c in text.chars() {
        if c == '\n' && previous != '\r' {
            out.push('\r');
        }
        out.push(c);
        previous = c;
    }
    out
}

pub fn to_string_verified_like(src: &str, doc: &DocumentMut, expected: &toml::Table) -> Result<String> {
    let text = doc.to_string();
    let text = if uses_crlf(src) { with_crlf(&text) } else { text };
    verify_text(text, expected)
}

fn verify_text(text: String, expected: &toml::Table) -> Result<String> {
    let reparsed: toml::Table = toml::from_str(&text).map_err(|err| Error::TomlParse {
        path: None,
        message: err.message().to_owned(),
    })?;
    if same_values(&reparsed, expected) {
        Ok(text)
    } else {
        Err(Error::TomlRoundTrip { path: None })
    }
}

pub fn update_text(src: &str, edits: &[(&[&str], Value)]) -> Result<String> {
    let mut doc = parse(src).map_err(|err| err.into_error(None))?;
    for (path, value) in edits {
        set_value(&mut doc, path, value.clone());
    }
    let expected = values(&doc);
    to_string_verified_like(src, &doc, &expected)
}

pub fn update_toml(src: &str, updates: &toml::Table) -> Result<String> {
    let mut doc = parse(src).map_err(|err| err.into_error(None))?;
    let expected = merged_values(&values(&doc), updates);
    merge_values(&mut doc, updates);
    to_string_verified_like(src, &doc, &expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_errors_carry_line_and_column() {
        let err = parse("a = 1\nb = = 2\n").unwrap_err();
        assert_eq!(err.line, Some(2));
        assert!(err.column.is_some());
    }

    #[test]
    fn values_reads_typed_table() {
        let doc = parse("# c\n[core]\nlevel = 3\n").unwrap();
        let table = values(&doc);
        assert_eq!(table["core"]["level"].as_integer(), Some(3));
    }
}
