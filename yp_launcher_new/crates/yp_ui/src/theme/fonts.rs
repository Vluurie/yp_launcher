use gpui::{Font, FontFallbacks, FontFeatures, FontStyle, FontWeight, SharedString};

pub const UI_FONT_FAMILY: &str = "Rajdhani";
pub const MONO_FONT_FAMILY: &str = "Consolas";
pub const UI_FONT_FALLBACKS: [&str; 6] = [
    "Segoe UI",
    "Microsoft YaHei UI",
    "Leelawadee UI",
    "Noto Sans",
    "Noto Sans CJK SC",
    "Noto Sans Thai",
];
pub const MONO_FONT_FALLBACKS: [&str; 6] = [
    "Cascadia Mono",
    "DejaVu Sans Mono",
    "Liberation Mono",
    "Noto Sans Mono",
    "Menlo",
    "Courier New",
];

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeFonts {
    pub ui: SharedString,
    pub ui_fallbacks: Vec<SharedString>,
    pub mono: SharedString,
    pub mono_fallbacks: Vec<SharedString>,
}

impl Default for ThemeFonts {
    fn default() -> Self {
        Self {
            ui: SharedString::new_static(UI_FONT_FAMILY),
            ui_fallbacks: UI_FONT_FALLBACKS
                .into_iter()
                .map(SharedString::new_static)
                .collect(),
            mono: SharedString::new_static(MONO_FONT_FAMILY),
            mono_fallbacks: MONO_FONT_FALLBACKS
                .into_iter()
                .map(SharedString::new_static)
                .collect(),
        }
    }
}

impl ThemeFonts {
    pub fn ui_font(&self) -> Font {
        Font {
            family: self.ui.clone(),
            features: FontFeatures::default(),
            fallbacks: Some(FontFallbacks::from_fonts(
                self.ui_fallbacks
                    .iter()
                    .map(|family| family.to_string())
                    .collect(),
            )),
            weight: FontWeight::MEDIUM,
            style: FontStyle::Normal,
        }
    }

    pub fn mono_font(&self) -> Font {
        Font {
            family: self.mono.clone(),
            features: FontFeatures::default(),
            fallbacks: Some(FontFallbacks::from_fonts(
                self.mono_fallbacks
                    .iter()
                    .map(|family| family.to_string())
                    .collect(),
            )),
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
        }
    }
}
