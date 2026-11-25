mod art;
mod drift;

use self::art::Layer;
pub(super) use self::art::closing;
use self::drift::CornerDrift;
use crate::{INK, Rect, Surface};
use gpui::{App, ElementId, IntoElement, RenderOnce, Window};
use std::{
    hash::{BuildHasher, RandomState},
    time::Instant,
};

#[derive(IntoElement)]
pub struct MenuBackdrop {
    id: ElementId,
    start: Option<Instant>,
    seed: Option<u32>,
}

impl MenuBackdrop {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            start: None,
            seed: None,
        }
    }
    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }
    pub(super) fn since(mut self, start: Instant) -> Self {
        self.start = Some(start);
        self
    }
}

impl RenderOnce for MenuBackdrop {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id, cx, |_, _| {
            let seed = self
                .seed
                .unwrap_or_else(|| RandomState::new().hash_one(()) as u32);
            (
                self.start.unwrap_or_else(Instant::now),
                CornerDrift::new(seed),
            )
        });
        let mut s = Surface::new(1600., 900.);
        state.update(cx, |(start, drift), _| {
            let seconds = self.start.unwrap_or(*start).elapsed().as_secs_f32();
            let frame = seconds * 60.;
            art::opening(&mut s, frame);
            let corners = drift.origins(seconds, [s.width, s.height]);
            {
                let opening = (frame - 42.).max(0.) as u8;
                let delayed = opening.saturating_sub(10);
                let last = opening.saturating_sub(20);
                for (side, origins) in corners.into_iter().enumerate() {
                    let opposite = side == 1;
                    let stages = [
                        (Layer::OuterRing, opening),
                        (Layer::InnerRing, delayed),
                        (Layer::ShortLine, if opposite { last } else { opening }),
                        (Layer::MiddleLine, delayed),
                        (Layer::LongLine, if opposite { opening } else { last }),
                    ];
                    for ((layer, age), origin) in stages.into_iter().zip(origins) {
                        art::paint(&mut s, layer, age, origin, opposite);
                    }
                }
            }
        });
        window.request_animation_frame();
        s
    }
}

pub(crate) fn borders(s: &mut Surface, header: f32, reveal: f32) {
    let width = 1600. * reveal;
    for (y, x) in [(99. + header, 1600. - width), (855., 0.)] {
        s.clip(Rect::new(x, y - 24., width, 48.), |s| {
            for i in 0..2 {
                s.sprite(
                    if i == 0 { "divider-reverse" } else { "divider" },
                    Rect::new(i as f32 * 800. - 0.25, y - 24., 800.5, 48.),
                    INK,
                );
            }
        });
    }
}
