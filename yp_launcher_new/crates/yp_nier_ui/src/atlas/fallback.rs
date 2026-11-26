use cosmic_text::{
    Attrs, Buffer, Color, Cursor, Family, FontSystem, Metrics, PhysicalGlyph, Shaping, SwashCache,
    Wrap, fontdb,
};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use unicode_segmentation::UnicodeSegmentation;

pub(super) fn fonts() -> &'static [Vec<u8>; 2] {
    static FONTS: OnceLock<[Vec<u8>; 2]> = OnceLock::new();
    FONTS.get_or_init(|| {
        [
            super::unpack_gzip(include_bytes!("../../assets/type/NotoSansThai.ttf.gz")),
            super::unpack_gzip(include_bytes!(
                "../../assets/type/NotoSansCJKsc-menu.otf.gz"
            )),
        ]
    })
}

pub(super) struct ShapedRun {
    pub width: f32,
    pub carets: Vec<(usize, f32)>,
    glyphs: Vec<PhysicalGlyph>,
}

pub(super) struct FallbackFonts {
    fonts: FontSystem,
    raster: SwashCache,
    runs: HashMap<(u16, String), Arc<ShapedRun>>,
}

impl FallbackFonts {
    pub fn new() -> Self {
        let mut db = fontdb::Database::new();
        for bytes in fonts() {
            db.load_font_source(fontdb::Source::Binary(Arc::new(bytes.as_slice())));
        }
        db.set_sans_serif_family("Noto Sans CJK SC");
        Self {
            fonts: FontSystem::new_with_locale_and_db("en-US".into(), db),
            raster: SwashCache::new(),
            runs: HashMap::new(),
        }
    }

    pub fn shape(&mut self, text: &str, font: u16, height: f32) -> Arc<ShapedRun> {
        let key = (font, text.to_owned());
        if let Some(run) = self.runs.get(&key) {
            return run.clone();
        }
        let family = if text.chars().any(|c| ('\u{e00}'..='\u{e7f}').contains(&c)) {
            Family::Name("Noto Sans Thai")
        } else {
            Family::SansSerif
        };
        let mut buffer = Buffer::new_empty(Metrics::new(height * 0.76, height));
        buffer.set_wrap(Wrap::None);
        let attrs = Attrs::new().family(family);
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        let mut shaped = ShapedRun {
            width: 0.,
            carets: Vec::new(),
            glyphs: Vec::new(),
        };
        if let Some(line) = buffer.layout_runs().next() {
            shaped.width = line.line_w;
            shaped.glyphs = line
                .glyphs
                .iter()
                .map(|g| g.physical((0., line.line_y), 1.))
                .collect();
            for (i, _) in text.grapheme_indices(true) {
                if let Some(x) = line.cursor_position(&Cursor::new(0, i)) {
                    shaped.carets.push((i, x));
                }
            }
        }
        let run = Arc::new(shaped);
        if self.runs.len() >= 512 {
            self.runs.clear();
        }
        self.runs.insert(key, run.clone());
        run
    }

    pub fn paint(&mut self, run: &ShapedRun, x: i32, y: i32, mut pixel: impl FnMut(i32, i32, u8)) {
        if self.raster.image_cache.len() > 4096 {
            self.raster.image_cache.clear();
        }
        for g in &run.glyphs {
            self.raster.with_pixels(
                &mut self.fonts,
                g.cache_key,
                Color::rgb(255, 255, 255),
                |i, j, c| {
                    if c.a() != 0 {
                        pixel(x + g.x + i, y + g.y + j, c.a());
                    }
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localized_runs_resolve_without_installed_fonts() {
        let mut fonts = FallbackFonts::new();
        for label in ["ตั้งค่า", "เปิดใช้งาน", "设置", "浏览文件夹"]
        {
            let run = fonts.shape(label, 36, 44.);
            assert!(run.width > 0.);
            assert!(!run.glyphs.is_empty());
            assert!(run.glyphs.iter().all(|g| g.cache_key.glyph_id != 0));
        }
    }
}
