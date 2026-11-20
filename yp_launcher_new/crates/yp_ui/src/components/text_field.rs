use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    AnyElement, App, DefiniteLength, ElementId, Entity, IntoElement, RenderOnce, SharedString,
    Subscription, Window, prelude::*,
};

use super::input::{Input, InputEvent, InputSize, InputState, InputValidator};
use crate::traits::ValueHandler;

#[derive(Default)]
struct TextHandlers {
    on_change: Option<ValueHandler<SharedString>>,
    on_submit: Option<ValueHandler<SharedString>>,
    on_blur: Option<ValueHandler<SharedString>>,
}

struct TextFieldState {
    input: Entity<InputState>,
    handlers: Rc<RefCell<TextHandlers>>,
    last_value: SharedString,
    placeholder: Option<SharedString>,
    _subscription: Subscription,
}

pub struct TextField {
    id: ElementId,
    value: SharedString,
    placeholder: Option<SharedString>,
    max_length: Option<usize>,
    validator: Option<InputValidator>,
    mono: bool,
    disabled: bool,
    invalid: bool,
    cleanable: bool,
    size: InputSize,
    width: Option<DefiniteLength>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    handlers: TextHandlers,
}

crate::impl_into_element!(TextField);

impl TextField {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            placeholder: None,
            max_length: None,
            validator: None,
            mono: false,
            disabled: false,
            invalid: false,
            cleanable: false,
            size: InputSize::default(),
            width: None,
            prefix: None,
            suffix: None,
            handlers: TextHandlers::default(),
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn validate(
        mut self,
        validator: impl Fn(&str) -> Result<(), SharedString> + 'static,
    ) -> Self {
        self.validator = Some(Rc::new(validator));
        self
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
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

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handlers.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_submit(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handlers.on_submit = Some(Rc::new(handler));
        self
    }

    pub fn on_blur(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.handlers.on_blur = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let initial = self.value.clone();
        let max_length = self.max_length;
        let validator = self.validator;
        let mono = self.mono;
        let state = window.use_keyed_state(self.id, cx, move |window, cx| {
            let input = cx.new(|cx| {
                let mut state = InputState::new(window, cx).default_value(initial.clone());
                if mono {
                    state = state.mono();
                }
                if let Some(max_length) = max_length {
                    state = state.max_length(max_length);
                }
                if let Some(validator) = validator {
                    state = state.validate(move |text| validator(text));
                }
                state
            });
            let handlers = Rc::new(RefCell::new(TextHandlers::default()));
            let relay = handlers.clone();
            let subscription = cx.subscribe_in(
                &input,
                window,
                move |_, input, event: &InputEvent, window, cx| {
                    let handler = {
                        let handlers = relay.borrow();
                        match event {
                            InputEvent::Change => handlers.on_change.clone(),
                            InputEvent::Submit => handlers.on_submit.clone(),
                            InputEvent::Blur => handlers.on_blur.clone(),
                            InputEvent::Focus => None,
                        }
                    };
                    if let Some(handler) = handler {
                        let text = input.read(cx).text();
                        handler(&text, window, cx);
                    }
                },
            );
            TextFieldState {
                input,
                handlers,
                last_value: initial,
                placeholder: None,
                _subscription: subscription,
            }
        });
        let (input, last_value, last_placeholder) = {
            let state = state.read(cx);
            *state.handlers.borrow_mut() = self.handlers;
            (
                state.input.clone(),
                state.last_value.clone(),
                state.placeholder.clone(),
            )
        };
        let external_change = last_value != self.value;
        let focused = input.read(cx).is_focused(window);
        let shown = input.read(cx).value() == self.value.as_ref();
        if !shown && (external_change || !focused) {
            let value = self.value.clone();
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        if external_change {
            let value = self.value.clone();
            state.update(cx, |state, _| state.last_value = value);
        }
        if last_placeholder != self.placeholder {
            let placeholder = self.placeholder.clone();
            state.update(cx, |state, _| state.placeholder = placeholder);
            let placeholder = self.placeholder.unwrap_or_default();
            input.update(cx, |input, cx| input.set_placeholder(placeholder, cx));
        }
        Input::new(&input)
            .size(self.size)
            .disabled(self.disabled)
            .invalid(self.invalid)
            .cleanable(self.cleanable)
            .when_some(self.width, |this, width| this.width(width))
            .when_some(self.prefix, |this, prefix| this.prefix(prefix))
            .when_some(self.suffix, |this, suffix| this.suffix(suffix))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use gpui::TestAppContext;

    use super::*;
    use crate::components::test_support::open_root;

    #[gpui::test]
    fn a_text_field_reports_edits_and_follows_external_changes(cx: &mut TestAppContext) {
        let value = Rc::new(RefCell::new(SharedString::new_static("2B")));
        let changes = Rc::new(RefCell::new(Vec::new()));
        let submits = Rc::new(RefCell::new(Vec::new()));
        let (render_value, change_sink, submit_sink) =
            (value.clone(), changes.clone(), submits.clone());
        let (root, cx) = open_root(cx, move |_, _| {
            let change_sink = change_sink.clone();
            let change_value = render_value.clone();
            let submit_sink = submit_sink.clone();
            TextField::new("field", render_value.borrow().clone())
                .max_length(6)
                .on_change(move |text, _, _| {
                    *change_value.borrow_mut() = text.clone();
                    change_sink.borrow_mut().push(text.to_string());
                })
                .on_submit(move |text, _, _| submit_sink.borrow_mut().push(text.to_string()))
                .into_any_element()
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("tab");
        cx.simulate_input("9S");
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert_eq!(*changes.borrow(), vec!["2B9".to_owned(), "2B9S".to_owned()]);
        assert_eq!(*submits.borrow(), vec!["2B9S".to_owned()]);
        cx.simulate_input("0123456789");
        cx.run_until_parked();
        assert_eq!(value.borrow().as_ref(), "2B9S01");
        *value.borrow_mut() = SharedString::new_static("A2");
        root.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        cx.simulate_input("!");
        cx.run_until_parked();
        assert_eq!(value.borrow().as_ref(), "A2!");
    }
}
