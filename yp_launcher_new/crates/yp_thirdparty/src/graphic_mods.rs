use std::path::{Path, PathBuf};

use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::error::Result;
use crate::flags;
use crate::game_mods::{dll_folders, size_summary};
use crate::model::{
    Classification, GameModEntry, GameModsInfo, InstallFailure, InstallOutcome, RuntimeStatus, ThirdPartyKind,
    ThirdPartyRuntime, ThirdPartyUpdateInfo,
};
use crate::paths;
use crate::runtime::GraphicsRuntime;
use crate::tree::{self, ExistingFiles};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphicModsRuntime;

impl GraphicModsRuntime {
    pub fn disabled_dir(&self, game: &GamePaths) -> PathBuf {
        paths::graphic_mods_disabled(game)
    }

    pub fn set_mod_disabled(&self, game: &GamePaths, mod_dir: &str, disabled: bool) {
        if !paths::is_plain_name(mod_dir) {
            return;
        }
        let (from_root, to_root) = if disabled {
            (paths::graphic_mods(game), self.disabled_dir(game))
        } else {
            (self.disabled_dir(game), paths::graphic_mods(game))
        };
        let from = from_root.join(mod_dir);
        if !from.is_dir() {
            return;
        }
        tree::move_merging(&from, &to_root.join(mod_dir));
    }

    pub fn path_of(&self, game: &GamePaths, mod_dir: &str) -> Option<PathBuf> {
        if !paths::is_plain_name(mod_dir) {
            return None;
        }
        [paths::graphic_mods(game).join(mod_dir), self.disabled_dir(game).join(mod_dir)]
            .into_iter()
            .find(|dir| dir.is_dir())
    }

    pub fn remove_mod(&self, game: &GamePaths, mod_dir: &str) {
        if !paths::is_plain_name(mod_dir) {
            return;
        }
        yp_core::fs::remove_dir_quiet(&paths::graphic_mods(game).join(mod_dir));
        yp_core::fs::remove_dir_quiet(&self.disabled_dir(game).join(mod_dir));
    }
}

fn entries_in(root: &Path, disabled: bool) -> Vec<GameModEntry> {
    paths::mod_dirs(root)
        .into_iter()
        .flat_map(|mod_dir| {
            let mod_name = tree::file_name(&mod_dir);
            paths::dlls_in(&mod_dir).into_iter().map(move |dll| GameModEntry {
                file_name: tree::file_name(&dll),
                mod_dir: mod_name.clone(),
                size_bytes: tree::file_size(&dll),
                disabled,
                settings: None,
            })
        })
        .collect()
}

impl GraphicsRuntime for GraphicModsRuntime {
    fn runtime(&self) -> ThirdPartyRuntime {
        ThirdPartyRuntime::GraphicMods
    }

    fn install_dir(&self, game: &GamePaths) -> PathBuf {
        paths::graphic_mods(game)
    }

    fn can_install(&self, classification: &Classification) -> bool {
        classification.kind == ThirdPartyKind::GraphicMod
    }

    fn install(&self, game: &GamePaths, classification: &Classification) -> Result<InstallOutcome> {
        let Some(first) = classification.graphic_mod_dlls.first() else {
            return Ok(InstallOutcome::Rejected(InstallFailure::NoDll));
        };
        let names: Vec<String> = classification.graphic_mod_dlls.iter().map(|dll| tree::file_name(dll)).collect();
        let active = paths::graphic_mods(game);

        let mut dest = paths::mod_dir_with_dll(&active, &names);
        if dest.is_none()
            && let Some(parked) = paths::mod_dir_with_dll(&self.disabled_dir(game), &names)
        {
            let revived = active.join(paths::unique_dir_name(&active, &tree::file_name(&parked)));
            tree::move_merging(&parked, &revived);
            dest = Some(revived);
        }
        let dest = dest.unwrap_or_else(|| {
            let name = paths::mod_dir_name(classification.source_name.as_deref().unwrap_or_default(), &tree::file_stem(first));
            active.join(paths::unique_dir_name(&active, &name))
        });
        std::fs::create_dir_all(&dest).io(IoOp::Create, &dest)?;

        for folder in dll_folders(&classification.graphic_mod_dlls) {
            tree::merge_tree(&folder, &dest, ExistingFiles::Overwrite);
        }

        flags::set_disabled(game, self.loading_flag(), false)?;
        Ok(InstallOutcome::installed(ThirdPartyRuntime::GraphicMods))
    }

    fn would_update(&self, game: &GamePaths, classification: &Classification) -> Option<ThirdPartyUpdateInfo> {
        classification.graphic_mod_dlls.iter().find_map(|incoming| {
            let name = tree::file_name(incoming);
            let installed = paths::find_dll(&paths::graphic_mods(game), &name)
                .or_else(|| paths::find_dll(&self.disabled_dir(game), &name))?;
            tree::files_differ(incoming, &installed).then(|| ThirdPartyUpdateInfo {
                runtime: ThirdPartyRuntime::GraphicMods,
                installed: size_summary(&installed),
                incoming: size_summary(incoming),
            })
        })
    }

    fn repair(&self, _game: &GamePaths) -> Result<()> {
        Ok(())
    }

    fn import_from_game_root(&self, _game: &GamePaths) -> Result<bool> {
        Ok(false)
    }

    fn status(&self, game: &GamePaths) -> RuntimeStatus {
        let mut mods = entries_in(&paths::graphic_mods(game), false);
        mods.extend(entries_in(&self.disabled_dir(game), true));
        mods.sort_by_cached_key(|entry| entry.key().to_lowercase());
        RuntimeStatus {
            installed: !mods.is_empty(),
            enabled: self.is_enabled(game),
            graphic_mods_info: Some(GameModsInfo { mods, has_config: false }),
            ..RuntimeStatus::default()
        }
    }

    fn remove(&self, game: &GamePaths) -> Result<()> {
        for root in [paths::graphic_mods(game), self.disabled_dir(game)] {
            for dir in paths::mod_dirs(&root) {
                yp_core::fs::remove_dir_quiet(&dir);
            }
        }
        flags::set_disabled(game, self.loading_flag(), true)
    }
}
