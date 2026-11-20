mod binding;
mod errors;
#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;
mod fields;
mod page;
mod rules;
mod text_field;

pub mod inspector;
pub mod lodmod;
pub mod naiom;
pub mod nams;
pub mod textures;
pub mod yorha;

use gpui::{App, AppContext as _, KeyBinding};
use yp_state::prelude::*;
use yp_state::{OverlayId, OverlayRequest};

pub use binding::{Capture, Modifier, ModifierState, PointerButton};
pub use errors::{
    config_error_text, config_failure_text, format_error_text, inspector_error_text,
    probe_warning_text, settings_failure_text, toml_error_text,
};
pub use fields::{Control, FieldRow, Marker, control_for};
pub use inspector::{InspectorPage, ModelPreview};
pub use lodmod::LodModTab;
pub use naiom::NaiomTab;
pub use nams::NamsTab;
pub use page::set_cards_expanded;
pub use textures::model::TexturesModel;
pub use textures::TexturesTab;
pub use yorha::YorhaProtocolTab;

pub fn init(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "escape",
        yp_ui::Cancel,
        Some(inspector::KEY_CONTEXT),
    )]);
    textures::shared_model(cx);
    let registry = cx.registry_mut();
    registry
        .tabs
        .register(TabId::Nams, |window, cx| cx.new(|cx| NamsTab::new(window, cx)).into());
    registry.tabs.register(TabId::YorhaProtocol, |window, cx| {
        cx.new(|cx| YorhaProtocolTab::new(window, cx)).into()
    });
    registry.tabs.register(TabId::LodMod, |window, cx| {
        cx.new(|cx| LodModTab::new(window, cx)).into()
    });
    registry
        .tabs
        .register(TabId::Naiom, |window, cx| cx.new(|cx| NaiomTab::new(window, cx)).into());
    registry.tabs.register(TabId::Textures, |window, cx| {
        cx.new(|cx| TexturesTab::new(window, cx)).into()
    });
    registry
        .overlays
        .register(OverlayId::Inspector, |request, window, cx| {
            let archives = match request {
                OverlayRequest::Inspector { archives } => archives.clone(),
                OverlayRequest::Docs { .. } => Default::default(),
            };
            cx.new(|cx| InspectorPage::new(archives, window, cx)).into()
        });
    registry
        .flows
        .set_texture_pack_install(textures::install_packs);
    registry.flows.set_model_preview(|request, window, cx| {
        cx.new(|cx| ModelPreview::new(request.archives, window, cx))
            .into()
    });
}
