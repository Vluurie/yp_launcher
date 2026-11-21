use serde::Deserialize;
use serde_json::Value as Json;

pub const KEY_BINDING_FORMAT: &str = "key_binding";
pub const VIRTUAL_KEY_FORMAT: &str = "virtual_key";
pub const PATH_FORMAT: &str = "path";
pub const PLUGIN_CATEGORY_PREFIX: &str = "plugins.";

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Schema {
    #[serde(default)]
    pub nams_version: String,
    #[serde(default)]
    pub config_version: u32,
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub nams_dir: String,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub files: Vec<FileInfo>,
    #[serde(default)]
    pub plugins: Vec<PluginInfo>,
    #[serde(default)]
    pub plugin_errors: Vec<Json>,
    #[serde(default)]
    pub settings: Vec<Setting>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Category {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct FileInfo {
    pub file: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PluginInfo {
    pub plugin: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Bool,
    Int,
    Float,
    String,
    Enum,
    List,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
pub struct Range {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Preset {
    #[serde(default)]
    pub label: String,
    pub value: Json,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Ui {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub presets: Vec<Preset>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Choice {
    pub value: Json,
    #[serde(default)]
    pub label: String,
}

impl Choice {
    pub fn display_label(&self) -> String {
        if !self.label.is_empty() {
            return self.label.clone();
        }
        match &self.value {
            Json::String(text) => text.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct TableField {
    pub key: String,
    #[serde(rename = "type", default)]
    pub kind: Kind,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Scalar(Kind),
    Table(Vec<TableField>),
    AnyOf(Vec<ItemType>),
    Unknown,
}

impl ItemType {
    pub fn from_json(json: &Json) -> Self {
        if let Some(options) = json.get("any_of").and_then(Json::as_array) {
            return ItemType::AnyOf(options.iter().map(ItemType::from_json).collect());
        }
        if let Some(fields) = json.get("fields") {
            return serde_json::from_value(fields.clone())
                .map(ItemType::Table)
                .unwrap_or(ItemType::Unknown);
        }
        json.get("type")
            .cloned()
            .and_then(|kind| serde_json::from_value::<Kind>(kind).ok())
            .map(ItemType::Scalar)
            .unwrap_or(ItemType::Unknown)
    }

    pub fn table_fields(&self) -> Option<&[TableField]> {
        match self {
            ItemType::Table(fields) => Some(fields),
            ItemType::AnyOf(options) => options.iter().find_map(ItemType::table_fields),
            _ => None,
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(self, ItemType::Scalar(Kind::String))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, ItemType::Scalar(Kind::Int | Kind::Float))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Setting {
    pub id: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub key: String,
    #[serde(rename = "type", default)]
    pub kind: Kind,
    #[serde(default)]
    pub default: Json,
    #[serde(default)]
    pub absent_value: Option<Json>,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub restart_required: bool,
    #[serde(default)]
    pub advanced: bool,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub type_range: Option<Range>,
    #[serde(default)]
    pub range: Option<Range>,
    #[serde(default)]
    pub zero_or_range: Option<Range>,
    #[serde(default)]
    pub ui: Option<Ui>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub items: Option<Json>,
    #[serde(default)]
    pub length: Option<usize>,
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default)]
    pub modifiers: Vec<String>,
}

impl Setting {
    pub fn item_type(&self) -> Option<ItemType> {
        self.items.as_ref().map(ItemType::from_json)
    }

    pub fn has_format(&self, format: &str) -> bool {
        self.format.as_deref() == Some(format)
    }

    pub fn is_key_binding(&self) -> bool {
        self.has_format(KEY_BINDING_FORMAT)
    }

    pub fn is_virtual_key(&self) -> bool {
        self.has_format(VIRTUAL_KEY_FORMAT)
    }

    pub fn is_path(&self) -> bool {
        self.has_format(PATH_FORMAT)
    }

    pub fn is_plugin(&self) -> bool {
        self.category.starts_with(PLUGIN_CATEGORY_PREFIX)
    }

    pub fn ui_min(&self) -> Option<f64> {
        self.ui
            .as_ref()
            .and_then(|ui| ui.min)
            .or_else(|| self.range.and_then(|range| range.min))
            .or_else(|| self.zero_or_range.map(|_| 0.0))
    }

    pub fn ui_max(&self) -> Option<f64> {
        self.ui
            .as_ref()
            .and_then(|ui| ui.max)
            .or_else(|| self.range.and_then(|range| range.max))
            .or_else(|| self.zero_or_range.and_then(|range| range.max))
    }

    pub fn step(&self) -> Option<f64> {
        self.ui.as_ref().and_then(|ui| ui.step)
    }

    pub fn presets(&self) -> &[Preset] {
        self.ui.as_ref().map_or(&[], |ui| ui.presets.as_slice())
    }

    pub fn limit_min(&self) -> Option<f64> {
        self.range
            .and_then(|range| range.min)
            .or_else(|| self.type_range.and_then(|range| range.min))
    }

    pub fn limit_max(&self) -> Option<f64> {
        self.range
            .and_then(|range| range.max)
            .or_else(|| self.zero_or_range.and_then(|range| range.max))
            .or_else(|| self.type_range.and_then(|range| range.max))
    }

    pub fn value_when_unset(&self) -> &Json {
        self.absent_value.as_ref().unwrap_or(&self.default)
    }

    pub fn is_default(&self, value: &Json) -> bool {
        values_equal(value, &self.default)
    }
}

pub fn values_equal(left: &Json, right: &Json) -> bool {
    const TOLERANCE: f64 = 1e-6;
    match (left, right) {
        (Json::Number(a), Json::Number(b)) => match (a.as_f64(), b.as_f64()) {
            (Some(a), Some(b)) => (a - b).abs() <= TOLERANCE * a.abs().max(b.abs()).max(1.0),
            _ => a == b,
        },
        (Json::Array(a), Json::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| values_equal(a, b))
        }
        (Json::Object(a), Json::Object(b)) => {
            a.len() == b.len()
                && a.iter().all(|(key, value)| b.get(key).is_some_and(|other| values_equal(value, other)))
        }
        _ => left == right,
    }
}

impl Schema {
    pub fn setting(&self, id: &str) -> Option<&Setting> {
        self.settings.iter().find(|setting| setting.id == id)
    }

    pub fn category(&self, id: &str) -> Option<&Category> {
        self.categories.iter().find(|category| category.id == id)
    }

    pub fn category_title(&self, id: &str) -> String {
        self.category(id)
            .map(|category| category.title.clone())
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| id.to_owned())
    }

    pub fn settings_in<'a>(&'a self, category: &'a str) -> impl Iterator<Item = &'a Setting> + 'a {
        self.settings.iter().filter(move |setting| setting.category == category)
    }

    pub fn is_empty(&self) -> bool {
        self.settings.is_empty()
    }
}

pub fn is_in_category(category: &str, root: &str) -> bool {
    category == root
        || category
            .strip_prefix(root)
            .is_some_and(|rest| rest.starts_with('/'))
}
