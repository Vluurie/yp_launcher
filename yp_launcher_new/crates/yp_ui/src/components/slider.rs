use std::rc::Rc;

use gpui::{
    App, Bounds, DefiniteLength, ElementId, IntoElement, MouseButton, Pixels, RenderOnce,
    SharedString, Window, div, prelude::*, px, relative,
};

use super::util::{KeyboardFocus, bounds_tracker, pointer_capture};
use crate::theme::{ActiveTheme, Sizes};
use crate::traits::ValueHandler;

mod menu;

#[derive(Default)]
struct SliderState {
    dragging: bool,
    bounds: Bounds<Pixels>,
    drag_value: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SliderRange {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub ticks: Vec<f32>,
}

impl SliderRange {
    pub fn snap(&self, raw: f32) -> f32 {
        let (low, high) = if self.min <= self.max {
            (self.min, self.max)
        } else {
            (self.max, self.min)
        };
        if !raw.is_finite() {
            return low;
        }
        if !self.ticks.is_empty() {
            return self
                .ticks
                .iter()
                .copied()
                .filter(|tick| tick.is_finite())
                .min_by(|a, b| (a - raw).abs().total_cmp(&(b - raw).abs()))
                .unwrap_or(low);
        }
        let snapped = if self.step > 0. && self.step.is_finite() {
            (raw / self.step).round() * self.step
        } else {
            raw
        };
        snapped.clamp(low, high)
    }

    pub fn fraction(&self, value: f32) -> f32 {
        let span = self.max - self.min;
        if span.abs() <= f32::EPSILON || !value.is_finite() {
            return 0.;
        }
        ((value - self.min) / span).clamp(0., 1.)
    }

    pub fn value_at(&self, fraction: f32) -> f32 {
        let fraction = if fraction.is_finite() {
            fraction.clamp(0., 1.)
        } else {
            0.
        };
        self.snap(self.min + (self.max - self.min) * fraction)
    }

    fn adjacent(&self, value: f32, forward: bool) -> f32 {
        if !self.ticks.is_empty() {
            let next = self
                .ticks
                .iter()
                .copied()
                .filter(|tick| tick.is_finite())
                .filter(|tick| {
                    if forward {
                        *tick > value
                    } else {
                        *tick < value
                    }
                });
            return if forward {
                next.min_by(f32::total_cmp)
            } else {
                next.max_by(f32::total_cmp)
            }
            .unwrap_or(value);
        }
        let step = if self.step.is_finite() && self.step > 0.0 {
            self.step
        } else {
            (self.max - self.min).abs() / 100.0
        };
        self.snap(value + if forward { step } else { -step })
    }
}

fn accessible_range(
    mut el: gpui::Stateful<gpui::Div>,
    range: Rc<SliderRange>,
    value: f32,
    apply: impl Fn(f32, &mut Window, &mut App) + 'static,
) -> gpui::Stateful<gpui::Div> {
    let apply = Rc::new(apply);
    for action in [
        gpui::AccessibleAction::Increment,
        gpui::AccessibleAction::Decrement,
        gpui::AccessibleAction::SetValue,
    ] {
        let range = range.clone();
        let apply = apply.clone();
        el = el.on_a11y_action(action, move |data, w, cx| {
            let next = match (action, data) {
                (
                    gpui::AccessibleAction::SetValue,
                    Some(gpui::accesskit::ActionData::NumericValue(n)),
                ) if n.is_finite() => range.snap(*n as f32),
                (gpui::AccessibleAction::Increment, _) => range.adjacent(value, true),
                (gpui::AccessibleAction::Decrement, _) => range.adjacent(value, false),
                _ => return,
            };
            apply(next, w, cx);
        });
    }
    el
}

pub struct Slider {
    id: ElementId,
    accessible_label: Option<SharedString>,
    value: f32,
    default_mark: Option<f32>,
    menu_row: Option<(SharedString, SharedString)>,
    range: SliderRange,
    disabled: bool,
    width: Option<DefiniteLength>,
    on_change: Option<ValueHandler<f32>>,
    on_commit: Option<ValueHandler<f32>>,
}

crate::impl_into_element!(Slider);

impl Slider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            accessible_label: None,
            value: 0.,
            default_mark: None,
            menu_row: None,
            range: SliderRange {
                min: 0.,
                max: 1.,
                step: 0.,
                ticks: Vec::new(),
            },
            disabled: false,
            width: None,
            on_change: None,
            on_commit: None,
        }
    }

    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub(super) fn menu_row(mut self, label: SharedString, value_label: SharedString) -> Self {
        self.menu_row = Some((label, value_label));
        self
    }

    pub fn default_mark(mut self, value: f32) -> Self {
        self.default_mark = value.is_finite().then_some(value);
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.range.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.range.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.range.step = step;
        self
    }

    pub fn ticks(mut self, ticks: Vec<f32>) -> Self {
        self.range.ticks = ticks;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_commit(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_commit = Some(Rc::new(handler));
        self
    }
}

