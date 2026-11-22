use std::ops::Range;

use gpui::{
    AnyElement, App, Bounds, DispatchPhase, Div, ElementId, Hitbox, HitboxBehavior, IntoElement,
    ListSizingBehavior, ListState, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, RenderOnce, ScrollHandle, Size, Stateful, UniformListScrollHandle, Window,
    canvas, div, fill, list, point, prelude::*, px, size, uniform_list,
};

use crate::theme::{ActiveTheme, Sizes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScrollbarAxis {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone)]
enum ScrollSource {
    Div(ScrollHandle),
    Uniform(UniformListScrollHandle),
    List(ListState),
}

impl ScrollSource {
    fn viewport(&self) -> Bounds<Pixels> {
        match self {
            ScrollSource::Div(handle) => handle.bounds(),
            ScrollSource::Uniform(handle) => handle.0.borrow().base_handle.bounds(),
            ScrollSource::List(state) => state.viewport_bounds(),
        }
    }

    fn offset(&self) -> Point<Pixels> {
        match self {
            ScrollSource::Div(handle) => {
                let offset = handle.offset();
                point(-offset.x, -offset.y)
            }
            ScrollSource::Uniform(handle) => {
                let offset = handle.0.borrow().base_handle.offset();
                point(-offset.x, -offset.y)
            }
            ScrollSource::List(state) => {
                let offset = state.scroll_px_offset_for_scrollbar();
                point(-offset.x, -offset.y)
            }
        }
    }

    fn max_offset(&self) -> Size<Pixels> {
        match self {
            ScrollSource::Div(handle) => {
                let max = handle.max_offset();
                size(max.x, max.y)
            }
            ScrollSource::Uniform(handle) => {
                let state = handle.0.borrow();
                match state.last_item_size {
                    Some(item_size) => size(
                        (item_size.contents.width - item_size.item.width).max(px(0.)),
                        (item_size.contents.height - item_size.item.height).max(px(0.)),
                    ),
                    None => Size::default(),
                }
            }
            ScrollSource::List(state) => {
                let max = state.max_offset_for_scrollbar();
                size(max.x, max.y)
            }
        }
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        let scrolled = point(-offset.x, -offset.y);
        match self {
            ScrollSource::Div(handle) => handle.set_offset(scrolled),
            ScrollSource::Uniform(handle) => handle.0.borrow().base_handle.set_offset(scrolled),
            ScrollSource::List(state) => state.set_offset_from_scrollbar(scrolled),
        }
    }

    fn drag_started(&self) {
        if let ScrollSource::List(state) = self {
            state.scrollbar_drag_started();
        }
    }

