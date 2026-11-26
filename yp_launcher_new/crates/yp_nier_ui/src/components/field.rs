mod input;

use crate::{INK, LIGHT, PANEL, Rect, Surface, Theme};
use gpui::{
    App, Bounds, ClipboardItem, Context, ElementId, ElementInputHandler, Entity, EventEmitter,
    FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseButton, Pixels, RenderOnce,
    SharedString, Window, canvas, div, prelude::*, px,
};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldEvent {
    Edited,
    Submitted,
    Cancelled,
}

pub struct MenuFieldState {
    value: String,
    anchor: usize,
    head: usize,
    mark: Option<Range<usize>>,
    stops: Vec<(usize, f32)>,
    measured: u64,
    scroll: f32,
    scale: f32,
    bounds: Bounds<Pixels>,
    focus: FocusHandle,
    previous: Option<FocusHandle>,
}
impl EventEmitter<FieldEvent> for MenuFieldState {}
impl Focusable for MenuFieldState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl MenuFieldState {
    pub fn new(value: impl Into<String>, cx: &mut Context<Self>) -> Self {
        let value = Self::single_line(&value.into());
        let stops = Theme::bundled().caret_positions(&value, 36, 0.8);
        let measured = Theme::bundled().generation();
        Self {
            anchor: value.len(),
            head: value.len(),
            value,
            mark: None,
            stops,
            measured,
            scroll: 0.,
            scale: 1.,
            bounds: Default::default(),
            focus: cx.focus_handle(),
            previous: None,
        }
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn set_value(&mut self, value: &str, w: &mut Window, cx: &mut Context<Self>) {
        self.replace(0..self.value.len(), value, w, cx);
    }
    fn single_line(value: &str) -> String {
        value.chars().filter(|c| !c.is_control()).collect()
    }
    fn range(&self) -> Range<usize> {
        self.head.min(self.anchor)..self.head.max(self.anchor)
    }
    fn replace(
        &mut self,
        range: Range<usize>,
        text: &str,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) -> Range<usize> {
        let text = Self::single_line(text);
        self.value.replace_range(range.clone(), &text);
        let end = range.start + text.len();
        self.anchor = end;
        self.head = end;
        self.mark = None;
        self.measure();
        w.notify_text_input_changed(&self.focus);
        cx.emit(FieldEvent::Edited);
        cx.notify();
        range.start..end
    }
    fn measure(&mut self) {
        self.stops = Theme::bundled().caret_positions(&self.value, 36, 0.8);
        self.measured = Theme::bundled().generation();
    }
    fn x(&self, byte: usize) -> f32 {
        self.stops[self
            .stops
            .partition_point(|&(at, _)| at <= byte)
            .saturating_sub(1)]
        .1
    }
    fn byte_at(&self, x: Pixels) -> usize {
        let x = (x - self.bounds.left()) / px(self.scale.max(0.001)) - 12. + self.scroll;
        let i = self
            .stops
            .partition_point(|&(_, p)| p < x)
            .min(self.stops.len() - 1);
        if i > 0 && x - self.stops[i - 1].1 < self.stops[i].1 - x {
            self.stops[i - 1].0
        } else {
            self.stops[i].0
        }
    }
    fn select(&mut self, to: usize, extend: bool, w: &mut Window, cx: &mut Context<Self>) {
        self.head = to;
        if !extend {
            self.anchor = to;
        }
        self.mark = None;
        w.notify_text_selection_changed(&self.focus);
        cx.notify();
    }
    fn handle_key(&mut self, ev: &KeyDownEvent, w: &mut Window, cx: &mut Context<Self>) {
        let k = ev.keystroke.key.as_str();
        let m = ev.keystroke.modifiers;
        let shortcut = m.control || m.platform;
        let selected = self.range();
        let before = || {
            self.value[..self.head]
                .grapheme_indices(true)
                .next_back()
                .map_or(0, |(i, _)| i)
        };
        let after = || {
            self.head
                + self.value[self.head..]
                    .graphemes(true)
                    .next()
                    .map_or(0, str::len)
        };
        match (shortcut, k) {
            (true, "a") => {
                self.anchor = 0;
                self.select(self.value.len(), true, w, cx);
            }
            (true, "c" | "x") => {
                if !selected.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        self.value[selected.clone()].to_owned(),
                    ));
                    if k == "x" {
                        self.replace(selected, "", w, cx);
                    }
                }
            }
            (true, "v") => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.replace(selected, &text, w, cx);
                }
            }
            (false, "home" | "end" | "left" | "right") => {
                let pos = match k {
                    "home" => 0,
                    "end" => self.value.len(),
                    "left" if !selected.is_empty() && !m.shift => selected.start,
                    "right" if !selected.is_empty() && !m.shift => selected.end,
                    "left" => before(),
                    _ => after(),
                };
                self.select(pos, m.shift, w, cx);
            }
            (false, "backspace" | "delete") => {
                let cut = if !selected.is_empty() {
                    selected
                } else if k == "delete" {
                    self.head..after()
                } else {
                    before()..self.head
                };
                self.replace(cut, "", w, cx);
            }
            (false, "enter" | "escape") if self.mark.is_none() => {
                cx.emit(if k == "enter" {
                    FieldEvent::Submitted
                } else {
                    FieldEvent::Cancelled
                });
                if let Some(prev) = self.previous.take() {
                    prev.focus(w, cx);
                }
                cx.notify();
            }
            _ => return,
        }
        cx.stop_propagation();
    }
}

