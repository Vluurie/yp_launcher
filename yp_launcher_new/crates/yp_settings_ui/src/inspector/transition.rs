use super::{InspectorModel, InspectorPage};
use gpui::{Bounds, DevicePixels, RenderImage, Size, canvas, fill, point, size};
use std::sync::Arc;
use std::time::{Duration, Instant};
use yp_ui::prelude::*;

const DURATION: Duration = Duration::from_millis(335);

pub(super) struct ViewportTransition {
    slide: Slide,
    image: Arc<RenderImage>,
}

pub(super) struct Slide {
    started: Instant,
    from: f32,
    to: f32,
    duration: Duration,
}

impl Slide {
    fn new(started: Instant, from: f32, open: bool, duration: Duration) -> Self {
        Self {
            started,
            from,
            to: f32::from(u8::from(open)),
            duration,
        }
    }

    fn progress_at(&self, now: Instant) -> (f32, bool) {
        let distance = (self.to - self.from).abs();
        if distance <= f32::EPSILON {
            return (self.to, true);
        }
        let elapsed = now.duration_since(self.started).as_secs_f32()
            / (self.duration.as_secs_f32() * distance);
        let t = elapsed.clamp(0., 1.);
        let eased = t * t * (3. - 2. * t);
        (self.from + (self.to - self.from) * eased, elapsed >= 1.)
    }
}

fn interpolate(a: Bounds<Pixels>, b: Bounds<Pixels>, t: f32) -> Bounds<Pixels> {
    let mix = |a: Pixels, b: Pixels| a + (b - a) * t;
    Bounds::new(
        point(mix(a.origin.x, b.origin.x), mix(a.origin.y, b.origin.y)),
        size(
            mix(a.size.width, b.size.width),
            mix(a.size.height, b.size.height),
        ),
    )
}

fn model_image_bounds(rect: Bounds<Pixels>, source: Size<DevicePixels>) -> Bounds<Pixels> {
    let short_side = source.width.0.min(source.height.0).max(1) as f32;
    let scale = rect.size.width.min(rect.size.height) / short_side;
    Bounds::centered_at(
        rect.center(),
        size(
            scale * source.width.0 as f32,
            scale * source.height.0 as f32,
        ),
    )
}

fn frame(model: &InspectorModel, cx: &App) -> Option<Arc<RenderImage>> {
    model.session()?.read(cx).latest_frame()
}

impl InspectorPage {
    pub(super) fn toggle_controls(&mut self, cx: &mut Context<Self>) {
        let now = cx.background_executor().now();
        let from = self
            .controls_slide
            .as_ref()
            .map_or(f32::from(u8::from(self.controls_open)), |s| {
                s.progress_at(now).0
            });
        self.controls_open = !self.controls_open;
        self.controls_slide = (!yp_nier_ui::ReducedMotion::enabled(cx))
            .then(|| Slide::new(now, from, self.controls_open, Duration::from_millis(240)));
        cx.notify();
    }

    pub(super) fn controls_progress(&mut self, window: &mut Window, cx: &App) -> (f32, bool) {
        if !yp_nier_ui::ReducedMotion::enabled(cx)
            && let Some(slide) = &self.controls_slide
            && let (progress, false) = slide.progress_at(cx.background_executor().now())
        {
            window.request_animation_frame();
            return (progress, true);
        }
        self.controls_slide = None;
        (f32::from(u8::from(self.controls_open)), false)
    }

    pub(super) fn start_transition(&mut self, opening: bool, cx: &mut Context<Self>) -> bool {
        let Some(preview) = self.preview.as_ref().and_then(|p| p.upgrade()) else {
            return false;
        };
        if yp_nier_ui::ReducedMotion::enabled(cx)
            || preview
                .read(cx)
                .scene_bounds
                .get()
                .is_none_or(|b| b.is_empty())
        {
            return false;
        }
        let preview_image = frame(preview.read(cx), cx);
        let image = if opening {
            preview_image
        } else {
            frame(self.model.read(cx), cx).or(preview_image)
        };
        let Some(image) = image else { return false };
        let now = cx.background_executor().now();
        let from = self
            .transition
            .as_ref()
            .map(|t| t.slide.progress_at(now).0)
            .unwrap_or(if opening { 0. } else { 1. });
        self.transition = Some(ViewportTransition {
            slide: Slide::new(now, from, opening, DURATION),
            image,
        });
        if !opening {
            self.sync_preview_outfit(cx);
            self.returning = true;
            if let Some(session) = preview.read(cx).session().cloned() {
                let camera = self.model.read(cx).session().map(|s| *s.read(cx).camera());
                session.update(cx, |s, cx| {
                    if let Some(camera) = camera {
                        s.set_camera(camera, cx);
                    } else {
                        s.request_render(cx);
                    }
                });
            }
        }
        cx.notify();
        true
    }

    pub(super) fn transition_view(
        &mut self,
        content: AnyElement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(transition) = &self.transition else {
            return content;
        };
        let (progress, done) = transition.slide.progress_at(cx.background_executor().now());
        let preview = self.preview.as_ref().and_then(|p| p.upgrade());
        let small = preview.as_ref().and_then(|p| p.read(cx).scene_bounds.get());
        if done || small.is_none() {
            self.transition = None;
            if self.returning {
                let owner = cx.entity().downgrade();
                cx.defer(move |cx| {
                    let _ = owner.update(cx, |this, cx| this.close(cx));
                });
                return gpui::Empty.into_any_element();
            }
            return content;
        }
        let small = small.unwrap();
        let image = transition.image.clone();
        let page = self.model.read(cx);
        let large = page.scene_bounds.clone();
        let chrome = ((progress - 0.5) / 0.25).clamp(0., 1.);
        let handoff = ((progress - 0.75) / 0.25).clamp(0., 1.);
        let moving = canvas(
            |_, _, _| (),
            move |_, _, w, _| {
                let full = large.get().filter(|b| !b.is_empty()).unwrap_or(small);
                let rect = interpolate(small, full, progress);
                w.paint_quad(fill(rect, yp_nier_ui::ModelViewer::background()));
                let inner = model_image_bounds(rect, image.size(0));
                let _ = w.paint_image(rect, inner, Default::default(), image, 0, false);
            },
        )
        .absolute()
        .inset_0()
        .size_full();
        window.request_animation_frame();
        div()
            .relative()
            .size_full()
            .occlude()
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .opacity(progress * progress)
                    .bg(cx.theme().colors.background_primary),
            )
            .child(div().absolute().inset_0().opacity(chrome).child(content))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .opacity(1. - handoff)
                    .child(moving),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_lands_on_the_measured_off_centre_preview() {
        let preview = Bounds::new(point(px(713.), px(216.)), size(px(386.), px(218.)));
        let inspector = Bounds::new(point(px(8.), px(86.)), size(px(1120.), px(630.)));
        assert_eq!(interpolate(preview, inspector, 0.), preview);
        assert_eq!(interpolate(preview, inspector, 1.), inspector);
        assert_eq!(interpolate(preview, inspector, 0.5).origin.x, px(360.5));
    }

    #[test]
    fn moving_between_a_wide_preview_and_square_details_keeps_the_model_at_camera_scale() {
        let wide = size(DevicePixels(960), DevicePixels(540));
        let square = Bounds::new(point(px(30.), px(20.)), size(px(300.), px(300.)));
        let image = model_image_bounds(square, wide);
        assert_eq!(image.size.height, square.size.height);
        assert!(image.size.width > square.size.width);
        assert_eq!(image.center(), square.center());
    }
}
