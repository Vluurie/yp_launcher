use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, DefiniteLength,
    Element, ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle,
    Focusable, GlobalElementId, Hsla, InspectorElementId, IntoElement, KeyBinding, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, RenderOnce,
    ScrollEvent, SharedString, Style, Subscription, Task, TextAlign, TextRun, UTF16Selection,
    UnderlineStyle, Window, WrappedLine, actions, div, fill, point, prelude::*, px, relative, size,
};
use unicode_segmentation::UnicodeSegmentation;

use super::button::{ButtonSize, IconButton};
use super::icon::{IconName, IconSize};
use super::menu::ContextMenuEntry;
use super::text_menu::{self, KEY_COPY, KEY_CUT, KEY_PASTE, KEY_SELECT_ALL, TextMenu};
use super::tooltip::Tooltip;
use super::util::pointer_capture;
use crate::actions::host_key_binding;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::{ButtonCommon, Clickable};

actions!(
    yp_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        SelectToHome,
        SelectToEnd,
        WordLeft,
        WordRight,
        SelectWordLeft,
        SelectWordRight,
        DeleteWordLeft,
        DeleteWordRight,
        Copy,
        Cut,
        Paste,
        Undo,
        Redo,
        Enter,
        Submit,
    ]
);

pub(crate) const INPUT_CONTEXT: &str = "YpInput";
const HISTORY_LIMIT: usize = 200;

pub(crate) fn init(cx: &mut App) {
    let context = Some(INPUT_CONTEXT);
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("shift-backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("up", Up, context),
        KeyBinding::new("down", Down, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("shift-up", SelectUp, context),
        KeyBinding::new("shift-down", SelectDown, context),
        host_key_binding(&format!("secondary-{KEY_SELECT_ALL}"), SelectAll, context, cx),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("shift-home", SelectToHome, context),
        KeyBinding::new("shift-end", SelectToEnd, context),
        KeyBinding::new("ctrl-left", WordLeft, context),
        KeyBinding::new("ctrl-right", WordRight, context),
        KeyBinding::new("alt-left", WordLeft, context),
        KeyBinding::new("alt-right", WordRight, context),
        KeyBinding::new("ctrl-shift-left", SelectWordLeft, context),
        KeyBinding::new("ctrl-shift-right", SelectWordRight, context),
        KeyBinding::new("alt-shift-left", SelectWordLeft, context),
        KeyBinding::new("alt-shift-right", SelectWordRight, context),
        KeyBinding::new("ctrl-backspace", DeleteWordLeft, context),
        KeyBinding::new("alt-backspace", DeleteWordLeft, context),
        KeyBinding::new("ctrl-delete", DeleteWordRight, context),
        KeyBinding::new("alt-delete", DeleteWordRight, context),
        host_key_binding(&format!("secondary-{KEY_COPY}"), Copy, context, cx),
        host_key_binding(&format!("secondary-{KEY_CUT}"), Cut, context, cx),
        host_key_binding(&format!("secondary-{KEY_PASTE}"), Paste, context, cx),
        KeyBinding::new("ctrl-insert", Copy, context),
        KeyBinding::new("shift-insert", Paste, context),
        KeyBinding::new("shift-delete", Cut, context),
        host_key_binding("secondary-z", Undo, context, cx),
        host_key_binding("secondary-shift-z", Redo, context, cx),
        host_key_binding("secondary-y", Redo, context, cx),
        KeyBinding::new("enter", Enter, context),
        host_key_binding("secondary-enter", Submit, context, cx),
    ]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Change,
    Submit,
    Focus,
    Blur,
}

pub type InputValidator = Rc<dyn Fn(&str) -> Result<(), SharedString>>;

#[derive(Clone)]
struct Snapshot {
    text: String,
    selected_range: Range<usize>,
    selection_reversed: bool,
}

#[derive(Clone)]
struct RowSpan {
    start: usize,
    end: usize,
    start_x: Pixels,
}

#[derive(Clone)]
struct LineSnapshot {
    start: usize,
    top: Pixels,
    line: Rc<WrappedLine>,
    rows: Vec<RowSpan>,
}

impl LineSnapshot {
    fn len(&self) -> usize {
        self.line.len()
    }

    fn end(&self) -> usize {
        self.start + self.len()
    }

    fn height(&self, line_height: Pixels) -> Pixels {
        line_height * self.rows.len() as f32
    }
}

#[derive(Clone)]
struct LayoutSnapshot {
    lines: Vec<LineSnapshot>,
    line_height: Pixels,
    bounds: Bounds<Pixels>,
    content_size: gpui::Size<Pixels>,
}

impl LayoutSnapshot {
    fn line_for_offset(&self, offset: usize) -> Option<&LineSnapshot> {
        self.lines
            .iter()
            .rev()
            .find(|line| line.start <= offset)
            .or(self.lines.first())
    }

    fn line_for_y(&self, y: Pixels) -> Option<&LineSnapshot> {
        self.lines
            .iter()
            .rev()
            .find(|line| line.top <= y)
            .or(self.lines.first())
    }

    fn position_for_offset(&self, offset: usize) -> Point<Pixels> {
        let Some(line) = self.line_for_offset(offset) else {
            return Point::default();
        };
        let local = offset.saturating_sub(line.start).min(line.len());
        let position = line
            .line
            .position_for_index(local, self.line_height)
            .unwrap_or_default();
        point(position.x, position.y + line.top)
    }

    fn offset_for_position(&self, position: Point<Pixels>) -> usize {
        let Some(line) = self.line_for_y(position.y.max(px(0.))) else {
            return 0;
        };
        let height = line.height(self.line_height);
        let inner_y = (position.y - line.top)
            .max(px(0.))
            .min(height - self.line_height / 2.);
        let local = match line
            .line
            .closest_index_for_position(point(position.x, inner_y), self.line_height)
        {
            Ok(index) | Err(index) => index,
        };
        line.start + local.min(line.len())
    }

    fn row_count(&self) -> usize {
        self.lines.iter().map(|line| line.rows.len()).sum()
    }
}

fn rows_for_line(line: &WrappedLine) -> Vec<RowSpan> {
    let mut rows = Vec::with_capacity(line.wrap_boundaries().len() + 1);
    let mut start = 0;
    let mut start_x = px(0.);
    for boundary in line.wrap_boundaries() {
        let Some(glyph) = line
            .runs()
            .get(boundary.run_ix)
            .and_then(|run| run.glyphs.get(boundary.glyph_ix))
        else {
            continue;
        };
        rows.push(RowSpan {
            start,
            end: glyph.index,
            start_x,
        });
        start = glyph.index;
        start_x = glyph.position.x;
    }
    rows.push(RowSpan {
        start,
        end: line.len(),
        start_x,
    });
    rows
}

pub fn previous_grapheme_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .rev()
        .find_map(|(index, _)| (index < offset).then_some(index))
        .unwrap_or(0)
}

