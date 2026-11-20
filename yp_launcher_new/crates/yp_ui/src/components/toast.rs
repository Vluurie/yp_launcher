use std::collections::VecDeque;

use gpui::{
    Anchor, Animation, AnimationExt, App, Context, DismissEvent, ElementId, Entity, EventEmitter,
    FocusHandle, Focusable, IntoElement, Render, ScrollHandle, SharedString, Subscription, Task,
    WeakEntity, Window, div, point, prelude::*, px, relative,
};

use super::button::{ButtonSize, ButtonStyle, IconButton};
use super::button_extra::CopyButton;
use super::callout::Callout;
use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LINE_HEIGHT_TIGHT};
use super::menu::{PopoverMenu, PopoverMenuHandle, menu_container};
use super::scroll::ScrollableElement;
use super::selectable_text::SelectableText;
use super::text::Text;
use super::tooltip::Tooltip;
use crate::actions::{Cancel, TOAST_HISTORY_CONTEXT};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Severity, Sizes, easing};
use crate::traits::{ButtonCommon, Clickable};

const HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToastId(pub SharedString);

impl From<&'static str> for ToastId {
    fn from(id: &'static str) -> Self {
        ToastId(SharedString::new_static(id))
    }
}

impl From<String> for ToastId {
    fn from(id: String) -> Self {
        ToastId(SharedString::from(id))
    }
}

impl From<SharedString> for ToastId {
    fn from(id: SharedString) -> Self {
        ToastId(id)
    }
}

