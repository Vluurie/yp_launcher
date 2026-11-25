use super::Tint;
use gpui::{FontId, FontRun, LineLayout, PlatformTextSystem, RenderGlyphParams, font, point, px};
use image::{Rgba, RgbaImage};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use unicode_segmentation::UnicodeSegmentation;

pub(super) fn dimensions(face: u16) -> (f32, f32) {
    match face {
        2 => (61., 76.),
        1 => (34., 44.),
        6 => (26., 44.),
        _ => (30., 44.),
    }
}

type Lines = HashMap<(u16, String), Arc<LineLayout>>;
struct Bitmap {
    x: i32,
    y: i32,
    width: usize,
    coverage: Vec<u8>,
}
pub(super) struct Typography {
    system: Arc<dyn PlatformTextSystem>,
    face: FontId,
    lines: Mutex<Lines>,
    glyphs: Mutex<HashMap<RenderGlyphParams, Arc<Bitmap>>>,
}

impl Typography {
    pub fn new(system: Arc<dyn PlatformTextSystem>) -> Self {
        let desc = font(".SystemUIFont");
        let face = system
            .font_id(&desc)
            .or_else(|_| system.font_id(&font("sans-serif")))
            .expect("system UI font");
        Self {
            system,
            face,
            lines: Default::default(),
            glyphs: Default::default(),
        }
    }
    fn line(&self, text: &str, font: u16) -> Arc<LineLayout> {
        let key = (font, text.to_owned());
        let mut cache = self.lines.lock().unwrap();
        if let Some(value) = cache.get(&key) {
            return value.clone();
        }
        let runs = [FontRun {
            len: text.len(),
            font_id: self.face,
        }];
        let line = Arc::new(self.system.layout_line(text, px(dimensions(font).0), &runs));
        if cache.len() >= 768 {
            cache.clear();
        }
        cache.insert(key, line.clone());
        line
    }
    pub fn measure(&self, text: &str, font: u16, spacing: f32) -> [f32; 2] {
        let mut width = 0_f32;
        let mut rows = 0;
        for row in text.split('\n') {
            let line = self.line(row, font);
            let glyphs = line.runs.iter().map(|r| r.glyphs.len()).sum::<usize>();
            width = width.max(f32::from(line.width) + spacing * glyphs.saturating_sub(1) as f32);
            rows += 1;
        }
        [width, rows as f32 * dimensions(font).1]
    }
    pub fn carets(&self, text: &str, font: u16, scale: f32) -> Vec<(usize, f32)> {
        let mut out = Vec::new();
        let mut base = 0;
        for row in text.split('\n') {
            let line = self.line(row, font);
            let spacing = super::Theme::tracking(font);
            let glyphs: Vec<_> = line
                .runs
                .iter()
                .flat_map(|run| run.glyphs.iter())
                .map(|g| f32::from(g.position.x))
                .collect();
            for (i, _) in row.grapheme_indices(true) {
                let x = f32::from(line.x_for_index(i));
                let before = glyphs.iter().filter(|&&pos| pos < x).count();
                out.push((base + i, (x + before as f32 * spacing) * scale));
            }
            out.push((
                base + row.len(),
                (f32::from(line.width) + glyphs.len().saturating_sub(1) as f32 * spacing) * scale,
            ));
            base += row.len() + 1;
        }
        out
    }
    fn glyph(&self, params: RenderGlyphParams) -> Option<Arc<Bitmap>> {
        let mut cache = self.glyphs.lock().unwrap();
        if let Some(g) = cache.get(&params) {
            return Some(g.clone());
        }
        let bounds = self.system.glyph_raster_bounds(&params).ok()?;
        let (size, raw) = self.system.rasterize_glyph(&params, bounds).ok()?;
        let pixels = (size.width.0 * size.height.0) as usize;
        let coverage = if raw.len() == pixels {
            raw
        } else if raw.len() == pixels * 4 {
            raw.as_chunks::<4>().0.iter().map(|p| p[3]).collect()
        } else {
            return None;
        };
        let glyph = Arc::new(Bitmap {
            x: bounds.origin.x.0,
            y: bounds.origin.y.0,
            width: size.width.0 as usize,
            coverage,
        });
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(params, glyph.clone());
        Some(glyph)
    }
    pub fn draw(&self, text: &str, font: u16, tint: Tint, spacing: f32) -> Option<RgbaImage> {
        let [w, h] = self.measure(text, font, spacing);
        if w <= 0. || w > 16384. || h > 8192. || w * h > 16_777_216. {
            return None;
        }
        let (size, line_height) = dimensions(font);
        let mut dest = RgbaImage::new(w.ceil() as u32 + 8, h.ceil() as u32);
        for (row_no, row) in text.split('\n').enumerate() {
            let line = self.line(row, font);
            let ascent = f32::from(line.ascent);
            let baseline = row_no as f32 * line_height
                + (line_height - ascent - f32::from(line.descent)) * 0.5
                + ascent;
            let mut tracking = 0.;
            for run in &line.runs {
                for g in &run.glyphs {
                    let x = (f32::from(g.position.x) + tracking).round() as i32;
                    let y = (baseline + f32::from(g.position.y)).round() as i32;
                    tracking += spacing;
                    let params = RenderGlyphParams {
                        font_id: run.font_id,
                        glyph_id: g.id,
                        font_size: px(size),
                        subpixel_variant: point(0, 0),
                        scale_factor: 1.,
                        is_emoji: g.is_emoji,
                        subpixel_rendering: false,
                        dilation: 0,
                    };
                    if let Some(b) = self.glyph(params) {
                        for (j, scan) in b.coverage.chunks(b.width.max(1)).enumerate() {
                            let dy = y + b.y + j as i32;
                            if dy < 0 || dy >= dest.height() as i32 {
                                continue;
                            }
                            for (i, &a) in scan.iter().enumerate().filter(|(_, a)| **a > 0) {
                                let dx = x + b.x + i as i32;
                                if let Some(p) = dest.get_pixel_mut_checked(dx as u32, dy as u32) {
                                    let old = u16::from(p[3]);
                                    *p = Rgba([
                                        tint[0],
                                        tint[1],
                                        tint[2],
                                        (old + (255 - old) * u16::from(a) / 255) as u8,
                                    ]);
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(dest)
    }
}
