use serde_json::{Map, Value as Json};
use yp_ui::prelude::*;
use yp_ui::{
    ApplyKind, ApplyMarker, Button, ButtonSize, ButtonStyle, IconButton, IconName, IconSize, Label,
    LabelSize, NumberField, NumberKind, Select, SelectItem, TintColor, Tooltip,
};

use crate::fields::ValueSink;
use crate::text_field::TextField;

pub const HIGH_GRIDS_ROOMS: [(u32, &str); 10] = [
    (0x100, "Ruined City"),
    (0x110, "Resistance Camp"),
    (0x120, "Factory"),
    (0x130, "Amusement Park"),
    (0x140, "Pascal's Village"),
    (0x150, "Desert"),
    (0x160, "Forest"),
    (0x170, "Flooded City"),
    (0x200, "City Ruins (Route B)"),
    (0x520, "Underground"),
];

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnKind {
    Hex,
    Text,
    Choice { options: Vec<i64>, hex: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleColumn {
    pub key: SharedString,
    pub label: SharedString,
    pub kind: ColumnKind,
    pub fallback: Json,
}

impl RuleColumn {
    pub fn hex(key: &'static str, label: impl Into<SharedString>, fallback: i64) -> Self {
        Self {
            key: SharedString::new_static(key),
            label: label.into(),
            kind: ColumnKind::Hex,
            fallback: Json::from(fallback),
        }
    }

    pub fn text(key: &'static str, label: impl Into<SharedString>) -> Self {
        Self {
            key: SharedString::new_static(key),
            label: label.into(),
            kind: ColumnKind::Text,
            fallback: Json::String(String::new()),
        }
    }

    pub fn choice(
        key: &'static str,
        label: impl Into<SharedString>,
        options: Vec<i64>,
        fallback: i64,
    ) -> Self {
        Self {
            key: SharedString::new_static(key),
            label: label.into(),
            kind: ColumnKind::Choice {
                options,
                hex: false,
            },
            fallback: Json::from(fallback),
        }
    }

    pub fn room(key: &'static str, label: impl Into<SharedString>) -> Self {
        let options: Vec<i64> = HIGH_GRIDS_ROOMS
            .iter()
            .map(|(room, _)| i64::from(*room))
            .collect();
        let fallback = options.first().copied().unwrap_or_default();
        Self {
            key: SharedString::new_static(key),
            label: label.into(),
            kind: ColumnKind::Choice { options, hex: true },
            fallback: Json::from(fallback),
        }
    }

    pub fn text_owned(key: &str, label: impl Into<SharedString>) -> Self {
        Self {
            key: SharedString::from(key.to_owned()),
            label: label.into(),
            kind: ColumnKind::Text,
            fallback: Json::String(String::new()),
        }
    }

    pub fn choice_owned(key: &str, label: impl Into<SharedString>, options: Vec<i64>, fallback: i64) -> Self {
        Self {
            key: SharedString::from(key.to_owned()),
            label: label.into(),
            kind: ColumnKind::Choice { options, hex: false },
            fallback: Json::from(fallback),
        }
    }

    pub fn room_owned(key: &str, label: impl Into<SharedString>) -> Self {
        let options: Vec<i64> = HIGH_GRIDS_ROOMS.iter().map(|(room, _)| i64::from(*room)).collect();
        let fallback = options.first().copied().unwrap_or_default();
        Self {
            key: SharedString::from(key.to_owned()),
            label: label.into(),
            kind: ColumnKind::Choice { options, hex: true },
            fallback: Json::from(fallback),
        }
    }

    fn weight(&self) -> f32 {
        match self.kind {
            ColumnKind::Text => 2.,
            _ => 1.,
        }
    }
}

pub fn infer_columns(rules: &[Json]) -> Vec<RuleColumn> {
    let mut columns: Vec<RuleColumn> = Vec::new();
    for table in rules.iter().filter_map(Json::as_object) {
        for (key, value) in table {
            if columns.iter().any(|column| column.key.as_ref() == key) {
                continue;
            }
            let (kind, fallback) = match value {
                Json::String(_) => (ColumnKind::Text, Json::String(String::new())),
                _ => (ColumnKind::Hex, Json::from(0)),
            };
            columns.push(RuleColumn {
                key: SharedString::from(key.clone()),
                label: SharedString::from(key.clone()),
                kind,
                fallback,
            });
        }
    }
    columns
}

pub fn row_at(rules: &[Json], index: usize) -> Map<String, Json> {
    rules
        .get(index)
        .and_then(Json::as_object)
        .cloned()
        .unwrap_or_default()
}

fn normalized(rules: &[Json]) -> Vec<Map<String, Json>> {
    (0..rules.len()).map(|index| row_at(rules, index)).collect()
}

fn into_values(rows: Vec<Map<String, Json>>) -> Vec<Json> {
    rows.into_iter().map(Json::Object).collect()
}

pub fn with_cell(rules: &[Json], index: usize, key: &str, value: Json) -> Vec<Json> {
    let mut rows = normalized(rules);
    if let Some(row) = rows.get_mut(index) {
        row.insert(key.to_owned(), value);
    }
    into_values(rows)
}

pub fn with_added(rules: &[Json], columns: &[RuleColumn]) -> Vec<Json> {
    let mut rows = normalized(rules);
    rows.push(
        columns
            .iter()
            .map(|column| (column.key.to_string(), column.fallback.clone()))
            .collect(),
    );
    into_values(rows)
}

pub fn with_removed(rules: &[Json], index: usize) -> Vec<Json> {
    let mut rows = normalized(rules);
    if index < rows.len() {
        rows.remove(index);
    }
    into_values(rows)
}

pub fn int_from(raw: Option<&Json>, fallback: i64) -> i64 {
    match raw {
        Some(Json::Number(number)) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|value| value.trunc() as i64))
            .unwrap_or(fallback),
        Some(Json::String(text)) => text.trim().parse().unwrap_or(fallback),
        _ => fallback,
    }
}

