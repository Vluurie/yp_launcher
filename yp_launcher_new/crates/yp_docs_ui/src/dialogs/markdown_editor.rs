use std::sync::Arc;

use futures::channel::oneshot;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    Render, ScrollHandle, SharedString, Subscription, Task, Window, actions, div,
    prelude::*,
};
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, ButtonStyle, InputEvent, ModalView, Sizes, TintColor, Tooltip};

use crate::markdown::{Document, MarkdownContext, parse_document, render_document};

actions!(yp_docs, [SaveDocument]);

const EDITOR_CONTEXT: &str = "YpDocsMarkdownEditor";
const CHROME_HEIGHT: f32 = 190.;
const MIN_ROWS: usize = 6;
const MAX_ROWS: usize = 60;
const INPUT_LINE_HEIGHT: f32 = 1.3;

pub(crate) fn init(cx: &mut App) {
    let save = yp_ui::host_key_binding("secondary-s", SaveDocument, Some(EDITOR_CONTEXT), cx);
    cx.bind_keys([save]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownEditorOptions {
    pub file_name: SharedString,
    pub file_path: SharedString,
    pub initial: String,
    pub save_label: SharedString,
}

pub fn editor_rows(window_height: f32, line_height: f32) -> usize {
    let available = window_height
        - f32::from(Sizes::TITLE_BAR_HEIGHT)
        - f32::from(Sizes::LIGHTBOX_INSET) * 2.
        - CHROME_HEIGHT;
    if line_height <= 0. {
        return MIN_ROWS;
    }
    ((available / line_height).floor() as i64).clamp(MIN_ROWS as i64, MAX_ROWS as i64) as usize
}

pub struct MarkdownEditorDialog {
    options: MarkdownEditorOptions,
    editor: Entity<InputState>,
    preview: Arc<Document>,
    preview_scroll: ScrollHandle,
    dirty: bool,
    sender: Option<oneshot::Sender<Option<String>>>,
    _subscriptions: Vec<Subscription>,
}

impl MarkdownEditorDialog {
    fn new(
        options: MarkdownEditorOptions,
        sender: oneshot::Sender<Option<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let sizes = window.sizes(cx);
        let rows = editor_rows(
            f32::from(sizes.window_height),
            f32::from(sizes.font_sm) * INPUT_LINE_HEIGHT,
        );
        let initial = options.initial.clone();
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(rows)
                .max_rows(rows)
                .mono()
                .default_value(initial)
        });
        let subscriptions = vec![cx.subscribe(&editor, |this, editor, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                let dirty = editor.read(cx).value() != this.options.initial;
                if dirty != this.dirty {
                    this.dirty = dirty;
                    cx.notify();
                }
            }
        })];
        window.focus(&editor.focus_handle(cx), cx);
        Self {
            preview: Arc::new(parse_document(&options.initial)),
            options,
            editor,
            preview_scroll: ScrollHandle::new(),
            dirty: false,
            sender: Some(sender),
            _subscriptions: subscriptions,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn text(&self, cx: &App) -> String {
        self.editor.read(cx).value().to_owned()
    }

    pub fn preview(&self) -> &Document {
        &self.preview
    }

    pub fn set_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = text.to_owned();
        self.editor
            .update(cx, |editor, cx| editor.set_value(value, window, cx));
        self.dirty = text != self.options.initial;
        cx.notify();
    }

    pub fn refresh_preview(&mut self, cx: &mut Context<Self>) {
        self.preview = Arc::new(parse_document(self.editor.read(cx).value()));
        cx.notify();
    }

    fn finish(&mut self, result: Option<String>, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(result);
        }
        cx.emit(DismissEvent);
    }

    pub fn save(&mut self, cx: &mut Context<Self>) {
        let text = self.text(cx);
        self.finish(Some(text), cx);
    }
}

impl EventEmitter<DismissEvent> for MarkdownEditorDialog {}

impl Focusable for MarkdownEditorDialog {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle(cx)
    }
}

