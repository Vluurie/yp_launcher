use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, CursorStyle, ElementId, ImageSource, IntoElement, MouseButton, ObjectFit,
    Pixels, Point, RenderImage, RenderOnce, SharedString, Size, Window, div, img, point,
    prelude::*,
};
use image::{Frame, RgbaImage};

use super::label::{Label, LabelSize};
use super::progress::Spinner;
use super::util::{bounds_tracker, pointer_capture};
use crate::i18n::ActiveStrings;
use crate::styled_ext::v_flex;
use crate::theme::{ActiveTheme, Color, ResponsiveExt};
use crate::traits::ValueHandler;

pub fn rgba_to_bgra(pixels: &mut [u8]) {
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
}

pub fn render_image_from_rgba(
    width: u32,
    height: u32,
    mut rgba: Vec<u8>,
) -> Option<Arc<RenderImage>> {
    let expected = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(4)?;
    if width == 0 || height == 0 || rgba.len() != expected {
        return None;
    }
    rgba_to_bgra(&mut rgba);
    let buffer = RgbaImage::from_raw(width, height, rgba)?;
    Some(Arc::new(RenderImage::new(vec![Frame::new(buffer)])))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragMode {
    Orbit,
    Pan,
}

#[derive(Default)]
struct ViewportState {
    drag: Option<(DragMode, Point<Pixels>)>,
    shown: Option<Arc<RenderImage>>,
    size: Option<Size<Pixels>>,
}

pub struct FrameViewport {
    id: ElementId,
    frame: Option<Arc<RenderImage>>,
    loading: bool,
    loading_label: Option<SharedString>,
    fit: ObjectFit,
    on_orbit: Option<ValueHandler<Point<Pixels>>>,
    on_pan: Option<ValueHandler<Point<Pixels>>>,
    on_zoom: Option<ValueHandler<f32>>,
    on_resize: Option<ValueHandler<Size<Pixels>>>,
    overlays: Vec<AnyElement>,
}

crate::impl_into_element!(FrameViewport);

impl FrameViewport {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            frame: None,
            loading: false,
            loading_label: None,
            fit: ObjectFit::Contain,
            on_orbit: None,
            on_pan: None,
            on_zoom: None,
            on_resize: None,
            overlays: Vec::new(),
        }
    }

    pub fn frame(mut self, frame: Option<Arc<RenderImage>>) -> Self {
        self.frame = frame;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn loading_label(mut self, label: impl Into<SharedString>) -> Self {
        self.loading_label = Some(label.into());
        self
    }

    pub fn fit(mut self, fit: ObjectFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn on_orbit(
        mut self,
        handler: impl Fn(&Point<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_orbit = Some(Rc::new(handler));
        self
    }

    pub fn on_pan(
        mut self,
        handler: impl Fn(&Point<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_pan = Some(Rc::new(handler));
        self
    }

    pub fn on_zoom(mut self, handler: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_zoom = Some(Rc::new(handler));
        self
    }

    pub fn on_resize(
        mut self,
        handler: impl Fn(&Size<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resize = Some(Rc::new(handler));
        self
    }

    pub fn overlay(mut self, overlay: impl IntoElement) -> Self {
        self.overlays.push(overlay.into_any_element());
        self
    }
}

impl RenderOnce for FrameViewport {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| {
            cx.on_release(|state: &mut ViewportState, cx| {
                if let Some(image) = state.shown.take() {
                    cx.drop_image(image, None);
                }
            })
            .detach();
            ViewportState::default()
        });
        let (drag, previous, measured) = {
            let state = state.read(cx);
            (state.drag, state.shown.clone(), state.size)
        };
        let replaced = match (&previous, &self.frame) {
            (Some(previous), Some(frame)) => previous.id != frame.id,
            (Some(_), None) => true,
            _ => false,
        };
        if replaced && let Some(previous) = previous.clone() {
            window.defer(cx, move |window, _| {
                let _ = window.drop_image(previous);
            });
        }
        let frame_changed = match (&previous, &self.frame) {
            (Some(previous), Some(frame)) => previous.id != frame.id,
            (None, None) => false,
            _ => true,
        };
        if frame_changed {
            let frame = self.frame.clone();
            state.update(cx, |state, _| state.shown = frame);
        }

        let interactive = self.on_orbit.is_some() || self.on_pan.is_some();
        let on_orbit = self.on_orbit;
        let on_pan = self.on_pan;
        let on_zoom = self.on_zoom;
        let can_orbit = on_orbit.is_some();
        let can_pan = on_pan.is_some();
        let down_state = state.clone();
        let pan_state = state.clone();
        let middle_state = state.clone();
        let move_state = state.clone();
        let up_state = state.clone();
        let size_state = state.clone();
        let on_resize = self.on_resize;
        let line_height = window.line_height();
        let loading_label = self
            .loading_label
            .unwrap_or_else(|| SharedString::new_static(cx.t().ui_viewport_loading()));
        let start_drag = |state: gpui::Entity<ViewportState>, mode: DragMode| {
            move |event: &gpui::MouseDownEvent, _: &mut Window, cx: &mut App| {
                cx.stop_propagation();
                state.update(cx, |state, cx| {
                    state.drag = Some((mode, event.position));
                    cx.notify();
                });
            }
        };

        div()
            .id(self.id)
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(theme.colors.scrim)
            .when(interactive, |this| {
                this.cursor(if drag.is_some() {
                    CursorStyle::ClosedHand
                } else {
                    CursorStyle::OpenHand
                })
            })
            .when(can_orbit, |this| {
                this.on_mouse_down(MouseButton::Left, start_drag(down_state, DragMode::Orbit))
            })
            .when(can_pan, |this| {
                this.on_mouse_down(MouseButton::Right, start_drag(pan_state, DragMode::Pan))
                    .on_mouse_down(MouseButton::Middle, start_drag(middle_state, DragMode::Pan))
            })
            .when_some(on_zoom, |this, handler| {
                this.on_scroll(move |event, window, cx| {
                    cx.stop_propagation();
                    let delta = event.delta.pixel_delta(line_height);
                    handler(&f32::from(-delta.y), window, cx);
                })
            })
            .child(pointer_capture(
                drag.is_some(),
                move |event, window, cx| {
                    let Some((mode, last)) = move_state.read(cx).drag else {
                        return;
                    };
                    let delta = point(event.position.x - last.x, event.position.y - last.y);
                    move_state.update(cx, |state, _| state.drag = Some((mode, event.position)));
                    let handler = match mode {
                        DragMode::Orbit => on_orbit.as_ref(),
                        DragMode::Pan => on_pan.as_ref(),
                    };
                    if let Some(handler) = handler {
                        handler(&delta, window, cx);
                    }
                },
                move |_, _, cx| {
                    up_state.update(cx, |state, cx| {
                        state.drag = None;
                        cx.notify();
                    });
                },
            ))
            .child(bounds_tracker(move |bounds, window, cx| {
                let size = bounds.size;
                if size_state.read(cx).size == Some(size) {
                    return;
                }
                window.defer(cx, move |window, cx| {
                    size_state.update(cx, |state, cx| {
                        state.size = Some(size);
                        cx.notify();
                    });
                    if let Some(handler) = &on_resize {
                        handler(&size, window, cx);
                    }
                });
            }))
            .when_some(self.frame.zip(measured), |this, (frame, size)| {
                this.child(
                    div().absolute().inset_0().flex().child(
                        img(ImageSource::Render(frame))
                            .flex_none()
                            .w(size.width)
                            .h(size.height)
                            .object_fit(self.fit),
                    ),
                )
            })
            .when(self.loading, |this| {
                this.child(
                    v_flex()
                        .absolute()
                        .inset_0()
                        .items_center()
                        .justify_center()
                        .gap(sizes.spacing_md)
                        .child(Spinner::new())
                        .child(
                            Label::new(loading_label)
                                .size(LabelSize::XSmall)
                                .color(Color::OnDark),
                        ),
                )
            })
            .children(self.overlays)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_are_swapped_in_place() {
        let mut pixels = vec![1, 2, 3, 4, 10, 20, 30, 40];
        rgba_to_bgra(&mut pixels);
        assert_eq!(pixels, vec![3, 2, 1, 4, 30, 20, 10, 40]);
    }

    #[test]
    fn frames_with_the_wrong_byte_count_are_rejected() {
        assert!(render_image_from_rgba(2, 2, vec![0; 16]).is_some());
        assert!(render_image_from_rgba(2, 2, vec![0; 15]).is_none());
        assert!(render_image_from_rgba(0, 2, Vec::new()).is_none());
        assert!(render_image_from_rgba(u32::MAX, u32::MAX, Vec::new()).is_none());
    }

    #[test]
    fn converted_frames_keep_their_size_and_bgra_order() {
        let image = render_image_from_rgba(1, 2, vec![9, 8, 7, 6, 1, 2, 3, 4]).unwrap();
        assert_eq!(image.as_bytes(0), Some(&[7, 8, 9, 6, 3, 2, 1, 4][..]));
        let size = image.size(0);
        assert_eq!((i32::from(size.width), i32::from(size.height)), (1, 2));
    }
}