#[derive(IntoElement)]
pub struct MenuTextField {
    id: ElementId,
    state: Entity<MenuFieldState>,
    placeholder: SharedString,
    width: f32,
    scale: f32,
    enabled: bool,
}
impl MenuTextField {
    pub fn new(id: impl Into<ElementId>, state: Entity<MenuFieldState>) -> Self {
        Self {
            id: id.into(),
            state,
            placeholder: SharedString::default(),
            width: 420.,
            scale: 1.,
            enabled: true,
        }
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(24.);
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.001);
        self
    }
}
impl RenderOnce for MenuTextField {
    fn render(self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.state.read(cx).measured != Theme::bundled().generation() {
            self.state.update(cx, |state, _| state.measure());
        }
        let focus = self.state.read(cx).focus.clone();
        let enabled = self.enabled;
        if !enabled && focus.is_focused(w) {
            self.state.update(cx, |s, cx| {
                s.mark = None;
                if let Some(previous) = s.previous.take() {
                    previous.focus(w, cx);
                } else {
                    w.blur(cx);
                }
                w.dismiss_virtual_keyboard();
            });
        }
        let focused = enabled && focus.is_focused(w);
        let click = self.state.clone();
        let drag = click.clone();
        let key = click.clone();
        let draw = click.clone();
        let width = self.width;
        let scale = self.scale;
        div()
            .id(self.id)
            .track_focus(&focus)
            .tab_stop(enabled)
            .when(!enabled, |el| el.opacity(0.5))
            .w(px(width * scale))
            .h(px(40. * scale))
            .when(enabled, |el| {
                el.cursor_text()
                    .on_mouse_down(MouseButton::Left, move |ev, w, cx| {
                        click.update(cx, |s, cx| {
                            if !s.focus.is_focused(w) {
                                s.previous = w.focused(cx);
                            }
                            s.focus.focus(w, cx);
                            w.request_virtual_keyboard();
                            if ev.click_count >= 2 {
                                s.anchor = 0;
                                s.select(s.value.len(), true, w, cx);
                            } else {
                                s.select(s.byte_at(ev.position.x), ev.modifiers.shift, w, cx);
                            }
                        });
                        cx.stop_propagation();
                    })
                    .on_mouse_move(move |ev, w, cx| {
                        if focused && ev.pressed_button == Some(MouseButton::Left) {
                            drag.update(cx, |s, cx| {
                                s.select(s.byte_at(ev.position.x), true, w, cx)
                            });
                        }
                    })
                    .on_key_down(move |ev, w, cx| key.update(cx, |s, cx| s.handle_key(ev, w, cx)))
            })
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, w, cx| {
                        if enabled {
                            w.handle_input(
                                &focus,
                                ElementInputHandler::new(bounds, draw.clone()),
                                cx,
                            );
                        }
                        let mut surface = Surface::new(width, 40.);
                        draw.update(cx, |s, _| {
                            s.bounds = bounds;
                            s.scale = scale;
                            let x = s.x(s.head);
                            let inner = (width - 24.).max(1.);
                            s.scroll = s
                                .scroll
                                .min(x)
                                .max(x - inner)
                                .min((s.x(s.value.len()) - inner).max(0.));
                            let start = 12. - s.scroll;
                            surface.fill(Rect::new(0., 0., width, 40.), PANEL, 255);
                            surface.fill(
                                Rect::new(0., 38., width, 2.),
                                INK,
                                if focused { 255 } else { 115 },
                            );
                            surface.clip(Rect::new(9., 0., width - 18., 38.), |g| {
                                if focused {
                                    let sel = s.range();
                                    let left = s.x(sel.start);
                                    g.fill(
                                        Rect::new(start + left, 5., s.x(sel.end) - left, 30.),
                                        LIGHT,
                                        255,
                                    );
                                    g.fill(Rect::new(start + x, 6., 2., 28.), INK, 255);
                                }
                                if s.value.is_empty() {
                                    g.text(&self.placeholder, 36, 0.8, start, 20., INK);
                                } else {
                                    g.text(&s.value, 36, 0.8, start, 20., INK);
                                }
                                if let Some(mark) = &s.mark {
                                    let left = s.x(mark.start);
                                    g.fill(
                                        Rect::new(start + left, 34., s.x(mark.end) - left, 1.),
                                        INK,
                                        255,
                                    );
                                }
                            });
                        });
                        surface.paint(bounds, w, cx);
                    },
                )
                .size_full(),
            )
    }
}
