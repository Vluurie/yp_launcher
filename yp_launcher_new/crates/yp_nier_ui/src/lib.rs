mod animation;
mod atlas;
mod capture;
mod components;
mod draw;
mod effects;
mod filter_layer;
mod grid_noise;
mod menu;
mod motion;
mod screen;
#[cfg(feature = "uid-debug")]
mod paint;
mod static_noise;
mod text_reveal;
#[cfg(feature = "uid-debug")]
pub mod uid;

use gpui::App;
use image::GrayImage;

pub use atlas::{INK, LIGHT, PANEL, PAPER, Theme, Tint, WHITE};
pub use components::{
    ButtonPrompt, ConfirmDialog, DialogButton, FieldEvent, GameText, HelpBar, LoadingIcon,
    MenuActionMenu, MenuBackdrop, MenuChoice, MenuDropZone, MenuDropdown, MenuFieldState,
    MenuFrame, MenuItemRow, MenuList, MenuPanel, MenuProgress, MenuRow, MenuScrollbar, MenuSlider, MenuSliderTrack,
    MenuTab, MenuTextField, MenuTitle, MenuToggle, NineSlice, PulldownOption, SettingRow, Sprite,
};
pub use components::{MenuWorkspace, ModelViewer, PageReveal, ViewerAction, file_drop_highlight, menu_rule, workspace_drop_highlight};
pub use screen::{menu_popup_overlay, menu_screen_overlay};
pub use draw::{Rect, Surface};
pub use effects::Lens;
pub use filter_layer::EffectLayer;
pub use grid_noise::GridNoise;
pub use menu::{Focus, MenuAction, MenuEntry, MenuInput, MenuPage, PauseMenu};
pub use motion::ReducedMotion;
pub use text_reveal::{PageTextFrame, page_text, page_text_frame, page_text_scope};

pub fn use_bitmap_font(
    coverage: GrayImage,
    glyphs: impl IntoIterator<Item = ((u16, u16), [u16; 4], f32)>,
    kerning: impl IntoIterator<Item = ([u16; 3], i16)>,
    cx: &mut App,
) {
    Theme::bundled().use_bitmap_font(coverage, glyphs, kerning);
    draw::forget_text(cx);
    cx.refresh_windows();
}
