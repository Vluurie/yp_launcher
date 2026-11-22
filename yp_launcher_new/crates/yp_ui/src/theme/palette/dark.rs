#![allow(clippy::disallowed_methods)]

use gpui::{rgb, rgba};

use crate::theme::ThemeColors;

pub(crate) fn colors() -> ThemeColors {
    ThemeColors {
        background_primary: rgb(0x18181c).into(),
        background_secondary: rgb(0x141418).into(),
        background_card: rgb(0x222228).into(),
        surface_light: rgb(0x2c2c34).into(),
        surface_medium: rgb(0x1e1e24).into(),
        text_primary: rgb(0xffffff).into(),
        text_secondary: rgb(0xd4d0c8).into(),
        text_muted: rgb(0x9e9a90).into(),
        text_on_dark: rgb(0xffffff).into(),
        accent_primary: rgb(0xd4a86a).into(),
        accent_secondary: rgb(0xb89860).into(),
        accent_highlight: rgb(0xe4c88b).into(),
        border_light: rgb(0x3a3a42).into(),
        border_medium: rgb(0x4e4e58).into(),
        success: rgb(0x8bc34a).into(),
        error: rgb(0xef5350).into(),
        warning: rgb(0xffb74d).into(),
        button_background: rgb(0xd4a86a).into(),
        button_text: rgb(0x1a1a1e).into(),
        input_background: rgb(0x2a2a32).into(),
        log_background: rgb(0x222228).into(),
        log_header_background: rgb(0x1e1e24).into(),
        log_text: rgb(0xd4d0c8).into(),
        log_debug: rgb(0x64b5f6).into(),
        shadow: rgba(0x00000055).into(),
        tooltip_background: rgba(0x303030e6).into(),
        scrim: rgb(0x000000).into(),
        ornament: rgb(0x48463d).into(),
        transparent: rgba(0x00000000).into(),
    }
}
