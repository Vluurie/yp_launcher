use std::rc::Rc;

use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*, px, relative,
};

use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LabelSize};
use super::util::KeyboardFocus;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::{ToggleState, ValueHandler};

pub struct Checkbox {
    id: ElementId,
    state: ToggleState,
    label: Option<SharedString>,
    label_size: LabelSize,
    label_color: Color,
    tooltip: Option<SharedString>,
    disabled: bool,
    on_click: Option<ValueHandler<ToggleState>>,
}

crate::impl_into_element!(Checkbox);

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, state: impl Into<ToggleState>) -> Self {
        Self {
            id: id.into(),
            state: state.into(),
            label: None,
            label_size: LabelSize::Default,
            label_color: Color::Default,
            tooltip: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn label_size(mut self, size: LabelSize) -> Self {
        self.label_size = size;
        self
    }

    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = color;
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

    pub fn on_click(
        mut self,
        handler: impl Fn(&ToggleState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let box_size = sizes.checkbox_size - px(4.);
        let checked = self.state != ToggleState::Unselected;
        let accent = if theme.id == crate::ThemeId::NierNew {
            theme.colors.success
        } else {
            theme.colors.accent_primary
        };
        let group = SharedString::from(format!("yp-checkbox-{}", self.id));
        let next = self.state.inverse();
        let on_click = self.on_click;
        let disabled = self.disabled;
        let hover_border = accent;
        let hover_fill = accent.opacity(theme.alpha.hover);
        let label_color = self.label_color.color(cx);
        h_flex()
            .id(self.id)
            .role(gpui::Role::CheckBox)
            .aria_toggled(match self.state {
                ToggleState::Selected => gpui::Toggled::True,
                ToggleState::Unselected => gpui::Toggled::False,
                ToggleState::Indeterminate => gpui::Toggled::Mixed,
            })
            .when_some(self.label.clone(), |el, label| el.aria_label(label))
            .group(group.clone())
            .gap(sizes.spacing_md)
            .when_some(self.tooltip, |this, tooltip| {
                this.tooltip(super::tooltip::Tooltip::text(tooltip))
            })
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .rounded(Sizes::CONTROL_RADIUS)
                    .keyboard_activate(cx)
                    .when_some(on_click, |this, handler| {
                        this.on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            handler(&next, window, cx);
                        })
                    })
            })
            .child(
                div()
                    .flex_none()
                    .size(box_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(3.))
                    .border_1()
                    .border_color(if checked {
                        accent
                    } else {
                        theme.colors.border_medium
                    })
                    .bg(if checked {
                        accent
                    } else {
                        theme.colors.transparent
                    })
                    .when(!disabled, |this| {
                        this.group_hover(group, move |style| {
                            let style = style.border_color(hover_border);
                            if checked { style } else { style.bg(hover_fill) }
                        })
                    })
                    .when(self.state == ToggleState::Selected, |this| {
                        this.child(
                            Icon::new(IconName::Check)
                                .size(IconSize::Custom(box_size - px(4.)))
                                .color(Color::OnAccent),
                        )
                    })
                    .when(self.state == ToggleState::Indeterminate, |this| {
                        this.child(
                            Icon::new(IconName::Remove)
                                .size(IconSize::Custom(box_size - px(4.)))
                                .color(Color::OnAccent),
                        )
                    }),
            )
            .when_some(self.label, |this, label| {
                this.child(
                    v_flex()
                        .min_w_0()
                        .text_size(self.label_size.pixels(&sizes))
                        .line_height(relative(LINE_HEIGHT_BODY))
                        .text_color(label_color)
                        .child(super::text::Text::new(label)),
                )
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SwitchSize {
    Small,
    #[default]
    Default,
}

pub struct Switch {
    id: ElementId,
    accessible_label: Option<SharedString>,
    state: ToggleState,
    label: Option<SharedString>,
    size: SwitchSize,
    disabled: bool,
    on_click: Option<ValueHandler<ToggleState>>,
}

crate::impl_into_element!(Switch);

impl Switch {
    pub fn accessible_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    pub fn new(id: impl Into<ElementId>, state: impl Into<ToggleState>) -> Self {
        Self {
            id: id.into(),
            state: state.into(),
            label: None,
            size: SwitchSize::default(),
            accessible_label: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ToggleState, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let on = self.state.selected();
        let (width, height) = match self.size {
            SwitchSize::Small => (px(28.), px(16.)),
            SwitchSize::Default => (Sizes::SWITCH_WIDTH, Sizes::SWITCH_HEIGHT),
        };
        let accent = if theme.id == crate::ThemeId::NierNew {
            theme.colors.success
        } else {
            theme.colors.accent_primary
        };
        let thumb = if on { height * 0.6 } else { height * 0.5 };
        let inset = (height - thumb) / 2.;
        let radius = if theme.id == crate::ThemeId::NierNew {
            px(0.)
        } else {
            height
        };
        let track = if on { accent } else { theme.colors.transparent };
        let track_border = if on { accent } else { theme.colors.text_muted };
        let thumb_color = if on {
            theme.colors.button_text
        } else {
            theme.colors.text_secondary
        };
        let group = SharedString::from(format!("yp-switch-{}", self.id));
        let next = self.state.inverse();
        let on_click = self.on_click;
        let disabled = self.disabled;
        h_flex()
            .id(self.id)
            .role(gpui::Role::Switch)
            .aria_toggled(match self.state {
                ToggleState::Selected => gpui::Toggled::True,
                ToggleState::Unselected => gpui::Toggled::False,
                ToggleState::Indeterminate => gpui::Toggled::Mixed,
            })
            .when_some(
                self.accessible_label.or_else(|| self.label.clone()),
                |el, label| el.aria_label(label),
            )
            .group(group.clone())
            .gap(sizes.spacing_lg)
            .when(disabled, |this| {
                this.opacity(theme.alpha.half).cursor_not_allowed()
            })
            .when(!disabled, |this| {
                this.cursor_pointer()
                    .rounded(radius)
                    .keyboard_activate(cx)
                    .when_some(on_click, |this, handler| {
                        this.on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            handler(&next, window, cx);
                        })
                    })
            })
            .child(
                div()
                    .relative()
                    .flex_none()
                    .w(width)
                    .h(height)
                    .rounded(radius)
                    .bg(track)
                    .border_1()
                    .border_color(track_border)
                    .when(!disabled, |this| {
                        let hover_track = if on {
                            accent
                        } else {
                            accent.opacity(theme.alpha.hover)
                        };
                        this.group_hover(group, move |style| {
                            style.border_color(accent).bg(hover_track)
                        })
                    })
                    .child(
                        div()
                            .absolute()
                            .top(inset - px(1.))
                            .left(if on {
                                width - thumb - inset - px(2.)
                            } else {
                                inset
                            })
                            .size(thumb)
                            .rounded(radius.min(thumb))
                            .bg(thumb_color),
                    ),
            )
            .when_some(self.label, |this, label| {
                this.child(
                    v_flex()
                        .min_w_0()
                        .text_size(sizes.font_md)
                        .line_height(relative(LINE_HEIGHT_BODY))
                        .text_color(theme.colors.text_primary)
                        .child(super::text::Text::new(label)),
                )
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RadioLayout {
    #[default]
    Vertical,
    Horizontal,
}

pub struct RadioGroup {
    id: ElementId,
    options: Vec<(SharedString, Option<SharedString>)>,
    selected: Option<usize>,
    layout: RadioLayout,
    disabled: bool,
    on_change: Option<ValueHandler<usize>>,
}

crate::impl_into_element!(RadioGroup);

impl RadioGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            options: Vec::new(),
            selected: None,
            layout: RadioLayout::default(),
            disabled: false,
            on_change: None,
        }
    }

    pub fn option(mut self, label: impl Into<SharedString>) -> Self {
        self.options.push((label.into(), None));
        self
    }

    pub fn option_with_description(
        mut self,
        label: impl Into<SharedString>,
        description: impl Into<SharedString>,
    ) -> Self {
        self.options.push((label.into(), Some(description.into())));
        self
    }

    pub fn selected(mut self, index: impl Into<Option<usize>>) -> Self {
        self.selected = index.into();
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.layout = RadioLayout::Horizontal;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let hover_background = theme.colors.surface_light;
        let disabled = self.disabled;
        let on_change = self.on_change;
        let selected = self.selected;
        let rows = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, (label, description))| {
                let is_selected = selected == Some(index);
                let handler = on_change.clone();
                let (icon, color) = if is_selected {
                    (IconName::RadioButtonChecked, Color::Accent)
                } else {
                    (IconName::RadioButtonUnchecked, Color::Muted)
                };
                h_flex()
                    .id((self.id.clone(), SharedString::from(index.to_string())))
                    .items_start()
                    .gap(sizes.spacing_md)
                    .px(sizes.padding_sm)
                    .py(sizes.padding_xs)
                    .rounded(Sizes::ROW_RADIUS)
                    .when(disabled, |this| {
                        this.opacity(theme.alpha.half).cursor_not_allowed()
                    })
                    .when(!disabled, |this| {
                        this.cursor_pointer()
                            .keyboard_activate(cx)
                            .hover(move |style| style.bg(hover_background))
                            .when_some(handler, |this, handler| {
                                this.on_click(move |_, window, cx| {
                                    cx.stop_propagation();
                                    handler(&index, window, cx);
                                })
                            })
                    })
                    .child(Icon::new(icon).size(IconSize::Medium).color(color))
                    .child(
                        v_flex()
                            .min_w_0()
                            .child(
                                v_flex()
                                    .text_size(sizes.font_md)
                                    .line_height(relative(LINE_HEIGHT_BODY))
                                    .text_color(theme.colors.text_primary)
                                    .child(super::text::Text::new(label)),
                            )
                            .when_some(description, |this, description| {
                                this.child(
                                    v_flex()
                                        .text_size(sizes.font_xs)
                                        .line_height(relative(LINE_HEIGHT_BODY))
                                        .text_color(theme.colors.text_muted)
                                        .child(super::text::Text::new(description)),
                                )
                            }),
                    )
            })
            .collect::<Vec<_>>();
        match self.layout {
            RadioLayout::Vertical => v_flex().gap(sizes.spacing_sm).children(rows),
            RadioLayout::Horizontal => h_flex().flex_wrap().gap(sizes.spacing_lg).children(rows),
        }
    }
}
