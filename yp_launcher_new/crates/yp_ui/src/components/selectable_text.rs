use std::ops::Range;

use gpui::{
    App, ClipboardItem, CursorStyle, ElementId, FocusHandle, HighlightStyle, IntoElement,
    KeyBinding, MouseButton, Pixels, Point, RenderOnce, SharedString, StyledText, TextLayout,
    Window, div, prelude::*, relative,
};

use super::input::{Copy, SelectAll, word_range_at};
use super::label::{LINE_HEIGHT_BODY, LabelSize};
use super::text_menu::{self, KEY_COPY, KEY_SELECT_ALL, TextMenu};
use super::util::pointer_capture;
use crate::actions::host_key_binding;
use crate::i18n::ActiveStrings;
use crate::styled_ext::StyledExt;
use crate::theme::{ActiveTheme, Color, ResponsiveExt};

pub(crate) const SELECTABLE_TEXT_CONTEXT: &str = "YpSelectableText";

pub(crate) fn init(cx: &mut App) {
    let context = Some(SELECTABLE_TEXT_CONTEXT);
    cx.bind_keys([
        host_key_binding(&format!("secondary-{KEY_COPY}"), Copy, context, cx),
        KeyBinding::new("ctrl-insert", Copy, context),
        host_key_binding(&format!("secondary-{KEY_SELECT_ALL}"), SelectAll, context, cx),
    ]);
}

struct SelectionState {
    anchor: usize,
    head: usize,
    selecting: bool,
    focus_handle: FocusHandle,
}

impl SelectionState {
    fn range(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }
}

pub fn clamp_selection(text: &str, range: Range<usize>) -> Range<usize> {
    let snap = |offset: usize| {
        let mut offset = offset.min(text.len());
        while offset > 0 && !text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    };
    let start = snap(range.start);
    let end = snap(range.end).max(start);
    start..end
}

fn index_at(layout: &TextLayout, position: Point<Pixels>) -> usize {
    match layout.index_for_position(position) {
        Ok(index) | Err(index) => index,
    }
}

pub struct SelectableText {
    id: ElementId,
    text: SharedString,
    mono: bool,
    size: LabelSize,
    text_size: Option<Pixels>,
    color: Color,
    line_height: f32,
}

crate::impl_into_element!(SelectableText);

impl SelectableText {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            mono: false,
            size: LabelSize::Small,
            text_size: None,
            color: Color::Secondary,
            line_height: LINE_HEIGHT_BODY,
        }
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    pub fn text_size(mut self, size: Pixels) -> Self {
        self.text_size = Some(size);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn line_height(mut self, factor: f32) -> Self {
        self.line_height = factor;
        self
    }
}

