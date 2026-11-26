use std::rc::Rc;

use gpui::{
    AnyElement, AnyView, App, ClickEvent, CursorStyle, ElementId, FontWeight, IntoElement,
    MouseButton, ParentElement, Pixels, RenderOnce, SharedString, Window, div, prelude::*, px,
    relative,
};

use super::button::{Button, ButtonStyle, TintColor};
use super::chip::CountBadge;
use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LINE_HEIGHT_TIGHT, Label, LabelSize};
use super::text::Text;
use super::tooltip::Tooltip;
use super::util::{ControllerMode, KeyboardFocus, bounds_tracker, pointer_capture};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::{
    ActionHandler, ButtonCommon, ClickHandler, Clickable, ToggleState, TooltipBuilder,
};

pub struct PageHeader {
    title: SharedString,
    info: Option<(SharedString, Option<ActionHandler>)>,
    dirty: bool,
    start_slot: Option<AnyElement>,
    after_title: Option<AnyElement>,
    actions: Vec<AnyElement>,
    save_discard: Option<(ActionHandler, ActionHandler)>,
}

crate::impl_into_element!(PageHeader);

impl PageHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            info: None,
            dirty: false,
            start_slot: None,
            after_title: None,
            actions: Vec::new(),
            save_discard: None,
        }
    }

    pub fn info(
        mut self,
        tooltip: impl Into<SharedString>,
        on_reveal: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.info = Some((tooltip.into(), Some(Rc::new(on_reveal))));
        self
    }

    pub fn info_tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.info = Some((tooltip.into(), None));
        self
    }

    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub fn start_slot(mut self, slot: impl IntoElement) -> Self {
        self.start_slot = Some(slot.into_any_element());
        self
    }

    pub fn after_title(mut self, slot: impl IntoElement) -> Self {
        self.after_title = Some(slot.into_any_element());
        self
    }

    pub fn actions(mut self, actions: impl IntoElement) -> Self {
        self.actions.push(actions.into_any_element());
        self
    }

    pub fn save_discard(
        mut self,
        on_save: impl Fn(&mut Window, &mut App) + 'static,
        on_discard: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.save_discard = Some((Rc::new(on_save), Rc::new(on_discard)));
        self
    }
}

