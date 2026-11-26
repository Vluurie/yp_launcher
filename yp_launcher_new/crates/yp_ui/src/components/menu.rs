use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, Context, Anchor, DismissEvent, Div, ElementId, Entity, EventEmitter,
    FocusHandle, Focusable, FontWeight, IntoElement, ManagedView, MouseButton, Pixels, Point,
    Render, RenderOnce, ScrollHandle, SharedString, Subscription, WeakEntity, Window, anchored,
    deferred, div, point, prelude::*, px, relative,
};

use super::divider::Divider;
use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LINE_HEIGHT_TIGHT};
use super::scroll::ScrollableElement;
use super::text::Text;
use super::text_menu::text_menu_is_open;
use super::util::{KeyboardFocus, bounds_tracker};
use crate::actions::{
    Cancel, Confirm, MENU_CONTEXT, SelectFirst, SelectLast, SelectNext, SelectPrevious,
};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::ActionHandler;

pub(super) fn menu_shade(window: &Window, cx: &App) -> impl IntoElement {
    let size = window.viewport_size();
    deferred(
        anchored().position(point(px(0.), px(0.))).child(
            div()
                .w(size.width)
                .h(size.height)
                .bg(cx.theme().colors.accent_primary.opacity(0.21)),
        ),
    )
    .with_priority(1)
}

pub fn menu_container(window: &Window, cx: &App) -> Div {
    let theme = cx.theme();
    let sizes = window.sizes(cx);
    v_flex()
        .py(px(4.))
        .rounded(if theme.id == crate::ThemeId::NierNew {
            px(0.)
        } else {
            Sizes::CONTROL_RADIUS
        })
        .bg(theme.colors.background_card)
        .border_1()
        .border_color(theme.colors.border_medium)
        .elevation_2(cx)
        .font_ui(cx)
        .text_size(sizes.font_sm)
        .line_height(relative(LINE_HEIGHT_BODY))
        .text_color(theme.colors.text_primary)
}

pub struct ContextMenuEntry {
    label: SharedString,
    icon: Option<IconName>,
    icon_color: Option<Color>,
    toggle: Option<bool>,
    selected: bool,
    disabled: bool,
    shortcut: Option<SharedString>,
    handler: Option<ActionHandler>,
}

impl ContextMenuEntry {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            icon_color: None,
            toggle: None,
            selected: false,
            disabled: false,
            shortcut: None,
            handler: None,
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn icon_color(mut self, color: Color) -> Self {
        self.icon_color = Some(color);
        self
    }

    pub fn toggle(mut self, toggled: bool) -> Self {
        self.toggle = Some(toggled);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn handler(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.handler = Some(Rc::new(handler));
        self
    }
}

type CustomRender = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

enum ContextMenuItem {
    Header(SharedString),
    Separator,
    Entry(ContextMenuEntry),
    Link {
        label: SharedString,
        url: SharedString,
    },
    Custom {
        render: CustomRender,
        handler: Option<ActionHandler>,
    },
}

impl ContextMenuItem {
    fn is_selectable(&self) -> bool {
        match self {
            ContextMenuItem::Header(_) | ContextMenuItem::Separator => false,
            ContextMenuItem::Entry(entry) => !entry.disabled,
            ContextMenuItem::Link { .. } => true,
            ContextMenuItem::Custom { handler, .. } => handler.is_some(),
        }
    }
}

pub struct ContextMenu {
    items: Vec<ContextMenuItem>,
    focus_handle: FocusHandle,
    highlighted: Option<usize>,
    keep_open_on_confirm: bool,
    min_width: Option<Pixels>,
    max_width: Pixels,
    max_height: Pixels,
    scroll_handle: ScrollHandle,
}

impl EventEmitter<DismissEvent> for ContextMenu {}

impl Focusable for ContextMenu {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ContextMenu {
    pub fn build(
        window: &mut Window,
        cx: &mut App,
        build: impl FnOnce(Self, &mut Window, &mut Context<Self>) -> Self,
    ) -> Entity<Self> {
        cx.new(|cx| {
            let menu = Self {
                items: Vec::new(),
                focus_handle: cx.focus_handle(),
                highlighted: None,
                keep_open_on_confirm: false,
                min_width: None,
                max_width: Sizes::MENU_MAX_WIDTH,
                max_height: Sizes::SELECT_MENU_MAX_HEIGHT,
                scroll_handle: ScrollHandle::new(),
            };
            build(menu, window, cx)
        })
    }

    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        self.items.push(ContextMenuItem::Header(title.into()));
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuItem::Separator);
        self
    }

