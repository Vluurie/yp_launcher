mod colors;
mod fonts;
mod motion;
mod palette;
mod sizes;
mod style;

use std::sync::Arc;

use gpui::{App, Global};

pub use colors::{Alpha, Color, Severity, StatusColor, StatusColors, Surface, ThemeColors};
pub use fonts::{
    MONO_FONT_FALLBACKS, MONO_FONT_FAMILY, ThemeFonts, UI_FONT_FALLBACKS, UI_FONT_FAMILY,
};
pub use motion::{Motion, easing};
pub use sizes::{
    Breakpoint, ResponsiveExt, Scaled, Sizes, ThemeSizes, breakpoint_override,
    set_breakpoint_override,
};
pub use style::{ButtonHover, Elevation, NavHover, ThemeElevation, ThemeStyle};

#[derive(Clone, Copy)]
pub struct MenuTextSize(u16);

impl Global for MenuTextSize {}

impl MenuTextSize {
    pub const OPTIONS: [u16; 4] = [100, 125, 150, 200];

    pub fn new(percent: u16) -> Self {
        Self(if Self::OPTIONS.contains(&percent) {
            percent
        } else {
            100
        })
    }

    pub fn percent(cx: &App) -> u16 {
        cx.try_global::<Self>().map_or(100, |scale| scale.0)
    }

