use super::element;
use crate::{INK, Surface, Theme, Tint};
use gpui::{App, IntoElement, RenderOnce, SharedString, Window};

#[derive(IntoElement)]
pub struct GameText {
    text: SharedString,
    font: u16,
    scale: f32,
    tint: Tint,
    reveal: bool,
    wrap_width: Option<f32>,
}
impl GameText {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            font: 36,
            scale: 0.8,
            tint: INK,
            reveal: true,
            wrap_width: None,
        }
    }
    pub fn wrap(mut self, available: f32) -> Self {
        self.wrap_width = Some(available.max(0.));
        self
    }
    pub fn reveal(mut self, enabled: bool) -> Self {
        self.reveal = enabled;
        self
    }
    pub fn font_id(mut self, font: u16) -> Self {
        self.font = font;
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }
}
impl RenderOnce for GameText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::bundled();
        let text = match self.wrap_width {
            Some(max)
                if theme.measure(&self.text, self.font, self.scale)[0].ceil() + 8. * self.scale
                    > max =>
            {
                theme
                    .wrapped(
                        &self.text,
                        self.font,
                        self.scale,
                        (max - 8. * self.scale).max(0.),
                    )
                    .into()
            }
            _ => self.text,
        };
        let [w, h] = theme.measure(&text, self.font, self.scale);
        let shown = if self.reveal {
            crate::text_reveal::display(
                &text,
                text.clone(),
                0,
                crate::text_reveal::Cadence::Row,
                window,
                cx,
            )
        } else {
            text
        };
        let width = w.ceil() + 8. * self.scale;
        let mut s = Surface::new(
            self.wrap_width.map_or(width, |max| max.min(width)),
            h.ceil(),
        );
        let shown_height = Theme::bundled().measure(&shown, self.font, self.scale)[1];
        s.text(
            &shown,
            self.font,
            self.scale,
            0.,
            shown_height / 2.,
            self.tint,
        );
        element(s)
    }
}

#[derive(IntoElement)]
pub struct MenuTitle {
    text: SharedString,
    scale: f32,
}
impl MenuTitle {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            scale: 1.,
        }
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}
impl RenderOnce for MenuTitle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use gpui::{div, prelude::*, px};
        let [width, height] = Theme::bundled().measure_spaced(&self.text, 2, 10.);
        let shown = crate::text_reveal::display(
            &self.text,
            "title-letters",
            0,
            crate::text_reveal::Cadence::Title,
            window,
            cx,
        );
        let mut surface = Surface::new(width.ceil() + 15., height.ceil() + 7.);
        for (offset, tint, alpha) in [(6., [89, 87, 78], 100), (0., crate::INK, 255)] {
            surface.lettering(
                &shown,
                2,
                1.,
                10.,
                [offset, height / 2. + offset],
                tint,
                alpha,
            );
        }
        div()
            .w(px(surface.width * self.scale))
            .h(px(surface.height * self.scale))
            .child(surface)
    }
}
