use std::rc::Rc;

use serde_json::Value as Json;
use yp_config::{ItemType, Kind, Setting, TableField, values_equal};
use yp_ui::prelude::*;
use yp_ui::{
    ApplyKind, NumberKind, SelectItem, SettingNumber, SettingRow, SettingSelect, SettingSlider,
    SettingSwitch,
};

use crate::rules::{ColumnKind, NameListEditor, RuleColumn, RulesEditor, infer_columns};
use crate::text_field::TextField;

pub type ValueSink = Rc<dyn Fn(Json, &mut Window, &mut App)>;

const EQUAL_TOLERANCE: f64 = 1e-6;
const SNAP_DECIMALS: i32 = 9;
const DEFAULT_INT_STEP: f64 = 1.0;
const DEFAULT_FLOAT_STEPS: f64 = 100.0;
pub const ROOM_COLUMN: &str = "room";
pub const RINGS_COLUMN: &str = "rings";
pub const FROM_RINGS_COLUMN: &str = "from_rings";
pub const RING_OPTIONS: [i64; 4] = [1, 2, 3, 4];
pub const FROM_RING_OPTIONS: [i64; 3] = [2, 3, 4];
pub const REQUIRED_HEX_FALLBACKS: [(&str, i64); 2] = [("grid", 0x1121), ("from", 0x1517)];

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Switch,
    Select { options: Vec<(Json, SharedString)> },
    Slider { min: f64, max: f64, step: f64, zero_below: Option<f64> },
    Number { min: Option<f64>, max: Option<f64> },
    Text,
    Numbers { length: usize, min: f64, max: f64, step: f64 },
    Rules { columns: Vec<RuleColumn> },
    Names,
}

fn step_for(setting: &Setting, min: f64, max: f64) -> f64 {
    setting.step().unwrap_or(match setting.kind {
        Kind::Int => DEFAULT_INT_STEP,
        _ => (max - min) / DEFAULT_FLOAT_STEPS,
    })
}

fn hex_fallback(field: &TableField) -> i64 {
    REQUIRED_HEX_FALLBACKS
        .iter()
        .find(|(key, _)| field.required && *key == field.key)
        .map_or(0, |(_, fallback)| *fallback)
}

pub fn column_for(field: &TableField) -> RuleColumn {
    let key: &str = &field.key;
    let label = SharedString::from(field.key.clone());
    match field.kind {
        Kind::String | Kind::Enum => RuleColumn::text_owned(key, label),
        _ if key == ROOM_COLUMN => RuleColumn::room_owned(key, label),
        _ if key == RINGS_COLUMN => RuleColumn::choice_owned(key, label, RING_OPTIONS.to_vec(), RING_OPTIONS[0]),
        _ if key == FROM_RINGS_COLUMN => {
            RuleColumn::choice_owned(key, label, FROM_RING_OPTIONS.to_vec(), FROM_RING_OPTIONS[0])
        }
        _ => RuleColumn {
            key: SharedString::from(field.key.clone()),
            label,
            kind: ColumnKind::Hex,
            fallback: Json::from(hex_fallback(field)),
        },
    }
}

fn preset_options(setting: &Setting) -> Vec<(Json, SharedString)> {
    setting
        .presets()
        .iter()
        .map(|preset| (preset.value.clone(), SharedString::from(preset.label.clone())))
        .collect()
}

fn choice_options(setting: &Setting) -> Vec<(Json, SharedString)> {
    setting
        .choices
        .iter()
        .map(|choice| (choice.value.clone(), SharedString::from(choice.display_label())))
        .collect()
}

