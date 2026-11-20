use std::rc::Rc;

use gpui::{
    AnyElement, AnyView, App, ClickEvent, ElementId, FontWeight, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, RenderOnce, SharedString, Window, div, prelude::*, px, relative,
};

use super::card::chevron;
use super::chip::CountBadge;
use super::divider::Divider;
use super::icon::IconSize;
use super::label::LINE_HEIGHT_TIGHT;
use super::text::Text;
use super::util::KeyboardFocus;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::{ClickHandler, TooltipBuilder, ValueHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ListItemSpacing {
    ExtraDense,
    #[default]
    Dense,
    Sparse,
}

type MouseDownHandler = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

pub struct ListItem {
    id: ElementId,
    start_slot: Option<AnyElement>,
    end_slot: Option<AnyElement>,
    end_slot_on_hover: Option<AnyElement>,
    selected: bool,
    accent: bool,
    disabled: bool,
    spacing: ListItemSpacing,
    indent_level: usize,
    rounded: bool,
    on_click: Option<ClickHandler>,
    on_hover: Option<ValueHandler<bool>>,
    on_secondary_mouse_down: Option<MouseDownHandler>,
    tooltip: Option<TooltipBuilder>,
    group_name: Option<SharedString>,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(ListItem);

impl ListItem {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            start_slot: None,
            end_slot: None,
            end_slot_on_hover: None,
            selected: false,
            accent: false,
            disabled: false,
            spacing: ListItemSpacing::default(),
            indent_level: 0,
            rounded: true,
            on_click: None,
            on_hover: None,
            on_secondary_mouse_down: None,
            tooltip: None,
            group_name: None,
            children: Vec::new(),
        }
    }

    pub fn start_slot(mut self, slot: impl IntoElement) -> Self {
        self.start_slot = Some(slot.into_any_element());
        self
    }

    pub fn end_slot(mut self, slot: impl IntoElement) -> Self {
        self.end_slot = Some(slot.into_any_element());
        self
    }

    pub fn end_slot_on_hover(mut self, slot: impl IntoElement) -> Self {
        self.end_slot_on_hover = Some(slot.into_any_element());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn accent(mut self, accent: bool) -> Self {
        self.accent = accent;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn spacing(mut self, spacing: ListItemSpacing) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn indent_level(mut self, level: usize) -> Self {
        self.indent_level = level;
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn on_hover(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Rc::new(handler));
        self
    }

    pub fn on_secondary_mouse_down(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_secondary_mouse_down = Some(Rc::new(handler));
        self
    }

    pub fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Rc::new(tooltip));
        self
    }

    pub fn group_name(mut self, group: impl Into<SharedString>) -> Self {
        self.group_name = Some(group.into());
        self
    }
}

