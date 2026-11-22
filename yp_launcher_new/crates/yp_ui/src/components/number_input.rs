use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    App, Context, DefiniteLength, ElementId, Entity, IntoElement, RenderOnce, SharedString,
    Subscription, Window, div, prelude::*, relative,
};

use super::button::{ButtonSize, IconButton};
use super::icon::{IconName, IconSize};
use super::input::{Input, InputEvent, InputSize, InputState};
use super::tooltip::Tooltip;
use crate::i18n::ActiveStrings;
use crate::styled_ext::h_flex;
use crate::theme::{Color, ResponsiveExt};
use crate::traits::{ButtonCommon, Clickable, Disableable, ValueHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NumberKind {
    #[default]
    Int,
    Hex,
    Float {
        decimals: usize,
    },
}

impl NumberKind {
    pub fn parse(self, text: &str) -> Option<f64> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        match self {
            NumberKind::Int | NumberKind::Hex => parse_integer(text).map(|value| value as f64),
            NumberKind::Float { .. } => text.parse::<f64>().ok().filter(|value| value.is_finite()),
        }
    }

    pub fn format(self, value: f64) -> String {
        match self {
            NumberKind::Int => format!("{}", value.round() as i64),
            NumberKind::Hex => {
                let value = value.round() as i64;
                if value == 0 {
                    "0".to_owned()
                } else if value < 0 {
                    format!("-0x{:X}", value.unsigned_abs())
                } else {
                    format!("0x{value:X}")
                }
            }
            NumberKind::Float { decimals } => {
                if !value.is_finite() {
                    return "0.0".to_owned();
                }
                let rounded = round_to(value, decimals.max(1));
                if rounded == rounded.round() {
                    format!("{rounded:.1}")
                } else {
                    format!("{rounded}")
                }
            }
        }
    }
}

fn round_to(value: f64, decimals: usize) -> f64 {
    let factor = 10f64.powi(i32::try_from(decimals.min(12)).unwrap_or(12));
    (value * factor).round() / factor
}

