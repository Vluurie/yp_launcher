use super::{Theme, Tint};
use image::{Rgba, RgbaImage};

pub(super) const SYMBOLS: &[(&str, [u32; 2])] = &[
    ("panel", [64, 64]),
    ("square", [32, 32]),
    ("divider", [1602, 96]),
    ("divider-reverse", [1602, 96]),
    ("margin", [50, 90]),
    ("archive", [40, 40]),
    ("inventory", [40, 40]),
    ("profiles", [40, 40]),
    ("settings", [40, 40]),
    ("install", [40, 40]),
    ("back", [58, 40]),
    ("tab", [59, 40]),
    ("up", [40, 40]),
    ("left", [40, 40]),
    ("down", [40, 40]),
    ("right", [40, 40]),
    ("confirm", [62, 40]),
    ("rotate", [32, 40]),
    ("pointer", [38, 22]),
    ("ring", [512, 512]),
    ("tracer", [512, 512]),
    ("ring-reverse", [512, 512]),
    ("triangle-up", [200, 100]),
    ("triangle-down", [200, 100]),
];

fn canonical(name: &str) -> &str {
    match name {
        "core_box_20" => "panel",
        "core_box_00" => "square",
        "core_parts_00" => "divider",
        "core_parts_00_flip" => "divider-reverse",
        "core_parts_02" => "margin",
        "core_item_icon_01" => "archive",
        "core_tab_05" => "inventory",
        "core_tab_06" => "profiles",
        "core_tab_07" => "settings",
        "core_tab_02" => "install",
        "key_001" => "back",
        "key_028" => "tab",
        "key_030" => "up",
        "key_043" => "left",
        "key_044" => "down",
        "key_045" => "right",
        "key_054" => "confirm",
        "key_a01" => "rotate",
        other => other,
    }
}
pub(super) fn size(name: &str) -> Option<[u32; 2]> {
    let name = canonical(name);
    SYMBOLS
        .iter()
        .find_map(|&(n, size)| (n == name).then_some(size))
}

