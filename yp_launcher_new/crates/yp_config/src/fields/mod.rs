pub mod bindings;
pub mod content;
pub mod lodmod;
pub mod nams;
pub mod overlay;
pub mod texture_injection;

use serde_json::Value as JsonValue;
use yp_i18n::MsgKey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldMeta {
    pub key: &'static str,
    pub section: Option<&'static str>,
    pub label: MsgKey,
    pub tooltip: Option<MsgKey>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub allowed: &'static [f64],
    pub restart_required: bool,
}

impl FieldMeta {
    pub const fn new(key: &'static str, label: MsgKey) -> Self {
        Self {
            key,
            section: None,
            label,
            tooltip: None,
            min: None,
            max: None,
            step: None,
            allowed: &[],
            restart_required: false,
        }
    }

    pub const fn section(mut self, section: &'static str) -> Self {
        self.section = Some(section);
        self
    }

    pub const fn tooltip(mut self, tooltip: MsgKey) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    pub const fn range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    pub const fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub const fn allowed(mut self, allowed: &'static [f64]) -> Self {
        self.allowed = allowed;
        self
    }

    pub const fn restart_required(mut self) -> Self {
        self.restart_required = true;
        self
    }

    pub fn section_path(&self) -> Vec<&'static str> {
        self.section
            .map(|section| section.split('.').collect())
            .unwrap_or_default()
    }

    pub fn path(&self) -> Vec<&'static str> {
        let mut path = self.section_path();
        path.push(self.key);
        path
    }

    pub fn raw_in<'a>(&self, values: &'a toml::Table) -> Option<&'a toml::Value> {
        let mut current = values;
        for part in self.section_path() {
            current = current.get(part)?.as_table()?;
        }
        current.get(self.key)
    }

    pub fn raw_in_json<'a>(
        &self,
        values: &'a serde_json::Map<String, JsonValue>,
    ) -> Option<&'a JsonValue> {
        let mut current = values;
        for part in self.section_path() {
            current = current.get(part)?.as_object()?;
        }
        current.get(self.key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldKind {
    Bool,
    Int,
    Float,
    Text,
    List,
    TextList,
}

pub trait FieldType: Sized {
    const KIND: FieldKind;
    fn from_toml(value: &toml::Value) -> Option<Self>;
    fn from_json(value: &JsonValue) -> Option<Self>;
    fn to_toml(&self) -> toml::Value;
}

impl FieldType for bool {
    const KIND: FieldKind = FieldKind::Bool;

    fn from_toml(value: &toml::Value) -> Option<Self> {
        value.as_bool()
    }

    fn from_json(value: &JsonValue) -> Option<Self> {
        value.as_bool()
    }

    fn to_toml(&self) -> toml::Value {
        toml::Value::Boolean(*self)
    }
}

fn truncate_to_int(value: f64) -> i64 {
    value.trunc() as i64
}

impl FieldType for i64 {
    const KIND: FieldKind = FieldKind::Int;

    fn from_toml(value: &toml::Value) -> Option<Self> {
        match value {
            toml::Value::Integer(number) => Some(*number),
            toml::Value::Float(number) => Some(truncate_to_int(*number)),
            _ => None,
        }
    }

    fn from_json(value: &JsonValue) -> Option<Self> {
        value
            .as_i64()
            .or_else(|| value.as_f64().map(truncate_to_int))
    }

    fn to_toml(&self) -> toml::Value {
        toml::Value::Integer(*self)
    }
}

impl FieldType for f64 {
    const KIND: FieldKind = FieldKind::Float;

    fn from_toml(value: &toml::Value) -> Option<Self> {
        match value {
            toml::Value::Integer(number) => Some(*number as f64),
            toml::Value::Float(number) => Some(*number),
            _ => None,
        }
    }

    fn from_json(value: &JsonValue) -> Option<Self> {
        value.as_f64()
    }

    fn to_toml(&self) -> toml::Value {
        toml::Value::Float(*self)
    }
}

impl FieldType for String {
    const KIND: FieldKind = FieldKind::Text;

    fn from_toml(value: &toml::Value) -> Option<Self> {
        value.as_str().map(str::to_owned)
    }

    fn from_json(value: &JsonValue) -> Option<Self> {
        value.as_str().map(str::to_owned)
    }

    fn to_toml(&self) -> toml::Value {
        toml::Value::String(self.clone())
    }
}

impl FieldType for Vec<toml::Value> {
    const KIND: FieldKind = FieldKind::List;

    fn from_toml(value: &toml::Value) -> Option<Self> {
        value.as_array().cloned()
    }

    fn from_json(value: &JsonValue) -> Option<Self> {
        value.as_array().map(|items| {
            items
                .iter()
                .filter_map(crate::values::json_to_toml)
                .collect()
        })
    }

    fn to_toml(&self) -> toml::Value {
        toml::Value::Array(self.clone())
    }
}

impl FieldType for Vec<String> {
    const KIND: FieldKind = FieldKind::TextList;

    fn from_toml(value: &toml::Value) -> Option<Self> {
        value
            .as_array()?
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect()
    }

    fn from_json(value: &JsonValue) -> Option<Self> {
        value
            .as_array()?
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect()
    }

    fn to_toml(&self) -> toml::Value {
        toml::Value::Array(self.iter().cloned().map(toml::Value::String).collect())
    }
}

pub struct ConfigField<T: 'static> {
    pub meta: FieldMeta,
    default: fn() -> T,
}

impl<T: FieldType> ConfigField<T> {
    pub const fn new(meta: FieldMeta, default: fn() -> T) -> Self {
        Self { meta, default }
    }

    pub fn key(&self) -> &'static str {
        self.meta.key
    }

    pub fn section(&self) -> Option<&'static str> {
        self.meta.section
    }

    pub fn default_value(&self) -> T {
        (self.default)()
    }

    pub fn value_in(&self, values: &toml::Table) -> T {
        self.meta
            .raw_in(values)
            .and_then(T::from_toml)
            .unwrap_or_else(|| self.default_value())
    }

    pub fn value_in_json(&self, values: &serde_json::Map<String, JsonValue>) -> T {
        self.meta
            .raw_in_json(values)
            .and_then(T::from_json)
            .unwrap_or_else(|| self.default_value())
    }
}

impl<T> std::fmt::Debug for ConfigField<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigField")
            .field("meta", &self.meta)
            .finish_non_exhaustive()
    }
}

pub trait AnyField: Sync {
    fn meta(&self) -> &FieldMeta;
    fn kind(&self) -> FieldKind;
    fn default_toml(&self) -> toml::Value;
}

impl<T: FieldType + 'static> AnyField for ConfigField<T> {
    fn meta(&self) -> &FieldMeta {
        &self.meta
    }

    fn kind(&self) -> FieldKind {
        T::KIND
    }

    fn default_toml(&self) -> toml::Value {
        self.default_value().to_toml()
    }
}

impl std::fmt::Debug for dyn AnyField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyField")
            .field("meta", self.meta())
            .field("kind", &self.kind())
            .finish()
    }
}

pub fn all() -> Vec<&'static dyn AnyField> {
    let mut fields = Vec::new();
    fields.extend(nams::ALL.iter().copied());
    fields.extend(content::all());
    fields.extend(bindings::all());
    fields.extend(lodmod::ALL.iter().copied());
    fields.extend(texture_injection::ALL.iter().copied());
    fields.extend(overlay::all());
    fields
}
