use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Context, DismissEvent, ElementId, EventEmitter, FocusHandle,
    Focusable, FontWeight, ImageSource, IntoElement, ObjectFit, Pixels, Render, RenderOnce,
    SharedString, Window, div, img, prelude::*, px, relative,
};

use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_TIGHT, Label, LabelSize};
use super::modal::ModalView;
use super::progress::Spinner;
use super::root::WindowExt;
use super::slider::Slider;
use super::tooltip::Tooltip;
use super::util::KeyboardFocus;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::ClickHandler;

fn image_fallback(icon: IconName, size: Pixels, cx: &App) -> impl Fn() -> AnyElement + 'static {
    let color = cx.theme().colors.text_muted;
    let background = cx.theme().colors.surface_medium;
    move || {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(background)
            .child(
                gpui::svg()
                    .path(icon.path())
                    .size(size)
                    .flex_none()
                    .text_color(color),
            )
            .into_any_element()
    }
}

fn image_loading() -> impl Fn() -> AnyElement + 'static {
    || {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(Spinner::new().size(IconSize::Small))
            .into_any_element()
    }
}

pub struct Thumbnail {
    id: ElementId,
    source: ImageSource,
    fit: ObjectFit,
    rounded: bool,
    fallback: IconName,
    width: Option<Pixels>,
    height: Option<Pixels>,
    lightbox: bool,
    on_click: Option<ClickHandler>,
    tooltip: Option<SharedString>,
}

crate::impl_into_element!(Thumbnail);

impl Thumbnail {
    pub fn new(id: impl Into<ElementId>, source: impl Into<ImageSource>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            fit: ObjectFit::Cover,
            rounded: true,
            fallback: IconName::BrokenImageOutlined,
            width: None,
            height: None,
            lightbox: false,
            on_click: None,
            tooltip: None,
        }
    }

    pub fn fit(mut self, fit: ObjectFit) -> Self {
        self.fit = fit;
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn fallback(mut self, icon: IconName) -> Self {
        self.fallback = icon;
        self
    }

    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn lightbox(mut self, lightbox: bool) -> Self {
        self.lightbox = lightbox;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

impl RenderOnce for Thumbnail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let width = self.width.unwrap_or(Sizes::PREVIEW_THUMB_WIDTH);
        let height = self.height.unwrap_or(Sizes::PREVIEW_ROW_HEIGHT);
        let clickable = self.lightbox || self.on_click.is_some();
        let hover_border = theme.colors.accent_primary.opacity(theme.alpha.half);
        let lightbox_source = self.lightbox.then(|| self.source.clone());
        let on_click = self.on_click;
        let fallback_size = (height * 0.5).min(sizes.icon_lg * 2.);
        let border = Sizes::BORDER_WIDTH * 2.;
        let image_width = (width - border).max(px(0.));
        let image_height = (height - border).max(px(0.));
        div()
            .id(self.id)
            .flex()
            .flex_none()
            .w(width)
            .h(height)
            .overflow_hidden()
            .border_1()
            .border_color(theme.colors.border_light)
            .bg(theme.colors.surface_medium)
            .when(self.rounded, |this| this.rounded(Sizes::ROW_RADIUS))
            .when(clickable, |this| {
                this.cursor_pointer()
                    .hover(move |style| style.border_color(hover_border))
                    .on_click(move |event, window, cx| {
                        cx.stop_propagation();
                        if let Some(source) = lightbox_source.clone() {
                            Lightbox::open(source, window, cx);
                        }
                        if let Some(handler) = &on_click {
                            handler(event, window, cx);
                        }
                    })
            })
            .when_some(self.tooltip, |this, tooltip| {
                this.tooltip(Tooltip::text(tooltip))
            })
            .child(
                img(self.source)
                    .flex_none()
                    .w(image_width)
                    .h(image_height)
                    .object_fit(self.fit)
                    .with_loading(image_loading())
                    .with_fallback(image_fallback(self.fallback, fallback_size, cx)),
            )
    }
}

pub struct Lightbox {
    source: ImageSource,
    focus_handle: FocusHandle,
}

impl Lightbox {
    pub fn open(source: impl Into<ImageSource>, window: &mut Window, cx: &mut App) {
        let source = source.into();
        window.open_modal(cx, |_, cx| Lightbox {
            source,
            focus_handle: cx.focus_handle(),
        });
    }
}

impl EventEmitter<DismissEvent> for Lightbox {}

