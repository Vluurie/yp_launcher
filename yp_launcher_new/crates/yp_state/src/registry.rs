use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gpui::{AnyView, App, Global, Task, Window};
use yp_inspector::ArchivePaths;
use yp_mods::InstallFailure;

use crate::tabs::{OverlayId, TabId};

pub type TabFactory = Rc<dyn Fn(&mut Window, &mut App) -> AnyView>;

#[derive(Default)]
pub struct TabRegistry {
    factories: HashMap<TabId, TabFactory>,
}

impl TabRegistry {
    pub fn register(
        &mut self,
        tab: TabId,
        factory: impl Fn(&mut Window, &mut App) -> AnyView + 'static,
    ) {
        self.factories.insert(tab, Rc::new(factory));
    }

    pub fn contains(&self, tab: TabId) -> bool {
        self.factories.contains_key(&tab)
    }

    pub fn factory(&self, tab: TabId) -> Option<TabFactory> {
        self.factories.get(&tab).cloned()
    }

    pub fn registered(&self) -> Vec<TabId> {
        let mut tabs: Vec<TabId> = self.factories.keys().copied().collect();
        tabs.sort();
        tabs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayRequest {
    Docs { page: Option<String> },
    Inspector { archives: ArchivePaths },
}

impl OverlayRequest {
    pub fn id(&self) -> OverlayId {
        match self {
            OverlayRequest::Docs { .. } => OverlayId::Docs,
            OverlayRequest::Inspector { .. } => OverlayId::Inspector,
        }
    }
}

pub type OverlayFactory = Rc<dyn Fn(&OverlayRequest, &mut Window, &mut App) -> AnyView>;

#[derive(Default)]
pub struct OverlayRegistry {
    factories: HashMap<OverlayId, OverlayFactory>,
}

impl OverlayRegistry {
    pub fn register(
        &mut self,
        overlay: OverlayId,
        factory: impl Fn(&OverlayRequest, &mut Window, &mut App) -> AnyView + 'static,
    ) {
        self.factories.insert(overlay, Rc::new(factory));
    }

    pub fn contains(&self, overlay: OverlayId) -> bool {
        self.factories.contains_key(&overlay)
    }

    pub fn factory(&self, overlay: OverlayId) -> Option<OverlayFactory> {
        self.factories.get(&overlay).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModInstallMode {
    #[default]
    Guided,
    Bulk,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModInstallRequest {
    pub sources: Vec<PathBuf>,
    pub mode: ModInstallMode,
    pub texture_packs: bool,
}

impl ModInstallRequest {
    pub fn guided(sources: Vec<PathBuf>) -> Self {
        Self {
            sources,
            mode: ModInstallMode::Guided,
            texture_packs: false,
        }
    }

    pub fn bulk(sources: Vec<PathBuf>) -> Self {
        Self {
            sources,
            mode: ModInstallMode::Bulk,
            texture_packs: false,
        }
    }

    pub fn with_texture_packs(mut self) -> Self {
        self.texture_packs = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModInstallReport {
    pub installed: Vec<String>,
    pub texture_packs: Vec<String>,
    pub failed: Vec<(PathBuf, InstallFailure)>,
    pub cancelled: bool,
}

impl ModInstallReport {
    pub fn any_installed(&self) -> bool {
        !self.installed.is_empty() || !self.texture_packs.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModelPreviewRequest {
    pub archives: ArchivePaths,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePackOutcome {
    Installed,
    Cancelled,
    Failed,
}

pub type ModInstallFlow =
    Rc<dyn Fn(ModInstallRequest, &mut Window, &mut App) -> Task<ModInstallReport>>;
pub type TexturePackInstallFlow =
    Rc<dyn Fn(Vec<PathBuf>, &mut Window, &mut App) -> Task<TexturePackOutcome>>;
pub type ModelPreviewFlow = Rc<dyn Fn(ModelPreviewRequest, &mut Window, &mut App) -> AnyView>;

#[derive(Default)]
pub struct Flows {
    mod_install: Option<ModInstallFlow>,
    texture_pack_install: Option<TexturePackInstallFlow>,
    model_preview: Option<ModelPreviewFlow>,
}

impl Flows {
    pub fn set_mod_install(
        &mut self,
        flow: impl Fn(ModInstallRequest, &mut Window, &mut App) -> Task<ModInstallReport> + 'static,
    ) {
        self.mod_install = Some(Rc::new(flow));
    }

    pub fn set_texture_pack_install(
        &mut self,
        flow: impl Fn(Vec<PathBuf>, &mut Window, &mut App) -> Task<TexturePackOutcome> + 'static,
    ) {
        self.texture_pack_install = Some(Rc::new(flow));
    }

    pub fn set_model_preview(
        &mut self,
        flow: impl Fn(ModelPreviewRequest, &mut Window, &mut App) -> AnyView + 'static,
    ) {
        self.model_preview = Some(Rc::new(flow));
    }

    pub fn mod_install(&self) -> Option<ModInstallFlow> {
        self.mod_install.clone()
    }

    pub fn texture_pack_install(&self) -> Option<TexturePackInstallFlow> {
        self.texture_pack_install.clone()
    }

    pub fn model_preview(&self) -> Option<ModelPreviewFlow> {
        self.model_preview.clone()
    }
}

#[derive(Default)]
pub struct Registry {
    pub tabs: TabRegistry,
    pub overlays: OverlayRegistry,
    pub docs_library: bool,
    pub flows: Flows,
}

impl Global for Registry {}

impl Registry {
    pub fn build_tab(tab: TabId, window: &mut Window, cx: &mut App) -> Option<AnyView> {
        let factory = cx.try_global::<Registry>()?.tabs.factory(tab)?;
        Some(factory(window, cx))
    }

    pub fn build_overlay(
        request: &OverlayRequest,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyView> {
        let factory = cx
            .try_global::<Registry>()?
            .overlays
            .factory(request.id())?;
        Some(factory(request, window, cx))
    }

    pub fn install_mods(
        request: ModInstallRequest,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Task<ModInstallReport>> {
        let flow = cx.try_global::<Registry>()?.flows.mod_install()?;
        Some(flow(request, window, cx))
    }

    pub fn install_texture_pack(
        sources: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Task<TexturePackOutcome>> {
        let flow = cx.try_global::<Registry>()?.flows.texture_pack_install()?;
        Some(flow(sources, window, cx))
    }

    pub fn model_preview(
        request: ModelPreviewRequest,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyView> {
        let flow = cx.try_global::<Registry>()?.flows.model_preview()?;
        Some(flow(request, window, cx))
    }
}
