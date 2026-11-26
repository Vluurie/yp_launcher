use futures::channel::oneshot;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render,
    SharedString, Subscription, Task, Window, div, prelude::*, relative,
};

use super::button::{Button, ButtonStyle, TintColor};
use super::button_extra::ChoiceCard;
use super::icon::IconName;
use super::input::{Input, InputEvent, InputState, InputValidator};
use super::label::{LINE_HEIGHT_BODY, Label, LabelSize};
use super::modal::{Modal, ModalFooter, ModalHeader, ModalView};
use super::root::WindowExt;
use super::toggle::Checkbox;
use crate::actions::Confirm;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt};
use crate::traits::{ButtonCommon, Clickable, Disableable, ToggleState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ConfirmTone {
    #[default]
    Default,
    Warning,
    Danger,
}

impl ConfirmTone {
    fn headline_color(self) -> Color {
        match self {
            ConfirmTone::Default => Color::Default,
            ConfirmTone::Warning => Color::Warning,
            ConfirmTone::Danger => Color::Error,
        }
    }

    fn action_color(self) -> Color {
        match self {
            ConfirmTone::Default => Color::Accent,
            ConfirmTone::Warning => Color::Warning,
            ConfirmTone::Danger => Color::Error,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConfirmOptions {
    pub title: SharedString,
    pub body: SharedString,
    pub confirm_label: SharedString,
    pub cancel_label: SharedString,
    pub tone: ConfirmTone,
}

impl ConfirmOptions {
    pub fn new(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            ..Self::default()
        }
    }

    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn tone(mut self, tone: ConfirmTone) -> Self {
        self.tone = tone;
        self
    }
}

fn label_or(label: &SharedString, fallback: &'static str) -> SharedString {
    if label.is_empty() {
        SharedString::new_static(fallback)
    } else {
        label.clone()
    }
}

struct ConfirmDialog {
    options: ConfirmOptions,
    sender: Option<oneshot::Sender<bool>>,
    focus_handle: FocusHandle,
}

impl ConfirmDialog {
    fn finish(&mut self, confirmed: bool, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(confirmed);
        }
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for ConfirmDialog {}

impl Focusable for ConfirmDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for ConfirmDialog {
    fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.sender.take();
        true
    }
}

impl Render for ConfirmDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let cancel = label_or(&self.options.cancel_label, strings.action_cancel());
        let confirm = label_or(&self.options.confirm_label, strings.ui_confirm());
        let tone = self.options.tone;
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Confirm, _, cx| this.finish(true, cx)))
            .child(
                Modal::new("yp-confirm-dialog", None)
                    .header(
                        ModalHeader::new()
                            .headline(self.options.title.clone())
                            .headline_color(tone.headline_color()),
                    )
                    .section(
                        div()
                            .text_size(sizes.font_sm)
                            .line_height(relative(1.4))
                            .child(self.options.body.clone()),
                    )
                    .footer(
                        ModalFooter::new().start_slot(
                            h_flex()
                                .min_w_0()
                                .max_w_full()
                                .flex_wrap()
                                .gap(sizes.spacing_sm)
                                .child(
                                    Button::new("yp-confirm-cancel", cancel)
                                        .style(ButtonStyle::Subtle)
                                        .color(Color::Muted)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.finish(false, cx);
                                        })),
                                )
                                .child(
                                    Button::new("yp-confirm-accept", confirm)
                                        .style(ButtonStyle::Subtle)
                                        .color(tone.action_color())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.finish(true, cx);
                                        })),
                                ),
                        ),
                    ),
            )
    }
}

pub fn confirm(options: ConfirmOptions, window: &mut Window, cx: &mut App) -> Task<bool> {
    let (sender, receiver) = oneshot::channel();
    window.open_modal(cx, |_, cx| ConfirmDialog {
        options,
        sender: Some(sender),
        focus_handle: cx.focus_handle(),
    });
    cx.spawn(async move |_| receiver.await.unwrap_or(false))
}

#[derive(Clone, Default)]
pub struct PromptOptions {
    pub title: SharedString,
    pub label: SharedString,
    pub initial: SharedString,
    pub placeholder: SharedString,
    pub max_len: Option<usize>,
    pub validate: Option<InputValidator>,
    pub confirm_label: SharedString,
    pub cancel_label: SharedString,
    pub allow_empty: bool,
}

impl PromptOptions {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn initial(mut self, initial: impl Into<SharedString>) -> Self {
        self.initial = initial.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn max_len(mut self, max_len: usize) -> Self {
        self.max_len = Some(max_len);
        self
    }