    pub fn entry(
        mut self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Entry(
            ContextMenuEntry::new(label).handler(handler),
        ));
        self
    }

    pub fn icon_entry(
        mut self,
        label: impl Into<SharedString>,
        icon: IconName,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Entry(
            ContextMenuEntry::new(label).icon(icon).handler(handler),
        ));
        self
    }

    pub fn toggleable_entry(
        mut self,
        label: impl Into<SharedString>,
        toggled: bool,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Entry(
            ContextMenuEntry::new(label)
                .toggle(toggled)
                .handler(handler),
        ));
        self
    }

    pub fn item(mut self, entry: ContextMenuEntry) -> Self {
        self.items.push(ContextMenuItem::Entry(entry));
        self
    }

    pub fn link(mut self, label: impl Into<SharedString>, url: impl Into<SharedString>) -> Self {
        self.items.push(ContextMenuItem::Link {
            label: label.into(),
            url: url.into(),
        });
        self
    }

    pub fn custom_entry(
        mut self,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Custom {
            render: Rc::new(render),
            handler: Some(Rc::new(handler)),
        });
        self
    }

    pub fn custom_row(
        mut self,
        render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.items.push(ContextMenuItem::Custom {
            render: Rc::new(render),
            handler: None,
        });
        self
    }

    pub fn keep_open_on_confirm(mut self, keep_open: bool) -> Self {
        self.keep_open_on_confirm = keep_open;
        self
    }

    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn max_width(mut self, width: Pixels) -> Self {
        self.max_width = width;
        self
    }

    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = height;
        self
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    fn selectable_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.is_selectable().then_some(index))
            .collect()
    }

    fn highlight(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        if self.highlighted != index {
            self.highlighted = index;
            if let Some(index) = index {
                self.scroll_handle.scroll_to_item(index);
            }
            cx.notify();
        }
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        let selectable = self.selectable_indices();
        let next = match self.highlighted {
            Some(current) => selectable
                .iter()
                .copied()
                .find(|index| *index > current)
                .or(selectable.first().copied()),
            None => selectable.first().copied(),
        };
        self.highlight(next, cx);
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        let selectable = self.selectable_indices();
        let previous = match self.highlighted {
            Some(current) => selectable
                .iter()
                .rev()
                .copied()
                .find(|index| *index < current)
                .or(selectable.last().copied()),
            None => selectable.last().copied(),
        };
        self.highlight(previous, cx);
    }

    fn select_first(&mut self, _: &SelectFirst, _: &mut Window, cx: &mut Context<Self>) {
        let first = self.selectable_indices().first().copied();
        self.highlight(first, cx);
    }

    fn select_last(&mut self, _: &SelectLast, _: &mut Window, cx: &mut Context<Self>) {
        let last = self.selectable_indices().last().copied();
        self.highlight(last, cx);
    }

    fn cancel(&mut self, _: &Cancel, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.highlighted {
            self.activate(index, window, cx);
        }
    }

    pub fn activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(index) else {
            return;
        };
        if !item.is_selectable() {
            return;
        }
        let handler: Option<ActionHandler> = match item {
            ContextMenuItem::Entry(entry) => entry.handler.clone(),
            ContextMenuItem::Custom { handler, .. } => handler.clone(),
            ContextMenuItem::Link { url, .. } => {
                let url = url.clone();
                Some(Rc::new(move |_: &mut Window, cx: &mut App| {
                    super::util::open_link(&url, cx)
                }))
            }
            ContextMenuItem::Header(_) | ContextMenuItem::Separator => None,
        };
        if !self.keep_open_on_confirm {
            cx.emit(DismissEvent);
        }
        if let Some(handler) = handler {
            window.defer(cx, move |window, cx| handler(window, cx));
        }
    }
}

