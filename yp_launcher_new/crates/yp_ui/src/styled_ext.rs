use gpui::{App, Div, IntoElement, ParentElement, Styled, div};

use crate::theme::{ActiveTheme, Elevation, Surface};

pub trait StyledExt: Styled + Sized {
    fn h_flex(self) -> Self {
        self.flex().flex_row().items_center()
    }

    fn v_flex(self) -> Self {
        self.flex().flex_col()
    }

    fn elevation(self, elevation: Elevation, cx: &App) -> Self {
        match elevation {
            Elevation::None => self,
            other => self.shadow(cx.theme().elevation.shadows(other)),
        }
    }

    fn elevation_1(self, cx: &App) -> Self {
        self.elevation(Elevation::Card, cx)
    }

    fn elevation_2(self, cx: &App) -> Self {
        self.elevation(Elevation::Popover, cx)
    }

    fn elevation_3(self, cx: &App) -> Self {
        self.elevation(Elevation::Modal, cx)
    }

    fn surface(self, cx: &App, surface: Surface) -> Self {
        self.bg(surface.color(cx))
    }

    fn font_ui(self, cx: &App) -> Self {
        self.font(cx.theme().fonts.ui_font())
    }

    fn font_mono(self, cx: &App) -> Self {
        self.font(cx.theme().fonts.mono_font())
    }
}

impl<E: Styled> StyledExt for E {}

pub fn h_flex() -> Div {
    div().h_flex()
}

pub fn v_flex() -> Div {
    div().v_flex()
}

pub fn fill_parent(content: impl IntoElement) -> Div {
    div()
        .relative()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .child(div().absolute().inset_0().child(content))
}
