use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    App, ElementId, Entity, FocusHandle, FontWeight, IntoElement, Keystroke, Modifiers,
    MouseButton, Pixels, Point, RenderOnce, SharedString, Window, prelude::*, px, relative,
};

use super::label::LINE_HEIGHT_TIGHT;
use super::text::Text;
use crate::i18n::ActiveStrings;
use crate::styled_ext::h_flex;
use crate::theme::{ActiveTheme, ResponsiveExt, Sizes};
use crate::traits::ValueHandler;

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureInput {
    Key(Keystroke),
    Modifiers(Modifiers),
    Mouse(MouseButton),
    Wheel(Point<Pixels>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptureFlow {
    Continue,
    Finish,
}

type CaptureHandler = Rc<dyn Fn(&CaptureInput, &mut Window, &mut App) -> CaptureFlow>;

pub struct CaptureState {
    capturing: bool,
    focus_handle: FocusHandle,
}

#[derive(Clone, Default)]
pub struct KeyCaptureHandle(Rc<RefCell<Option<Entity<CaptureState>>>>);

impl KeyCaptureHandle {
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> Option<Entity<CaptureState>> {
        self.0.borrow().clone()
    }

    pub fn is_capturing(&self, cx: &App) -> bool {
        self.state().is_some_and(|state| state.read(cx).capturing)
    }

    pub fn start(&self, window: &mut Window, cx: &mut App) {
        if let Some(state) = self.state() {
            let focus_handle = state.read(cx).focus_handle.clone();
            window.focus(&focus_handle, cx);
            set_capturing(&state, true, cx);
        }
    }

    pub fn stop(&self, cx: &mut App) {
        if let Some(state) = self.state() {
            set_capturing(&state, false, cx);
        }
    }
}

fn set_capturing(state: &Entity<CaptureState>, capturing: bool, cx: &mut App) {
    state.update(cx, |state, cx| {
        if state.capturing != capturing {
            state.capturing = capturing;
            cx.notify();
        }
    });
}

pub struct KeyCapture {
    id: ElementId,
    current: SharedString,
    disabled: bool,
    invalid: bool,
    min_width: Option<Pixels>,
    capturing_label: Option<SharedString>,
    modifiers: bool,
    mouse: bool,
    wheel: bool,
    handle: Option<KeyCaptureHandle>,
    on_capture: Option<ValueHandler<Keystroke>>,
    on_input: Option<CaptureHandler>,
    on_capturing_change: Option<ValueHandler<bool>>,
}

crate::impl_into_element!(KeyCapture);

impl KeyCapture {
    pub fn new(id: impl Into<ElementId>, current: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            current: current.into(),
            disabled: false,
            invalid: false,
            min_width: None,
            capturing_label: None,
            modifiers: false,
            mouse: false,
            wheel: false,
            handle: None,
            on_capture: None,
            on_input: None,
            on_capturing_change: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn min_width(mut self, width: Pixels) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn capturing_label(mut self, label: impl Into<SharedString>) -> Self {
        self.capturing_label = Some(label.into());
        self
    }

    pub fn capture_modifiers(mut self, capture: bool) -> Self {
        self.modifiers = capture;
        self
    }

    pub fn capture_mouse(mut self, capture: bool) -> Self {
        self.mouse = capture;
        self
    }

    pub fn capture_wheel(mut self, capture: bool) -> Self {
        self.wheel = capture;
        self
    }

    pub fn with_handle(mut self, handle: &KeyCaptureHandle) -> Self {
        self.handle = Some(handle.clone());
        self
    }

    pub fn on_capture(
        mut self,
        handler: impl Fn(&Keystroke, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_capture = Some(Rc::new(handler));
        self
    }

    pub fn on_input(
        mut self,
        handler: impl Fn(&CaptureInput, &mut Window, &mut App) -> CaptureFlow + 'static,
    ) -> Self {
        self.on_input = Some(Rc::new(handler));
        self
    }

    pub fn on_capturing_change(
        mut self,
        handler: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_capturing_change = Some(Rc::new(handler));
        self
    }
}

#[derive(Clone)]
struct Reporter {
    state: Entity<CaptureState>,
    on_capture: Option<ValueHandler<Keystroke>>,
    on_input: Option<CaptureHandler>,
    on_capturing_change: Option<ValueHandler<bool>>,
}

impl Reporter {
    fn set_capturing(&self, capturing: bool, window: &mut Window, cx: &mut App) {
        if self.state.read(cx).capturing == capturing {
            return;
        }
        set_capturing(&self.state, capturing, cx);
        if let Some(handler) = &self.on_capturing_change {
            handler(&capturing, window, cx);
        }
    }

    fn report(&self, input: CaptureInput, window: &mut Window, cx: &mut App) {
        if super::util::ControllerMode::is_active(cx) {
            return;
        }
        let flow = match (&self.on_input, &input) {
            (Some(handler), _) => handler(&input, window, cx),
            (None, CaptureInput::Key(keystroke)) => {
                if let Some(handler) = &self.on_capture {
                    handler(keystroke, window, cx);
                }
                CaptureFlow::Finish
            }
            (None, _) => CaptureFlow::Continue,
        };
        if flow == CaptureFlow::Finish {
            self.set_capturing(false, window, cx);
        }
    }
}

impl RenderOnce for KeyCapture {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| CaptureState {
            capturing: false,
            focus_handle: cx.focus_handle(),
        });
        if let Some(handle) = &self.handle {
            *handle.0.borrow_mut() = Some(state.clone());
        }
        let (was_capturing, focus_handle) = {
            let state = state.read(cx);
            (state.capturing, state.focus_handle.clone())
        };
        let reporter = Reporter {
            state: state.clone(),
            on_capture: self.on_capture,
            on_input: self.on_input,
            on_capturing_change: self.on_capturing_change,
        };
        let disabled = self.disabled || super::util::ControllerMode::is_active(cx);
        let capturing = was_capturing && focus_handle.is_focused(window) && !disabled;
        if was_capturing && !capturing {
            let reporter = reporter.clone();
            window.defer(cx, move |window, cx| {
                reporter.set_capturing(false, window, cx);
            });
        }
        let accent = theme.colors.accent_primary;
        let border = if self.invalid {
            theme.colors.error
        } else if capturing {
            accent
        } else {
            theme.colors.border_medium
        };
        let label = if capturing {
            self.capturing_label
                .unwrap_or_else(|| SharedString::new_static(cx.t().press_key()))
        } else {
            self.current
        };
        let click_focus = focus_handle.clone();
        let click_reporter = reporter.clone();
        let key_reporter = reporter.clone();
        let modifier_reporter = reporter.clone();
        let mouse_reporter = reporter.clone();
        let wheel_reporter = reporter;
        let capture_left = capturing && self.mouse;
        let hover_border = accent.opacity(theme.alpha.vignette);
        let hover_background = accent.opacity(theme.alpha.subtle);
        let line_height = window.line_height();
        h_flex()
            .id(self.id)
            .track_focus(&focus_handle)
            .flex_none()
            .justify_center()
            .min_w(self.min_width.unwrap_or(sizes.config_input_width))
            .px(px(10.))
            .py(px(7.))
            .rounded(Sizes::CONTROL_RADIUS)
            .border_1()
            .border_color(border)
            .bg(if capturing {
                accent.opacity(theme.alpha.hover)
            } else {
                theme.colors.input_background
            })
            .text_size(sizes.font_sm)
            .line_height(relative(LINE_HEIGHT_TIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(if capturing {
                accent
            } else {
                theme.colors.text_primary
            })
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .when(!capturing, |this| {
                        this.hover(move |style| {
                            style.bg(hover_background).border_color(hover_border)
                        })
                    })
                    .when(!capture_left, |this| {
                        this.on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            window.focus(&click_focus, cx);
                            let capturing = click_reporter.state.read(cx).capturing;
                            click_reporter.set_capturing(!capturing, window, cx);
                        })
                    })
                    .when(capturing, |this| {
                        this.on_key_down(move |event, window, cx| {
                            cx.stop_propagation();
                            key_reporter.report(
                                CaptureInput::Key(event.keystroke.clone()),
                                window,
                                cx,
                            );
                        })
                    })
                    .when(capturing && self.modifiers, |this| {
                        this.on_modifiers_changed(move |event, window, cx| {
                            modifier_reporter.report(
                                CaptureInput::Modifiers(event.modifiers),
                                window,
                                cx,
                            );
                        })
                    })
                    .when(capturing && self.mouse, |this| {
                        this.on_any_mouse_down(move |event, window, cx| {
                            cx.stop_propagation();
                            mouse_reporter.report(CaptureInput::Mouse(event.button), window, cx);
                        })
                    })
                    .when(capturing && self.wheel, |this| {
                        this.on_scroll(move |event, window, cx| {
                            cx.stop_propagation();
                            wheel_reporter.report(
                                CaptureInput::Wheel(event.delta.pixel_delta(line_height)),
                                window,
                                cx,
                            );
                        })
                    })
            })
            .child(Text::new(label))
    }
}

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, point, size};

    use super::*;
    use crate::components::test_support::open_root;

    fn field(
        handle: &KeyCaptureHandle,
        log: &Rc<RefCell<Vec<CaptureInput>>>,
        states: &Rc<RefCell<Vec<bool>>>,
    ) -> KeyCapture {
        let log = log.clone();
        let states = states.clone();
        KeyCapture::new("capture", "F1")
            .with_handle(handle)
            .capture_modifiers(true)
            .capture_mouse(true)
            .capture_wheel(true)
            .on_input(move |input, _, _| {
                log.borrow_mut().push(input.clone());
                match input {
                    CaptureInput::Key(_) => CaptureFlow::Finish,
                    _ => CaptureFlow::Continue,
                }
            })
            .on_capturing_change(move |capturing, _, _| states.borrow_mut().push(*capturing))
    }

    #[gpui::test]
    fn allowed_inputs_are_reported_until_the_handler_finishes(cx: &mut TestAppContext) {
        let handle = KeyCaptureHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let states = Rc::new(RefCell::new(Vec::new()));
        let (render_handle, render_log, render_states) =
            (handle.clone(), log.clone(), states.clone());
        let (_root, cx) = open_root(cx, move |_, _| {
            gpui::div()
                .p_4()
                .child(field(&render_handle, &render_log, &render_states))
                .into_any_element()
        });
        cx.simulate_resize(size(px(400.), px(200.)));
        cx.run_until_parked();
        let inside = point(px(40.), px(30.));
        cx.simulate_click(inside, Modifiers::default());
        cx.run_until_parked();
        cx.update(|_, cx| assert!(handle.is_capturing(cx)));
        cx.simulate_modifiers_change(Modifiers::control());
        cx.simulate_mouse_down(inside, MouseButton::Middle, Modifiers::default());
        cx.simulate_mouse_up(inside, MouseButton::Middle, Modifiers::default());
        cx.simulate_mouse_down(inside, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(inside, MouseButton::Left, Modifiers::default());
        cx.run_until_parked();
        cx.update(|_, cx| assert!(handle.is_capturing(cx)));
        cx.simulate_keystrokes("ctrl-k");
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!handle.is_capturing(cx)));
        let log = log.borrow();
        assert_eq!(log[0], CaptureInput::Modifiers(Modifiers::control()));
        assert_eq!(log[1], CaptureInput::Mouse(MouseButton::Middle));
        assert_eq!(log[2], CaptureInput::Mouse(MouseButton::Left));
        assert!(matches!(&log[3], CaptureInput::Key(keystroke) if keystroke.key == "k"));
        assert_eq!(*states.borrow(), vec![true, false]);
    }

    #[gpui::test]
    fn controller_scroll_does_not_change_an_active_binding(cx: &mut TestAppContext) {
        let handle = KeyCaptureHandle::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let states = Rc::new(RefCell::new(Vec::new()));
        let (render_handle, render_log, render_states) =
            (handle.clone(), log.clone(), states.clone());
        let (_root, cx) = open_root(cx, move |_, _| {
            gpui::div()
                .p_4()
                .child(field(&render_handle, &render_log, &render_states))
                .into_any_element()
        });
        cx.simulate_resize(size(px(400.), px(200.)));
        cx.run_until_parked();
        cx.update(|window, cx| handle.start(window, cx));
        cx.run_until_parked();
        cx.update(|window, cx| {
            super::super::util::ControllerMode::set_active(true, cx);
            window.dispatch_event(
                gpui::PlatformInput::Scroll(gpui::ScrollEvent {
                    position: point(px(40.), px(30.)),
                    delta: gpui::ScrollDelta::Lines(point(0., -3.)),
                    ..gpui::ScrollEvent::default()
                }),
                cx,
            );
        });
        cx.run_until_parked();
        assert!(log.borrow().is_empty());
        cx.update(|_, cx| assert!(!handle.is_capturing(cx)));
    }

    #[gpui::test]
    fn plain_capture_still_finishes_on_the_first_key(cx: &mut TestAppContext) {
        let handle = KeyCaptureHandle::new();
        let keys = Rc::new(RefCell::new(Vec::new()));
        let (render_handle, render_keys) = (handle.clone(), keys.clone());
        let (_root, cx) = open_root(cx, move |_, _| {
            let keys = render_keys.clone();
            gpui::div()
                .child(
                    KeyCapture::new("capture", "F1")
                        .with_handle(&render_handle)
                        .on_capture(move |keystroke, _, _| {
                            keys.borrow_mut().push(keystroke.key.clone());
                        }),
                )
                .into_any_element()
        });
        cx.run_until_parked();
        cx.update(|window, cx| handle.start(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("f2");
        cx.run_until_parked();
        assert_eq!(*keys.borrow(), vec!["f2".to_owned()]);
        cx.update(|_, cx| assert!(!handle.is_capturing(cx)));
    }
}
