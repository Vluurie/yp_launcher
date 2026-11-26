use crate::{DialogButton, GameText, INK, LIGHT, PANEL, Rect, Surface};
use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px};
use gpui::{Entity, FocusHandle, KeyDownEvent};
use std::{rc::Rc, time::Instant};

type Answer = Rc<dyn Fn(bool, &mut Window, &mut App)>;
#[derive(IntoElement)]
pub struct ConfirmDialog {
    id: ElementId,
    title: SharedString,
    prompt: SharedString,
    labels: [SharedString; 2],
    modal_focus: Option<FocusHandle>,
    yes: Option<bool>,
    scale: f32,
    answer: Answer,
    selection: Option<Answer>,
    dismissal: Option<bool>,
}
impl ConfirmDialog {
    pub fn new(
        prompt: impl Into<SharedString>,
        answer: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: "confirm-dialog".into(),
            title: "SYSTEM MESSAGE".into(),
            prompt: prompt.into(),
            labels: ["Yes".into(), "No".into()],
            modal_focus: None,
            yes: None,
            scale: 1.,
            answer: Rc::new(answer),
            selection: None,
            dismissal: None,
        }
    }
    pub fn labels(mut self, yes: impl Into<SharedString>, no: impl Into<SharedString>) -> Self {
        self.labels = [yes.into(), no.into()];
        self
    }

    pub fn in_modal(mut self, focus: FocusHandle) -> Self {
        self.modal_focus = Some(focus);
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    pub fn dismiss(mut self, answer: Option<bool>) -> Self {
        self.dismissal = answer;
        self
    }

    pub fn affirmative(mut self, selected: bool) -> Self {
        self.yes = Some(selected);
        self
    }
    pub fn on_select(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.selection = Some(Rc::new(f));
        self
    }
}
struct Playback {
    opened: Instant,
    dismiss: Option<(Instant, bool)>,
    completed: bool,
    selected: bool,
    last_external: Option<bool>,
    focus: FocusHandle,
    previous: Option<FocusHandle>,
}

impl Playback {
    fn select(
        state: &Entity<Self>,
        yes: bool,
        callback: &Option<Answer>,
        w: &mut Window,
        cx: &mut App,
    ) {
        if state.read(cx).dismiss.is_some() || state.read(cx).selected == yes {
            return;
        }
        state.update(cx, |s, cx| {
            s.selected = yes;
            cx.notify();
        });
        if let Some(f) = callback {
            f(yes, w, cx);
        }
    }

    fn close(state: &Entity<Self>, yes: bool, cx: &mut App) {
        state.update(cx, |s, cx| {
            if s.dismiss.is_none() {
                s.selected = yes;
                let shown = s.opened + std::time::Duration::from_secs_f32(20. / 60.);
                s.dismiss = Some((Instant::now().max(shown), yes));
                cx.notify();
            }
        });
    }
}

