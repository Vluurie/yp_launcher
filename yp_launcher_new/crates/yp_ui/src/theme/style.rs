use gpui::{BoxShadow, Hsla, point, px};

use crate::theme::{Alpha, ThemeColors, ThemeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NavHover {
    #[default]
    Tint,
    InkSweep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonHover {
    #[default]
    Tint,
    Invert,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeStyle {
    pub nav_hover: NavHover,
    pub button_hover: ButtonHover,
    pub focus_ring: Hsla,
}

impl ThemeStyle {
    pub(crate) fn for_theme(id: ThemeId, colors: &ThemeColors, alpha: &Alpha) -> Self {
        match id {
            ThemeId::Dark => Self {
                nav_hover: NavHover::Tint,
                button_hover: ButtonHover::Tint,
                focus_ring: colors.accent_primary.opacity(alpha.vignette),
            },
            ThemeId::Nier | ThemeId::NierNew => Self {
                nav_hover: NavHover::InkSweep,
                button_hover: ButtonHover::Invert,
                focus_ring: if id == ThemeId::NierNew {
                    colors.accent_primary
                } else {
                    colors.accent_primary.opacity(alpha.vignette)
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Elevation {
    #[default]
    None,
    Card,
    Popover,
    Modal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeElevation {
    pub card: Vec<BoxShadow>,
    pub popover: Vec<BoxShadow>,
    pub modal: Vec<BoxShadow>,
}

impl ThemeElevation {
    pub(crate) fn from_colors(colors: &ThemeColors, alpha: &Alpha) -> Self {
        Self {
            card: vec![BoxShadow {
                color: colors.shadow,
                offset: point(px(0.), px(2.)),
                blur_radius: px(4.),
                spread_radius: px(0.),
                inset: false,
            }],
            popover: vec![BoxShadow {
                color: colors.scrim.opacity(alpha.vignette),
                offset: point(px(0.), px(3.)),
                blur_radius: px(12.),
                spread_radius: px(0.),
                inset: false,
            }],
            modal: vec![BoxShadow {
                color: colors.scrim.opacity(alpha.soft),
                offset: point(px(0.), px(2.)),
                blur_radius: px(8.),
                spread_radius: px(0.),
                inset: false,
            }],
        }
    }

    pub fn shadows(&self, elevation: Elevation) -> Vec<BoxShadow> {
        match elevation {
            Elevation::None => Vec::new(),
            Elevation::Card => self.card.clone(),
            Elevation::Popover => self.popover.clone(),
            Elevation::Modal => self.modal.clone(),
        }
    }
}
