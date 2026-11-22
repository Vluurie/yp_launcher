pub mod cutscenes;
pub mod labels;
pub mod markdown;
pub mod mods;
pub mod text;
pub mod thirdparty;

#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;

mod layout;
mod notify;

use gpui::{App, AppContext as _};
use yp_state::prelude::*;

pub use cutscenes::CutscenesView;
pub use mods::ModsView;
pub use mods::install::{run as install_mods, run_loose_folder as install_loose_folder};
pub use thirdparty::ThirdPartyView;

pub fn init(cx: &mut App) {
    mods::install::init(cx);
    let registry = cx.registry_mut();
    registry.tabs.register(TabId::Mods, |window, cx| {
        cx.new(|cx| ModsView::new(window, cx)).into()
    });
    registry.tabs.register(TabId::Cutscenes, |window, cx| {
        cx.new(|cx| CutscenesView::new(window, cx)).into()
    });
    registry.tabs.register(TabId::ThirdParty, |window, cx| {
        cx.new(|cx| ThirdPartyView::new(window, cx)).into()
    });
}