    pub fn validate(
        mut self,
        validate: impl Fn(&str) -> Result<(), SharedString> + 'static,
    ) -> Self {
        self.validate = Some(std::rc::Rc::new(validate));
        self
    }

    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn allow_empty(mut self, allow_empty: bool) -> Self {
        self.allow_empty = allow_empty;
        self
    }
}

struct PromptDialog {
    options: PromptOptions,
    input: Entity<InputState>,
    sender: Option<oneshot::Sender<Option<String>>>,
    _subscription: Subscription,
}

impl PromptDialog {
    fn new(
        options: PromptOptions,
        sender: oneshot::Sender<Option<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).placeholder(options.placeholder.clone());
            if let Some(max_len) = options.max_len {
                state = state.max_length(max_len);
            }
            if let Some(validate) = options.validate.clone() {
                state = state.validate(move |text| validate(text));
            }
            let mut state = state.default_value(options.initial.clone());
            state.select_all(window, cx);
            state
        });
        let subscription = cx.subscribe_in(
            &input,
            window,
            |this: &mut Self, _, event: &InputEvent, _, cx| match event {
                InputEvent::Submit => this.accept(cx),
                InputEvent::Change => cx.notify(),
                InputEvent::Focus | InputEvent::Blur => {}
            },
        );
        Self {
            options,
            input,
            sender: Some(sender),
            _subscription: subscription,
        }
    }

    fn can_accept(&self, cx: &App) -> bool {
        let input = self.input.read(cx);
        input.is_valid() && (self.options.allow_empty || !input.value().trim().is_empty())
    }

    fn accept(&mut self, cx: &mut Context<Self>) {
        if !self.can_accept(cx) {
            return;
        }
        let value = self.input.read(cx).value().trim().to_owned();
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Some(value));
        }
        cx.emit(DismissEvent);
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(None);
        }
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for PromptDialog {}

impl Focusable for PromptDialog {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl ModalView for PromptDialog {
    fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.sender.take();
        true
    }
}

impl Render for PromptDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let theme = cx.theme().clone();
        let cancel = label_or(&self.options.cancel_label, strings.button_cancel());
        let confirm = label_or(&self.options.confirm_label, strings.ui_ok());
        let can_accept = self.can_accept(cx);
        let error = self.input.read(cx).error().cloned();
        let counter = self.options.max_len.map(|max_len| {
            let used = self.input.read(cx).value().chars().count();
            SharedString::from(format!("{used}/{max_len}"))
        });
        Modal::new("yp-prompt-dialog", None)
            .header(ModalHeader::new().headline(self.options.title.clone()))
            .when(!self.options.label.is_empty(), |this| {
                this.section(
                    v_flex()
                        .text_size(sizes.font_sm)
                        .line_height(relative(LINE_HEIGHT_BODY))
                        .child(super::text::Text::new(self.options.label.clone())),
                )
            })
            .section(
                v_flex()
                    .gap(sizes.spacing_sm)
                    .child(Input::new(&self.input))
                    .child(
                        h_flex()
                            .gap(sizes.spacing_md)
                            .child(div().flex_1().min_w_0().children(error.map(|error| {
                                Label::new(error)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Error)
                            })))
                            .children(counter.map(|counter| {
                                div()
                                    .flex_none()
                                    .text_size(sizes.font_xs)
                                    .text_color(theme.colors.text_muted)
                                    .child(counter)
                            })),
                    ),
            )
            .footer(
                ModalFooter::new().end_slot(
                    h_flex()
                        .gap(sizes.spacing_sm)
                        .child(
                            Button::new("yp-prompt-cancel", cancel)
                                .style(ButtonStyle::Tinted(TintColor::Muted))
                                .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                        )
                        .child(
                            Button::new("yp-prompt-accept", confirm)
                                .style(ButtonStyle::Tinted(TintColor::Accent))
                                .disabled(!can_accept)
                                .on_click(cx.listener(|this, _, _, cx| this.accept(cx))),
                        ),
                ),
            )
    }
}

