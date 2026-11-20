use gpui::{App, FontWeight, IntoElement, RenderOnce, SharedString, Window, prelude::*};

use crate::styled_ext::{StyledExt, h_flex};
use crate::theme::{ActiveTheme, ResponsiveExt, Sizes};

#[derive(Clone)]
pub struct Kbd {
    text: SharedString,
    active: bool,
}

crate::impl_into_element!(Kbd);

impl Kbd {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            active: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl RenderOnce for Kbd {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let sizes = window.sizes(cx);
        let border = if self.active {
            theme.colors.accent_primary
        } else {
            theme.colors.border_medium
        };
        let color = if self.active {
            theme.colors.accent_primary
        } else {
            theme.colors.text_secondary
        };
        h_flex()
            .flex_none()
            .justify_center()
            .px(sizes.chip_padding_h)
            .py(sizes.chip_padding_v)
            .rounded(Sizes::CONTROL_RADIUS)
            .border_1()
            .border_color(border)
            .bg(theme.colors.input_background)
            .font_mono(cx)
            .font_weight(FontWeight::SEMIBOLD)
            .text_size(sizes.font_xs)
            .text_color(color)
            .child(super::text::Text::new(self.text))
    }
}