fn parse_integer(text: &str) -> Option<i64> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let magnitude = match digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        Some(hex) => i64::from_str_radix(hex, 16).ok()?,
        None => digits.parse::<i64>().ok()?,
    };
    Some(if negative { -magnitude } else { magnitude })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberBounds {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl NumberBounds {
    pub fn clamp(&self, value: f64) -> f64 {
        let value = match self.min {
            Some(min) if value < min => min,
            _ => value,
        };
        match self.max {
            Some(max) if value > max => max,
            _ => value,
        }
    }
}

#[derive(Clone)]
struct NumberConfig {
    kind: NumberKind,
    bounds: NumberBounds,
    on_change: Option<ValueHandler<f64>>,
    on_commit: Option<ValueHandler<f64>>,
}

struct NumberBinding {
    config: Rc<RefCell<NumberConfig>>,
    _subscription: Subscription,
}

impl NumberBinding {
    fn new(
        input: &Entity<InputState>,
        config: NumberConfig,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = Rc::new(RefCell::new(config));
        let handler_config = config.clone();
        let subscription = cx.subscribe_in(
            input,
            window,
            move |_, input, event: &InputEvent, window, cx| {
                let config = handler_config.borrow().clone();
                let parsed = config.kind.parse(input.read(cx).value());
                match event {
                    InputEvent::Change => {
                        let error = parsed
                            .is_none()
                            .then(|| SharedString::new_static(cx.t().enter_valid_number()));
                        input.update(cx, |input, cx| input.set_error(error, cx));
                        if let (Some(value), Some(handler)) = (parsed, config.on_change.as_ref()) {
                            handler(&config.bounds.clamp(value), window, cx);
                        }
                    }
                    InputEvent::Submit | InputEvent::Blur => {
                        let Some(value) = parsed else {
                            return;
                        };
                        let value = config.bounds.clamp(value);
                        let formatted = config.kind.format(value);
                        input.update(cx, |input, cx| input.set_value(formatted, window, cx));
                        if let Some(handler) = config.on_commit.as_ref() {
                            handler(&value, window, cx);
                        }
                    }
                    InputEvent::Focus => {}
                }
            },
        );
        Self {
            config,
            _subscription: subscription,
        }
    }
}

pub struct NumberInput {
    state: Entity<InputState>,
    kind: NumberKind,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    steppers: bool,
    disabled: bool,
    size: InputSize,
    width: Option<DefiniteLength>,
    on_change: Option<ValueHandler<f64>>,
    on_commit: Option<ValueHandler<f64>>,
}

crate::impl_into_element!(NumberInput);

impl NumberInput {
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            kind: NumberKind::default(),
            min: None,
            max: None,
            step: 1.,
            steppers: false,
            disabled: false,
            size: InputSize::default(),
            width: None,
            on_change: None,
            on_commit: None,
        }
    }

    pub fn kind(mut self, kind: NumberKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn min(mut self, min: impl Into<Option<f64>>) -> Self {
        self.min = min.into();
        self
    }

    pub fn max(mut self, max: impl Into<Option<f64>>) -> Self {
        self.max = max.into();
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn steppers(mut self, steppers: bool) -> Self {
        self.steppers = steppers;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_commit(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_commit = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for NumberInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let config = NumberConfig {
            kind: self.kind,
            bounds: NumberBounds {
                min: self.min,
                max: self.max,
            },
            on_change: self.on_change,
            on_commit: self.on_commit,
        };
        let input = self.state.clone();
        let initial = config.clone();
        let binding = window.use_keyed_state(
            ("yp-number-input", self.state.entity_id()),
            cx,
            move |window, cx| NumberBinding::new(&input, initial, window, cx),
        );
        *binding.read(cx).config.borrow_mut() = config.clone();

        let current = self.kind.parse(self.state.read(cx).value());
        let step = if self.step.is_finite() && self.step != 0. {
            self.step
        } else {
            1.
        };
        let disabled = self.disabled;
        let stepper = |id: &'static str, icon: IconName, delta: f64, tooltip: &'static str| {
            let state = self.state.clone();
            let config = config.clone();
            IconButton::new(id, icon)
                .icon_size(IconSize::XSmall)
                .icon_color(Color::Muted)
                .size(ButtonSize::Compact)
                .disabled(disabled)
                .tooltip(Tooltip::text(tooltip))
                .on_click(move |_, window, cx| {
                    let base = current.unwrap_or(0.);
                    let next = config.bounds.clamp(match config.kind {
                        NumberKind::Float { decimals } => round_to(base + delta, decimals.max(1)),
                        NumberKind::Int | NumberKind::Hex => (base + delta).round(),
                    });
                    let formatted = config.kind.format(next);
                    state.update(cx, |state, cx| {
                        state.set_value(formatted, window, cx);
                        state.set_error(None, cx);
                    });
                    if let Some(handler) = config.on_change.as_ref() {
                        handler(&next, window, cx);
                    }
                    if let Some(handler) = config.on_commit.as_ref() {
                        handler(&next, window, cx);
                    }
                })
        };
        let controller_mode = super::util::ControllerMode::is_active(cx);
        let input = Input::new(&self.state)
            .size(self.size)
            .disabled(disabled)
            .when(self.steppers && !controller_mode, |this| {
                this.suffix(
                    h_flex()
                        .gap(sizes.spacing_xs)
                        .child(stepper(
                            "yp-number-decrease",
                            IconName::Remove,
                            -step,
                            strings.ui_decrease(),
                        ))
                        .child(stepper(
                            "yp-number-increase",
                            IconName::Add,
                            step,
                            strings.ui_increase(),
                        )),
                )
            });
        let width = self.width.unwrap_or_else(|| sizes.config_input_width.into());
        if controller_mode && self.steppers {
            h_flex()
                .id(("yp-number-controller", self.state.entity_id()))
                .w(width)
                .gap(sizes.spacing_xs)
                .child(div().flex_1().min_w_0().child(input.width(relative(1.))))
                .child(stepper("yp-number-decrease", IconName::Remove, -step, strings.ui_decrease()))
                .child(stepper("yp-number-increase", IconName::Add, step, strings.ui_increase()))
                .into_any_element()
        } else {
            input.width(width).into_any_element()
        }
    }
}

struct NumberFieldState {
    input: Entity<InputState>,
    last_value: f64,
}

pub struct NumberField {
    id: ElementId,
    value: f64,
    kind: NumberKind,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    steppers: bool,
    disabled: bool,
    size: InputSize,
    width: Option<DefiniteLength>,
    on_change: Option<ValueHandler<f64>>,
    on_commit: Option<ValueHandler<f64>>,
}

crate::impl_into_element!(NumberField);

impl NumberField {
    pub fn new(id: impl Into<ElementId>, value: f64) -> Self {
        Self {
            id: id.into(),
            value,
            kind: NumberKind::default(),
            min: None,
            max: None,
            step: 1.,
            steppers: false,
            disabled: false,
            size: InputSize::default(),
            width: None,
            on_change: None,
            on_commit: None,
        }
    }

    pub fn kind(mut self, kind: NumberKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn min(mut self, min: impl Into<Option<f64>>) -> Self {
        self.min = min.into();
        self
    }

    pub fn max(mut self, max: impl Into<Option<f64>>) -> Self {
        self.max = max.into();
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn steppers(mut self, steppers: bool) -> Self {
        self.steppers = steppers;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_commit(mut self, handler: impl Fn(&f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_commit = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for NumberField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let kind = self.kind;
        let value = self.value;
        let state = window.use_keyed_state(self.id, cx, move |window, cx| NumberFieldState {
            input: cx.new(|cx| InputState::new(window, cx).default_value(kind.format(value))),
            last_value: value,
        });
        let (input, last_value) = {
            let state = state.read(cx);
            (state.input.clone(), state.last_value)
        };
        let shown = kind.parse(input.read(cx).value());
        let external_change = last_value != value;
        let focused = input.read(cx).is_focused(window);
        if shown != Some(value) && (external_change || !focused) {
            input.update(cx, |input, cx| {
                input.set_value(kind.format(value), window, cx);
                input.set_error(None, cx);
            });
        }
        if external_change {
            state.update(cx, |state, _| state.last_value = value);
        }
        NumberInput::new(&input)
            .kind(kind)
            .min(self.min)
            .max(self.max)
            .step(self.step)
            .steppers(self.steppers)
            .disabled(self.disabled)
            .size(self.size)
            .when_some(self.width, |this, width| this.width(width))
            .when_some(self.on_change, |this, handler| {
                this.on_change(move |value, window, cx| handler(value, window, cx))
            })
            .when_some(self.on_commit, |this, handler| {
                this.on_commit(move |value, window, cx| handler(value, window, cx))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_accept_signs_and_hex_like_dart() {
        assert_eq!(NumberKind::Int.parse("42"), Some(42.));
        assert_eq!(NumberKind::Int.parse(" -7 "), Some(-7.));
        assert_eq!(NumberKind::Int.parse("0x1F"), Some(31.));
        assert_eq!(NumberKind::Hex.parse("-0X10"), Some(-16.));
        assert_eq!(NumberKind::Int.parse("1.5"), None);
        assert_eq!(NumberKind::Int.parse(""), None);
        assert_eq!(NumberKind::Int.parse("0x"), None);
        assert_eq!(NumberKind::Int.parse("99999999999999999999"), None);
    }

    #[test]
    fn floats_reject_text_and_non_finite_values() {
        let kind = NumberKind::Float { decimals: 2 };
        assert_eq!(kind.parse("1.25"), Some(1.25));
        assert_eq!(kind.parse("-3"), Some(-3.));
        assert_eq!(kind.parse("abc"), None);
        assert_eq!(kind.parse("inf"), None);
        assert_eq!(kind.parse("NaN"), None);
    }

    #[test]
    fn formatting_matches_the_flutter_fields() {
        assert_eq!(NumberKind::Int.format(12.4), "12");
        assert_eq!(NumberKind::Hex.format(255.), "0xFF");
        assert_eq!(NumberKind::Hex.format(0.), "0");
        assert_eq!(NumberKind::Hex.format(-16.), "-0x10");
        let float = NumberKind::Float { decimals: 3 };
        assert_eq!(float.format(2.), "2.0");
        assert_eq!(float.format(0.125), "0.125");
        assert_eq!(float.format(0.1 + 0.2), "0.3");
        assert_eq!(float.format(f64::NAN), "0.0");
    }

    #[test]
    fn bounds_clamp_only_the_sides_that_are_set() {
        let bounds = NumberBounds {
            min: Some(1.),
            max: None,
        };
        assert_eq!(bounds.clamp(-5.), 1.);
        assert_eq!(bounds.clamp(500.), 500.);
        let both = NumberBounds {
            min: Some(0.),
            max: Some(10.),
        };
        assert_eq!(both.clamp(11.), 10.);
    }
}

#[cfg(test)]
mod field_tests {
    use std::cell::{Cell, RefCell};

    use gpui::TestAppContext;

    use super::*;
    use crate::components::test_support::open_root;

    #[gpui::test]
    fn a_number_field_reports_edits_and_follows_external_changes(cx: &mut TestAppContext) {
        let value = Rc::new(Cell::new(5.));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let commits = Rc::new(RefCell::new(Vec::new()));
        let render_value = value.clone();
        let change_sink = changes.clone();
        let commit_sink = commits.clone();
        let (root, cx) = open_root(cx, move |_, _| {
            let change_sink = change_sink.clone();
            let commit_sink = commit_sink.clone();
            NumberField::new("field", render_value.get())
                .kind(NumberKind::Int)
                .min(0.)
                .max(60.)
                .on_change(move |value, _, _| change_sink.borrow_mut().push(*value))
                .on_commit(move |value, _, _| commit_sink.borrow_mut().push(*value))
                .into_any_element()
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab");
        cx.simulate_input("0");
        assert_eq!(*changes.borrow(), vec![50.]);
        cx.simulate_input("0");
        assert_eq!(*changes.borrow(), vec![50., 60.]);
        cx.simulate_input("x");
        assert_eq!(changes.borrow().len(), 2);
        cx.simulate_keystrokes("backspace backspace enter");
        assert_eq!(*commits.borrow(), vec![50.]);

        value.set(7.);
        root.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        cx.simulate_input("1");
        assert_eq!(changes.borrow().last().copied(), Some(60.));
    }
}
