use gpui::{App, ElementId, IntoElement, SharedString, Window};
use yp_ui::prelude::*;
use yp_ui::{Card, CardPadding, IconSize, LabelSize, SelectableText, Sizes};

use crate::metrics;

pub fn section_card(
    icon: IconName,
    title: impl Into<SharedString>,
    color: Color,
    trailing: Option<gpui::AnyElement>,
    window: &Window,
    cx: &App,
) -> Card {
    let sizes = window.sizes(cx);
    let flagged = matches!(color, Color::Error | Color::Warning | Color::Success);
    let card = Card::new().padding(CardPadding::Compact);
    let card = if flagged { card.accent(color) } else { card };
    let card = card
        .header(
            h_flex()
                .gap(sizes.spacing_md)
                .child(Icon::new(icon).size(IconSize::Small).color(color))
                .child(
                    Label::new(title)
                        .size(LabelSize::XSmall)
                        .color(color)
                        .bold()
                        .truncate(),
                ),
        );
    match trailing {
        Some(trailing) => card.header_end(trailing),
        None => card,
    }
}

pub fn code_block(
    id: impl Into<ElementId>,
    text: impl Into<SharedString>,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme().clone();
    let sizes = window.sizes(cx);
    div()
        .w_full()
        .p(sizes.padding_sm)
        .rounded(Sizes::CONTROL_RADIUS)
        .bg(theme.colors.background_primary)
        .border_1()
        .border_color(theme.colors.border_light)
        .child(
            SelectableText::new(id, text)
                .mono()
                .size(LabelSize::XSmall)
                .color(Color::Secondary),
        )
}

pub fn header_tile(icon: IconName, color: Color, cx: &App) -> impl IntoElement {
    let theme = cx.theme().clone();
    let tint = color.color(cx);
    div()
        .flex_none()
        .size(metrics::HEADER_TILE_SIZE)
        .flex()
        .items_center()
        .justify_center()
        .rounded(Sizes::ROW_RADIUS)
        .bg(tint.opacity(theme.alpha.subtle))
        .border_1()
        .border_color(tint.opacity(theme.alpha.half))
        .child(Icon::new(icon).size(IconSize::Medium).color(color))
}

pub fn muted_line(text: impl Into<SharedString>) -> impl IntoElement {
    Label::new(text)
        .size(LabelSize::XSmall)
        .color(Color::Muted)
        .italic()
}

pub fn status_line(ok: bool, text: impl Into<SharedString>, window: &Window, cx: &App) -> impl IntoElement {
    let sizes = window.sizes(cx);
    let (icon, color) = if ok {
        (IconName::CheckCircle, Color::Success)
    } else {
        (IconName::ErrorOutline, Color::Warning)
    };
    h_flex()
        .w_full()
        .items_start()
        .gap(sizes.spacing_md)
        .child(Icon::new(icon).size(IconSize::Small).color(color))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .child(Label::new(text).size(LabelSize::Small).color(color)),
        )
}