impl From<u64> for ToastId {
    fn from(id: u64) -> Self {
        ToastId(SharedString::from(format!("toast-{id}")))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    pub id: Option<ToastId>,
    pub message: SharedString,
    pub severity: Severity,
    pub icon: Option<IconName>,
}

impl Toast {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            message: message.into(),
            severity: Severity::Accent,
            icon: None,
        }
    }

    pub fn info(message: impl Into<SharedString>) -> Self {
        Self::new(message).severity(Severity::Info)
    }

    pub fn success(message: impl Into<SharedString>) -> Self {
        Self::new(message).severity(Severity::Success)
    }

    pub fn warning(message: impl Into<SharedString>) -> Self {
        Self::new(message).severity(Severity::Warning)
    }

    pub fn error(message: impl Into<SharedString>) -> Self {
        Self::new(message).severity(Severity::Error)
    }

    pub fn id(mut self, id: impl Into<ToastId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastEvent {
    Dismissed(ToastId),
}

struct ToastEntry {
    id: ToastId,
    toast: Toast,
    leaving: bool,
    hovered: bool,
    lifetime: Option<Task<()>>,
    grace: Option<Task<()>>,
    removal: Option<Task<()>>,
}

struct ToastRecord {
    sequence: u64,
    toast: Toast,
}

pub struct ToastStack {
    entries: Vec<ToastEntry>,
    counter: u64,
    history: VecDeque<ToastRecord>,
    history_counter: u64,
    history_view: PopoverMenuHandle<ToastHistory>,
}

impl Default for ToastStack {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter<ToastEvent> for ToastStack {}

impl ToastStack {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            counter: 0,
            history: VecDeque::new(),
            history_counter: 0,
            history_view: PopoverMenuHandle::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ids(&self) -> Vec<ToastId> {
        self.entries.iter().map(|entry| entry.id.clone()).collect()
    }

    pub fn contains(&self, id: &ToastId) -> bool {
        self.entries.iter().any(|entry| &entry.id == id)
    }

    pub fn is_leaving(&self, id: &ToastId) -> bool {
        self.entries
            .iter()
            .any(|entry| &entry.id == id && entry.leaving)
    }

    pub fn push(&mut self, toast: Toast, cx: &mut Context<Self>) -> ToastId {
        if !toast.message.trim().is_empty() {
            self.history_counter += 1;
            self.history.push_back(ToastRecord {
                sequence: self.history_counter,
                toast: toast.clone(),
            });
            if self.history.len() > HISTORY_LIMIT {
                self.history.pop_front();
            }
        }
        let id = match toast.id.clone() {
            Some(id) => id,
            None => {
                self.counter += 1;
                ToastId::from(self.counter)
            }
        };
        let lifetime = cx.theme().motion.toast_lifetime(&toast.message);
        let timer_id = id.clone();
        let task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(lifetime).await;
            this.update(cx, |this, cx| this.lifetime_elapsed(&timer_id, cx))
                .ok();
        });
        match self.entries.iter_mut().find(|entry| entry.id == id) {
            Some(entry) => {
                entry.toast = toast;
                entry.leaving = false;
                entry.lifetime = Some(task);
                entry.grace = None;
                entry.removal = None;
            }
            None => self.entries.push(ToastEntry {
                id: id.clone(),
                toast,
                leaving: false,
                hovered: false,
                lifetime: Some(task),
                grace: None,
                removal: None,
            }),
        }
        cx.notify();
        id
    }

    pub fn dismiss(&mut self, id: &ToastId, cx: &mut Context<Self>) {
        self.begin_leave(id, cx);
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        let ids = self.ids();
        for id in ids {
            self.begin_leave(&id, cx);
        }
    }

    fn lifetime_elapsed(&mut self, id: &ToastId, cx: &mut Context<Self>) {
        let hovered = self
            .entries
            .iter()
            .any(|entry| &entry.id == id && entry.hovered);
        if !hovered {
            self.begin_leave(id, cx);
        }
    }

    pub fn set_hovered(&mut self, id: &ToastId, hovered: bool, cx: &mut Context<Self>) {
        let grace_period = cx.theme().motion.toast_hover_grace;
        let grace_id = id.clone();
        let Some(entry) = self.entries.iter_mut().find(|entry| &entry.id == id) else {
            return;
        };
        if entry.hovered == hovered {
            return;
        }
        entry.hovered = hovered;
        if hovered {
            entry.grace = None;
            return;
        }
        entry.grace = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(grace_period).await;
            this.update(cx, |this, cx| this.lifetime_elapsed(&grace_id, cx))
                .ok();
        }));
    }

    fn begin_leave(&mut self, id: &ToastId, cx: &mut Context<Self>) {
        let duration = cx.theme().motion.toast;
        let removal_id = id.clone();
        let Some(entry) = self.entries.iter_mut().find(|entry| &entry.id == id) else {
            return;
        };
        if entry.leaving {
            return;
        }
        entry.leaving = true;
        entry.lifetime = None;
        entry.grace = None;
        entry.removal = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(duration).await;
            this.update(cx, |this, cx| this.remove(&removal_id, cx))
                .ok();
        }));
        cx.notify();
    }

    fn remove(&mut self, id: &ToastId, cx: &mut Context<Self>) {
        let before = self.entries.len();
        self.entries.retain(|entry| &entry.id != id);
        if self.entries.len() != before {
            cx.emit(ToastEvent::Dismissed(id.clone()));
            cx.notify();
        }
    }
}

