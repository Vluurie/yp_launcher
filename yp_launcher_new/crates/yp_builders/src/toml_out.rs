use std::path::{Path, PathBuf};

use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};
use yp_core::{IoOp, IoResultExt};

pub(crate) fn basic_string(text: &str) -> Value {
    format!("\"{}\"", escape_basic(text))
        .parse()
        .unwrap_or_else(|_| Value::from(text))
}

pub(crate) fn hex_int(value: i64) -> Value {
    format!("0x{value:X}")
        .parse()
        .unwrap_or_else(|_| Value::from(value))
}

pub(crate) fn text_block(text: &str) -> Value {
    let body = split_lines(text)
        .iter()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    if !body.contains('\\') && !body.contains("'''") {
        let literal: Result<Value, _> = format!("'''\n{body}'''").parse();
        if let Ok(value) = literal
            && value.as_str() == Some(body.as_str())
        {
            return value;
        }
    }
    basic_string(&body)
}

pub(crate) fn escape_basic(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\t' => out.push('\t'),
            c if u32::from(c) < 0x20 || c == '\u{7f}' => {
                out.push_str(&format!("\\u{:04X}", u32::from(c)))
            }
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn split_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                lines.push(&text[start..index]);
                index += 1;
                start = index;
            }
            b'\r' => {
                lines.push(&text[start..index]);
                index += if bytes.get(index + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                start = index;
            }
            _ => index += 1,
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

pub(crate) struct TomlOut {
    doc: DocumentMut,
    next_position: usize,
}

impl TomlOut {
    pub(crate) fn new() -> Self {
        Self {
            doc: DocumentMut::new(),
            next_position: 1,
        }
    }

    pub(crate) fn set(&mut self, key: &str, value: Value) {
        self.doc.as_table_mut().insert(key, Item::Value(value));
    }

    pub(crate) fn gap_before_key(&mut self, key: &str) {
        if let Some(mut key) = self.doc.as_table_mut().key_mut(key) {
            key.leaf_decor_mut().set_prefix("\n");
        }
    }

    pub(crate) fn table(&mut self, gap: bool) -> Table {
        let mut table = Table::new();
        table.set_position(Some(
            isize::try_from(self.next_position).expect("TOML table position exceeds isize::MAX"),
        ));
        self.next_position += 1;
        if gap {
            table.decor_mut().set_prefix("\n");
        }
        table
    }

    pub(crate) fn header(&mut self, path: &[&str], entries: Vec<(&str, Value)>) {
        let Some((last, parents)) = path.split_last() else {
            return;
        };
        let mut table = self.table(true);
        for (key, value) in entries {
            table.insert(key, Item::Value(value));
        }
        let mut parent = self.doc.as_table_mut();
        for name in parents {
            let item = parent.entry(name).or_insert_with(|| {
                let mut implicit = Table::new();
                implicit.set_implicit(true);
                Item::Table(implicit)
            });
            let Some(next) = item.as_table_mut() else {
                return;
            };
            parent = next;
        }
        parent.insert(last, Item::Table(table));
    }

    pub(crate) fn array_entry(&mut self, key: &str, gap: bool, entries: Vec<(&str, Value)>) {
        let mut table = self.table(gap);
        for (name, value) in entries {
            table.insert(name, Item::Value(value));
        }
        let item = self
            .doc
            .as_table_mut()
            .entry(key)
            .or_insert_with(|| Item::ArrayOfTables(ArrayOfTables::new()));
        if let Some(array) = item.as_array_of_tables_mut() {
            array.push(table);
        }
    }

    pub(crate) fn finish(self) -> String {
        self.doc.to_string()
    }
}

pub(crate) fn inline(entries: Vec<(&str, Value)>) -> Value {
    let mut table = InlineTable::new();
    for (key, value) in entries {
        table.insert(key, value);
    }
    Value::InlineTable(table)
}

pub(crate) fn multiline_array(values: Vec<Value>) -> Value {
    let mut array = Array::new();
    for mut value in values {
        value.decor_mut().set_prefix("\n  ");
        value.decor_mut().set_suffix("");
        array.push_formatted(value);
    }
    array.set_trailing_comma(true);
    array.set_trailing("\n");
    Value::Array(array)
}

pub(crate) fn inline_array(values: Vec<Value>) -> Value {
    Value::Array(values.into_iter().collect())
}

pub(crate) fn write_file(path: &Path, text: &str) -> yp_core::Result<PathBuf> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).io(IoOp::Create, parent)?;
    }
    std::fs::write(path, text).io(IoOp::Write, path)?;
    Ok(path.to_path_buf())
}

pub(crate) fn slug(display_name: &str) -> String {
    let mut out = String::new();
    let mut last_was_underscore = true;
    for c in display_name.to_lowercase().chars() {
        if c.is_ascii_digit() || c.is_ascii_lowercase() {
            out.push(c);
            last_was_underscore = false;
        } else if !last_was_underscore {
            out.push('_');
            last_was_underscore = true;
        }
    }
    if out.ends_with('_') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_split_like_dart_line_splitter() {
        assert_eq!(split_lines("a\nb"), ["a", "b"]);
        assert_eq!(split_lines("a\r\nb\rc\n"), ["a", "b", "c"]);
        assert_eq!(split_lines("a\n\n"), ["a", ""]);
        assert!(split_lines("").is_empty());
    }

    #[test]
    fn blocks_prefer_multiline_literals() {
        assert_eq!(
            text_block("Something useful.").to_string(),
            "'''\nSomething useful.'''"
        );
        assert_eq!(text_block("a  \nb").to_string(), "'''\na\nb'''");
        assert_eq!(text_block("back\\slash").to_string(), "\"back\\\\slash\"");
        assert_eq!(text_block("x'''y\nz").to_string(), "\"x'''y\\nz\"");
        assert_eq!(text_block("bell\u{7}").as_str(), Some("bell\u{7}"));
    }

    #[test]
    fn basic_strings_escape_quotes_and_controls() {
        assert_eq!(
            basic_string("The \"Real\" Thing").to_string(),
            "\"The \\\"Real\\\" Thing\""
        );
        assert_eq!(basic_string("tab\there").as_str(), Some("tab\there"));
        assert_eq!(basic_string("nul\u{0}").as_str(), Some("nul\u{0}"));
        assert_eq!(hex_int(0x30100).to_string(), "0x30100");
        assert_eq!(hex_int(0x30100).as_integer(), Some(0x30100));
    }

    #[test]
    fn slugs_keep_ascii_words_only() {
        assert_eq!(slug("My Thing"), "my_thing");
        assert_eq!(slug("Engineer's  Wrench!"), "engineer_s_wrench");
        assert_eq!(slug("わたしのもの"), "");
        assert_eq!(slug("  a  b  "), "a_b");
    }
}