impl RenderOnce for PageHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let accent = theme.colors.accent_primary;
        let info_hover = accent.opacity(theme.alpha.subtle);
        let show_save = self.dirty;
        let menu = theme.id == crate::ThemeId::NierNew;
        let heading_name = self.title.clone();
        let heading = if menu {
            let scale = (f32::from(sizes.window_width) / 1840.).clamp(0.43, 0.74);
            let title = self.title.to_uppercase();
            let width = yp_nier_ui::Theme::bundled().measure(&title, 2, scale)[0]
                + title.chars().count() as f32 * 10. * scale;
            if crate::theme::MenuTextSize::percent(cx) == 100
                && width <= f32::from(sizes.window_width) * 0.52
            {
                yp_nier_ui::MenuTitle::new(title)
                    .for_page(super::WindowExt::yp_root(window).and_then(|root| root.read(cx).menu_page_start()))
                    .scale(scale)
                    .into_any_element()
            } else {
                Label::new(self.title)
                    .size(LabelSize::XLarge)
                    .semibold()
                    .into_any_element()
            }
        } else {
            v_flex()
                .min_w_0()
                .truncate()
                .text_size(sizes.font_xl)
                .line_height(relative(LINE_HEIGHT_TIGHT))
                .font_weight(FontWeight::BOLD)
                .text_color(theme.colors.text_primary)
                .child(Text::new(self.title))
                .into_any_element()
        };
        h_flex()
            .w_full()
            .flex_none()
            .gap(sizes.spacing_md)
            .min_w_0()
            .when(menu, |el| el.flex_wrap().gap_y(px(6.)).py(px(12.)))
            .when(!menu, |el| {
                el.px(sizes.card_padding_h)
                    .py(sizes.card_padding_v)
                    .bg(theme.colors.surface_medium)
                    .border_b_1()
                    .border_color(theme.colors.border_light)
            })
            .children(self.start_slot)
            .child(
                div()
                    .id("page-heading")
                    .role(gpui::Role::Heading)
                    .aria_level(1)
                    .aria_label(heading_name)
                    .min_w_0()
                    .max_w_full()
                    .child(heading),
            )
            .when_some(self.info, |this, (tooltip, on_reveal)| {
                let clickable = on_reveal.is_some();
                let controller_blocked = clickable && ControllerMode::is_active(cx);
                let tooltip = if controller_blocked {
                    strings.controller_requires_keyboard().into()
                } else {
                    tooltip
                };
                this.child(
                    div()
                        .id("yp-page-header-info")
                        .role(if clickable {
                            gpui::Role::Button
                        } else {
                            gpui::Role::Note
                        })
                        .aria_label(tooltip.clone())
                        .group("yp-page-header-info")
                        .flex_none()
                        .p(px(2.))
                        .rounded(Sizes::CONTROL_RADIUS)
                        .when(controller_blocked, |this| {
                            this.opacity(theme.alpha.half).cursor_not_allowed()
                        })
                        .when(clickable && !controller_blocked, |this| {
                            this.keyboard_activate(cx)
                                .cursor_pointer()
                                .hover(move |style| style.bg(info_hover))
                        })
                        .child(
                            Icon::new(IconName::InfoOutline)
                                .size(IconSize::Small)
                                .color(Color::Muted)
                                .group_hover_color("yp-page-header-info", Color::Accent),
                        )
                        .tooltip(Tooltip::text(tooltip))
                        .when_some(on_reveal.filter(|_| !controller_blocked), |this, handler| {
                            this.on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                handler(window, cx);
                            })
                        }),
                )
            })
            .when(self.dirty, |this| {
                this.child(
                    div()
                        .flex_none()
                        .text_size(sizes.font_xl)
                        .line_height(relative(LINE_HEIGHT_TIGHT))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.colors.warning)
                        .child("*"),
                )
            })
            .when_some(self.after_title, |this, slot| {
                this.child(h_flex().flex_none().pl(sizes.spacing_md).child(slot))
            })
            .child(div().flex_1())
            .children(self.actions)
            .when_some(
                self.save_discard.filter(|_| show_save),
                |this, (on_save, on_discard)| {
                    this.child(
                        h_flex()
                            .flex_none()
                            .gap(sizes.padding_xs)
                            .child(
                                Button::new("yp-page-header-save", strings.button_save())
                                    .style(ButtonStyle::Tinted(TintColor::Success))
                                    .on_click(move |_, window, cx| on_save(window, cx)),
                            )
                            .child(
                                Button::new("yp-page-header-discard", strings.button_discard())
                                    .style(ButtonStyle::Tinted(TintColor::Muted))
                                    .on_click(move |_, window, cx| on_discard(window, cx)),
                            ),
                    )
                },
            )
    }
}

#[derive(Default)]
struct MeasuredWidth {
    width: Option<Pixels>,
}

fn measured_width(id: ElementId, window: &mut Window, cx: &mut App) -> (Pixels, AnyElement) {
    let state = window.use_keyed_state(id, cx, |_, _| MeasuredWidth::default());
    let measured = state.read(cx).width;
    let width = measured.unwrap_or(window.sizes(cx).window_width);
    let tracker = bounds_tracker(move |bounds, window, cx| {
        let width = bounds.size.width;
        let changed = state.read(cx).width != Some(width);
        if changed {
            let state = state.clone();
            window.defer(cx, move |_, cx| {
                state.update(cx, |state, cx| {
                    state.width = Some(width);
                    cx.notify();
                });
            });
        }
    });
    (width, tracker.into_any_element())
}

