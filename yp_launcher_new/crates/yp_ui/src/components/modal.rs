use gpui::{
    AnyElement, App, Context, ElementId, FontWeight, IntoElement, ManagedView, ParentElement,
    Pixels, RenderOnce, ScrollHandle, SharedString, Window, div, prelude::*, relative,
};

use super::button::{ButtonSize, IconButton};
use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LINE_HEIGHT_TIGHT};
use super::root::WindowExt;
use super::scroll::ScrollableElement;
use super::text::Text;
use super::tooltip::Tooltip;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{StyledExt, h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::{ButtonCommon, Clickable};

pub trait ModalView: ManagedView {
    fn on_before_dismiss(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        true
    }

    fn dismiss_on_scrim_click(&self) -> bool {
        true
    }

    fn scrim_opacity(&self, _cx: &App) -> Option<f32> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ModalWidth {
    Small,
    #[default]
    Default,
    Large,
    Full,
    Custom(Pixels),
}

impl ModalWidth {
    pub fn pixels(self, sizes: &Sizes) -> Pixels {
        let available = sizes.window_width - Sizes::LIGHTBOX_INSET * 2.;
        let wanted = match self {
            ModalWidth::Small => sizes.dialog_width * 0.75,
            ModalWidth::Default => sizes.dialog_width,
            ModalWidth::Large => sizes.dialog_width * 1.6,
            ModalWidth::Full => available,
            ModalWidth::Custom(width) => width,
        };
        wanted
            .min(available)
            .max(Sizes::TOAST_MIN_WIDTH.min(available))
    }
}

#[derive(Default)]
pub struct ModalHeader {
    icon: Option<Icon>,
    headline: Option<SharedString>,
    headline_color: Color,
    description: Option<SharedString>,
    show_dismiss_button: bool,
    end_slot: Option<AnyElement>,
}

crate::impl_into_element!(ModalHeader);

impl ModalHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn headline(mut self, headline: impl Into<SharedString>) -> Self {
        self.headline = Some(headline.into());
        self
    }

    pub fn headline_color(mut self, color: Color) -> Self {
        self.headline_color = color;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn show_dismiss_button(mut self, show: bool) -> Self {
        self.show_dismiss_button = show;
        self
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.end_slot = Some(slot.into_any_element());
        self
    }
}

impl RenderOnce for ModalHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let close_tooltip = cx.t().tooltip_close();
        h_flex()
            .w_full()
            .items_start()
            .gap(sizes.spacing_md)
            .when_some(self.icon, |this, icon| {
                this.child(div().pt(sizes.spacing_xs).child(icon))
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(sizes.spacing_sm)
                    .when_some(self.headline, |this, headline| {
                        this.child(
                            v_flex()
                                .text_size(sizes.font_lg)
                                .line_height(relative(LINE_HEIGHT_TIGHT))
                                .font_weight(FontWeight::BOLD)
                                .text_color(self.headline_color.color(cx))
                                .child(Text::new(headline)),
                        )
                    })
                    .when_some(self.description, |this, description| {
                        this.child(
                            v_flex()
                                .text_size(sizes.font_sm)
                                .line_height(relative(LINE_HEIGHT_BODY))
                                .text_color(theme.colors.text_secondary)
                                .child(Text::new(description)),
                        )
                    }),
            )
            .when_some(self.end_slot, |this, slot| {
                this.child(div().flex_none().child(slot))
            })
            .when(self.show_dismiss_button, |this| {
                this.child(
                    IconButton::new("yp-modal-dismiss", IconName::Close)
                        .icon_size(IconSize::Small)
                        .icon_color(Color::Muted)
                        .size(ButtonSize::Compact)
                        .tooltip(Tooltip::text(close_tooltip))
                        .on_click(|_, window, cx| {
                            window.close_modal(cx);
                        }),
                )
            })
    }
}

#[derive(Default)]
pub struct ModalFooter {
    start_slot: Option<AnyElement>,
    end_slot: Option<AnyElement>,
}

crate::impl_into_element!(ModalFooter);

impl ModalFooter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_slot(mut self, slot: impl IntoElement) -> Self {
        self.start_slot = Some(slot.into_any_element());
        self
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.end_slot = Some(slot.into_any_element());
        self
    }
}