impl Render for ContextMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let accent = theme.colors.accent_primary;
        let hover_background = accent.opacity(theme.alpha.line);
        let selected_background = accent.opacity(theme.alpha.faint);
        let highlighted = self.highlighted;
        let max_height = self
            .max_height
            .min(sizes.window_height - Sizes::WINDOW_MARGIN * 2.)
            .max(Sizes::SELECT_MENU_MIN_HEIGHT);
        let rows = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| match item {
                ContextMenuItem::Header(title) => v_flex()
                    .px(Sizes::MENU_ITEM_PADDING_H)
                    .py(px(4.))
                    .text_size(sizes.font_xs)
                    .line_height(relative(LINE_HEIGHT_TIGHT))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.colors.text_muted)
                    .child(Text::new(title.to_uppercase()))
                    .into_any_element(),
                ContextMenuItem::Separator => div()
                    .py(px(4.))
                    .child(Divider::horizontal())
                    .into_any_element(),
                ContextMenuItem::Entry(entry) => {
                    let active = entry.selected || entry.toggle == Some(true);
                    let is_highlighted = highlighted == Some(index);
                    let color = if entry.disabled {
                        Color::Disabled
                    } else if active {
                        Color::Accent
                    } else {
                        Color::Default
                    };
                    let leading = match (entry.toggle, entry.selected, entry.icon) {
                        (Some(true), _, _) | (None, true, None) => {
                            Some((IconName::Check, Color::Accent))
                        }
                        (_, _, Some(icon)) => Some((icon, entry.icon_color.unwrap_or(color))),
                        _ => None,
                    };
                    let follows_hover = !entry.disabled && entry.icon_color.is_none();
                    let group = SharedString::from(format!("yp-menu-item-{index}"));
                    h_flex()
                        .id(("yp-menu-item", index))
                        .role(if entry.toggle.is_some() {
                            gpui::Role::MenuItemCheckBox
                        } else {
                            gpui::Role::MenuItem
                        })
                        .aria_label(entry.label.clone())
                        .when_some(entry.toggle, |el, on| {
                            el.aria_toggled(if on {
                                gpui::Toggled::True
                            } else {
                                gpui::Toggled::False
                            })
                        })
                        .when(is_highlighted, |el| el.aria_active_descendant())
                        .group(group.clone())
                        .w_full()
                        .gap(px(6.))
                        .px(Sizes::MENU_ITEM_PADDING_H)
                        .py(Sizes::MENU_ITEM_PADDING_V)
                        .text_color(color.color(cx))
                        .font_weight(if active {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .when(active && !is_highlighted, |this| {
                            this.bg(selected_background)
                        })
                        .when(is_highlighted, |this| {
                            this.bg(hover_background).text_color(accent)
                        })
                        .when(entry.disabled, |this| this.cursor_not_allowed())
                        .when(!entry.disabled, |this| {
                            this.cursor_pointer()
                                .hover(move |style| style.bg(hover_background).text_color(accent))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.activate(index, window, cx);
                                }))
                        })
                        .child(div().flex_none().w(px(14.)).children(leading.map(
                            |(icon, color)| {
                                let icon = Icon::new(icon).size(IconSize::Custom(px(14.))).color(
                                    if is_highlighted && follows_hover {
                                        Color::Accent
                                    } else {
                                        color
                                    },
                                );
                                if follows_hover {
                                    icon.group_hover_color(group, Color::Accent)
                                } else {
                                    icon
                                }
                            },
                        )))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .when(theme.id != crate::ThemeId::NierNew, |el| el.truncate())
                                .child(Text::new(entry.label.clone())),
                        )
                        .when_some(entry.shortcut.clone(), |this, shortcut| {
                            this.child(
                                h_flex()
                                    .flex_none()
                                    .pl(Sizes::MENU_ITEM_PADDING_H)
                                    .text_size(sizes.font_xs)
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(Color::Muted.color(cx))
                                    .child(Text::new(shortcut)),
                            )
                        })
                        .into_any_element()
                }
                ContextMenuItem::Link { label, .. } => {
                    let is_highlighted = highlighted == Some(index);
                    h_flex()
                        .id(("yp-menu-item", index))
                        .w_full()
                        .gap(px(6.))
                        .px(Sizes::MENU_ITEM_PADDING_H)
                        .py(Sizes::MENU_ITEM_PADDING_V)
                        .cursor_pointer()
                        .text_color(theme.colors.text_primary)
                        .when(is_highlighted, |this| {
                            this.bg(hover_background).text_color(accent)
                        })
                        .hover(move |style| style.bg(hover_background).text_color(accent))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.activate(index, window, cx);
                        }))
                        .child(div().flex_none().w(px(14.)))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .child(Text::new(label.clone())),
                        )
                        .child(
                            Icon::new(IconName::ArrowOutward)
                                .size(IconSize::XSmall)
                                .color(Color::Muted),
                        )
                        .into_any_element()
                }
                ContextMenuItem::Custom { render, handler } => {
                    let is_highlighted = highlighted == Some(index);
                    let interactive = handler.is_some();
                    div()
                        .id(("yp-menu-item", index))
                        .w_full()
                        .px(Sizes::MENU_ITEM_PADDING_H)
                        .py(Sizes::MENU_ITEM_PADDING_V)
                        .when(is_highlighted, |this| this.bg(hover_background))
                        .when(interactive, |this| {
                            this.cursor_pointer()
                                .hover(move |style| style.bg(hover_background))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.activate(index, window, cx);
                                }))
                        })
                        .child(render(window, cx))
                        .into_any_element()
                }
            })
            .collect::<Vec<_>>();
        menu_container(window, cx)
            .key_context(MENU_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_first))
            .on_action(cx.listener(Self::select_last))
            .when_some(self.min_width, |this, width| this.min_w(width))
            .max_w(self.max_width)
            .child(
                div()
                    .id("yp-context-menu-items")
                    .role(gpui::Role::Menu)
                    .max_h(max_height)
                    .flex()
                    .flex_col()
                    .children(rows)
                    .overflow_y_scrollbar(&self.scroll_handle),
            )
    }
}

