use std::path::PathBuf;

use gpui::{Global, ScrollHandle};
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    Callout, Card, CardPadding, CollapsibleCard, Divider, Icon, IconName, IconSize, Label,
    LabelSize, LoadingState, ScrollableElement,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CardDefaults {
    pub expanded: bool,
}

impl Global for CardDefaults {}

pub fn set_cards_expanded(expanded: bool, cx: &mut App) {
    cx.set_global(CardDefaults { expanded });
    cx.refresh_windows();
}

pub fn card(id: &'static str, title: impl Into<SharedString>, cx: &App) -> CollapsibleCard {
    let expanded = cx
        .try_global::<CardDefaults>()
        .is_some_and(|defaults| defaults.expanded);
    CollapsibleCard::new(id, title).default_open(expanded)
}

pub fn reveal(path: PathBuf, is_file: bool, cx: &mut App) {
    if yp_ui::ControllerMode::is_active(cx) {
        return;
    }
    let platform = cx.env().platform.clone();
    cx.background_spawn(async move {
        if let Err(error) = platform.reveal(&path, is_file) {
            log::warn!("could not reveal {}: {error}", path.display());
        }
    })
    .detach();
}

pub fn tab_page(
    id: &'static str,
    header: impl IntoElement,
    scroll: &ScrollHandle,
    loading: bool,
    content: impl IntoElement,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Div {
    let sizes = window.sizes(cx);
    let background = cx.theme().colors.background_primary;
    let body = if loading {
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .items_center()
            .justify_center()
            .child(LoadingState::new())
            .into_any_element()
    } else {
        div()
            .id(id)
            .flex_1()
            .min_h_0()
            .w_full()
            .p(sizes.content_padding)
            .child(content)
            .overflow_y_scrollbar(scroll)
            .into_any_element()
    };
    v_flex()
        .relative()
        .size_full()
        .bg(background)
        .child(header)
        .child(body)
}

pub fn column(children: impl IntoIterator<Item = AnyElement>, window: &Window, cx: &App) -> gpui::Div {
    let sizes = window.sizes(cx);
    v_flex().w_full().gap(sizes.spacing_lg).children(children)
}

pub fn banner(
    icon: IconName,
    text: impl Into<SharedString>,
    window: &Window,
    cx: &App,
) -> Card {
    let sizes = window.sizes(cx);
    Card::new().padding(CardPadding::Compact).child(
        h_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(Icon::new(icon).size(IconSize::Large).color(Color::Secondary))
            .child(
                v_flex().flex_1().min_w_0().child(
                    Label::new(text)
                        .size(LabelSize::XSmall)
                        .color(Color::Secondary)
                        .line_height(1.4),
                ),
            ),
    )
}

pub fn failure_banner(id: &'static str, text: impl Into<SharedString>) -> Callout {
    Callout::new()
        .id(id)
        .severity(Severity::Error)
        .description(text)
}

pub fn subheading(text: impl Into<SharedString>, window: &Window, cx: &App) -> gpui::Div {
    let sizes = window.sizes(cx);
    div().pt(sizes.spacing_md).pb(sizes.spacing_sm).child(
        Label::new(text)
            .size(LabelSize::Small)
            .color(Color::Secondary)
            .bold(),
    )
}

pub fn note(
    text: impl Into<SharedString>,
    icon: IconName,
    window: &Window,
    cx: &App,
) -> gpui::Div {
    let sizes = window.sizes(cx);
    h_flex()
        .w_full()
        .items_start()
        .gap(sizes.spacing_md)
        .px(sizes.padding_sm)
        .py(sizes.spacing_xs)
        .child(
            div().pt(sizes.spacing_xs).child(
                Icon::new(icon)
                    .size(IconSize::XSmall)
                    .color(Color::Muted),
            ),
        )
        .child(
            div().flex_1().min_w_0().child(
                Label::new(text)
                    .size(LabelSize::XSmall)
                    .color(Color::Muted)
                    .line_height(1.3),
            ),
        )
}

pub fn muted_text(text: impl Into<SharedString>, italic: bool, window: &Window, cx: &App) -> gpui::Div {
    let sizes = window.sizes(cx);
    let label = Label::new(text)
        .size(LabelSize::XSmall)
        .color(Color::Muted)
        .line_height(1.4);
    div()
        .w_full()
        .pb(sizes.spacing_md)
        .child(if italic { label.italic() } else { label })
}

pub fn row_divider(window: &Window, cx: &App) -> gpui::Div {
    let sizes = window.sizes(cx);
    div()
        .w_full()
        .py(sizes.padding_xs)
        .child(Divider::horizontal())
}

pub fn locked(
    enabled: bool,
    reason: impl Into<SharedString>,
    content: impl IntoElement,
    window: &Window,
    cx: &App,
) -> AnyElement {
    if enabled {
        return content.into_any_element();
    }
    let sizes = window.sizes(cx);
    v_flex()
        .w_full()
        .child(div().w_full().child(content))
        .child(
            h_flex()
                .gap(sizes.spacing_md)
                .pl(sizes.padding_sm)
                .pb(sizes.padding_xs)
                .child(
                    Icon::new(IconName::LockOutline)
                        .size(IconSize::XSmall)
                        .color(Color::Muted),
                )
                .child(
                    div().flex_1().min_w_0().child(
                        Label::new(reason)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .italic(),
                    ),
                ),
        )
        .into_any_element()
}
