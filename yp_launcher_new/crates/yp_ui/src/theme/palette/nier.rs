#![allow(clippy::disallowed_methods)]

use gpui::{rgb, rgba};

use crate::theme::ThemeColors;

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