pub(crate) struct PopoverState<M> {
    pub(crate) menu: Option<Entity<M>>,
    previous_focus: Option<FocusHandle>,
    pub(crate) trigger_bounds: Bounds<Pixels>,
    position: Option<Point<Pixels>>,
    subscription: Option<Subscription>,
}

impl<M> PopoverState<M> {
    pub(crate) fn position(&self) -> Option<Point<Pixels>> {
        self.position
    }
}

impl<M> Default for PopoverState<M> {
    fn default() -> Self {
        Self {
            menu: None,
            previous_focus: None,
            trigger_bounds: Bounds::default(),
            position: None,
            subscription: None,
        }
    }
}

type MenuBuilder<M> = Rc<dyn Fn(&mut Window, &mut App) -> Option<Entity<M>>>;

pub(crate) fn show_popover<M: ManagedView>(
    state: &Entity<PopoverState<M>>,
    builder: &MenuBuilder<M>,
    position: Option<Point<Pixels>>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(menu) = builder(window, cx) else {
        return;
    };
    let previous_focus = window.focused(cx);
    let weak = state.downgrade();
    let subscription = window.subscribe(&menu, cx, move |_, _: &DismissEvent, window, cx| {
        hide_popover(&weak, window, cx);
    });
    window.focus(&menu.focus_handle(cx), cx);
    state.update(cx, |state, cx| {
        state.menu = Some(menu);
        state.previous_focus = previous_focus;
        state.position = position;
        state.subscription = Some(subscription);
        cx.notify();
    });
}

pub(crate) fn hide_popover<M: ManagedView>(
    state: &WeakEntity<PopoverState<M>>,
    window: &mut Window,
    cx: &mut App,
) {
    let closed = state
        .update(cx, |state, cx| {
            let menu = state.menu.take()?;
            let subscription = state.subscription.take();
            cx.notify();
            Some((menu, state.previous_focus.take(), subscription))
        })
        .ok()
        .flatten();
    let Some((menu, previous_focus, subscription)) = closed else {
        return;
    };
    let menu_had_focus = menu.focus_handle(cx).contains_focused(window, cx);
    if menu_had_focus && let Some(previous) = previous_focus {
        window.focus(&previous, cx);
    }
    window.defer(cx, move |_, _| drop(subscription));
}

struct PopoverHandleState<M> {
    state: Entity<PopoverState<M>>,
    builder: MenuBuilder<M>,
}