impl ModalView for MarkdownEditorDialog {
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

impl Render for MarkdownEditorDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let context = MarkdownContext::new("docs-editor-preview").compact(true);
        let preview = render_document(&self.preview, &context, window, cx);
        let headline = if self.dirty {
            SharedString::from(format!("{} *", self.options.file_name))
        } else {
            self.options.file_name.clone()
        };
        div()
            .key_context(EDITOR_CONTEXT)
            .on_action(cx.listener(|this, _: &SaveDocument, _, cx| this.save(cx)))
            .child(
                Modal::new("markdown-editor-dialog", None)
                    .width(ModalWidth::Full)
                    .header(
                        ModalHeader::new()
                            .icon(Icon::new(IconName::EditNote).color(Color::Accent))
                            .headline(headline)
                            .headline_color(if self.dirty {
                                Color::Warning
                            } else {
                                Color::Default
                            })
                            .description(self.options.file_path.clone()),
                    )
                    .section(
                        div()
                            .flex()
                            .flex_row()
                            .w_full()
                            .gap(sizes.spacing_lg)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Input::new(&self.editor).size(InputSize::Small)),
                            )
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_w_0()
                                    .rounded(Sizes::ROW_RADIUS)
                                    .border_1()
                                    .border_color(theme.colors.border_light)
                                    .bg(theme.colors.background_primary)
                                    .child(
                                        div()
                                            .id("markdown-editor-preview")
                                            .absolute()
                                            .inset_0()
                                            .flex()
                                            .flex_col()
                                            .p(sizes.padding_md)
                                            .gap(sizes.spacing_md)
                                            .children(preview)
                                            .overflow_y_scrollbar(&self.preview_scroll),
                                    ),
                            ),
                    )
                    .footer(
                        ModalFooter::new()
                            .start_slot(
                                Button::new("markdown-editor-preview-refresh", strings.docs_preview())
                                    .style(ButtonStyle::Tinted(TintColor::Accent))
                                    .start_icon(IconName::Refresh)
                                    .on_click(cx.listener(|this, _, _, cx| this.refresh_preview(cx))),
                            )
                            .end_slot(
                                h_flex()
                                    .gap(sizes.spacing_md)
                                    .child(
                                        Button::new("markdown-editor-cancel", strings.cancel_button())
                                            .style(ButtonStyle::Tinted(TintColor::Muted))
                                            .on_click(
                                                cx.listener(|this, _, _, cx| this.finish(None, cx)),
                                            ),
                                    )
                                    .child(
                                        Button::new(
                                            "markdown-editor-save",
                                            self.options.save_label.clone(),
                                        )
                                        .style(ButtonStyle::Tinted(TintColor::Success))
                                        .tooltip(Tooltip::text(strings.docs_save_shortcut()))
                                        .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                                    ),
                            ),
                    ),
            )
    }
}

pub fn edit_markdown(
    options: MarkdownEditorOptions,
    window: &mut Window,
    cx: &mut App,
) -> Task<Option<String>> {
    let (sender, receiver) = oneshot::channel();
    window.open_modal(cx, |window, cx| {
        MarkdownEditorDialog::new(options, sender, window, cx)
    });
    cx.spawn(async move |_| receiver.await.ok().flatten())
}

#[cfg(all(test, feature = "tools"))]
pub(crate) fn open_for_test(
    options: MarkdownEditorOptions,
    window: &mut Window,
    cx: &mut App,
) -> (Option<Entity<MarkdownEditorDialog>>, Task<Option<String>>) {
    let (sender, receiver) = oneshot::channel();
    let dialog = window.open_modal(cx, |window, cx| {
        MarkdownEditorDialog::new(options, sender, window, cx)
    });
    (dialog, cx.spawn(async move |_| receiver.await.ok().flatten()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_editor_fills_the_window_between_sensible_limits() {
        assert_eq!(editor_rows(300., 20.), MIN_ROWS);
        assert_eq!(editor_rows(750., 20.), 22);
        assert_eq!(editor_rows(5000., 10.), MAX_ROWS);
        assert_eq!(editor_rows(750., 0.), MIN_ROWS);
    }
}