pub struct ResponsiveGrid {
    id: ElementId,
    item_width: Pixels,
    max_columns: u16,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(ResponsiveGrid);

impl ResponsiveGrid {
    pub fn new(id: impl Into<ElementId>, item_width: Pixels, max_columns: u16) -> Self {
        Self {
            id: id.into(),
            item_width,
            max_columns: max_columns.max(1),
            children: Vec::new(),
        }
    }
}

impl ParentElement for ResponsiveGrid {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}

impl RenderOnce for ResponsiveGrid {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let gap = window.sizes(cx).spacing_md;
        let (width, tracker) = measured_width(self.id, window, cx);
        let columns = (f32::from(width + gap) / f32::from(self.item_width + gap).max(1.))
            .floor()
            .clamp(1., f32::from(self.max_columns)) as u16;
        div().relative().w_full().min_w_0().child(tracker).child(
            div()
                .w_full()
                .min_w_0()
                .grid()
                .grid_cols(columns)
                .gap(gap)
                .children(self.children),
        )
    }
}

pub struct TwoColumn {
    id: ElementId,
    left: AnyElement,
    right: AnyElement,
    breakpoint: Pixels,
    ratio: (f32, f32),
    fill_height: bool,
}

crate::impl_into_element!(TwoColumn);

impl TwoColumn {
    pub fn new(left: impl IntoElement, right: impl IntoElement) -> Self {
        Self {
            id: ElementId::Name("yp-two-column".into()),
            left: left.into_any_element(),
            right: right.into_any_element(),
            breakpoint: Sizes::TWO_COLUMN_BREAKPOINT,
            ratio: (1., 1.),
            fill_height: false,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn breakpoint(mut self, breakpoint: Pixels) -> Self {
        self.breakpoint = breakpoint;
        self
    }

    pub fn ratio(mut self, left: f32, right: f32) -> Self {
        self.ratio = (left.max(0.01), right.max(0.01));
        self
    }

    pub fn fill_height(mut self, fill_height: bool) -> Self {
        self.fill_height = fill_height;
        self
    }
}

impl RenderOnce for TwoColumn {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let (width, tracker) = measured_width(self.id, window, cx);
        let stack = width < self.breakpoint;
        let fill = self.fill_height;
        let total = self.ratio.0 + self.ratio.1;
        let container = div()
            .relative()
            .w_full()
            .flex()
            .gap(sizes.spacing_lg)
            .when(fill, |this| this.h_full().min_h_0())
            .child(tracker);
        let pane = move |frame: gpui::Div, content: AnyElement| {
            if fill {
                frame
                    .relative()
                    .child(div().absolute().inset_0().child(content))
            } else {
                frame.flex().flex_col().child(content)
            }
        };
        if stack {
            container
                .flex_col()
                .child(pane(
                    div().w_full().when(fill, |this| this.flex_1().min_h_0()),
                    self.left,
                ))
                .child(pane(
                    div().w_full().when(fill, |this| this.flex_1().min_h_0()),
                    self.right,
                ))
        } else {
            container
                .flex_row()
                .when(!fill, |this| this.items_start())
                .child(pane(
                    div()
                        .min_w_0()
                        .flex_basis(relative(self.ratio.0 / total))
                        .flex_grow_1()
                        .flex_shrink_1()
                        .when(fill, |this| this.h_full().min_h_0()),
                    self.left,
                ))
                .child(pane(
                    div()
                        .min_w_0()
                        .flex_basis(relative(self.ratio.1 / total))
                        .flex_grow_1()
                        .flex_shrink_1()
                        .when(fill, |this| this.h_full().min_h_0()),
                    self.right,
                ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResizableAxis {
    Horizontal,
    Vertical,
}

pub struct ResizablePanel {
    size: Pixels,
    min: Pixels,
    max: Option<Pixels>,
    content: Option<AnyElement>,
    flexible: bool,
}

pub fn resizable_panel() -> ResizablePanel {
    ResizablePanel {
        size: px(240.),
        min: px(80.),
        max: None,
        content: None,
        flexible: false,
    }
}

impl ResizablePanel {
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn min(mut self, min: Pixels) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: Pixels) -> Self {
        self.max = Some(max);
        self
    }

    pub fn flexible(mut self) -> Self {
        self.flexible = true;
        self
    }

    pub fn child(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }
}

#[derive(Default)]
struct ResizableState {
    sizes: Vec<Pixels>,
    dragging: Option<(usize, Pixels, Pixels)>,
}

type ResizeHandler = Rc<dyn Fn(&[Pixels], &mut Window, &mut App)>;

pub struct Resizable {
    id: ElementId,
    axis: ResizableAxis,
    panels: Vec<ResizablePanel>,
    on_resize: Option<ResizeHandler>,
}

crate::impl_into_element!(Resizable);

pub fn h_resizable(id: impl Into<ElementId>) -> Resizable {
    Resizable {
        id: id.into(),
        axis: ResizableAxis::Horizontal,
        panels: Vec::new(),
        on_resize: None,
    }
}

pub fn v_resizable(id: impl Into<ElementId>) -> Resizable {
    Resizable {
        id: id.into(),
        axis: ResizableAxis::Vertical,
        panels: Vec::new(),
        on_resize: None,
    }
}

impl Resizable {
    pub fn child(mut self, panel: ResizablePanel) -> Self {
        self.panels.push(panel);
        self
    }

    pub fn on_resize(
        mut self,
        handler: impl Fn(&[Pixels], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resize = Some(Rc::new(handler));
        self
    }
}

pub fn clamp_panel_size(size: Pixels, min: Pixels, max: Option<Pixels>) -> Pixels {
    let size = size.max(min);
    match max {
        Some(max) if max >= min => size.min(max),
        _ => size,
    }
}

impl RenderOnce for Resizable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let axis = self.axis;
        let panel_count = self.panels.len();
        let defaults: Vec<Pixels> = self.panels.iter().map(|panel| panel.size).collect();
        let limits: Vec<(Pixels, Option<Pixels>)> = self
            .panels
            .iter()
            .map(|panel| (panel.min, panel.max))
            .collect();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| ResizableState::default());
        state.update(cx, |state, _| {
            if state.sizes.len() != panel_count {
                state.sizes = defaults.clone();
            }
        });
        let (sizes, dragging) = {
            let state = state.read(cx);
            (state.sizes.clone(), state.dragging)
        };
        let accent = theme.colors.accent_primary;
        let handle_hover = accent.opacity(theme.alpha.line);
        let on_resize = self.on_resize;
        let move_state = state.clone();
        let up_state = state.clone();
        let move_limits = limits.clone();
        let move_handler = on_resize.clone();
        let capture = pointer_capture(
            dragging.is_some(),
            move |event, window, cx| {
                let Some((index, start_pointer, start_size)) = move_state.read(cx).dragging else {
                    return;
                };
                let pointer = match axis {
                    ResizableAxis::Horizontal => event.position.x,
                    ResizableAxis::Vertical => event.position.y,
                };
                let (min, max) = move_limits[index];
                let next = clamp_panel_size(start_size + (pointer - start_pointer), min, max);
                let sizes = move_state.update(cx, |state, cx| {
                    if state.sizes[index] != next {
                        state.sizes[index] = next;
                        cx.notify();
                    }
                    state.sizes.clone()
                });
                if let Some(handler) = &move_handler {
                    handler(&sizes, window, cx);
                }
            },
            move |_, _, cx| {
                up_state.update(cx, |state, cx| {
                    state.dragging = None;
                    cx.notify();
                });
            },
        );
        let mut container = div()
            .id(self.id)
            .relative()
            .size_full()
            .flex()
            .child(capture);
        container = match axis {
            ResizableAxis::Horizontal => container.flex_row(),
            ResizableAxis::Vertical => container.flex_col(),
        };
        for (index, panel) in self.panels.into_iter().enumerate() {
            let is_last = index + 1 == panel_count;
            let size = clamp_panel_size(sizes[index], panel.min, panel.max);
            let flexible = panel.flexible || is_last;
            let pane = div()
                .relative()
                .overflow_hidden()
                .map(|this| match (flexible, axis) {
                    (true, _) => this.flex_1().min_w_0().min_h_0(),
                    (false, ResizableAxis::Horizontal) => this.flex_none().w(size).h_full(),
                    (false, ResizableAxis::Vertical) => this.flex_none().h(size).w_full(),
                })
                .children(
                    panel
                        .content
                        .map(|content| div().absolute().inset_0().child(content)),
                );
            container = container.child(pane);
            if !is_last {
                let handle_state = state.clone();
                let active = dragging.is_some_and(|(dragged, _, _)| dragged == index);
                let handle = div()
                    .id(("yp-resize-handle", index))
                    .flex_none()
                    .map(|this| match axis {
                        ResizableAxis::Horizontal => this
                            .w(Sizes::RESIZE_HANDLE_WIDTH)
                            .h_full()
                            .cursor(CursorStyle::ResizeLeftRight),
                        ResizableAxis::Vertical => this
                            .h(Sizes::RESIZE_HANDLE_WIDTH)
                            .w_full()
                            .cursor(CursorStyle::ResizeUpDown),
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(active, |this| this.bg(accent.opacity(theme.alpha.soft)))
                    .hover(move |style| style.bg(handle_hover))
                    .child(div().bg(theme.colors.border_light).map(|this| match axis {
                        ResizableAxis::Horizontal => this.w(Sizes::BORDER_WIDTH).h_full(),
                        ResizableAxis::Vertical => this.h(Sizes::BORDER_WIDTH).w_full(),
                    }))
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        cx.stop_propagation();
                        let pointer = match axis {
                            ResizableAxis::Horizontal => event.position.x,
                            ResizableAxis::Vertical => event.position.y,
                        };
                        handle_state.update(cx, |state, cx| {
                            state.dragging = Some((index, pointer, state.sizes[index]));
                            cx.notify();
                        });
                    });
                container = container.child(handle);
            }
        }
        container
    }
}

pub struct EmptyState {
    icon: Option<IconName>,
    title: SharedString,
    description: Option<SharedString>,
    action: Option<AnyElement>,
    compact: bool,
}

crate::impl_into_element!(EmptyState);

impl EmptyState {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            title: title.into(),
            description: None,
            action: None,
            compact: false,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let icon_size = if self.compact {
            sizes.icon_lg
        } else {
            sizes.icon_lg * 2.
        };
        v_flex()
            .w_full()
            .items_center()
            .justify_center()
            .gap(sizes.spacing_md)
            .p(if self.compact {
                sizes.padding_md
            } else {
                sizes.padding_xl
            })
            .when(!self.compact, |this| this.h_full())
            .when_some(self.icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::Custom(icon_size))
                        .color(Color::Muted)
                        .alpha(cx.theme().alpha.glass),
                )
            })
            .child(
                Label::new(self.title)
                    .size(LabelSize::Default)
                    .color(Color::Secondary)
                    .bold()
                    .centered(),
            )
            .when_some(self.description, |this, description| {
                this.child(
                    Label::new(description)
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .centered(),
                )
            })
            .children(self.action)
    }
}

