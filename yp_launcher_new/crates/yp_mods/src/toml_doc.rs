use std::path::Path;

use toml_edit::{Array, DocumentMut, InlineTable, Item, Value};
use yp_core::Error;

pub(crate) fn read_values(text: &str) -> toml::Table {
    text.parse::<toml::Table>().unwrap_or_default()
}

pub(crate) fn read_values_strict(text: &str, path: &Path) -> Result<toml::Table, Error> {
    text.parse::<toml::Table>().map_err(|err| Error::TomlParse {
        path: Some(path.to_path_buf()),
        message: err.message().to_owned(),
    })
}

pub(crate) fn parse_document(text: &str, path: &Path) -> Result<DocumentMut, Error> {
    text.parse::<DocumentMut>().map_err(|err| Error::TomlParse {
        path: Some(path.to_path_buf()),
        message: err.message().to_owned(),
    })
}

pub(crate) fn string_list(value: Option<&toml::Value>) -> Option<Vec<String>> {
    let array = value?.as_array()?;
    Some(
        array
            .iter()
            .filter_map(|item| item.as_str().map(str::to_owned))
            .collect(),
    )
}

pub(crate) fn string_array<I, S>(items: I) -> Value
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut array = Array::new();
    for item in items {
        array.push(item.into());
    }
    Value::Array(array)
}

pub(crate) fn inline_table(entries: Vec<(&str, Value)>) -> Value {
    let mut table = InlineTable::new();
    for (key, value) in entries {
        table.insert(key, value);
    }
    Value::InlineTable(table)
}

pub(crate) fn set_root_value(doc: &mut DocumentMut, key: &str, mut value: Value) {
    let had_items = !doc.as_table().is_empty();
    let root = doc.as_table_mut();
    if let Some(Item::Value(existing)) = root.get_mut(key) {
        let decor = existing.decor().clone();
        *value.decor_mut() = decor;
        *existing = value;
        return;
    }
    root.insert(key, Item::Value(value));
    if !had_items {
        let trailing = doc.trailing().as_str().unwrap_or_default().to_owned();
        if !trailing.trim().is_empty() {
            doc.set_trailing("");
            if let Some(mut key_mut) = doc.as_table_mut().key_mut(key) {
                key_mut.leaf_decor_mut().set_prefix(trailing);
            }
        }
    }
}

pub(crate) fn root_array_mut<'a>(doc: &'a mut DocumentMut, key: &str) -> Option<&'a mut Array> {
    doc.as_table_mut()
        .get_mut(key)?
        .as_value_mut()?
        .as_array_mut()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(text: &str, key: &str, value: Value) -> String {
        let mut doc = parse_document(text, Path::new("t.toml")).unwrap();
        set_root_value(&mut doc, key, value);
        doc.to_string()
    }

    #[test]
    fn replaces_in_place_and_keeps_comments() {
        let out = render(
            "# keep\nlist = [] # tail\n[t]\na = 1\n",
            "list",
            string_array(["a", "b"]),
        );
        assert_eq!(out, "# keep\nlist = [\"a\", \"b\"] # tail\n[t]\na = 1\n");
    }

    #[test]
    fn inserts_missing_keys_before_tables() {
        let out = render("x = 1\n\n[t]\na = 1\n", "list", string_array(["a"]));
        assert_eq!(out, "x = 1\nlist = [\"a\"]\n\n[t]\na = 1\n");
    }

    #[test]
    fn keeps_a_comment_only_header_above_the_first_key() {
        let out = render("# header\n\n", "list", string_array(Vec::<String>::new()));
        assert_eq!(out, "# header\n\nlist = []\n");
    }

    #[test]
    fn inline_tables_render_with_spaces() {
        let value = Value::Array(Array::from_iter([inline_table(vec![
            ("path", Value::from("mods/a")),
            ("outfit_id", Value::from(3)),
        ])]));
        assert_eq!(
            render("", "outfit_config", value),
            "outfit_config = [{ path = \"mods/a\", outfit_id = 3 }]\n"
        );
    }

    #[test]
    fn lenient_reads_fall_back_to_empty() {
        assert!(read_values("a = = 1").is_empty());
        assert_eq!(
            string_list(read_values("a = [\"x\", 1]").get("a")),
            Some(vec!["x".to_owned()])
        );
        assert!(read_values_strict("a = = 1", Path::new("x")).is_err());
    }
}