pub struct PopoverMenuHandle<M>(Rc<RefCell<Option<PopoverHandleState<M>>>>);

impl<M> Clone for PopoverMenuHandle<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<M> Default for PopoverMenuHandle<M> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }
}

impl<M: ManagedView> PopoverMenuHandle<M> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_deployed(&self, cx: &App) -> bool {
        self.0
            .borrow()
            .as_ref()
            .is_some_and(|handle| handle.state.read(cx).menu.is_some())
    }

    pub fn show(&self, window: &mut Window, cx: &mut App) {
        let target = self
            .0
            .borrow()
            .as_ref()
            .map(|handle| (handle.state.clone(), handle.builder.clone()));
        if let Some((state, builder)) = target
            && state.read(cx).menu.is_none()
        {
            show_popover(&state, &builder, None, window, cx);
        }
    }

    pub fn hide(&self, window: &mut Window, cx: &mut App) {
        let state = self.0.borrow().as_ref().map(|handle| handle.state.clone());
        if let Some(state) = state {
            hide_popover(&state.downgrade(), window, cx);
        }
    }

    pub fn toggle(&self, window: &mut Window, cx: &mut App) {
        if self.is_deployed(cx) {
            self.hide(window, cx);
        } else {
            self.show(window, cx);
        }
    }
}

type TriggerBuilder = Box<dyn FnOnce(bool, &mut Window, &mut App) -> AnyElement>;

fn trigger_slot(open: bool, full_width: bool, trigger: Option<AnyElement>) -> impl IntoElement {
    div()
        .id(if open {
            "yp-popover-trigger-open"
        } else {
            "yp-popover-trigger-closed"
        })
        .flex()
        .flex_col()
        .when(full_width, |this| this.w_full())
        .when(!full_width, |this| this.items_start())
        .children(trigger)
}

pub struct PopoverMenu<M: ManagedView> {
    accessible_label: Option<SharedString>,
    id: ElementId,
    trigger: Option<TriggerBuilder>,
    menu: Option<MenuBuilder<M>>,
    anchor: Anchor,
    attach: Anchor,
    offset: Point<Pixels>,
    handle: Option<PopoverMenuHandle<M>>,
    full_width: bool,
    disabled: bool,
}

crate::impl_into_element!([M: ManagedView] PopoverMenu<M>);