impl RenderOnce for SelectableText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| SelectionState {
            anchor: 0,
            head: 0,
            selecting: false,
            focus_handle: cx.focus_handle(),
        });
        let (selection, selecting, focus_handle) = {
            let state = state.read(cx);
            (
                clamp_selection(&self.text, state.range()),
                state.selecting,
                state.focus_handle.clone(),
            )
        };
        let focused = focus_handle.is_focused(window);
        let highlight_color = if focused {
            theme.colors.accent_primary.opacity(theme.alpha.soft)
        } else {
            theme.colors.accent_primary.opacity(theme.alpha.hover)
        };
        let highlights = (!selection.is_empty())
            .then(|| {
                (
                    selection.clone(),
                    HighlightStyle {
                        background_color: Some(highlight_color),
                        ..Default::default()
                    },
                )
            })
            .into_iter()
            .collect::<Vec<_>>();
        let styled = StyledText::new(self.text.clone()).with_highlights(highlights);
        let layout = styled.layout().clone();
        let text = self.text.clone();

        let down_state = state.clone();
        let down_layout = layout.clone();
        let down_text = text.clone();
        let down_focus = focus_handle.clone();
        let move_state = state.clone();
        let move_layout = layout.clone();
        let up_state = state.clone();
        let out_state = state.clone();
        let copy_state = state.clone();
        let copy_text = text.clone();
        let all_state = state.clone();
        let all_len = text.len();
        let text_menu = TextMenu::new((self.id.clone(), "yp-text-menu"), window, cx);
        let menu_overlay = text_menu.overlay(cx);
        let menu_state = state.clone();
        let menu_text = text.clone();
        let menu_focus = focus_handle.clone();

        div()
            .id(self.id)
            .key_context(SELECTABLE_TEXT_CONTEXT)
            .track_focus(&focus_handle)
            .relative()
            .flex()
            .flex_col()
            .min_w_0()
            .cursor(CursorStyle::IBeam)
            .text_size(self.text_size.unwrap_or_else(|| self.size.pixels(&sizes)))
            .line_height(relative(self.line_height))
            .text_color(self.color.color(cx))
            .when(self.mono, |this| this.font_mono(cx))
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                cx.stop_propagation();
                window.focus(&down_focus, cx);
                let index = index_at(&down_layout, event.position);
                down_state.update(cx, |state, cx| {
                    match event.click_count {
                        2 => {
                            let word = word_range_at(&down_text, index.min(down_text.len()));
                            state.anchor = word.start;
                            state.head = word.end;
                            state.selecting = false;
                        }
                        count if count >= 3 => {
                            state.anchor = 0;
                            state.head = down_text.len();
                            state.selecting = false;
                        }
                        _ => {
                            if event.modifiers.shift {
                                state.head = index;
                            } else {
                                state.anchor = index;
                                state.head = index;
                            }
                            state.selecting = true;
                        }
                    }
                    cx.notify();
                });
            })
            .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                cx.stop_propagation();
                window.focus(&menu_focus, cx);
                let strings = cx.t();
                let selection = clamp_selection(&menu_text, menu_state.read(cx).range());
                let copied = if selection.is_empty() {
                    menu_text.to_string()
                } else {
                    menu_text[selection].to_owned()
                };
                let has_text = !menu_text.is_empty();
                let select_state = menu_state.clone();
                let length = menu_text.len();
                let entries = vec![
                    text_menu::entry(strings.docs_copy(), KEY_COPY, has_text, cx, move |_, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                    }),
                    text_menu::entry(
                        strings.ui_select_all(),
                        KEY_SELECT_ALL,
                        has_text,
                        cx,
                        move |_, cx| {
                            select_state.update(cx, |state, cx| {
                                state.anchor = 0;
                                state.head = length;
                                cx.notify();
                            });
                        },
                    ),
                ];
                text_menu.open(event.position, entries, window, cx);
            })
            .children(menu_overlay)
            .on_mouse_down_out(move |_, _, cx| {
                if text_menu::text_menu_is_open(cx) {
                    return;
                }
                let has_selection = {
                    let state = out_state.read(cx);
                    state.anchor != state.head
                };
                if has_selection {
                    out_state.update(cx, |state, cx| {
                        state.anchor = 0;
                        state.head = 0;
                        cx.notify();
                    });
                }
            })
            .on_action(move |_: &Copy, _, cx| {
                let range = clamp_selection(&copy_text, copy_state.read(cx).range());
                if !range.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(copy_text[range].to_owned()));
                }
            })
            .on_action(move |_: &SelectAll, _, cx| {
                all_state.update(cx, |state, cx| {
                    state.anchor = 0;
                    state.head = all_len;
                    cx.notify();
                });
            })
            .child(styled)
            .child(pointer_capture(
                selecting,
                move |event, _, cx| {
                    let index = index_at(&move_layout, event.position);
                    let changed = move_state.read(cx).head != index;
                    if changed {
                        move_state.update(cx, |state, cx| {
                            state.head = index;
                            cx.notify();
                        });
                    }
                },
                move |_, _, cx| {
                    up_state.update(cx, |state, cx| {
                        state.selecting = false;
                        cx.notify();
                    });
                },
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_clamped_to_the_text_and_to_char_boundaries() {
        let text = "aé文b";
        assert_eq!(clamp_selection(text, 0..99), 0..text.len());
        assert_eq!(clamp_selection(text, 2..3), 1..3);
        let reversed = Range { start: 5, end: 4 };
        assert_eq!(clamp_selection(text, reversed), 3..3);
        assert_eq!(&text[clamp_selection(text, 1..6)], "é文");
    }
}
