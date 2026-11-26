use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, ClickEvent, ElementId, FontWeight, Hsla, IntoElement,
    RenderOnce, Rgba, ScrollHandle, SharedString, Window, div, prelude::*, px, relative, svg,
};

use super::card::chevron;
use super::divider::Divider;
use super::icon::{IconName, IconSize};
use super::label::LINE_HEIGHT_TIGHT;
use super::scroll::ScrollableElement;
use super::text::Text;
use super::tooltip::Tooltip;
use super::tour::{AnchorId, tour_anchor_tracker};
use super::util::{KeyboardFocus, hover_probe, use_hover_state};
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, NavHover, ResponsiveExt, Sizes, easing};
use crate::traits::ClickHandler;

pub fn mix_colors(from: Hsla, to: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0., 1.);
    let from = Rgba::from(from);
    let to = Rgba::from(to);
    Hsla::from(Rgba {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    })
}

pub struct SidebarItem {
    id: ElementId,
    icon: IconName,
    label: SharedString,
    active: bool,
    tour_anchor: Option<AnchorId>,
    on_click: Option<ClickHandler>,
}

impl SidebarItem {
    pub fn new(id: impl Into<ElementId>, icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon,
            label: label.into(),
            active: false,
            tour_anchor: None,
            on_click: None,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn tour_anchor(mut self, anchor: impl Into<AnchorId>) -> Self {
        self.tour_anchor = Some(anchor.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

pub struct SidebarGroup {
    title: SharedString,
    collapsible: bool,
    collapsed: bool,
    count: Option<usize>,
    on_toggle: Option<ClickHandler>,
    items: Vec<SidebarItem>,
}

impl SidebarGroup {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            collapsible: false,
            collapsed: false,
            count: None,
            on_toggle: None,
            items: Vec::new(),
        }
    }

    pub fn collapsible(mut self) -> Self {
        self.collapsible = true;
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }

    pub fn item(mut self, item: SidebarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SidebarItem>) -> Self {
        self.items.extend(items);
        self
    }
}

pub struct Sidebar {
    id: ElementId,
    icon_only: Option<bool>,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    groups: Vec<SidebarGroup>,
}

crate::impl_into_element!(Sidebar);

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            id: ElementId::Name("yp-sidebar".into()),
            icon_only: None,
            header: None,
            footer: None,
            groups: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn icon_only(mut self, icon_only: bool) -> Self {
        self.icon_only = Some(icon_only);
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn group(mut self, group: SidebarGroup) -> Self {
        self.groups.push(group);
        self
    }
}

#[derive(Clone)]
struct ItemRow {
    icon: IconName,
    label: SharedString,
    show_label: bool,
    active: bool,
    sizes: Sizes,
}

impl ItemRow {
    fn render(&self, icon_color: Hsla, text_color: Hsla) -> gpui::Div {
        let sizes = self.sizes;
        let active = self.active;
        let label = self.label.clone();
        h_flex()
            .size_full()
            .when(!self.show_label, |this| this.justify_center())
            .when(self.show_label, |this| {
                this.pl(sizes.card_padding_h).gap(sizes.spacing_md)
            })
            .child(
                svg()
                    .path(self.icon.path())
                    .size(sizes.icon_md)
                    .flex_none()
                    .text_color(icon_color),
            )
            .when(self.show_label, |this| {
                this.child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(sizes.font_sm)
                        .line_height(relative(LINE_HEIGHT_TIGHT))
                        .font_weight(if active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(text_color)
                        .child(Text::new(label)),
                )
            })
    }
}

fn render_item(
    item: SidebarItem,
    show_label: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = cx.theme().clone();
    let sizes = window.sizes(cx);
    let accent = theme.colors.accent_primary;
    let active = item.active;
    let on_click = item.on_click;
    let label = item.label.clone();
    let icon = item.icon;
    let bar = px(3.);
    let base = div()
        .id(item.id.clone())
        .role(gpui::Role::Link).aria_label(label.clone())
        .relative()
        .w_full()
        .h(sizes.sidebar_row_height)
        .flex_none()
        .overflow_hidden()
        .cursor_pointer()
        .when_some(on_click, |this, handler| {
            this.keyboard_activate(cx).on_click(move |event, window, cx| {
                cx.stop_propagation();
                handler(event, window, cx);
            })
        })
        .when(!show_label, |this| {
            this.tooltip(Tooltip::text(label.clone()))
        })
        .when_some(item.tour_anchor, |this, anchor| {
            this.child(tour_anchor_tracker(anchor))
        });

    match theme.style.nav_hover {
        NavHover::Tint => {
            let group = SharedString::from(format!("yp-sidebar-item-{}", item.id));
            let idle_icon = if active {
                accent
            } else {
                theme.colors.text_muted
            };
            let idle_text = if active {
                accent
            } else {
                theme.colors.text_secondary
            };
            let hover_color = if active {
                accent
            } else {
                theme.colors.text_primary
            };
            let hover_background = if active {
                accent.opacity(theme.alpha.hover)
            } else {
                theme.colors.surface_light
            };
            base.group(group.clone())
                .border_l(bar)
                .border_color(if active {
                    accent
                } else {
                    theme.colors.transparent
                })
                .when(active, |this| this.bg(accent.opacity(theme.alpha.subtle)))
                .hover(move |style| style.bg(hover_background))
                .child(
                    h_flex()
                        .size_full()
                        .when(!show_label, |this| this.justify_center())
                        .when(show_label, |this| {
                            this.pl(sizes.card_padding_h).gap(sizes.spacing_md)
                        })
                        .child(
                            svg()
                                .path(icon.path())
                                .size(sizes.icon_md)
                                .flex_none()
                                .text_color(idle_icon)
                                .group_hover(group.clone(), move |style| {
                                    style.text_color(hover_color)
                                }),
                        )
                        .when(show_label, |this| {
                            this.child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(sizes.font_sm)
                                    .line_height(relative(LINE_HEIGHT_TIGHT))
                                    .font_weight(if active {
                                        FontWeight::SEMIBOLD
                                    } else {
                                        FontWeight::NORMAL
                                    })
                                    .text_color(idle_text)
                                    .group_hover(group, move |style| style.text_color(hover_color))
                                    .child(Text::new(label)),
                            )
                        }),
                )
                .into_any_element()
        }
        NavHover::InkSweep => {
            let state = use_hover_state((item.id.clone(), "ink"), window, cx);
            let (hovered, generation) = {
                let state = state.read(cx);
                (state.hovered(), state.changes())
            };
            let ink = theme.colors.text_primary;
            let paper = theme.colors.background_primary;
            let base = base.child(hover_probe(&state));
            let row = ItemRow {
                icon,
                label,
                show_label,
                active,
                sizes,
            };
            let paint = move |t: f32| {
                let fg = mix_colors(ink, paper, t);
                div()
                    .absolute()
                    .inset_0()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .w(relative(t))
                            .bg(ink),
                    )
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .w(bar)
                            .bg(ink.opacity(t)),
                    )
                    .child(div().absolute().inset_0().child(row.render(fg, fg)))
            };
            if active || generation == 0 {
                let t = if active { 1. } else { 0. };
                return base.child(paint(t)).into_any_element();
            }
            let duration = theme.motion.sweep;
            let ink_layer = div().absolute().inset_0().with_animation(
                ("yp-ink", generation),
                Animation::new(duration),
                move |this, delta| {
                    let eased = easing::nier_fill(delta);
                    let t = if hovered { eased } else { 1. - eased };
                    this.child(paint(t))
                },
            );
            base.child(ink_layer).into_any_element()
        }
    }
}

