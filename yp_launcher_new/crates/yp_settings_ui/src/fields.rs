use std::rc::Rc;

use serde_json::Value as JsonValue;
use yp_config::{AnyField, FieldKind};
use yp_i18n::Strings;
use yp_ui::prelude::*;
use yp_ui::{
    ApplyKind, NumberKind, SelectItem, SettingNumber, SettingRow, SettingSelect, SettingSlider,
    SettingSwitch,
};

use crate::rules::{NameListEditor, RulesEditor, infer_columns};
use crate::text_field::TextField;

pub type FieldRef = &'static dyn AnyField;
pub type ValueSink = Rc<dyn Fn(toml::Value, &mut Window, &mut App)>;

const EQUAL_TOLERANCE: f64 = 1e-6;
const SNAP_DECIMALS: i32 = 9;

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Switch,
    Slider { min: f64, max: f64, step: f64 },
    Choice { options: Vec<f64> },
    Number { min: Option<f64>, max: Option<f64> },
    Text,
    Rules,
    Names,
}

pub fn control_for(field: &dyn AnyField) -> Control {
    let meta = field.meta();
    match field.kind() {
        FieldKind::Bool => Control::Switch,
        FieldKind::Int | FieldKind::Float => {
            if !meta.allowed.is_empty() {
                return Control::Choice {
                    options: meta.allowed.to_vec(),
                };
            }
            match (meta.min, meta.max, meta.step) {
                (Some(min), Some(max), Some(step)) if step > 0. && max > min => {
                    Control::Slider { min, max, step }
                }
                _ => Control::Number {
                    min: meta.min,
                    max: meta.max,
                },
            }
        }
        FieldKind::Text => Control::Text,
        FieldKind::List => Control::Rules,
        FieldKind::TextList => Control::Names,
    }
}

pub fn coerce(kind: FieldKind, raw: &toml::Value) -> Option<toml::Value> {
    match (kind, raw) {
        (FieldKind::Bool, toml::Value::Boolean(_)) => Some(raw.clone()),
        (FieldKind::Int, toml::Value::Integer(_)) => Some(raw.clone()),
        (FieldKind::Int, toml::Value::Float(number)) => {
            Some(toml::Value::Integer(number.trunc() as i64))
        }
        (FieldKind::Float, toml::Value::Float(_)) => Some(raw.clone()),
        (FieldKind::Float, toml::Value::Integer(number)) => {
            Some(toml::Value::Float(*number as f64))
        }
        (FieldKind::Text, toml::Value::String(_)) => Some(raw.clone()),
        (FieldKind::List, toml::Value::Array(_)) => Some(raw.clone()),
        (FieldKind::TextList, toml::Value::Array(items)) => {
            items.iter().all(toml::Value::is_str).then(|| raw.clone())
        }
        _ => None,
    }
}

pub fn value_in(field: &dyn AnyField, values: &toml::Table) -> toml::Value {
    field
        .meta()
        .raw_in(values)
        .and_then(|raw| coerce(field.kind(), raw))
        .unwrap_or_else(|| field.default_toml())
}

pub fn value_in_json(
    field: &dyn AnyField,
    values: &serde_json::Map<String, JsonValue>,
) -> toml::Value {
    field
        .meta()
        .raw_in_json(values)
        .and_then(yp_config::values::json_to_toml)
        .and_then(|raw| coerce(field.kind(), &raw))
        .unwrap_or_else(|| field.default_toml())
}

pub fn to_json(value: &toml::Value) -> JsonValue {
    match value {
        toml::Value::Boolean(flag) => JsonValue::Bool(*flag),
        toml::Value::Integer(number) => JsonValue::from(*number),
        toml::Value::Float(number) => serde_json::Number::from_f64(*number)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        toml::Value::String(text) => JsonValue::String(text.clone()),
        toml::Value::Datetime(datetime) => JsonValue::String(datetime.to_string()),
        toml::Value::Array(items) => JsonValue::Array(items.iter().map(to_json).collect()),
        toml::Value::Table(table) => JsonValue::Object(
            table
                .iter()
                .map(|(key, value)| (key.clone(), to_json(value)))
                .collect(),
        ),
    }
}

pub fn number_of(value: &toml::Value) -> Option<f64> {
    match value {
        toml::Value::Integer(number) => Some(*number as f64),
        toml::Value::Float(number) => Some(*number),
        _ => None,
    }
}

pub fn bool_of(value: &toml::Value) -> bool {
    value.as_bool().unwrap_or(false)
}

