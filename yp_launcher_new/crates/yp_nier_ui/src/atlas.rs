mod symbols;
mod typography;

use gpui::{PlatformTextSystem, RenderImage};
use image::{Frame, RgbaImage};
use std::sync::{Arc, OnceLock};
use typography::Typography;

pub type Tint = [u8; 3];
pub const INK: Tint = [75, 73, 64];
pub const PAPER: Tint = [202, 198, 173];
pub const PANEL: Tint = [179, 175, 152];
pub const LIGHT: Tint = [217, 212, 187];
pub const WHITE: Tint = [255; 3];

pub(crate) fn image(mut pixels: RgbaImage) -> Arc<RenderImage> {
    for p in pixels.pixels_mut() {
        p.0.swap(0, 2);
    }
    Arc::new(RenderImage::new(vec![Frame::new(pixels)]))
}

pub struct Theme {
    text: Typography,
}
static THEME: OnceLock<Theme> = OnceLock::new();

impl Theme {
    pub fn with_text_system(system: Arc<dyn PlatformTextSystem>) -> &'static Self {
        THEME.get_or_init(|| Self {
            text: Typography::new(system),
        })
    }
    pub fn bundled() -> &'static Self {
        THEME.get_or_init(|| Self {
            text: Typography::new(gpui_platform::current_platform(true).text_system()),
        })
    }
    pub(crate) fn tracking(font: u16) -> f32 {
        match font {
            1 => 2.,
            36 => 1.6,
            6 => 0.7,
            _ => 0.,
        }
    }
    pub fn font_height(&self, font: u16) -> f32 {
        typography::dimensions(font).1
    }
    pub fn sprite_names(&self) -> impl Iterator<Item = &str> {
        symbols::SYMBOLS.iter().map(|&(n, _)| n)
    }
    pub fn sprite_size(&self, name: &str) -> Option<[u32; 2]> {
        symbols::size(name)
    }
    pub(crate) fn sprite_image(&self, name: &str, tint: Tint) -> Option<Arc<RenderImage>> {
        symbols::draw(name, tint).map(image)
    }
    pub fn measure(&self, text: &str, font: u16, scale: f32) -> [f32; 2] {
        self.measure_spaced(text, font, Self::tracking(font))
            .map(|x| x * scale)
    }
    pub(crate) fn measure_spaced(&self, text: &str, font: u16, spacing: f32) -> [f32; 2] {
        self.text.measure(text, font, spacing)
    }
    pub(crate) fn caret_positions(&self, text: &str, font: u16, scale: f32) -> Vec<(usize, f32)> {
        self.text.carets(text, font, scale)
    }
    pub(crate) fn run_image(
        &self,
        text: &str,
        font: u16,
        tint: Tint,
        spacing: f32,
        alpha: u8,
    ) -> Option<Arc<RenderImage>> {
        let mut run = self.text.draw(text, font, tint, spacing)?;
        if alpha < 255 {
            for p in run.pixels_mut() {
                p[3] = (u16::from(p[3]) * u16::from(alpha) / 255) as u8;
            }
        }
        Some(image(run))
    }
    #[cfg(feature = "uid-debug")]
    pub(crate) fn text_image_spaced(
        &self,
        text: &str,
        font: u16,
        tint: Tint,
        spacing: f32,
    ) -> Option<Arc<RenderImage>> {
        self.run_image(text, font, tint, spacing, 255)
    }
    pub(crate) fn wrapped(&self, value: &str, font: u16, scale: f32, width: f32) -> String {
        use unicode_segmentation::UnicodeSegmentation;
        let fits = |s: &str| self.measure(s, font, scale)[0] <= width;
        let mut lines = Vec::new();
        for paragraph in value.split('\n') {
            let mut line = String::new();
            for word in paragraph.split_word_bounds() {
                let mut joined = line.clone();
                joined.push_str(word);
                if fits(&joined) {
                    line = joined;
                    continue;
                }
                if !line.is_empty() {
                    lines.push(line.trim_end().to_owned());
                    line.clear();
                }
                if word.trim().is_empty() {
                    continue;
                }
                if fits(word) {
                    line.push_str(word);
                    continue;
                }
                for glyph in word.graphemes(true) {
                    let end = line.len();
                    line.push_str(glyph);
                    if end != 0 && !fits(&line) {
                        line.truncate(end);
                        lines.push(std::mem::take(&mut line));
                        line.push_str(glyph);
                    }
                }
            }
            lines.push(line);
        }
        lines.join("\n")
    }
}