impl Focusable for Lightbox {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for Lightbox {
    fn scrim_opacity(&self, cx: &App) -> Option<f32> {
        Some(cx.theme().alpha.heavy)
    }
}

impl Render for Lightbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let inset = Sizes::LIGHTBOX_INSET;
        let width = (sizes.window_width - inset * 2.).max(px(1.));
        let height = (sizes.window_height - Sizes::TITLE_BAR_HEIGHT - inset * 2.).max(px(1.));
        let tooltip = cx.t().ui_click_to_close();
        div()
            .id("yp-lightbox")
            .track_focus(&self.focus_handle)
            .flex()
            .w(width)
            .h(height)
            .cursor_pointer()
            .on_click(cx.listener(|_, _, _, cx| cx.emit(DismissEvent)))
            .tooltip(Tooltip::text(tooltip))
            .child(
                img(self.source.clone())
                    .flex_none()
                    .w(width)
                    .h(height)
                    .object_fit(ObjectFit::Contain)
                    .with_loading(image_loading())
                    .with_fallback(image_fallback(IconName::ImageNotSupported, px(48.), cx)),
            )
    }
}

pub struct PreviewImage {
    id: ElementId,
    image: ImageSource,
    label: SharedString,
}

crate::impl_into_element!(PreviewImage);

impl PreviewImage {
    pub fn new(
        id: impl Into<ElementId>,
        image: impl Into<ImageSource>,
        label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            image: image.into(),
            label: label.into(),
        }
    }
}

fn preview_row(id: ElementId, cx: &App) -> gpui::Stateful<gpui::Div> {
    let theme = cx.theme();
    let accent = theme.colors.accent_primary;
    let hover_background = accent.opacity(theme.alpha.faint);
    let hover_border = accent.opacity(theme.alpha.half);
    h_flex()
        .id(id)
        .w_full()
        .h(Sizes::PREVIEW_ROW_HEIGHT)
        .overflow_hidden()
        .rounded(Sizes::ROW_RADIUS)
        .border_1()
        .border_color(theme.colors.border_light)
        .bg(theme.colors.surface_light)
        .cursor_pointer()
        .keyboard_focus(cx)
        .hover(move |style| style.bg(hover_background).border_color(hover_border))
}

fn preview_thumb(source: ImageSource, cx: &App) -> impl IntoElement {
    let height = Sizes::PREVIEW_ROW_HEIGHT - Sizes::BORDER_WIDTH * 2.;
    div()
        .flex()
        .flex_none()
        .w(Sizes::PREVIEW_THUMB_WIDTH)
        .h(height)
        .overflow_hidden()
        .child(
            img(source)
                .flex_none()
                .w(Sizes::PREVIEW_THUMB_WIDTH)
                .h(height)
                .object_fit(ObjectFit::Cover)
                .with_fallback(image_fallback(IconName::BrokenImageOutlined, px(16.), cx)),
        )
}

impl RenderOnce for PreviewImage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let group = SharedString::from(format!("yp-preview-{}", self.id));
        let source = self.image.clone();
        div().w_full().pt(px(4.)).pb(px(8.)).child(
            preview_row(self.id, cx)
                .group(group.clone())
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    Lightbox::open(source.clone(), window, cx);
                })
                .child(preview_thumb(self.image, cx))
                .child(
                    div().flex_1().min_w_0().flex().justify_center().child(
                        Label::new(self.label)
                            .size(LabelSize::XSmall)
                            .color(Color::Secondary)
                            .bold()
                            .truncate(),
                    ),
                )
                .child(
                    div().flex_none().pr(px(10.)).child(
                        Icon::new(IconName::Fullscreen)
                            .size(IconSize::Custom(px(16.)))
                            .color(Color::Muted)
                            .group_hover_color(group, Color::Accent),
                    ),
                ),
        )
    }
}

pub struct ImageComparison {
    id: ElementId,
    before: ImageSource,
    after: ImageSource,
    before_label: SharedString,
    after_label: SharedString,
}

crate::impl_into_element!(ImageComparison);

impl ImageComparison {
    pub fn new(
        id: impl Into<ElementId>,
        before: impl Into<ImageSource>,
        after: impl Into<ImageSource>,
        before_label: impl Into<SharedString>,
        after_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            before: before.into(),
            after: after.into(),
            before_label: before_label.into(),
            after_label: after_label.into(),
        }
    }
}

