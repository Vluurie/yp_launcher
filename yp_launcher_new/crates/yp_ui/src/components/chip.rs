use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyView, App, ClickEvent, ElementId, FontWeight, IntoElement,
    RenderOnce, SharedString, Window, div, prelude::*, px, relative,
};

use super::icon::{Icon, IconName, IconSize};
use super::label::LINE_HEIGHT_TIGHT;
use super::text::Text;
use super::tooltip::Tooltip;
use super::util::KeyboardFocus;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Severity, Sizes, StatusColor};
use crate::traits::{ClickHandler, TooltipBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChipTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Error,
    Muted,
    Info,
}

impl ChipTone {
    pub fn color(self) -> Color {
        match self {
            ChipTone::Neutral => Color::Secondary,
            ChipTone::Accent => Color::Accent,
            ChipTone::Success => Color::Success,
            ChipTone::Warning => Color::Warning,
            ChipTone::Error => Color::Error,
            ChipTone::Muted => Color::Muted,
            ChipTone::Info => Color::LogDebug,
        }
    }

    pub fn status(self, cx: &App) -> StatusColor {
        let theme = cx.theme();
        match self {
            ChipTone::Neutral => StatusColor {
                fg: theme.colors.text_secondary,
                background: theme.colors.surface_light,
                border: theme.colors.border_medium,
            },
            ChipTone::Accent => theme.status.accent,
            ChipTone::Success => theme.status.success,
            ChipTone::Warning => theme.status.warning,
            ChipTone::Error => theme.status.error,
            ChipTone::Muted => theme.status.muted,
            ChipTone::Info => theme.status.info,
        }
    }
}

impl From<Severity> for ChipTone {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Info => ChipTone::Info,
            Severity::Accent => ChipTone::Accent,
            Severity::Success => ChipTone::Success,
            Severity::Warning => ChipTone::Warning,
            Severity::Error => ChipTone::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChipSize {
    Small,
    #[default]
    Default,
}

pub struct Chip {
    id: Option<ElementId>,
    label: SharedString,
    icon: Option<IconName>,
    end_icon: Option<IconName>,
    tone: ChipTone,
    size: ChipSize,
    outlined: bool,
    selected: bool,
    uppercase: bool,
    truncate: bool,
    full_width: bool,
    on_click: Option<ClickHandler>,
    tooltip: Option<TooltipBuilder>,
}

crate::impl_into_element!(Chip);

impl Chip {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            label: label.into(),
            icon: None,
            end_icon: None,
            tone: ChipTone::default(),
            size: ChipSize::default(),
            outlined: false,
            selected: false,
            uppercase: false,
            truncate: false,
            full_width: false,
            on_click: None,
            tooltip: None,
        }
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<Option<IconName>>) -> Self {
        self.icon = icon.into();
        self
    }

    pub fn end_icon(mut self, icon: impl Into<Option<IconName>>) -> Self {
        self.end_icon = icon.into();
        self
    }

    pub fn tone(mut self, tone: ChipTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }

    pub fn outlined(mut self, outlined: bool) -> Self {
        self.outlined = outlined;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Rc::new(tooltip));
        self
    }

    pub fn tooltip_text(self, text: impl Into<SharedString>) -> Self {
        self.tooltip(Tooltip::text(text))
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let theme = cx.theme().clone();
        let status = self.tone.status(cx);
        let color = self.tone.color();
        let clickable = self.on_click.is_some();
        let id = self
            .id
            .unwrap_or_else(|| ElementId::Name(self.label.clone()));
        let (font, icon_size) = match self.size {
            ChipSize::Small => (sizes.font_xs, IconSize::XSmall),
            ChipSize::Default => (sizes.font_xs, IconSize::Small),
        };
        let background = if self.outlined && !self.selected {
            theme.colors.transparent
        } else if self.selected {
            status.fg.opacity(theme.alpha.hover)
        } else {
            status.background
        };
        let border = if self.selected {
            status.fg
        } else {
            status.border
        };
        let hover_background = status.fg.opacity(theme.alpha.line);
        let hover_border = status.fg.opacity(theme.alpha.vignette);
        let label = if self.uppercase {
            SharedString::from(self.label.to_uppercase())
        } else {
            self.label
        };
        let on_click = self.on_click;
        h_flex()
            .id(id)
            .flex_none()
            .max_w_full()
            .when(self.full_width, |this| this.w_full().justify_center())
            .gap(sizes.spacing_sm)
            .px(sizes.chip_padding_h)
            .py(sizes.chip_padding_v)
            .rounded(Sizes::CONTROL_RADIUS)
            .border_1()
            .map(|el| {
                if theme.id == crate::ThemeId::NierNew && !clickable && !self.selected {
                    el.border_color(theme.colors.transparent)
                } else {
                    el.border_color(border).bg(background)
                }
            })
            .text_size(font)
            .line_height(relative(LINE_HEIGHT_TIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(status.fg)
            .when_some(self.icon, |this, icon| {
                this.child(Icon::new(icon).size(icon_size).color(color))
            })
            .child(
                v_flex()
                    .min_w_0()
                    .when(self.truncate, |this| this.min_w_0().truncate())
                    .child(Text::new(label)),
            )
            .when_some(self.end_icon, |this, icon| {
                this.child(
                    Icon::new(icon)
                        .size(IconSize::XSmall)
                        .color(color)
                        .alpha(theme.alpha.glass),
                )
            })
            .when(clickable, |this| {
                this.cursor_pointer()
                    .keyboard_activate(cx)
                    .hover(move |style| style.bg(hover_background).border_color(hover_border))
                    .when_some(on_click, |this, handler| {
                        this.on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            handler(event, window, cx);
                        })
                    })
            })
            .when_some(self.tooltip, |this, tooltip| {
                this.tooltip(move |window, cx| tooltip(window, cx))
            })
    }
}