impl Render for ToastStack {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let max_width = sizes.toast_width();
        let slide = f32::from(Sizes::TOAST_MIN_WIDTH);
        let toasts = self
            .entries
            .iter()
            .map(|entry| {
                let status = theme.status.for_severity(entry.toast.severity);
                let icon = entry
                    .toast
                    .icon
                    .unwrap_or_else(|| Callout::default_icon(entry.toast.severity));
                let leaving = entry.leaving;
                let hover_id = entry.id.clone();
                let click_id = entry.id.clone();
                let hover_border = status.fg.opacity(theme.alpha.vignette);
                let phase = if leaving { "out" } else { "in" };
                h_flex()
                    .id(ElementId::Name(entry.id.0.clone()))
                    .relative()
                    .items_start()
                    .gap(px(8.))
                    .px(px(12.))
                    .py(px(8.))
                    .max_w(max_width)
                    .rounded(Sizes::ROW_RADIUS)
                    .bg(theme.colors.background_card.opacity(theme.alpha.panel))
                    .border_1()
                    .border_color(status.border)
                    .elevation_3(cx)
                    .cursor_pointer()
                    .hover(move |style| style.border_color(hover_border))
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        this.set_hovered(&hover_id, *hovered, cx);
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.dismiss(&click_id, cx);
                    }))
                    .child(
                        div().pt(px(2.)).child(
                            Icon::new(icon)
                                .size(IconSize::Custom(px(16.)))
                                .color(Color::from(entry.toast.severity)),
                        ),
                    )
                    .child(
                        v_flex()
                            .min_w_0()
                            .text_size(sizes.font_sm)
                            .line_height(relative(LINE_HEIGHT_BODY))
                            .text_color(theme.colors.text_primary)
                            .child(Text::new(entry.toast.message.clone())),
                    )
                    .with_animation(
                        ElementId::from((ElementId::Name(entry.id.0.clone()), phase)),
                        Animation::new(theme.motion.toast),
                        move |this, delta| {
                            let progress = if leaving { 1. - delta } else { delta };
                            this.left(px(slide * (1. - easing::nier_fill(progress))))
                                .opacity(easing::nier_rise(progress))
                        },
                    )
            })
            .collect::<Vec<_>>();
        div()
            .absolute()
            .bottom(sizes.padding_lg)
            .right(sizes.padding_lg)
            .flex()
            .flex_col()
            .items_end()
            .gap(px(6.))
            .children(toasts)
            .when(!self.history.is_empty(), |this| {
                let stack = cx.entity().downgrade();
                this.child(
                    PopoverMenu::new("yp-toast-history-popover")
                        .with_handle(self.history_view.clone())
                        .anchor(Anchor::BottomRight)
                        .attach(Anchor::TopRight)
                        .offset(point(px(0.), -Sizes::MENU_GAP))
                        .trigger(
                            IconButton::new("yp-toast-history-open", IconName::ListAltOutlined)
                                .style(ButtonStyle::Transparent)
                                .size(ButtonSize::Compact)
                                .icon_color(Color::Muted)
                                .tooltip(Tooltip::text(cx.t().toast_history_title())),
                        )
                        .menu(move |_, cx| {
                            let stack = stack.upgrade()?;
                            Some(cx.new(|cx| ToastHistory::new(stack, cx)))
                        }),
                )
            })
    }
}

struct ToastHistory {
    stack: WeakEntity<ToastStack>,
    focus_handle: FocusHandle,
    scroll: ScrollHandle,
    _subscription: Subscription,
}

impl ToastHistory {
    fn new(stack: Entity<ToastStack>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&stack, |_, _, cx| cx.notify());
        Self {
            stack: stack.downgrade(),
            focus_handle: cx.focus_handle().tab_stop(false),
            scroll: ScrollHandle::new(),
            _subscription: subscription,
        }
    }
}

impl EventEmitter<DismissEvent> for ToastHistory {}

