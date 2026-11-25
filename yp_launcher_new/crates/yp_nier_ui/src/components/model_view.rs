mod chrome;
mod motion;

use crate::{INK, Rect, Surface};
use gpui::{
    AnyElement, App, ElementId, FocusHandle, IntoElement, KeyDownEvent, MouseButton,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, RenderOnce, SharedString, Window, canvas, div,
    prelude::*, px,
};
use std::{
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewerAction {
    Open,
    Back,
    Rotate { dx: f32, dy: f32 },
    Animation,
    Equipment,
    Previous,
    Next,
}

struct State {
    expanded: bool,
    changed: Option<Instant>,
    focus: FocusHandle,
    previous: Option<FocusHandle>,
    last: Option<Point<Pixels>>,
}

#[derive(IntoElement)]
pub struct ModelViewer {
    id: ElementId,
    name: SharedString,
    description: SharedString,
    scene: AnyElement,
    expanded: bool,
    equipment: bool,
    motions: Option<(usize, usize)>,
    scale: f32,
    action: chrome::Action,
}
impl ModelViewer {
    pub fn new(
        id: impl Into<ElementId>,
        name: impl Into<SharedString>,
        scene: impl IntoElement,
        action: impl Fn(ViewerAction, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: SharedString::default(),
            scene: scene.into_any_element(),
            expanded: false,
            equipment: false,
            motions: None,
            scale: 1.,
            action: Rc::new(action),
        }
    }
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = text.into();
        self
    }
    pub fn expanded(mut self, open: bool) -> Self {
        self.expanded = open;
        self
    }
    pub fn equipment(mut self, available: bool) -> Self {
        self.equipment = available;
        self
    }
    pub fn motions(mut self, current: usize, total: usize) -> Self {
        self.motions = (total > 0).then_some((current.min(total), total));
        self
    }
    pub fn scale(mut self, s: f32) -> Self {
        self.scale = s.max(0.001);
        self
    }
}
impl RenderOnce for ModelViewer {
    fn render(self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = self.scale;
        let state = w.use_keyed_state(self.id.clone(), cx, |_, cx| State {
            expanded: false,
            changed: None,
            focus: cx.focus_handle(),
            previous: None,
            last: None,
        });
        let (elapsed, focus) = state.update(cx, |st, cx| {
            if st.expanded != self.expanded {
                st.expanded = self.expanded;
                st.changed = Some(Instant::now());
                st.last = None;
                if st.expanded {
                    st.previous = w.focused(cx);
                    st.focus.focus(w, cx);
                } else if st.focus.is_focused(w)
                    && let Some(ref prev) = st.previous
                {
                    prev.focus(w, cx);
                }
            }
            (
                st.changed
                    .map_or(Duration::from_secs(1), |t| t.elapsed())
                    .as_secs_f32(),
                st.focus.clone(),
            )
        });
        let motion = motion::Motion::native();
        let rect = motion.viewport(self.expanded, elapsed);
        let transitioning = elapsed < if self.expanded { 0.5 } else { 20. / 60. };
        if transitioning {
            w.request_animation_frame();
        }
        let click = self.action.clone();
        let rotate = self.action.clone();
        let press = state.clone();
        let drag = state.clone();
        let key = self.action.clone();
        let open = self.expanded;
        let mut root = div().id(self.id).relative().size_full().when(open, |el| {
            el.track_focus(&focus)
                .on_key_down(move |event: &KeyDownEvent, w, cx| {
                    let action = match event.keystroke.key.as_str() {
                        "escape" => ViewerAction::Back,
                        "enter" | "space" if self.motions.is_some() => ViewerAction::Animation,
                        "tab" if self.equipment => ViewerAction::Equipment,
                        "up" => ViewerAction::Previous,
                        "down" => ViewerAction::Next,
                        _ => return,
                    };
                    key(action, w, cx);
                    cx.stop_propagation();
                })
        });
        if !open || transitioning {
            root = root.child(chrome::details(&self.name, &self.description, s));
        }
        let mut bg = Surface::new(1., 1.);
        bg.fill(Rect::new(0., 0., 1., 1.), INK, 255);
        let mut scene = div()
            .id("model-surface")
            .relative()
            .size_full()
            .occlude()
            .overflow_hidden()
            .child(div().absolute().inset_0().child(bg))
            .child(self.scene);
        if open {
            scene = scene.child(
                div()
                    .id("orbit")
                    .absolute()
                    .inset_0()
                    .cursor_grab()
                    .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                        press.update(cx, |st, _| st.last = Some(e.position));
                        cx.stop_propagation();
                    })
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |_, _, w, _| {
                                w.on_mouse_event(move |ev: &MouseMoveEvent, phase, w, cx| {
                                    if !phase.capture()
                                        || ev.pressed_button != Some(MouseButton::Left)
                                    {
                                        return;
                                    }
                                    if let Some(last) = drag.read(cx).last {
                                        let delta = ev.position - last;
                                        drag.update(cx, |st, _| st.last = Some(ev.position));
                                        rotate(
                                            ViewerAction::Rotate {
                                                dx: delta.x / px(s),
                                                dy: delta.y / px(s),
                                            },
                                            w,
                                            cx,
                                        );
                                        cx.stop_propagation();
                                    }
                                });
                                w.on_mouse_event(move |ev: &MouseUpEvent, phase, _, cx| {
                                    if phase.capture()
                                        && ev.button == MouseButton::Left
                                        && state.read(cx).last.is_some()
                                    {
                                        state.update(cx, |st, _| st.last = None);
                                        cx.stop_propagation();
                                    }
                                });
                            },
                        )
                        .size_full(),
                    ),
            );
        } else {
            scene = scene.child(
                div()
                    .id("open-model")
                    .absolute()
                    .inset_0()
                    .cursor_pointer()
                    .on_click(move |_, w, cx| {
                        click(ViewerAction::Open, w, cx);
                        cx.stop_propagation();
                    }),
            );
        }
        root = root.child(chrome::place(rect, s, scene));
        if open || elapsed < 10. / 60. {
            let (x, alpha) = motion.title(open, elapsed);
            let (dx, opacity) = motion.footer(open, elapsed);
            root = root
                .child(chrome::title(&self.name, x, alpha, s))
                .child(chrome::footer(
                    dx,
                    opacity,
                    s,
                    self.motions,
                    self.equipment,
                    self.action,
                ));
        }
        root
    }
}
