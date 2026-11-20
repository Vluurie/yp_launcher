use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FontWeight, IntoElement, ParentElement,
    RenderOnce, SharedString, Window, div, prelude::*, pulsating_between, relative,
};

use super::button::{ButtonSize, IconButton};
use super::icon::{Icon, IconName, IconSize};
use super::label::{LINE_HEIGHT_BODY, LabelSize};
use super::text::Text;
use super::tooltip::Tooltip;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Severity, Sizes, StatusColor};
use crate::traits::{ActionHandler, ButtonCommon, Clickable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BorderPosition {
    #[default]
    All,
    Left,
    Top,
    Bottom,
    None,
}

pub struct Callout {
    id: ElementId,
    severity: Severity,
    icon: Option<IconName>,
    show_icon: bool,
    title: Option<SharedString>,
    description: Option<SharedString>,
    description_slot: Option<AnyElement>,
    actions_slot: Option<AnyElement>,
    dismiss_action: Option<AnyElement>,
    on_dismiss: Option<ActionHandler>,
    pulse: bool,
    border_position: BorderPosition,
    title_size: LabelSize,
    filled: bool,
    neutral: bool,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(Callout);

impl Default for Callout {
    fn default() -> Self {
        Self::new()
    }
}

impl Callout {
    pub fn new() -> Self {
        Self {
            id: ElementId::Name(SharedString::new_static("callout")),
            severity: Severity::Info,
            icon: None,
            show_icon: true,
            title: None,
            description: None,
            description_slot: None,
            actions_slot: None,
            dismiss_action: None,
            on_dismiss: None,
            pulse: false,
            border_position: BorderPosition::All,
            title_size: LabelSize::Default,
            filled: true,
            neutral: false,
            children: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn no_icon(mut self) -> Self {
        self.show_icon = false;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn title_size(mut self, size: LabelSize) -> Self {
        self.title_size = size;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn description_slot(mut self, slot: impl IntoElement) -> Self {
        self.description_slot = Some(slot.into_any_element());
        self
    }

    pub fn actions_slot(mut self, slot: impl IntoElement) -> Self {
        self.actions_slot = Some(slot.into_any_element());
        self
    }

    pub fn dismiss_action(mut self, action: impl IntoElement) -> Self {
        self.dismiss_action = Some(action.into_any_element());
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    pub fn border_position(mut self, position: BorderPosition) -> Self {
        self.border_position = position;
        self
    }

    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    pub fn neutral(mut self) -> Self {
        self.neutral = true;
        self
    }

    pub fn default_icon(severity: Severity) -> IconName {
        match severity {
            Severity::Info => IconName::InfoOutline,
            Severity::Accent => IconName::LightbulbOutline,
            Severity::Success => IconName::CheckCircleOutline,
            Severity::Warning => IconName::WarningAmber,
            Severity::Error => IconName::ErrorOutline,
        }
    }
}

impl ParentElement for Callout {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Callout {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let status = if self.neutral {
            StatusColor {
                fg: theme.colors.text_secondary,
                background: theme.colors.surface_light,
                border: theme.colors.border_medium,
            }
        } else {
            theme.status.for_severity(self.severity)
        };
        let color = if self.neutral {
            Color::Muted
        } else {
            Color::from(self.severity)
        };
        let icon = self
            .icon
            .unwrap_or_else(|| Callout::default_icon(self.severity));
        let border = if self.neutral {
            status.border
        } else {
            status.fg.opacity(theme.alpha.half)
        };
        let dismiss_tooltip = cx.t().ui_dismiss();
        let dismiss = self.dismiss_action.or_else(|| {
            self.on_dismiss.map(|handler| {
                IconButton::new((self.id.clone(), "dismiss"), IconName::Close)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Muted)
                    .size(ButtonSize::Compact)
                    .tooltip(Tooltip::text(dismiss_tooltip))
                    .on_click(move |_, window, cx| handler(window, cx))
                    .into_any_element()
            })
        });
        let body = h_flex()
            .id(self.id.clone())
            .w_full()
            .items_start()
            .gap(sizes.spacing_md)
            .px(sizes.padding_md)
            .py(sizes.padding_sm)
            .rounded(sizes.border_radius)
            .when(self.filled, |this| this.bg(status.background))
            .map(|this| match self.border_position {
                BorderPosition::All => this.border_1().border_color(border),
                BorderPosition::Left => this
                    .border_l(Sizes::ACCENT_BAR_WIDTH)
                    .border_color(status.fg),
                BorderPosition::Top => this.border_t_1().border_color(border),
                BorderPosition::Bottom => this.border_b_1().border_color(border),
                BorderPosition::None => this,
            })
            .when(self.show_icon, |this| {
                this.child(
                    div()
                        .pt(sizes.spacing_xs)
                        .child(Icon::new(icon).size(IconSize::Medium).color(color)),
                )
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(sizes.spacing_sm)
                    .when_some(self.title, |this, title| {
                        this.child(
                            v_flex()
                                .text_size(self.title_size.pixels(&sizes))
                                .line_height(relative(LINE_HEIGHT_BODY))
                                .font_weight(FontWeight::BOLD)
                                .text_color(status.fg)
                                .child(Text::new(title)),
                        )
                    })
                    .when_some(self.description, |this, description| {
                        this.child(
                            v_flex()
                                .text_size(sizes.font_xs)
                                .line_height(relative(LINE_HEIGHT_BODY))
                                .text_color(theme.colors.text_secondary)
                                .child(Text::new(description)),
                        )
                    })
                    .when_some(self.description_slot, |this, slot| this.child(slot))
                    .children(self.children),
            )
            .when_some(self.actions_slot, |this, slot| {
                this.child(div().flex_none().child(slot))
            })
            .when_some(dismiss, |this, dismiss| {
                this.child(div().flex_none().child(dismiss))
            });
        if self.pulse {
            body.with_animation(
                (self.id, "pulse"),
                Animation::new(theme.motion.pulse * 2)
                    .repeat()
                    .with_easing(pulsating_between(0.6, 1.)),
                |this, delta| this.opacity(delta),
            )
            .into_any_element()
        } else {
            body.into_any_element()
        }
    }
}
