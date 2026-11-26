#[cfg(test)]
mod tests;

use crate::motion::{row_alpha, row_slide};
use crate::text_reveal::ListRowText;
use crate::{PANEL, Rect, Surface};
use gpui::{
    AnyElement, App, ElementId, IntoElement, Pixels, RenderOnce, UniformListScrollHandle, Window,
    div, prelude::*, px, uniform_list,
};
use std::ops::Range;
use std::rc::Rc;
use std::time::{Duration, Instant};

type Row = Rc<dyn Fn(usize, &mut Window, &mut App) -> AnyElement>;

struct ListState {
    started: Instant,
    scroll: UniformListScrollHandle,
    opening: Option<(Pixels, Range<usize>)>,
    scrolled: bool,
}

impl ListState {
    fn opening_rows(
        &mut self,
        handle: &UniformListScrollHandle,
        row_height: Pixels,
    ) -> Option<Range<usize>> {
        let handle = handle.0.borrow();
        let height = handle.last_item_size?.item.height;
        if self.scrolled || height <= px(0.) || row_height <= px(0.) {
            return None;
        }
        let top = -handle.base_handle.offset().y;
        let (initial_top, rows) = self.opening.get_or_insert_with(|| {
            let first = (top / row_height).floor() as usize;
            let end = ((top + height) / row_height).ceil() as usize;
            (top, first..end)
        });
        self.scrolled = top != *initial_top;
        (!self.scrolled).then(|| rows.clone())
    }
}

#[derive(IntoElement)]
pub struct MenuList {
    id: ElementId,
    count: usize,
    scale: f32,
    item_layout: bool,
    row: Row,
    scroll: Option<UniformListScrollHandle>,
}
impl MenuList {
    pub fn new<E: IntoElement>(
        id: impl Into<ElementId>,
        count: usize,
        row: impl Fn(usize, &mut Window, &mut App) -> E + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            count,
            row: Rc::new(move |i, w, cx| row(i, w, cx).into_any_element()),
            scale: 1.,
            item_layout: false,
            scroll: None,
        }
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    pub fn item_layout(mut self, enabled: bool) -> Self {
        self.item_layout = enabled;
        self
    }
    pub fn track_scroll(mut self, handle: UniformListScrollHandle) -> Self {
        self.scroll = Some(handle);
        self
    }
}
impl RenderOnce for MenuList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| ListState {
            started: Instant::now(),
            scroll: UniformListScrollHandle::new(),
            opening: None,
            scrolled: false,
        });
        let current = state.read(cx);
        let start = current.started;
        let handle = self.scroll.unwrap_or_else(|| current.scroll.clone());
        let scale = self.scale;
        let step = if self.item_layout { 50. } else { 64. };
        let row_height = px(step * scale);
        let slots = current.opening.as_ref().map_or(1, |(_, rows)| rows.len());
        let reduced = crate::ReducedMotion::enabled(cx);
        if !reduced
            && !current.scrolled
            && start.elapsed().as_secs_f32() * 60. < 30. + 2. * slots.min(self.count) as f32
        {
            window.request_animation_frame();
        }
        let h = (self.count.min(8) * 64).saturating_sub(16) as f32;
        let mut rail = Surface::new(24., h.max(1.));
        if self.count > 0 {
            rail.fill(Rect::new(2., 0., 12., h), PANEL, 255);
            rail.fill(Rect::new(19., 0., 2., h), PANEL, 255);
        }

        let row = self.row;
        let row_scroll = handle.clone();
        let list = uniform_list(self.id, self.count, move |range, w, cx| {
            let opening = if reduced {
                None
            } else {
                state.update(cx, |list, _| list.opening_rows(&row_scroll, row_height))
            };
            let now = Instant::now();
            range
                .map(|i| {
                    let root = div().relative().w_full().h(row_height);
                    let begins = opening
                        .as_ref()
                        .filter(|rows| rows.contains(&i))
                        .map(|rows| {
                            start
                                + Duration::from_secs_f64(
                                    (10. + 2. * (i - rows.start) as f64) / 60.,
                                )
                        });
                    if begins.is_some_and(|begins| begins > now) {
                        return root;
                    }
                    let age =
                        begins.map_or(20., |begins| now.duration_since(begins).as_secs_f32() * 60.);
                    root.child(
                        div()
                            .absolute()
                            .top_0()
                            .left(px(row_slide(age) * scale))
                            .opacity(row_alpha(age))
                            .child(ListRowText::new(row(i, w, cx), begins)),
                    )
                })
                .collect()
        })
        .size_full()
        .track_scroll(&handle);
        if self.item_layout {
            let mut paper = Surface::new(480., 480.);
            paper.fill(Rect::new(0., 0., 480., 480.), crate::LIGHT, 255);
            paper.fill(Rect::new(1., 0., 12., 480.), PANEL, 255);
            for y in [6., 472.] {
                paper.fill(Rect::new(60., y, 380., 2.), PANEL, 255);
            }
            return div()
                .relative()
                .mt(px(6. * scale))
                .w(px(480. * scale))
                .h(px(480. * scale))
                .child(div().absolute().inset_0().child(paper))
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .top(px(15. * scale))
                        .w(px(440. * scale))
                        .h(px(450. * scale))
                        .child(list),
                )
                .child(
                    div().absolute().right_0().top_0().child(
                        super::MenuScrollbar::for_list("unit-scrollbar", handle, self.count, step)
                            .compact()
                            .height(480.)
                            .scale(scale),
                    ),
                )
                .into_any_element();
        }
        div()
            .size_full()
            .relative()
            .child(
                div()
                    .absolute()
                    .top(px(10. * self.scale))
                    .left_0()
                    .w(px(24. * self.scale))
                    .h(px(h * self.scale))
                    .child(rail),
            )
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .left(px(6. * self.scale))
                    .right(px(40. * scale))
                    .overflow_hidden()
                    .child(list),
            )
            .child(
                div().absolute().right_0().top_0().child(
                    super::MenuScrollbar::for_list("scrollbar", handle, self.count, 64.)
                        .height(512.)
                        .scale(scale),
                ),
            )
            .into_any_element()
    }
}