impl RenderOnce for ConfirmDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hosted = self.modal_focus.is_some();
        let state = window.use_keyed_state(self.id.clone(), cx, |w, cx| {
            let previous = w.focused(cx);
            let focus = self.modal_focus.unwrap_or_else(|| {
                let handle = cx.focus_handle();
                handle.focus(w, cx);
                handle
            });
            Playback {
                opened: Instant::now(),
                dismiss: None,
                selected: self.yes.unwrap_or(false),
                last_external: self.yes,
                completed: false,
                focus,
                previous,
            }
        });
        if let Some(yes) = self.dismissal
            && state.read(cx).dismiss.is_none()
        {
            Playback::close(&state, yes, cx);
        }
        let (clip, age, yes, focus) = state.update(cx, |st, _| {
            if st.dismiss.is_none() && self.yes != st.last_external {
                if let Some(yes) = self.yes {
                    st.selected = yes;
                }
                st.last_external = self.yes;
            }
            let (clip, since) = st
                .dismiss
                .filter(|(t, _)| *t <= Instant::now())
                .map_or((0, st.opened), |(t, _)| (2, t));
            (
                clip,
                since.elapsed().as_secs_f32(),
                st.selected,
                st.focus.clone(),
            )
        });
        let duration = if clip == 2 { 10. / 60. } else { 20. / 60. };
        let reduced = crate::ReducedMotion::enabled(cx);
        if age < duration && !(hosted && reduced) {
            window.request_animation_frame();
        } else if clip == 2 && !state.read(cx).completed {
            state.update(cx, |st, _| st.completed = true);
            let answer = self.answer.clone();
            let previous = state.read(cx).previous.clone();
            window.defer(cx, move |w, cx| {
                if let Some(focus) = previous {
                    focus.focus(w, cx);
                }
                answer(yes, w, cx);
            });
        }
        let t = if hosted && reduced {
            1.
        } else {
            (age / duration).clamp(0., 1.)
        };
        let opacity = if clip == 2 {
            (1. - t).powf(2.2)
        } else {
            crate::motion::ease_out(t)
        };
        let scale = self.scale
            * if clip == 2 {
                1. - 0.1 * t * t
            } else {
                0.88 + 0.12 * crate::motion::ease_out(t)
            };
        let scrim = super::modal_backdrop(opacity);
        let button_opacity = if clip == 2 { 1. - t } else { 1. };
        let keys = state.clone();
        let key_selection = self.selection.clone();
        let mut panel = Surface::new(674., 320.);
        panel.fill(Rect::new(8., 8., 658., 302.), LIGHT, 255);
        panel.fill(Rect::new(8., 8., 658., 42.), INK, 255);
        panel.fill(Rect::new(20., 19., 19., 19.), LIGHT, 255);
        panel.text(&self.title, 1, 0.68, 50., 29., LIGHT);
        panel.fill(Rect::new(19., 218., 635., 2.), PANEL, 255);
        let answer: Answer = if hosted {
            self.answer
        } else {
            let state = state.clone();
            Rc::new(move |yes, _, cx| Playback::close(&state, yes, cx))
        };
        let keyboard_answer = answer.clone();
        let [yes_label, no_label] = self.labels;
        let buttons =
            [(yes_label, true), (no_label, false)]
                .into_iter()
                .map(move |(label, choice)| {
                    let hovered = state.clone();
                    let click = answer.clone();
                    let selection = self.selection.clone();
                    let button =
                        DialogButton::new(if choice { "dialog-yes" } else { "dialog-no" }, label)
                            .scale(scale)
                            .selected(yes == choice)
                            .on_hover(move |over, w, cx| {
                                if over {
                                    Playback::select(&hovered, choice, &selection, w, cx);
                                }
                            })
                            .on_click(move |_, w, cx| click(choice, w, cx));
                    div()
                        .debug_selector(move || {
                            if choice {
                                "system-dialog-yes"
                            } else {
                                "system-dialog-no"
                            }
                            .into()
                        })
                        .absolute()
                        .left(px(if choice { 62. } else { 342. } * scale))
                        .top(px(230. * scale))
                        .opacity(button_opacity)
                        .child(button)
                });
        div()
            .id(self.id)
            .role(gpui::Role::Dialog)
            .aria_label(self.prompt.clone())
            .track_focus(&focus)
            .on_key_down(move |event: &KeyDownEvent, w, cx| {
                let yes = keys.read(cx).selected;
                match event.keystroke.key.as_str() {
                    "left" | "right" | "a" | "d" => {
                        Playback::select(&keys, !yes, &key_selection, w, cx)
                    }
                    "enter" | "space" if !event.is_held => keyboard_answer(yes, w, cx),
                    "escape" if hosted => keyboard_answer(false, w, cx),
                    "escape" if yes => Playback::select(&keys, false, &key_selection, w, cx),
                    "escape" => Playback::close(&keys, false, cx),
                    _ => {}
                }
                cx.stop_propagation();
            })
            .when(!hosted, |el| el.size_full())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .occlude()
            .when(!hosted, |el| {
                el.child(div().absolute().inset_0().child(scrim))
            })
            .child(
                div()
                    .relative()
                    .opacity(opacity)
                    .w(px(674. * scale))
                    .h(px(320. * scale))
                    .mt(px(-4. * scale))
                    .child(div().absolute().inset_0().child(panel))
                    .child(
                        div()
                            .absolute()
                            .left(px(40. * scale))
                            .right(px(40. * scale))
                            .top(px(74. * scale))
                            .flex()
                            .justify_center()
                            .overflow_hidden()
                            .child(
                                GameText::new(self.prompt)
                                    .reveal(false)
                                    .scale(0.8 * scale)
                                    .wrap(590. * scale),
                            ),
                    )
                    .children(buttons),
            )
    }
}