#[derive(Clone)]
pub struct CountBadge {
    count: usize,
    tone: ChipTone,
}

crate::impl_into_element!(CountBadge);

impl CountBadge {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            tone: ChipTone::Muted,
        }
    }

    pub fn tone(mut self, tone: ChipTone) -> Self {
        self.tone = tone;
        self
    }
}

impl RenderOnce for CountBadge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let status = self.tone.status(cx);
        h_flex()
            .flex_none()
            .justify_center()
            .min_w(sizes.icon_md)
            .px(sizes.chip_padding_h)
            .rounded(sizes.icon_md)
            .bg(status.background)
            .text_size(sizes.font_xs)
            .line_height(relative(LINE_HEIGHT_TIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(status.fg)
            .child(Text::new(self.count.to_string()))
    }
}

#[derive(Clone)]
pub struct Indicator {
    tone: ChipTone,
    pulse: bool,
    pulse_key: SharedString,
}

crate::impl_into_element!(Indicator);

impl Indicator {
    pub fn dot() -> Self {
        Self {
            tone: ChipTone::Accent,
            pulse: false,
            pulse_key: SharedString::new_static("yp-indicator"),
        }
    }

    pub fn tone(mut self, tone: ChipTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    pub fn pulse_key(mut self, key: impl Into<SharedString>) -> Self {
        self.pulse_key = key.into();
        self
    }
}

impl RenderOnce for Indicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let color = self.tone.status(cx).fg;
        let dot = Sizes::INDICATOR_SIZE;
        let halo = dot * 2.;
        let core = div().size(dot).rounded(dot).bg(color);
        let frame = div()
            .relative()
            .flex_none()
            .size(halo)
            .flex()
            .items_center()
            .justify_center();
        if !self.pulse {
            return frame.child(core).into_any_element();
        }
        frame
            .child(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .size(halo)
                    .rounded(halo)
                    .bg(color)
                    .with_animation(
                        ElementId::Name(self.pulse_key),
                        Animation::new(theme.motion.pulse).repeat(),
                        move |this, delta| {
                            let scale = 0.5 + 0.5 * delta;
                            let inset = halo * ((1. - scale) / 2.);
                            this.size(halo * scale)
                                .top(inset)
                                .left(inset)
                                .opacity(0.45 * (1. - delta))
                        },
                    ),
            )
            .child(core)
            .into_any_element()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApplyKind {
    Live,
    Restart,
}

#[derive(Clone)]
pub struct ApplyMarker {
    kind: ApplyKind,
    id: ElementId,
}

crate::impl_into_element!(ApplyMarker);

impl ApplyMarker {
    pub fn new(kind: ApplyKind) -> Self {
        Self {
            kind,
            id: ElementId::Name(SharedString::new_static("apply-marker")),
        }
    }

    pub fn live() -> Self {
        Self::new(ApplyKind::Live)
    }

    pub fn restart() -> Self {
        Self::new(ApplyKind::Restart)
    }

    pub fn for_restart_required(restart_required: bool) -> Self {
        if restart_required {
            Self::restart()
        } else {
            Self::live()
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl RenderOnce for ApplyMarker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let strings = cx.t();
        let (icon, color, tip) = match self.kind {
            ApplyKind::Live => (
                IconName::Bolt,
                Color::Success,
                strings.config_applies_live(),
            ),
            ApplyKind::Restart => (
                IconName::RestartAlt,
                Color::Warning,
                strings.config_applies_on_restart(),
            ),
        };
        div()
            .id(self.id)
            .flex_none()
            .child(
                Icon::new(icon)
                    .size(IconSize::Custom(px(14.)))
                    .color(color)
                    .alpha(0.85),
            )
            .tooltip(Tooltip::text(tip))
    }
}