    fn drag_ended(&self) {
        if let ScrollSource::List(state) = self {
            state.scrollbar_drag_ended();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThumbMetrics {
    pub length: Pixels,
    pub position: Pixels,
}

pub fn thumb_metrics(
    viewport: Pixels,
    max_offset: Pixels,
    offset: Pixels,
    track: Pixels,
) -> Option<ThumbMetrics> {
    if max_offset < Sizes::SCROLL_OVERFLOW_MIN || viewport <= px(0.) || track <= px(0.) {
        return None;
    }
    let content = viewport + max_offset;
    let length = (track * (viewport / content))
        .max(Sizes::SCROLLBAR_MIN_THUMB)
        .min(track);
    let travel = track - length;
    let progress = (offset / max_offset).clamp(0., 1.);
    Some(ThumbMetrics {
        length,
        position: travel * progress,
    })
}

pub fn offset_for_thumb(
    position: Pixels,
    max_offset: Pixels,
    track: Pixels,
    length: Pixels,
) -> Pixels {
    let travel = track - length;
    if travel <= px(0.) {
        return px(0.);
    }
    max_offset * (position / travel).clamp(0., 1.)
}

#[derive(Default)]
struct ScrollbarState {
    grab: Option<Pixels>,
    hovered: bool,
}

struct ScrollbarLayout {
    track: Bounds<Pixels>,
    thumb: Option<Bounds<Pixels>>,
    metrics: Option<ThumbMetrics>,
    max_offset: Pixels,
    hitbox: Hitbox,
}

#[derive(Clone)]
pub struct Scrollbar {
    id: ElementId,
    source: ScrollSource,
    axis: ScrollbarAxis,
    always_visible: bool,
}

crate::impl_into_element!(Scrollbar);

impl Scrollbar {
    fn new(source: ScrollSource, axis: ScrollbarAxis) -> Self {
        Self {
            id: ElementId::Name("yp-scrollbar".into()),
            source,
            axis,
            always_visible: false,
        }
    }

    pub fn vertical(handle: &ScrollHandle) -> Self {
        Self::new(ScrollSource::Div(handle.clone()), ScrollbarAxis::Vertical)
    }

    pub fn horizontal(handle: &ScrollHandle) -> Self {
        Self::new(ScrollSource::Div(handle.clone()), ScrollbarAxis::Horizontal).id("yp-scrollbar-x")
    }

    pub fn uniform(handle: &UniformListScrollHandle) -> Self {
        Self::new(
            ScrollSource::Uniform(handle.clone()),
            ScrollbarAxis::Vertical,
        )
    }

    pub fn list(state: &ListState) -> Self {
        Self::new(ScrollSource::List(state.clone()), ScrollbarAxis::Vertical)
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn always_visible(mut self, always_visible: bool) -> Self {
        self.always_visible = always_visible;
        self
    }
}

fn along(axis: ScrollbarAxis, point: Point<Pixels>) -> Pixels {
    match axis {
        ScrollbarAxis::Vertical => point.y,
        ScrollbarAxis::Horizontal => point.x,
    }
}

fn length(axis: ScrollbarAxis, size: Size<Pixels>) -> Pixels {
    match axis {
        ScrollbarAxis::Vertical => size.height,
        ScrollbarAxis::Horizontal => size.width,
    }
}

impl RenderOnce for Scrollbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| ScrollbarState::default());
        let axis = self.axis;
        let source = self.source;
        let always_visible = self.always_visible;
        let prepaint_source = source.clone();
        let prepaint_active = {
            let state = state.read(cx);
            state.hovered || state.grab.is_some() || always_visible
        };
        canvas(
            move |_, window, _| {
                let viewport = prepaint_source.viewport();
                if viewport.size.width <= px(0.) || viewport.size.height <= px(0.) {
                    return None;
                }
                let width = Sizes::SCROLLBAR_WIDTH;
                let track = match axis {
                    ScrollbarAxis::Vertical => Bounds::new(
                        point(viewport.right() - width, viewport.top()),
                        size(width, viewport.size.height),
                    ),
                    ScrollbarAxis::Horizontal => Bounds::new(
                        point(viewport.left(), viewport.bottom() - width),
                        size(viewport.size.width, width),
                    ),
                };
                let max_offset = length(axis, prepaint_source.max_offset());
                let metrics = thumb_metrics(
                    length(axis, viewport.size),
                    max_offset,
                    along(axis, prepaint_source.offset()),
                    length(axis, track.size),
                );
                let thickness = if prepaint_active {
                    Sizes::SCROLLBAR_THUMB_ACTIVE
                } else {
                    Sizes::SCROLLBAR_THUMB_IDLE
                };
                let inset = px(2.);
                let thumb = metrics.map(|metrics| match axis {
                    ScrollbarAxis::Vertical => Bounds::new(
                        point(
                            track.right() - inset - thickness,
                            track.top() + metrics.position,
                        ),
                        size(thickness, metrics.length),
                    ),
                    ScrollbarAxis::Horizontal => Bounds::new(
                        point(
                            track.left() + metrics.position,
                            track.bottom() - inset - thickness,
                        ),
                        size(metrics.length, thickness),
                    ),
                });
                if thumb.is_none() && !always_visible {
                    return None;
                }
                let hitbox = window.insert_hitbox(track, HitboxBehavior::Normal);
                Some(ScrollbarLayout {
                    track,
                    thumb,
                    metrics,
                    max_offset,
                    hitbox,
                })
            },
            move |_, layout: Option<ScrollbarLayout>, window, cx| {
                let Some(layout) = layout else {
                    return;
                };
                let (dragging, hovered) = {
                    let state = state.read(cx);
                    (state.grab.is_some(), state.hovered)
                };
                if always_visible || dragging {
                    window.paint_quad(fill(
                        layout.track,
                        theme.colors.border_light.opacity(theme.alpha.line),
                    ));
                }
                if let Some(thumb) = layout.thumb {
                    let color = if dragging {
                        theme.colors.accent_primary.opacity(theme.alpha.glass)
                    } else if hovered {
                        theme.colors.text_muted.opacity(theme.alpha.glass)
                    } else {
                        theme.colors.text_muted.opacity(theme.alpha.accent_border)
                    };
                    window.paint_quad(fill(thumb, color).corner_radii(Sizes::SCROLLBAR_THUMB_IDLE));
                }
                let Some(metrics) = layout.metrics else {
                    return;
                };
                let track = layout.track;
                let max_offset = layout.max_offset;
                let track_length = length(axis, track.size);
                let track_start = along(axis, track.origin);

                let down_state = state.clone();
                let down_source = source.clone();
                let down_hitbox = layout.hitbox.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble
                        || event.button != MouseButton::Left
                        || !down_hitbox.is_hovered(window)
                    {
                        return;
                    }
                    cx.stop_propagation();
                    let pointer = along(axis, event.position) - track_start;
                    let on_thumb =
                        pointer >= metrics.position && pointer <= metrics.position + metrics.length;
                    let grab = if on_thumb {
                        pointer - metrics.position
                    } else {
                        metrics.length / 2.
                    };
                    if !on_thumb {
                        let target = offset_for_thumb(
                            pointer - grab,
                            max_offset,
                            track_length,
                            metrics.length,
                        );
                        let mut offset = down_source.offset();
                        match axis {
                            ScrollbarAxis::Vertical => offset.y = target,
                            ScrollbarAxis::Horizontal => offset.x = target,
                        }
                        down_source.set_offset(offset);
                    }
                    down_source.drag_started();
                    down_state.update(cx, |state, cx| {
                        state.grab = Some(grab);
                        cx.notify();
                    });
                });

                let move_state = state.clone();
                let move_source = source.clone();
                let move_hitbox = layout.hitbox.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture {
                        return;
                    }
                    let (grab, was_hovered) = {
                        let state = move_state.read(cx);
                        (state.grab, state.hovered)
                    };
                    if let Some(grab) = grab {
                        let pointer = along(axis, event.position) - track_start;
                        let target = offset_for_thumb(
                            pointer - grab,
                            max_offset,
                            track_length,
                            metrics.length,
                        );
                        let mut offset = move_source.offset();
                        match axis {
                            ScrollbarAxis::Vertical => offset.y = target,
                            ScrollbarAxis::Horizontal => offset.x = target,
                        }
                        move_source.set_offset(offset);
                        move_state.update(cx, |_, cx| cx.notify());
                        return;
                    }
                    let hovered = move_hitbox.is_hovered(window);
                    if hovered != was_hovered {
                        move_state.update(cx, |state, cx| {
                            state.hovered = hovered;
                            cx.notify();
                        });
                    }
                });