pub struct KeyValueRow {
    label: SharedString,
    value: Option<SharedString>,
    value_slot: Option<AnyElement>,
    value_color: Color,
    mono: bool,
    label_width: Option<Pixels>,
}

crate::impl_into_element!(KeyValueRow);

impl KeyValueRow {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: Some(value.into()),
            value_slot: None,
            value_color: Color::Default,
            mono: false,
            label_width: None,
        }
    }

    pub fn with_slot(label: impl Into<SharedString>, value: impl IntoElement) -> Self {
        Self {
            label: label.into(),
            value: None,
            value_slot: Some(value.into_any_element()),
            value_color: Color::Default,
            mono: false,
            label_width: None,
        }
    }

    pub fn value_color(mut self, color: Color) -> Self {
        self.value_color = color;
        self
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn label_width(mut self, width: Pixels) -> Self {
        self.label_width = Some(width);
        self
    }
}

impl RenderOnce for KeyValueRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let value = self.value.map(|value| {
            let label = Label::new(value)
                .size(LabelSize::Small)
                .color(self.value_color);
            if self.mono { label.mono() } else { label }
        });
        h_flex()
            .w_full()
            .items_start()
            .gap(sizes.spacing_lg)
            .py(sizes.spacing_xs)
            .child(
                v_flex()
                    .flex_none()
                    .map(|this| match self.label_width {
                        Some(width) => this.w(width),
                        None => this.w(relative(0.35)),
                    })
                    .child(
                        Label::new(self.label)
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .children(value)
                    .children(self.value_slot),
            )
    }
}

