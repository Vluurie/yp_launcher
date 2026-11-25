use super::MenuFieldState;
use gpui::{Bounds, Context, EntityInputHandler, Pixels, Point, UTF16Selection, Window, point, px};
use std::ops::Range;

fn byte_index(value: &str, utf16: usize) -> usize {
    let mut used = 0;
    for (i, ch) in value.char_indices() {
        if utf16 < used + ch.len_utf16() {
            return i;
        }
        used += ch.len_utf16();
    }
    value.len()
}
impl MenuFieldState {
    fn bytes(&self, r: Range<usize>) -> Range<usize> {
        byte_index(&self.value, r.start)..byte_index(&self.value, r.end.max(r.start))
    }
    fn utf16(&self, byte: usize) -> usize {
        self.value[..byte].encode_utf16().count()
    }
    fn units(&self, r: Range<usize>) -> Range<usize> {
        self.utf16(r.start)..self.utf16(r.end)
    }
    fn replacement(&self, range: Option<Range<usize>>) -> Range<usize> {
        range
            .map(|r| self.bytes(r))
            .or_else(|| self.mark.clone())
            .unwrap_or_else(|| self.range())
    }
}
impl EntityInputHandler for MenuFieldState {
    fn text_for_range(
        &mut self,
        r: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.bytes(r);
        *actual = Some(self.units(r.clone()));
        Some(self.value[r].to_owned())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.units(self.range()),
            reversed: self.head < self.anchor,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.mark.clone().map(|r| self.units(r))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.mark = None;
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        value: &str,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace(self.replacement(r), value, w, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        v: &str,
        sel: Option<Range<usize>>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = self.replace(self.replacement(r), v, w, cx);
        if let Some(sel) = sel {
            let t = &self.value[r.clone()];
            self.anchor = r.start + byte_index(t, sel.start);
            self.head = r.start + byte_index(t, sel.end);
        }
        self.mark = (!r.is_empty()).then_some(r);
        w.notify_text_selection_changed(&self.focus);
    }
    fn bounds_for_range(
        &mut self,
        r: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let r = self.bytes(r);
        let x = |i| bounds.left() + px((12. - self.scroll + self.x(i)) * self.scale);
        Some(Bounds::from_corners(
            point(x(r.start), bounds.top()),
            point(x(r.end).max(x(r.start) + px(1.)), bounds.bottom()),
        ))
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.utf16(self.byte_at(p.x)))
    }
    fn set_selected_text_range(&mut self, r: Range<usize>, w: &mut Window, cx: &mut Context<Self>) {
        let r = self.bytes(r);
        self.anchor = r.start;
        self.select(r.end, true, w, cx);
    }
    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.value.encode_utf16().count())
    }
}