pub fn prompt_text(
    options: PromptOptions,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<String>> {
    if super::util::ControllerMode::is_active(cx) {
        return Task::ready(None);
    }
    let (sender, receiver) = oneshot::channel();
    window.open_modal(cx, |window, cx| {
        PromptDialog::new(options, sender, window, cx)
    });
    cx.spawn(async move |_| receiver.await.ok().flatten())
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickItem<T> {
    pub label: SharedString,
    pub description: Option<SharedString>,
    pub icon: Option<IconName>,
    pub value: T,
    pub selected: bool,
    pub disabled: bool,
}

impl<T> PickItem<T> {
    pub fn new(label: impl Into<SharedString>, value: T) -> Self {
        Self {
            label: label.into(),
            description: None,
            icon: None,
            value,
            selected: false,
            disabled: false,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickOptions<T> {
    pub title: SharedString,
    pub body: SharedString,
    pub items: Vec<PickItem<T>>,
    pub confirm_label: SharedString,
    pub cancel_label: SharedString,
}

impl<T> PickOptions<T> {
    pub fn new(title: impl Into<SharedString>, items: Vec<PickItem<T>>) -> Self {
        Self {
            title: title.into(),
            body: SharedString::default(),
            items,
            confirm_label: SharedString::default(),
            cancel_label: SharedString::default(),
        }
    }

    pub fn body(mut self, body: impl Into<SharedString>) -> Self {
        self.body = body.into();
        self
    }

    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }
}

struct PickEntry {
    label: SharedString,
    description: Option<SharedString>,
    icon: Option<IconName>,
    selected: bool,
    disabled: bool,
}

struct PickDialog {
    title: SharedString,
    body: SharedString,
    confirm_label: SharedString,
    cancel_label: SharedString,
    entries: Vec<PickEntry>,
    multiple: bool,
    sender: Option<oneshot::Sender<Option<Vec<usize>>>>,
    focus_handle: FocusHandle,
}

impl PickDialog {
    fn finish(&mut self, picked: Option<Vec<usize>>, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(picked);
        }
        cx.emit(DismissEvent);
    }

    fn selected_indices(&self) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| entry.selected.then_some(index))
            .collect()
    }

    fn toggle(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.multiple {
            self.finish(Some(vec![index]), cx);
            return;
        }
        if let Some(entry) = self.entries.get_mut(index) {
            entry.selected = !entry.selected;
            cx.notify();
        }
    }
}

impl EventEmitter<DismissEvent> for PickDialog {}

impl Focusable for PickDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for PickDialog {
    fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.sender.take();
        true
    }
}

impl Render for PickDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let cancel = label_or(&self.cancel_label, strings.action_cancel());
        let confirm = label_or(&self.confirm_label, strings.ui_confirm());
        let multiple = self.multiple;
        let selected = self.selected_indices();
        let selected_count = i64::try_from(selected.len()).unwrap_or(i64::MAX);
        let rows = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                if multiple {
                    h_flex()
                        .w_full()
                        .gap(sizes.spacing_md)
                        .child(
                            Checkbox::new(("yp-pick-check", index), entry.selected)
                                .label(entry.label.clone())
                                .disabled(entry.disabled)
                                .on_click(cx.listener(move |this, _: &ToggleState, _, cx| {
                                    this.toggle(index, cx);
                                })),
                        )
                        .children(entry.description.clone().map(|description| {
                            Label::new(description)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .truncate()
                        }))
                        .into_any_element()
                } else {
                    let card = ChoiceCard::new(("yp-pick-choice", index), entry.label.clone())
                        .compact()
                        .selected(entry.selected)
                        .disabled(entry.disabled)
                        .on_click(cx.listener(move |this, _, _, cx| this.toggle(index, cx)));
                    let card = match entry.description.clone() {
                        Some(description) => card.description(description),
                        None => card,
                    };
                    match entry.icon {
                        Some(icon) => card.icon(icon).into_any_element(),
                        None => card.into_any_element(),
                    }
                }
            })
            .collect::<Vec<_>>();
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(move |this, _: &Confirm, _, cx| {
                if this.multiple {
                    let picked = this.selected_indices();
                    this.finish(Some(picked), cx);
                }
            }))
            .child(
                Modal::new("yp-pick-dialog", None)
                    .header(ModalHeader::new().headline(self.title.clone()))
                    .when(!self.body.is_empty(), |this| {
                        this.section(
                            v_flex()
                                .text_size(sizes.font_sm)
                                .line_height(relative(LINE_HEIGHT_BODY))
                                .child(super::text::Text::new(self.body.clone())),
                        )
                    })
                    .section(v_flex().w_full().gap(sizes.spacing_md).children(rows))
                    .footer(
                        ModalFooter::new()
                            .start_slot(div().when(multiple, |this| {
                                this.child(
                                    Label::new(strings.ui_selected_count(selected_count))
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                )
                            }))
                            .end_slot(
                                h_flex()
                                    .gap(sizes.spacing_sm)
                                    .child(
                                        Button::new("yp-pick-cancel", cancel)
                                            .style(ButtonStyle::Tinted(TintColor::Muted))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.finish(None, cx);
                                            })),
                                    )
                                    .when(multiple, |this| {
                                        this.child(
                                            Button::new("yp-pick-accept", confirm)
                                                .style(ButtonStyle::Tinted(TintColor::Accent))
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    let picked = this.selected_indices();
                                                    this.finish(Some(picked), cx);
                                                })),
                                        )
                                    }),
                            ),
                    ),
            )
    }
}

