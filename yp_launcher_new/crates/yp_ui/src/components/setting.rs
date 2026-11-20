use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, IntoElement, Pixels, RenderOnce,
    SharedString, Window, div, prelude::*, px, relative,
};

use super::callout::Callout;
use super::chip::{ApplyKind, ApplyMarker, ChipTone, Indicator};
use super::label::{LINE_HEIGHT_BODY, LabelSize};
use super::number_input::{NumberField, NumberKind};
use super::select::{Select, SelectItem};
use super::selectable_text::SelectableText;
use super::slider::Slider;
use super::toggle::Switch;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Severity, Sizes};
use crate::traits::{ClickHandler, ToggleState, ValueHandler};

pub struct SettingRow {
    id: ElementId,
    label: SharedString,
    description: Option<SharedString>,
    description_slot: Option<AnyElement>,
    apply_marker: Option<ApplyKind>,
    changed: bool,
    disabled: bool,
    hoverable: bool,
    leading: Option<AnyElement>,
    control: Option<AnyElement>,
    value_label: Option<SharedString>,
    below: Option<AnyElement>,
    error: Option<SharedString>,
    on_click: Option<ClickHandler>,
}

crate::impl_into_element!(SettingRow);

impl SettingRow {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            description_slot: None,
            apply_marker: None,
            changed: false,
            disabled: false,
            hoverable: true,
            leading: None,
            control: None,
            value_label: None,
            below: None,
            error: None,
            on_click: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn description_opt(mut self, description: Option<SharedString>) -> Self {
        self.description = description;
        self
    }

    pub fn description_slot(mut self, slot: impl IntoElement) -> Self {
        self.description_slot = Some(slot.into_any_element());
        self
    }

    pub fn apply_marker(mut self, kind: impl Into<Option<ApplyKind>>) -> Self {
        self.apply_marker = kind.into();
        self
    }

    pub fn changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn hoverable(mut self, hoverable: bool) -> Self {
        self.hoverable = hoverable;
        self
    }

    pub fn leading(mut self, leading: impl IntoElement) -> Self {
        self.leading = Some(leading.into_any_element());
        self
    }

    pub fn control(mut self, control: impl IntoElement) -> Self {
        self.control = Some(control.into_any_element());
        self
    }

    pub fn value_label(mut self, value: impl Into<SharedString>) -> Self {
        self.value_label = Some(value.into());
        self
    }

    pub fn below(mut self, below: impl IntoElement) -> Self {
        self.below = Some(below.into_any_element());
        self
    }

    pub fn error(mut self, error: impl Into<Option<SharedString>>) -> Self {
        self.error = error.into();
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SettingRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let hover_background = theme.colors.surface_light;
        let clickable = self.on_click.is_some() && !self.disabled;
        let on_click = self.on_click;
        let marker_id = ElementId::from((self.id.clone(), "apply"));
        let title = h_flex()
            .gap(px(6.))
            .min_w_0()
            .child(
                v_flex()
                    .min_w_0()
                    .text_size(sizes.font_md)
                    .line_height(relative(LINE_HEIGHT_BODY))
                    .text_color(theme.colors.text_primary)
                    .child(super::text::Text::new(self.label)),
            )
            .when_some(self.apply_marker, |this, kind| {
                this.child(ApplyMarker::new(kind).id(marker_id))
            })
            .when(self.changed, |this| {
                this.child(Indicator::dot().tone(ChipTone::Success))
            });
        v_flex()
            .id(self.id)
            .w_full()
            .px(px(8.))
            .py(px(8.))
            .my(px(2.))
            .rounded(Sizes::ROW_RADIUS)
            .when(self.disabled, |this| this.opacity(theme.alpha.half))
            .when(self.hoverable && !self.disabled, |this| {
                this.hover(move |style| style.bg(hover_background))
            })
            .when(clickable, |this| {
                this.cursor_pointer().when_some(on_click, |this, handler| {
                    this.on_click(move |event, window, cx| handler(event, window, cx))
                })
            })
            .child(
                h_flex()
                    .w_full()
                    .items_start()
                    .gap(px(12.))
                    .when_some(self.leading, |this, leading| {
                        this.child(div().flex_none().pt(px(2.)).child(leading))
                    })
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(3.))
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap(px(12.))
                                    .child(h_flex().flex_1().min_w_0().child(title))
                                    .when_some(self.value_label, |this, value| {
                                        this.child(
                                            v_flex()
                                                .flex_none()
                                                .text_size(sizes.font_md)
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme.colors.accent_primary)
                                                .child(super::text::Text::new(value)),
                                        )
                                    }),
                            )
                            .when_some(self.description, |this, description| {
                                this.child(
                                    v_flex()
                                        .text_size(sizes.font_xs)
                                        .line_height(relative(1.3))
                                        .text_color(theme.colors.text_muted)
                                        .child(super::text::Text::new(description)),
                                )
                            })
                            .children(self.description_slot),
                    )
                    .when_some(self.control, |this, control| {
                        this.child(
                            v_flex()
                                .flex_none()
                                .items_end()
                                .gap(px(3.))
                                .child(control)
                                .when_some(self.error, |this, error| {
                                    this.child(
                                        div()
                                            .text_size(LabelSize::XSmall.pixels(&sizes))
                                            .text_color(theme.colors.error)
                                            .child(error),
                                    )
                                }),
                        )
                    }),
            )
            .children(self.below)
    }
}