pub fn text_of(value: &toml::Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

pub fn values_equal(left: &toml::Value, right: &toml::Value) -> bool {
    if let (Some(left), Some(right)) = (number_of(left), number_of(right)) {
        return (left - right).abs() < EQUAL_TOLERANCE;
    }
    match (left, right) {
        (toml::Value::Array(left), toml::Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| values_equal(left, right))
        }
        (toml::Value::Table(left), toml::Value::Table(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, value)| {
                    right
                        .get(key)
                        .is_some_and(|other| values_equal(value, other))
                })
        }
        _ => left == right,
    }
}

pub fn is_changed(field: &dyn AnyField, current: &toml::Value) -> bool {
    !values_equal(current, &field.default_toml())
}

pub fn label_of(field: &dyn AnyField, strings: &'static Strings) -> SharedString {
    SharedString::new_static(field.meta().label.text(strings))
}

pub fn tooltip_of(field: &dyn AnyField, strings: &'static Strings) -> Option<SharedString> {
    field
        .meta()
        .tooltip
        .map(|key| SharedString::new_static(key.text(strings)))
}

pub fn changed_in(
    fields: &[FieldRef],
    values: &toml::Table,
    strings: &'static Strings,
) -> Vec<SharedString> {
    fields
        .iter()
        .filter(|field| is_changed(**field, &value_in(**field, values)))
        .map(|field| label_of(*field, strings))
        .collect()
}

pub fn changed_in_json(
    fields: &[FieldRef],
    values: &serde_json::Map<String, JsonValue>,
    strings: &'static Strings,
) -> Vec<SharedString> {
    fields
        .iter()
        .filter(|field| is_changed(**field, &value_in_json(**field, values)))
        .map(|field| label_of(*field, strings))
        .collect()
}

pub fn snap_to_step(value: f64, min: f64, max: f64, step: f64) -> f64 {
    if step.is_nan() || step <= 0. || !value.is_finite() {
        return value.clamp(min, max);
    }
    let snapped = (value / step).round() * step;
    let factor = 10f64.powi(SNAP_DECIMALS);
    ((snapped * factor).round() / factor).clamp(min, max)
}

pub fn number_value(kind: FieldKind, value: f64) -> toml::Value {
    match kind {
        FieldKind::Int => toml::Value::Integer(value.round() as i64),
        _ => toml::Value::Float(value),
    }
}

pub fn format_option(kind: FieldKind, value: f64) -> String {
    match kind {
        FieldKind::Int => format!("{}", value.round() as i64),
        _ => {
            if value == value.round() {
                format!("{value:.0}")
            } else {
                format!("{value}")
            }
        }
    }
}

