use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Context, Anchor, DismissEvent, ElementId, Entity, EventEmitter, FocusHandle,
    Focusable, FontWeight, IntoElement, Pixels, Point, Render, RenderOnce, ScrollHandle,
    SharedString, Size, Subscription, Transformation, Window, anchored, deferred, div, percentage,
    point, prelude::*, px, relative, svg,
};

use super::icon::{Icon, IconName, IconSize};
use super::input::{self, Input, InputEvent, InputSize, InputState};
use super::label::LINE_HEIGHT_TIGHT;
use super::menu::{PopoverState, hide_popover, menu_container, show_popover};
use super::scroll::ScrollableElement;
use super::text::Text;
use super::tooltip::Tooltip;
use super::util::{KeyboardFocus, bounds_tracker};
use crate::actions::{
    Cancel, Confirm, MENU_CONTEXT, SelectFirst, SelectLast, SelectNext, SelectPrevious,
};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::ValueHandler;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectItem<T> {
    pub label: SharedString,
    pub value: T,
    pub icon: Option<IconName>,
    pub disabled: bool,
}

impl<T> SelectItem<T> {
    pub fn new(label: impl Into<SharedString>, value: T) -> Self {
        Self {
            label: label.into(),
            value,
            icon: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectMenuEntry {
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub disabled: bool,
}

type PickHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type SelectMenuBuilder = Rc<dyn Fn(&mut Window, &mut App) -> Option<Entity<SelectMenu>>>;

pub struct SelectMenu {
    entries: Vec<SelectMenuEntry>,
    visible: Vec<usize>,
    selected: Option<usize>,
    highlighted: Option<usize>,
    on_pick: PickHandler,
    search: Option<Entity<InputState>>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    min_width: Pixels,
    max_width: Pixels,
    max_height: Pixels,
    fixed_width: Option<Pixels>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<DismissEvent> for SelectMenu {}

impl Focusable for SelectMenu {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        if super::util::ControllerMode::is_active(cx) {
            return self.focus_handle.clone();
        }
        match &self.search {
            Some(search) => search.focus_handle(cx),
            None => self.focus_handle.clone(),
        }
    }
}

pub fn filter_entries(entries: &[SelectMenuEntry], query: &str) -> Vec<usize> {
    let needle = query.trim().to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| needle.is_empty() || entry.label.to_lowercase().contains(&needle))
        .map(|(index, _)| index)
        .collect()
}

impl SelectMenu {
    pub fn new(
        entries: Vec<SelectMenuEntry>,
        selected: Option<usize>,
        searchable: bool,
        on_pick: impl Fn(usize, &mut Window, &mut App) + 'static,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subscriptions = Vec::new();
        let search = searchable.then(|| {
            let placeholder = cx.t().ui_search();
            let search = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
            subscriptions.push(cx.subscribe_in(
                &search,
                window,
                |this: &mut Self, _, event: &InputEvent, window, cx| match event {
                    InputEvent::Change => this.refilter(cx),
                    InputEvent::Submit => this.confirm(&Confirm, window, cx),
                    InputEvent::Focus | InputEvent::Blur => {}
                },
            ));
            search
        });
        let visible = (0..entries.len()).collect();
        let scroll_handle = ScrollHandle::new();
        if let Some(selected) = selected {
            scroll_handle.scroll_to_item(selected);
        }
        Self {
            entries,
            visible,
            selected,
            highlighted: selected,
            on_pick: Rc::new(on_pick),
            search,
            focus_handle: cx.focus_handle(),
            scroll_handle,
            min_width: px(0.),
            max_width: Sizes::MENU_MAX_WIDTH,
            max_height: Sizes::SELECT_MENU_MAX_HEIGHT,
            fixed_width: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn entries(&self) -> &[SelectMenuEntry] {
        &self.entries
    }

    pub fn visible(&self) -> &[usize] {
        &self.visible
    }

    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    pub fn set_constraints(
        &mut self,
        min_width: Pixels,
        max_width: Pixels,
        max_height: Pixels,
        fixed_width: Option<Pixels>,
    ) {
        self.min_width = min_width;
        self.max_width = max_width;
        self.max_height = max_height;
        self.fixed_width = fixed_width;
    }

    fn refilter(&mut self, cx: &mut Context<Self>) {
        let query = self
            .search
            .as_ref()
            .map(|search| search.read(cx).value().to_owned())
            .unwrap_or_default();
        self.visible = filter_entries(&self.entries, &query);
        let keep = self
            .highlighted
            .filter(|highlighted| self.visible.contains(highlighted));
        self.highlighted = keep.or_else(|| self.first_enabled());
        cx.notify();
    }

    fn enabled_visible(&self) -> Vec<usize> {
        self.visible
            .iter()
            .copied()
            .filter(|index| !self.entries[*index].disabled)
            .collect()
    }

    fn first_enabled(&self) -> Option<usize> {
        self.enabled_visible().first().copied()
    }

    fn highlight(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        self.highlighted = index;
        if let Some(position) =
            index.and_then(|index| self.visible.iter().position(|i| *i == index))
        {
            self.scroll_handle.scroll_to_item(position);
        }
        cx.notify();
    }

    fn step(&mut self, forward: bool, cx: &mut Context<Self>) {
        let enabled = self.enabled_visible();
        if enabled.is_empty() {
            return;
        }
        let current = self
            .highlighted
            .and_then(|highlighted| enabled.iter().position(|index| *index == highlighted));
        let next = match (current, forward) {
            (Some(position), true) => (position + 1) % enabled.len(),
            (Some(position), false) => (position + enabled.len() - 1) % enabled.len(),
            (None, true) => 0,
            (None, false) => enabled.len() - 1,
        };
        self.highlight(Some(enabled[next]), cx);
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.step(true, cx);
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.step(false, cx);
    }

    fn select_first(&mut self, _: &SelectFirst, _: &mut Window, cx: &mut Context<Self>) {
        let first = self.first_enabled();
        self.highlight(first, cx);
    }

    fn select_last(&mut self, _: &SelectLast, _: &mut Window, cx: &mut Context<Self>) {
        let last = self.enabled_visible().last().copied();
        self.highlight(last, cx);
    }

    fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.highlighted {
            self.pick(index, window, cx);
        }
    }

    pub fn pick(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(entry) = self.entries.get(index) else {
            return;
        };
        if entry.disabled {
            return;
        }
        cx.emit(DismissEvent);
        let on_pick = self.on_pick.clone();
        window.defer(cx, move |window, cx| on_pick(index, window, cx));
    }
}

impl Render for SelectMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let accent = theme.colors.accent_primary;
        let hover_background = accent.opacity(theme.alpha.line);
        let selected_background = accent.opacity(theme.alpha.faint);
        let no_matches = cx.t().ui_no_matches();
        let rows = self
            .visible
            .iter()
            .map(|index| {
                let index = *index;
                let entry = &self.entries[index];
                let selected = self.selected == Some(index);
                let highlighted = self.highlighted == Some(index);
                let color = if entry.disabled {
                    theme.colors.text_muted.opacity(theme.alpha.glass)
                } else if selected || highlighted {
                    accent
                } else {
                    theme.colors.text_primary
                };
                let group = SharedString::from(format!("yp-select-item-{index}"));
                h_flex()
                    .id(("yp-select-item", index))
                    .group(group.clone())
                    .w_full()
                    .flex_none()
                    .gap(px(6.))
                    .px(Sizes::MENU_ITEM_PADDING_H)
                    .py(Sizes::MENU_ITEM_PADDING_V)
                    .text_size(sizes.font_sm)
                    .line_height(relative(LINE_HEIGHT_TIGHT))
                    .text_color(color)
                    .font_weight(if selected {
                        FontWeight::BOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .when(selected && !highlighted, |this| {
                        this.bg(selected_background)
                    })
                    .when(highlighted, |this| this.bg(hover_background))
                    .when(entry.disabled, |this| this.cursor_not_allowed())
                    .when(!entry.disabled, |this| {
                        this.cursor_pointer()
                            .hover(move |style| style.bg(hover_background).text_color(accent))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.pick(index, window, cx);
                            }))
                    })
                    .child(div().flex_none().w(px(14.)).when(selected, |this| {
                        this.child(
                            Icon::new(IconName::Check)
                                .size(IconSize::XSmall)
                                .color(Color::Accent),
                        )
                    }))
                    .when_some(entry.icon, |this, icon| {
                        let icon = Icon::new(icon).size(IconSize::Small).color(
                            if selected || (highlighted && !entry.disabled) {
                                Color::Accent
                            } else {
                                Color::Secondary
                            },
                        );
                        this.child(if entry.disabled {
                            icon
                        } else {
                            icon.group_hover_color(group, Color::Accent)
                        })
                    })
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(Text::new(entry.label.clone())),
                    )
            })
            .collect::<Vec<_>>();
        let is_empty = rows.is_empty();
        menu_container(window, cx)
            .key_context(MENU_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .on_action(cx.listener(|this, _: &input::Down, _, cx| this.step(true, cx)))
            .on_action(cx.listener(|this, _: &input::Up, _, cx| this.step(false, cx)))
            .map(|this| match self.fixed_width {
                Some(width) => this.w(width),
                None => this.min_w(self.min_width).max_w(self.max_width),
            })
            .max_h(self.max_height)
            .when_some(self.search.as_ref(), |this, search| {
                this.child(
                    div().flex_none().px(px(6.)).pb(px(4.)).child(
                        Input::new(search).size(InputSize::Small).prefix(
                            Icon::new(IconName::Search)
                                .size(IconSize::Small)
                                .color(Color::Muted),
                        ),
                    ),
                )
            })
            .child(
                div()
                    .id("yp-select-items")
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .children(rows)
                    .when(is_empty, |this| {
                        this.child(
                            v_flex()
                                .px(Sizes::MENU_ITEM_PADDING_H)
                                .py(Sizes::MENU_ITEM_PADDING_V)
                                .text_size(sizes.font_sm)
                                .text_color(theme.colors.text_muted)
                                .child(Text::new(no_matches)),
                        )
                    })
                    .overflow_y_scrollbar(&self.scroll_handle),
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenuPlacement {
    pub position: Point<Pixels>,
    pub anchor: Anchor,
    pub offset: Point<Pixels>,
    pub min_width: Pixels,
    pub max_width: Pixels,
    pub max_height: Pixels,
}

pub fn place_menu(trigger: Bounds<Pixels>, viewport: Size<Pixels>) -> MenuPlacement {
    let margin = Sizes::WINDOW_MARGIN;
    let preferred = Sizes::SELECT_MENU_MAX_HEIGHT;
    let zero = px(0.);
    let space_below = viewport.height - trigger.bottom() - margin;
    let space_above = trigger.top() - margin;
    let open_up = space_below < space_above && space_below < preferred;
    let room_vertical = if open_up { space_above } else { space_below }.max(zero);
    let max_height = room_vertical
        .max(Sizes::SELECT_MENU_MIN_HEIGHT)
        .min(preferred);
    let room_right = viewport.width - trigger.left() - margin;
    let room_left = trigger.right() - margin;
    let align_right = room_left > room_right;
    let room_horizontal = if align_right { room_left } else { room_right }.max(zero);
    let max_width = room_horizontal
        .min(trigger.size.width.max(Sizes::MENU_MAX_WIDTH))
        .max(zero);
    let min_width = trigger.size.width.min(max_width);
    let x = if align_right {
        trigger.right()
    } else {
        trigger.left()
    };
    let (y, anchor, gap) = match (open_up, align_right) {
        (true, true) => (trigger.top(), Anchor::BottomRight, -Sizes::MENU_GAP),
        (true, false) => (trigger.top(), Anchor::BottomLeft, -Sizes::MENU_GAP),
        (false, true) => (trigger.bottom(), Anchor::TopRight, Sizes::MENU_GAP),
        (false, false) => (trigger.bottom(), Anchor::TopLeft, Sizes::MENU_GAP),
    };
    MenuPlacement {
        position: point(x, y),
        anchor,
        offset: point(zero, gap),
        min_width,
        max_width,
        max_height,
    }
}

pub struct Select<T: Clone + 'static> {
    id: ElementId,
    selected_label: SharedString,
    items: Vec<SelectItem<T>>,
    selected_index: Option<usize>,
    on_select: Option<ValueHandler<T>>,
    icon: Option<IconName>,
    trigger_icon: Option<IconName>,
    highlight: bool,
    min_width: Option<Pixels>,
    max_width: Option<Pixels>,
    menu_matches_trigger_width: bool,
    searchable: bool,
    disabled: bool,
    full_width: bool,
    tooltip: Option<SharedString>,
}

crate::impl_into_element!([T: Clone + 'static] Select<T>);

impl<T: Clone + 'static> Select<T> {
    pub fn new(id: impl Into<ElementId>, selected_label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            selected_label: selected_label.into(),
            items: Vec::new(),
            selected_index: None,
            on_select: None,
            icon: None,
            trigger_icon: None,
            highlight: false,
            min_width: None,
            max_width: None,
            menu_matches_trigger_width: false,
            searchable: false,
            disabled: false,
            full_width: false,
            tooltip: None,
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

    pub fn on_select(mut self, handler: impl Fn(&T, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn trigger_icon(mut self, icon: IconName) -> Self {
        self.trigger_icon = Some(icon);
        self
    }

    pub fn highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn menu_matches_trigger_width(mut self, matches: bool) -> Self {
        self.menu_matches_trigger_width = matches;
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

impl<T: Clone + 'static> RenderOnce for Select<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let entries = self
            .items
            .iter()
            .map(|item| SelectMenuEntry {
                label: item.label.clone(),
                icon: item.icon,
                disabled: item.disabled,
            })
            .collect();
        let values: Vec<T> = self.items.into_iter().map(|item| item.value).collect();
        let on_pick = self.on_select.map(|handler| -> PickHandler {
            Rc::new(move |index, window, cx| {
                if let Some(value) = values.get(index) {
                    handler(value, window, cx);
                }
            })
        });
        render_select(
            SelectParts {
                id: self.id,
                selected_label: self.selected_label,
                entries,
                selected_index: self.selected_index,
                on_pick,
                icon: self.icon,
                trigger_icon: self.trigger_icon,
                highlight: self.highlight,
                min_width: self.min_width,
                max_width: self.max_width,
                menu_matches_trigger_width: self.menu_matches_trigger_width,
                searchable: self.searchable,
                disabled: self.disabled,
                full_width: self.full_width,
                tooltip: self.tooltip,
            },
            window,
            cx,
        )
    }
}

struct SelectParts {
    id: ElementId,
    selected_label: SharedString,
    entries: Vec<SelectMenuEntry>,
    selected_index: Option<usize>,
    on_pick: Option<PickHandler>,
    icon: Option<IconName>,
    trigger_icon: Option<IconName>,
    highlight: bool,
    min_width: Option<Pixels>,
    max_width: Option<Pixels>,
    menu_matches_trigger_width: bool,
    searchable: bool,
    disabled: bool,
    full_width: bool,
    tooltip: Option<SharedString>,
}

fn render_select(select: SelectParts, window: &mut Window, cx: &mut App) -> AnyElement {
    let theme = cx.theme().clone();
    let sizes = window.sizes(cx);
    let state = window.use_keyed_state(select.id.clone(), cx, |_, _| {
        PopoverState::<SelectMenu>::default()
    });
    let (menu, trigger_bounds) = {
        let state = state.read(cx);
        (state.menu.clone(), state.trigger_bounds)
    };
    let open = menu.is_some();
    let enabled = !select.disabled && select.on_pick.is_some() && !select.entries.is_empty();
    let highlight = select.highlight && enabled;
    let accent = theme.colors.accent_primary;
    let selected_index = select.selected_index.or_else(|| {
        select
            .entries
            .iter()
            .position(|entry| entry.label == select.selected_label)
    });
    let placement = place_menu(trigger_bounds, window.viewport_size());
    let fixed_width = select
        .menu_matches_trigger_width
        .then_some(trigger_bounds.size.width);
    if let Some(menu) = &menu {
        menu.update(cx, |menu, _| {
            menu.set_constraints(
                placement.min_width,
                placement.max_width,
                placement.max_height,
                fixed_width,
            );
        });
    }

    let entries = select.entries;
    let on_pick = select.on_pick;
    let searchable = select.searchable;
    let builder: SelectMenuBuilder = Rc::new(move |window, cx| {
        let entries = entries.clone();
        let on_pick = on_pick.clone();
        Some(cx.new(|cx| {
            SelectMenu::new(
                entries,
                selected_index,
                searchable,
                move |index, window, cx| {
                    if let Some(pick) = &on_pick {
                        pick(index, window, cx);
                    }
                },
                window,
                cx,
            )
        }))
    });

    let border = if !enabled {
        theme.colors.border_light
    } else if open || highlight {
        accent.opacity(theme.alpha.vignette)
    } else {
        theme.colors.border_medium
    };
    let fill = if !enabled {
        theme.colors.input_background.opacity(theme.alpha.glass)
    } else if open {
        accent.opacity(theme.alpha.subtle)
    } else {
        theme.colors.input_background
    };
    let icon_color = if !enabled {
        Color::Muted
    } else if highlight || open {
        Color::Accent
    } else {
        Color::Secondary
    };
    let text_color = if enabled {
        theme.colors.text_primary
    } else {
        theme.colors.text_muted
    };
    let hover_fill = accent.opacity(theme.alpha.subtle);
    let group = SharedString::from(format!("yp-select-{}", select.id));
    let stretch_label = select.full_width || select.min_width.is_some();

    let bounds_state = state.clone();
    let click_state = state.clone();
    let dismiss_state = state.downgrade();
    let trigger = match select.trigger_icon {
        Some(icon) => div()
            .id("yp-select-trigger")
            .group(group.clone())
            .relative()
            .flex_none()
            .p(px(5.))
            .rounded(Sizes::CONTROL_RADIUS)
            .border_1()
            .border_color(if open {
                accent.opacity(theme.alpha.vignette)
            } else {
                theme.colors.transparent
            })
            .when(open, |this| this.bg(hover_fill))
            .when(enabled, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.bg(hover_fill))
            })
            .child(
                Icon::new(icon)
                    .size(IconSize::Custom(px(18.)))
                    .color(if open { Color::Accent } else { icon_color })
                    .group_hover_color(group, Color::Accent),
            ),
        None => {
            let arrow = svg()
                .path(select.icon.unwrap_or(IconName::ArrowDropDown).path())
                .size(px(18.))
                .flex_none()
                .text_color(icon_color.color(cx))
                .when(enabled, |this| {
                    this.group_hover(group.clone(), move |style| style.text_color(accent))
                })
                .when(open, |this| {
                    this.with_transformation(Transformation::rotate(percentage(0.5)))
                });
            h_flex()
                .id("yp-select-trigger")
                .group(group)
                .relative()
                .gap(px(4.))
                .px(px(10.))
                .py(px(7.))
                .rounded(Sizes::CONTROL_RADIUS)
                .border_1()
                .border_color(border)
                .bg(fill)
                .text_size(sizes.font_sm)
                .line_height(relative(LINE_HEIGHT_TIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(text_color)
                .when_some(select.min_width, |this, width| this.min_w(width))
                .when_some(select.max_width, |this, width| this.max_w(width))
                .when(select.full_width, |this| this.w_full())
                .when(enabled, |this| {
                    this.cursor_pointer()
                        .hover(move |style| style.bg(hover_fill))
                })
                .when(!enabled, |this| this.cursor_not_allowed())
                .child(
                    v_flex()
                        .when(stretch_label, |this| this.flex_1())
                        .min_w_0()
                        .truncate()
                        .child(Text::new(select.selected_label.clone())),
                )
                .child(arrow)
        }
    };
    let trigger = trigger
        .child(bounds_tracker(move |bounds, _, cx| {
            if bounds_state.read(cx).trigger_bounds != bounds {
                bounds_state.update(cx, |state, _| state.trigger_bounds = bounds);
            }
        }))
        .when(enabled, |this| {
            this.keyboard_activate(cx).on_click(move |_, window, cx| {
                cx.stop_propagation();
                if click_state.read(cx).menu.is_some() {
                    hide_popover(&click_state.downgrade(), window, cx);
                } else {
                    show_popover(&click_state, &builder, None, window, cx);
                }
            })
        })
        .when_some(select.tooltip.filter(|_| !open), |this, tooltip| {
            this.tooltip(Tooltip::text(tooltip))
        });

    div()
        .id(select.id)
        .relative()
        .flex()
        .flex_col()
        .when(select.full_width, |this| this.w_full())
        .when(!select.full_width, |this| this.flex_none().items_start())
        .min_w_0()
        .child(trigger)
        .when_some(menu, |this, menu| {
            this.child(
                deferred(
                    anchored()
                        .position(placement.position)
                        .anchor(placement.anchor)
                        .offset(placement.offset)
                        .snap_to_window_with_margin(Sizes::WINDOW_MARGIN)
                        .child(
                            div()
                                .occlude()
                                .on_mouse_down_out(move |event, window, cx| {
                                    if !trigger_bounds.contains(&event.position)
                                        && !super::text_menu::text_menu_is_open(cx)
                                    {
                                        hide_popover(&dismiss_state, window, cx);
                                    }
                                })
                                .child(menu),
                        ),
                )
                .with_priority(1),
            )
        })
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    fn entry(label: &'static str) -> SelectMenuEntry {
        SelectMenuEntry {
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }

    #[test]
    fn filter_is_case_insensitive_and_keeps_order() {
        let entries = vec![entry("Bloom 2017"), entry("bloom 2021"), entry("Vanilla")];
        assert_eq!(filter_entries(&entries, ""), vec![0, 1, 2]);
        assert_eq!(filter_entries(&entries, " BLOOM "), vec![0, 1]);
        assert_eq!(filter_entries(&entries, "x"), Vec::<usize>::new());
    }

    #[test]
    fn menu_opens_below_and_left_aligned_when_there_is_room() {
        let trigger = Bounds::new(point(px(100.), px(100.)), size(px(120.), px(30.)));
        let placement = place_menu(trigger, size(px(1200.), px(800.)));
        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, point(px(100.), px(130.)));
        assert_eq!(placement.max_height, Sizes::SELECT_MENU_MAX_HEIGHT);
        assert_eq!(placement.min_width, px(120.));
        assert_eq!(placement.max_width, Sizes::MENU_MAX_WIDTH);
    }

    #[test]
    fn menu_flips_up_and_right_near_the_window_edges() {
        let trigger = Bounds::new(point(px(1000.), px(700.)), size(px(150.), px(30.)));
        let placement = place_menu(trigger, size(px(1200.), px(800.)));
        assert_eq!(placement.anchor, Anchor::BottomRight);
        assert_eq!(placement.position, point(px(1150.), px(700.)));
        assert_eq!(placement.offset.y, -Sizes::MENU_GAP);
    }

    #[test]
    fn menu_height_never_drops_below_the_minimum() {
        let trigger = Bounds::new(point(px(10.), px(40.)), size(px(100.), px(30.)));
        let placement = place_menu(trigger, size(px(300.), px(120.)));
        assert_eq!(placement.max_height, Sizes::SELECT_MENU_MIN_HEIGHT);
        assert!(placement.max_width <= px(300.));
    }
}

#[cfg(test)]
mod menu_tests {
    use std::cell::RefCell;

    use gpui::TestAppContext;

    use super::*;
    use crate::components::test_support::open_empty_root;

    fn menu(
        cx: &mut gpui::VisualTestContext,
        searchable: bool,
        picked: Rc<RefCell<Vec<usize>>>,
    ) -> Entity<SelectMenu> {
        let entries = vec![
            SelectMenuEntry {
                label: "Auto".into(),
                icon: None,
                disabled: false,
            },
            SelectMenuEntry {
                label: "1024".into(),
                icon: None,
                disabled: true,
            },
            SelectMenuEntry {
                label: "2048".into(),
                icon: None,
                disabled: false,
            },
            SelectMenuEntry {
                label: "4096".into(),
                icon: None,
                disabled: false,
            },
        ];
        cx.update(|window, cx| {
            cx.new(|cx| {
                SelectMenu::new(
                    entries,
                    Some(0),
                    searchable,
                    move |index, _, _| picked.borrow_mut().push(index),
                    window,
                    cx,
                )
            })
        })
    }

    #[gpui::test]
    fn arrows_wrap_around_and_skip_disabled_entries(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let menu = menu(cx, false, Rc::new(RefCell::new(Vec::new())));
        menu.update_in(cx, |menu, window, cx| {
            assert_eq!(menu.highlighted(), Some(0));
            menu.select_next(&SelectNext, window, cx);
            assert_eq!(menu.highlighted(), Some(2));
            menu.select_next(&SelectNext, window, cx);
            menu.select_next(&SelectNext, window, cx);
            assert_eq!(menu.highlighted(), Some(0));
            menu.select_previous(&SelectPrevious, window, cx);
            assert_eq!(menu.highlighted(), Some(3));
            menu.select_first(&SelectFirst, window, cx);
            assert_eq!(menu.highlighted(), Some(0));
            menu.select_last(&SelectLast, window, cx);
            assert_eq!(menu.highlighted(), Some(3));
        });
    }

    #[gpui::test]
    fn picking_dismisses_first_and_never_picks_disabled_entries(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let picked = Rc::new(RefCell::new(Vec::new()));
        let menu = menu(cx, false, picked.clone());
        let dismissed = Rc::new(RefCell::new(0));
        let counter = dismissed.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&menu, move |_, _: &DismissEvent, _| {
                *counter.borrow_mut() += 1
            })
        });
        menu.update_in(cx, |menu, window, cx| {
            menu.pick(1, window, cx);
            menu.pick(9, window, cx);
            menu.select_last(&SelectLast, window, cx);
            menu.confirm(&Confirm, window, cx);
        });
        cx.run_until_parked();
        assert_eq!(*picked.borrow(), vec![3]);
        assert_eq!(*dismissed.borrow(), 1);
    }

    #[gpui::test]
    fn typing_in_the_search_field_filters_and_moves_the_highlight(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let picked = Rc::new(RefCell::new(Vec::new()));
        let menu = menu(cx, true, picked.clone());
        let search = menu.read_with(cx, |menu, _| menu.search.clone().expect("search field"));
        search.update_in(cx, |search, window, cx| search.insert("40", window, cx));
        cx.run_until_parked();
        menu.read_with(cx, |menu, _| {
            assert_eq!(menu.visible(), &[3]);
            assert_eq!(menu.highlighted(), Some(3));
        });
        search.update_in(cx, |search, window, cx| {
            search.select_all(window, cx);
            search.insert("zzz", window, cx);
        });
        cx.run_until_parked();
        menu.update_in(cx, |menu, window, cx| {
            assert!(menu.visible().is_empty());
            assert_eq!(menu.highlighted(), None);
            menu.confirm(&Confirm, window, cx);
        });
        cx.run_until_parked();
        assert!(picked.borrow().is_empty());
    }
}