impl Focusable for ToastHistory {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ToastHistory {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let stack = self.stack.upgrade();
        let history = stack.as_ref().map(|stack| &stack.read(cx).history);
        let copied = history
            .into_iter()
            .flatten()
            .rev()
            .map(|record| record.toast.message.as_ref())
            .collect::<Vec<_>>()
            .join("\n\n");
        let messages = history
            .into_iter()
            .flatten()
            .rev()
            .map(|record| {
                let toast = &record.toast;
                let status = cx.theme().status.for_severity(toast.severity);
                h_flex()
                    .id(("yp-toast-history-entry", record.sequence))
                    .flex_none()
                    .items_start()
                    .gap(px(6.))
                    .px(px(6.))
                    .py(px(4.))
                    .rounded(Sizes::ROW_RADIUS)
                    .bg(status.background)
                    .border_1()
                    .border_color(status.border)
                    .child(
                        div().flex_none().pt(px(1.)).child(
                            Icon::new(
                                toast
                                    .icon
                                    .unwrap_or_else(|| Callout::default_icon(toast.severity)),
                            )
                            .size(IconSize::Custom(px(12.)))
                            .color(Color::from(toast.severity)),
                        ),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            SelectableText::new(
                                ("yp-toast-history-message", record.sequence),
                                toast.message.clone(),
                            )
                            .text_size(px(12.))
                            .line_height(LINE_HEIGHT_TIGHT)
                            .color(Color::Default),
                        ),
                    )
            })
            .collect::<Vec<_>>();
        let viewport = window.viewport_size();
        let width = px(320.).min((viewport.width - sizes.padding_lg * 2.).max(px(0.)));
        let height = px(260.).min((viewport.height - sizes.padding_lg * 2. - px(40.)).max(px(0.)));
        menu_container(window, cx)
            .id("yp-toast-history")
            .relative()
            .key_context(TOAST_HISTORY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|_, _: &Cancel, _, cx| cx.emit(DismissEvent)))
            .w(width)
            .max_h(height)
            .text_size(px(13.))
            .gap(px(4.))
            .p(px(8.))
            .child(
                h_flex()
                    .flex_none()
                    .justify_between()
                    .gap(sizes.spacing_sm)
                    .child(Text::new(strings.toast_history_title()))
                    .child(
                        IconButton::new("yp-toast-history-close", IconName::Close)
                            .style(ButtonStyle::Transparent)
                            .size(ButtonSize::Compact)
                            .icon_size(IconSize::Custom(px(12.)))
                            .tooltip(Tooltip::text(strings.diagnostics_close()))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent))),
                    ),
            )
            .child(
                v_flex()
                    .id("yp-toast-history-messages")
                    .min_h_0()
                    .pr(Sizes::SCROLLBAR_WIDTH + px(4.))
                    .gap(px(4.))
                    .children(messages)
                    .overflow_y_scrollbar(&self.scroll),
            )
            .child(
                h_flex().flex_none().child(
                    CopyButton::new("yp-toast-history-copy", copied)
                        .icon_size(IconSize::Custom(px(12.)))
                        .tooltip(strings.toast_history_copy()),
                ),
            )
            .with_animation(
                "yp-toast-history-slide",
                Animation::new(cx.theme().motion.toast),
                |this, delta| {
                    this.top(px(16. * (1. - easing::nier_fill(delta))))
                        .opacity(easing::nier_rise(delta))
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    use gpui::{Entity, TestAppContext};

    use super::*;
    use crate::components::test_support::{open_empty_root, open_root};

    fn stack(cx: &mut TestAppContext) -> (Entity<ToastStack>, Rc<RefCell<Vec<ToastEvent>>>) {
        cx.update(crate::init_for_tests);
        let stack = cx.new(|_| ToastStack::new());
        let events = Rc::new(RefCell::new(Vec::new()));
        let sink = events.clone();
        cx.update(|cx| {
            cx.subscribe(&stack, move |_, event: &ToastEvent, _| {
                sink.borrow_mut().push(event.clone());
            })
            .detach();
        });
        (stack, events)
    }

    fn advance(cx: &mut TestAppContext, millis: u64) {
        cx.executor().advance_clock(Duration::from_millis(millis));
        cx.run_until_parked();
    }

    #[gpui::test]
    fn history_keeps_expired_and_replaced_messages(cx: &mut TestAppContext) {
        let (stack, _) = stack(cx);
        stack.update(cx, |stack, cx| {
            stack.push(Toast::warning("First message").id("status"), cx);
            stack.push(Toast::error("Second message\nMore detail").id("status"), cx);
        });
        advance(cx, 30_000);
        advance(cx, 1000);
        stack.read_with(cx, |stack, _| {
            assert!(stack.is_empty());
            let messages: Vec<_> = stack
                .history
                .iter()
                .rev()
                .map(|record| (record.toast.message.as_ref(), record.toast.severity))
                .collect();
            assert_eq!(
                messages,
                [
                    ("Second message\nMore detail", Severity::Error),
                    ("First message", Severity::Warning),
                ]
            );
        });
    }

    #[gpui::test]
    fn history_skips_blank_messages_and_keeps_recent_entries(cx: &mut TestAppContext) {
        let (stack, _) = stack(cx);
        stack.update(cx, |stack, cx| {
            stack.push(Toast::new(" \n "), cx);
            assert!(stack.history.is_empty());
            for index in 0..HISTORY_LIMIT + 5 {
                stack.push(Toast::new(index.to_string()).id("progress"), cx);
            }
            assert_eq!(stack.history.len(), HISTORY_LIMIT);
            assert_eq!(stack.history.front().unwrap().toast.message.as_ref(), "5");
            assert_eq!(
                stack.history.back().unwrap().toast.message.as_ref(),
                (HISTORY_LIMIT + 4).to_string()
            );
        });
    }

    #[gpui::test]
    fn the_corner_button_reopens_expired_messages_and_copies_live_history(cx: &mut TestAppContext) {
        let (root, cx) = open_empty_root(cx);
        cx.simulate_resize(gpui::size(px(900.), px(700.)));
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let corner = cx.update(|window, cx| {
            let margin = window.sizes(cx).padding_lg + px(8.);
            let viewport = window.viewport_size();
            gpui::point(viewport.width - margin, viewport.height - margin)
        });
        cx.simulate_click(corner, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(!root.read_with(cx, |root, _| root.has_modal()));
        let stack = root.read_with(cx, |root, _| root.toasts().clone());
        stack.update(cx, |stack, cx| {
            stack.push(Toast::warning("Missed warning"), cx);
        });
        advance(cx, 30_000);
        advance(cx, 1000);
        assert!(stack.read_with(cx, |stack, _| stack.is_empty()));
        cx.simulate_click(corner, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(!root.read_with(cx, |root, _| root.has_modal()));
        assert!(stack.read_with(cx, |stack, cx| stack.history_view.is_deployed(cx)));
        cx.simulate_click(corner, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(!stack.read_with(cx, |stack, cx| stack.history_view.is_deployed(cx)));
        cx.simulate_click(corner, gpui::Modifiers::none());
        cx.run_until_parked();
        stack.update(cx, |stack, cx| {
            stack.push(Toast::error("Newer failure"), cx);
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab tab enter");
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::KeyUp(gpui::KeyUpEvent {
                    keystroke: gpui::Keystroke::parse("enter").unwrap(),
                }),
                cx,
            );
        });
        cx.run_until_parked();
        let copied = cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
        assert_eq!(copied.as_deref(), Some("Newer failure\n\nMissed warning"));
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert!(!root.read_with(cx, |root, _| root.has_modal()));
        assert!(!stack.read_with(cx, |stack, cx| stack.history_view.is_deployed(cx)));
        cx.simulate_click(corner, gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(stack.read_with(cx, |stack, cx| stack.history_view.is_deployed(cx)));
        cx.simulate_click(gpui::point(px(30.), px(30.)), gpui::Modifiers::none());
        cx.run_until_parked();
        assert!(!stack.read_with(cx, |stack, cx| stack.history_view.is_deployed(cx)));
    }

    #[gpui::test]
    fn the_history_popup_stays_compact_and_scrolls_on_small_windows(cx: &mut TestAppContext) {
        cx.update(crate::init_for_tests);
        let stack = cx.new(|_| ToastStack::new());
        stack.update(cx, |stack, cx| {
            for index in 0..HISTORY_LIMIT {
                stack.push(Toast::warning(format!("Message {index}\nMore details")), cx);
            }
        });
        let history = cx.new(|cx| ToastHistory::new(stack.clone(), cx));
        let view = history.clone();
        let (_, cx) = open_root(cx, move |_, _| {
            div()
                .absolute()
                .bottom(px(32.))
                .right(px(16.))
                .child(view.clone())
                .into_any_element()
        });
        for (width, height) in [(900., 700.), (480., 320.)] {
            cx.simulate_resize(gpui::size(px(width), px(height)));
            advance(cx, 1000);
            history.read_with(cx, |history, _| {
                let bounds = history.scroll.bounds();
                assert!(bounds.size.width <= px(320.));
                assert!(bounds.size.height > px(40.));
                assert!(bounds.size.height < px(260.));
                assert!(bounds.origin.x >= px(0.));
                assert!(bounds.origin.y >= px(0.));
                assert!(bounds.bottom() < px(height));
                assert!(history.scroll.max_offset().y > px(0.));
            });
            let before = history.read_with(cx, |history, _| history.scroll.offset().y);
            let position = history.read_with(cx, |history, _| history.scroll.bounds().center());
            cx.simulate_event(gpui::ScrollEvent {
                position,
                delta: gpui::ScrollDelta::Lines(point(0., -3.)),
                ..Default::default()
            });
            cx.run_until_parked();
            assert!(history.read_with(cx, |history, _| history.scroll.offset().y < before));
        }
    }

    #[gpui::test]
    fn toasts_leave_after_their_reading_time(cx: &mut TestAppContext) {
        let (stack, events) = stack(cx);
        let id = stack.update(cx, |stack, cx| stack.push(Toast::new("short"), cx));
        advance(cx, 3900);
        stack.read_with(cx, |stack, _| {
            assert!(stack.contains(&id));
            assert!(!stack.is_leaving(&id));
        });
        advance(cx, 200);
        stack.read_with(cx, |stack, _| assert!(stack.is_leaving(&id)));
        advance(cx, 300);
        stack.read_with(cx, |stack, _| assert!(stack.is_empty()));
        assert_eq!(*events.borrow(), vec![ToastEvent::Dismissed(id)]);
    }

    #[gpui::test]
    fn hovering_keeps_a_toast_until_two_seconds_after_the_pointer_leaves(cx: &mut TestAppContext) {
        let (stack, events) = stack(cx);
        let id = stack.update(cx, |stack, cx| stack.push(Toast::new("hover me"), cx));
        stack.update(cx, |stack, cx| stack.set_hovered(&id, true, cx));
        advance(cx, 6000);
        stack.read_with(cx, |stack, _| assert!(!stack.is_leaving(&id)));
        stack.update(cx, |stack, cx| stack.set_hovered(&id, false, cx));
        advance(cx, 1900);
        stack.read_with(cx, |stack, _| assert!(!stack.is_leaving(&id)));
        advance(cx, 200);
        stack.read_with(cx, |stack, _| assert!(stack.is_leaving(&id)));
        advance(cx, 300);
        assert_eq!(events.borrow().len(), 1);
    }

    #[gpui::test]
    fn pushing_an_existing_id_replaces_the_toast_and_restarts_its_timer(cx: &mut TestAppContext) {
        let (stack, events) = stack(cx);
        let id = stack.update(cx, |stack, cx| stack.push(Toast::new("one").id("busy"), cx));
        advance(cx, 3000);
        stack.update(cx, |stack, cx| {
            stack.push(Toast::error("two").id("busy"), cx);
            assert_eq!(stack.len(), 1);
            assert_eq!(stack.ids(), vec![id.clone()]);
        });
        advance(cx, 3000);
        stack.read_with(cx, |stack, _| assert!(!stack.is_leaving(&id)));
        advance(cx, 1500);
        assert_eq!(events.borrow().len(), 1);
    }

    #[gpui::test]
    fn dismiss_and_clear_animate_out_before_removal(cx: &mut TestAppContext) {
        let (stack, events) = stack(cx);
        let (first, second) = stack.update(cx, |stack, cx| {
            (
                stack.push(Toast::new("a"), cx),
                stack.push(Toast::new("b"), cx),
            )
        });
        assert_ne!(first, second);
        stack.update(cx, |stack, cx| {
            stack.dismiss(&first, cx);
            stack.dismiss(&first, cx);
            assert_eq!(stack.len(), 2);
        });
        advance(cx, 300);
        stack.update(cx, |stack, cx| {
            assert_eq!(stack.ids(), vec![second.clone()]);
            stack.clear(cx);
        });
        advance(cx, 300);
        assert_eq!(events.borrow().len(), 2);
    }
}