pub fn hex_label(value: i64) -> String {
    format!("0x{value:X}")
}

pub fn option_label(option: i64, hex: bool) -> String {
    if !hex {
        return option.to_string();
    }
    let name = HIGH_GRIDS_ROOMS
        .iter()
        .find(|(room, _)| i64::from(*room) == option)
        .map(|(_, name)| *name);
    match name {
        Some(name) => format!("{}  {name}", hex_label(option)),
        None => hex_label(option),
    }
}

pub fn names_with(names: &[String], index: usize, name: &str) -> Vec<String> {
    let mut next = names.to_vec();
    if let Some(slot) = next.get_mut(index) {
        *slot = name.to_owned();
    }
    next
}

pub fn names_added(names: &[String]) -> Vec<String> {
    let mut next = names.to_vec();
    next.push(String::new());
    next
}

pub fn names_removed(names: &[String], index: usize) -> Vec<String> {
    let mut next = names.to_vec();
    if index < next.len() {
        next.remove(index);
    }
    next
}

fn names_value(names: Vec<String>) -> Json {
    Json::Array(names.into_iter().map(Json::String).collect())
}

fn list_header(
    id: &ElementId,
    label: SharedString,
    description: Option<SharedString>,
    apply: Option<ApplyKind>,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Div {
    let sizes = window.sizes(cx);
    v_flex()
        .w_full()
        .gap(sizes.spacing_xs)
        .pb(sizes.spacing_sm)
        .child(
            h_flex()
                .gap(sizes.spacing_sm)
                .child(Label::new(label))
                .when_some(apply, |this, kind| {
                    this.child(ApplyMarker::new(kind).id((id.clone(), "apply")))
                }),
        )
        .when_some(description, |this, description| {
            this.child(
                Label::new(description)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .line_height(1.3),
            )
        })
}

fn empty_line(text: SharedString, window: &Window, cx: &App) -> gpui::Div {
    let sizes = window.sizes(cx);
    div().pb(sizes.spacing_sm).child(
        Label::new(text)
            .size(LabelSize::XSmall)
            .color(Color::Muted)
            .italic(),
    )
}

fn row_frame(window: &Window, cx: &App) -> gpui::Div {
    let theme = cx.theme();
    let sizes = window.sizes(cx);
    h_flex()
        .w_full()
        .gap(sizes.spacing_md)
        .mb(sizes.spacing_sm)
        .px(sizes.padding_sm)
        .py(sizes.spacing_sm)
        .rounded(Sizes::ROW_RADIUS)
        .border_1()
        .border_color(theme.colors.border_light)
        .bg(theme.colors.surface_light)
}

fn remove_button(id: ElementId, cx: &App) -> IconButton {
    IconButton::new(id, IconName::Close)
        .icon_size(IconSize::Small)
        .icon_color(Color::Muted)
        .size(ButtonSize::Compact)
        .style(ButtonStyle::Tinted(TintColor::Error))
        .tooltip(Tooltip::text(cx.t().settings_remove_entry()))
}

fn add_button(id: ElementId, label: SharedString) -> Button {
    Button::new(id, label)
        .start_icon(IconName::Add)
        .style(ButtonStyle::Subtle)
        .size(ButtonSize::Compact)
        .color(Color::Accent)
        .label_size(LabelSize::XSmall)
}

pub struct RulesEditor {
    id: ElementId,
    label: SharedString,
    rules: Vec<Json>,
    columns: Vec<RuleColumn>,
    description: Option<SharedString>,
    apply: Option<ApplyKind>,
    sink: Option<ValueSink>,
}

yp_ui::impl_into_element!(RulesEditor);

impl RulesEditor {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, rules: Vec<Json>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            rules,
            columns: Vec::new(),
            description: None,
            apply: None,
            sink: None,
        }
    }

    pub fn columns(mut self, columns: Vec<RuleColumn>) -> Self {
        self.columns = columns;
        self
    }

    pub fn description(mut self, description: Option<SharedString>) -> Self {
        self.description = description;
        self
    }

    pub fn apply_marker(mut self, apply: Option<ApplyKind>) -> Self {
        self.apply = apply;
        self
    }

    pub fn sink(mut self, sink: ValueSink) -> Self {
        self.sink = Some(sink);
        self
    }
}