pub struct SettingSwitch {
    id: ElementId,
    label: SharedString,
    value: bool,
    description: Option<SharedString>,
    apply_marker: Option<ApplyKind>,
    changed: bool,
    disabled: bool,
    on_change: Option<ValueHandler<bool>>,
}

crate::impl_into_element!(SettingSwitch);

impl SettingSwitch {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, value: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            description: None,
            apply_marker: None,
            changed: false,
            disabled: false,
            on_change: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn apply_marker(mut self, kind: impl Into<Option<ApplyKind>>) -> Self {
        self.apply_marker = kind.into();
        self
    }

    pub fn changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SettingSwitch {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let next = !self.value;
        let row_handler = self.on_change.clone();
        let switch_handler = self.on_change;
        let switch = Switch::new((self.id.clone(), "switch"), self.value)
            .disabled(self.disabled)
            .when_some(switch_handler, |this, handler| {
                this.on_click(move |state: &ToggleState, window, cx| {
                    handler(&state.selected(), window, cx);
                })
            });
        SettingRow::new(self.id, self.label)
            .description_opt(self.description)
            .apply_marker(self.apply_marker)
            .changed(self.changed)
            .disabled(self.disabled)
            .leading(switch)
            .when_some(row_handler, |this, handler| {
                this.on_click(move |_, window, cx| handler(&next, window, cx))
            })
    }
}

pub struct SettingNumber {
    id: ElementId,
    label: SharedString,
    value: f64,
    kind: NumberKind,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    steppers: bool,
    description: Option<SharedString>,
    apply_marker: Option<ApplyKind>,
    changed: bool,
    disabled: bool,
    width: Option<Pixels>,
    on_change: Option<ValueHandler<f64>>,
    on_commit: Option<ValueHandler<f64>>,
}

crate::impl_into_element!(SettingNumber);