pub fn control_for(setting: &Setting) -> Control {
    match setting.kind {
        Kind::Bool => Control::Switch,
        Kind::Enum => Control::Select {
            options: choice_options(setting),
        },
        Kind::Int | Kind::Float if !setting.choices.is_empty() => Control::Select {
            options: choice_options(setting),
        },
        Kind::Int | Kind::Float if !setting.presets().is_empty() && setting.step().is_none() => Control::Select {
            options: preset_options(setting),
        },
        Kind::Int | Kind::Float => {
            match (setting.ui_min(), setting.ui_max(), setting.step()) {
                (Some(min), Some(max), Some(step)) if max > min && step > 0.0 => Control::Slider {
                    min,
                    max,
                    step,
                    zero_below: setting.zero_or_range.and_then(|range| range.min),
                },
                _ => Control::Number {
                    min: setting.limit_min(),
                    max: setting.limit_max(),
                },
            }
        }
        Kind::String => Control::Text,
        Kind::List => match setting.item_type() {
            Some(ItemType::Scalar(Kind::Int | Kind::Float)) if setting.length.is_some() => {
                let min = setting.ui_min().unwrap_or(0.0);
                let max = setting.ui_max().unwrap_or(1.0);
                Control::Numbers {
                    length: setting.length.unwrap_or_default(),
                    min,
                    max,
                    step: step_for(setting, min, max),
                }
            }
            Some(item) => match item.table_fields() {
                Some(fields) => Control::Rules {
                    columns: fields.iter().map(column_for).collect(),
                },
                None => Control::Names,
            },
            None => Control::Names,
        },
        Kind::Other => Control::Text,
    }
}

pub fn number_of(value: &Json) -> Option<f64> {
    value.as_f64()
}

pub fn bool_of(value: &Json) -> bool {
    value.as_bool().unwrap_or(false)
}

pub fn text_of(value: &Json) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

pub fn texts_of(value: &Json) -> Vec<String> {
    value
        .as_array()
        .map(|items| items.iter().map(text_of).collect())
        .unwrap_or_default()
}

pub fn snap_to_step(value: f64, min: f64, max: f64, step: f64) -> f64 {
    if step.is_nan() || step <= 0. || !value.is_finite() {
        return value.clamp(min, max);
    }
    let snapped = (value / step).round() * step;
    let factor = 10f64.powi(SNAP_DECIMALS);
    ((snapped * factor).round() / factor).clamp(min, max)
}

pub fn snap_zero_or_range(next: f64, current: f64, minimum: f64) -> f64 {
    if next >= minimum || next <= 0.0 {
        return next.max(0.0);
    }
    if current >= minimum { 0.0 } else { minimum }
}

pub fn number_value(kind: Kind, value: f64) -> Json {
    match kind {
        Kind::Int => Json::from(value.round() as i64),
        _ => serde_json::Number::from_f64(value).map(Json::Number).unwrap_or(Json::Null),
    }
}

pub fn format_number(kind: Kind, value: f64) -> String {
    match kind {
        Kind::Int => format!("{}", value.round() as i64),
        _ if value == value.round() => format!("{value:.0}"),
        _ => format!("{value}"),
    }
}

pub fn decimals_for(setting: &Setting) -> usize {
    if setting.kind == Kind::Int {
        return 0;
    }
    let step = setting.step().unwrap_or(0.01);
    let text = format!("{step}");
    text.split_once('.').map_or(0, |(_, fraction)| fraction.len()).clamp(0, 4)
}

