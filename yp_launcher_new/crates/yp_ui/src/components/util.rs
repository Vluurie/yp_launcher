use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    App, Bounds, BoxShadow, Context, DispatchPhase, Div, ElementId, Entity, FocusHandle, Global,
    HitboxBehavior, IntoElement, MouseMoveEvent, MouseUpEvent, Pixels, Stateful, Window, canvas,
    point, prelude::*, px,
};

use crate::actions::ACTIVATION_CONTEXT;
use crate::theme::{ActiveTheme, Sizes};

pub(crate) fn pointer_capture(
    active: bool,
    on_move: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
    on_up: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            if !active {
                return;
            }
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Capture {
                    on_move(event, window, cx);
                }
            });
            window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Capture {
                    on_up(event, window, cx);
                }
            });
        },
    )
    .absolute()
    .size_full()
}

pub(crate) fn bounds_tracker(
    on_bounds: impl FnOnce(Bounds<Pixels>, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    canvas(
        move |bounds, window, cx| on_bounds(bounds, window, cx),
        |_, _, _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

type LinkOpenerFn = Rc<dyn Fn(&str, &mut App)>;

struct LinkOpener(LinkOpenerFn);

impl Global for LinkOpener {}

pub fn set_link_opener(opener: impl Fn(&str, &mut App) + 'static, cx: &mut App) {
    cx.set_global(LinkOpener(Rc::new(opener)));
}

pub fn open_link(url: &str, cx: &mut App) {
    let opener = cx.try_global::<LinkOpener>().map(|opener| opener.0.clone());
    match opener {
        Some(opener) => opener(url, cx),
        None => cx.open_url(url),
    }
}

#[derive(Default)]
pub struct HoverState {
    hovered: bool,
    changes: usize,
    change_started: Option<std::time::Instant>,
    from: f32,
}

impl HoverState {
    pub fn hovered(&self) -> bool {
        self.hovered
    }

    pub fn changes(&self) -> usize {
        self.changes
    }

    fn fill_fraction(&self) -> f32 {
        let target = u8::from(self.hovered) as f32;
        let Some(start) = self.change_started else {
            return target;
        };
        let t = (start.elapsed().as_secs_f32() / 0.19).min(1.);
        target + (self.from - target) * (1. - t).powi(3)
    }

    fn set(&mut self, hovered: bool, cx: &mut Context<Self>) {
        if self.hovered != hovered {
            self.from = self.fill_fraction();
            self.change_started = Some(std::time::Instant::now());
            self.hovered = hovered;
            self.changes += 1;
            cx.notify();
        }
    }
}

pub fn use_hover_state(
    id: impl Into<ElementId>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<HoverState> {
    window.use_keyed_state(id.into(), cx, |_, _| HoverState::default())
}

pub fn menu_hover_fill(
    id: ElementId,
    color: gpui::Hsla,
    foreground: Option<gpui::AnyElement>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement + use<> {
    let state = use_hover_state((id, "menu-fill"), window, cx);
    let target = u8::from(state.read(cx).hovered) as f32;
    let progress = if yp_nier_ui::ReducedMotion::enabled(cx) {
        target
    } else {
        state.read(cx).fill_fraction()
    };
    if (progress - target).abs() > 0.001 {
        window.request_animation_frame();
    }
    gpui::div().absolute().inset_0()
        .when(progress > 0.001, |layer| {
            let fill = gpui::div().absolute().left_0().top_0().bottom_0()
                .w(gpui::relative(progress)).bg(color).overflow_hidden()
                .when_some(foreground, |band, content| band.child(
                    gpui::div().absolute().left_0().top_0().bottom_0()
                        .w(gpui::relative(1. / progress)).child(content)
                ));
            layer.child(super::root::layer("hover-colors", true, fill))
        })
        .child(hover_probe(&state))
}

pub fn hover_probe(state: &Entity<HoverState>) -> impl IntoElement {
    let state = state.clone();
    canvas(
        |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
        move |_, hitbox, window, cx| {
            let hovered = !cx.has_active_drag() && hitbox.is_hovered(window);
            if state.read(cx).hovered != hovered {
                let state = state.clone();
                window.defer(cx, move |_, cx| {
                    state.update(cx, |state, cx| state.set(hovered, cx));
                });
            }
            window.on_mouse_event(move |_: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Capture {
                    return;
                }
                let hovered = !cx.has_active_drag() && hitbox.is_hovered(window);
                if state.read(cx).hovered != hovered {
                    state.update(cx, |state, cx| state.set(hovered, cx));
                }
            });
        },
    )
    .absolute()
    .inset_0()
}

#[derive(Default, Clone, Copy)]
pub struct ControllerMode {
    active: bool,
    injecting: bool,
}

impl Global for ControllerMode {}

impl ControllerMode {
    pub fn is_active(cx: &App) -> bool {
        cx.try_global::<Self>().is_some_and(|mode| mode.active)
    }

    pub fn set_active(active: bool, cx: &mut App) -> bool {
        if Self::is_active(cx) == active {
            return false;
        }
        let injecting = cx.try_global::<Self>().is_some_and(|mode| mode.injecting);
        cx.set_global(Self { active, injecting });
        cx.refresh_windows();
        true
    }

    pub fn synthetic_input<R>(cx: &mut App, input: impl FnOnce(&mut App) -> R) -> R {
        let previous = cx.global::<Self>().injecting;
        cx.global_mut::<Self>().injecting = true;
        let result = input(cx);
        cx.global_mut::<Self>().injecting = previous;
        result
    }
}

#[derive(Default, Clone, Copy)]
pub struct FocusVisible(bool);

impl Global for FocusVisible {}

impl FocusVisible {
    const KEYS: [&'static str; 9] = [
        "tab", "up", "down", "left", "right", "home", "end", "enter", "space",
    ];

    pub(crate) fn init(cx: &mut App) {
        if cx.try_global::<FocusVisible>().is_some() {
            return;
        }
        cx.set_global(FocusVisible::default());
        cx.set_global(ControllerMode::default());
        cx.set_global(KeyClickGuard::default());
        cx.intercept_keystrokes(|event, window, cx| {
            if !cx.global::<ControllerMode>().injecting {
                ControllerMode::set_active(false, cx);
            }
            let key = event.keystroke.key.as_str();
            if Self::KEYS.contains(&key) {
                FocusVisible::set(cx, true);
            }
            KeyClickGuard::record(cx, key, window.focused(cx));
        })
        .detach();
    }

    pub fn get(cx: &App) -> bool {
        cx.try_global::<FocusVisible>()
            .is_some_and(|visible| visible.0)
    }

    pub fn set(cx: &mut App, visible: bool) {
        if cx.has_global::<FocusVisible>() && Self::get(cx) != visible {
            cx.set_global(FocusVisible(visible));
        }
    }
}

type PressedKey = (String, Option<FocusHandle>);

#[derive(Default, Clone)]
pub(crate) struct KeyClickGuard(Rc<RefCell<Option<PressedKey>>>);

impl Global for KeyClickGuard {}

impl KeyClickGuard {
    fn record(cx: &App, key: &str, focus: Option<FocusHandle>) {
        if let Some(guard) = cx.try_global::<KeyClickGuard>() {
            *guard.0.borrow_mut() = Some((key.to_owned(), focus));
        }
    }

    pub(crate) fn allows(cx: &App, key: &str, focus: Option<FocusHandle>) -> bool {
        let Some(guard) = cx.try_global::<KeyClickGuard>() else {
            return true;
        };
        let pressed = guard.0.borrow_mut().take();
        pressed.is_some_and(|(pressed_key, pressed_focus)| {
            pressed_key == key && pressed_focus == focus
        })
    }
}

pub(crate) fn focus_ring_shadow(cx: &App) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: cx.theme().style.focus_ring,
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: Sizes::FOCUS_RING_WIDTH,
        inset: false,
    }]
}

pub trait KeyboardFocus: Sized {
    fn keyboard_focus(self, cx: &App) -> Self;

    fn keyboard_activate(self, cx: &App) -> Self;

    fn on_disclosure_toggle(
        self,
        open: bool,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self;
}

impl KeyboardFocus for Stateful<Div> {
    fn keyboard_focus(self, cx: &App) -> Self {
        let visible = FocusVisible::get(cx);
        let ring = focus_ring_shadow(cx);
        self.tab_index(0).when(visible, move |this| {
            this.focus(move |style| style.shadow(ring))
        })
    }

    fn on_disclosure_toggle(
        self,
        open: bool,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        let action = if open { gpui::AccessibleAction::Collapse } else { gpui::AccessibleAction::Expand };
        let handler = Rc::new(handler);
        let accessible = handler.clone();
        self.on_a11y_action(action, move |_, w, cx| accessible(&gpui::ClickEvent::default(), w, cx))
            .on_click(move |event, w, cx| handler(event, w, cx))
    }

    fn keyboard_activate(self, cx: &App) -> Self {
        self.keyboard_focus(cx).key_context(ACTIVATION_CONTEXT)
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Modifiers, TestAppContext};

    use super::*;
    use crate::components::test_support::open_empty_root;

    #[gpui::test]
    fn focus_rings_follow_the_last_input_device(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        cx.update(|_, cx| assert!(!FocusVisible::get(cx)));
        cx.simulate_keystrokes("tab");
        cx.update(|_, cx| assert!(FocusVisible::get(cx)));
        cx.simulate_click(point(px(20.), px(20.)), Modifiers::default());
        cx.update(|_, cx| assert!(!FocusVisible::get(cx)));
        cx.simulate_keystrokes("a");
        cx.update(|_, cx| assert!(!FocusVisible::get(cx)));
    }
}
