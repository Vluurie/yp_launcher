use std::path::PathBuf;

use yp_core::{GamePaths, HostEnv};

use crate::error::Result;
use crate::flags::{self, LoadingFlag};
use crate::game_mods::GameModsRuntime;
use crate::graphic_mods::GraphicModsRuntime;
use crate::migoto::MigotoRuntime;
use crate::model::{
    Classification, InstallOutcome, RuntimePresence, RuntimeStatus, ThirdPartyRuntime, ThirdPartyUpdateInfo,
};
use crate::reshade::ReShadeRuntime;

pub trait GraphicsRuntime: Send + Sync {
    fn runtime(&self) -> ThirdPartyRuntime;

    fn loading_flag(&self) -> LoadingFlag {
        self.runtime().loading_flag()
    }

    fn install_dir(&self, game: &GamePaths) -> PathBuf;

    fn can_install(&self, classification: &Classification) -> bool;

    fn install(&self, game: &GamePaths, classification: &Classification) -> Result<InstallOutcome>;

    fn would_update(&self, _game: &GamePaths, _classification: &Classification) -> Option<ThirdPartyUpdateInfo> {
        None
    }

    fn repair(&self, game: &GamePaths) -> Result<()>;

    fn import_from_game_root(&self, game: &GamePaths) -> Result<bool>;

    fn status(&self, game: &GamePaths) -> RuntimeStatus;

    fn remove(&self, game: &GamePaths) -> Result<()>;

    fn set_enabled(&self, game: &GamePaths, enabled: bool) -> Result<()> {
        flags::set_disabled(game, self.loading_flag(), !enabled)
    }

    fn is_enabled(&self, game: &GamePaths) -> bool {
        !flags::is_disabled(game, self.loading_flag())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThirdPartyStatuses {
    pub reshade: RuntimeStatus,
    pub migoto: RuntimeStatus,
    pub game_mods: RuntimeStatus,
    pub graphic_mods: RuntimeStatus,
}

impl ThirdPartyStatuses {
    pub fn get(&self, which: ThirdPartyRuntime) -> &RuntimeStatus {
        match which {
            ThirdPartyRuntime::ReShade => &self.reshade,
            ThirdPartyRuntime::Migoto => &self.migoto,
            ThirdPartyRuntime::GameMods => &self.game_mods,
            ThirdPartyRuntime::GraphicMods => &self.graphic_mods,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Runtimes {
    pub reshade: ReShadeRuntime,
    pub migoto: MigotoRuntime,
    pub game_mods: GameModsRuntime,
    pub graphic_mods: GraphicModsRuntime,
}

impl Runtimes {
    pub fn new(host: &HostEnv) -> Self {
        Self {
            reshade: ReShadeRuntime::new(host),
            migoto: MigotoRuntime::new(host),
            game_mods: GameModsRuntime,
            graphic_mods: GraphicModsRuntime,
        }
    }

    pub fn get(&self, which: ThirdPartyRuntime) -> &dyn GraphicsRuntime {
        match which {
            ThirdPartyRuntime::ReShade => &self.reshade,
            ThirdPartyRuntime::Migoto => &self.migoto,
            ThirdPartyRuntime::GameMods => &self.game_mods,
            ThirdPartyRuntime::GraphicMods => &self.graphic_mods,
        }
    }

    pub fn installer_for(&self, classification: &Classification) -> Option<&dyn GraphicsRuntime> {
        ThirdPartyRuntime::ALL
            .into_iter()
            .map(|which| self.get(which))
            .find(|runtime| runtime.can_install(classification))
    }

    pub fn found_in_game_root(&self, game: &GamePaths, which: ThirdPartyRuntime) -> bool {
        match which {
            ThirdPartyRuntime::ReShade => self.reshade.find_game_root_dll(game).is_some(),
            ThirdPartyRuntime::Migoto => self.migoto.find_game_root_dll(game).is_some(),
            ThirdPartyRuntime::GameMods | ThirdPartyRuntime::GraphicMods => false,
        }
    }

    pub fn presence(&self, game: &GamePaths, which: ThirdPartyRuntime, status: &RuntimeStatus) -> RuntimePresence {
        RuntimePresence::of(status.installed, !status.installed && self.found_in_game_root(game, which))
    }

    pub fn status_all(&self, game: &GamePaths) -> ThirdPartyStatuses {
        ThirdPartyStatuses {
            reshade: self.reshade.status(game),
            migoto: self.migoto.status(game),
            game_mods: self.game_mods.status(game),
            graphic_mods: self.graphic_mods.status(game),
        }
    }
}

pub fn status_all(game: &GamePaths, host: &HostEnv) -> ThirdPartyStatuses {
    Runtimes::new(host).status_all(game)
}