impl RenderOnce for ModalFooter {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        h_flex()
            .w_full()
            .flex_wrap()
            .gap(sizes.spacing_sm)
            .pt(sizes.spacing_md)
            .child(
                h_flex()
                    .flex_1()
                    .gap(sizes.spacing_sm)
                    .children(self.start_slot),
            )
            .child(
                h_flex()
                    .min_w_0()
                    .max_w_full()
                    .flex_wrap()
                    .justify_end()
                    .gap(sizes.spacing_sm)
                    .children(self.end_slot),
            )
    }
}

pub struct Modal {
    id: ElementId,
    scroll_handle: Option<ScrollHandle>,
    width: ModalWidth,
    header: Option<ModalHeader>,
    footer: Option<ModalFooter>,
    padded: bool,
    fill_height: bool,
    sections: Vec<AnyElement>,
}

crate::impl_into_element!(Modal);

impl Modal {
    pub fn new(id: impl Into<ElementId>, scroll_handle: Option<ScrollHandle>) -> Self {
        Self {
            id: id.into(),
            scroll_handle,
            width: ModalWidth::default(),
            header: None,
            footer: None,
            padded: true,
            fill_height: false,
            sections: Vec::new(),
        }
    }

    pub fn width(mut self, width: ModalWidth) -> Self {
        self.width = width;
        self
    }

    pub fn header(mut self, header: ModalHeader) -> Self {
        self.header = Some(header);
        self
    }

    pub fn footer(mut self, footer: ModalFooter) -> Self {
        self.footer = Some(footer);
        self
    }

    pub fn section(mut self, section: impl IntoElement) -> Self {
        self.sections.push(section.into_any_element());
        self
    }

    pub fn padded(mut self, padded: bool) -> Self {
        self.padded = padded;
        self
    }

    pub fn fill_height(mut self) -> Self {
        self.fill_height = true;
        self
    }
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.sections.extend(elements);
    }
}

impl RenderOnce for Modal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let width = self
            .width
            .pixels(&sizes)
            .min((sizes.window_width - Sizes::WINDOW_MARGIN * 2.).max(gpui::px(1.)));
        let max_height =
            (sizes.window_height - Sizes::TITLE_BAR_HEIGHT - Sizes::LIGHTBOX_INSET * 2.)
                .max(Sizes::SELECT_MENU_MIN_HEIGHT);
        let scroll_handle = match self.scroll_handle {
            Some(handle) => handle,
            None => window
                .use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| ScrollHandle::new())
                .read(cx)
                .clone(),
        };
        v_flex()
            .id(self.id)
            .role(gpui::Role::Dialog)
            .when_some(
                self.header.as_ref().and_then(|h| h.headline.clone()),
                |el, title| el.aria_label(title),
            )
            .w(width)
            .max_h(max_height)
            .when(self.fill_height, |this| this.h(max_height))
            .gap(sizes.spacing_md)
            .when(self.padded, |this| this.p(sizes.padding_lg))
            .rounded(sizes.border_radius)
            .bg(theme.colors.background_card)
            .border_1()
            .border_color(theme.colors.border_light)
            .elevation_3(cx)
            .children(self.header)
            .child(
                div()
                    .id("yp-modal-body")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(sizes.spacing_md)
                    .text_size(sizes.font_sm)
                    .line_height(relative(1.4))
                    .text_color(theme.colors.text_secondary)
                    .children(self.sections)
                    .overflow_y_scrollbar(&scroll_handle),
            )
            .children(self.footer)
    }
}
