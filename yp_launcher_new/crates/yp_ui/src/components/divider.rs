use gpui::{App, IntoElement, RenderOnce, Window, div, prelude::*};

use crate::theme::{ActiveTheme, ResponsiveExt, Sizes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DividerColor {
    #[default]
    Light,
    Medium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DividerDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct Divider {
    direction: DividerDirection,
    color: DividerColor,
    inset: bool,
}

crate::impl_into_element!(Divider);

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            direction: DividerDirection::Horizontal,
            color: DividerColor::default(),
            inset: false,
        }
    }

    pub fn vertical() -> Self {
        Self {
            direction: DividerDirection::Vertical,
            color: DividerColor::default(),
            inset: false,
        }
    }

    pub fn color(mut self, color: DividerColor) -> Self {
        self.color = color;
        self
    }

    pub fn inset(mut self) -> Self {
        self.inset = true;
        self
    }
}

impl RenderOnce for Divider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let sizes = window.sizes(cx);
        let color = match self.color {
            DividerColor::Light => colors.border_light,
            DividerColor::Medium => colors.border_medium,
        };
        match self.direction {
            DividerDirection::Horizontal => div()
                .h(Sizes::BORDER_WIDTH)
                .w_full()
                .flex_none()
                .when(self.inset, |this| this.mx(sizes.card_padding_h))
                .bg(color),
            DividerDirection::Vertical => div()
                .w(Sizes::BORDER_WIDTH)
                .h_full()
                .flex_none()
                .when(self.inset, |this| this.my(sizes.spacing_sm))
                .bg(color),
        }
    }
}
