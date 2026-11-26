use gpui::{App, Hsla};

use crate::theme::ActiveTheme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeColors {
    pub background_primary: Hsla,
    pub background_secondary: Hsla,
    pub background_card: Hsla,
    pub surface_light: Hsla,
    pub surface_medium: Hsla,
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,
    pub text_on_dark: Hsla,
    pub accent_primary: Hsla,
    pub accent_secondary: Hsla,
    pub accent_highlight: Hsla,
    pub border_light: Hsla,
    pub border_medium: Hsla,
    pub success: Hsla,
    pub error: Hsla,
    pub warning: Hsla,
    pub button_background: Hsla,
    pub button_text: Hsla,
    pub input_background: Hsla,
    pub log_background: Hsla,
    pub log_header_background: Hsla,
    pub log_text: Hsla,
    pub log_debug: Hsla,
    pub shadow: Hsla,
    pub tooltip_background: Hsla,
    pub scrim: Hsla,
    pub ornament: Hsla,
    pub transparent: Hsla,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alpha {
    pub faint: f32,
    pub subtle: f32,
    pub hover: f32,
    pub line: f32,
    pub soft: f32,
    pub accent_border: f32,
    pub overlay: f32,
    pub half: f32,
    pub vignette: f32,
    pub glass: f32,
    pub heavy: f32,
    pub panel: f32,
    pub tooltip: f32,
}

impl Default for Alpha {
    fn default() -> Self {
        Self {
            faint: 0.08,
            subtle: 0.10,
            hover: 0.15,
            line: 0.20,
            soft: 0.30,
            accent_border: 0.35,
            overlay: 0.40,
            half: 0.50,
            vignette: 0.55,
            glass: 0.70,
            heavy: 0.87,
            panel: 0.95,
            tooltip: 0.96,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColor {
    pub fg: Hsla,
    pub background: Hsla,
    pub border: Hsla,
}

impl StatusColor {
    fn from_fg(fg: Hsla, alpha: &Alpha) -> Self {
        Self {
            fg,
            background: fg.opacity(alpha.faint),
            border: fg.opacity(alpha.soft),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColors {
    pub info: StatusColor,
    pub accent: StatusColor,
    pub success: StatusColor,
    pub warning: StatusColor,
    pub error: StatusColor,
    pub muted: StatusColor,
}

impl StatusColors {
    pub(crate) fn from_colors(colors: &ThemeColors, alpha: &Alpha) -> Self {
        Self {
            info: StatusColor::from_fg(colors.log_debug, alpha),
            accent: StatusColor::from_fg(colors.accent_primary, alpha),
            success: StatusColor::from_fg(colors.success, alpha),
            warning: StatusColor::from_fg(colors.warning, alpha),
            error: StatusColor::from_fg(colors.error, alpha),
            muted: StatusColor::from_fg(colors.text_muted, alpha),
        }
    }

    pub fn for_severity(&self, severity: Severity) -> StatusColor {
        match severity {
            Severity::Info => self.info,
            Severity::Accent => self.accent,
            Severity::Success => self.success,
            Severity::Warning => self.warning,
            Severity::Error => self.error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Severity {
    #[default]
    Info,
    Accent,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Color {
    #[default]
    Default,
    Secondary,
    Muted,
    Accent,
    AccentSecondary,
    Highlight,
    Success,
    Warning,
    Error,
    OnAccent,
    OnDark,
    Disabled,
    Placeholder,
    LogText,
    LogDebug,
}

impl Color {
    pub fn color(self, cx: &App) -> Hsla {
        let theme = cx.theme();
        let colors = &theme.colors;
        match self {
            Color::Default => colors.text_primary,
            Color::Secondary => colors.text_secondary,
            Color::Muted => colors.text_muted,
            Color::Accent => colors.accent_primary,
            Color::AccentSecondary => colors.accent_secondary,
            Color::Highlight => colors.accent_highlight,
            Color::Success => colors.success,
            Color::Warning => colors.warning,
            Color::Error => colors.error,
            Color::OnAccent => colors.button_text,
            Color::OnDark => colors.text_on_dark,
            Color::Disabled => colors.text_muted.opacity(theme.alpha.glass),
            Color::Placeholder if theme.id == super::ThemeId::NierNew => colors.text_muted,
            Color::Placeholder => colors.text_muted.opacity(theme.alpha.glass),
            Color::LogText => colors.log_text,
            Color::LogDebug => colors.log_debug,
        }
    }
}

impl From<Severity> for Color {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Info => Color::LogDebug,
            Severity::Accent => Color::Accent,
            Severity::Success => Color::Success,
            Severity::Warning => Color::Warning,
            Severity::Error => Color::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Surface {
    Primary,
    Secondary,
    #[default]
    Card,
    Light,
    Medium,
    Input,
}

impl Surface {
    pub fn color(self, cx: &App) -> Hsla {
        let colors = &cx.theme().colors;
        match self {
            Surface::Primary => colors.background_primary,
            Surface::Secondary => colors.background_secondary,
            Surface::Card => colors.background_card,
            Surface::Light => colors.surface_light,
            Surface::Medium => colors.surface_medium,
            Surface::Input => colors.input_background,
        }
    }
}