pub fn next_grapheme_boundary(text: &str, offset: usize) -> usize {
    text.grapheme_indices(true)
        .find_map(|(index, _)| (index > offset).then_some(index))
        .unwrap_or(text.len())
}

pub fn previous_word_boundary(text: &str, offset: usize) -> usize {
    let mut result = 0;
    for (index, word) in text.split_word_bound_indices() {
        if index >= offset {
            break;
        }
        if !word.chars().all(char::is_whitespace) {
            result = index;
        }
    }
    result
}

pub fn next_word_boundary(text: &str, offset: usize) -> usize {
    for (index, word) in text.split_word_bound_indices() {
        let end = index + word.len();
        if end > offset && !word.chars().all(char::is_whitespace) {
            return end;
        }
    }
    text.len()
}

pub fn word_range_at(text: &str, offset: usize) -> Range<usize> {
    for (index, word) in text.split_word_bound_indices() {
        let end = index + word.len();
        if offset < end || end == text.len() {
            return index..end;
        }
    }
    0..text.len()
}

fn clamp_to_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

pub struct InputState {
    focus_handle: FocusHandle,
    text: String,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    multi_line: bool,
    min_rows: usize,
    max_rows: usize,
    mono: bool,
    max_length: Option<usize>,
    validator: Option<InputValidator>,
    error: Option<SharedString>,
    disabled: bool,
    element_disabled: bool,
    read_only: bool,
    layout: Option<LayoutSnapshot>,
    scroll: Point<Pixels>,
    visual_rows: usize,
    reveal_cursor: bool,
    selecting: bool,
    cursor_visible: bool,
    blink_epoch: usize,
    blink_task: Option<Task<()>>,
    preferred_x: Option<Pixels>,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<InputEvent> for InputState {}

impl Focusable for InputState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl InputState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle().tab_stop(true);
        let subscriptions = vec![
            cx.on_focus(&focus_handle, window, |this, _, cx| this.handle_focus(cx)),
            cx.on_blur(&focus_handle, window, |this, _, cx| this.handle_blur(cx)),
        ];
        Self {
            focus_handle,
            text: String::new(),
            placeholder: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            multi_line: false,
            min_rows: 1,
            max_rows: 1,
            mono: false,
            max_length: None,
            validator: None,
            error: None,
            disabled: false,
            element_disabled: false,
            read_only: false,
            layout: None,
            scroll: Point::default(),
            visual_rows: 1,
            reveal_cursor: false,
            selecting: false,
            cursor_visible: false,
            blink_epoch: 0,
            blink_task: None,
            preferred_x: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            _subscriptions: subscriptions,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        let value = value.into();
        self.text = self.sanitize(&value);
        self.text = self.limit_inserted(0, &self.text.clone());
        self.selected_range = self.text.len()..self.text.len();
        self.revalidate();
        self
    }

    pub fn multi_line(mut self, min_rows: usize) -> Self {
        self.multi_line = true;
        self.min_rows = min_rows.max(1);
        self.max_rows = self.max_rows.max(self.min_rows).max(12);
        self.visual_rows = self.min_rows;
        self
    }

