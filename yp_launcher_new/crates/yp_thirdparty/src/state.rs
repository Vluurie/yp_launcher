use std::path::{Path, PathBuf};

use yp_core::{AppPaths, GamePaths, HostEnv};

use crate::classify;
use crate::error::Result;
use crate::migration::{self, MigrationResult};
use crate::model::{
    Classification, GameModSettings, InstallOutcome, MigotoConfig, ReShadeConfig, RuntimePresence, ThirdPartyRuntime,
    ThirdPartyUpdateInfo,
};
use crate::{paths, reshade};
use crate::runtime::{GraphicsRuntime, Runtimes, ThirdPartyStatuses};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThirdPartyNotice {
    Migrated { to: PathBuf },
}

impl ThirdPartyNotice {
    pub const MIGRATED_ID: &'static str = "thirdparty_migrated";

    pub fn id(&self) -> &'static str {
        match self {
            ThirdPartyNotice::Migrated { .. } => Self::MIGRATED_ID,
        }
    }

    pub fn from_migration(result: &MigrationResult) -> Vec<ThirdPartyNotice> {
        if result.moved_files > 0 {
            vec![ThirdPartyNotice::Migrated { to: result.to.clone() }]
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WatchDepth {
    TopLevel,
    Recursive,
}

#[derive(Debug, Clone)]
pub struct ThirdPartyState {
    runtimes: Runtimes,
    legacy_root: PathBuf,
    game: Option<GamePaths>,
    statuses: ThirdPartyStatuses,
    game_root_copies: GameRootCopies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct GameRootCopies {
    reshade: bool,
    migoto: bool,
}

impl GameRootCopies {
    fn scan(runtimes: &Runtimes, game: &GamePaths, statuses: &ThirdPartyStatuses) -> Self {
        let found = |which: ThirdPartyRuntime| !statuses.get(which).installed && runtimes.found_in_game_root(game, which);
        Self {
            reshade: found(ThirdPartyRuntime::ReShade),
            migoto: found(ThirdPartyRuntime::Migoto),
        }
    }

    fn has(&self, which: ThirdPartyRuntime) -> bool {
        match which {
            ThirdPartyRuntime::ReShade => self.reshade,
            ThirdPartyRuntime::Migoto => self.migoto,
            ThirdPartyRuntime::GameMods | ThirdPartyRuntime::GraphicMods => false,
        }
    }
}

impl ThirdPartyState {
    pub fn new(host: &HostEnv, app: &AppPaths) -> Self {
        Self::with_legacy_root(host, app.launcher_thirdparty_dir())
    }

    pub fn with_legacy_root(host: &HostEnv, legacy_root: PathBuf) -> Self {
        Self {
            runtimes: Runtimes::new(host),
            legacy_root,
            game: None,
            statuses: ThirdPartyStatuses::default(),
            game_root_copies: GameRootCopies::default(),
        }
    }

    pub fn runtimes(&self) -> &Runtimes {
        &self.runtimes
    }

    pub fn game(&self) -> Option<&GamePaths> {
        self.game.as_ref()
    }

    pub fn set_game(&mut self, game: Option<GamePaths>) {
        if self.game != game {
            self.statuses = ThirdPartyStatuses::default();
            self.game_root_copies = GameRootCopies::default();
        }
        self.game = game;
    }

    pub fn statuses(&self) -> &ThirdPartyStatuses {
        &self.statuses
    }

    pub fn presence(&self, which: ThirdPartyRuntime) -> RuntimePresence {
        RuntimePresence::of(self.statuses.get(which).installed, self.game_root_copies.has(which))
    }

    pub fn existing_install_dir(&self, which: ThirdPartyRuntime) -> Option<PathBuf> {
        let game = self.game.as_ref()?;
        Some(self.runtimes.get(which).install_dir(game)).filter(|dir| dir.is_dir())
    }

    pub fn existing_shaders_dir(&self) -> Option<PathBuf> {
        let game = self.game.as_ref()?;
        Some(reshade::shaders_dir(game)).filter(|dir| dir.is_dir())
    }

    pub fn watch_paths(&self) -> Vec<(PathBuf, WatchDepth)> {
        self.game
            .iter()
            .filter(|game| game.root().is_dir())
            .map(|game| (game.root().to_path_buf(), WatchDepth::TopLevel))
            .collect()
    }

    pub fn is_relevant_change(path: &Path) -> bool {
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().to_lowercase().ends_with(".dll"))
    }

    pub fn refresh(&mut self, game_running: bool) -> Result<Vec<ThirdPartyNotice>> {
        let Some(game) = self.game.clone() else {
            return Ok(Vec::new());
        };
        let notices = self.migrate_legacy(game_running);
        self.auto_import_from_game_root(&game)?;
        self.runtimes.reshade.repair(&game)?;
        self.reload();
        Ok(notices)
    }

    pub fn reload(&mut self) {
        if let Some(game) = &self.game {
            self.statuses = self.runtimes.status_all(game);
            self.game_root_copies = GameRootCopies::scan(&self.runtimes, game, &self.statuses);
        }
    }

    pub fn on_game_root_changed(&mut self, changed: &[PathBuf]) -> Result<bool> {
        if !changed.iter().any(|path| Self::is_relevant_change(path)) {
            return Ok(false);
        }
        let Some(game) = self.game.clone() else {
            return Ok(false);
        };
        let imported = self.auto_import_from_game_root(&game)?;
        if imported {
            self.reload();
            return Ok(true);
        }
        let copies = GameRootCopies::scan(&self.runtimes, &game, &self.statuses);
        let changed = copies != self.game_root_copies;
        self.game_root_copies = copies;
        Ok(changed)
    }

    pub fn migrate_legacy(&self, game_running: bool) -> Vec<ThirdPartyNotice> {
        match &self.game {
            Some(game) => ThirdPartyNotice::from_migration(&migration::migrate_from(game, &self.legacy_root, game_running)),
            None => Vec::new(),
        }
    }

    fn auto_import_from_game_root(&self, game: &GamePaths) -> Result<bool> {
        let mut changed = false;
        let reshade = &self.runtimes.reshade;
        if !reshade.status(game).installed && reshade.is_enabled(game) && reshade.find_game_root_dll(game).is_some() {
            changed |= reshade.import_from_game_root(game)?;
        }
        let migoto = &self.runtimes.migoto;
        if !migoto.status(game).installed && migoto.is_enabled(game) && migoto.find_game_root_dll(game).is_some() {
            changed |= migoto.import_from_game_root(game)?;
        }
        Ok(changed)
    }

    pub fn classify(&self, extracted_root: &Path) -> Classification {
        classify::classify(extracted_root)
    }

    pub fn would_update(&self, classification: &Classification) -> Option<ThirdPartyUpdateInfo> {
        let game = self.game.as_ref()?;
        self.runtimes.installer_for(classification)?.would_update(game, classification)
    }

    pub fn install(&mut self, classification: &Classification) -> Result<Option<InstallOutcome>> {
        let Some(game) = self.game.clone() else {
            return Ok(None);
        };
        let Some(runtime) = self.runtimes.installer_for(classification) else {
            return Ok(None);
        };
        let outcome = runtime.install(&game, classification)?;
        self.reload();
        Ok(Some(outcome))
    }

    pub fn import_from_game_root(&mut self, which: ThirdPartyRuntime) -> Result<bool> {
        let Some(game) = self.game.clone() else {
            return Ok(false);
        };
        let imported = self.runtimes.get(which).import_from_game_root(&game)?;
        self.reload();
        Ok(imported)
    }

    pub fn set_enabled(&mut self, which: ThirdPartyRuntime, enabled: bool) -> Result<()> {
        self.with_game(|runtimes, game| runtimes.get(which).set_enabled(game, enabled))
    }

    pub fn apply_reshade_config(&mut self, config: &ReShadeConfig) -> Result<()> {
        self.with_game(|runtimes, game| runtimes.reshade.apply_config(game, config))
    }

    pub fn apply_migoto_config(&mut self, config: &MigotoConfig) -> Result<()> {
        self.with_game(|runtimes, game| runtimes.migoto.apply_config(game, config))
    }

    pub fn set_game_mod_disabled(&mut self, file_name: &str, disabled: bool) -> Result<()> {
        self.with_game(|runtimes, game| runtimes.game_mods.set_mod_disabled(game, file_name, disabled).map(drop))
    }

    pub fn set_game_mod_settings(&mut self, file_name: &str, settings: &GameModSettings) -> Result<()> {
        self.with_game(|runtimes, game| runtimes.game_mods.set_mod_settings(game, file_name, settings).map(drop))
    }

    pub fn set_graphic_mod_disabled(&mut self, mod_dir: &str, disabled: bool) -> Result<()> {
        self.with_game(|runtimes, game| {
            runtimes.graphic_mods.set_mod_disabled(game, mod_dir, disabled);
            Ok(())
        })
    }

    pub fn remove_graphic_mod(&mut self, mod_dir: &str) -> Result<()> {
        self.with_game(|runtimes, game| {
            runtimes.graphic_mods.remove_mod(game, mod_dir);
            Ok(())
        })
    }

    pub fn remove_game_mod(&mut self, mod_dir: &str) -> Result<()> {
        self.with_game(|runtimes, game| {
            runtimes.game_mods.remove_mod(game, mod_dir);
            Ok(())
        })
    }

    pub fn move_mod(&mut self, mod_dir: &str, from: ThirdPartyRuntime) -> Result<bool> {
        let Some(game) = self.game.clone() else {
            return Ok(false);
        };
        let moved = move_mod_between(&self.runtimes, &game, mod_dir, from)?;
        if moved {
            self.reload();
        }
        Ok(moved)
    }

    pub fn remove(&mut self, which: ThirdPartyRuntime) -> Result<()> {
        self.with_game(|runtimes, game| runtimes.get(which).remove(game))
    }

    fn with_game(&mut self, action: impl FnOnce(&Runtimes, &GamePaths) -> Result<()>) -> Result<()> {
        let Some(game) = self.game.clone() else {
            return Ok(());
        };
        action(&self.runtimes, &game)?;
        self.reload();
        Ok(())
    }
}

pub fn move_mod_between(runtimes: &Runtimes, game: &GamePaths, mod_dir: &str, from: ThirdPartyRuntime) -> Result<bool> {
    let to_graphic = from == ThirdPartyRuntime::GameMods;
    let source = if to_graphic {
        runtimes.game_mods.path_of(game, mod_dir)
    } else {
        runtimes.graphic_mods.path_of(game, mod_dir)
    };
    let Some(source) = source else {
        return Ok(false);
    };
    let target_runtime: &dyn GraphicsRuntime = if to_graphic { &runtimes.graphic_mods } else { &runtimes.game_mods };
    let target_root = target_runtime.install_dir(game);
    let target = target_root.join(paths::unique_dir_name(&target_root, mod_dir));
    crate::tree::move_merging(&source, &target);
    if !target.is_dir() {
        return Ok(false);
    }
    target_runtime.set_enabled(game, true)?;
    Ok(true)
}
