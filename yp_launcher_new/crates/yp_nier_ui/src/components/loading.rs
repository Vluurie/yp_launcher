use crate::{INK, Rect, Surface, Tint};
use gpui::{App, ElementId, IntoElement, RenderOnce, Window, div, prelude::*, px};
use std::time::Instant;

#[derive(IntoElement)]
pub struct LoadingIcon {
    id: ElementId,
    active: bool,
    tint: Tint,
    saving: bool,
    scale: f32,
}
impl LoadingIcon {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            active: true,
            tint: INK,
            saving: false,
            scale: 1.,
        }
    }
    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    pub fn saving(mut self) -> Self {
        self.saving = true;
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.);
        self
    }
}
impl RenderOnce for LoadingIcon {
    fn render(self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = w.use_keyed_state(self.id.clone(), cx, |_, _| (self.active, Instant::now()));
        let elapsed = state.update(cx, |s, _| {
            if s.0 != self.active {
                *s = (self.active, Instant::now());
            }
            s.1.elapsed().as_secs_f32()
        });
        let dim = if self.saving { 52. } else { 48. };
        let reduced = crate::ReducedMotion::enabled(cx);
        let alpha = if reduced {
            if self.active { 1. } else { 0. }
        } else if self.active {
            crate::motion::ease_out(elapsed * 6.)
        } else {
            (1. - elapsed * 6.).max(0.)
        };
        let mut art = Surface::new(dim, dim);
        if alpha > 0. || self.active {
            if !reduced {
                w.request_animation_frame();
            }
            let elapsed = if reduced { 0. } else { elapsed };
            let phase = (elapsed * 10.) as usize;
            let center = dim * 0.5;
            for i in 0..12 {
                let angle = std::f32::consts::TAU * i as f32 / 12.;
                let age = (12 + phase % 12 - i) % 12;
                let a = alpha * (1. - age as f32 / 14.);
                let radius = if self.saving { 16. } else { 18. };
                let x = center + radius * angle.sin();
                let y = center - radius * angle.cos();
                let side = if age == 0 { 5.5 } else { 3.5 };
                art.fill(
                    Rect::new(x - side / 2., y - side / 2., side, side),
                    self.tint,
                    (255. * a) as u8,
                );
            }
            if self.saving {
                let pulse = (elapsed * 3.).sin().abs();
                art.fill(
                    Rect::new(center - 5., center - 5., 10., 10.),
                    self.tint,
                    ((70. + 160. * pulse) * alpha) as u8,
                );
            }
        }
        div()
            .id(self.id)
            .w(px(dim * self.scale))
            .h(px(dim * self.scale))
            .child(art)
    }
}
