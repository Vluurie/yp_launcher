use gpui::{App, Global, Pixels, Window, px};

use crate::theme::ActiveTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Breakpoint {
    Compact,
    Medium,
    #[default]
    Wide,
}

impl Breakpoint {
    pub const ALL: [Breakpoint; 3] = [Breakpoint::Compact, Breakpoint::Medium, Breakpoint::Wide];
    pub const MEDIUM_MIN_WIDTH: Pixels = px(800.);
    pub const WIDE_MIN_WIDTH: Pixels = px(1100.);
    pub const SHORT_MAX_HEIGHT: Pixels = px(500.);
    pub const ONE_COLUMN_MAX_WIDTH: Pixels = px(900.);

    pub fn for_width(width: Pixels) -> Self {
        if width < Self::MEDIUM_MIN_WIDTH {
            Breakpoint::Compact
        } else if width < Self::WIDE_MIN_WIDTH {
            Breakpoint::Medium
        } else {
            Breakpoint::Wide
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scaled {
    pub compact: f32,
    pub medium: f32,
    pub wide: f32,
}

impl Scaled {
    pub const fn new(compact: f32, medium: f32, wide: f32) -> Self {
        Self {
            compact,
            medium,
            wide,
        }
    }

    pub fn at(&self, breakpoint: Breakpoint) -> Pixels {
        px(match breakpoint {
            Breakpoint::Compact => self.compact,
            Breakpoint::Medium => self.medium,
            Breakpoint::Wide => self.wide,
        })
    }
}

macro_rules! scaled_tokens {
    ($($name:ident: $compact:expr, $medium:expr, $wide:expr;)*) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct ThemeSizes {
            $(pub $name: Scaled,)*
        }

        impl Default for ThemeSizes {
            fn default() -> Self {
                Self {
                    $($name: Scaled::new($compact, $medium, $wide),)*
                }
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct Sizes {
            pub breakpoint: Breakpoint,
            pub window_width: Pixels,
            pub window_height: Pixels,
            $(pub $name: Pixels,)*
        }

        impl ThemeSizes {
            pub fn resolve(
                &self,
                breakpoint: Breakpoint,
                window_width: Pixels,
                window_height: Pixels,
            ) -> Sizes {
                Sizes {
                    breakpoint,
                    window_width,
                    window_height,
                    $($name: self.$name.at(breakpoint),)*
                }
            }
        }
    };
}

scaled_tokens! {
    font_xs: 13., 15., 17.;
    font_sm: 14., 16., 19.;
    font_md: 15., 17., 20.;
    font_lg: 17., 19., 22.;
    font_xl: 18., 21., 24.;
    font_xxl: 24., 30., 36.;
    icon_sm: 14., 16., 18.;
    icon_md: 16., 19., 22.;
    icon_lg: 18., 22., 26.;
    padding_xs: 2., 4., 6.;
    padding_sm: 4., 6., 8.;
    padding_md: 6., 10., 14.;
    padding_lg: 8., 14., 20.;
    padding_xl: 12., 20., 32.;
    spacing_xs: 1., 2., 3.;
    spacing_sm: 2., 4., 6.;
    spacing_md: 4., 6., 8.;
    spacing_lg: 6., 10., 16.;
    border_radius: 4., 6., 8.;
    card_padding_h: 8., 12., 16.;
    card_padding_v: 6., 8., 10.;
    info_bar_bottom: 28., 34., 40.;
    info_bar_padding_h: 8., 10., 14.;
    info_bar_padding_v: 6., 8., 10.;
    chip_padding_h: 4., 6., 8.;
    chip_padding_v: 1., 2., 3.;
    drop_zone_padding_v: 14., 18., 24.;
    content_padding: 8., 14., 20.;
    sidebar_width: 64., 180., 200.;
    docs_nav_width: 150., 220., 280.;
    docs_outline_width: 120., 170., 220.;
    sidebar_row_height: 44., 38., 40.;
    sidebar_section_gap: 10., 14., 18.;
    dialog_width: 400., 480., 560.;
    play_button_width: 160., 200., 250.;
    play_button_height: 44., 56., 68.;
    play_button_loading_size: 22., 28., 34.;
    config_input_width: 70., 90., 110.;
    checkbox_size: 20., 23., 26.;
    tab_padding_h: 6., 10., 16.;
    tab_font: 10., 14., 15.;
}

impl Sizes {
    pub const TITLE_BAR_HEIGHT: Pixels = px(38.);
    pub const TITLE_BAR_BUTTON_WIDTH: Pixels = px(50.);
    pub const TWO_COLUMN_BREAKPOINT: Pixels = px(760.);
    pub const TOOLTIP_MAX_WIDTH: Pixels = px(320.);
    pub const MENU_MAX_WIDTH: Pixels = px(480.);
    pub const SELECT_MENU_MIN_HEIGHT: Pixels = px(96.);
    pub const SELECT_MENU_MAX_HEIGHT: Pixels = px(320.);
    pub const TOAST_MIN_WIDTH: Pixels = px(280.);
    pub const TOAST_MAX_WIDTH: Pixels = px(720.);
    pub const TOAST_WINDOW_MARGIN: Pixels = px(80.);
    pub const SCROLLBAR_WIDTH: Pixels = px(10.);
    pub const SCROLLBAR_THUMB_IDLE: Pixels = px(3.);
    pub const SCROLLBAR_THUMB_ACTIVE: Pixels = px(6.);
    pub const FOCUS_RING_WIDTH: Pixels = px(2.);
    pub const CAPTION_BUTTON_WIDTH: Pixels = px(46.);
    pub const CAPTION_GLYPH_SIZE: Pixels = px(10.);
    pub const SCROLLBAR_MIN_THUMB: Pixels = px(24.);
    pub const SCROLL_OVERFLOW_MIN: Pixels = px(1.5);
    pub const BORDER_WIDTH: Pixels = px(1.);
    pub const ACCENT_BAR_WIDTH: Pixels = px(2.);
    pub const ROW_RADIUS: Pixels = px(6.);
    pub const CONTROL_RADIUS: Pixels = px(4.);
    pub const SLIDER_TRACK_HEIGHT: Pixels = px(3.);
    pub const SLIDER_THUMB_SIZE: Pixels = px(14.);
    pub const SLIDER_HIT_HEIGHT: Pixels = px(32.);
    pub const SWITCH_WIDTH: Pixels = px(36.);
    pub const SWITCH_HEIGHT: Pixels = px(20.);
    pub const PREVIEW_ROW_HEIGHT: Pixels = px(48.);
    pub const PREVIEW_THUMB_WIDTH: Pixels = px(80.);
    pub const LIGHTBOX_INSET: Pixels = px(32.);
    pub const MENU_ITEM_PADDING_H: Pixels = px(12.);
    pub const MENU_ITEM_PADDING_V: Pixels = px(7.);
    pub const MENU_GAP: Pixels = px(2.);
    pub const WINDOW_MARGIN: Pixels = px(8.);
    pub const INDICATOR_SIZE: Pixels = px(8.);
    pub const NIER_GRID_SPACING: Pixels = px(10.);
    pub const NIER_BORDER_TILE_WIDTH: Pixels = px(64.);
    pub const NIER_BORDER_TILE_HEIGHT: Pixels = px(14.);
    pub const NIER_BORDER_INSET: Pixels = px(20.);
    pub const SHEET_MIN_WIDTH: Pixels = px(320.);
    pub const SHEET_DEFAULT_WIDTH: Pixels = px(520.);
    pub const RESIZE_HANDLE_WIDTH: Pixels = px(6.);
    pub const TOUR_CARD_WIDTH: Pixels = px(320.);
    pub const TOUR_SPOTLIGHT_PADDING: Pixels = px(6.);

    pub fn is_compact(&self) -> bool {
        self.breakpoint == Breakpoint::Compact
    }

    pub fn is_medium(&self) -> bool {
        self.breakpoint == Breakpoint::Medium
    }

    pub fn is_wide(&self) -> bool {
        self.breakpoint == Breakpoint::Wide
    }

    pub fn is_short(&self) -> bool {
        self.window_height < Breakpoint::SHORT_MAX_HEIGHT
    }

    pub fn use_one_column(&self) -> bool {
        self.window_width < Breakpoint::ONE_COLUMN_MAX_WIDTH
    }

    pub fn sidebar_labels_visible(&self) -> bool {
        self.breakpoint != Breakpoint::Compact
    }

    pub fn title_bar_height(&self) -> Pixels {
        Self::TITLE_BAR_HEIGHT
    }

    pub fn title_bar_button_width(&self) -> Pixels {
        Self::TITLE_BAR_BUTTON_WIDTH
    }

    pub fn two_column_breakpoint(&self) -> Pixels {
        Self::TWO_COLUMN_BREAKPOINT
    }

    pub fn tooltip_max_width(&self) -> Pixels {
        Self::TOOLTIP_MAX_WIDTH
    }

    pub fn menu_max_width(&self) -> Pixels {
        Self::MENU_MAX_WIDTH
    }

    pub fn toast_width(&self) -> Pixels {
        (self.window_width - Self::TOAST_WINDOW_MARGIN)
            .max(Self::TOAST_MIN_WIDTH)
            .min(Self::TOAST_MAX_WIDTH)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct BreakpointOverride(Option<Breakpoint>);

impl Global for BreakpointOverride {}

pub fn set_breakpoint_override(breakpoint: Option<Breakpoint>, cx: &mut App) {
    cx.set_global(BreakpointOverride(breakpoint));
    cx.refresh_windows();
}

pub fn breakpoint_override(cx: &App) -> Option<Breakpoint> {
    cx.try_global::<BreakpointOverride>()
        .and_then(|forced| forced.0)
}

pub trait ResponsiveExt {
    fn breakpoint(&self, cx: &App) -> Breakpoint;
    fn sizes(&self, cx: &App) -> Sizes;

    fn is_compact(&self, cx: &App) -> bool {
        self.breakpoint(cx) == Breakpoint::Compact
    }

    fn is_medium(&self, cx: &App) -> bool {
        self.breakpoint(cx) == Breakpoint::Medium
    }

    fn is_wide(&self, cx: &App) -> bool {
        self.breakpoint(cx) == Breakpoint::Wide
    }

    fn is_short(&self, cx: &App) -> bool {
        self.sizes(cx).is_short()
    }

    fn use_one_column(&self, cx: &App) -> bool {
        self.sizes(cx).use_one_column()
    }

    fn sidebar_labels_visible(&self, cx: &App) -> bool {
        self.sizes(cx).sidebar_labels_visible()
    }
}

impl ResponsiveExt for Window {
    fn breakpoint(&self, cx: &App) -> Breakpoint {
        breakpoint_override(cx).unwrap_or_else(|| Breakpoint::for_width(self.viewport_size().width))
    }

    fn sizes(&self, cx: &App) -> Sizes {
        let viewport = self.viewport_size();
        let mut sizes =
            cx.theme()
                .sizes
                .resolve(self.breakpoint(cx), viewport.width, viewport.height);
        let factor = super::MenuTextSize::factor(cx);
        for font in [
            &mut sizes.font_xs,
            &mut sizes.font_sm,
            &mut sizes.font_md,
            &mut sizes.font_lg,
            &mut sizes.font_xl,
            &mut sizes.font_xxl,
        ] {
            *font *= factor;
        }
        if factor > 1. {
            sizes.play_button_height = sizes.play_button_height.max(sizes.font_xl * 1.7);
        }
        sizes
    }
}