impl RenderOnce for Sidebar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let show_labels = match self.icon_only {
            Some(icon_only) => !icon_only,
            None => sizes.sidebar_labels_visible(),
        };
        let scroll_handle = window
            .use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let focus_scope = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let mut body = div()
            .id((self.id.clone(), "groups"))
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .py(sizes.padding_md);
        for (index, group) in self.groups.into_iter().enumerate() {
            let collapsed = group.collapsible && group.collapsed;
            let open = !collapsed;
            if index > 0 {
                body = body.child(div().flex_none().h(sizes.sidebar_section_gap));
            }
            let item_count = group.count.unwrap_or(group.items.len());
            let hover_color = theme.colors.text_secondary;
            if show_labels {
                let on_toggle = group.on_toggle.clone();
                let header = h_flex()
                    .id(("yp-sidebar-group", index))
                    .role(if group.collapsible { gpui::Role::Button } else { gpui::Role::Heading })
                    .aria_label(group.title.clone())
                    .when(group.collapsible, |el| el.aria_expanded(open))
                    .flex_none()
                    .w_full()
                    .gap(px(6.))
                    .pl(sizes.card_padding_h)
                    .pr(sizes.card_padding_h)
                    .pt(sizes.spacing_md)
                    .pb(sizes.spacing_sm)
                    .text_size(sizes.font_xs)
                    .line_height(relative(LINE_HEIGHT_TIGHT))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.colors.text_muted)
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(Text::new(group.title.to_uppercase())),
                    )
                    .when(group.collapsible, |this| {
                        this.cursor_pointer()
                            .hover(move |style| style.text_color(hover_color))
                            .when(collapsed, |this| {
                                this.child(
                                    v_flex()
                                        .flex_none()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(Text::new(item_count.to_string())),
                                )
                            })
                            .child(chevron(open, 0, IconSize::Small, Color::Muted, window, cx))
                            .when_some(on_toggle, |this, handler| {
                                this.keyboard_activate(cx).on_disclosure_toggle(open, move |event, window, cx| {
                                    cx.stop_propagation();
                                    handler(event, window, cx);
                                })
                            })
                    });
                body = body.child(header);
            } else {
                if index > 0 {
                    body = body.child(
                        div()
                            .flex_none()
                            .px(sizes.card_padding_h)
                            .py(sizes.spacing_sm)
                            .child(Divider::horizontal()),
                    );
                }
                if group.collapsible {
                    let on_toggle = group.on_toggle.clone();
                    let hover_background = theme.colors.surface_light;
                    body = body.child(
                        h_flex()
                            .id(("yp-sidebar-group-toggle", index))
                            .role(gpui::Role::Button).aria_label(group.title.clone()).aria_expanded(open)
                            .flex_none()
                            .w_full()
                            .justify_center()
                            .py(px(4.))
                            .cursor_pointer()
                            .hover(move |style| style.bg(hover_background))
                            .child(chevron(open, 0, IconSize::Small, Color::Muted, window, cx))
                            .tooltip(Tooltip::text(group.title.clone()))
                            .when_some(on_toggle, |this, handler| {
                                this.keyboard_activate(cx).on_disclosure_toggle(open, move |event, window, cx| {
                                    cx.stop_propagation();
                                    handler(event, window, cx);
                                })
                            }),
                    );
                }
            }
            if open {
                for item in group.items {
                    body = body.child(render_item(item, show_labels, window, cx));
                }
            }
        }
        v_flex()
            .id(self.id)
            .track_focus(&focus_scope)
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key != "right" || !focus_scope.contains_focused(window, cx) {
                    return;
                }
                let original = window.focused(cx);
                cx.stop_propagation();
                window.prevent_default();
                window.focus_next(cx);
                let first = window.focused(cx);
                while focus_scope.contains_focused(window, cx) {
                    window.focus_next(cx);
                    if window.focused(cx) == first {
                        if let Some(original) = original {
                            window.focus(&original, cx);
                        }
                        break;
                    }
                }
            })
            .flex_none()
            .h_full()
            .w(sizes.sidebar_width)
            .bg(theme.colors.surface_medium)
            .border_r_1()
            .border_color(theme.colors.border_light)
            .children(self.header.map(|header| div().flex_none().child(header)))
            .child(body.overflow_y_scrollbar(&scroll_handle))
            .children(self.footer.map(|footer| {
                v_flex()
                    .flex_none()
                    .w_full()
                    .px(sizes.card_padding_h)
                    .py(sizes.spacing_sm)
                    .border_t_1()
                    .border_color(theme.colors.border_light)
                    .child(footer)
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn mixing_interpolates_each_channel_and_clamps() {
        let colors = Theme::dark().colors;
        let black = colors.scrim;
        let white = colors.text_on_dark;
        let middle = Rgba::from(mix_colors(black, white, 0.5));
        assert!((middle.r - 0.5).abs() < 0.01);
        assert!((middle.g - 0.5).abs() < 0.01);
        assert!((middle.b - 0.5).abs() < 0.01);
        assert_eq!(Rgba::from(mix_colors(black, white, -3.)), Rgba::from(black));
        let full = Rgba::from(mix_colors(black, white, 7.));
        assert!((full.r - 1.).abs() < 0.01);
    }
}