#[derive(Default)]
pub struct DescriptionList {
    rows: Vec<KeyValueRow>,
    label_width: Option<Pixels>,
}

crate::impl_into_element!(DescriptionList);

impl DescriptionList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row(mut self, label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.rows.push(KeyValueRow::new(label, value));
        self
    }

    pub fn item(mut self, row: KeyValueRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn label_width(mut self, width: Pixels) -> Self {
        self.label_width = Some(width);
        self
    }
}

impl RenderOnce for DescriptionList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let label_width = self.label_width;
        v_flex()
            .w_full()
            .gap(sizes.spacing_xs)
            .children(self.rows.into_iter().map(|row| match label_width {
                Some(width) if row.label_width.is_none() => row.label_width(width),
                _ => row,
            }))
    }
}

pub struct StepDots {
    count: usize,
    current: usize,
}

crate::impl_into_element!(StepDots);

impl StepDots {
    pub fn new(count: usize, current: usize) -> Self {
        Self { count, current }
    }
}

impl RenderOnce for StepDots {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let dot = Sizes::INDICATOR_SIZE;
        h_flex()
            .flex_none()
            .gap(px(6.))
            .children((0..self.count).map(|index| {
                let active = index == self.current;
                let done = index < self.current;
                div()
                    .h(dot)
                    .w(if active { dot * 2.5 } else { dot })
                    .rounded(dot)
                    .bg(if active {
                        theme.colors.accent_primary
                    } else if done {
                        theme.colors.accent_primary.opacity(theme.alpha.half)
                    } else {
                        theme.colors.border_medium
                    })
            }))
    }
}

