use futures::channel::oneshot;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, Render,
    SharedString, Subscription, Task, Window, prelude::*, px,
};
use yp_core::toml_io::parse_values;
use yp_ui::prelude::*;
use yp_ui::{ButtonStyle, ChipTone, InputEvent, ModalView, Severity, Sizes, TintColor};

const EDITOR_MIN_ROWS: usize = 6;
const EDITOR_MAX_ROWS: usize = 18;
const CHROME_HEIGHT: f32 = 400.;
const INPUT_LINE_HEIGHT: f32 = 1.3;

pub fn editor_max_rows(window_height: f32, line_height: f32) -> usize {
    if line_height <= 0. {
        return EDITOR_MIN_ROWS;
    }
    let available = window_height
        - f32::from(Sizes::TITLE_BAR_HEIGHT)
        - f32::from(Sizes::LIGHTBOX_INSET) * 2.
        - CHROME_HEIGHT;
    ((available / line_height).floor() as i64)
        .clamp(EDITOR_MIN_ROWS as i64, EDITOR_MAX_ROWS as i64) as usize
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlEditorOptions {
    pub title: SharedString,
    pub file_name: SharedString,
    pub explanation: SharedString,
    pub initial: String,
    pub reset: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlKey {
    pub name: String,
    pub is_table: bool,
}

impl TomlKey {
    pub fn label(&self) -> String {
        if self.is_table {
            format!("[{}]", self.name)
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TomlCheck {
    Valid(Vec<TomlKey>),
    Invalid(String),
}

impl TomlCheck {
    pub fn is_valid(&self) -> bool {
        matches!(self, TomlCheck::Valid(_))
    }
}

pub fn check_toml(text: &str) -> TomlCheck {
    match parse_values(text) {
        Ok(table) => TomlCheck::Valid(
            table
                .iter()
                .map(|(name, value)| TomlKey {
                    name: name.clone(),
                    is_table: value.is_table(),
                })
                .collect(),
        ),
        Err(error) => TomlCheck::Invalid(error.to_string()),
    }
}

pub struct TomlEditorDialog {
    options: TomlEditorOptions,
    editor: Entity<InputState>,
    check: TomlCheck,
    sender: Option<oneshot::Sender<Option<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl TomlEditorDialog {
    fn new(
        options: TomlEditorOptions,
        sender: oneshot::Sender<Option<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let initial = options.initial.clone();
        let sizes = window.sizes(cx);
        let max_rows = editor_max_rows(
            f32::from(sizes.window_height),
            f32::from(sizes.font_sm) * INPUT_LINE_HEIGHT,
        );
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(EDITOR_MIN_ROWS)
                .max_rows(max_rows)
                .mono()
                .default_value(initial)
        });
        let subscriptions = vec![cx.subscribe(&editor, |this, editor, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.check = check_toml(editor.read(cx).value());
                cx.notify();
            }
        })];
        window.focus(&editor.focus_handle(cx), cx);
        Self {
            check: check_toml(&options.initial),
            options,
            editor,
            sender: Some(sender),
            _subscriptions: subscriptions,
        }
    }

    pub fn check(&self) -> &TomlCheck {
        &self.check
    }

    pub fn text(&self, cx: &App) -> String {
        self.editor.read(cx).value().to_owned()
    }

    pub fn set_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = text.to_owned();
        self.editor
            .update(cx, |editor, cx| editor.set_value(value, window, cx));
        self.check = check_toml(text);
        cx.notify();
    }

    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let reset = self.options.reset.clone();
        self.set_text(&reset, window, cx);
    }

    fn finish(&mut self, result: Option<String>, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(result);
        }
        cx.emit(DismissEvent);
    }

    pub fn write(&mut self, cx: &mut Context<Self>) {
        if self.check.is_valid() {
            let text = self.text(cx);
            self.finish(Some(text), cx);
        }
    }

    fn preview(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        match &self.check {
            TomlCheck::Invalid(error) => Some(
                Callout::new()
                    .id("toml-editor-invalid")
                    .severity(Severity::Error)
                    .title(strings.toml_editor_invalid())
                    .title_size(LabelSize::XSmall)
                    .child(
                        Label::new(SharedString::from(error.clone()))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .mono()
                            .line_height(1.4),
                    )
                    .into_any_element(),
            ),
            TomlCheck::Valid(keys) if keys.is_empty() => None,
            TomlCheck::Valid(keys) => Some(
                Callout::new()
                    .id("toml-editor-valid")
                    .severity(Severity::Success)
                    .icon(IconName::CheckCircleOutline)
                    .title(strings.toml_editor_valid())
                    .title_size(LabelSize::XSmall)
                    .child(
                        h_flex()
                            .w_full()
                            .flex_wrap()
                            .gap(sizes.spacing_md)
                            .children(keys.iter().map(|key| {
                                Chip::new(SharedString::from(key.label())).tone(if key.is_table {
                                    ChipTone::Accent
                                } else {
                                    ChipTone::Neutral
                                })
                            })),
                    )
                    .into_any_element(),
            ),
        }
    }
}