fn open_pick_dialog<T: 'static>(
    options: PickOptions<T>,
    multiple: bool,
    window: &mut Window,
    cx: &mut App,
) -> (Vec<Option<T>>, oneshot::Receiver<Option<Vec<usize>>>) {
    let (sender, receiver) = oneshot::channel();
    let mut values = Vec::with_capacity(options.items.len());
    let mut entries = Vec::with_capacity(options.items.len());
    for item in options.items {
        entries.push(PickEntry {
            label: item.label,
            description: item.description,
            icon: item.icon,
            selected: item.selected,
            disabled: item.disabled,
        });
        values.push(Some(item.value));
    }
    window.open_modal(cx, |_, cx| PickDialog {
        title: options.title,
        body: options.body,
        confirm_label: options.confirm_label,
        cancel_label: options.cancel_label,
        entries,
        multiple,
        sender: Some(sender),
        focus_handle: cx.focus_handle(),
    });
    (values, receiver)
}

pub fn pick_one<T: 'static>(
    options: PickOptions<T>,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<T>> {
    let (mut values, receiver) = open_pick_dialog(options, false, window, cx);
    cx.spawn(async move |_| {
        let picked = receiver.await.ok().flatten()?;
        let index = picked.first().copied()?;
        values.get_mut(index)?.take()
    })
}

pub fn pick_many<T: 'static>(
    options: PickOptions<T>,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<Vec<T>>> {
    let (mut values, receiver) = open_pick_dialog(options, true, window, cx);
    cx.spawn(async move |_| {
        let picked = receiver.await.ok().flatten()?;
        Some(
            picked
                .into_iter()
                .filter_map(|index| values.get_mut(index).and_then(Option::take))
                .collect(),
        )
    })
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;

    use super::*;
    use crate::components::test_support::{has_modal, open_empty_root, redraw};

    #[gpui::test]
    async fn confirm_resolves_true_on_enter(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let task = cx.update(|window, cx| {
            confirm(
                ConfirmOptions::new("Title", "Body").tone(ConfirmTone::Danger),
                window,
                cx,
            )
        });
        cx.run_until_parked();
        assert!(has_modal(cx));
        cx.simulate_keystrokes("enter");
        assert!(task.await);
        assert!(!has_modal(cx));
    }

    #[gpui::test]
    async fn enter_activates_the_focused_cancel_button(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let task =
            cx.update(|window, cx| confirm(ConfirmOptions::new("Title", "Body"), window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("tab");
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            window.dispatch_event(
                gpui::PlatformInput::KeyUp(gpui::KeyUpEvent {
                    keystroke: gpui::Keystroke::parse("enter").unwrap(),
                }),
                cx,
            );
        });
        cx.run_until_parked();
        assert!(!has_modal(cx));
        assert!(!task.await);
    }

    #[gpui::test]
    async fn confirm_resolves_false_on_escape(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let task =
            cx.update(|window, cx| confirm(ConfirmOptions::new("Title", "Body"), window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("escape");
        redraw(cx);
        assert!(!task.await);
        assert!(!has_modal(cx));
    }

    #[gpui::test]
    async fn prompt_replaces_the_selected_initial_text_and_respects_max_len(
        cx: &mut TestAppContext,
    ) {
        let (_root, cx) = open_empty_root(cx);
        let task = cx.update(|window, cx| {
            prompt_text(
                PromptOptions::new("Name").initial("old").max_len(7),
                window,
                cx,
            )
        });
        cx.run_until_parked();
        cx.simulate_input("profiletwo");
        cx.simulate_keystrokes("enter");
        assert_eq!(task.await.as_deref(), Some("profile"));
    }

    #[gpui::test]
    async fn prompt_refuses_empty_and_invalid_text_until_it_is_fixed(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let task = cx.update(|window, cx| {
            prompt_text(
                PromptOptions::new("Name").validate(|text| {
                    if text.contains('!') {
                        Err("no bangs".into())
                    } else {
                        Ok(())
                    }
                }),
                window,
                cx,
            )
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        assert!(has_modal(cx));
        cx.simulate_input("a!");
        cx.simulate_keystrokes("enter");
        assert!(has_modal(cx));
        cx.simulate_keystrokes("backspace enter");
        assert_eq!(task.await.as_deref(), Some("a"));
    }

    #[gpui::test]
    async fn prompt_and_pickers_return_none_when_cancelled(cx: &mut TestAppContext) {
        let (_root, cx) = open_empty_root(cx);
        let prompt = cx.update(|window, cx| prompt_text(PromptOptions::new("Name"), window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("escape");
        redraw(cx);
        assert_eq!(prompt.await, None);

        let items = || {
            vec![
                PickItem::new("One", 1),
                PickItem::new("Two", 2).selected(true),
            ]
        };
        let one = cx.update(|window, cx| pick_one(PickOptions::new("Pick", items()), window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("escape");
        redraw(cx);
        assert_eq!(one.await, None);

        let many = cx.update(|window, cx| pick_many(PickOptions::new("Pick", items()), window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("enter");
        assert_eq!(many.await, Some(vec![2]));
    }
}
