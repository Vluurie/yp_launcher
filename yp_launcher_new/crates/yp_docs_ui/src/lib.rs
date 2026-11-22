#[cfg(feature = "tools")]
pub mod builders;
pub mod dialogs;
pub mod docs;
#[cfg(feature = "tools")]
pub mod errors;
pub mod markdown;
pub mod syntax;
#[cfg(feature = "tools")]
pub mod tools;
mod util;

#[cfg(all(test, feature = "tools"))]
mod tests;

use gpui::{App, AppContext as _, Global};
use yp_state::prelude::*;
use yp_state::OverlayId;
#[cfg(feature = "tools")]
use yp_state::TabId;

pub use docs::{DocsConfig, DocsOverlay};
#[cfg(feature = "tools")]
pub use tools::{ToolCard, ToolsView};

pub fn init(cx: &mut App) {
    init_docs(cx);
    #[cfg(feature = "tools")]
    init_tools(cx);
}

struct SharedBindings;

impl Global for SharedBindings {}

fn init_shared(cx: &mut App) {
    if cx.try_global::<SharedBindings>().is_some() {
        return;
    }
    cx.set_global(SharedBindings);
    markdown::rich_text::init(cx);
}

pub fn init_docs(cx: &mut App) {
    init_shared(cx);
    dialogs::markdown_editor::init(cx);
    if cx.try_global::<DocsConfig>().is_none() {
        let config = DocsConfig::detect(cx);
        cx.set_global(config);
    }
    let registry = cx.registry_mut();
    registry.docs_library = cfg!(feature = "library");
    registry
        .overlays
        .register(OverlayId::Docs, |request, window, cx| {
            cx.new(|cx| DocsOverlay::from_request(request, window, cx))
                .into()
        });
}

#[cfg(feature = "tools")]
pub fn init_tools(cx: &mut App) {
    init_shared(cx);
    cx.registry_mut().tabs.register(TabId::Tools, |window, cx| {
        cx.new(|cx| ToolsView::new(window, cx)).into()
    });
}
