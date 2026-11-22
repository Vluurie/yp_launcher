use std::cell::RefCell;
use std::rc::Rc;

use gpui::Subscription;
use yp_ui::prelude::*;
use yp_ui::{Input, InputEvent, InputSize, InputState};

type TextHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;

struct TextFieldState {
    input: Entity<InputState>,
    handler: Rc<RefCell<Option<TextHandler>>>,
    last_value: SharedString,
    _subscription: Subscription,
}

pub struct TextField {
    id: ElementId,
    value: SharedString,
    mono: bool,
    size: InputSize,
    disabled: bool,
    on_change: Option<TextHandler>,
}

yp_ui::impl_into_element!(TextField);

impl TextField {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            mono: false,
            size: InputSize::Small,
            disabled: false,
            on_change: None,
        }
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value = self.value.clone();
        let mono = self.mono;
        let state = window.use_keyed_state(self.id, cx, move |window, cx| {
            let input = cx.new(|cx| {
                let state = InputState::new(window, cx).default_value(value.clone());
                if mono { state.mono() } else { state }
            });
            let handler: Rc<RefCell<Option<TextHandler>>> = Rc::new(RefCell::new(None));
            let relay = handler.clone();
            let subscription = cx.subscribe_in(
                &input,
                window,
                move |_, input, event: &InputEvent, window, cx| {
                    if !matches!(event, InputEvent::Change) {
                        return;
                    }
                    let text = input.read(cx).value().trim().to_owned();
                    let handler = relay.borrow().clone();
                    if let Some(handler) = handler {
                        handler(&text, window, cx);
                    }
                },
            );
            TextFieldState {
                input,
                handler,
                last_value: value,
                _subscription: subscription,
            }
        });
        let (input, last_value) = {
            let state = state.read(cx);
            *state.handler.borrow_mut() = self.on_change;
            (state.input.clone(), state.last_value.clone())
        };
        let external_change = last_value != self.value;
        let focused = input.read(cx).is_focused(window);
        let shown = input.read(cx).value().trim() == self.value.as_ref();
        if !shown && (external_change || !focused) {
            let value = self.value.clone();
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        if external_change {
            let value = self.value.clone();
            state.update(cx, |state, _| state.last_value = value);
        }
        Input::new(&input)
            .size(self.size)
            .disabled(self.disabled)
            .width(relative(1.))
    }
}