                let up_state = state.clone();
                let up_source = source.clone();
                window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                    if phase != DispatchPhase::Capture || up_state.read(cx).grab.is_none() {
                        return;
                    }
                    up_source.drag_ended();
                    up_state.update(cx, |state, cx| {
                        state.grab = None;
                        cx.notify();
                    });
                });
            },
        )
        .absolute()
        .size_0()
    }
}

pub trait ScrollableElement: Sized {
    fn overflow_y_scrollbar(self, handle: &ScrollHandle) -> Self;
    fn overflow_x_scrollbar(self, handle: &ScrollHandle) -> Self;
    fn overflow_scrollbar(self, handle: &ScrollHandle) -> Self;
}

impl ScrollableElement for Stateful<Div> {
    fn overflow_y_scrollbar(self, handle: &ScrollHandle) -> Self {
        self.overflow_y_scroll()
            .track_scroll(handle)
            .child(Scrollbar::vertical(handle))
    }

    fn overflow_x_scrollbar(self, handle: &ScrollHandle) -> Self {
        self.overflow_x_scroll()
            .track_scroll(handle)
            .child(Scrollbar::horizontal(handle))
    }

    fn overflow_scrollbar(self, handle: &ScrollHandle) -> Self {
        self.overflow_scroll()
            .track_scroll(handle)
            .child(Scrollbar::vertical(handle))
            .child(Scrollbar::horizontal(handle))
    }
}

