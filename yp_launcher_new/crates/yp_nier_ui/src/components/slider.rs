mod input;
use super::control::{SettingRow, layout::Transition};
use crate::{LIGHT, Rect, Surface};
use gpui::{
    App, ElementId, FocusHandle, IntoElement, KeyDownEvent, Pixels, RenderOnce, SharedString,
    Window, div, prelude::*, px,
};
use std::{rc::Rc, time::Instant};

type Changed = Rc<dyn Fn(usize, &mut Window, &mut App)>;
struct State {
    focus: FocusHandle,
    previous: Option<FocusHandle>,
    draft: Option<usize>,
    transition: Option<Instant>,
    last_hit_x: Option<Pixels>,
}
impl State {
    fn enter(&mut self, value: usize, w: &mut Window, cx: &mut App) {
        if !self.focus.is_focused(w) {
            self.previous = w.focused(cx);
        }
        self.focus.focus(w, cx);
        self.draft = Some(value);
        self.transition = Some(Instant::now());
    }
    fn exit(&mut self, w: &mut Window, cx: &mut App) {
        if self.draft.take().is_some() {
            self.transition = Some(Instant::now());
        }
        self.last_hit_x = None;
        if self.focus.is_focused(w)
            && let Some(f) = self.previous.take()
        {
            f.focus(w, cx);
        }
    }
}
#[derive(IntoElement)]
struct Step {
    index: usize,
    active: bool,
    filled: bool,
    marked: bool,
    scale: f32,
}
impl RenderOnce for Step {
    fn render(self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = w.use_keyed_state(("step", self.index), cx, |_, _| {
            (Transition::new(self.filled), Transition::new(self.active))
        });
        let ((fill, a), (active, b)) = state.update(cx, |s, _| {
            (s.0.sample(self.filled, 5.), s.1.sample(self.active, 4.))
        });
        if a || b {
            w.request_animation_frame();
        }
        let mut art = Surface::new(12., 64.);
        let height = 4. + 12. * fill + 6. * active;
        art.fill(Rect::new(4., 32. - height / 2., 4., height), LIGHT, 255);
        if self.marked {
            for y in [14., 47.] {
                art.fill(Rect::new(3., y, 6., 3.), LIGHT, 255);
            }
        }
        div()
            .w(px(12. * self.scale))
            .h(px(64. * self.scale))
            .child(art)
    }
}
#[derive(IntoElement)]
pub struct MenuSlider {
    id: ElementId,
    label: SharedString,
    value: usize,
    steps: usize,
    minimum: usize,
    default: Option<usize>,
    scale: f32,
    enabled: bool,
    changed: Changed,
}
impl MenuSlider {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        value: usize,
        steps: usize,
        on_change: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        let steps = steps.clamp(1, 20);
        Self {
            id: id.into(),
            label: label.into(),
            value: value.min(steps),
            steps,
            minimum: 0,
            default: None,
            scale: 1.,
            enabled: true,
            changed: Rc::new(on_change),
        }
    }
    pub fn minimum(mut self, minimum: usize) -> Self {
        self.minimum = minimum.min(self.steps);
        self.value = self.value.max(self.minimum);
        self
    }
    pub fn default_mark(mut self, v: usize) -> Self {
        self.default = Some(v.min(self.steps));
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
impl RenderOnce for MenuSlider {
    fn render(self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = w.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            focus: cx.focus_handle(),
            previous: None,
            draft: None,
            transition: None,
            last_hit_x: None,
        });
        let (focus, draft, transition) = state.update(cx, |s, cx| {
            if s.draft.is_some() && (!self.enabled || !s.focus.is_focused(w)) {
                s.exit(w, cx);
            }
            (s.focus.clone(), s.draft, s.transition)
        });
        let enable = self.enabled;
        let current = self.value;
        let steps = self.steps;
        let minimum = self.minimum;
        let elapsed = transition.map(|t| t.elapsed().as_secs_f32()).unwrap_or(1.);
        let sliding = elapsed < 10. / 60.;
        if sliding {
            w.request_animation_frame();
        }
        let group_visible = draft.is_some() || sliding;
        let scale = self.scale;
        let click = state.clone();
        let action = self.changed.clone();
        let key = state.clone();
        let change = self.changed.clone();
        let mut row = SettingRow::new("slider-head", self.label)
            .scale(scale)
            .enabled(enable)
            .editing(draft.is_some())
            .steps(draft.unwrap_or(current), steps, self.default, group_visible)
            .on_click(move |_, w, cx| {
                let d = click.read(cx).draft;
                click.update(cx, |s, cx| {
                    if d.is_some() {
                        s.exit(w, cx);
                    } else {
                        s.enter(current, w, cx);
                    }
                    cx.notify();
                });
                if let Some(v) = d {
                    action(v, w, cx);
                }
                cx.stop_propagation();
            });
        if focus.is_focused(w) {
            row = row.selected(true);
        }
        let mut root = div()
            .id(self.id)
            .relative()
            .w(px(692. * scale))
            .h(px(64. * scale))
            .track_focus(&focus)
            .tab_stop(enable)
            .child(row)
            .on_key_down(move |ev: &KeyDownEvent, w, cx| {
                if !enable {
                    return;
                }
                let k = ev.keystroke.key.as_str();
                let d = key.read(cx).draft;
                match (d, k) {
                    (None, "enter" | "space") => key.update(cx, |s, cx| {
                        s.enter(current, w, cx);
                        cx.notify();
                    }),
                    (Some(v), "left" | "a" | "right" | "d") => key.update(cx, |s, cx| {
                        s.last_hit_x = None;
                        s.draft = Some(if matches!(k, "left" | "a") {
                            v.saturating_sub(1).max(minimum)
                        } else {
                            (v + 1).min(steps)
                        });
                        cx.notify();
                    }),
                    (Some(v), "enter" | "space" | "escape") => {
                        key.update(cx, |s, cx| {
                            s.exit(w, cx);
                            cx.notify();
                        });
                        change(v, w, cx);
                    }
                    _ => return,
                }
                cx.stop_propagation();
            });
        if group_visible {
            let travel = (elapsed * 60. / 5.).clamp(0., 1.);
            let openness = if draft.is_some() { travel } else { 1. - travel };
            let last_center = 633.5 + 55. * openness;
            let first_center = last_center - (steps - 1) as f32 * 8.;
            let value = draft.unwrap_or(current);
            let slots = (0..steps).map(|i| {
                div()
                    .absolute()
                    .top_0()
                    .left(px((first_center - 6. + i as f32 * 8.) * scale))
                    .w(px(12. * scale))
                    .h(px(64. * scale))
                    .child(Step {
                        index: i,
                        active: draft.is_some() && value == i + 1,
                        filled: i < value,
                        marked: self.default.is_some_and(|v| v.saturating_sub(1) == i),
                        scale,
                    })
            });
            root = root.children(slots);
            if draft.is_some() && scale > 0. {
                root = root.child(input::SegmentInput {
                    state,
                    first_center,
                    scale,
                    steps,
                    minimum,
                    changed: self.changed,
                });
            }
        }
        root
    }
}
