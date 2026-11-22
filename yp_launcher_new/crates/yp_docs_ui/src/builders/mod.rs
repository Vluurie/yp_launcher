pub mod first_mod;
pub mod forms;
pub mod item;
pub mod mesh_tuner;
pub mod model_view;
pub mod outfit;
pub mod weapon;
pub(crate) mod widgets;

#[cfg(test)]
pub(crate) mod tests;

use gpui::{AnyView, App, Window};
use yp_i18n::Strings;
use yp_ui::IconName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ToolKind {
    FirstMod,
    ItemBuilder,
    OutfitBuilder,
    WeaponBuilder,
}

impl ToolKind {
    pub const ALL: [ToolKind; 4] = [
        ToolKind::FirstMod,
        ToolKind::ItemBuilder,
        ToolKind::OutfitBuilder,
        ToolKind::WeaponBuilder,
    ];

    pub fn from_marker(id: &str) -> Option<ToolKind> {
        ToolKind::ALL.into_iter().find(|kind| kind.marker_id() == id)
    }

    pub const fn marker_id(self) -> &'static str {
        match self {
            ToolKind::FirstMod => "first-mod",
            ToolKind::ItemBuilder => "item-builder",
            ToolKind::OutfitBuilder => "outfit-builder",
            ToolKind::WeaponBuilder => "weapon-builder",
        }
    }

    pub const fn icon(self) -> IconName {
        match self {
            ToolKind::FirstMod => IconName::PlayLessonOutlined,
            ToolKind::ItemBuilder => IconName::AddBoxOutlined,
            ToolKind::OutfitBuilder => IconName::CheckroomOutlined,
            ToolKind::WeaponBuilder => IconName::HardwareOutlined,
        }
    }

    pub fn title(self, strings: &Strings) -> &'static str {
        match self {
            ToolKind::FirstMod => strings.first_mod_title(),
            ToolKind::ItemBuilder => strings.item_builder_title(),
            ToolKind::OutfitBuilder => strings.outfit_builder_title(),
            ToolKind::WeaponBuilder => strings.weapon_builder_title(),
        }
    }

    pub fn subtitle(self, strings: &Strings) -> &'static str {
        match self {
            ToolKind::FirstMod => strings.first_mod_body(),
            ToolKind::ItemBuilder => strings.item_builder_body(),
            ToolKind::OutfitBuilder => strings.outfit_builder_desc(),
            ToolKind::WeaponBuilder => strings.weapon_builder_desc(),
        }
    }

    pub const fn initially_open(self) -> bool {
        matches!(self, ToolKind::FirstMod)
    }

    pub fn build(self, window: &mut Window, cx: &mut App) -> AnyView {
        match self {
            ToolKind::FirstMod => first_mod::build(window, cx),
            ToolKind::ItemBuilder => item::build(window, cx),
            ToolKind::OutfitBuilder => outfit::build(window, cx),
            ToolKind::WeaponBuilder => weapon::build(window, cx),
        }
    }
}
