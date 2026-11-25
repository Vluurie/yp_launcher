use crate::{
    draw::{Prepared, Rect},
    motion::next_random,
};
use gpui::{App, Window};
use std::time::Instant;

#[derive(Clone, Copy)]
struct Block {
    rect: Rect,
    active: bool,
    duration: f32,
    elapsed: f32,
}

pub(crate) struct HackingStatic {
    blocks: [Block; 35],
    rng: u32,
    last: Instant,
}

fn unit(rng: &mut u32) -> f32 {
    1. - (next_random(rng) >> 8) as f32 / 16_777_216.
}

fn rectangle(rng: &mut u32) -> Rect {
    let (x, y) = (unit(rng), unit(rng));
    let (w, h) = (unit(rng), unit(rng) * 0.1);
    for _ in 0..3 {
        next_random(rng);
    }
    Rect::new(x - w * 0.5, y - h * 0.5, w, h)
}

impl HackingStatic {
    pub(crate) fn new(mut seed: u32) -> Self {
        let blocks = std::array::from_fn(|_| Block {
            rect: rectangle(&mut seed),
            active: false,
            duration: 0.,
            elapsed: 0.,
        });
        Self {
            blocks,
            rng: seed,
            last: Instant::now(),
        }
    }

    pub(crate) fn advance(&mut self) -> Vec<Rect> {
        let now = Instant::now();
        let frames = (now - self.last).as_secs_f32() * 60.;
        self.last = now;
        for b in &mut self.blocks {
            b.elapsed += frames;
            if b.duration > 0. && b.elapsed < b.duration {
                continue;
            }
            b.elapsed = if b.duration > 0. {
                b.elapsed % b.duration
            } else {
                0.
            };
            if b.active {
                b.active = false;
            } else {
                if (100. * unit(&mut self.rng)) as u32 >= 100 {
                    b.duration = 0.;
                    continue;
                }
                b.rect = rectangle(&mut self.rng);
                b.active = true;
            }
            b.duration = (5 + (((next_random(&mut self.rng) >> 16) * 7) >> 16)) as f32;
        }
        self.blocks
            .iter()
            .filter(|b| b.active)
            .map(|b| b.rect)
            .collect()
    }
}

pub(crate) fn paint(
    regions: &[Rect],
    draws: &[Prepared<'_>],
    screen: Rect,
    dst: Rect,
    window: &mut Window,
    _: &mut App,
) {
    let unit = Rect::new(0., 0., 1., 1.);
    for (i, block) in regions.iter().enumerate() {
        if let Some(r) = block.intersect(unit) {
            let to = r.mapped(unit, dst);
            let mut from = r.mapped(unit, screen);
            from.x = (from.x + screen.w * (0.065 + 0.013 * (i % 3) as f32)).min(screen.w - from.w);
            for draw in draws {
                draw.paint(from, to, window);
            }
            let color = if i % 2 == 0 { 0x738d920f } else { 0x9f776811 };
            window.paint_quad(gpui::fill(to.bounds(), gpui::rgba(color)));
        }
    }
}