    pub fn max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = max_rows.max(self.min_rows);
        self
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self.text = self.limit_inserted(0, &self.text.clone());
        self.selected_range = self.text.len()..self.text.len();
        self
    }

    pub fn validate(
        mut self,
        validator: impl Fn(&str) -> Result<(), SharedString> + 'static,
    ) -> Self {
        self.validator = Some(Rc::new(validator));
        self.revalidate();
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn value(&self) -> &str {
        &self.text
    }

    pub fn text(&self) -> SharedString {
        SharedString::from(self.text.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn error(&self) -> Option<&SharedString> {
        self.error.as_ref()
    }

    pub fn is_valid(&self) -> bool {
        self.error.is_none()
    }

    pub fn is_multi_line(&self) -> bool {
        self.multi_line
    }

    pub fn is_mono(&self) -> bool {
        self.mono
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled || self.element_disabled
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    pub fn selection(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selected_range.clone()]
    }

    pub fn cursor(&self) -> usize {
        self.cursor_offset()
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus_handle, cx);
    }

    pub fn set_value(
        &mut self,
        value: impl Into<SharedString>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        let sanitized = self.sanitize(&value);
        let limited = self.limit_inserted(0, &sanitized);
        if limited == self.text {
            return;
        }
        self.text = limited;
        self.selected_range = self.text.len()..self.text.len();
        self.selection_reversed = false;
        self.marked_range = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.reveal_cursor = true;
        self.revalidate();
        cx.notify();
    }

    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.text.is_empty() {
            return;
        }
        self.select_all(window, cx);
        self.replace_selection("", cx);
    }

    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.placeholder = placeholder.into();
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        if self.disabled != disabled {
            self.disabled = disabled;
            cx.notify();
        }
    }

    pub fn set_error(&mut self, error: Option<SharedString>, cx: &mut Context<Self>) {
        self.error = error;
        cx.notify();
    }

    pub fn select_all(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = 0..self.text.len();
        self.selection_reversed = false;
        self.touch_cursor(cx);
    }

    pub fn select_range(&mut self, range: Range<usize>, cx: &mut Context<Self>) {
        let start = clamp_to_boundary(&self.text, range.start);
        let end = clamp_to_boundary(&self.text, range.end.max(range.start));
        self.selected_range = start..end;
        self.selection_reversed = false;
        self.touch_cursor(cx);
    }

    pub fn insert(&mut self, text: &str, _window: &mut Window, cx: &mut Context<Self>) {
        self.replace_selection(text, cx);
    }

    fn sanitize(&self, text: &str) -> String {
        if self.multi_line {
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            text.replace("\r\n", " ").replace(['\n', '\r'], " ")
        }
    }

    fn limit_inserted(&self, kept_chars: usize, inserted: &str) -> String {
        match self.max_length {
            Some(max_length) => {
                let room = max_length.saturating_sub(kept_chars);
                inserted.chars().take(room).collect()
            }
            None => inserted.to_owned(),
        }
    }

    fn revalidate(&mut self) {
        self.error = self
            .validator
            .as_ref()
            .and_then(|validator| validator(&self.text).err());
    }

    fn handle_focus(&mut self, cx: &mut Context<Self>) {
        self.start_blink(cx);
        cx.emit(InputEvent::Focus);
        cx.notify();
    }

    fn handle_blur(&mut self, cx: &mut Context<Self>) {
        self.blink_task = None;
        self.blink_epoch += 1;
        self.cursor_visible = false;
        self.selecting = false;
        self.marked_range = None;
        cx.emit(InputEvent::Blur);
        cx.notify();
    }

    fn start_blink(&mut self, cx: &mut Context<Self>) {
        self.cursor_visible = true;
        self.blink_epoch += 1;
        let epoch = self.blink_epoch;
        let interval = cx.theme().motion.cursor_blink;
        self.blink_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(interval).await;
                let alive = this
                    .update(cx, |this, cx| {
                        if this.blink_epoch != epoch {
                            return false;
                        }
                        this.cursor_visible = !this.cursor_visible;
                        cx.notify();
                        true
                    })
                    .unwrap_or(false);
                if !alive {
                    break;
                }
            }
        }));
    }

    fn touch_cursor(&mut self, cx: &mut Context<Self>) {
        self.reveal_cursor = true;
        if self.blink_task.is_some() {
            self.start_blink(cx);
        }
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = clamp_to_boundary(&self.text, offset);
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.touch_cursor(cx);
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = clamp_to_boundary(&self.text, offset);
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        self.touch_cursor(cx);
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            text: self.text.clone(),
            selected_range: self.selected_range.clone(),
            selection_reversed: self.selection_reversed,
        }
    }

    fn restore(&mut self, snapshot: Snapshot, cx: &mut Context<Self>) {
        self.text = snapshot.text;
        self.selected_range = snapshot.selected_range;
        self.selection_reversed = snapshot.selection_reversed;
        self.marked_range = None;
        self.revalidate();
        cx.emit(InputEvent::Change);
        self.touch_cursor(cx);
    }

    fn can_edit(&self) -> bool {
        !self.is_disabled() && !self.read_only
    }

    fn replace_selection(&mut self, new_text: &str, cx: &mut Context<Self>) {
        let range = self.selected_range.clone();
        self.replace_range(range, new_text, cx);
    }

    fn replace_range(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        cx: &mut Context<Self>,
    ) -> usize {
        let start = clamp_to_boundary(&self.text, range.start);
        let end = clamp_to_boundary(&self.text, range.end.max(start));
        let sanitized = self.sanitize(new_text);
        let kept_chars = self.text[..start].chars().count() + self.text[end..].chars().count();
        let inserted = self.limit_inserted(kept_chars, &sanitized);
        if start == end && inserted.is_empty() {
            return 0;
        }
        self.undo_stack.push(self.snapshot());
        if self.undo_stack.len() > HISTORY_LIMIT {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.text.replace_range(start..end, &inserted);
        let caret = start + inserted.len();
        self.selected_range = caret..caret;
        self.selection_reversed = false;
        self.marked_range = None;
        self.preferred_x = None;
        self.revalidate();
        cx.emit(InputEvent::Change);
        self.touch_cursor(cx);
        inserted.len()
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if self.selected_range.is_empty() {
            let previous = previous_grapheme_boundary(&self.text, self.cursor_offset());
            self.selected_range = previous..self.cursor_offset();
        }
        self.replace_selection("", cx);
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if self.selected_range.is_empty() {
            let next = next_grapheme_boundary(&self.text, self.cursor_offset());
            self.selected_range = self.cursor_offset()..next;
        }
        self.replace_selection("", cx);
    }

    fn delete_word_left(&mut self, _: &DeleteWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if self.selected_range.is_empty() {
            let previous = previous_word_boundary(&self.text, self.cursor_offset());
            self.selected_range = previous..self.cursor_offset();
        }
        self.replace_selection("", cx);
    }

    fn delete_word_right(&mut self, _: &DeleteWordRight, _: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if self.selected_range.is_empty() {
            let next = next_word_boundary(&self.text, self.cursor_offset());
            self.selected_range = self.cursor_offset()..next;
        }
        self.replace_selection("", cx);
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        if self.selected_range.is_empty() {
            self.move_to(
                previous_grapheme_boundary(&self.text, self.cursor_offset()),
                cx,
            );
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        if self.selected_range.is_empty() {
            self.move_to(next_grapheme_boundary(&self.text, self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(
            previous_grapheme_boundary(&self.text, self.cursor_offset()),
            cx,
        );
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(next_grapheme_boundary(&self.text, self.cursor_offset()), cx);
    }

    fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.move_to(previous_word_boundary(&self.text, self.cursor_offset()), cx);
    }

    fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.move_to(next_word_boundary(&self.text, self.cursor_offset()), cx);
    }

    fn select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(previous_word_boundary(&self.text, self.cursor_offset()), cx);
    }

    fn select_word_right(&mut self, _: &SelectWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(next_word_boundary(&self.text, self.cursor_offset()), cx);
    }

    fn vertical_target(&mut self, direction: i32) -> Option<usize> {
        let layout = self.layout.as_ref()?;
        let cursor = self.cursor_offset();
        let position = layout.position_for_offset(cursor);
        let x = self.preferred_x.unwrap_or(position.x);
        self.preferred_x = Some(x);
        let target_y = position.y + layout.line_height * direction as f32 + layout.line_height / 2.;
        if target_y < px(0.) {
            return Some(0);
        }
        if target_y > layout.content_size.height {
            return Some(self.text.len());
        }
        Some(layout.offset_for_position(point(x, target_y)))
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        if !self.multi_line {
            cx.propagate();
            return;
        }
        if let Some(target) = self.vertical_target(-1) {
            let preferred = self.preferred_x;
            self.move_to(target, cx);
            self.preferred_x = preferred;
        }
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        if !self.multi_line {
            cx.propagate();
            return;
        }
        if let Some(target) = self.vertical_target(1) {
            let preferred = self.preferred_x;
            self.move_to(target, cx);
            self.preferred_x = preferred;
        }
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        if !self.multi_line {
            self.select_to(0, cx);
            return;
        }
        if let Some(target) = self.vertical_target(-1) {
            let preferred = self.preferred_x;
            self.select_to(target, cx);
            self.preferred_x = preferred;
        }
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        if !self.multi_line {
            self.select_to(self.text.len(), cx);
            return;
        }
        if let Some(target) = self.vertical_target(1) {
            let preferred = self.preferred_x;
            self.select_to(target, cx);
            self.preferred_x = preferred;
        }
    }

    fn line_start(&self, offset: usize) -> usize {
        if !self.multi_line {
            return 0;
        }
        self.text[..offset]
            .rfind('\n')
            .map(|index| index + 1)
            .unwrap_or(0)
    }

    fn line_end(&self, offset: usize) -> usize {
        if !self.multi_line {
            return self.text.len();
        }
        self.text[offset..]
            .find('\n')
            .map(|index| offset + index)
            .unwrap_or(self.text.len())
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.move_to(self.line_start(self.cursor_offset()), cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.move_to(self.line_end(self.cursor_offset()), cx);
    }

    fn select_to_home(&mut self, _: &SelectToHome, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(self.line_start(self.cursor_offset()), cx);
    }

    fn select_to_end(&mut self, _: &SelectToEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.preferred_x = None;
        self.select_to(self.line_end(self.cursor_offset()), cx);
    }

    fn select_all_action(&mut self, _: &SelectAll, window: &mut Window, cx: &mut Context<Self>) {
        self.select_all(window, cx);
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.selected_text().to_owned()));
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.selected_text().to_owned()));
        if self.can_edit() {
            self.replace_selection("", cx);
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_selection(&text, cx);
        }
    }

    fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(self.snapshot());
            self.restore(snapshot, cx);
        }
    }

    fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if !self.can_edit() {
            return;
        }
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(self.snapshot());
            self.restore(snapshot, cx);
        }
    }

    fn enter(&mut self, _: &Enter, _: &mut Window, cx: &mut Context<Self>) {
        if self.multi_line {
            if self.can_edit() {
                self.replace_selection("\n", cx);
            }
        } else {
            cx.emit(InputEvent::Submit);
        }
    }

    fn submit(&mut self, _: &Submit, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(InputEvent::Submit);
    }

    fn offset_for_mouse(&self, position: Point<Pixels>) -> usize {
        let Some(layout) = self.layout.as_ref() else {
            return 0;
        };
        if self.text.is_empty() {
            return 0;
        }
        let local = point(
            position.x - layout.bounds.left() + self.scroll.x,
            position.y - layout.bounds.top() + self.scroll.y,
        );
        layout.offset_for_position(local)
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_disabled() {
            return;
        }
        window.focus(&self.focus_handle, cx);
        self.preferred_x = None;
        let offset = self.offset_for_mouse(event.position);
        match event.click_count {
            2 => {
                let range = word_range_at(&self.text, offset);
                self.selected_range = range;
                self.selection_reversed = false;
                self.touch_cursor(cx);
            }
            count if count >= 3 => {
                let start = self.line_start(offset);
                let end = self.line_end(offset);
                self.selected_range = start..end;
                self.selection_reversed = false;
                self.touch_cursor(cx);
            }
            _ => {
                self.selecting = true;
                if event.modifiers.shift {
                    self.select_to(offset, cx);
                } else {
                    self.move_to(offset, cx);
                }
            }
        }
    }

    fn prepare_context_menu(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_disabled() {
            return;
        }
        window.focus(&self.focus_handle, cx);
        self.selecting = false;
        let offset = self.offset_for_mouse(position);
        let inside = !self.selected_range.is_empty()
            && self.selected_range.start <= offset
            && offset <= self.selected_range.end;
        if !inside {
            self.preferred_x = None;
            self.move_to(offset, cx);
        }
    }

    fn context_menu_entries(state: &Entity<Self>, cx: &App) -> Vec<ContextMenuEntry> {
        let strings = cx.t();
        let (has_selection, can_edit, is_empty) = {
            let state = state.read(cx);
            (
                !state.selected_range.is_empty(),
                state.can_edit(),
                state.text.is_empty(),
            )
        };
        let can_paste = can_edit
            && cx
                .read_from_clipboard()
                .and_then(|item| item.text())
                .is_some_and(|text| !text.is_empty());
        let (cut, copy, paste, all) = (state.clone(), state.clone(), state.clone(), state.clone());
        vec![
            text_menu::entry(
                strings.ui_cut(),
                KEY_CUT,
                can_edit && has_selection,
                cx,
                move |window, cx| cut.update(cx, |state, cx| state.cut(&Cut, window, cx)),
            ),
            text_menu::entry(
                strings.docs_copy(),
                KEY_COPY,
                has_selection,
                cx,
                move |window, cx| copy.update(cx, |state, cx| state.copy(&Copy, window, cx)),
            ),
            text_menu::entry(
                strings.ui_paste(),
                KEY_PASTE,
                can_paste,
                cx,
                move |window, cx| paste.update(cx, |state, cx| state.paste(&Paste, window, cx)),
            ),
            text_menu::entry(
                strings.ui_select_all(),
                KEY_SELECT_ALL,
                !is_empty,
                cx,
                move |window, cx| all.update(cx, |state, cx| state.select_all(window, cx)),
            ),
        ]
    }

    fn on_drag_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.selecting {
            let offset = self.offset_for_mouse(event.position);
            self.select_to(offset, cx);
        }
    }

    fn on_drag_end(&mut self, cx: &mut Context<Self>) {
        if self.selecting {
            self.selecting = false;
            cx.notify();
        }
    }

    fn on_scroll_wheel(
        &mut self,
        event: &ScrollEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(layout) = self.layout.as_ref() else {
            return;
        };
        if !self.multi_line {
            return;
        }
        let max = (layout.content_size.height - layout.bounds.size.height).max(px(0.));
        if max <= px(0.) {
            return;
        }
        let delta = event.delta.pixel_delta(layout.line_height);
        let next = (self.scroll.y - delta.y).max(px(0.)).min(max);
        if next != self.scroll.y {
            self.scroll.y = next;
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.text.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }
}