impl EventEmitter<DismissEvent> for TomlEditorDialog {}

impl Focusable for TomlEditorDialog {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle(cx)
    }
}

impl ModalView for TomlEditorDialog {
    fn on_before_dismiss(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(None);
        }
        true
    }

    fn dismiss_on_scrim_click(&self) -> bool {
        false
    }
}

impl Render for TomlEditorDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let valid = self.check.is_valid();
        let preview = self.preview(window, cx);
        Modal::new("toml-editor-dialog", None)
            .width(ModalWidth::Custom(px(820.)))
            .header(
                ModalHeader::new()
                    .icon(Icon::new(IconName::DescriptionOutlined).color(Color::Accent))
                    .headline(self.options.title.clone())
                    .description(self.options.file_name.clone()),
            )
            .section(
                Label::new(self.options.explanation.clone())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .line_height(1.5),
            )
            .section(Input::new(&self.editor).size(InputSize::Small).invalid(!valid))
            .children(preview)
            .footer(
                ModalFooter::new()
                    .start_slot(
                        Button::new("toml-editor-reset", strings.toml_editor_reset())
                            .style(ButtonStyle::Tinted(TintColor::Muted))
                            .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
                    )
                    .end_slot(
                        h_flex()
                            .gap(sizes.spacing_md)
                            .child(
                                Button::new("toml-editor-cancel", strings.cancel_button())
                                    .style(ButtonStyle::Tinted(TintColor::Muted))
                                    .on_click(cx.listener(|this, _, _, cx| this.finish(None, cx))),
                            )
                            .child(
                                Button::new("toml-editor-write", strings.toml_editor_write())
                                    .style(ButtonStyle::Tinted(if valid {
                                        TintColor::Success
                                    } else {
                                        TintColor::Muted
                                    }))
                                    .disabled(!valid)
                                    .on_click(cx.listener(|this, _, _, cx| this.write(cx))),
                            ),
                    ),
            )
    }
}

pub fn edit_toml(
    options: TomlEditorOptions,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<String>> {
    let (sender, receiver) = oneshot::channel();
    window.open_modal(cx, |window, cx| TomlEditorDialog::new(options, sender, window, cx));
    cx.spawn(async move |_| receiver.await.ok().flatten())
}

#[cfg(test)]
pub(crate) fn open_for_test(
    options: TomlEditorOptions,
    window: &mut Window,
    cx: &mut App,
) -> (Option<Entity<TomlEditorDialog>>, Task<Option<String>>) {
    let (sender, receiver) = oneshot::channel();
    let dialog =
        window.open_modal(cx, |window, cx| TomlEditorDialog::new(options, sender, window, cx));
    (dialog, cx.spawn(async move |_| receiver.await.ok().flatten()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_editor_leaves_room_for_the_check_result() {
        assert_eq!(editor_max_rows(420., 24.), EDITOR_MIN_ROWS);
        assert_eq!(editor_max_rows(2000., 24.), EDITOR_MAX_ROWS);
        assert_eq!(editor_max_rows(750., 0.), EDITOR_MIN_ROWS);
        let rows = editor_max_rows(750., 24.);
        assert!(rows > EDITOR_MIN_ROWS && rows < EDITOR_MAX_ROWS);
    }

    #[test]
    fn valid_toml_lists_its_top_level_keys_in_order() {
        let check = check_toml("id = \"my_first_mod\"\nversion = \"1.0.0\"\n\n[text.name]\n_us = \"x\"\n\n[auto_give]\nquantity = 1\n");
        let TomlCheck::Valid(keys) = check else {
            panic!("expected valid TOML");
        };
        let labels: Vec<String> = keys.iter().map(TomlKey::label).collect();
        assert_eq!(labels, vec!["id", "version", "[text]", "[auto_give]"]);
    }

    #[test]
    fn broken_toml_reports_why_and_empty_text_is_valid() {
        let check = check_toml("id = \n");
        assert!(!check.is_valid());
        assert!(matches!(check, TomlCheck::Invalid(reason) if !reason.is_empty()));
        assert_eq!(check_toml(""), TomlCheck::Valid(Vec::new()));
        assert!(!check_toml("a = 1\na = 2\n").is_valid());
    }
}
