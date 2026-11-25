mod backdrop;
mod control;
mod dialog;
mod dropdown;
mod field;
mod frame;
pub use field::{FieldEvent, MenuFieldState, MenuTextField};
mod help;
mod list;
mod loading;
mod model_view;
pub use model_view::{ModelViewer, ViewerAction};
mod panel;
mod transfer;
pub use transfer::{MenuDropZone, MenuProgress};
mod scrollbar;
mod slider;
pub use scrollbar::MenuScrollbar;
mod sprite;
pub use slider::MenuSlider;
mod text;
use crate::Surface;
pub use backdrop::MenuBackdrop;
pub(crate) use backdrop::borders;
pub use control::{DialogButton, MenuItemRow, MenuRow, MenuTab, PulldownOption, SettingRow};
pub use dialog::ConfirmDialog;
pub use dropdown::{MenuActionMenu, MenuChoice, MenuDropdown, MenuToggle};
pub use frame::MenuFrame;
use gpui::{IntoElement, div, prelude::*, px};
pub use help::{ButtonPrompt, HelpBar};
pub use list::MenuList;
pub use loading::LoadingIcon;
pub use panel::MenuPanel;
pub use sprite::{NineSlice, Sprite};
pub use text::{GameText, MenuTitle};
fn element(surface: Surface) -> impl IntoElement {
    div()
        .w(px(surface.width))
        .h(px(surface.height))
        .child(surface)
}

fn modal_backdrop(opacity: f32) -> impl IntoElement {
    div()
        .size_full()
        .bg(gpui::rgba((51. * opacity.clamp(0., 1.)).round() as u32))
}