    pub fn factor(cx: &App) -> f32 {
        if cx.theme().id == ThemeId::NierNew {
            f32::from(Self::percent(cx)) / 100.
        } else {
            1.
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum ThemeId {
    #[default]
    Dark,
    Nier,
    NierNew,
}

impl ThemeId {
    pub const ALL: [ThemeId; 3] = [ThemeId::Dark, ThemeId::Nier, ThemeId::NierNew];

    pub fn code(self) -> &'static str {
        match self {
            ThemeId::Dark => "dark",
            ThemeId::Nier => "nier",
            ThemeId::NierNew => "nier-new",
        }
    }

    pub fn from_code(code: &str) -> Option<ThemeId> {
        ThemeId::ALL.into_iter().find(|id| id.code() == code)
    }

    pub fn toggled(self) -> ThemeId {
        match self {
            ThemeId::Dark => ThemeId::Nier,
            ThemeId::Nier | ThemeId::NierNew => ThemeId::Dark,
        }
    }

    pub fn appearance(self) -> Appearance {
        match self {
            ThemeId::Dark => Appearance::Dark,
            ThemeId::Nier | ThemeId::NierNew => Appearance::Light,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Appearance {
    #[default]
    Dark,
    Light,
}

impl Appearance {
    pub fn is_light(self) -> bool {
        self == Appearance::Light
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub id: ThemeId,
    pub appearance: Appearance,
    pub colors: ThemeColors,
    pub alpha: Alpha,
    pub status: StatusColors,
    pub sizes: ThemeSizes,
    pub motion: Motion,
    pub fonts: ThemeFonts,
    pub style: ThemeStyle,
    pub elevation: ThemeElevation,
}

impl Theme {
    pub fn by_id(id: ThemeId) -> Self {
        let colors = palette::colors(id);
        let alpha = Alpha::default();
        Self {
            id,
            appearance: id.appearance(),
            status: StatusColors::from_colors(&colors, &alpha),
            style: ThemeStyle::for_theme(id, &colors, &alpha),
            elevation: ThemeElevation::from_colors(&colors, &alpha),
            colors,
            alpha,
            sizes: if id == ThemeId::NierNew {
                ThemeSizes {
                    font_xs: Scaled::new(13., 14., 15.),
                    font_sm: Scaled::new(14., 15., 16.),
                    font_md: Scaled::new(15., 16., 17.),
                    font_lg: Scaled::new(17., 19., 21.),
                    border_radius: Scaled::new(0., 0., 0.),
                    ..ThemeSizes::default()
                }
            } else {
                ThemeSizes::default()
            },
            motion: Motion::default(),
            fonts: if id == ThemeId::NierNew {
                ThemeFonts {
                    ui: ".SystemUIFont".into(),
                    ..ThemeFonts::default()
                }
            } else {
                ThemeFonts::default()
            },
        }
    }

    pub fn dark() -> Self {
        Self::by_id(ThemeId::Dark)
    }

    pub fn nier() -> Self {
        Self::by_id(ThemeId::Nier)
    }

    pub fn id(&self) -> ThemeId {
        self.id
    }

    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    pub fn colors(&self) -> &ThemeColors {
        &self.colors
    }

    pub fn alpha(&self) -> &Alpha {
        &self.alpha
    }

    pub fn status(&self) -> &StatusColors {
        &self.status
    }

    pub fn sizes(&self) -> &ThemeSizes {
        &self.sizes
    }

    pub fn motion(&self) -> &Motion {
        &self.motion
    }

    pub fn fonts(&self) -> &ThemeFonts {
        &self.fonts
    }

    pub fn style(&self) -> &ThemeStyle {
        &self.style
    }

    pub fn elevation(&self) -> &ThemeElevation {
        &self.elevation
    }
}

pub struct GlobalTheme(pub Arc<Theme>);

impl Global for GlobalTheme {}

pub trait ActiveTheme {
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        &self.global::<GlobalTheme>().0
    }
}

pub fn init(cx: &mut App) {
    if cx.try_global::<GlobalTheme>().is_none() {
        cx.set_global(GlobalTheme(Arc::new(Theme::by_id(ThemeId::default()))));
    }
}

pub fn set_theme(id: ThemeId, cx: &mut App) {
    cx.set_global(GlobalTheme(Arc::new(Theme::by_id(id))));
    cx.refresh_windows();
}

pub fn toggle_theme(cx: &mut App) {
    let next = cx.theme().id.toggled();
    set_theme(next, cx);
}

pub fn active_theme_id(cx: &App) -> ThemeId {
    cx.theme().id
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use gpui::{TestAppContext, px};

    use super::*;
    use crate::actions::ToggleTheme;

    #[test]
    fn theme_ids_round_trip_through_their_codes() {
        for id in ThemeId::ALL {
            assert_eq!(ThemeId::from_code(id.code()), Some(id));
        }
        assert_eq!(ThemeId::Nier.toggled(), ThemeId::Dark);
        assert_eq!(ThemeId::Dark.toggled(), ThemeId::Nier);
        assert_eq!(ThemeId::NierNew.toggled(), ThemeId::Dark);
        assert_eq!(ThemeId::from_code("light"), None);
        assert!(ThemeId::Nier.appearance().is_light());
        assert!(!ThemeId::Dark.appearance().is_light());
    }

    #[test]
    fn both_palettes_fill_every_token_and_differ_where_it_matters() {
        let dark = Theme::dark();
        let nier = Theme::nier();
        assert_ne!(
            dark.colors.background_primary,
            nier.colors.background_primary
        );
        assert_ne!(dark.colors.text_primary, nier.colors.text_primary);
        assert_ne!(dark.colors.accent_primary, nier.colors.accent_primary);
        assert_eq!(dark.colors.scrim, nier.colors.scrim);
        assert_eq!(dark.colors.transparent.a, 0.);
        assert!(dark.colors.shadow.a < 1.);
        for theme in [&dark, &nier] {
            assert_eq!(theme.status.error.fg, theme.colors.error);
            assert!(theme.status.error.background.a < theme.status.error.border.a);
            assert_eq!(theme.elevation.shadows(Elevation::None).len(), 0);
            assert_eq!(theme.elevation.shadows(Elevation::Modal).len(), 1);
        }
        assert_eq!(nier.style.nav_hover, NavHover::InkSweep);
        assert_eq!(dark.style.nav_hover, NavHover::Tint);
    }

    #[test]
    fn breakpoints_switch_at_800_and_1100() {
        assert_eq!(Breakpoint::for_width(px(799.9)), Breakpoint::Compact);
        assert_eq!(Breakpoint::for_width(px(800.)), Breakpoint::Medium);
        assert_eq!(Breakpoint::for_width(px(1099.9)), Breakpoint::Medium);
        assert_eq!(Breakpoint::for_width(px(1100.)), Breakpoint::Wide);
    }

    #[test]
    fn size_tokens_match_the_flutter_tables() {
        let tokens = ThemeSizes::default();
        let compact = tokens.resolve(Breakpoint::Compact, px(780.), px(560.));
        let medium = tokens.resolve(Breakpoint::Medium, px(1000.), px(680.));
        let wide = tokens.resolve(Breakpoint::Wide, px(1300.), px(450.));
        assert_eq!(
            (compact.font_md, medium.font_md, wide.font_md),
            (px(15.), px(17.), px(20.))
        );
        assert_eq!(
            (
                compact.sidebar_width,
                medium.sidebar_width,
                wide.sidebar_width
            ),
            (px(64.), px(180.), px(200.))
        );
        assert_eq!(
            (compact.dialog_width, medium.dialog_width, wide.dialog_width),
            (px(400.), px(480.), px(560.))
        );
        assert_eq!(compact.sidebar_row_height, px(44.));
        assert!(compact.is_compact() && !compact.sidebar_labels_visible());
        assert!(medium.is_medium() && medium.sidebar_labels_visible());
        assert!(wide.is_wide() && wide.is_short());
        assert!(compact.use_one_column() && !medium.use_one_column());
        assert_eq!(compact.toast_width(), px(700.));
        assert_eq!(wide.toast_width(), Sizes::TOAST_MAX_WIDTH);
        assert_eq!(
            tokens
                .resolve(Breakpoint::Compact, px(200.), px(200.))
                .toast_width(),
            Sizes::TOAST_MIN_WIDTH
        );
    }

    #[test]
    fn toast_lifetime_is_clamped_between_four_and_twelve_seconds() {
        let motion = Motion::default();
        assert_eq!(motion.toast_lifetime(""), Duration::from_secs(4));
        assert_eq!(
            motion.toast_lifetime(&"x".repeat(100)),
            Duration::from_millis(5500)
        );
        assert_eq!(
            motion.toast_lifetime(&"x".repeat(5000)),
            Duration::from_secs(12)
        );
        assert_eq!(
            motion.toast_lifetime(&"\u{6587}".repeat(100)),
            Duration::from_millis(5500)
        );
    }

    #[test]
    fn easing_curves_start_at_zero_end_at_one_and_never_go_backwards() {
        let curves: [fn(f32) -> f32; 7] = [
            easing::nier_fill,
            easing::nier_smooth,
            easing::nier_rise,
            easing::nier_fade,
            easing::nier_slide,
            easing::nier_mask_wipe,
            easing::ease_out_cubic,
        ];
        for curve in curves {
            assert_eq!(curve(0.), 0.);
            assert!((curve(1.) - 1.).abs() < 1e-6);
            assert_eq!(curve(-5.), curve(0.));
            assert!((curve(9.) - 1.).abs() < 1e-6);
            let mut previous = 0.;
            for step in 0..=200 {
                let value = curve(step as f32 / 200.);
                assert!(value + 1e-6 >= previous);
                previous = value;
            }
        }
        assert_eq!(easing::ping_pong(0.), 0.);
        assert!((easing::ping_pong(0.5) - 1.).abs() < 1e-6);
        assert!(easing::ping_pong(1.).abs() < 1e-6);
    }

    #[gpui::test]
    fn the_toggle_action_swaps_the_global_theme(cx: &mut TestAppContext) {
        cx.update(crate::init_for_tests);
        cx.update(|cx| {
            assert_eq!(active_theme_id(cx), ThemeId::Dark);
            let dark_text = Color::Default.color(cx);
            cx.dispatch_action(&ToggleTheme);
            assert_eq!(active_theme_id(cx), ThemeId::Nier);
            assert_ne!(Color::Default.color(cx), dark_text);
            assert_eq!(Surface::Card.color(cx), cx.theme().colors.background_card);
            set_theme(ThemeId::Dark, cx);
            assert_eq!(cx.theme().id(), ThemeId::Dark);
        });
    }

    #[gpui::test]
    fn window_sizes_follow_the_override_before_the_window_width(cx: &mut TestAppContext) {
        use crate::components::test_support::open_empty_root;
        let (_root, cx) = open_empty_root(cx);
        cx.update(|window, cx| {
            let natural = Breakpoint::for_width(window.viewport_size().width);
            assert_eq!(window.breakpoint(cx), natural);
            set_breakpoint_override(Some(Breakpoint::Compact), cx);
            assert!(window.is_compact(cx));
            assert_eq!(window.sizes(cx).font_xs, px(13.));
            set_breakpoint_override(Some(Breakpoint::Wide), cx);
            assert_eq!(window.sizes(cx).font_xs, px(17.));
            set_breakpoint_override(None, cx);
            assert_eq!(window.breakpoint(cx), natural);
        });
    }
}
