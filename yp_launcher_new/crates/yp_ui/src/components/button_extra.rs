use std::rc::Rc;

use gpui::{
    App, ClickEvent, ClipboardItem, ElementId, FontWeight, IntoElement, RenderOnce, SharedString,
    Task, Window, div, prelude::*, px, relative,
};

use super::button::{Button, ButtonSize, ButtonStyle, IconButton};
use super::chip::CountBadge;
use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LINE_HEIGHT_TIGHT};
use super::text::Text;
use super::tooltip::Tooltip;
use super::util::KeyboardFocus;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::{ButtonCommon, ClickHandler, Clickable, ValueHandler};

#[derive(Default)]
struct CopyState {
    copied: bool,
    reset: Option<Task<()>>,
}

pub struct CopyButton {
    id: ElementId,
    text: SharedString,
    label: Option<SharedString>,
    tooltip: Option<SharedString>,
    icon_size: IconSize,
    size: ButtonSize,
    style: ButtonStyle,
    bordered: bool,
    on_copied: Option<ClickHandler>,
}

crate::impl_into_element!(CopyButton);

impl CopyButton {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            label: None,
            tooltip: None,
            icon_size: IconSize::Small,
            size: ButtonSize::Compact,
            style: ButtonStyle::Subtle,
            bordered: false,
            on_copied: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn icon_size(mut self, size: IconSize) -> Self {
        self.icon_size = size;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    pub fn on_copied(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_copied = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for CopyButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let feedback = cx.theme().motion.copy_feedback;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| CopyState::default());
        let copied = state.read(cx).copied;
        let icon = if copied {
            IconName::Check
        } else {
            IconName::ContentCopy
        };
        let color = if copied {
            Color::Success
        } else {
            Color::Accent
        };
        let tooltip = if copied {
            SharedString::new_static(strings.docs_copied())
        } else {
            self.tooltip
                .unwrap_or_else(|| SharedString::new_static(strings.docs_copy()))
        };
        let text = self.text;
        let on_copied = self.on_copied;
        let on_click = move |event: &ClickEvent, window: &mut Window, cx: &mut App| {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
            state.update(cx, |state, cx| {
                state.copied = true;
                state.reset = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(feedback).await;
                    this.update(cx, |state, cx| {
                        state.copied = false;
                        cx.notify();
                    })
                    .ok();
                }));
                cx.notify();
            });
            if let Some(handler) = &on_copied {
                handler(event, window, cx);
            }
        };
        match self.label {
            Some(label) => Button::new(self.id, label)
                .start_icon(icon)
                .icon_size(self.icon_size)
                .color(color)
                .style(self.style)
                .size(self.size)
                .tooltip(Tooltip::text(tooltip))
                .on_click(on_click)
                .into_any_element(),
            None => IconButton::new(self.id, icon)
                .icon_size(self.icon_size)
                .icon_color(color)
                .style(self.style)
                .size(self.size)
                .bordered(self.bordered)
                .tooltip(Tooltip::text(tooltip))
                .on_click(on_click)
                .into_any_element(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToggleOption {
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub count: Option<usize>,
    pub tooltip: Option<SharedString>,
    pub disabled: bool,
}

impl ToggleOption {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            count: None,
            tooltip: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: impl Into<Option<IconName>>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct ToggleButtonGroup {
    id: ElementId,
    options: Vec<ToggleOption>,
    selected: Option<usize>,
    size: ButtonSize,
    full_width: bool,
    on_select: Option<ValueHandler<usize>>,
}

crate::impl_into_element!(ToggleButtonGroup);

impl ToggleButtonGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            options: Vec::new(),
            selected: None,
            size: ButtonSize::Default,
            full_width: false,
            on_select: None,
        }
    }

    pub fn option(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<Option<IconName>>,
    ) -> Self {
        self.options.push(ToggleOption::new(label).icon(icon));
        self
    }

    pub fn option_with(mut self, option: ToggleOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn selected(mut self, index: impl Into<Option<usize>>) -> Self {
        self.selected = index.into();
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ToggleButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let accent = theme.colors.accent_primary;
        let selected_hover = accent.opacity(theme.alpha.line);
        let (padding_h, padding_v) = self.size.padding(&sizes);
        let font = self.size.label_size().pixels(&sizes);
        let last = self.options.len().saturating_sub(1);
        let selected = self.selected;
        let on_select = self.on_select;
        let hover_background = accent.opacity(theme.alpha.subtle);
        let buttons = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let is_selected = selected == Some(index);
                let handler = on_select.clone();
                let color = if is_selected {
                    Color::Accent
                } else {
                    Color::Secondary
                };
                let group = SharedString::from(format!("yp-toggle-{}-{index}", self.id));
                h_flex()
                    .id((self.id.clone(), SharedString::from(index.to_string())))
                    .group(group.clone())
                    .justify_center()
                    .gap(sizes.spacing_md)
                    .px(padding_h)
                    .py(padding_v)
                    .when(self.full_width, |this| this.flex_1().min_w_0())
                    .when(index == 0, |this| this.rounded_l(Sizes::CONTROL_RADIUS))
                    .when(index == last, |this| this.rounded_r(Sizes::CONTROL_RADIUS))
                    .border_1()
                    .when(index > 0, |this| this.border_l_0())
                    .border_color(theme.colors.border_medium)
                    .text_size(font)
                    .line_height(relative(LINE_HEIGHT_TIGHT))
                    .font_weight(if is_selected {
                        FontWeight::BOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(color.color(cx))
                    .when(is_selected, |this| {
                        this.bg(accent.opacity(theme.alpha.hover))
                    })
                    .when(option.disabled, |this| {
                        this.opacity(theme.alpha.half).cursor_not_allowed()
                    })
                    .when(!option.disabled, |this| {
                        this.cursor_pointer()
                            .keyboard_focus(cx)
                            .when(!is_selected, |this| {
                                this.hover(move |style| {
                                    style.bg(hover_background).text_color(accent)
                                })
                            })
                            .when(is_selected, |this| {
                                this.hover(move |style| style.bg(selected_hover))
                            })
                            .when_some(handler, |this, handler| {
                                this.on_click(move |_, window, cx| {
                                    cx.stop_propagation();
                                    handler(&index, window, cx);
                                })
                            })
                    })
                    .when_some(option.icon, |this, icon| {
                        this.child(
                            Icon::new(icon)
                                .size(self.size.icon_size())
                                .color(color)
                                .map(|icon| {
                                    if option.disabled {
                                        icon
                                    } else {
                                        icon.group_hover_color(group, Color::Accent)
                                    }
                                }),
                        )
                    })
                    .child(
                        v_flex()
                            .when(self.full_width, |this| this.min_w_0().truncate())
                            .child(Text::new(option.label)),
                    )
                    .when_some(option.count, |this, count| {
                        this.child(CountBadge::new(count))
                    })
                    .when_some(option.tooltip, |this, tooltip| {
                        this.tooltip(Tooltip::text(tooltip))
                    })
            })
            .collect::<Vec<_>>();
        h_flex()
            .id(self.id)
            .when(self.full_width, |this| this.w_full())
            .when(!self.full_width, |this| this.flex_none())
            .children(buttons)
    }
}

pub struct ChoiceCard {
    id: ElementId,
    title: SharedString,
    description: Option<SharedString>,
    icon: Option<IconName>,
    selected: bool,
    disabled: bool,
    compact: bool,
    on_click: Option<ClickHandler>,
}

crate::impl_into_element!(ChoiceCard);

impl ChoiceCard {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            icon: None,
            selected: false,
            disabled: false,
            compact: false,
            on_click: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
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

impl RenderOnce for ChoiceCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let accent = theme.colors.accent_primary;
        let selected = self.selected;
        let disabled = self.disabled;
        let hover_background = accent.opacity(theme.alpha.subtle);
        let hover_border = accent.opacity(theme.alpha.half);
        let icon_color = if selected {
            Color::Accent
        } else {
            Color::Muted
        };
        let group = SharedString::from(format!("yp-choice-{}", self.id));
        let on_click = self.on_click;
        h_flex()
            .id(self.id)
            .group(group.clone())
            .w_full()
            .items_start()
            .gap(sizes.spacing_lg)
            .px(sizes.card_padding_h)
            .py(if self.compact {
                sizes.card_padding_v
            } else {
                sizes.card_padding_h
            })
            .rounded(sizes.border_radius)
            .border_1()
            .border_color(if selected {
                accent
            } else {
                theme.colors.border_light
            })
            .bg(if selected {
                accent.opacity(theme.alpha.subtle)
            } else {
                theme.colors.surface_light
            })
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .keyboard_focus(cx)
                    .hover(move |style| style.bg(hover_background).border_color(hover_border))
                    .when_some(on_click, |this, handler| {
                        this.on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx);
                        })
                    })
            })
            .when_some(self.icon, |this, icon| {
                this.child(
                    div().pt(px(2.)).child(
                        Icon::new(icon)
                            .size(IconSize::Large)
                            .color(icon_color)
                            .group_hover_color(group, Color::Accent),
                    ),
                )
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(sizes.spacing_sm)
                    .child(
                        v_flex()
                            .text_size(sizes.font_md)
                            .line_height(relative(LINE_HEIGHT_TIGHT))
                            .font_weight(FontWeight::BOLD)
                            .text_color(if selected {
                                accent
                            } else {
                                theme.colors.text_primary
                            })
                            .child(Text::new(self.title)),
                    )
                    .when_some(self.description, |this, description| {
                        this.child(
                            v_flex()
                                .text_size(sizes.font_xs)
                                .line_height(relative(LINE_HEIGHT_BODY))
                                .text_color(theme.colors.text_muted)
                                .child(Text::new(description)),
                        )
                    }),
            )
            .when(selected, |this| {
                this.child(
                    Icon::new(IconName::CheckCircle)
                        .size(IconSize::Medium)
                        .color(Color::Accent),
                )
            })
    }
}