impl RenderOnce for RulesEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let id = self.id.clone();
        let rules = self.rules;
        let columns = self.columns;
        let sink = self.sink;
        let add_label = SharedString::new_static(strings.grid_rule_add());
        let empty_label = SharedString::new_static(strings.grid_rule_empty());
        let sizes = window.sizes(cx);
        let total_weight: f32 = columns
            .iter()
            .map(RuleColumn::weight)
            .sum::<f32>()
            .max(1.);
        let mut rows: Vec<AnyElement> = Vec::with_capacity(rules.len());
        for index in 0..rules.len() {
            let row = row_at(&rules, index);
            let mut cells: Vec<AnyElement> = Vec::with_capacity(columns.len());
            for column in &columns {
                let cell_id = ElementId::from((
                    id.clone(),
                    SharedString::from(format!("{index}:{}", column.key)),
                ));
                let key = column.key.clone();
                let cell_rules = rules.clone();
                let cell_sink = sink.clone();
                let emit = move |value: Json, window: &mut Window, cx: &mut App| {
                    if let Some(sink) = &cell_sink {
                        sink(
                            Json::Array(with_cell(&cell_rules, index, &key, value)),
                            window,
                            cx,
                        );
                    }
                };
                let control = match &column.kind {
                    ColumnKind::Text => {
                        let value = row
                            .get(column.key.as_ref())
                            .and_then(Json::as_str)
                            .map(str::to_owned)
                            .unwrap_or_else(|| {
                                column.fallback.as_str().unwrap_or_default().to_owned()
                            });
                        TextField::new(cell_id, value)
                            .mono()
                            .on_change(move |text, window, cx| {
                                emit(Json::String(text.to_owned()), window, cx)
                            })
                            .into_any_element()
                    }
                    ColumnKind::Hex => {
                        let fallback = column.fallback.as_i64().unwrap_or_default();
                        let value = int_from(row.get(column.key.as_ref()), fallback);
                        NumberField::new(cell_id, value as f64)
                            .kind(NumberKind::Hex)
                            .width(relative(1.))
                            .on_change(move |value, window, cx| {
                                emit(Json::from(value.round() as i64), window, cx)
                            })
                            .into_any_element()
                    }
                    ColumnKind::Choice { options, hex } => {
                        let fallback = column.fallback.as_i64().unwrap_or_default();
                        let raw = int_from(row.get(column.key.as_ref()), fallback);
                        let value = if options.contains(&raw) {
                            raw
                        } else {
                            options.first().copied().unwrap_or(raw)
                        };
                        let hex = *hex;
                        Select::new(cell_id, option_label(value, hex))
                            .items(
                                options
                                    .iter()
                                    .map(|option| SelectItem::new(option_label(*option, hex), *option)),
                            )
                            .selected_index(options.iter().position(|option| *option == value))
                            .highlight(true)
                            .full_width()
                            .on_select(move |option: &i64, window, cx| {
                                emit(Json::from(*option), window, cx)
                            })
                            .into_any_element()
                    }
                };
                cells.push(
                    v_flex()
                        .min_w_0()
                        .flex_basis(relative(column.weight() / total_weight))
                        .flex_grow_1()
                        .flex_shrink_1()
                        .gap(sizes.spacing_xs)
                        .child(
                            Label::new(column.label.clone())
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        )
                        .child(control)
                        .into_any_element(),
                );
            }
            let remove_rules = rules.clone();
            let remove_sink = sink.clone();
            rows.push(
                row_frame(window, cx)
                    .items_end()
                    .children(cells)
                    .child(
                        remove_button(ElementId::from((id.clone(), format!("remove-{index}"))), cx)
                            .on_click(move |_, window, cx| {
                                if let Some(sink) = &remove_sink {
                                    sink(
                                        Json::Array(with_removed(&remove_rules, index)),
                                        window,
                                        cx,
                                    );
                                }
                            }),
                    )
                    .into_any_element(),
            );
        }
        let add_rules = rules.clone();
        let add_columns = columns.clone();
        let add_sink = sink.clone();
        v_flex()
            .w_full()
            .py(sizes.spacing_sm)
            .px(sizes.padding_sm)
            .child(list_header(
                &id,
                self.label,
                self.description,
                self.apply,
                window,
                cx,
            ))
            .when(rules.is_empty(), |this| this.child(empty_line(empty_label, window, cx)))
            .children(rows)
            .child(
                h_flex().child(
                    add_button(ElementId::from((id.clone(), "add")), add_label).on_click(
                        move |_, window, cx| {
                            if let Some(sink) = &add_sink {
                                sink(
                                    Json::Array(with_added(&add_rules, &add_columns)),
                                    window,
                                    cx,
                                );
                            }
                        },
                    ),
                ),
            )
    }
}