pub struct VirtualList;

impl VirtualList {
    pub fn uniform<R: IntoElement>(
        id: impl Into<ElementId>,
        item_count: usize,
        handle: &UniformListScrollHandle,
        render: impl Fn(Range<usize>, &mut Window, &mut App) -> Vec<R> + 'static,
    ) -> Div {
        let id = id.into();
        div()
            .relative()
            .size_full()
            .min_h_0()
            .child(
                uniform_list(id.clone(), item_count, render)
                    .track_scroll(handle)
                    .size_full(),
            )
            .child(Scrollbar::uniform(handle).id((id, "scrollbar")))
    }

    pub fn list(
        id: impl Into<ElementId>,
        state: &ListState,
        render: impl FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Div {
        let id = id.into();
        div()
            .relative()
            .size_full()
            .min_h_0()
            .child(
                list(state.clone(), render)
                    .with_sizing_behavior(ListSizingBehavior::Auto)
                    .size_full(),
            )
            .child(Scrollbar::list(state).id((id, "scrollbar")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumb_is_hidden_when_nothing_overflows() {
        assert_eq!(thumb_metrics(px(300.), px(0.), px(0.), px(300.)), None);
        assert_eq!(thumb_metrics(px(300.), px(0.4), px(0.), px(300.)), None);
        assert_eq!(thumb_metrics(px(300.), px(1.), px(0.), px(300.)), None);
        assert!(thumb_metrics(px(300.), px(2.), px(0.), px(300.)).is_some());
        assert_eq!(thumb_metrics(px(0.), px(50.), px(0.), px(300.)), None);
    }

    #[test]
    fn thumb_length_tracks_the_visible_share_and_has_a_minimum() {
        let half = thumb_metrics(px(300.), px(300.), px(0.), px(300.)).unwrap();
        assert_eq!(half.length, px(150.));
        assert_eq!(half.position, px(0.));
        let tiny = thumb_metrics(px(300.), px(30000.), px(30000.), px(300.)).unwrap();
        assert_eq!(tiny.length, Sizes::SCROLLBAR_MIN_THUMB);
        assert_eq!(tiny.position, px(300.) - Sizes::SCROLLBAR_MIN_THUMB);
    }

    #[test]
    fn thumb_position_round_trips_to_the_scroll_offset() {
        let metrics = thumb_metrics(px(200.), px(600.), px(150.), px(200.)).unwrap();
        let offset = offset_for_thumb(metrics.position, px(600.), px(200.), metrics.length);
        assert!((f32::from(offset) - 150.).abs() < 0.01);
        assert_eq!(
            offset_for_thumb(px(-40.), px(600.), px(200.), px(50.)),
            px(0.)
        );
        assert_eq!(
            offset_for_thumb(px(900.), px(600.), px(200.), px(50.)),
            px(600.)
        );
        assert_eq!(
            offset_for_thumb(px(10.), px(600.), px(50.), px(50.)),
            px(0.)
        );
    }
}