impl ParentElement for ListItem {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let accent = theme.colors.accent_primary;
        let padding_v = match self.spacing {
            ListItemSpacing::ExtraDense => sizes.spacing_xs,
            ListItemSpacing::Dense => sizes.padding_sm,
            ListItemSpacing::Sparse => sizes.padding_md,
        };
        let group = self
            .group_name
            .unwrap_or_else(|| SharedString::from(format!("yp-list-item-{}", self.id)));
        let tinted = self.accent && !self.selected;
        let hover_background = if self.selected {
            accent.opacity(theme.alpha.hover)
        } else if tinted {
            accent.opacity(theme.alpha.subtle)
        } else {
            theme.colors.surface_light
        };
        let disabled = self.disabled;
        let interactive = !disabled;
        let on_click = self.on_click;
        let on_hover = self.on_hover;
        let on_secondary = self.on_secondary_mouse_down;
        let clickable = on_click.is_some();
        let indent = sizes.padding_lg * self.indent_level as f32;
        h_flex()
            .id(self.id)
            .group(group.clone())
            .w_full()
            .gap(sizes.spacing_md)
            .pl(sizes.card_padding_h + indent)
            .pr(sizes.card_padding_h)
            .py(padding_v)
            .when(self.rounded, |this| this.rounded(Sizes::ROW_RADIUS))
            .border_l(Sizes::ACCENT_BAR_WIDTH)
            .border_color(if self.selected {
                accent
            } else {
                theme.colors.transparent
            })
            .when(self.selected, |this| {
                this.bg(accent.opacity(theme.alpha.subtle))
            })
            .when(tinted, |this| this.bg(accent.opacity(theme.alpha.faint)))
            .when(disabled, |this| this.opacity(theme.alpha.half))
            .when(interactive, |this| {
                this.hover(move |style| style.bg(hover_background))
                    .when(clickable, |this| this.cursor_pointer().keyboard_activate(cx))
                    .when_some(on_click, |this, handler| {
                        this.on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx);
                        })
                    })
                    .when_some(on_hover, |this, handler| {
                        this.on_hover(move |hovered, window, cx| handler(hovered, window, cx))
                    })
                    .when_some(on_secondary, |this, handler| {
                        this.on_mouse_down(MouseButton::Right, move |event, window, cx| {
                            handler(event, window, cx);
                        })
                    })
            })
            .when_some(self.tooltip, |this, tooltip| {
                this.tooltip(move |window, cx| tooltip(window, cx))
            })
            .when_some(self.start_slot, |this, slot| {
                this.child(div().flex_none().child(slot))
            })
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(sizes.spacing_md)
                    .children(self.children),
            )
            .when_some(self.end_slot, |this, slot| {
                this.child(h_flex().flex_none().gap(sizes.spacing_sm).child(slot))
            })
            .when_some(self.end_slot_on_hover, |this, slot| {
                this.child(
                    h_flex()
                        .flex_none()
                        .gap(sizes.spacing_sm)
                        .invisible()
                        .group_hover(group, |style| style.visible())
                        .child(slot),
                )
            })
    }
}

pub struct ListHeader {
    id: ElementId,
    label: SharedString,
    count: Option<usize>,
    collapsible: bool,
    collapsed: bool,
    on_toggle: Option<ClickHandler>,
    end_slot: Option<AnyElement>,
    start_slot: Option<AnyElement>,
}

crate::impl_into_element!(ListHeader);

impl ListHeader {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            count: None,
            collapsible: false,
            collapsed: false,
            on_toggle: None,
            end_slot: None,
            start_slot: None,
        }
    }

    pub fn count(mut self, count: impl Into<Option<usize>>) -> Self {
        self.count = count.into();
        self
    }

    pub fn collapsible(mut self, collapsed: bool) -> Self {
        self.collapsible = true;
        self.collapsed = collapsed;
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
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

impl RenderOnce for ListHeader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let hover_background = theme.colors.surface_light;
        let hover_color = theme.colors.text_secondary;
        let on_toggle = self.on_toggle;
        let collapsible = self.collapsible;
        let open = !self.collapsed;
        h_flex()
            .id(self.id)
            .w_full()
            .gap(sizes.spacing_md)
            .px(sizes.card_padding_h)
            .py(sizes.spacing_md)
            .rounded(Sizes::ROW_RADIUS)
            .text_size(sizes.font_xs)
            .line_height(relative(LINE_HEIGHT_TIGHT))
            .font_weight(FontWeight::BOLD)
            .text_color(theme.colors.text_muted)
            .when(collapsible, |this| {
                this.cursor_pointer()
                    .keyboard_focus(cx)
                    .hover(move |style| style.bg(hover_background).text_color(hover_color))
                    .when_some(on_toggle, |this, handler| {
                        this.on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx);
                        })
                    })
            })
            .when(collapsible, |this| {
                this.child(chevron(open, 0, IconSize::Small, Color::Muted, window, cx))
            })
            .when_some(self.start_slot, |this, slot| {
                this.child(div().flex_none().child(slot))
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(Text::new(self.label.to_uppercase())),
            )
            .when_some(self.count, |this, count| this.child(CountBadge::new(count)))
            .when_some(self.end_slot, |this, slot| {
                this.child(div().flex_none().child(slot))
            })
    }
}

#[derive(Default)]
pub struct ListSeparator;

crate::impl_into_element!(ListSeparator);

impl ListSeparator {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for ListSeparator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        div()
            .w_full()
            .px(sizes.card_padding_h)
            .py(sizes.spacing_sm)
            .min_h(px(1.))
            .child(Divider::horizontal())
    }
}
