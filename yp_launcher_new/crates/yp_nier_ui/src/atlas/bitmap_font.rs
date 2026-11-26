use image::GrayImage;
use std::collections::HashMap;

const FACE: u16 = 0x8000;
const DEFAULT_METRICS: [f32; 3] = [16., 44., -5.];
const METRICS: [(u16, [f32; 3]); 14] = [
    (0, [0., 4., -6.]),
    (1, [16., 44., -5.]),
    (2, [17., 76., -10.]),
    (3, [26., 56., 0.]),
    (4, [12., 44., -5.]),
    (5, [8., 38., -10.]),
    (6, [16., 44., -5.]),
    (7, [8., 38., -10.]),
    (8, [11., 46., 1.]),
    (9, [11., 38., -7.]),
    (11, [12., 43., -7.]),
    (35, [8., 38., -11.]),
    (36, [14., 44., -7.]),
    (37, [14., 44., -7.]),
];

pub(super) fn metrics(font: u16) -> [f32; 3] {
    METRICS
        .iter()
        .find(|(id, _)| *id == font)
        .map_or(DEFAULT_METRICS, |(_, metrics)| *metrics)
}

#[derive(Clone, Copy)]
pub(super) struct Glyph {
    rect: [u16; 4],
    size: [f32; 2],
    offset: [f32; 2],
    pub advance: f32,
}

#[derive(Default)]
pub(super) struct Atlas {
    pixels: GrayImage,
    glyphs: HashMap<(u16, u16), Glyph>,
    kerning: HashMap<(u16, u16, u16), i16>,
}

impl Atlas {
    pub fn new(
        pixels: GrayImage,
        glyphs: impl IntoIterator<Item = ((u16, u16), [u16; 4], f32)>,
        kerning: impl IntoIterator<Item = ([u16; 3], i16)>,
    ) -> Self {
        let (width, height) = pixels.dimensions();
        let inside = |rect: &[u16; 4]| {
            let [x, y, w, h] = rect.map(u32::from);
            x + w <= width && y + h <= height
        };
        let glyphs = glyphs
            .into_iter()
            .filter(|(_, rect, _)| inside(rect))
            .map(|(key, rect, advance)| {
                let glyph = Glyph {
                    rect,
                    size: [f32::from(rect[2]), f32::from(rect[3])],
                    offset: [0.; 2],
                    advance,
                };
                (key, glyph)
            })
            .collect();
        let kerning = kerning
            .into_iter()
            .map(|([font, left, right], value)| ((font, left, right), value))
            .collect();
        Self {
            pixels,
            glyphs,
            kerning,
        }
    }

    fn face(font: u16) -> u16 {
        FACE | match font {
            4 => 4,
            5 | 25 | 35 => 5,
            11 => 11,
            _ => 1,
        }
    }

    pub fn glyph(&self, font: u16, ch: char) -> Option<Glyph> {
        let code = u16::try_from(u32::from(ch)).ok()?;
        if let Some(glyph) = self.glyphs.get(&(font, code)) {
            return Some(*glyph);
        }
        let (face, mut glyph) = [Self::face(font), FACE | 1]
            .into_iter()
            .find_map(|face| self.glyphs.get(&(face, code)).map(|g| (face, *g)))?;
        let [_, height, spacing] = metrics(font);
        let (factor, bearing_x, bearing_y, advance_adjustment) = if font == 2 {
            let cap_scale = 36. / 22.;
            (
                cap_scale,
                6. - 6. * cap_scale,
                25. - 11. * cap_scale,
                30. - 28. * cap_scale,
            )
        } else {
            (height / metrics(face & !FACE)[1], 0., 0., spacing)
        };
        glyph.size = glyph.size.map(|v| v * factor);
        glyph.offset = [
            glyph.offset[0] * factor + bearing_x,
            glyph.offset[1] * factor + bearing_y,
        ];
        glyph.advance = glyph.size[0] + advance_adjustment;
        Some(glyph)
    }

    pub fn kern(&self, font: u16, prev: char, ch: char) -> f32 {
        let pair = (prev as u16, ch as u16);
        self.kerning
            .get(&(font, pair.0, pair.1))
            .or_else(|| self.kerning.get(&(Self::face(font), pair.0, pair.1)))
            .copied()
            .unwrap_or_default()
            .into()
    }