pub struct StepButton {
    id: ElementId,
    icon: IconName,
    disabled: bool,
    on_click: Option<ClickHandler>,
    tooltip: Option<TooltipBuilder>,
}

crate::impl_into_element!(StepButton);

impl StepButton {
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            disabled: false,
            on_click: None,
            tooltip: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Rc::new(tooltip));
        self
    }
}

impl RenderOnce for StepButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let accent = theme.colors.accent_primary;
        let enabled = !self.disabled && self.on_click.is_some();
        let group = SharedString::from(format!("yp-step-button-{}", self.id));
        let on_click = self.on_click;
        div()
            .id(self.id)
            .group(group.clone())
            .flex_none()
            .p(px(4.))
            .rounded(Sizes::CONTROL_RADIUS)
            .border_1()
            .border_color(theme.colors.border_light)
            .bg(theme.colors.surface_light)
            .when(!enabled, |this| this.cursor_not_allowed())
            .when(enabled, |this| {
                this.cursor_pointer()
                    .keyboard_focus(cx)
                    .hover(move |style| {
                        style
                            .bg(accent.opacity(theme.alpha.hover))
                            .border_color(accent)
                    })
                    .when_some(on_click, |this, handler| {
                        this.on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx);
                        })
                    })
            })
            .child(
                Icon::new(self.icon)
                    .size(IconSize::Small)
                    .color(if enabled {
                        Color::Secondary
                    } else {
                        Color::Muted
                    })
                    .map(|icon| {
                        if enabled {
                            icon.group_hover_color(group, Color::Accent)
                        } else {
                            icon
                        }
                    }),
            )
            .when_some(self.tooltip, |this, tooltip| {
                this.tooltip(move |window, cx| tooltip(window, cx))
            })
    }
}

