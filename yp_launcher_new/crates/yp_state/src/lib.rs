mod cell;
mod game_font;
mod health;
#[cfg(test)]
mod tests;
mod watch;
mod wiring;

pub mod actions;
pub mod busy;
pub mod config;
pub mod detection;
pub mod env;
pub mod extraction;
pub mod inspector;
pub mod launch_options;
pub mod logs;
pub mod mods;
pub mod navigation;
pub mod notifications;
pub mod prefs;
pub mod registry;
pub mod session;
pub mod tabs;
pub mod thirdparty;

use gpui::AppContext as _;
use gpui::{App, Entity, Global, Task};
use yp_core::runtime::RuntimeStatus;

pub use actions::{
    CloseLogs, OpenDiagnostics, OpenDocs, OpenLogs, OpenTab, OpenThreeDInspector, Play,
    RefreshDetections, SetLocale, Stop, ToggleTheme, register_default_handlers,
};
pub use busy::{BusyEvent, BusyHandle, BusyScope, BusyTracker};
pub use config::{CliFactory, ConfigEvent, ConfigFailure, ConfigOp, ConfigStore};
pub use detection::{DetectionEvent, DetectionSnapshot, DetectionStore, TextureChip};
pub use env::AppEnv;
pub use extraction::{archive_failure_key, extraction_error_key};
pub use inspector::{InspectorEvent, InspectorService, ModelEvent, ModelSession, ModelStatus};
pub use launch_options::{LaunchOptions, LaunchOptionsEvent};
pub use logs::{LogEvent, LogStore};
pub use mods::{
    DefaultModsStore, DisabledModsStore, ModDataEvent, ModLoadOrderStore, ModNamesStore,
    ModProfilesStore, ModsEvent, ModsStore, ProfilesEvent, ProgressReceiver, ProgressSender,
    progress_channel,
};
pub use navigation::{
    CloseGuard, Navigation, NavigationBlock, NavigationEvent, TabRequest, TabRequestOutcome,
};
pub use notifications::{Notice, NoticeKind, NoticeSeverity, Notifications, NotificationsEvent};
pub use prefs::Prefs;
pub use registry::{
    Flows, ModInstallFlow, ModInstallMode, ModInstallReport, ModInstallRequest, ModelPreviewFlow,
    ModelPreviewRequest, OverlayFactory, OverlayRegistry, OverlayRequest, Registry, TabFactory,
    TabRegistry, TexturePackInstallFlow, TexturePackOutcome,
};
pub use session::{
    DirectoryChange, GameError, GameSession, LaunchRequest, LaunchStatus, PlayState,
    SelectionCheck, SessionError, SessionEvent,
};
pub use tabs::{OverlayId, SidebarSection, TabId};
pub use thirdparty::{ThirdPartyError, ThirdPartyEvent, ThirdPartyNotice, ThirdPartyStore};

pub mod prelude {
    pub use crate::actions::*;
    pub use crate::{AppEnv, BusyScope, Prefs, Registry, StateExt, Stores, TabId};
}

#[derive(Clone)]
pub struct Stores {
    pub session: Entity<GameSession>,
    pub navigation: Entity<Navigation>,
    pub busy: Entity<BusyTracker>,
    pub notifications: Entity<Notifications>,
    pub launch_options: Entity<LaunchOptions>,
    pub detection: Entity<DetectionStore>,
    pub config: Entity<ConfigStore>,
    pub mods: Entity<ModsStore>,
    pub profiles: Entity<ModProfilesStore>,
    pub disabled_mods: Entity<DisabledModsStore>,
    pub default_mods: Entity<DefaultModsStore>,
    pub load_order: Entity<ModLoadOrderStore>,
    pub mod_names: Entity<ModNamesStore>,
    pub logs: Entity<LogStore>,
    pub thirdparty: Entity<ThirdPartyStore>,
    pub inspector: Entity<InspectorService>,
}

impl Global for Stores {}

pub trait StateExt {
    fn stores(&self) -> &Stores;
    fn env(&self) -> &AppEnv;
    fn prefs(&self) -> &Prefs;
    fn registry(&self) -> &Registry;
    fn registry_mut(&mut self) -> &mut Registry;
}

impl StateExt for App {
    fn stores(&self) -> &Stores {
        self.global::<Stores>()
    }

    fn env(&self) -> &AppEnv {
        self.global::<AppEnv>()
    }

    fn prefs(&self) -> &Prefs {
        self.global::<Prefs>()
    }

    fn registry(&self) -> &Registry {
        self.global::<Registry>()
    }

    fn registry_mut(&mut self) -> &mut Registry {
        self.default_global::<Registry>()
    }
}

pub fn init(cx: &mut App) {
    let env = match AppEnv::detect() {
        Ok(env) => env,
        Err(err) => {
            log::error!("could not open the launcher data folder: {err}");
            AppEnv::fallback()
        }
    };
    init_with(env, cx);
}

pub fn init_with(env: AppEnv, cx: &mut App) {
    cx.set_global(Prefs::new(env.settings.clone(), cx.background_executor()));
    prefs::bind_ui(&env.host, cx);
    cx.on_app_quit(|cx| {
        let flushed = cx.global::<Prefs>().flush();
        async move {
            let _ = flushed.await;
        }
    })
    .detach();
    cx.default_global::<Registry>();
    game_font::restore(&env, cx);
    cx.set_global(env.clone());

    let busy = cx.new(BusyTracker::new);
    let config = cx.new(|cx| ConfigStore::new(&env, cx));
    let navigation = cx.new(|cx| Navigation::new(busy.clone(), config.clone(), cx));
    let stores = Stores {
        session: cx.new(|cx| GameSession::new(env.clone(), cx)),
        navigation,
        busy,
        notifications: cx.new(|cx| Notifications::new(env.clone(), cx)),
        launch_options: cx.new(|cx| LaunchOptions::new(env.clone(), cx)),
        detection: cx.new(DetectionStore::new),
        config,
        mods: cx.new(|cx| ModsStore::new(env.clone(), cx)),
        profiles: cx.new(ModProfilesStore::new),
        disabled_mods: cx.new(DisabledModsStore::new),
        default_mods: cx.new(DefaultModsStore::new),
        load_order: cx.new(ModLoadOrderStore::new),
        mod_names: cx.new(ModNamesStore::new),
        logs: cx.new(|cx| LogStore::new(&env, cx)),
        thirdparty: cx.new(|cx| ThirdPartyStore::new(env, cx)),
        inspector: cx.new(InspectorService::new),
    };
    wiring::wire(&stores, cx);
    cx.set_global(stores);
}

pub fn startup(cx: &mut App) -> Task<yp_core::Result<RuntimeStatus>> {
    let env = cx.env().clone();
    let session = cx.stores().session.clone();
    let setup = cx.background_spawn(async move { env.ensure_runtime() });
    cx.spawn(async move |cx| {
        let status = setup.await;
        if let Err(err) = &status {
            log::error!("could not prepare the bundled runtime: {err}");
        }
        session.update(cx, |session, cx| session.restore(cx));
        status
    })
}