    pub fn paint(&self, glyph: Glyph, x: f32, y: f32, mut pixel: impl FnMut(i32, i32, u8)) {
        let [left, top, w, h] = glyph.rect.map(u32::from);
        let [width, height] = glyph.size.map(|v| v.round().max(1.) as u32);
        let x = (x + glyph.offset[0]).round() as i32;
        let y = (y + glyph.offset[1]).round() as i32;
        for j in 0..height {
            for i in 0..width {
                let alpha = self
                    .pixels
                    .get_pixel(left + i * w / width, top + j * h / height)[0];
                if alpha != 0 {
                    pixel(x + i as i32, y + j as i32, alpha);
                }
            }
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use image::Luma;
    use std::env;
    use yp_formats::font::{build, in_common_ranges};

    pub(in crate::atlas) fn sample() -> Atlas {
        let mut pixels = GrayImage::new(16, 8);
        for (x, y) in [(0, 0), (1, 1), (4, 0), (7, 3), (8, 0), (11, 3)] {
            pixels.put_pixel(x, y, Luma([255]));
        }
        Atlas::new(
            pixels,
            [
                ((36, 'A' as u16), [0, 0, 4, 4], 3.),
                ((36, 'V' as u16), [4, 0, 4, 4], 4.),
                ((FACE | 1, 'b' as u16), [8, 0, 4, 4], 5.),
                ((FACE | 5, 'c' as u16), [8, 0, 4, 4], 5.),
                ((36, 'X' as u16), [14, 6, 4, 4], 9.),
            ],
            [
                ([36, 'A' as u16, 'V' as u16], -2),
                ([FACE | 1, 'A' as u16, 'b' as u16], 1),
            ],
        )
    }

    #[test]
    fn rectangles_outside_the_sheet_are_dropped() {
        let atlas = sample();
        assert!(atlas.glyph(36, 'A').is_some());
        assert!(atlas.glyph(36, 'X').is_none());
        assert!(Atlas::default().glyph(36, 'A').is_none());
    }

    #[test]
    fn missing_message_glyphs_borrow_the_face_scaled_to_the_font_height() {
        let atlas = sample();
        let own = atlas.glyph(36, 'A').unwrap();
        assert_eq!((own.size, own.advance), ([4., 4.], 3.));
        let borrowed = atlas.glyph(9, 'b').unwrap();
        let factor = 38. / 44.;
        assert_eq!(borrowed.size, [4. * factor, 4. * factor]);
        assert_eq!(borrowed.advance, 4. * factor - 7.);
        let sibling = atlas.glyph(35, 'c').unwrap();
        assert_eq!(sibling.size, [4., 4.]);
        assert!(atlas.glyph(35, 'b').is_some());
        assert!(atlas.glyph(4, 'c').is_none());
    }

    #[test]
    fn font_kerning_wins_over_the_face_and_unknown_pairs_are_zero() {
        let atlas = sample();
        assert_eq!(atlas.kern(36, 'A', 'V'), -2.);
        assert_eq!(atlas.kern(36, 'A', 'b'), 1.);
        assert_eq!(atlas.kern(36, 'V', 'A'), 0.);
    }

    #[test]
    fn painting_scales_the_rectangle_to_the_glyph_size() {
        let atlas = sample();
        let mut lit = Vec::new();
        atlas.paint(atlas.glyph(36, 'A').unwrap(), 10., 20., |x, y, a| {
            lit.push((x, y, a))
        });
        assert_eq!(lit, [(10, 20, 255), (11, 21, 255)]);
        let mut doubled = atlas.glyph(36, 'V').unwrap();
        doubled.size = [8., 8.];
        let mut count = 0;
        atlas.paint(doubled, 0., 0., |_, _, _| count += 1);
        assert_eq!(count, 8);
    }

    #[test]
    #[ignore = "needs YP_NIER_DIR pointing at a NieR:Automata install"]
    fn title_capitals_from_the_game_share_a_baseline_with_digits() {
        let root = env::var_os("YP_NIER_DIR").expect("YP_NIER_DIR");
        let font = build(root.as_ref(), in_common_ranges).unwrap();
        let atlas = Atlas::new(
            GrayImage::from_raw(font.width, font.height, font.coverage).unwrap(),
            font.glyphs
                .iter()
                .map(|g| ((g.font, g.code), g.rect, g.advance)),
            font.kerning,
        );
        for ch in 'A'..='Z' {
            assert!(atlas.glyphs.contains_key(&(2, ch as u16)), "{ch}");
        }
        for ch in "TEXTURESZ124".chars() {
            let mut top = i32::MAX;
            let mut bottom = i32::MIN;
            atlas.paint(atlas.glyph(2, ch).unwrap(), 0., 0., |_, y, alpha| {
                if alpha >= 128 {
                    top = top.min(y);
                    bottom = bottom.max(y);
                }
            });
            assert!(
                (23..=26).contains(&top) && (59..=62).contains(&bottom),
                "{ch}: {top}..={bottom}"
            );
        }
    }

    #[test]
    fn unknown_fonts_use_the_body_metrics() {
        assert_eq!(metrics(36), [14., 44., -7.]);
        assert_eq!(metrics(2)[1], 76.);
        assert_eq!(metrics(999), DEFAULT_METRICS);
    }
}
