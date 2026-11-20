use toml_edit::{Item, TableLike, Value};

pub(crate) fn set_keeping_decor(table: &mut dyn TableLike, key: &str, mut value: Value) {
    if let Some(Item::Value(existing)) = table.get_mut(key) {
        *value.decor_mut() = existing.decor().clone();
        *existing = value;
        return;
    }
    table.insert(key, Item::Value(value));
}

pub(crate) fn lenient_values(text: &str) -> toml::Table {
    toml::from_str(text).unwrap_or_default()
}