impl EntityInputHandler for InputState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        self.replace_range(range, new_text, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_edit() {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let start = clamp_to_boundary(&self.text, range.start);
        let inserted = self.replace_range(range, new_text, cx);
        self.marked_range = (inserted > 0).then_some(start..start + inserted);
        if let Some(selected_utf16) = new_selected_range_utf16 {
            let marked_text = &self.text[start..start + inserted];
            let mut utf8_start = 0;
            let mut utf8_end = 0;
            let mut utf16_count = 0;
            for ch in marked_text.chars() {
                if utf16_count < selected_utf16.start {
                    utf8_start += ch.len_utf8();
                }
                if utf16_count < selected_utf16.end {
                    utf8_end += ch.len_utf8();
                }
                utf16_count += ch.len_utf16();
            }
            self.selected_range = start + utf8_start..start + utf8_end.max(utf8_start);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let start = layout.position_for_offset(range.start);
        let end = layout.position_for_offset(range.end);
        let origin = point(bounds.left() - self.scroll.x, bounds.top() - self.scroll.y);
        Some(Bounds::from_corners(
            point(origin.x + start.x, origin.y + start.y),
            point(
                origin.x + end.x.max(start.x),
                origin.y + start.y + layout.line_height,
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        self.layout.as_ref()?;
        let offset = self.offset_for_mouse(point);
        Some(self.offset_to_utf16(offset))
    }
}

struct TextElement {
    state: Entity<InputState>,
    text_color: Hsla,
    placeholder_color: Hsla,
    selection_color: Hsla,
    cursor_color: Hsla,
}

struct TextPrepaint {
    lines: Vec<LineSnapshot>,
    placeholder: Option<Vec<WrappedLine>>,
    selections: Vec<Bounds<Pixels>>,
    cursor: Option<Bounds<Pixels>>,
    scroll: Point<Pixels>,
    line_height: Pixels,
}


impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = TextPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let state = self.state.read(cx);
        let rows = if state.multi_line {
            state.visual_rows.clamp(state.min_rows, state.max_rows)
        } else {
            1
        };
        let mut style = Style {
            flex_grow: 1.,
            flex_shrink: 1.,
            flex_basis: relative(0.).into(),
            ..Default::default()
        };
        style.min_size.width = px(0.).into();
        style.size.height = (window.line_height() * rows as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let (text, placeholder, selected_range, cursor, marked_range, multi_line, focused) = {
            let state = self.state.read(cx);
            (
                SharedString::from(state.text.clone()),
                state.placeholder.clone(),
                state.selected_range.clone(),
                state.cursor_offset(),
                state.marked_range.clone(),
                state.multi_line,
                state.focus_handle.is_focused(window) && state.cursor_visible,
            )
        };
        let wrap_width = multi_line.then_some(bounds.size.width);
        let base_run = TextRun {
            len: text.len(),
            font: style.font(),
            color: self.text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs: Vec<TextRun> = match marked_range.as_ref() {
            Some(marked) if marked.end <= text.len() && marked.start <= marked.end => vec![
                TextRun {
                    len: marked.start,
                    ..base_run.clone()
                },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(self.text_color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..base_run.clone()
                },
                TextRun {
                    len: text.len() - marked.end,
                    ..base_run.clone()
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect(),
            _ => vec![base_run.clone()],
        };
        let shaped = window
            .text_system()
            .shape_text(text.clone(), font_size, &runs, wrap_width, None)
            .unwrap_or_default();
        let mut lines = Vec::with_capacity(shaped.len());
        let mut start = 0;
        let mut top = px(0.);
        let mut content_width = px(0.);
        for line in shaped {
            let rows = rows_for_line(&line);
            let len = line.len();
            let height = line_height * rows.len() as f32;
            content_width = content_width.max(line.unwrapped_layout.width);
            lines.push(LineSnapshot {
                start,
                top,
                line: Rc::new(line),
                rows,
            });
            start += len + 1;
            top += height;
        }
        let snapshot = LayoutSnapshot {
            lines: lines.clone(),
            line_height,
            bounds,
            content_size: size(content_width, top),
        };
        let cursor_position = snapshot.position_for_offset(cursor);
        let row_count = snapshot.row_count();
        let cursor_width = px(1.5);

        let placeholder_lines = (text.is_empty() && !placeholder.is_empty()).then(|| {
            let run = TextRun {
                len: placeholder.len(),
                font: style.font(),
                color: self.placeholder_color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            window
                .text_system()
                .shape_text(placeholder.clone(), font_size, &[run], wrap_width, None)
                .map(|lines| lines.into_vec())
                .unwrap_or_default()
        });

        let state_entity = self.state.clone();
        let (scroll, rows_changed) = self.state.update(cx, |state, _| {
            let mut scroll = state.scroll;
            if multi_line {
                scroll.x = px(0.);
                let max = (snapshot.content_size.height - bounds.size.height).max(px(0.));
                if state.reveal_cursor {
                    if cursor_position.y < scroll.y {
                        scroll.y = cursor_position.y;
                    } else if cursor_position.y + line_height > scroll.y + bounds.size.height {
                        scroll.y = cursor_position.y + line_height - bounds.size.height;
                    }
                }
                scroll.y = scroll.y.max(px(0.)).min(max);
            } else {
                scroll.y = px(0.);
                let visible = bounds.size.width - cursor_width;
                let max = (snapshot.content_size.width - visible).max(px(0.));
                if state.reveal_cursor {
                    if cursor_position.x < scroll.x {
                        scroll.x = cursor_position.x;
                    } else if cursor_position.x > scroll.x + visible {
                        scroll.x = cursor_position.x - visible;
                    }
                }
                scroll.x = scroll.x.max(px(0.)).min(max);
            }
            state.reveal_cursor = false;
            state.scroll = scroll;
            let rows_changed = state.visual_rows != row_count;
            state.visual_rows = row_count;
            state.layout = Some(snapshot.clone());
            (scroll, rows_changed)
        });
        if rows_changed && multi_line {
            window.defer(cx, move |_, cx| {
                state_entity.update(cx, |_, cx| cx.notify());
            });
        }

        let origin = point(bounds.left() - scroll.x, bounds.top() - scroll.y);
        let mut selections = Vec::new();
        if !selected_range.is_empty() {
            for line in &lines {
                if selected_range.end < line.start || selected_range.start > line.end() {
                    continue;
                }
                for (row_index, row) in line.rows.iter().enumerate() {
                    let row_start = line.start + row.start;
                    let row_end = line.start + row.end;
                    let from = selected_range.start.max(row_start);
                    let to = selected_range.end.min(row_end);
                    let is_last_row = row_index + 1 == line.rows.len();
                    let includes_break = is_last_row && selected_range.end > line.end();
                    if from > to || (from == to && !includes_break) {
                        continue;
                    }
                    let layout = &line.line.unwrapped_layout;
                    let left = layout.x_for_index(from - line.start) - row.start_x;
                    let mut right = layout.x_for_index(to - line.start) - row.start_x;
                    if includes_break {
                        right += line_height / 3.;
                    }
                    let top = line.top + line_height * row_index as f32;
                    selections.push(Bounds::from_corners(
                        point(origin.x + left, origin.y + top),
                        point(origin.x + right, origin.y + top + line_height),
                    ));
                }
            }
        }
        let cursor_bounds = (focused && selected_range.is_empty()).then(|| {
            Bounds::new(
                point(origin.x + cursor_position.x, origin.y + cursor_position.y),
                size(cursor_width, line_height),
            )
        });
        TextPrepaint {
            lines,
            placeholder: placeholder_lines,
            selections,
            cursor: cursor_bounds,
            scroll,
            line_height,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (focus_handle, accepts_input) = {
            let state = self.state.read(cx);
            (state.focus_handle.clone(), state.can_edit())
        };
        if accepts_input {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.state.clone()),
                cx,
            );
        }
        let origin = point(
            bounds.left() - prepaint.scroll.x,
            bounds.top() - prepaint.scroll.y,
        );
        let line_height = prepaint.line_height;
        let selection_color = self.selection_color;
        let cursor_color = self.cursor_color;
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for selection in prepaint.selections.drain(..) {
                window.paint_quad(fill(selection, selection_color));
            }
            if let Some(placeholder) = prepaint.placeholder.take() {
                let mut top = px(0.);
                for line in placeholder {
                    let height = line.size(line_height).height;
                    let _ = line.paint(
                        point(bounds.left(), bounds.top() + top),
                        line_height,
                        TextAlign::Left,
                        None,
                        window,
                        cx,
                    );
                    top += height;
                }
            }
            for line in &prepaint.lines {
                let line_origin = point(origin.x, origin.y + line.top);
                let height = line.height(line_height);
                if line_origin.y > bounds.bottom() || line_origin.y + height < bounds.top() {
                    continue;
                }
                let _ =
                    line.line
                        .paint(line_origin, line_height, TextAlign::Left, None, window, cx);
            }
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(fill(cursor, cursor_color));
            }
        });
    }
}

fn listener<E: 'static>(
    state: &Entity<InputState>,
    handler: impl Fn(&mut InputState, &E, &mut Window, &mut Context<InputState>) + 'static,
) -> impl Fn(&E, &mut Window, &mut App) + 'static {
    let state = state.downgrade();
    move |event, window, cx| {
        state
            .update(cx, |state, cx| handler(state, event, window, cx))
            .ok();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InputSize {
    Small,
    #[default]
    Default,
    Large,
}

pub struct Input {
    state: Entity<InputState>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    cleanable: bool,
    size: InputSize,
    disabled: bool,
    invalid: bool,
    bordered: bool,
    width: Option<DefiniteLength>,
}

crate::impl_into_element!(Input);

impl Input {
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            prefix: None,
            suffix: None,
            cleanable: false,
            size: InputSize::default(),
            disabled: false,
            invalid: false,
            bordered: true,
            width: None,
        }
    }

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
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

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = self.state;
        let element_disabled = self.disabled || super::util::ControllerMode::is_active(cx);
        state.update(cx, |state, _| state.element_disabled = element_disabled);
        let (focus_handle, disabled, mono, multi_line, has_error, is_empty, selecting) = {
            let state = state.read(cx);
            (
                state.focus_handle.clone(),
                state.is_disabled(),
                state.mono,
                state.multi_line,
                state.error.is_some(),
                state.text.is_empty(),
                state.selecting,
            )
        };
        let focused = focus_handle.is_focused(window);
        let focus_handle = focus_handle.tab_stop(!disabled);
        let invalid = self.invalid || has_error;
        let border = if invalid {
            theme.colors.error
        } else if focused {
            theme.colors.accent_primary
        } else {
            theme.colors.border_medium
        };
        let hover_border = if invalid {
            theme.colors.error
        } else {
            theme.colors.accent_primary.opacity(theme.alpha.vignette)
        };
        let (padding_h, padding_v, font) = match self.size {
            InputSize::Small => (px(8.), px(4.), sizes.font_sm),
            InputSize::Default => (px(10.), px(6.), sizes.font_md),
            InputSize::Large => (px(12.), px(10.), sizes.font_lg),
        };
        let clear_state = state.clone();
        let clear_tooltip = cx.t().ui_clear_input();
        let text_menu = TextMenu::new(("yp-input-menu", state.entity_id()), window, cx);
        let menu_overlay = text_menu.overlay(cx);
        let menu_state = state.clone();
        let move_state = state.downgrade();
        let up_state = state.downgrade();
        let accessible_state = state.clone();
        let placeholder = state.read(cx).placeholder.clone();
        let text_element = TextElement {
            state: state.clone(),
            text_color: if disabled {
                theme.colors.text_muted
            } else {
                theme.colors.text_primary
            },
            placeholder_color: Color::Placeholder.color(cx),
            selection_color: theme.colors.accent_primary.opacity(theme.alpha.soft),
            cursor_color: theme.colors.accent_primary,
        };
        h_flex()
            .id(("yp-input", state.entity_id()))
            .role(if multi_line {
                gpui::Role::MultilineTextInput
            } else {
                gpui::Role::TextInput
            })
            .aria_value(state.read(cx).text.clone())
            .aria_label(placeholder.clone())
            .aria_placeholder(placeholder)
            .when(!disabled && !state.read(cx).read_only, |el| {
                el.on_a11y_action(gpui::AccessibleAction::SetValue, move |data, w, cx| {
                    if let Some(gpui::accesskit::ActionData::Value(text)) = data {
                        accessible_state.update(cx, |input, cx| {
                            input.select_all(w, cx);
                            input.replace_selection(text, cx);
                        });
                    }
                })
            })
            .key_context(INPUT_CONTEXT)
            .track_focus(&focus_handle)
            .map(|this| match self.width {
                Some(width) => this.w(width),
                None => this.w_full(),
            })
            .when(multi_line, |this| this.items_start())
            .gap(sizes.spacing_md)
            .px(padding_h)
            .py(padding_v)
            .rounded(Sizes::CONTROL_RADIUS)
            .bg(theme.colors.input_background)
            .when(self.bordered, |this| this.border_1().border_color(border))
            .text_size(font)
            .line_height(relative(1.3))
            .text_color(theme.colors.text_primary)
            .when(mono, |this| this.font_mono(cx))
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor(CursorStyle::IBeam)
                    .when(self.bordered && !focused, |this| {
                        this.hover(move |style| style.border_color(hover_border))
                    })
                    .on_action(listener(&state, InputState::backspace))
                    .on_action(listener(&state, InputState::delete))
                    .on_action(listener(&state, InputState::delete_word_left))
                    .on_action(listener(&state, InputState::delete_word_right))
                    .on_action(listener(&state, InputState::left))
                    .on_action(listener(&state, InputState::right))
                    .on_action(listener(&state, InputState::up))
                    .on_action(listener(&state, InputState::down))
                    .on_action(listener(&state, InputState::select_left))
                    .on_action(listener(&state, InputState::select_right))
                    .on_action(listener(&state, InputState::select_up))
                    .on_action(listener(&state, InputState::select_down))
                    .on_action(listener(&state, InputState::word_left))
                    .on_action(listener(&state, InputState::word_right))
                    .on_action(listener(&state, InputState::select_word_left))
                    .on_action(listener(&state, InputState::select_word_right))
                    .on_action(listener(&state, InputState::select_all_action))
                    .on_action(listener(&state, InputState::home))
                    .on_action(listener(&state, InputState::end))
                    .on_action(listener(&state, InputState::select_to_home))
                    .on_action(listener(&state, InputState::select_to_end))
                    .on_action(listener(&state, InputState::copy))
                    .on_action(listener(&state, InputState::cut))
                    .on_action(listener(&state, InputState::paste))
                    .on_action(listener(&state, InputState::undo))
                    .on_action(listener(&state, InputState::redo))
                    .on_action(listener(&state, InputState::enter))
                    .on_action(listener(&state, InputState::submit))
                    .on_mouse_down(
                        MouseButton::Left,
                        listener(&state, InputState::on_mouse_down),
                    )
                    .on_scroll(listener(&state, InputState::on_scroll_wheel))
            })
            .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                cx.stop_propagation();
                menu_state.update(cx, |state, cx| {
                    state.prepare_context_menu(event.position, window, cx);
                });
                let entries = InputState::context_menu_entries(&menu_state, cx);
                text_menu.open(event.position, entries, window, cx);
            })
            .children(menu_overlay)
            .children(self.prefix.map(|prefix| div().flex_none().child(prefix)))
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .child(text_element)
                    .child(pointer_capture(
                        selecting,
                        move |event: &MouseMoveEvent, _, cx| {
                            move_state
                                .update(cx, |state, cx| state.on_drag_move(event, cx))
                                .ok();
                        },
                        move |_: &MouseUpEvent, _, cx| {
                            up_state.update(cx, |state, cx| state.on_drag_end(cx)).ok();
                        },
                    )),
            )
            .when(self.cleanable && !is_empty && !disabled, |this| {
                this.child(
                    IconButton::new("yp-input-clear", IconName::Close)
                        .icon_size(IconSize::XSmall)
                        .icon_color(Color::Muted)
                        .size(ButtonSize::Compact)
                        .accessible_label(clear_tooltip)
                        .tooltip(Tooltip::text(clear_tooltip))
                        .on_click(move |_, window, cx| {
                            clear_state.update(cx, |state, cx| {
                                state.clear(window, cx);
                                state.focus(window, cx);
                            });
                        }),
                )
            })
            .children(self.suffix.map(|suffix| div().flex_none().child(suffix)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    fn new_input(
        cx: &mut TestAppContext,
        build: impl FnOnce(InputState) -> InputState + 'static,
    ) -> (Entity<InputView>, &mut gpui::VisualTestContext) {
        cx.update(crate::init_for_tests);
        cx.add_window_view(|window, cx| {
            let state = cx.new(|cx| build(InputState::new(window, cx)));
            InputView { state }
        })
    }

    struct InputView {
        state: Entity<InputState>,
    }

    impl gpui::Render for InputView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(Input::new(&self.state))
        }
    }

    fn value(view: &Entity<InputView>, cx: &mut gpui::VisualTestContext) -> String {
        view.read_with(cx, |view, cx| view.state.read(cx).value().to_owned())
    }

    fn focus(view: &Entity<InputView>, cx: &mut gpui::VisualTestContext) {
        view.update_in(cx, |view, window, cx| window.focus(&view.state.focus_handle(cx), cx));
        cx.run_until_parked();
    }

    #[test]
    fn word_boundaries_skip_whitespace_and_punctuation_runs() {
        let text = "hello  wide world";
        assert_eq!(previous_word_boundary(text, 17), 12);
        assert_eq!(previous_word_boundary(text, 12), 7);
        assert_eq!(previous_word_boundary(text, 7), 0);
        assert_eq!(previous_word_boundary(text, 0), 0);
        assert_eq!(next_word_boundary(text, 0), 5);
        assert_eq!(next_word_boundary(text, 5), 11);
        assert_eq!(next_word_boundary(text, 17), 17);
        assert_eq!(word_range_at(text, 8), 7..11);
        assert_eq!(word_range_at("", 0), 0..0);
    }

    #[test]
    fn grapheme_boundaries_keep_clusters_together() {
        let text = "a\u{1F469}\u{200D}\u{1F680}b";
        let after_a = 1;
        let after_cluster = text.len() - 1;
        assert_eq!(next_grapheme_boundary(text, after_a), after_cluster);
        assert_eq!(previous_grapheme_boundary(text, after_cluster), after_a);
        assert_eq!(previous_grapheme_boundary(text, 0), 0);
        assert_eq!(next_grapheme_boundary(text, text.len()), text.len());
    }

    #[gpui::test]
    fn typing_inserts_text_and_emits_change(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| state);
        let changes = std::rc::Rc::new(std::cell::Cell::new(0));
        let counter = changes.clone();
        let state = view.read_with(cx, |view, _| view.state.clone());
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&state, move |_, event: &InputEvent, _| {
                if *event == InputEvent::Change {
                    counter.set(counter.get() + 1);
                }
            })
        });
        focus(&view, cx);
        cx.simulate_input("abc");
        assert_eq!(value(&view, cx), "abc");
        assert_eq!(changes.get(), 3);
        cx.simulate_keystrokes("left backspace");
        assert_eq!(value(&view, cx), "ac");
        cx.simulate_keystrokes("home delete");
        assert_eq!(value(&view, cx), "c");
    }

    #[gpui::test]
    fn selection_shortcuts_replace_and_copy(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| state.default_value("hello world"));
        focus(&view, cx);
        cx.simulate_keystrokes("ctrl-shift-left");
        view.read_with(cx, |view, cx| {
            assert_eq!(view.state.read(cx).selected_text(), "world");
        });
        cx.simulate_keystrokes("secondary-c");
        let copied = cx.read_from_clipboard().and_then(|item| item.text());
        assert_eq!(copied.as_deref(), Some("world"));
        cx.simulate_input("X");
        assert_eq!(value(&view, cx), "hello X");
        cx.simulate_keystrokes("secondary-a secondary-x");
        assert_eq!(value(&view, cx), "");
        cx.simulate_keystrokes("secondary-v secondary-v");
        assert_eq!(value(&view, cx), "hello Xhello X");
    }

    #[gpui::test]
    fn undo_and_redo_walk_the_edit_history(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| state);
        focus(&view, cx);
        cx.simulate_input("ab");
        cx.simulate_keystrokes("secondary-z");
        assert_eq!(value(&view, cx), "a");
        cx.simulate_keystrokes("secondary-z secondary-z");
        assert_eq!(value(&view, cx), "");
        cx.simulate_keystrokes("secondary-y secondary-shift-z");
        assert_eq!(value(&view, cx), "ab");
    }

    #[gpui::test]
    fn max_length_counts_characters_not_bytes(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| state.max_length(4).default_value("äöüßxyz"));
        assert_eq!(value(&view, cx), "äöüß");
        focus(&view, cx);
        cx.simulate_input("q");
        assert_eq!(value(&view, cx), "äöüß");
        cx.simulate_keystrokes("backspace");
        cx.simulate_input("简体");
        assert_eq!(value(&view, cx), "äöü简");
    }

    #[gpui::test]
    fn single_line_flattens_newlines_and_submits_on_enter(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| state);
        let submitted = std::rc::Rc::new(std::cell::Cell::new(false));
        let flag = submitted.clone();
        let state = view.read_with(cx, |view, _| view.state.clone());
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&state, move |_, event: &InputEvent, _| {
                if *event == InputEvent::Submit {
                    flag.set(true);
                }
            })
        });
        focus(&view, cx);
        cx.write_to_clipboard(ClipboardItem::new_string("one\r\ntwo\nthree".to_owned()));
        cx.simulate_keystrokes("secondary-v");
        assert_eq!(value(&view, cx), "one two three");
        cx.simulate_keystrokes("enter");
        assert!(submitted.get());
        assert_eq!(value(&view, cx), "one two three");
    }

    #[gpui::test]
    fn multi_line_enter_inserts_a_line_break_and_home_end_stay_on_the_line(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = new_input(cx, |state| state.multi_line(3));
        focus(&view, cx);
        cx.simulate_input("ab");
        cx.simulate_keystrokes("enter");
        cx.simulate_input("cd");
        assert_eq!(value(&view, cx), "ab\ncd");
        cx.simulate_keystrokes("home");
        cx.simulate_input("X");
        assert_eq!(value(&view, cx), "ab\nXcd");
        cx.simulate_keystrokes("up end");
        cx.simulate_input("Y");
        assert_eq!(value(&view, cx), "abY\nXcd");
    }

    #[gpui::test]
    fn validation_reports_the_error_and_set_value_is_silent(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| {
            state.validate(|text| {
                if text.contains(' ') {
                    Err("no spaces".into())
                } else {
                    Ok(())
                }
            })
        });
        let events = std::rc::Rc::new(std::cell::Cell::new(0));
        let counter = events.clone();
        let state = view.read_with(cx, |view, _| view.state.clone());
        let _subscription = cx.update(|_, cx| {
            cx.subscribe(&state, move |_, _: &InputEvent, _| {
                counter.set(counter.get() + 1)
            })
        });
        view.update_in(cx, |view, window, cx| {
            view.state
                .update(cx, |state, cx| state.set_value("a b", window, cx));
        });
        cx.run_until_parked();
        assert_eq!(events.get(), 0);
        view.read_with(cx, |view, cx| {
            let state = view.state.read(cx);
            assert_eq!(state.error().map(|error| error.as_ref()), Some("no spaces"));
            assert!(!state.is_valid());
        });
    }

    #[gpui::test]
    fn disabled_input_ignores_typing(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| state.default_value("keep"));
        view.update_in(cx, |view, _, cx| {
            view.state
                .update(cx, |state, cx| state.set_disabled(true, cx));
        });
        focus(&view, cx);
        cx.simulate_input("x");
        cx.simulate_keystrokes("backspace");
        assert_eq!(value(&view, cx), "keep");
    }

    #[gpui::test]
    fn ime_composition_marks_text_until_it_is_committed(cx: &mut TestAppContext) {
        let (view, cx) = new_input(cx, |state| state.default_value("a"));
        view.update_in(cx, |view, window, cx| {
            view.state.update(cx, |state, cx| {
                state.replace_and_mark_text_in_range(None, "ni", Some(2..2), window, cx);
                assert_eq!(state.value(), "ani");
                assert_eq!(state.marked_text_range(window, cx), Some(1..3));
                state.replace_text_in_range(None, "你", window, cx);
                assert_eq!(state.value(), "a你");
                assert_eq!(state.marked_text_range(window, cx), None);
                let selection = state.selected_text_range(false, window, cx).unwrap();
                assert_eq!(selection.range, 2..2);
            });
        });
    }
}
