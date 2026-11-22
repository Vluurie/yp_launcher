use std::path::PathBuf;

use indexmap::IndexMap;
use yp_core::GamePaths;

use crate::defaults::{self, DefaultEntry, DefaultKind};
use crate::disabled;
use crate::error::Result;
use crate::load_order;
use crate::model::InstalledMod;
use crate::names;
use crate::profiles::ModProfileState;
use crate::scan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchTarget {
    pub path: PathBuf,
    pub recursive: bool,
}

impl WatchTarget {
    fn file(path: PathBuf) -> Self {
        Self {
            path,
            recursive: false,
        }
    }

    fn tree(path: PathBuf) -> Self {
        Self {
            path,
            recursive: true,
        }
    }
}

fn has_game_dir(game: &GamePaths) -> bool {
    !game.root().as_os_str().is_empty()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModsData {
    pub mods: Vec<InstalledMod>,
}

impl ModsData {
    pub fn load(game: &GamePaths) -> Result<Self> {
        if !has_game_dir(game) {
            return Ok(Self::default());
        }
        Ok(Self {
            mods: scan::list_installed(game)?,
        })
    }

    pub fn reload(&mut self, game: &GamePaths) -> Result<()> {
        if has_game_dir(game) {
            *self = Self::load(game)?;
        }
        Ok(())
    }

    pub fn find(&self, id: &str) -> Option<&InstalledMod> {
        self.mods.iter().find(|installed| installed.id == id)
    }

    pub fn watch_paths(game: &GamePaths) -> Vec<WatchTarget> {
        vec![WatchTarget::tree(game.mods_dir())]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DisabledModsData {
    pub entries: Vec<String>,
}

impl DisabledModsData {
    pub fn load(game: &GamePaths) -> Result<Self> {
        if !has_game_dir(game) {
            return Ok(Self::default());
        }
        Ok(Self {
            entries: disabled::list(game)?,
        })
    }

    pub fn reload(&mut self, game: &GamePaths) -> Result<()> {
        if has_game_dir(game) {
            *self = Self::load(game)?;
        }
        Ok(())
    }

    pub fn set_disabled(&mut self, game: &GamePaths, rel_path: &str, disabled: bool) -> Result<()> {
        disabled::set_disabled(game, rel_path, disabled)?;
        self.reload(game)
    }

    pub fn is_disabled(&self, rel_path: &str) -> bool {
        disabled::matches(&self.entries, rel_path)
    }

    pub fn watch_paths(game: &GamePaths) -> Vec<WatchTarget> {
        vec![WatchTarget::file(game.disabled_mods_toml())]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DefaultModsData {
    pub entries: Vec<DefaultEntry>,
    pub default_outfits_enabled: bool,
}

impl DefaultModsData {
    pub fn load(game: &GamePaths) -> Result<Self> {
        if !has_game_dir(game) || !defaults::is_feature_enabled(game) {
            return Ok(Self::default());
        }
        Ok(Self {
            entries: defaults::list(game)?,
            default_outfits_enabled: true,
        })
    }

    pub fn reload(&mut self, game: &GamePaths) -> Result<()> {
        if has_game_dir(game) {
            *self = Self::load(game)?;
        }
        Ok(())
    }

    pub fn set_default(
        &mut self,
        game: &GamePaths,
        rel_path: &str,
        kind: Option<DefaultKind>,
        outfit_id: Option<i64>,
    ) -> Result<()> {
        defaults::set_default(game, rel_path, kind, outfit_id)?;
        self.reload(game)
    }

    pub fn is_default(&self, rel_path: &str) -> bool {
        defaults::matches(&self.entries, rel_path)
    }

    pub fn kind_of(&self, rel_path: &str) -> Option<DefaultKind> {
        defaults::entry_for(&self.entries, rel_path).map(|entry| entry.kind)
    }

    pub fn outfit_id_of(&self, rel_path: &str) -> Option<i64> {
        defaults::entry_for(&self.entries, rel_path).and_then(|entry| entry.outfit_id)
    }

    pub fn would_replace(&self, rel_path: &str, kind: DefaultKind) -> Option<&DefaultEntry> {
        let rel = defaults::normalize(rel_path);
        let stem = defaults::stem_of(&rel)?;
        self.entries.iter().find(|entry| {
            entry.path != rel
                && kind.conflicts_with(entry.kind)
                && defaults::stem_of(&entry.path).as_deref() == Some(stem.as_str())
        })
    }

    pub fn entries_under(&self, rel_path: &str) -> Vec<&DefaultEntry> {
        let rel = defaults::normalize(rel_path);
        self.entries
            .iter()
            .filter(|entry| {
                entry.path == rel
                    || entry
                        .path
                        .strip_prefix(rel.as_str())
                        .is_some_and(|rest| rest.starts_with('/'))
            })
            .collect()
    }

    pub fn watch_paths(game: &GamePaths) -> Vec<WatchTarget> {
        vec![
            WatchTarget::file(game.default_mods_toml()),
            WatchTarget::file(game.nams_toml()),
        ]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModLoadOrderData {
    pub order: Vec<String>,
}

impl ModLoadOrderData {
    pub fn load(game: &GamePaths) -> Result<Self> {
        if !has_game_dir(game) {
            return Ok(Self::default());
        }
        let mut order = load_order::list(game)?;
        order.reverse();
        Ok(Self { order })
    }

    pub fn reload(&mut self, game: &GamePaths) -> Result<()> {
        if has_game_dir(game) {
            *self = Self::load(game)?;
        }
        Ok(())
    }

    pub fn save(&mut self, game: &GamePaths, order: &[String]) -> Result<()> {
        let stored: Vec<String> = order.iter().rev().cloned().collect();
        load_order::save(game, &stored)?;
        self.reload(game)
    }

    pub fn index_of(&self, rel_path: &str) -> Option<usize> {
        let rel = load_order::normalize(rel_path);
        self.order.iter().position(|entry| *entry == rel)
    }

    pub fn watch_paths(game: &GamePaths) -> Vec<WatchTarget> {
        vec![WatchTarget::file(game.mod_load_order_toml())]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModNamesData {
    pub names: IndexMap<String, String>,
}

impl ModNamesData {
    pub fn load(game: &GamePaths) -> Self {
        Self {
            names: names::load(game),
        }
    }

    pub fn reload(&mut self, game: &GamePaths) {
        if has_game_dir(game) {
            *self = Self::load(game);
        }
    }

    pub fn rename(&mut self, game: &GamePaths, mod_id: &str, name: Option<&str>) {
        names::set_name(game, mod_id, name);
        self.reload(game);
    }

    pub fn custom_name_of(&self, mod_id: &str) -> Option<&str> {
        self.names.get(mod_id).map(String::as_str)
    }

    pub fn watch_paths(game: &GamePaths) -> Vec<WatchTarget> {
        vec![WatchTarget::file(game.mod_names_json())]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModsWorkspace {
    pub mods: ModsData,
    pub disabled: DisabledModsData,
    pub defaults: DefaultModsData,
    pub load_order: ModLoadOrderData,
    pub names: ModNamesData,
    pub profiles: ModProfileState,
}

impl ModsWorkspace {
    pub fn load(game: &GamePaths) -> Result<Self> {
        if !has_game_dir(game) {
            return Ok(Self::default());
        }
        let profiles = crate::profiles::load(game)?;
        Ok(Self {
            mods: ModsData::load(game)?,
            disabled: DisabledModsData::load(game)?,
            defaults: DefaultModsData::load(game)?,
            load_order: ModLoadOrderData::load(game)?,
            names: ModNamesData::load(game),
            profiles,
        })
    }

    pub fn apply_profile_change(
        &mut self,
        game: &GamePaths,
        profiles: ModProfileState,
    ) -> Result<()> {
        self.profiles = profiles;
        self.mods.reload(game)?;
        self.disabled.reload(game)?;
        self.defaults.reload(game)?;
        self.load_order.reload(game)
    }

    pub fn watch_paths(game: &GamePaths) -> Vec<WatchTarget> {
        let mut targets = ModsData::watch_paths(game);
        targets.extend(DisabledModsData::watch_paths(game));
        targets.extend(DefaultModsData::watch_paths(game));
        targets.extend(ModLoadOrderData::watch_paths(game));
        targets.extend(ModNamesData::watch_paths(game));
        targets.push(WatchTarget::file(game.profiles_toml()));
        targets
    }
}