impl<M: ManagedView> PopoverMenu<M> {
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            accessible_label: None,
            menu: None,
            anchor: Anchor::TopLeft,
            attach: Anchor::BottomLeft,
            offset: point(px(0.), Sizes::MENU_GAP),
            handle: None,
            full_width: false,
            disabled: false,
        }
    }

    pub fn trigger(mut self, trigger: impl IntoElement + 'static) -> Self {
        self.trigger = Some(Box::new(move |_, _, _| trigger.into_any_element()));
        self
    }

    pub fn trigger_with(
        mut self,
        trigger: impl FnOnce(bool, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger = Some(Box::new(trigger));
        self
    }

    pub fn menu(
        mut self,
        menu: impl Fn(&mut Window, &mut App) -> Option<Entity<M>> + 'static,
    ) -> Self {
        self.menu = Some(Rc::new(menu));
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn attach(mut self, attach: Anchor) -> Self {
        self.attach = attach;
        self
    }

    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_handle(mut self, handle: PopoverMenuHandle<M>) -> Self {
        self.handle = Some(handle);
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<M: ManagedView> RenderOnce for PopoverMenu<M> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state =
            window.use_keyed_state(self.id.clone(), cx, |_, _| PopoverState::<M>::default());
        let builder: MenuBuilder<M> = self.menu.unwrap_or_else(|| Rc::new(|_, _| None));
        if let Some(handle) = &self.handle {
            *handle.0.borrow_mut() = Some(PopoverHandleState {
                state: state.clone(),
                builder: builder.clone(),
            });
        }
        let (menu, trigger_bounds) = {
            let state = state.read(cx);
            (state.menu.clone(), state.trigger_bounds)
        };
        let open = menu.is_some();
        let trigger = self.trigger.map(|trigger| trigger(open, window, cx));
        let bounds_state = state.clone();
        let click_state = state.clone();
        let shield_state = state.downgrade();
        let dismiss_state = state.downgrade();
        let disabled = self.disabled;
        let attach_point = trigger_bounds.corner(self.attach);
        div()
            .id(self.id)
            .relative()
            .flex()
            .flex_col()
            .when(self.full_width, |this| this.w_full())
            .when(!self.full_width, |this| this.flex_none().items_start())
            .child(
                div()
                    .id("yp-popover-trigger")
                    .role(gpui::Role::Button)
                    .aria_expanded(open)
                    .when(disabled, |el| el.aria_description(cx.t().mod_disabled()))
                    .when_some(self.accessible_label, |el, label| el.aria_label(label))
                    .relative()
                    .flex()
                    .flex_col()
                    .when(self.full_width, |this| this.w_full())
                    .when(!self.full_width, |this| this.items_start())
                    .child(trigger_slot(open, self.full_width, trigger))
                    .child(bounds_tracker(move |bounds, _, cx| {
                        if bounds_state.read(cx).trigger_bounds != bounds {
                            bounds_state.update(cx, |state, _| state.trigger_bounds = bounds);
                        }
                    }))
                    .when(!disabled, |this| {
                        this.rounded(Sizes::CONTROL_RADIUS)
                            .keyboard_activate(cx)
                            .on_disclosure_toggle(open, move |_, window, cx| {
                                cx.stop_propagation();
                                if click_state.read(cx).menu.is_some() {
                                    hide_popover(&click_state.downgrade(), window, cx);
                                } else {
                                    show_popover(&click_state, &builder, None, window, cx);
                                }
                            })
                    })
                    .when(open, |this| {
                        this.child(
                            div()
                                .id("yp-popover-trigger-shield")
                                .absolute()
                                .inset_0()
                                .occlude()
                                .cursor_pointer()
                                .on_click(move |_, window, cx| {
                                    cx.stop_propagation();
                                    hide_popover(&shield_state, window, cx);
                                }),
                        )
                    }),
            )
            .when(open && cx.theme().id == crate::ThemeId::NierNew, |el| {
                el.child(menu_shade(window, cx))
            })
            .when_some(menu, |this, menu| {
                this.child(
                    deferred(
                        anchored()
                            .position(attach_point)
                            .anchor(self.anchor)
                            .offset(self.offset)
                            .snap_to_window_with_margin(Sizes::WINDOW_MARGIN)
                            .child(
                                div()
                                    .occlude()
                                    .on_mouse_down_out(move |event, window, cx| {
                                        if !trigger_bounds.contains(&event.position)
                                            && !text_menu_is_open(cx)
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
    }
}

pub struct RightClickMenu<M: ManagedView> {
    id: ElementId,
    trigger: Option<TriggerBuilder>,
    menu: Option<MenuBuilder<M>>,
}

crate::impl_into_element!([M: ManagedView] RightClickMenu<M>);

pub fn right_click_menu<M: ManagedView>(id: impl Into<ElementId>) -> RightClickMenu<M> {
    RightClickMenu {
        id: id.into(),
        trigger: None,
        menu: None,
    }
}

impl<M: ManagedView> RightClickMenu<M> {
    pub fn trigger(mut self, trigger: impl IntoElement + 'static) -> Self {
        self.trigger = Some(Box::new(move |_, _, _| trigger.into_any_element()));
        self
    }

    pub fn trigger_with(
        mut self,
        trigger: impl FnOnce(bool, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger = Some(Box::new(trigger));
        self
    }

    pub fn menu(mut self, menu: impl Fn(&mut Window, &mut App) -> Entity<M> + 'static) -> Self {
        self.menu = Some(Rc::new(move |window, cx| Some(menu(window, cx))));
        self
    }
}

impl<M: ManagedView> RenderOnce for RightClickMenu<M> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state =
            window.use_keyed_state(self.id.clone(), cx, |_, _| PopoverState::<M>::default());
        let builder: MenuBuilder<M> = self.menu.unwrap_or_else(|| Rc::new(|_, _| None));
        let (menu, position) = {
            let state = state.read(cx);
            (state.menu.clone(), state.position)
        };
        let trigger = self
            .trigger
            .map(|trigger| trigger(menu.is_some(), window, cx));
        let open_state = state.clone();
        let dismiss_state = state.downgrade();
        let open = menu.is_some();
        div()
            .id(self.id)
            .relative()
            .flex()
            .flex_col()
            .items_start()
            .child(trigger_slot(open, false, trigger))
            .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                cx.stop_propagation();
                hide_popover(&open_state.downgrade(), window, cx);
                show_popover(&open_state, &builder, Some(event.position), window, cx);
            })
            .when_some(menu.zip(position), |this, (menu, position)| {
                this.child(
                    deferred(
                        anchored()
                            .position(position)
                            .anchor(Anchor::TopLeft)
                            .snap_to_window_with_margin(Sizes::WINDOW_MARGIN)
                            .child(
                                div()
                                    .occlude()
                                    .on_mouse_down_out(move |_, window, cx| {
                                        hide_popover(&dismiss_state, window, cx);
                                    })
                                    .child(menu),
                            ),
                    )
                    .with_priority(2),
                )
            })
    }
}

type PopoverContentBuilder = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

pub struct PopoverContent {
    focus_handle: FocusHandle,
    content: PopoverContentBuilder,
    padded: bool,
}

impl EventEmitter<DismissEvent> for PopoverContent {}

impl Focusable for PopoverContent {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopoverContent {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let content = (self.content)(window, cx);
        menu_container(window, cx)
            .block()
            .key_context(MENU_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|_, _: &Cancel, _, cx| cx.emit(DismissEvent)))
            .max_w(Sizes::MENU_MAX_WIDTH)
            .when(self.padded, |this| {
                this.px(sizes.padding_md).py(sizes.padding_sm)
            })
            .child(content)
    }
}

pub struct Popover {
    id: ElementId,
    trigger: Option<TriggerBuilder>,
    content: Option<PopoverContentBuilder>,
    anchor: Anchor,
    attach: Anchor,
    offset: Point<Pixels>,
    padded: bool,
    handle: Option<PopoverMenuHandle<PopoverContent>>,
}

crate::impl_into_element!(Popover);

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            anchor: Anchor::TopLeft,
            attach: Anchor::BottomLeft,
            offset: point(px(0.), Sizes::MENU_GAP),
            padded: true,
            handle: None,
        }
    }

    pub fn trigger(mut self, trigger: impl IntoElement + 'static) -> Self {
        self.trigger = Some(Box::new(move |_, _, _| trigger.into_any_element()));
        self
    }

    pub fn trigger_with(
        mut self,
        trigger: impl FnOnce(bool, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.trigger = Some(Box::new(trigger));
        self
    }

    pub fn content(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(Rc::new(content));
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn attach(mut self, attach: Anchor) -> Self {
        self.attach = attach;
        self
    }

    pub fn offset(mut self, offset: Point<Pixels>) -> Self {
        self.offset = offset;
        self
    }

    pub fn padded(mut self, padded: bool) -> Self {
        self.padded = padded;
        self
    }

    pub fn with_handle(mut self, handle: PopoverMenuHandle<PopoverContent>) -> Self {
        self.handle = Some(handle);
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let content = self.content;
        let padded = self.padded;
        let mut menu = PopoverMenu::<PopoverContent>::new(self.id)
            .anchor(self.anchor)
            .attach(self.attach)
            .offset(self.offset)
            .menu(move |_, cx| {
                let content = content.clone()?;
                Some(cx.new(|cx| PopoverContent {
                    focus_handle: cx.focus_handle(),
                    content,
                    padded,
                }))
            });
        menu.trigger = self.trigger;
        match self.handle {
            Some(handle) => menu.with_handle(handle),
            None => menu,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use gpui::TestAppContext;

    use super::*;
    use crate::components::test_support::{open_empty_root, open_root};

    fn sample_menu(
        cx: &mut gpui::VisualTestContext,
        keep_open: bool,
        picked: Rc<Cell<usize>>,
    ) -> Entity<ContextMenu> {
        cx.update(|window, cx| {
            ContextMenu::build(window, cx, |menu, _, _| {
                let first = picked.clone();
                let last = picked.clone();
                menu.header("Header")
                    .entry("First", move |_, _| first.set(first.get() + 1))
                    .separator()
                    .item(ContextMenuEntry::new("Disabled").disabled(true))
                    .entry("Last", move |_, _| last.set(last.get() + 100))
                    .keep_open_on_confirm(keep_open)
            })
        })
    }

    #[gpui::test]
    fn keyboard_navigation_skips_headers_separators_and_disabled_rows(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let menu = sample_menu(cx, false, Rc::new(Cell::new(0)));
        menu.update_in(cx, |menu, window, cx| {
            assert_eq!(menu.item_count(), 5);
            menu.select_next(&SelectNext, window, cx);
            assert_eq!(menu.highlighted(), Some(1));
            menu.select_next(&SelectNext, window, cx);
            assert_eq!(menu.highlighted(), Some(4));
            menu.select_next(&SelectNext, window, cx);
            assert_eq!(menu.highlighted(), Some(1));
            menu.select_previous(&SelectPrevious, window, cx);
            assert_eq!(menu.highlighted(), Some(4));
            menu.select_first(&SelectFirst, window, cx);
            assert_eq!(menu.highlighted(), Some(1));
            menu.select_last(&SelectLast, window, cx);
            assert_eq!(menu.highlighted(), Some(4));
        });
    }

    #[gpui::test]
    fn confirm_runs_the_handler_after_dismissing(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let picked = Rc::new(Cell::new(0));
        let menu = sample_menu(cx, false, picked.clone());
        let dismissed = Rc::new(Cell::new(0));
        let counter = dismissed.clone();
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&menu, move |_, _: &DismissEvent, _| {
                counter.set(counter.get() + 1)
            })
        });
        menu.update_in(cx, |menu, window, cx| {
            menu.confirm(&Confirm, window, cx);
            menu.activate(3, window, cx);
            menu.activate(0, window, cx);
            menu.select_last(&SelectLast, window, cx);
            menu.confirm(&Confirm, window, cx);
        });
        cx.run_until_parked();
        assert_eq!(picked.get(), 100);
        assert_eq!(dismissed.get(), 1);
    }

    #[gpui::test]
    fn keep_open_menus_stay_open_after_a_pick(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let picked = Rc::new(Cell::new(0));
        let menu = sample_menu(cx, true, picked.clone());
        let dismissed = Rc::new(Cell::new(false));
        let flag = dismissed.clone();
        let _subscription =
            cx.update(|_, cx| cx.subscribe(&menu, move |_, _: &DismissEvent, _| flag.set(true)));
        menu.update_in(cx, |menu, window, cx| {
            menu.activate(1, window, cx);
            menu.activate(1, window, cx);
        });
        cx.run_until_parked();
        assert_eq!(picked.get(), 2);
        assert!(!dismissed.get());
    }

    #[gpui::test]
    fn popover_handle_shows_hides_and_escape_restores_focus(cx: &mut TestAppContext) {
        let handle = PopoverMenuHandle::<ContextMenu>::new();
        let render_handle = handle.clone();
        let (root, cx) = open_root(cx, move |_, _| {
            PopoverMenu::<ContextMenu>::new("test-popover")
                .with_handle(render_handle.clone())
                .trigger(div().child("open"))
                .menu(|window, cx| {
                    Some(ContextMenu::build(window, cx, |menu, _, _| {
                        menu.entry("Only", |_, _| {})
                    }))
                })
                .into_any_element()
        });
        cx.run_until_parked();
        let root_focus = root.read_with(cx, |root, cx| root.focus_handle(cx));
        cx.update(|window, cx| {
            assert!(!handle.is_deployed(cx));
            handle.show(window, cx);
            assert!(handle.is_deployed(cx));
            assert!(!root_focus.is_focused(window));
        });
        cx.run_until_parked();
        let menu = handle
            .0
            .borrow()
            .as_ref()
            .and_then(|handle| handle.state.read_with(cx, |state, _| state.menu.clone()))
            .expect("open menu");
        menu.read_with(cx, |menu, _| {
            assert_eq!(menu.scroll_handle.max_offset().y, px(0.));
        });
        cx.simulate_keystrokes("escape");
        cx.update(|window, cx| {
            assert!(!handle.is_deployed(cx));
            assert!(root_focus.is_focused(window));
            handle.toggle(window, cx);
            assert!(handle.is_deployed(cx));
            handle.toggle(window, cx);
            assert!(!handle.is_deployed(cx));
        });
    }
}