pub struct NameListEditor {
    id: ElementId,
    label: SharedString,
    names: Vec<String>,
    description: Option<SharedString>,
    apply: Option<ApplyKind>,
    add_label: Option<SharedString>,
    empty_label: Option<SharedString>,
    sink: Option<ValueSink>,
}

yp_ui::impl_into_element!(NameListEditor);

impl NameListEditor {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, names: Vec<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            names,
            description: None,
            apply: None,
            add_label: None,
            empty_label: None,
            sink: None,
        }
    }

    pub fn description(mut self, description: Option<SharedString>) -> Self {
        self.description = description;
        self
    }

    pub fn apply_marker(mut self, apply: Option<ApplyKind>) -> Self {
        self.apply = apply;
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

    pub fn sink(mut self, sink: ValueSink) -> Self {
        self.sink = Some(sink);
        self
    }
}

impl RenderOnce for NameListEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let id = self.id.clone();
        let names = self.names;
        let sink = self.sink;
        let add_label = self
            .add_label
            .unwrap_or_else(|| SharedString::new_static(strings.grid_rule_add()));
        let empty_label = self
            .empty_label
            .unwrap_or_else(|| SharedString::new_static(strings.grid_rule_empty()));
        let sizes = window.sizes(cx);
        let mut rows: Vec<AnyElement> = Vec::with_capacity(names.len());
        for (index, name) in names.iter().enumerate() {
            let edit_names = names.clone();
            let edit_sink = sink.clone();
            let remove_names = names.clone();
            let remove_sink = sink.clone();
            rows.push(
                row_frame(window, cx)
                    .child(
                        div().flex_1().min_w_0().child(
                            TextField::new(
                                ElementId::from((id.clone(), format!("name-{index}"))),
                                name.clone(),
                            )
                            .mono()
                            .on_change(move |text, window, cx| {
                                if let Some(sink) = &edit_sink {
                                    sink(
                                        names_value(names_with(&edit_names, index, text)),
                                        window,
                                        cx,
                                    );
                                }
                            }),
                        ),
                    )
                    .child(
                        remove_button(ElementId::from((id.clone(), format!("remove-{index}"))), cx)
                            .on_click(move |_, window, cx| {
                                if let Some(sink) = &remove_sink {
                                    sink(
                                        names_value(names_removed(&remove_names, index)),
                                        window,
                                        cx,
                                    );
                                }
                            }),
                    )
                    .into_any_element(),
            );
        }
        let add_names = names.clone();
        let add_sink = sink.clone();
        v_flex()
            .w_full()
            .py(sizes.spacing_sm)
            .px(sizes.padding_sm)
            .child(list_header(
                &id,
                self.label,
                self.description,
                self.apply,
                window,
                cx,
            ))
            .when(names.is_empty(), |this| this.child(empty_line(empty_label, window, cx)))
            .children(rows)
            .child(
                h_flex().child(
                    add_button(ElementId::from((id.clone(), "add")), add_label).on_click(
                        move |_, window, cx| {
                            if let Some(sink) = &add_sink {
                                sink(names_value(names_added(&add_names)), window, cx);
                            }
                        },
                    ),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(pairs: &[(&str, Json)]) -> Json {
        Json::Object(
            pairs
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect(),
        )
    }

    #[test]
    fn adding_a_rule_uses_every_column_fallback() {
        let columns = vec![
            RuleColumn::room("room", "Room"),
            RuleColumn::choice("rings", "Rings", vec![1, 2, 3, 4, 5], 1),
        ];
        let rules = with_added(&[], &columns);
        assert_eq!(rules.len(), 1);
        let row = row_at(&rules, 0);
        assert_eq!(row.get("room"), Some(&Json::from(0x100)));
        assert_eq!(row.get("rings"), Some(&Json::from(1)));
    }

    #[test]
    fn editing_and_removing_keep_the_other_rows() {
        let rules = vec![
            rule(&[
                ("mesh", Json::from("a")),
                ("grid", Json::from(1)),
            ]),
            Json::from(7),
            rule(&[("mesh", Json::from("c"))]),
        ];
        let edited = with_cell(&rules, 2, "grid", Json::from(0x1121));
        assert_eq!(
            row_at(&edited, 2).get("grid"),
            Some(&Json::from(0x1121))
        );
        assert_eq!(
            row_at(&edited, 0).get("mesh"),
            Some(&Json::from("a"))
        );
        assert!(row_at(&edited, 1).is_empty());
        let removed = with_removed(&edited, 0);
        assert_eq!(removed.len(), 2);
        assert_eq!(
            row_at(&removed, 1).get("mesh"),
            Some(&Json::from("c"))
        );
        assert_eq!(with_removed(&rules, 9).len(), 3);
    }

    #[test]
    fn integers_are_read_from_any_stored_shape() {
        assert_eq!(int_from(Some(&Json::from(5)), 1), 5);
        assert_eq!(int_from(Some(&serde_json::json!(5.9)), 1), 5);
        assert_eq!(int_from(Some(&Json::from(" 12 ")), 1), 12);
        assert_eq!(int_from(Some(&Json::from("x")), 3), 3);
        assert_eq!(int_from(None, 4), 4);
    }

    #[test]
    fn room_options_show_the_hex_id_and_the_name() {
        assert_eq!(option_label(0x100, true), "0x100  Ruined City");
        assert_eq!(option_label(0x999, true), "0x999");
        assert_eq!(option_label(3, false), "3");
    }

    #[test]
    fn columns_are_inferred_from_the_stored_rows() {
        let rules = vec![rule(&[
            ("mesh", Json::from("a")),
            ("from_rings", Json::from(2)),
        ])];
        let columns = infer_columns(&rules);
        assert_eq!(columns.len(), 2);
        assert!(columns.iter().any(|column| column.kind == ColumnKind::Text));
        assert!(columns.iter().any(|column| column.kind == ColumnKind::Hex));
    }

    #[test]
    fn name_lists_edit_in_place() {
        let names = vec!["a".to_owned(), "b".to_owned()];
        assert_eq!(names_with(&names, 1, "z"), vec!["a", "z"]);
        assert_eq!(names_added(&names), vec!["a", "b", ""]);
        assert_eq!(names_removed(&names, 0), vec!["b"]);
        assert_eq!(names_removed(&names, 5), names);
    }
}
