use gpui::{
    Animation, AnimationExt, App, ElementId, IntoElement, RenderOnce, SharedString, Transformation,
    Window, div, percentage, prelude::*, svg,
};

use super::icon::{IconName, IconSize};
use super::label::{Label, LabelSize};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};

#[derive(Clone)]
pub struct Spinner {
    size: IconSize,
    color: Color,
}

crate::impl_into_element!(Spinner);

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: IconSize::Medium,
            color: Color::Accent,
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
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = self.size.pixels(window, cx);
        let duration = cx.theme().motion.spin;
        svg()
            .path(IconName::ProgressRing.path())
            .size(size)
            .flex_none()
            .text_color(self.color.color(cx))
            .with_animation(
                "yp-spinner",
                Animation::new(duration).repeat(),
                |this, delta| this.with_transformation(Transformation::rotate(percentage(delta))),
            )
    }
}

#[derive(Clone)]
pub struct ProgressBar {
    id: ElementId,
    value: f32,
    max: f32,
    color: Color,
    indeterminate: bool,
}

crate::impl_into_element!(ProgressBar);

impl ProgressBar {
    pub fn new(id: impl Into<ElementId>, value: f32, max: f32) -> Self {
        Self {
            id: id.into(),
            value,
            max,
            color: Color::Accent,
            indeterminate: false,
        }
    }

    pub fn indeterminate(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: 0.,
            max: 1.,
            color: Color::Accent,
            indeterminate: true,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= 0. || !self.max.is_finite() || !self.value.is_finite() {
            return 0.;
        }
        (self.value / self.max).clamp(0., 1.)
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let fraction = self.fraction();
        let color = self.color.color(cx);
        let track = div()
            .id(self.id.clone())
            .w_full()
            .h(Sizes::SLIDER_TRACK_HEIGHT)
            .rounded(Sizes::SLIDER_TRACK_HEIGHT)
            .bg(theme.colors.border_light)
            .overflow_hidden();
        if self.indeterminate {
            track.child(
                div()
                    .h_full()
                    .w(gpui::relative(0.3))
                    .rounded(Sizes::SLIDER_TRACK_HEIGHT)
                    .bg(color)
                    .with_animation(
                        self.id,
                        Animation::new(theme.motion.pulse * 2).repeat(),
                        |this, delta| this.ml(gpui::relative(delta * 1.3 - 0.3)),
                    ),
            )
        } else {
            track.child(
                div()
                    .h_full()
                    .w(gpui::relative(fraction))
                    .rounded(Sizes::SLIDER_TRACK_HEIGHT)
                    .bg(color),
            )
        }
    }
}

#[derive(Clone)]
pub struct BusyOverlay {
    message: SharedString,
}

crate::impl_into_element!(BusyOverlay);

impl BusyOverlay {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl RenderOnce for BusyOverlay {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        div()
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.colors.scrim.opacity(theme.alpha.overlay))
            .child(
                h_flex()
                    .gap(sizes.spacing_lg)
                    .px(sizes.padding_xl)
                    .py(sizes.padding_lg)
                    .max_w(gpui::relative(0.8))
                    .rounded(sizes.border_radius)
                    .bg(theme.colors.background_card)
                    .border_1()
                    .border_color(theme.colors.border_light)
                    .elevation_3(cx)
                    .child(Spinner::new().size(IconSize::Large))
                    .child(Label::new(self.message)),
            )
    }
}

#[derive(Clone, Default)]
pub struct LoadingState {
    message: Option<SharedString>,
    compact: bool,
}

crate::impl_into_element!(LoadingState);

impl LoadingState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn message(mut self, message: impl Into<SharedString>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
}

impl RenderOnce for LoadingState {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let message = self
            .message
            .unwrap_or_else(|| SharedString::new_static(cx.t().ui_loading()));
        if self.compact {
            return h_flex()
                .gap(sizes.spacing_md)
                .child(Spinner::new().size(IconSize::Small))
                .child(
                    Label::new(message)
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                )
                .into_any_element();
        }
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap(sizes.spacing_lg)
            .p(sizes.padding_xl)
            .child(Spinner::new().size(IconSize::Custom(sizes.play_button_loading_size)))
            .child(
                Label::new(message)
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_fraction_is_clamped_and_never_nan() {
        assert_eq!(ProgressBar::new("p", 5., 10.).fraction(), 0.5);
        assert_eq!(ProgressBar::new("p", 50., 10.).fraction(), 1.);
        assert_eq!(ProgressBar::new("p", -3., 10.).fraction(), 0.);
        assert_eq!(ProgressBar::new("p", 3., 0.).fraction(), 0.);
        assert_eq!(ProgressBar::new("p", f32::NAN, 10.).fraction(), 0.);
    }
}