pub struct Tab {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    end_slot: Option<AnyElement>,
    count: Option<usize>,
    selected: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
    tooltip: Option<TooltipBuilder>,
}

crate::impl_into_element!(Tab);

impl Tab {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: SharedString::default(),
            icon: None,
            end_slot: None,
            count: None,
            selected: false,
            disabled: false,
            on_click: None,
            tooltip: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.end_slot = Some(slot.into_any_element());
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn toggle_state(mut self, state: impl Into<ToggleState>) -> Self {
        self.selected = state.into().selected();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Rc::new(tooltip));
        self
    }
}

impl RenderOnce for Tab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let accent = theme.colors.accent_primary;
        let selected = self.selected;
        let disabled = self.disabled;
        let color = if selected {
            Color::Accent
        } else {
            Color::Secondary
        };
        let hover_background = accent.opacity(theme.alpha.subtle);
        let on_click = self.on_click;
        let group = SharedString::from(format!("yp-tab-{}", self.id));
        h_flex()
            .id(self.id)
            .group(group.clone())
            .flex_none()
            .gap(sizes.spacing_md)
            .px(sizes.tab_padding_h)
            .py(sizes.padding_sm)
            .border_b_2()
            .border_color(if selected {
                accent
            } else {
                theme.colors.transparent
            })
            .text_size(sizes.tab_font)
            .line_height(relative(LINE_HEIGHT_BODY))
            .font_weight(if selected {
                FontWeight::BOLD
            } else {
                FontWeight::MEDIUM
            })
            .text_color(color.color(cx))
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .keyboard_focus(cx)
                    .hover(move |style| style.bg(hover_background).text_color(accent))
                    .when_some(on_click, |this, handler| {
                        this.on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx);
                        })
                    })
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::Small)
                        .color(color)
                        .map(|icon| {
                            if disabled {
                                icon
                            } else {
                                icon.group_hover_color(group, Color::Accent)
                            }
                        }),
                )
            })
            .child(Text::new(self.label))
            .when_some(self.count, |this, count| this.child(CountBadge::new(count)))
            .children(self.end_slot)
            .when_some(self.tooltip, |this, tooltip| {
                this.tooltip(move |window, cx| tooltip(window, cx))
            })
    }
}

#[derive(Default)]
pub struct TabBar {
    tabs: Vec<Tab>,
    start_slot: Option<AnyElement>,
    end_slot: Option<AnyElement>,
    borderless: bool,
}

crate::impl_into_element!(TabBar);

impl TabBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }

    pub fn tabs(mut self, tabs: impl IntoIterator<Item = Tab>) -> Self {
        self.tabs.extend(tabs);
        self
    }

    pub fn start_slot(mut self, slot: impl IntoElement) -> Self {
        self.start_slot = Some(slot.into_any_element());
        self
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.end_slot = Some(slot.into_any_element());
        self
    }

    pub fn borderless(mut self) -> Self {
        self.borderless = true;
        self
    }
}

impl RenderOnce for TabBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        h_flex()
            .w_full()
            .flex_none()
            .gap(sizes.spacing_sm)
            .when(!self.borderless, |this| {
                this.border_b_1().border_color(theme.colors.border_light)
            })
            .children(self.start_slot)
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .children(self.tabs),
            )
            .children(self.end_slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_sizes_respect_min_and_max() {
        assert_eq!(clamp_panel_size(px(50.), px(80.), None), px(80.));
        assert_eq!(
            clamp_panel_size(px(500.), px(80.), Some(px(300.))),
            px(300.)
        );
        assert_eq!(
            clamp_panel_size(px(120.), px(80.), Some(px(300.))),
            px(120.)
        );
        assert_eq!(clamp_panel_size(px(500.), px(80.), Some(px(40.))), px(500.));
    }
}
