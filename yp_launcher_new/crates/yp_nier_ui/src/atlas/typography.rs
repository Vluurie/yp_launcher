use super::{
    Tint,
    bitmap_font::{Atlas, Glyph, metrics},
    fallback::{FallbackFonts, ShapedRun},
};
use image::{Rgba, RgbaImage};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError, RwLock};
use unicode_segmentation::UnicodeSegmentation;

enum Ink {
    Bitmap(Glyph),
    Shaped(Arc<ShapedRun>),
}

pub(super) struct Typography {
    atlas: RwLock<Arc<Atlas>>,
    generation: AtomicU64,
    fallback: Mutex<FallbackFonts>,
}

fn glyph(atlas: &Atlas, font: u16, cluster: &str) -> Option<(char, Glyph)> {
    let mut chars = cluster.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    atlas.glyph(font, ch).map(|g| (ch, g))
}

impl Typography {
    pub fn new() -> Self {
        Self::with_atlas(Atlas::default())
    }

    pub fn with_atlas(atlas: Atlas) -> Self {
        Self {
            atlas: RwLock::new(Arc::new(atlas)),
            generation: AtomicU64::new(0),
            fallback: Mutex::new(FallbackFonts::new()),
        }
    }

    pub fn atlas(&self) -> Arc<Atlas> {
        self.atlas
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn replace(&self, atlas: Atlas) {
        *self.atlas.write().unwrap_or_else(PoisonError::into_inner) = Arc::new(atlas);
        self.generation.fetch_add(1, Ordering::AcqRel);
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    fn layout(
        &self,
        text: &str,
        font: u16,
        spacing: f32,
        mut paint: impl FnMut(Ink, f32, f32),
        mut caret: impl FnMut(usize, f32),
    ) -> [f32; 2] {
        let atlas = self.atlas();
        let [space, height, _] = metrics(font);
        let mut width = 0_f32;
        let mut y = 0.;
        let mut base = 0;
        for row in text.split('\n') {
            let mut x = 0.;
            let mut previous = None;
            let mut clusters = row.grapheme_indices(true).peekable();
            while let Some((start, cluster)) = clusters.next() {
                if start > 0 {
                    x += spacing;
                }
                if cluster == " " {
                    caret(base + start, x);
                    x += space;
                    previous = None;
                } else if let Some((ch, g)) = glyph(&atlas, font, cluster) {
                    if let Some(prev) = previous {
                        x += atlas.kern(font, prev, ch);
                    }
                    caret(base + start, x);
                    paint(Ink::Bitmap(g), x, y);
                    x += g.advance;
                    previous = Some(ch);
                } else {
                    while let Some(&(_, next)) = clusters.peek() {
                        if next == " " || glyph(&atlas, font, next).is_some() {
                            break;
                        }
                        clusters.next();
                    }
                    let end = clusters.peek().map_or(row.len(), |(i, _)| *i);
                    let run = self
                        .fallback
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .shape(&row[start..end], font, height);
                    for &(index, offset) in &run.carets {
                        caret(base + start + index, x + offset);
                    }
                    let advance = run.width;
                    paint(Ink::Shaped(run), x, y);
                    x += advance;
                    previous = None;
                }
            }
            caret(base + row.len(), x);
            width = width.max(x);
            y += height;
            base += row.len() + 1;
        }
        [width, y]
    }

    pub fn measure(&self, text: &str, font: u16, spacing: f32) -> [f32; 2] {
        self.layout(text, font, spacing, |_, _, _| {}, |_, _| {})
    }

    pub fn carets(&self, text: &str, font: u16, scale: f32) -> Vec<(usize, f32)> {
        let mut stops = Vec::new();
        self.layout(
            text,
            font,
            0.,
            |_, _, _| {},
            |index, x| stops.push((index, x * scale)),
        );
        stops
    }

    pub fn draw(&self, text: &str, font: u16, tint: Tint, spacing: f32) -> Option<RgbaImage> {
        let [w, h] = self.measure(text, font, spacing);
        if w <= 0. || w > 16384. || h > 8192. || w * h > 16_777_216. {
            return None;
        }
        let atlas = self.atlas();
        let mut dest = RgbaImage::new(w.ceil() as u32 + 8, h.ceil() as u32);
        let mut pixel = |x: i32, y: i32, alpha: u8| {
            if let Some(out) = dest.get_pixel_mut_checked(x as u32, y as u32) {
                let old = u16::from(out[3]);
                *out = Rgba([
                    tint[0],
                    tint[1],
                    tint[2],
                    (old + (255 - old) * u16::from(alpha) / 255) as u8,
                ]);
            }
        };
        self.layout(
            text,
            font,
            spacing,
            |ink, x, y| match ink {
                Ink::Bitmap(glyph) => atlas.paint(glyph, x, y, &mut pixel),
                Ink::Shaped(run) => self
                    .fallback
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .paint(&run, x.round() as i32, y.round() as i32, &mut pixel),
            },
            |_, _| {},
        );
        Some(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::bitmap_font::tests::sample;

    #[test]
    fn bitmap_runs_apply_pair_kerning_and_paint_the_sheet() {
        let text = Typography::with_atlas(sample());
        assert_eq!(text.measure("AV", 36, 0.), [3. - 2. + 4., 44.]);
        let glyph = text.draw("A", 36, [1, 2, 3], 0.).unwrap();
        assert!(glyph.pixels().any(|p| p[3] > 0));
    }

    #[test]
    fn replacing_the_sheet_moves_text_from_the_fallback_to_the_bitmap() {
        let text = Typography::new();
        let before = text.generation();
        let shaped = text.measure("AV", 36, 0.)[0];
        text.replace(sample());
        assert_eq!(text.generation(), before + 1);
        assert_eq!(text.measure("AV", 36, 0.)[0], 5.);
        assert_ne!(shaped, 5.);
    }

    #[test]
    fn mixed_script_carets_and_pixels_follow_the_same_layout() {
        let text = Typography::with_atlas(sample());
        let label = "Aตั้งค่า设置V";
        let stops = text.carets(label, 36, 1.);
        let indices: Vec<_> = label
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(label.len()))
            .collect();
        assert_eq!(stops.iter().map(|s| s.0).collect::<Vec<_>>(), indices);
        assert!(stops.windows(2).all(|pair| pair[0].1 <= pair[1].1));
        assert_eq!(stops.last().unwrap().1, text.measure(label, 36, 0.)[0]);
        let base = text.draw("ก", 36, [1, 2, 3], 0.).unwrap();
        let marked = text.draw("ก่", 36, [1, 2, 3], 0.).unwrap();
        assert_ne!(base, marked);
        assert!(marked.pixels().any(|p| p[3] > 0));
    }
}
