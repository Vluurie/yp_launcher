use gpui::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, RenderOnce, SharedString, Window,
    prelude::*, relative,
};

use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LabelSize};
use super::text::Text;
use super::tooltip::Tooltip;
use super::util::{ControllerMode, KeyboardFocus};
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt};
use crate::traits::ClickHandler;

pub struct Link {
    id: ElementId,
    label: SharedString,
    href: Option<SharedString>,
    on_click: Option<ClickHandler>,
    size: LabelSize,
    color: Color,
    hover_color: Color,
    icon: Option<IconName>,
    end_icon: Option<IconName>,
    weight: Option<FontWeight>,
    tooltip: Option<SharedString>,
    underline: bool,
    truncate: bool,
    controller_supported: bool,
}

crate::impl_into_element!(Link);

impl Link {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            href: None,
            on_click: None,
            size: LabelSize::Small,
            color: Color::Muted,
            hover_color: Color::Accent,
            icon: None,
            end_icon: None,
            weight: None,
            tooltip: None,
            underline: false,
            truncate: false,
            controller_supported: true,
        }
    }

    pub fn href(mut self, href: impl Into<SharedString>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn controller_support(mut self, supported: bool) -> Self {
        self.controller_supported = supported;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn end_icon(mut self, icon: IconName) -> Self {
        self.end_icon = Some(icon);
        self
    }

    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }
}

impl RenderOnce for Link {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let color = self.color.color(cx);
        let hover = self.hover_color.color(cx);
        let group = SharedString::from(format!("link-{}", self.id));
        let href = self.href;
        let on_click = self.on_click;
        let controller_blocked = ControllerMode::is_active(cx) && !self.controller_supported;
        h_flex()
            .id(self.id)
            .role(gpui::Role::Link)
            .aria_label(self.label.clone())
            .group(group.clone())
            .gap(sizes.spacing_sm)
            .when(self.truncate, |this| this.min_w_0().max_w_full())
            .when(controller_blocked, |this| {
                this.opacity(cx.theme().alpha.half).cursor_not_allowed()
            })
            .when(!controller_blocked, |this| {
                this.cursor_pointer().keyboard_focus(cx)
            })
            .text_size(self.size.pixels(&sizes))
            .line_height(relative(LINE_HEIGHT_BODY))
            .text_color(color)
            .when_some(self.weight, |this, weight| this.font_weight(weight))
            .when(self.underline, |this| {
                this.underline().text_decoration_color(color)
            })
            .when(!controller_blocked, |this| {
                this.hover(move |style| {
                    style
                        .text_color(hover)
                        .underline()
                        .text_decoration_color(hover)
                })
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::Small)
                        .color(self.color)
                        .group_hover_color(group.clone(), self.hover_color),
                )
            })
            .child(
                v_flex()
                    .min_w_0()
                    .whitespace_nowrap()
                    .when(self.truncate, |this| this.truncate())
                    .child(Text::new(self.label)),
            )
            .when_some(self.end_icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::XSmall)
                        .color(self.color)
                        .group_hover_color(group.clone(), self.hover_color),
                )
            })
            .when_some(
                self.tooltip.filter(|_| !controller_blocked),
                |this, tooltip| this.tooltip(Tooltip::text(tooltip)),
            )
            .when(controller_blocked, |this| {
                this.tooltip(Tooltip::text(cx.t().controller_requires_keyboard()))
            })
            .when(!controller_blocked, |this| {
                this.on_click(move |event, window, cx| {
                    cx.stop_propagation();
                    if let Some(href) = &href {
                        super::util::open_link(href, cx);
                    }
                    if let Some(handler) = &on_click {
                        handler(event, window, cx);
                    }
                })
            })
    }
}