pub fn field_id(scope: &str, field: &dyn AnyField) -> ElementId {
    ElementId::Name(SharedString::from(format!(
        "{scope}:{}",
        field.meta().path().join(".")
    )))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Marker {
    #[default]
    None,
    Always,
    RestartOnly,
}

impl Marker {
    pub fn kind(self, restart_required: bool) -> Option<ApplyKind> {
        match (self, restart_required) {
            (Marker::None, _) | (Marker::RestartOnly, false) => None,
            (_, true) => Some(ApplyKind::Restart),
            (Marker::Always, false) => Some(ApplyKind::Live),
        }
    }
}

type OptionLabel = Rc<dyn Fn(f64) -> Option<SharedString>>;
type Snap = Rc<dyn Fn(f64, f64) -> f64>;

pub struct FieldRow {
    scope: &'static str,
    field: FieldRef,
    value: toml::Value,
    marker: Marker,
    decimals: usize,
    disabled: bool,
    options: Option<Vec<f64>>,
    option_label: Option<OptionLabel>,
    value_label: Option<SharedString>,
    label: Option<SharedString>,
    description: Option<Option<SharedString>>,
    snap: Option<Snap>,
    id_suffix: Option<usize>,
    on_change: Option<ValueSink>,
}

yp_ui::impl_into_element!(FieldRow);

impl FieldRow {
    pub fn new(scope: &'static str, field: FieldRef, value: toml::Value) -> Self {
        Self {
            scope,
            field,
            value,
            marker: Marker::None,
            decimals: 2,
            disabled: false,
            options: None,
            option_label: None,
            value_label: None,
            label: None,
            description: None,
            snap: None,
            id_suffix: None,
            on_change: None,
        }
    }

    pub fn toml(scope: &'static str, field: FieldRef, values: &toml::Table) -> Self {
        Self::new(scope, field, value_in(field, values))
    }

    pub fn json(
        scope: &'static str,
        field: FieldRef,
        values: &serde_json::Map<String, JsonValue>,
    ) -> Self {
        Self::new(scope, field, value_in_json(field, values))
    }

    pub fn marker(mut self, marker: Marker) -> Self {
        self.marker = marker;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn options(mut self, options: Vec<f64>) -> Self {
        self.options = Some(options);
        self
    }

    pub fn option_label(mut self, label: impl Fn(f64) -> Option<SharedString> + 'static) -> Self {
        self.option_label = Some(Rc::new(label));
        self
    }

    pub fn value_label(mut self, label: impl Into<Option<SharedString>>) -> Self {
        self.value_label = label.into();
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn description(mut self, description: Option<SharedString>) -> Self {
        self.description = Some(description);
        self
    }

    pub fn snap(mut self, snap: impl Fn(f64, f64) -> f64 + 'static) -> Self {
        self.snap = Some(Rc::new(snap));
        self
    }

    pub fn id_suffix(mut self, suffix: usize) -> Self {
        self.id_suffix = Some(suffix);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(toml::Value, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn sink(mut self, sink: ValueSink) -> Self {
        self.on_change = Some(sink);
        self
    }

    pub fn control(&self) -> Control {
        match &self.options {
            Some(options) => Control::Choice {
                options: options.clone(),
            },
            None => control_for(self.field),
        }
    }
}

impl RenderOnce for FieldRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let control = self.control();
        let field = self.field;
        let meta = field.meta();
        let kind = field.kind();
        let base_id = field_id(self.scope, field);
        let id = match self.id_suffix {
            Some(suffix) => ElementId::from((base_id, suffix.to_string())),
            None => base_id,
        };
        let label = self.label.unwrap_or_else(|| label_of(field, strings));
        let description = match self.description {
            Some(description) => description,
            None => tooltip_of(field, strings),
        };
        let apply = self.marker.kind(meta.restart_required);
        let disabled = self.disabled;
        let sink = self.on_change;
        match control {
            Control::Switch => SettingSwitch::new(id, label, bool_of(&self.value))
                .apply_marker(apply)
                .disabled(disabled)
                .when_some(description, |this, description| {
                    this.description(description)
                })
                .when_some(sink, |this, sink| {
                    this.on_change(move |value, window, cx| {
                        sink(toml::Value::Boolean(*value), window, cx)
                    })
                })
                .into_any_element(),
            Control::Slider { min, max, step } => {
                let current = number_of(&self.value).unwrap_or(min);
                let snap = self.snap;
                SettingSlider::new(id, label, current as f32)
                    .min(min as f32)
                    .max(max as f32)
                    .step(step as f32)
                    .decimals(self.decimals)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .when_some(self.value_label, |this, label| this.value_label(label))
                    .when_some(description, |this, description| {
                        this.description(description)
                    })
                    .when_some(sink, |this, sink| {
                        this.on_change(move |value, window, cx| {
                            let snapped = snap_to_step(f64::from(*value), min, max, step);
                            let next = match &snap {
                                Some(snap) => snap(snapped, current),
                                None => snapped,
                            };
                            if (next - current).abs() < EQUAL_TOLERANCE {
                                return;
                            }
                            sink(number_value(kind, next), window, cx)
                        })
                    })
                    .into_any_element()
            }
            Control::Choice { options } => {
                let current = number_of(&self.value);
                let option_label = self.option_label;
                let label_for = move |option: f64| -> SharedString {
                    option_label
                        .as_ref()
                        .and_then(|label| label(option))
                        .unwrap_or_else(|| SharedString::from(format_option(kind, option)))
                };
                let selected_index = current.and_then(|current| {
                    options
                        .iter()
                        .position(|option| (option - current).abs() < EQUAL_TOLERANCE)
                });
                let selected_label = match (selected_index, current) {
                    (Some(index), _) => label_for(options[index]),
                    (None, Some(current)) => SharedString::from(format_option(kind, current)),
                    (None, None) => SharedString::default(),
                };
                let items: Vec<SelectItem<f64>> = options
                    .iter()
                    .map(|option| SelectItem::new(label_for(*option), *option))
                    .collect();
                SettingSelect::new(id, label, selected_label)
                    .items(items)
                    .selected_index(selected_index)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .when_some(description, |this, description| {
                        this.description(description)
                    })
                    .when_some(sink, |this, sink| {
                        this.on_select(move |option: &f64, window, cx| {
                            sink(number_value(kind, *option), window, cx)
                        })
                    })
                    .into_any_element()
            }
            Control::Number { min, max } => {
                let number_kind = match kind {
                    FieldKind::Int => NumberKind::Int,
                    _ => NumberKind::Float {
                        decimals: self.decimals.max(1),
                    },
                };
                SettingNumber::new(id, label, number_of(&self.value).unwrap_or_default())
                    .kind(number_kind)
                    .min(min)
                    .max(max)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .when_some(description, |this, description| {
                        this.description(description)
                    })
                    .when_some(sink, |this, sink| {
                        this.on_change(move |value, window, cx| {
                            sink(number_value(kind, *value), window, cx)
                        })
                    })
                    .into_any_element()
            }
            Control::Text => {
                let field_id = ElementId::from((id.clone(), "text"));
                SettingRow::new(id, label)
                    .description_opt(description)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .control(
                        div().w(sizes.config_input_width * 2.).child(
                            TextField::new(field_id, text_of(&self.value))
                                .disabled(disabled)
                                .when_some(sink, |this, sink| {
                                    this.on_change(move |text, window, cx| {
                                        sink(toml::Value::String(text.to_owned()), window, cx)
                                    })
                                }),
                        ),
                    )
                    .into_any_element()
            }
            Control::Rules => {
                let rows = self.value.as_array().cloned().unwrap_or_default();
                let columns = infer_columns(&rows);
                RulesEditor::new(id, label, rows)
                    .columns(columns)
                    .description(description)
                    .apply_marker(Marker::Always.kind(meta.restart_required))
                    .when_some(sink, |this, sink| this.sink(sink))
                    .into_any_element()
            }
            Control::Names => {
                let names = self
                    .value
                    .as_array()
                    .map(|items| items.iter().map(text_of).collect())
                    .unwrap_or_default();
                NameListEditor::new(id, label, names)
                    .description(description)
                    .apply_marker(Marker::Always.kind(meta.restart_required))
                    .when_some(sink, |this, sink| this.sink(sink))
                    .into_any_element()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use yp_config::fields::{bindings, content, lodmod, nams, overlay, texture_injection};
    use yp_i18n::Locale;

    use super::*;

    fn table(text: &str) -> toml::Table {
        text.parse::<toml::Table>().unwrap()
    }

    #[test]
    fn every_registry_field_maps_to_a_control_that_matches_its_kind() {
        let mut seen = std::collections::HashSet::new();
        for field in yp_config::fields::all() {
            let control = control_for(field);
            let meta = field.meta();
            match field.kind() {
                FieldKind::Bool => assert_eq!(control, Control::Switch, "{}", meta.key),
                FieldKind::Int | FieldKind::Float => match &control {
                    Control::Choice { options } => assert_eq!(options, meta.allowed),
                    Control::Slider { min, max, step } => {
                        assert_eq!(Some(*min), meta.min);
                        assert_eq!(Some(*max), meta.max);
                        assert_eq!(Some(*step), meta.step);
                        assert!(meta.allowed.is_empty());
                    }
                    Control::Number { min, max } => {
                        assert_eq!((*min, *max), (meta.min, meta.max));
                        assert!(meta.step.is_none());
                    }
                    other => panic!("{} got {other:?}", meta.key),
                },
                FieldKind::Text => assert_eq!(control, Control::Text),
                FieldKind::List => assert_eq!(control, Control::Rules),
                FieldKind::TextList => assert_eq!(control, Control::Names),
            }
            seen.insert(std::mem::discriminant(&control));
        }
        assert_eq!(seen.len(), 7);
    }

    #[test]
    fn representative_fields_get_the_expected_controls() {
        assert_eq!(control_for(&nams::VALIDATE_SCRIPTS), Control::Switch);
        assert_eq!(
            control_for(&nams::SENSITIVITY),
            Control::Slider {
                min: 0.25,
                max: 10.,
                step: 0.25
            }
        );
        assert_eq!(
            control_for(&lodmod::SHADOW_RESOLUTION),
            Control::Choice {
                options: vec![512., 1024., 2048., 4096., 8192.]
            }
        );
        assert_eq!(
            control_for(&lodmod::FPS_LIMIT),
            Control::Number {
                min: Some(0.),
                max: Some(1000.)
            }
        );
        assert_eq!(
            control_for(&overlay::cheats::DAMAGE_MULTIPLIER),
            Control::Number {
                min: None,
                max: None
            }
        );
        assert_eq!(control_for(&bindings::STANDARD_JUMP), Control::Text);
        assert_eq!(control_for(&lodmod::HIGH_GRIDS_ROOM_RINGS), Control::Rules);
        assert_eq!(control_for(&texture_injection::LOAD_ORDER), Control::Names);
    }

    #[test]
    fn values_come_from_the_file_or_fall_back_to_the_default() {
        let values = table(
            "validate_scripts = true\n[mouse]\nsensitivity = 3\n[content]\ncustom_items = \"x\"\n",
        );
        assert_eq!(
            value_in(&nams::VALIDATE_SCRIPTS, &values),
            toml::Value::Boolean(true)
        );
        assert_eq!(
            value_in(&nams::SENSITIVITY, &values),
            toml::Value::Float(3.)
        );
        assert_eq!(
            value_in(&content::CUSTOM_ITEMS, &values),
            toml::Value::Boolean(true)
        );
        assert_eq!(
            value_in(&nams::FIX_WIND_TIMER_BUG, &values),
            toml::Value::Boolean(true)
        );
        let lod = table("bloom_reference_height = 900.7\n");
        assert_eq!(
            value_in(&lodmod::BLOOM_REFERENCE_HEIGHT, &lod),
            toml::Value::Integer(900)
        );
    }

    #[test]
    fn json_sources_follow_the_section_path() {
        let settings: serde_json::Map<String, JsonValue> = serde_json::from_str(
            r#"{"main":{"yorha_protocol":"F2"},"yorha_protocol":{"free_cam":"K"},"damageMultiplier":2}"#,
        )
        .unwrap();
        assert_eq!(
            text_of(&value_in_json(&overlay::keybinds::OPEN_WORKSPACE, &settings)),
            "F2"
        );
        assert_eq!(
            text_of(&value_in_json(&overlay::keybinds::FREE_CAM, &settings)),
            "K"
        );
        assert_eq!(
            text_of(&value_in_json(&overlay::keybinds::DEV_MODE, &settings)),
            "Insert"
        );
        assert_eq!(
            value_in_json(&overlay::cheats::DAMAGE_MULTIPLIER, &settings),
            toml::Value::Float(2.)
        );
        assert_eq!(to_json(&toml::Value::Float(2.5)), serde_json::json!(2.5));
        assert_eq!(to_json(&toml::Value::Boolean(true)), JsonValue::Bool(true));
    }

    #[test]
    fn changed_labels_list_only_fields_that_left_their_default() {
        let strings = Strings::for_locale(Locale::En);
        let values = table("validate_scripts = true\nfix_wind_timer_bug = true\n");
        let fields: [FieldRef; 3] = [
            &nams::VALIDATE_SCRIPTS,
            &nams::FIX_WIND_TIMER_BUG,
            &nams::VALIDATE_MODEL_DATA,
        ];
        let labels = changed_in(&fields, &values, strings);
        assert_eq!(labels, vec![label_of(&nams::VALIDATE_SCRIPTS, strings)]);
        assert!(values_equal(
            &toml::Value::Float(1.0),
            &toml::Value::Integer(1)
        ));
        assert!(values_equal(
            &toml::Value::Float(0.1 + 0.2),
            &toml::Value::Float(0.3)
        ));
        assert!(!values_equal(
            &toml::Value::Array(vec![toml::Value::Integer(1)]),
            &toml::Value::Array(Vec::new())
        ));
    }

    #[test]
    fn slider_values_snap_cleanly_and_keep_the_field_type() {
        assert_eq!(snap_to_step(0.30000001192, 0., 1., 0.05), 0.3);
        assert_eq!(snap_to_step(12.2, 0.25, 10., 0.25), 10.);
        assert_eq!(snap_to_step(0.0004999, 0.0002, 0.01, 0.0001), 0.0005);
        assert_eq!(
            number_value(FieldKind::Int, 899.6),
            toml::Value::Integer(900)
        );
        assert_eq!(
            number_value(FieldKind::Float, 0.75),
            toml::Value::Float(0.75)
        );
        assert_eq!(format_option(FieldKind::Int, 4096.), "4096");
        assert_eq!(format_option(FieldKind::Float, 0.75), "0.75");
        assert_eq!(format_option(FieldKind::Float, 10.), "10");
    }

    #[test]
    fn markers_follow_the_page_policy() {
        assert_eq!(Marker::None.kind(true), None);
        assert_eq!(Marker::Always.kind(false), Some(ApplyKind::Live));
        assert_eq!(Marker::Always.kind(true), Some(ApplyKind::Restart));
        assert_eq!(Marker::RestartOnly.kind(false), None);
        assert_eq!(Marker::RestartOnly.kind(true), Some(ApplyKind::Restart));
    }

    #[test]
    fn choice_override_replaces_the_registry_control() {
        let row = FieldRow::new(
            "t",
            &texture_injection::VRAM_BUDGET_MB,
            toml::Value::Integer(0),
        );
        assert!(matches!(row.control(), Control::Number { .. }));
        let row = row.options(vec![0., 1024.]);
        assert_eq!(
            row.control(),
            Control::Choice {
                options: vec![0., 1024.]
            }
        );
    }
}