fn value_for_position(range: &SliderRange, bounds: Bounds<Pixels>, x: Pixels) -> f32 {
    let thumb = Sizes::SLIDER_THUMB_SIZE;
    let usable = bounds.size.width - thumb;
    if usable <= px(0.) {
        return range.snap(range.min);
    }
    let offset = x - bounds.left() - thumb / 2.;
    range.value_at(offset / usable)
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        if theme.id == crate::ThemeId::NierNew {
            return menu::render(self, window, cx);
        }
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| SliderState::default());
        let (dragging, drag_value) = {
            let state = state.read(cx);
            (state.dragging, state.drag_value)
        };
        let shown = if dragging {
            drag_value.unwrap_or(self.value)
        } else {
            self.value
        };
        let fraction = self.range.fraction(shown);
        let accent = theme.colors.accent_primary;
        let thumb = Sizes::SLIDER_THUMB_SIZE;
        let halo = thumb + px(14.);
        let group = SharedString::from(format!("yp-slider-{}", self.id));
        let disabled = self.disabled;
        let range = Rc::new(self.range);
        let on_change = self.on_change;
        let on_commit = self.on_commit;

        let bounds_state = state.clone();
        let down_state = state.clone();
        let down_range = range.clone();
        let down_change = on_change.clone();
        let move_state = state.clone();
        let move_range = range.clone();
        let move_change = on_change.clone();
        let up_state = state.clone();
        let release_range = range.clone();
        let release_change = on_change.clone();
        let fallback = self.value;
        let keyboard_range = range.clone();
        let keyboard_change = on_change.clone();
        let keyboard_commit = on_commit.clone();

        div()
            .id(self.id)
            .role(gpui::Role::Slider)
            .when_some(self.accessible_label, |el, label| el.aria_label(label))
            .aria_numeric_value(shown.into())
            .aria_orientation(gpui::Orientation::Horizontal)
            .when(range.step > 0., |el| {
                el.aria_numeric_value_step(range.step.into())
            })
            .when(!disabled, |el| {
                let change = on_change.clone();
                let commit = on_commit.clone();
                accessible_range(el, range.clone(), shown, move |next, w, cx| {
                    if let Some(f) = &change {
                        f(&next, w, cx);
                    }
                    if let Some(f) = &commit {
                        f(&next, w, cx);
                    }
                })
            })
            .aria_min_numeric_value(range.min.min(range.max).into())
            .aria_max_numeric_value(range.max.max(range.min).into())
            .group(group.clone())
            .relative()
            .h(Sizes::SLIDER_HIT_HEIGHT)
            .flex()
            .items_center()
            .map(|this| match self.width {
                Some(width) => this.w(width),
                None => this.w_full(),
            })
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .keyboard_focus(cx)
                    .on_key_down(move |event, window, cx| {
                        let value = match event.keystroke.key.as_str() {
                            "left" => keyboard_range.adjacent(fallback, false),
                            "right" => keyboard_range.adjacent(fallback, true),
                            "home" => keyboard_range.snap(keyboard_range.min),
                            "end" => keyboard_range.snap(keyboard_range.max),
                            _ => return,
                        };
                        cx.stop_propagation();
                        window.prevent_default();
                        if value != fallback {
                            if let Some(handler) = &keyboard_change {
                                handler(&value, window, cx);
                            }
                            if let Some(handler) = &keyboard_commit {
                                handler(&value, window, cx);
                            }
                        }
                    })
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        cx.stop_propagation();
                        let bounds = down_state.read(cx).bounds;
                        let value = value_for_position(&down_range, bounds, event.position.x);
                        down_state.update(cx, |state, cx| {
                            state.dragging = true;
                            state.drag_value = Some(value);
                            cx.notify();
                        });
                        if let Some(handler) = &down_change {
                            handler(&value, window, cx);
                        }
                    })
            })
            .child(bounds_tracker(move |bounds, _, cx| {
                if bounds_state.read(cx).bounds != bounds {
                    bounds_state.update(cx, |state, _| state.bounds = bounds);
                }
            }))
            .child(pointer_capture(
                !disabled,
                move |event, window, cx| {
                    let (bounds, previous) = {
                        let state = move_state.read(cx);
                        if !state.dragging {
                            return;
                        }
                        (state.bounds, state.drag_value)
                    };
                    let value = value_for_position(&move_range, bounds, event.position.x);
                    if previous == Some(value) {
                        return;
                    }
                    move_state.update(cx, |state, cx| {
                        state.drag_value = Some(value);
                        cx.notify();
                    });
                    if let Some(handler) = &move_change {
                        handler(&value, window, cx);
                    }
                },
                move |event, window, cx| {
                    let state = up_state.read(cx);
                    if event.button != MouseButton::Left || !state.dragging {
                        return;
                    }
                    let value = value_for_position(&release_range, state.bounds, event.position.x);
                    let previous = up_state.update(cx, |state, cx| {
                        state.dragging = false;
                        cx.notify();
                        state.drag_value.take()
                    });
                    if previous != Some(value) && let Some(handler) = &release_change {
                        handler(&value, window, cx);
                    }
                    if let Some(handler) = &on_commit {
                        handler(&value, window, cx);
                    }
                },
            ))
            .child(
                div()
                    .w_full()
                    .mx(thumb / 2.)
                    .h(Sizes::SLIDER_TRACK_HEIGHT)
                    .rounded(Sizes::SLIDER_TRACK_HEIGHT)
                    .bg(theme.colors.border_light)
                    .child(
                        div()
                            .h_full()
                            .w(relative(fraction))
                            .rounded(Sizes::SLIDER_TRACK_HEIGHT)
                            .bg(accent),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(thumb / 2.)
                    .right(thumb / 2.)
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(relative(fraction))
                            .w(px(0.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .flex_none()
                                    .size(halo)
                                    .rounded(halo)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(dragging, |this| {
                                        this.bg(accent.opacity(theme.alpha.hover))
                                    })
                                    .when(!disabled && !dragging, |this| {
                                        let hover = accent.opacity(theme.alpha.hover);
                                        this.group_hover(group, move |style| style.bg(hover))
                                    })
                                    .child(div().flex_none().size(thumb).rounded(thumb).bg(accent)),
                            ),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(min: f32, max: f32, step: f32) -> SliderRange {
        SliderRange {
            min,
            max,
            step,
            ticks: Vec::new(),
        }
    }

    #[test]
    fn snap_rounds_to_step_and_clamps() {
        let range = range(0., 10., 0.5);
        assert_eq!(range.snap(3.26), 3.5);
        assert_eq!(range.snap(-4.), 0.);
        assert_eq!(range.snap(99.), 10.);
        assert_eq!(range.snap(f32::NAN), 0.);
    }

    #[test]
    fn snap_prefers_the_nearest_tick() {
        let mut range = range(0., 8192., 0.);
        range.ticks = vec![1024., 2048., 4096., 8192.];
        assert_eq!(range.snap(3000.), 2048.);
        assert_eq!(range.snap(3200.), 4096.);
        assert_eq!(range.snap(-1.), 1024.);
    }

    #[test]
    fn keyboard_steps_respect_limits_and_nonuniform_ticks() {
        let range = range(0., 10., 0.5);
        assert_eq!(range.adjacent(1., true), 1.5);
        assert_eq!(range.adjacent(0., false), 0.);
        assert_eq!(range.adjacent(10., true), 10.);
        let range = SliderRange {
            min: 0.,
            max: 8192.,
            step: 0.,
            ticks: vec![8192., 1024., 4096., f32::NAN, 2048.],
        };
        assert_eq!(range.adjacent(2048., true), 4096.);
        assert_eq!(range.adjacent(2048., false), 1024.);
        assert_eq!(range.adjacent(8192., true), 8192.);
    }

    #[test]
    fn fraction_handles_empty_and_inverted_ranges() {
        assert_eq!(range(5., 5., 1.).fraction(5.), 0.);
        assert_eq!(range(0., 4., 1.).fraction(1.), 0.25);
        assert_eq!(range(0., 4., 1.).fraction(40.), 1.);
        assert_eq!(range(4., 0., 1.).snap(9.), 4.);
    }

    #[test]
    fn position_maps_to_values_inside_the_track() {
        let range = range(0., 100., 1.);
        let bounds = Bounds::new(gpui::point(px(100.), px(0.)), gpui::size(px(214.), px(32.)));
        assert_eq!(value_for_position(&range, bounds, px(0.)), 0.);
        assert_eq!(value_for_position(&range, bounds, px(207.)), 50.);
        assert_eq!(value_for_position(&range, bounds, px(900.)), 100.);
        let empty = Bounds::new(gpui::point(px(0.), px(0.)), gpui::size(px(4.), px(32.)));
        assert_eq!(value_for_position(&range, empty, px(2.)), 0.);
    }
}