impl SettingNumber {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, value: f64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            kind: NumberKind::Int,
            min: None,
            max: None,
            step: 1.,
            steppers: false,
            description: None,
            apply_marker: None,
            changed: false,
            disabled: false,
            width: None,
            on_change: None,
            on_commit: None,
        }
    }

    pub fn kind(mut self, kind: NumberKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn min(mut self, min: impl Into<Option<f64>>) -> Self {
        self.min = min.into();
        self
    }

    pub fn max(mut self, max: impl Into<Option<f64>>) -> Self {
        self.max = max.into();
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn steppers(mut self, steppers: bool) -> Self {
        self.steppers = steppers;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn apply_marker(mut self, kind: impl Into<Option<ApplyKind>>) -> Self {
        self.apply_marker = kind.into();
        self
    }

    pub fn changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_commit(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_commit = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SettingNumber {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let field = NumberField::new((self.id.clone(), "field"), self.value)
            .kind(self.kind)
            .min(self.min)
            .max(self.max)
            .step(self.step)
            .steppers(self.steppers)
            .disabled(self.disabled)
            .when_some(self.width, |this, width| this.width(width))
            .when_some(self.on_change, |this, handler| {
                this.on_change(move |value, window, cx| handler(value, window, cx))
            })
            .when_some(self.on_commit, |this, handler| {
                this.on_commit(move |value, window, cx| handler(value, window, cx))
            });
        SettingRow::new(self.id, self.label)
            .description_opt(self.description)
            .apply_marker(self.apply_marker)
            .changed(self.changed)
            .disabled(self.disabled)
            .control(field)
    }
}

pub struct SettingSlider {
    id: ElementId,
    label: SharedString,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    decimals: usize,
    ticks: Vec<f32>,
    value_label: Option<SharedString>,
    description: Option<SharedString>,
    apply_marker: Option<ApplyKind>,
    changed: bool,
    disabled: bool,
    on_change: Option<ValueHandler<f32>>,
    on_commit: Option<ValueHandler<f32>>,
}

crate::impl_into_element!(SettingSlider);

impl SettingSlider {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            min: 0.,
            max: 1.,
            step: 0.01,
            decimals: 2,
            ticks: Vec::new(),
            value_label: None,
            description: None,
            apply_marker: None,
            changed: false,
            disabled: false,
            on_change: None,
            on_commit: None,
        }
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn ticks(mut self, ticks: Vec<f32>) -> Self {
        self.ticks = ticks;
        self
    }

    pub fn value_label(mut self, label: impl Into<SharedString>) -> Self {
        self.value_label = Some(label.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn apply_marker(mut self, kind: impl Into<Option<ApplyKind>>) -> Self {
        self.apply_marker = kind.into();
        self
    }

    pub fn changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_commit(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_commit = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SettingSlider {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (low, high) = if self.min <= self.max {
            (self.min, self.max)
        } else {
            (self.max, self.min)
        };
        let clamped = if self.value.is_finite() {
            self.value.clamp(low, high)
        } else {
            low
        };
        let value_label = self
            .value_label
            .unwrap_or_else(|| SharedString::from(format!("{clamped:.*}", self.decimals)));
        let slider = Slider::new((self.id.clone(), "slider"))
            .value(clamped)
            .min(self.min)
            .max(self.max)
            .step(self.step)
            .ticks(self.ticks)
            .disabled(self.disabled)
            .when_some(self.on_change, |this, handler| {
                this.on_change(move |value, window, cx| handler(value, window, cx))
            })
            .when_some(self.on_commit, |this, handler| {
                this.on_commit(move |value, window, cx| handler(value, window, cx))
            });
        SettingRow::new(self.id, self.label)
            .description_opt(self.description)
            .apply_marker(self.apply_marker)
            .changed(self.changed)
            .disabled(self.disabled)
            .hoverable(false)
            .value_label(value_label)
            .below(slider)
    }
}

pub struct SettingSelect<T: Clone + 'static> {
    id: ElementId,
    label: SharedString,
    selected_label: SharedString,
    items: Vec<SelectItem<T>>,
    selected_index: Option<usize>,
    description: Option<SharedString>,
    apply_marker: Option<ApplyKind>,
    changed: bool,
    disabled: bool,
    searchable: bool,
    max_width: Pixels,
    on_select: Option<ValueHandler<T>>,
}

crate::impl_into_element!([T: Clone + 'static] SettingSelect<T>);

impl<T: Clone + 'static> SettingSelect<T> {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        selected_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected_label: selected_label.into(),
            items: Vec::new(),
            selected_index: None,
            description: None,
            apply_marker: None,
            changed: false,
            disabled: false,
            searchable: false,
            max_width: px(200.),
            on_select: None,
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectItem<T>>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn selected_index(mut self, index: impl Into<Option<usize>>) -> Self {
        self.selected_index = index.into();
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn apply_marker(mut self, kind: impl Into<Option<ApplyKind>>) -> Self {
        self.apply_marker = kind.into();
        self
    }

    pub fn changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = width;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl<T: Clone + 'static> RenderOnce for SettingSelect<T> {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let select = Select::new((self.id.clone(), "select"), self.selected_label)
            .items(self.items)
            .selected_index(self.selected_index)
            .max_width(self.max_width)
            .highlight(true)
            .searchable(self.searchable)
            .disabled(self.disabled)
            .when_some(self.on_select, |this, handler| {
                this.on_select(move |value: &T, window, cx| handler(value, window, cx))
            });
        SettingRow::new(self.id, self.label)
            .description_opt(self.description)
            .apply_marker(self.apply_marker)
            .changed(self.changed)
            .disabled(self.disabled)
            .control(select)
    }
}

pub struct ConfigErrorBanner {
    id: ElementId,
    file_name: SharedString,
    error: SharedString,
}

crate::impl_into_element!(ConfigErrorBanner);

impl ConfigErrorBanner {
    pub fn new(file_name: impl Into<SharedString>, error: impl Into<SharedString>) -> Self {
        let file_name = file_name.into();
        Self {
            id: ElementId::Name(SharedString::from(format!("config-error-{file_name}"))),
            file_name,
            error: error.into(),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for ConfigErrorBanner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        div().w_full().pb(sizes.padding_md).child(
            Callout::new()
                .id(self.id.clone())
                .severity(Severity::Error)
                .title(strings.config_parse_error_title(&self.file_name))
                .description(strings.config_parse_error_body())
                .description_slot(
                    SelectableText::new((self.id, "details"), self.error)
                        .mono()
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
        )
    }
}

pub struct ConfigSection {
    title: SharedString,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(ConfigSection);

impl ConfigSection {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
        }
    }
}

impl gpui::ParentElement for ConfigSection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ConfigSection {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        super::card::Section::new(self.title).children(self.children)
    }
}
