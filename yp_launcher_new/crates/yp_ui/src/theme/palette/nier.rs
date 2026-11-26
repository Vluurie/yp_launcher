#![allow(clippy::disallowed_methods)]

use gpui::{rgb, rgba};

use crate::theme::ThemeColors;

pub(crate) fn menu_colors() -> ThemeColors {
    ThemeColors {
        background_primary: rgb(0xcac6ad).into(),
        background_card: rgb(0xd9d5bf).into(),
        background_secondary: rgb(0xc0bda5).into(),
        surface_light: rgb(0xcac7b3).into(),
        surface_medium: rgb(0xb9b59e).into(),
        text_muted: rgb(0x454338).into(),
        text_secondary: rgb(0x424035).into(),
        accent_primary: rgb(0x464338).into(),
        border_medium: rgb(0x605e50).into(),
        success: rgb(0x214a2d).into(),
        log_debug: rgb(0x21465e).into(),
        text_on_dark: rgb(0xf1efdf).into(),
        button_text: rgb(0xf1efdf).into(),
        error: rgb(0x72301f).into(),
        warning: rgb(0x603912).into(),
        ..colors()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(color: gpui::Hsla) -> f32 {
        let rgb = color.to_rgb();
        let linear = |s: f32| {
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(rgb.r) + 0.7152 * linear(rgb.g) + 0.0722 * linear(rgb.b)
    }

    #[test]
    fn launcher_text_and_status_symbols_remain_legible_on_menu_surfaces() {
        let p = menu_colors();
        let contrast = |a, b| {
            let a = luminance(a);
            let b = luminance(b);
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        };
        let focus = crate::theme::ThemeStyle::for_theme(
            crate::ThemeId::NierNew,
            &p,
            &crate::theme::Alpha::default(),
        )
        .focus_ring;
        assert_eq!(focus.a, 1.0);
        for bg in [
            p.background_primary,
            p.background_card,
            p.surface_medium,
            p.input_background,
        ] {
            for fg in [
                p.text_primary,
                p.text_muted,
                p.text_secondary,
                p.accent_primary,
                p.success,
                p.error,
                p.warning,
                p.log_debug,
            ] {
                assert!(
                    contrast(fg, bg) >= 4.5,
                    "{fg:?} on {bg:?} has contrast {}",
                    contrast(fg, bg)
                );
            }
            assert!(contrast(p.border_medium, bg) >= 3.);
            assert!(contrast(focus, bg) >= 3.);
        }
        assert!(contrast(p.button_text, p.button_background) >= 4.5);
        assert!(contrast(p.button_text, p.success) >= 4.5);
    }
}

pub(crate) fn colors() -> ThemeColors {
    ThemeColors {
        background_primary: rgb(0xdad4bb).into(),
        background_secondary: rgb(0xd2cbb0).into(),
        background_card: rgb(0xd0c9ae).into(),
        surface_light: rgb(0xc9c2a6).into(),
        surface_medium: rgb(0xb4af9a).into(),
        text_primary: rgb(0x3a382f).into(),
        text_secondary: rgb(0x4e4b42).into(),
        text_muted: rgb(0x6e6857).into(),
        text_on_dark: rgb(0xffffff).into(),
        accent_primary: rgb(0x4e4b42).into(),
        accent_secondary: rgb(0x635f53).into(),
        accent_highlight: rgb(0x3a3830).into(),
        border_light: rgb(0xb4af9a).into(),
        border_medium: rgb(0x908a78).into(),
        success: rgb(0x57632f).into(),
        error: rgb(0xd75028).into(),
        warning: rgb(0xb4682a).into(),
        button_background: rgb(0x4e4b42).into(),
        button_text: rgb(0xdad4bb).into(),
        input_background: rgb(0xc9c2a6).into(),
        log_background: rgb(0xd0c9ae).into(),
        log_header_background: rgb(0xb4af9a).into(),
        log_text: rgb(0x4e4b42).into(),
        log_debug: rgb(0x3b6e99).into(),
        shadow: rgba(0x4e4b4233).into(),
        tooltip_background: rgba(0x303030e6).into(),
        scrim: rgb(0x000000).into(),
        ornament: rgb(0x48463d).into(),
        transparent: rgba(0x00000000).into(),
    }
}
