use super::super::control::mix;
use crate::{INK, PAPER, Rect, Surface};

pub(super) enum Layer {
    OuterRing,
    InnerRing,
    ShortLine,
    MiddleLine,
    LongLine,
}

// Hisayoshi Kijima shows the system menu opening and describes how players can skip its closing animation.
// https://www.platinumgames.com/official-blog/article/9624
pub(super) fn opening(s: &mut Surface, frame: f32) {
    let screen = Rect::new(0., 0., s.width, s.height);
    if frame >= 42. {
        s.fill(screen, PAPER, 255);
        return;
    }
    s.fill(screen, mix(PAPER, INK, 0.70), 255);
    if frame < 20. {
        return;
    }
    triangles(s, PAPER, |up, step| {
        let start = if up { 22. } else { 20. };
        frame >= start + step
    });
}

pub(in crate::components) fn closing(s: &mut Surface, frame: f32) {
    let shade = mix(PAPER, INK, 0.70);
    if frame >= 21. {
        s.fill(Rect::new(0., 0., s.width, s.height), shade, 255);
    } else {
        triangles(s, shade, |_, step| frame >= 21. - step);
    }
}

fn triangles(s: &mut Surface, tint: crate::Tint, visible: impl Fn(bool, f32) -> bool) {
    let half_width = s.width / 16.;
    let height = s.height / 9.;
    let mut order = 0xd761_a29b;
    for row in 0..9 {
        for column in -1..8 {
            let x = (2 * column + row % 2) as f32 * half_width;
            for (symbol, left, up) in [
                ("triangle-down", x, false),
                ("triangle-up", x + half_width, true),
            ] {
                let step = (crate::motion::next_random(&mut order) >> 16) % 21;
                if visible(up, step as f32) {
                    let rect = Rect::new(left, row as f32 * height, 2. * half_width, height);
                    s.sprite(symbol, rect, tint);
                }
            }
        }
    }
}

pub(super) fn paint(s: &mut Surface, layer: Layer, frame: u8, origin: [f32; 2], opposite: bool) {
    let (symbol, length, duration) = match layer {
        Layer::OuterRing => ("ring", 505., 30.),
        Layer::InnerRing => ("ring", 484., 30.),
        Layer::ShortLine => ("tracer", 708., 20.),
        Layer::MiddleLine => ("tracer", 802., 20.),
        Layer::LongLine => ("tracer", 730., 20.),
    };
    let progress = (f32::from(frame) / duration).clamp(0., 1.);
    if progress <= 0. {
        return;
    }
    let [x, y] = origin;
    let rect = Rect::new(
        if opposite { x - length } else { x },
        if opposite { y - length } else { y },
        length,
        length,
    );
    let tint = mix(PAPER, INK, 0.15);
    s.clip(
        Rect::new(
            rect.x,
            if opposite {
                rect.y + length * (1. - progress)
            } else {
                rect.y
            },
            length,
            length * progress,
        ),
        |s| {
            s.sprite(
                if opposite && symbol == "ring" {
                    "ring-reverse"
                } else {
                    symbol
                },
                rect,
                tint,
            );
        },
    );
}
