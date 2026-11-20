use serde_json::Value as JsonValue;

pub fn json_to_toml(value: &JsonValue) -> Option<toml::Value> {
    match value {
        JsonValue::Null => None,
        JsonValue::Bool(flag) => Some(toml::Value::Boolean(*flag)),
        JsonValue::Number(number) => number
            .as_i64()
            .map(toml::Value::Integer)
            .or_else(|| number.as_f64().map(toml::Value::Float)),
        JsonValue::String(text) => Some(toml::Value::String(text.clone())),
        JsonValue::Array(items) => Some(toml::Value::Array(
            items.iter().filter_map(json_to_toml).collect(),
        )),
        JsonValue::Object(map) => Some(toml::Value::Table(
            map.iter()
                .filter_map(|(key, value)| json_to_toml(value).map(|value| (key.clone(), value)))
                .collect(),
        )),
    }
}

pub fn section_path(section: Option<&str>) -> Vec<&str> {
    section
        .map(|section| section.split('.').collect())
        .unwrap_or_default()
}

pub fn set_in(values: &mut toml::Table, section: Option<&str>, key: &str, value: toml::Value) {
    let mut current = values;
    for part in section_path(section) {
        let entry = current
            .entry(part.to_owned())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        if !entry.is_table() {
            *entry = toml::Value::Table(toml::Table::new());
        }
        let toml::Value::Table(next) = entry else {
            return;
        };
        current = next;
    }
    current.insert(key.to_owned(), value);
}

pub fn with_value(
    values: &toml::Table,
    section: Option<&str>,
    key: &str,
    value: toml::Value,
) -> toml::Table {
    let mut updated = values.clone();
    set_in(&mut updated, section, key, value);
    updated
}

pub fn table_at<'a>(values: &'a toml::Table, path: &[&str]) -> Option<&'a toml::Table> {
    let mut current = values;
    for part in path {
        current = current.get(*part)?.as_table()?;
    }
    Some(current)
}