pub fn field_id(scope: &str, id: &str) -> ElementId {
    ElementId::Name(SharedString::from(format!("{scope}:{id}")))
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

type Snap = Rc<dyn Fn(f64, f64) -> f64>;

pub struct FieldRow {
    scope: &'static str,
    setting: Setting,
    value: Json,
    marker: Marker,
    decimals: Option<usize>,
    disabled: bool,
    control: Option<Control>,
    value_label: Option<SharedString>,
    label: Option<SharedString>,
    description: Option<Option<SharedString>>,
    add_label: Option<SharedString>,
    empty_label: Option<SharedString>,
    item_labels: Vec<SharedString>,
    snap: Option<Snap>,
    on_change: Option<ValueSink>,
}

yp_ui::impl_into_element!(FieldRow);

impl FieldRow {
    pub fn new(scope: &'static str, setting: &Setting, value: Json) -> Self {
        Self {
            scope,
            setting: setting.clone(),
            value,
            marker: Marker::RestartOnly,
            decimals: None,
            disabled: false,
            control: None,
            value_label: None,
            label: None,
            description: None,
            add_label: None,
            empty_label: None,
            item_labels: Vec::new(),
            snap: None,
            on_change: None,
        }
    }

    pub fn item_labels(mut self, labels: Vec<SharedString>) -> Self {
        self.item_labels = labels;
        self
    }

    pub fn marker(mut self, marker: Marker) -> Self {
        self.marker = marker;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = Some(decimals);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn control_override(mut self, control: Control) -> Self {
        self.control = Some(control);
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

    pub fn add_label(mut self, label: impl Into<SharedString>) -> Self {
        self.add_label = Some(label.into());
        self
    }

    pub fn empty_label(mut self, label: impl Into<SharedString>) -> Self {
        self.empty_label = Some(label.into());
        self
    }

    pub fn snap(mut self, snap: impl Fn(f64, f64) -> f64 + 'static) -> Self {
        self.snap = Some(Rc::new(snap));
        self
    }

    pub fn on_change(mut self, handler: impl Fn(Json, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn sink(mut self, sink: ValueSink) -> Self {
        self.on_change = Some(sink);
        self
    }

    pub fn control(&self) -> Control {
        self.control.clone().unwrap_or_else(|| control_for(&self.setting))
    }
}

impl RenderOnce for FieldRow {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(_cx);
        let control = self.control();
        let setting = self.setting;
        let kind = setting.kind;
        let id = field_id(self.scope, &setting.id);
        let label = self
            .label
            .unwrap_or_else(|| SharedString::from(setting.label.clone()));
        let description = match self.description {
            Some(description) => description,
            None => Some(SharedString::from(setting.description.clone())).filter(|text| !text.is_empty()),
        };
        let apply = self.marker.kind(setting.restart_required);
        let decimals = self.decimals.unwrap_or_else(|| decimals_for(&setting));
        let disabled = self.disabled;
        let sink = self.on_change;
        match control {
            Control::Switch => SettingSwitch::new(id, label, bool_of(&self.value))
                .apply_marker(apply)
                .disabled(disabled)
                .when_some(description, |this, description| this.description(description))
                .when_some(sink, |this, sink| {
                    this.on_change(move |value, window, cx| sink(Json::Bool(*value), window, cx))
                })
                .into_any_element(),
            Control::Slider { min, max, step, zero_below } => {
                let current = number_of(&self.value).unwrap_or(min);
                let snap = self.snap;
                SettingSlider::new(id, label, current as f32)
                    .min(min as f32)
                    .max(max as f32)
                    .step(step as f32)
                    .decimals(decimals)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .when_some(self.value_label, |this, label| this.value_label(label))
                    .when_some(description, |this, description| this.description(description))
                    .when_some(sink, |this, sink| {
                        this.on_change(move |value, window, cx| {
                            let snapped = snap_to_step(f64::from(*value), min, max, step);
                            let zeroed = match zero_below {
                                Some(minimum) => snap_zero_or_range(snapped, current, minimum),
                                None => snapped,
                            };
                            let next = match &snap {
                                Some(snap) => snap(zeroed, current),
                                None => zeroed,
                            };
                            if (next - current).abs() < EQUAL_TOLERANCE {
                                return;
                            }
                            sink(number_value(kind, next), window, cx)
                        })
                    })
                    .into_any_element()
            }
            Control::Select { options } => {
                let selected_index = options
                    .iter()
                    .position(|(value, _)| values_equal(value, &self.value));
                let selected_label = match selected_index {
                    Some(index) => options[index].1.clone(),
                    None => match number_of(&self.value) {
                        Some(number) => SharedString::from(format_number(kind, number)),
                        None => SharedString::from(text_of(&self.value)),
                    },
                };
                let items: Vec<SelectItem<Json>> = options
                    .iter()
                    .map(|(value, label)| SelectItem::new(label.clone(), value.clone()))
                    .collect();
                SettingSelect::new(id, label, selected_label)
                    .items(items)
                    .selected_index(selected_index)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .when_some(description, |this, description| this.description(description))
                    .when_some(sink, |this, sink| {
                        this.on_select(move |value: &Json, window, cx| sink(value.clone(), window, cx))
                    })
                    .into_any_element()
            }
            Control::Number { min, max } => {
                let number_kind = match kind {
                    Kind::Int => NumberKind::Int,
                    _ => NumberKind::Float {
                        decimals: decimals.max(1),
                    },
                };
                SettingNumber::new(id, label, number_of(&self.value).unwrap_or_default())
                    .kind(number_kind)
                    .min(min)
                    .max(max)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .when_some(description, |this, description| this.description(description))
                    .when_some(sink, |this, sink| {
                        this.on_change(move |value, window, cx| sink(number_value(kind, *value), window, cx))
                    })
                    .into_any_element()
            }
            Control::Text => {
                let text_id = ElementId::from((id.clone(), "text"));
                SettingRow::new(id, label)
                    .description_opt(description)
                    .apply_marker(apply)
                    .disabled(disabled)
                    .control(
                        div().w(sizes.config_input_width * 2.).child(
                            TextField::new(text_id, text_of(&self.value))
                                .disabled(disabled)
                                .when_some(sink, |this, sink| {
                                    this.on_change(move |text, window, cx| {
                                        sink(Json::String(text.to_owned()), window, cx)
                                    })
                                }),
                        ),
                    )
                    .into_any_element()
            }
            Control::Numbers { length, min, max, step } => {
                let current: Vec<f64> = (0..length)
                    .map(|index| {
                        self.value
                            .as_array()
                            .and_then(|items| items.get(index))
                            .and_then(Json::as_f64)
                            .unwrap_or(1.0)
                    })
                    .collect();
                let items = (0..length).map(|index| {
                    let current = current.clone();
                    let sink = sink.clone();
                    let item_id = ElementId::from((id.clone(), index.to_string()));
                    let item_label = self
                        .item_labels
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| SharedString::from(format!("{label} {}", index + 1)));
                    let item_description = (index == 0).then(|| description.clone()).flatten();
                    SettingSlider::new(item_id, item_label, current[index] as f32)
                        .min(min as f32)
                        .max(max as f32)
                        .step(step as f32)
                        .decimals(decimals)
                        .apply_marker(apply)
                        .disabled(disabled)
                        .when_some(item_description, |this, description| this.description(description))
                        .when_some(sink, |this, sink| {
                            this.on_change(move |value, window, cx| {
                                let mut next = current.clone();
                                next[index] = snap_to_step(f64::from(*value), min, max, step);
                                let list = next.into_iter().map(|value| number_value(Kind::Float, value)).collect();
                                sink(Json::Array(list), window, cx)
                            })
                        })
                        .into_any_element()
                });
                div().flex().flex_col().w_full().children(items).into_any_element()
            }
            Control::Rules { columns } => {
                let rows = self.value.as_array().cloned().unwrap_or_default();
                let columns = if columns.is_empty() { infer_columns(&rows) } else { columns };
                RulesEditor::new(id, label, rows)
                    .columns(columns)
                    .description(description)
                    .apply_marker(apply)
                    .when_some(sink, |this, sink| this.sink(sink))
                    .into_any_element()
            }
            Control::Names => {
                let names = texts_of(&self.value);
                NameListEditor::new(id, label, names)
                    .description(description)
                    .apply_marker(apply)
                    .when_some(self.add_label, |this, label| this.add_label(label))
                    .when_some(self.empty_label, |this, label| this.empty_label(label))
                    .when_some(sink, |this, sink| this.sink(sink))
                    .into_any_element()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn setting(json: Json) -> Setting {
        serde_json::from_value(json).unwrap()
    }

    fn values(control: &Control) -> Vec<Json> {
        match control {
            Control::Select { options } => options.iter().map(|(value, _)| value.clone()).collect(),
            other => panic!("expected a dropdown, got {other:?}"),
        }
    }

    #[test]
    fn presets_without_a_step_are_dropdowns() {
        let lod = setting(json!({ "id": "lodmod.lod_multiplier", "type": "float", "default": 0.0, "range": { "min": 0.0, "max": 10.0 }, "ui": { "presets": [{ "label": "Best quality", "value": 0.0 }, { "label": "Performance", "value": 0.75 }, { "label": "Vanilla", "value": 1.0 }, { "label": "Far", "value": 10.0 }] } }));
        assert_eq!(values(&control_for(&lod)), vec![json!(0.0), json!(0.75), json!(1.0), json!(10.0)]);
        let rings = setting(json!({ "id": "lodmod.high_grids.rings", "type": "int", "default": 4, "absent_value": 1, "range": { "min": 1, "max": 4 }, "type_range": { "min": 0, "max": 18446744073709551615u64 }, "ui": { "presets": [{ "label": "7 grids (vanilla)", "value": 1 }, { "label": "19 grids", "value": 2 }, { "label": "37 grids", "value": 3 }, { "label": "61 grids", "value": 4 }] } }));
        assert_eq!(values(&control_for(&rings)), vec![json!(1), json!(2), json!(3), json!(4)]);
        let drop = setting(json!({ "id": "lodmod.bloom_drop_coarse_levels", "type": "int", "default": 0, "range": { "min": 0, "max": 2 }, "ui": { "presets": [{ "label": "Vanilla", "value": 0 }, { "label": "Drop the widest", "value": 1 }, { "label": "Drop the two widest", "value": 2 }] } }));
        assert_eq!(values(&control_for(&drop)).len(), 3);
        let gi = setting(json!({ "id": "lodmod.gi_workgroup_size", "type": "int", "default": 128, "ui": { "presets": [{ "label": "16", "value": 16 }, { "label": "128 (vanilla)", "value": 128 }] } }));
        assert_eq!(values(&control_for(&gi)), vec![json!(16), json!(128)]);
    }

    #[test]
    fn a_range_without_a_step_is_a_number_field() {
        let fps = setting(json!({ "id": "lodmod.fps_limit", "type": "int", "default": 0, "range": { "min": 0, "max": 1000 }, "type_range": { "min": 0, "max": 4294967295u64 } }));
        assert_eq!(control_for(&fps), Control::Number { min: Some(0.0), max: Some(1000.0) });
    }

    #[test]
    fn a_stepped_range_is_a_slider_even_with_presets() {
        let bloom = setting(json!({ "id": "lodmod.bloom_reference_height", "type": "int", "default": 0, "type_range": { "min": 0, "max": 4294967295u64 }, "zero_or_range": { "min": 360, "max": 4320 }, "ui": { "step": 20, "presets": [{ "label": "Vanilla", "value": 0 }, { "label": "2017 footprint", "value": 900 }] } }));
        assert_eq!(
            control_for(&bloom),
            Control::Slider { min: 0.0, max: 4320.0, step: 20.0, zero_below: Some(360.0) }
        );
        let scale = setting(json!({ "id": "lodmod.render_scale", "type": "float", "default": 1.0, "range": { "min": 0.5, "max": 4.0 }, "ui": { "step": 0.25 } }));
        assert_eq!(control_for(&scale), Control::Slider { min: 0.5, max: 4.0, step: 0.25, zero_below: None });
    }

    #[test]
    fn grid_rule_columns_start_from_usable_values() {
        let blocked = setting(json!({ "id": "lodmod.high_grids.blocked_from_grid", "type": "list", "default": [], "items": { "type": "table", "fields": [{ "key": "from", "required": true, "type": "int" }, { "key": "grid", "required": true, "type": "int" }] } }));
        let Control::Rules { columns } = control_for(&blocked) else { panic!("expected rules") };
        assert_eq!(columns[0].fallback, json!(0x1517));
        assert_eq!(columns[1].fallback, json!(0x1121));
        let hidden = setting(json!({ "id": "lodmod.high_grids.hidden_meshes", "type": "list", "default": [], "items": { "type": "table", "fields": [{ "key": "mesh", "required": false, "type": "string" }, { "key": "from_rings", "required": false, "type": "int" }, { "key": "grid", "required": false, "type": "int" }] } }));
        let Control::Rules { columns } = control_for(&hidden) else { panic!("expected rules") };
        assert_eq!(columns[1].kind, ColumnKind::Choice { options: vec![2, 3, 4], hex: false });
        assert_eq!(columns[1].fallback, json!(2));
        assert_eq!(columns[2].fallback, json!(0));
    }
}
