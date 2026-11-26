use gpui::{AnyElement, App, Window};
use yp_ui::prelude::*;
use yp_ui::{Headline, HeadlineSize};

fn content_width(window: &Window, cx: &App) -> f32 {
    let padding = window.sizes(cx).padding_xl;
    f32::from(
        (window.viewport_size().width - padding * 2.)
            .min(crate::metrics::ONBOARDING_CARD_MAX_WIDTH)
            - padding * 2.
            - px(32.),
    )
}

pub(super) fn heading(label: &'static str, window: &Window, cx: &App) -> AnyElement {
    if cx.theme().id == ThemeId::NierNew {
        let width = yp_ui::nier::Theme::bundled().measure(label, 2, 1.)[0]
            + label.chars().count() as f32 * 10.
            + 15.;
        let scale = (content_width(window, cx) / width).min(0.64);
        if scale >= 0.4 {
            return div()
                .id("onboarding-welcome-title")
                .role(gpui::Role::Heading)
                .aria_label(label)
                .child(yp_ui::nier::MenuTitle::new(label).scale(scale))
                .into_any_element();
        }
    }
    Headline::new(label)
        .size(HeadlineSize::Hero)
        .color(Color::Accent)
        .centered()
        .into_any_element()
}

pub(super) fn choice(
    id: &'static str,
    label: &'static str,
    selected: bool,
    window: &Window,
    cx: &App,
    click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    if cx.theme().id == ThemeId::NierNew {
        let width = ((content_width(window, cx) / 0.8) - 72.).min(620.);
        if yp_ui::nier::Theme::bundled().measure(label, 36, 0.8)[0] + 50. <= width {
            return yp_ui::nier::MenuRow::new(id, label)
                .scale(0.8)
                .width(width)
                .when(selected, |row| row.selected(true))
                .on_click(click)
                .into_any_element();
        }
    }
    yp_ui::ChoiceCard::new(id, label)
        .selected(selected)
        .compact()
        .on_click(click)
        .into_any_element()
}
