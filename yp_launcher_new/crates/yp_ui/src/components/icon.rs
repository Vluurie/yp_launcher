use gpui::{
    App, IntoElement, Pixels, Radians, RenderOnce, SharedString, Transformation, Window,
    prelude::*, px, svg,
};

pub use super::icon_name::IconName;
use crate::theme::{Color, ResponsiveExt};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum IconSize {
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
    Custom(Pixels),
}

impl IconSize {
    pub const XSMALL: Pixels = px(12.);

    pub fn pixels(self, window: &Window, cx: &App) -> Pixels {
        let sizes = window.sizes(cx);
        match self {
            IconSize::XSmall => Self::XSMALL,
            IconSize::Small => sizes.icon_sm,
            IconSize::Medium => sizes.icon_md,
            IconSize::Large => sizes.icon_lg,
            IconSize::Custom(size) => size,
        }
    }
}

#[derive(Clone)]
pub struct Icon {
    name: IconName,
    size: IconSize,
    color: Color,
    alpha: Option<f32>,
    rotation: Option<Radians>,
    group_hover: Option<(SharedString, Color)>,
}

crate::impl_into_element!(Icon);

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: IconSize::default(),
            color: Color::Default,
            alpha: None,
            rotation: None,
            group_hover: None,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = Some(alpha);
        self
    }

    pub fn rotate(mut self, rotation: impl Into<Radians>) -> Self {
        self.rotation = Some(rotation.into());
        self
    }

    pub fn group_hover_color(mut self, group: impl Into<SharedString>, color: Color) -> Self {
        self.group_hover = Some((group.into(), color));
        self
    }
}

impl From<IconName> for Icon {
    fn from(name: IconName) -> Self {
        Icon::new(name)
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = self.size.pixels(window, cx);
        let color = match self.alpha {
            Some(alpha) => self.color.color(cx).opacity(alpha),
            None => self.color.color(cx),
        };
        let hover = self
            .group_hover
            .map(|(group, color)| (group, color.color(cx)));
        svg()
            .path(self.name.path())
            .size(size)
            .flex_none()
            .text_color(color)
            .when_some(self.rotation, |this, rotation| {
                this.with_transformation(Transformation::rotate(rotation))
            })
            .when_some(hover, |this, (group, color)| {
                this.group_hover(group, move |style| style.text_color(color))
            })
    }
}