fn segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let q = [p[0] - a[0], p[1] - a[1]];
    let t = ((q[0] * d[0] + q[1] * d[1]) / (d[0] * d[0] + d[1] * d[1]).max(0.0001)).clamp(0., 1.);
    (q[0] - t * d[0]).hypot(q[1] - t * d[1])
}
pub(super) fn draw(name: &str, tint: Tint) -> Option<RgbaImage> {
    let [w, h] = size(name)?;
    let name = canonical(name);
    let key = match name {
        "back" => Some("Esc"),
        "tab" => Some("Tab"),
        "up" => Some("W"),
        "left" => Some("A"),
        "down" => Some("S"),
        "right" => Some("D"),
        "confirm" => Some("Enter"),
        _ => None,
    };
    let mut out = RgbaImage::from_fn(w, h, |x, y| {
        let (x, y) = (x as f32 + 0.5, y as f32 + 0.5);
        let (u, v) = (x / w as f32, y / h as f32);
        let line = |a, b, width: f32| (width + 0.5 - segment([x, y], a, b)).clamp(0., 1.);
        let edge = x.min(y).min(w as f32 - x).min(h as f32 - y);
        let coverage = match name {
            "panel" | "square" => 1.,
            "divider" | "divider-reverse" => {
                let dx = if name == "divider" { x } else { w as f32 - x };
                let base = line([0., 48.], [w as f32, 48.], 1.4);
                let tick = (dx - 20.).rem_euclid(132.);
                let diamond = |a: f32, b: f32, r: f32| {
                    (r + 0.5 - ((tick - a).abs() + (y - b).abs())).clamp(0., 1.)
                };
                let stem = if tick < 9. && (48. ..54.).contains(&y) {
                    1.
                } else {
                    0.
                };
                base.max(stem)
                    .max(diamond(63., 56., 4.4))
                    .max(diamond(81., 56., 4.4))
                    .max(diamond(72., 68., 4.))
            }
            "margin" => {
                if (u > 0.34 && u < 0.49) || (u > 0.59 && u < 0.63) {
                    1.
                } else {
                    0.
                }
            }
            "ring" => (1.2 - (x.hypot(y) - 497.).abs()).clamp(0., 1.),
            "ring-reverse" => {
                (1.2 - ((w as f32 - x).hypot(h as f32 - y) - 497.).abs()).clamp(0., 1.)
            }
            "tracer" => line([0., 0.], [510., 510.], 0.85),
            "triangle-up" | "triangle-down" => {
                let span = if name == "triangle-up" { v } else { 1. - v };
                ((span - (2. * u - 1.).abs()) * h as f32 + 0.5).clamp(0., 1.)
            }
            "pointer" => {
                let body = if u < 0.78 {
                    (0.42 - (u - 0.34).abs() * 0.83 - (v - 0.5).abs()).clamp(0., 1.) * 18.
                } else {
                    0.
                };
                let eye = (((u - 0.34) * 1.7).hypot(v - 0.5) - 0.12) * 16.;
                let dots =
                    if u > 0.82 && u < 0.95 && ((v - 0.13).abs() < 0.08 || (v - 0.87).abs() < 0.08)
                    {
                        1.
                    } else {
                        0.
                    };
                body.clamp(0., 1.).min(eye.clamp(0., 1.)).max(dots)
            }
            "settings" => (2.4 - ((x - 20.).hypot(y - 20.) - 11.).abs())
                .clamp(0., 1.)
                .max(line([16., 20.], [24., 20.], 1.6)),
            "profiles" => line([7., 12.], [29., 12.], 1.9)
                .max(line([7., 20.], [33., 20.], 1.9))
                .max(line([7., 28.], [25., 28.], 1.9)),
            "install" => line([20., 6.], [20., 26.], 1.7)
                .max(line([12., 18.], [20., 26.], 1.7))
                .max(line([20., 26.], [28., 18.], 1.7))
                .max(line([8., 32.], [32., 32.], 1.7)),
            "inventory" | "archive" => {
                let d = (x - 20.).abs() + (y - 20.).abs();
                (2.5 - (d - 14.).abs())
                    .clamp(0., 1.)
                    .max(line([13., 20.], [27., 20.], 1.2))
            }
            "rotate" => {
                let d = ((u - 0.5).abs() - 0.16)
                    .max(0.)
                    .hypot(((v - 0.5).abs() - 0.24).max(0.));
                (1. - (d - 0.17).abs() * 30.)
                    .clamp(0., 1.)
                    .max(line([16., 6.], [16., 19.], 0.8))
            }
            _ => (edge - 2.8).clamp(0., 1.),
        };
        let coverage = if matches!(name, "settings" | "install" | "profiles" | "inventory") {
            (edge - 2.).clamp(0., 1.) * (1. - coverage)
        } else {
            coverage
        };
        Rgba([tint[0], tint[1], tint[2], (255. * coverage).round() as u8])
    });
    if let Some(key) = key
        && let Some(label) = Theme::bundled().text.draw(key, 6, tint, 0.)
    {
        let ink_width = Theme::bundled().measure_spaced(key, 6, 0.)[0];
        let scale = ((w as f32 - 12.) / ink_width.max(1.)).min(0.88);
        let label = image::imageops::resize(
            &label,
            (label.width() as f32 * scale).ceil() as u32,
            (label.height() as f32 * scale).ceil() as u32,
            image::imageops::FilterType::Triangle,
        );
        let left = ((w as f32 - ink_width * scale) * 0.5) as u32;
        let top = h.saturating_sub(label.height()) / 2;
        for (x, y, letter) in label.enumerate_pixels() {
            if let Some(pixel) = out.get_pixel_mut_checked(left + x, top + y) {
                pixel[3] = (u16::from(pixel[3]) * (255 - u16::from(letter[3])) / 255) as u8;
            }
        }
    }
    Some(out)
}