impl RenderOnce for ImageComparison {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let group = SharedString::from(format!("yp-comparison-{}", self.id));
        let before = self.before.clone();
        let after = self.after.clone();
        let before_label = self.before_label.clone();
        let after_label = self.after_label.clone();
        div().w_full().pt(px(4.)).pb(px(8.)).child(
            preview_row(self.id, cx)
                .group(group.clone())
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    let before = before.clone();
                    let after = after.clone();
                    let before_label = before_label.clone();
                    let after_label = after_label.clone();
                    window.open_modal(cx, |_, cx| ComparisonDialog {
                        before,
                        after,
                        before_label,
                        after_label,
                        blend: 0.,
                        focus_handle: cx.focus_handle(),
                    });
                })
                .child(preview_thumb(self.before, cx))
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .justify_center()
                        .gap(px(8.))
                        .px(px(10.))
                        .child(
                            Label::new(self.before_label)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .bold()
                                .truncate(),
                        )
                        .child(
                            Icon::new(IconName::CompareArrows)
                                .size(IconSize::Custom(px(16.)))
                                .color(Color::Muted)
                                .group_hover_color(group, Color::Accent),
                        )
                        .child(
                            Label::new(self.after_label)
                                .size(LabelSize::XSmall)
                                .color(Color::Secondary)
                                .bold()
                                .truncate(),
                        ),
                )
                .child(preview_thumb(self.after, cx)),
        )
    }
}

struct ComparisonDialog {
    before: ImageSource,
    after: ImageSource,
    before_label: SharedString,
    after_label: SharedString,
    blend: f32,
    focus_handle: FocusHandle,
}

impl EventEmitter<DismissEvent> for ComparisonDialog {}

impl Focusable for ComparisonDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for ComparisonDialog {
    fn scrim_opacity(&self, cx: &App) -> Option<f32> {
        Some(cx.theme().alpha.heavy)
    }
}

impl Render for ComparisonDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let inset = Sizes::LIGHTBOX_INSET;
        let controls_height = px(72.);
        let max_width = (sizes.window_width - inset * 2.).max(px(1.));
        let max_height =
            (sizes.window_height - Sizes::TITLE_BAR_HEIGHT - inset * 2. - controls_height)
                .max(px(1.));
        let width = max_width.min(max_height * (16. / 9.));
        let height = width * (9. / 16.);
        let blend = self.blend;
        let label = |text: SharedString, active: bool| {
            v_flex()
                .flex_none()
                .text_size(sizes.font_md)
                .line_height(relative(LINE_HEIGHT_TIGHT))
                .font_weight(FontWeight::BOLD)
                .text_color(if active {
                    theme.colors.accent_primary
                } else {
                    theme.colors.text_muted
                })
                .child(super::text::Text::new(text))
        };
        v_flex()
            .track_focus(&self.focus_handle)
            .items_center()
            .gap(px(12.))
            .child(
                div()
                    .relative()
                    .flex()
                    .w(width)
                    .h(height)
                    .overflow_hidden()
                    .rounded(px(8.))
                    .bg(theme.colors.surface_medium)
                    .child(
                        img(self.before.clone())
                            .flex_none()
                            .w(width)
                            .h(height)
                            .object_fit(ObjectFit::Cover)
                            .with_loading(image_loading())
                            .with_fallback(image_fallback(
                                IconName::ImageNotSupported,
                                px(48.),
                                cx,
                            )),
                    )
                    .child(
                        div().absolute().inset_0().flex().opacity(blend).child(
                            img(self.after.clone())
                                .flex_none()
                                .w(width)
                                .h(height)
                                .object_fit(ObjectFit::Cover),
                        ),
                    ),
            )
            .child(
                h_flex()
                    .gap(px(12.))
                    .px(px(16.))
                    .py(px(10.))
                    .rounded(px(8.))
                    .bg(theme.colors.background_card)
                    .elevation_3(cx)
                    .child(label(self.before_label.clone(), blend <= 0.3))
                    .child(
                        Slider::new("yp-comparison-blend")
                            .value(blend)
                            .min(0.)
                            .max(1.)
                            .width(px(300.).min(max_width * 0.5))
                            .on_change(cx.listener(|this, value: &f32, _, cx| {
                                this.blend = *value;
                                cx.notify();
                            })),
                    )
                    .child(label(self.after_label.clone(), blend >= 0.7)),
            )
    }
}
