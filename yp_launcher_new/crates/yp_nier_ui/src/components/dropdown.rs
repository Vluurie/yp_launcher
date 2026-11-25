mod action;
mod popup;
use super::SettingRow;
pub use action::MenuActionMenu;
use gpui::{
    App, Bounds, ElementId, Entity, FocusHandle, IntoElement, KeyDownEvent, Pixels, RenderOnce,
    SharedString, Window, canvas, div, prelude::*, px,
};
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

type Changed = Rc<dyn Fn(usize, &mut Window, &mut App)>;
#[derive(Clone)]
pub struct MenuChoice {
    pub label: SharedString,
    pub enabled: bool,
}
impl MenuChoice {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}
struct State {
    focus: FocusHandle,
    previous: Option<FocusHandle>,
    bounds: Bounds<Pixels>,
    cursor: Option<usize>,
    first: usize,
    closing: Option<Instant>,
    opened: Instant,
    position: f32,
    scroll_from: Option<(f32, Instant)>,
    shown: usize,
}
impl State {
    fn move_to(&mut self, i: usize, count: usize) {
        if i >= count {
            return;
        }
        self.cursor = Some(i);
        let first = self
            .first
            .min(i)
            .max(i.saturating_sub(self.shown - 1))
            .min(count.saturating_sub(self.shown));
        if self.first != first {
            self.scroll_from =
                (self.first.abs_diff(first) == 1).then(|| (self.scroll_position(), Instant::now()));
            self.first = first;
            self.position = first as f32;
        }
    }
    fn scroll_position(&self) -> f32 {
        let Some((from, time)) = self.scroll_from else {
            return self.position;
        };
        let elapsed = time.elapsed().as_secs_f32();
        let amount = crate::motion::ease_out(elapsed * 60. / 19.);
        from + (self.position - from) * amount.clamp(0., 1.)
    }
    fn open(&mut self, i: usize, count: usize, w: &mut Window, cx: &mut App) {
        if count == 0 {
            return;
        }
        if !self.focus.is_focused(w) {
            self.previous = w.focused(cx);
        }
        self.focus.focus(w, cx);
        self.closing = None;
        self.opened = Instant::now();
        self.first = i.min(count.saturating_sub(self.shown));
        self.position = self.first as f32;
        self.scroll_from = None;
        self.move_to(i.min(count - 1), count);
    }
    fn close(&mut self, w: &mut Window, cx: &mut App) {
        if self.cursor.take().is_some() {
            self.closing = Some(Instant::now());
        }
        if self.focus.is_focused(w)
            && let Some(focus) = self.previous.take()
        {
            focus.focus(w, cx);
        }
    }
}
#[derive(IntoElement)]
pub struct MenuDropdown {
    id: ElementId,
    label: SharedString,
    choices: Rc<[MenuChoice]>,
    selected: usize,
    enabled: bool,
    scale: f32,
    changed: Changed,
    scrolling: bool,
    slot: usize,
    custom_trigger: Option<action::Trigger>,
    anchor: Option<[f32; 2]>,
}
impl MenuDropdown {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        choices: impl IntoIterator<Item = MenuChoice>,
        selected: usize,
        on_change: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            choices: choices.into_iter().collect(),
            selected,
            enabled: true,
            scale: 1.,
            changed: Rc::new(on_change),
            scrolling: false,
            slot: 0,
            custom_trigger: None,
            anchor: None,
        }
    }
    pub fn scrolling(mut self) -> Self {
        self.scrolling = true;
        self
    }
    pub fn row_slot(mut self, slot: usize) -> Self {
        self.slot = slot;
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
fn commit(
    index: usize,
    state: &Entity<State>,
    choices: &[MenuChoice],
    change: &Changed,
    w: &mut Window,
    cx: &mut App,
) {
    if choices.get(index).is_some_and(|o| o.enabled) {
        state.update(cx, |s, cx| {
            s.close(w, cx);
            cx.notify();
        });
        change(index, w, cx);
    }
}
impl RenderOnce for MenuDropdown {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = w.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            focus: cx.focus_handle(),
            previous: None,
            bounds: Default::default(),
            cursor: None,
            first: 0,
            closing: None,
            opened: Instant::now(),
            position: 0.,
            scroll_from: None,
            shown: 1,
        });
        let enabled = self.enabled && !self.choices.is_empty();
        let count = self.choices.len();
        let large = self.scrolling || count > 8;
        let (focus, cursor, closing) = state.update(cx, |s, cx| {
            s.shown = if large { 9 } else { 8 };
            if let Some(i) = s.cursor
                && count > 0
                && (i >= count || s.first > count.saturating_sub(s.shown))
            {
                s.move_to(i.min(count - 1), count);
            }
            if s.cursor.is_some() && (!enabled || !s.focus.is_focused(w)) {
                s.close(w, cx);
            }
            if s.closing
                .is_some_and(|t| t.elapsed() >= Duration::from_secs_f32(10. / 60.))
            {
                s.closing = None;
            }
            (s.focus.clone(), s.cursor, s.closing.is_some())
        });
        if closing {
            w.request_animation_frame();
        }
        let selected = self.selected;
        let trigger = state.clone();
        let track = state.clone();
        let key = state.clone();
        let options = self.choices.clone();
        let on_key = self.changed.clone();
        let focused = focus.is_focused(w);
        let toggle = move |_: &gpui::ClickEvent, w: &mut Window, cx: &mut App| {
            trigger.update(cx, |s, cx| {
                if s.cursor.is_some() {
                    s.close(w, cx);
                } else {
                    s.open(selected, count, w, cx);
                }
                cx.notify();
            });
            cx.stop_propagation();
        };
        let (row, width, height) = if let Some(custom) = self.custom_trigger.take() {
            let contents = (custom.render)(cursor.is_some());
            (
                div()
                    .id("action-trigger")
                    .size_full()
                    .when(enabled, |el| el.cursor_pointer().on_click(toggle))
                    .when(!enabled, |el| el.opacity(0.5))
                    .child(contents)
                    .into_any_element(),
                custom.size[0],
                custom.size[1],
            )
        } else {
            let row = SettingRow::new("value-header", self.label.clone())
                .editing(cursor.is_some())
                .scale(self.scale)
                .enabled(enabled)
                .value(
                    self.choices
                        .get(selected)
                        .map(|o| o.label.clone())
                        .unwrap_or_default(),
                )
                .on_click(toggle);
            (
                row.when(focused || cursor.is_some(), |r| r.selected(true))
                    .into_any_element(),
                692.,
                64.,
            )
        };
        let mut root = div()
            .id(self.id.clone())
            .relative()
            .track_focus(&focus)
            .tab_stop(enabled)
            .w(px(width * self.scale))
            .h(px(height * self.scale))
            .child(row)
            .child(
                canvas(
                    move |b, _, cx| {
                        track.update(cx, |s, _| s.bounds = b);
                    },
                    |_, _, _, _| (),
                )
                .absolute()
                .inset_0(),
            )
            .on_key_down(move |ev: &KeyDownEvent, w, cx| {
                if !enabled {
                    return;
                }
                let k = ev.keystroke.key.as_str();
                if key.read(cx).cursor.is_some() {
                    let idx = key.read(cx).cursor.unwrap();
                    match k {
                        "up" | "w" | "down" | "s" => {
                            let step = if k == "up" || k == "w" { count - 1 } else { 1 };
                            key.update(cx, |s, cx| {
                                s.move_to((idx + step) % count, count);
                                cx.notify();
                            });
                        }
                        "enter" | "space" => commit(idx, &key, &options, &on_key, w, cx),
                        "escape" => key.update(cx, |s, cx| {
                            s.close(w, cx);
                            cx.notify();
                        }),
                        "tab" => {
                            key.update(cx, |s, cx| {
                                s.close(w, cx);
                                cx.notify();
                            });
                            return;
                        }
                        _ => return,
                    }
                } else if matches!(k, "enter" | "space") {
                    key.update(cx, |s, cx| {
                        s.open(selected, count, w, cx);
                        cx.notify();
                    });
                } else {
                    return;
                }
                cx.stop_propagation();
            });
        if cursor.is_some() || closing {
            root = root.child(popup::Popup {
                dropdown: self,
                state,
                large,
            });
        }
        root
    }
}
#[derive(IntoElement)]
pub struct MenuToggle(MenuDropdown);
impl MenuToggle {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        on: bool,
        change: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self(MenuDropdown::new(
            id,
            label,
            [MenuChoice::new("On"), MenuChoice::new("Off")],
            usize::from(!on),
            move |i, w, cx| change(i == 0, w, cx),
        ))
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.0 = self.0.enabled(enabled);
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.0 = self.0.scale(scale);
        self
    }
}
impl RenderOnce for MenuToggle {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.0
    }
}
