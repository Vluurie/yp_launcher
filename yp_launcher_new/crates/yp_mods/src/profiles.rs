use std::io;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
use jiff::Timestamp;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};
use yp_core::{GamePaths, IoOp};

use crate::bundled::{read_packs_file, render_packs, sidecar_path};
use crate::disabled::normalize as normalize_disabled;
use crate::error::{ModsError, ProfileRejection, ProfileStep, Result};
use crate::fsx;
use crate::pack_name;
use crate::probe::is_hidden_name;
use crate::text::{dart_trim, is_regex_whitespace};
use crate::texture_config::{with_disabled_packs, with_renamed_load_order};
use crate::toml_doc::{
    parse_document, read_values_strict, set_root_value, string_array, string_list,
};

pub const DEFAULT_PROFILE_NAME: &str = "default";
pub const PROFILE_DIR_PREFIX: &str = "mods_profile_";
const ORPHAN_BACKUP_DIR: &str = ".orphan_backup";
const MAX_NAME_LEN: usize = 48;
const PROFILES_HEADER: &str = "# YP Launcher mod profile state. Edit via the launcher.\n";
const EMPTY_DISABLED_FILE: &str = "disabled = []\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModProfile {
    pub name: String,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModProfileState {
    pub active_name: String,
    pub profiles: Vec<ModProfile>,
}

impl Default for ModProfileState {
    fn default() -> Self {
        Self {
            active_name: DEFAULT_PROFILE_NAME.to_owned(),
            profiles: Vec::new(),
        }
    }
}

impl ModProfileState {
    pub fn active(&self) -> Option<&ModProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.name == self.active_name)
    }

    fn has(&self, name: &str) -> bool {
        self.profiles.iter().any(|profile| profile.name == name)
    }
}

pub trait ProfileFs: Send + Sync {
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn create_dir(&self, path: &Path) -> io::Result<()>;
    fn remove_dir(&self, path: &Path) -> io::Result<()>;
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RealProfileFs;

impl ProfileFs for RealProfileFs {
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir(path)
    }

    fn remove_dir(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_dir(path)
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        std::fs::write(path, contents)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }
}

#[derive(Clone, Copy)]
struct Sidecar {
    active: fn(&GamePaths) -> PathBuf,
    inactive_prefix: &'static str,
    backup_prefix: &'static str,
}

impl Sidecar {
    fn active_path(&self, game: &GamePaths) -> PathBuf {
        (self.active)(game)
    }

    fn inactive_path(&self, game: &GamePaths, profile: &str) -> PathBuf {
        game.nams_dir()
            .join(format!("{}{profile}.toml", self.inactive_prefix))
    }
}

const SIDECARS: [Sidecar; 3] = [
    Sidecar {
        active: GamePaths::disabled_mods_toml,
        inactive_prefix: "disabled_mods_profile_",
        backup_prefix: "disabled",
    },
    Sidecar {
        active: GamePaths::default_mods_toml,
        inactive_prefix: "default_mods_profile_",
        backup_prefix: "default_mods",
    },
    Sidecar {
        active: GamePaths::mod_load_order_toml,
        inactive_prefix: "mod_load_order_profile_",
        backup_prefix: "load_order",
    },
];

pub fn profile_mods_dir(game: &GamePaths, profile: &str) -> PathBuf {
    game.nams_dir()
        .join(format!("{PROFILE_DIR_PREFIX}{profile}"))
}

pub fn load(game: &GamePaths) -> Result<ModProfileState> {
    ModProfiles::new(game).load()
}

pub fn active_name(game: &GamePaths) -> String {
    std::fs::read_to_string(game.profiles_toml())
        .ok()
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|table| {
            table
                .get("active")
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| DEFAULT_PROFILE_NAME.to_owned())
}

pub fn sanitize_name(raw: &str) -> String {
    let lower = dart_trim(raw).to_lowercase();
    let mut spaced = String::with_capacity(lower.len());
    let mut in_whitespace = false;
    for c in lower.chars() {
        if is_regex_whitespace(c) {
            if !in_whitespace {
                spaced.push('_');
            }
            in_whitespace = true;
        } else {
            in_whitespace = false;
            spaced.push(c);
        }
    }
    let restricted: String = spaced
        .chars()
        .map(|c| if is_name_char(c) { c } else { '_' })
        .collect();
    let stripped = restricted.trim_start_matches(['.', '_']);
    let mut collapsed = String::with_capacity(stripped.len());
    for c in stripped.chars() {
        if c == '_' && collapsed.ends_with('_') {
            continue;
        }
        collapsed.push(c);
    }
    collapsed.truncate(MAX_NAME_LEN);
    collapsed
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
}

pub fn is_valid_name(sanitized: &str) -> bool {
    let mut chars = sanitized.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    sanitized != "mods"
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(is_name_char)
}

fn format_timestamp(timestamp: Timestamp) -> String {
    timestamp.strftime("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn parse_timestamp(text: &str) -> Option<Timestamp> {
    if let Ok(timestamp) = text.parse::<Timestamp>() {
        return Some(timestamp);
    }

    let utc = jiff::tz::TimeZone::UTC;

    if let Ok(datetime) = text.parse::<jiff::civil::DateTime>() {
        return utc.to_timestamp(datetime).ok();
    }

    text.parse::<jiff::civil::Date>()
        .ok()
        .and_then(|date| utc.to_timestamp(date.at(0, 0, 0, 0)).ok())
}

fn io_error(op: IoOp, path: &Path, source: io::Error) -> yp_core::Error {
    yp_core::Error::Io {
        op,
        path: path.to_path_buf(),
        source,
    }
}

fn step_failed(step: ProfileStep, source: yp_core::Error) -> ModsError {
    ModsError::ProfileStep { step, source }
}

fn is_shared_texture_path(entry: &str) -> bool {
    entry == "inject" || entry == "inject/textures" || entry.starts_with("inject/textures/")
}

fn disabled_entries(text: &str, path: &Path) -> std::result::Result<Vec<String>, yp_core::Error> {
    let values = read_values_strict(text, path)?;
    Ok(string_list(values.get("disabled"))
        .unwrap_or_default()
        .iter()
        .map(|entry| normalize_disabled(entry))
        .collect())
}

fn read_text(path: &Path) -> std::result::Result<String, yp_core::Error> {
    std::fs::read_to_string(path).map_err(|source| io_error(IoOp::Read, path, source))
}

fn ensure_dir(path: &Path) -> std::result::Result<(), yp_core::Error> {
    if path.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(path).map_err(|source| io_error(IoOp::Create, path, source))
}

pub struct ModProfiles<'a> {
    game: &'a GamePaths,
    fs: &'a dyn ProfileFs,
}

impl<'a> ModProfiles<'a> {
    pub fn new(game: &'a GamePaths) -> Self {
        Self {
            game,
            fs: &RealProfileFs,
        }
    }

    pub fn with_fs(game: &'a GamePaths, fs: &'a dyn ProfileFs) -> Self {
        Self { game, fs }
    }

    fn mods_dir(&self) -> PathBuf {
        self.game.mods_dir()
    }

    fn inactive_dir(&self, profile: &str) -> PathBuf {
        profile_mods_dir(self.game, profile)
    }

    fn rename(&self, from: &Path, to: &Path) -> std::result::Result<(), yp_core::Error> {
        self.fs
            .rename(from, to)
            .map_err(|source| io_error(IoOp::Rename, from, source))
    }

    fn write(&self, path: &Path, contents: &str) -> std::result::Result<(), yp_core::Error> {
        self.fs
            .write(path, contents)
            .map_err(|source| io_error(IoOp::Write, path, source))
    }

    pub fn load(&self) -> Result<ModProfileState> {
        let nams = self.game.nams_dir();
        ensure_dir(&nams)?;
        let profiles_file = self.game.profiles_toml();
        if !profiles_file.is_file() {
            return self.migrate_first_run();
        }
        let Some(parsed) = std::fs::read_to_string(&profiles_file)
            .ok()
            .and_then(|text| text.parse::<toml::Table>().ok())
        else {
            return self.migrate_first_run();
        };

        let active = parsed
            .get("active")
            .and_then(toml::Value::as_str)
            .unwrap_or(DEFAULT_PROFILE_NAME)
            .to_owned();
        let mut profiles: Vec<ModProfile> = Vec::new();
        if let Some(entries) = parsed.get("profile").and_then(toml::Value::as_array) {
            for entry in entries {
                let Some(table) = entry.as_table() else {
                    continue;
                };
                let Some(name) = table
                    .get("name")
                    .and_then(toml::Value::as_str)
                    .filter(|name| !name.is_empty())
                else {
                    continue;
                };
                let created_at = table
                    .get("created_at")
                    .and_then(toml::Value::as_str)
                    .and_then(parse_timestamp)
                    .unwrap_or_else(Timestamp::now);
                profiles.push(ModProfile {
                    name: name.to_owned(),
                    created_at,
                });
            }
        }

        let mut filtered: Vec<ModProfile> = Vec::new();
        for profile in &profiles {
            if profile.name == active {
                filtered.push(profile.clone());
                ensure_dir(&self.mods_dir())?;
            } else if self.inactive_dir(&profile.name).is_dir() {
                filtered.push(profile.clone());
            }
        }

        let mut known: IndexSet<String> = filtered
            .iter()
            .map(|profile| profile.name.clone())
            .collect();
        for entity in fsx::child_dirs(&nams) {
            let Some(inferred) = entity.name.strip_prefix(PROFILE_DIR_PREFIX) else {
                continue;
            };
            if inferred.is_empty() || known.contains(inferred) || !is_valid_name(inferred) {
                continue;
            }
            filtered.push(ModProfile {
                name: inferred.to_owned(),
                created_at: fsx::mtime(&entity.path).unwrap_or_else(Timestamp::now),
            });
            known.insert(inferred.to_owned());
        }

        if !filtered.iter().any(|profile| profile.name == active) {
            ensure_dir(&self.mods_dir())?;
            filtered.insert(
                0,
                ModProfile {
                    name: active.clone(),
                    created_at: Timestamp::now(),
                },
            );
        }

        let renames = self.plan_pack_upgrade_renames(&filtered);
        if !renames.is_empty() {
            self.apply_pack_renames(&renames);
            self.rewrite_load_order(&renames)?;
        }

        self.write_disabled_packs(&self.compute_disabled_packs(&filtered, &active))?;

        let state_changed = filtered.len() != profiles.len()
            || !filtered
                .iter()
                .all(|profile| profiles.iter().any(|other| other.name == profile.name))
            || !renames.is_empty();
        if state_changed {
            self.write_profiles_toml(&active, &filtered)?;
        }

        Ok(ModProfileState {
            active_name: active,
            profiles: filtered,
        })
    }

    fn migrate_first_run(&self) -> Result<ModProfileState> {
        let nams = self.game.nams_dir();
        ensure_dir(&nams)?;
        ensure_dir(&self.mods_dir())?;

        let mut profiles = vec![ModProfile {
            name: DEFAULT_PROFILE_NAME.to_owned(),
            created_at: fsx::mtime(&self.mods_dir()).unwrap_or_else(Timestamp::now),
        }];
        for entity in fsx::child_dirs(&nams) {
            let Some(inferred) = entity.name.strip_prefix(PROFILE_DIR_PREFIX) else {
                continue;
            };
            if !is_valid_name(inferred) || inferred == DEFAULT_PROFILE_NAME {
                continue;
            }
            profiles.push(ModProfile {
                name: inferred.to_owned(),
                created_at: fsx::mtime(&entity.path).unwrap_or_else(Timestamp::now),
            });
        }

        let renames = self.plan_pack_upgrade_renames(&profiles);
        if !renames.is_empty() {
            self.apply_pack_renames(&renames);
            self.rewrite_load_order(&renames)?;
        }
        self.write_disabled_packs(&self.compute_disabled_packs(&profiles, DEFAULT_PROFILE_NAME))?;
        self.write_profiles_toml(DEFAULT_PROFILE_NAME, &profiles)?;

        Ok(ModProfileState {
            active_name: DEFAULT_PROFILE_NAME.to_owned(),
            profiles,
        })
    }

    pub fn create(&self, name: &str) -> Result<ModProfileState> {
        let state = self.load()?;
        let sanitized = sanitize_name(name);
        if !is_valid_name(&sanitized) {
            return Err(ProfileRejection::InvalidName.into());
        }
        if state.has(&sanitized) || self.inactive_dir(&sanitized).is_dir() {
            return Err(ProfileRejection::NameCollision.into());
        }

        let from = state.active_name.clone();
        let texture_disables = self.shared_texture_disables()?;
        self.backup_orphan(&from)?;

        let park_active = |sidecar: &Sidecar| sidecar.active_path(self.game);
        let park_inactive = |sidecar: &Sidecar| sidecar.inactive_path(self.game, &from);
        let moved_from = self
            .move_sidecars(&park_active, &park_inactive)
            .map_err(|source| step_failed(ProfileStep::ParkSidecars, source))?;
        let undo_park = || self.restore_sidecars(&moved_from, &park_inactive, &park_active);

        let active_mods = self.mods_dir();
        let from_inactive = self.inactive_dir(&from);
        if let Err(source) = self.rename(&active_mods, &from_inactive) {
            undo_park();
            return Err(step_failed(ProfileStep::ParkMods, source));
        }

        if let Err(source) = self.fs.create_dir(&active_mods) {
            let _ = self.fs.rename(&from_inactive, &active_mods);
            undo_park();
            return Err(step_failed(
                ProfileStep::CreateMods,
                io_error(IoOp::Create, &active_mods, source),
            ));
        }

        let mut new_profiles = state.profiles.clone();
        new_profiles.push(ModProfile {
            name: sanitized.clone(),
            created_at: Timestamp::now(),
        });
        let texture_state = self
            .write_shared_texture_disables(&texture_disables)
            .and_then(|()| {
                self.write_disabled_packs(&self.compute_disabled_packs(&new_profiles, &sanitized))
            });
        if let Err(source) = texture_state {
            self.restore_disabled_mods(None);
            let _ = self.fs.remove_dir(&active_mods);
            let _ = self.fs.rename(&from_inactive, &active_mods);
            undo_park();
            return Err(step_failed(ProfileStep::TextureState, source));
        }
        self.write_profiles_toml(&sanitized, &new_profiles)?;

        Ok(ModProfileState {
            active_name: sanitized,
            profiles: new_profiles,
        })
    }

    pub fn switch_to(&self, to: &str) -> Result<ModProfileState> {
        let state = self.load()?;
        let from = state.active_name.clone();
        if from == to {
            return Ok(state);
        }
        if !state.has(to) {
            return Err(ProfileRejection::UnknownTarget.into());
        }
        if !self.inactive_dir(to).is_dir() {
            return Err(ProfileRejection::TargetMissing.into());
        }
        if !self.mods_dir().is_dir() {
            return Err(ProfileRejection::ActiveMissing.into());
        }

        let texture_disables = self.shared_texture_disables()?;
        let target_disabled_file = SIDECARS[0].inactive_path(self.game, to);
        let target_disabled_raw = if target_disabled_file.is_file() {
            Some(read_text(&target_disabled_file)?)
        } else {
            None
        };
        self.backup_orphan(&from)?;

        let active_path = |sidecar: &Sidecar| sidecar.active_path(self.game);
        let from_path = |sidecar: &Sidecar| sidecar.inactive_path(self.game, &from);
        let to_path = |sidecar: &Sidecar| sidecar.inactive_path(self.game, to);

        let moved_from = self
            .move_sidecars(&active_path, &from_path)
            .map_err(|source| step_failed(ProfileStep::ParkSidecars, source))?;
        let undo_park = || self.restore_sidecars(&moved_from, &from_path, &active_path);

        let active_mods = self.mods_dir();
        let from_inactive = self.inactive_dir(&from);
        let to_inactive = self.inactive_dir(to);

        if let Err(source) = self.rename(&active_mods, &from_inactive) {
            undo_park();
            return Err(step_failed(ProfileStep::ParkMods, source));
        }

        if let Err(source) = self.rename(&to_inactive, &active_mods) {
            let _ = self.fs.rename(&from_inactive, &active_mods);
            undo_park();
            return Err(step_failed(ProfileStep::ActivateMods, source));
        }

        let undo_mods = || {
            let _ = self.fs.rename(&active_mods, &to_inactive);
            let _ = self.fs.rename(&from_inactive, &active_mods);
            undo_park();
        };

        let moved_to = match self.move_sidecars(&to_path, &active_path) {
            Ok(moved) => moved,
            Err(source) => {
                undo_mods();
                return Err(step_failed(ProfileStep::ActivateSidecars, source));
            }
        };

        let new_disabled_packs = self.compute_disabled_packs(&state.profiles, to);
        let texture_state = self
            .write_shared_texture_disables(&texture_disables)
            .and_then(|()| self.write_disabled_packs(&new_disabled_packs));
        if let Err(source) = texture_state {
            self.restore_disabled_mods(target_disabled_raw.as_deref());
            self.restore_sidecars(&moved_to, &active_path, &to_path);
            undo_mods();
            return Err(step_failed(ProfileStep::TextureState, source));
        }

        self.write_profiles_toml(to, &state.profiles)
            .map_err(|source| step_failed(ProfileStep::WriteProfiles, source))?;

        Ok(ModProfileState {
            active_name: to.to_owned(),
            profiles: state.profiles,
        })
    }

    pub fn delete(&self, name: &str) -> Result<ModProfileState> {
        let state = self.load()?;
        if state.profiles.len() <= 1 {
            return Err(ProfileRejection::CannotDeleteLast.into());
        }
        if state.active_name == name {
            return Err(ProfileRejection::CannotDeleteActive.into());
        }
        if !state.has(name) {
            return Err(ProfileRejection::UnknownTarget.into());
        }

        let owned_packs = self.collect_profile_bundled_packs(name);

        let inactive = self.inactive_dir(name);
        if inactive.is_dir() {
            let _ = std::fs::remove_dir_all(&inactive);
        }
        for sidecar in &SIDECARS {
            let file = sidecar.inactive_path(self.game, name);
            if file.is_file() {
                let _ = self.fs.remove_file(&file);
            }
        }

        let inject_root = self.game.inject_textures_dir();
        for pack in &owned_packs {
            let pack_dir = inject_root.join(pack);
            if pack_dir.is_dir() {
                let _ = std::fs::remove_dir_all(&pack_dir);
            }
        }

        let renames: IndexMap<String, Option<String>> = owned_packs
            .iter()
            .map(|pack| (pack.clone(), None))
            .collect();
        self.rewrite_load_order(&renames)?;

        let new_profiles: Vec<ModProfile> = state
            .profiles
            .iter()
            .filter(|profile| profile.name != name)
            .cloned()
            .collect();
        self.write_disabled_packs(&self.compute_disabled_packs(&new_profiles, &state.active_name))?;
        self.write_profiles_toml(&state.active_name, &new_profiles)?;

        Ok(ModProfileState {
            active_name: state.active_name,
            profiles: new_profiles,
        })
    }

    pub fn rename_profile(&self, old_name: &str, new_name: &str) -> Result<ModProfileState> {
        let state = self.load()?;
        let sanitized = sanitize_name(new_name);
        if !is_valid_name(&sanitized) {
            return Err(ProfileRejection::InvalidName.into());
        }
        if sanitized == old_name {
            return Ok(state);
        }
        if state.has(&sanitized) || self.inactive_dir(&sanitized).is_dir() {
            return Err(ProfileRejection::NameCollision.into());
        }
        if !state.has(old_name) {
            return Err(ProfileRejection::UnknownTarget.into());
        }

        let is_active = state.active_name == old_name;
        let renames = self.plan_profile_prefix_renames(old_name, &sanitized, is_active);

        if !is_active {
            let old_dir = self.inactive_dir(old_name);
            let new_dir = self.inactive_dir(&sanitized);
            self.rename(&old_dir, &new_dir)
                .map_err(|source| step_failed(ProfileStep::RenameMods, source))?;
            let old_path = |sidecar: &Sidecar| sidecar.inactive_path(self.game, old_name);
            let new_path = |sidecar: &Sidecar| sidecar.inactive_path(self.game, &sanitized);
            if let Err(source) = self.move_sidecars(&old_path, &new_path) {
                let _ = self.fs.rename(&new_dir, &old_dir);
                return Err(step_failed(ProfileStep::RenameSidecars, source));
            }
        }

        self.apply_pack_renames(&renames);
        self.rewrite_load_order(&renames)?;

        let new_profiles: Vec<ModProfile> = state
            .profiles
            .iter()
            .map(|profile| {
                if profile.name == old_name {
                    ModProfile {
                        name: sanitized.clone(),
                        created_at: profile.created_at,
                    }
                } else {
                    profile.clone()
                }
            })
            .collect();
        let new_active = if is_active {
            sanitized
        } else {
            state.active_name.clone()
        };
        self.write_disabled_packs(&self.compute_disabled_packs(&new_profiles, &new_active))?;
        self.write_profiles_toml(&new_active, &new_profiles)?;

        Ok(ModProfileState {
            active_name: new_active,
            profiles: new_profiles,
        })
    }

    fn shared_texture_disables(&self) -> std::result::Result<Vec<String>, yp_core::Error> {
        let file = self.game.disabled_mods_toml();
        if !file.is_file() {
            return Ok(Vec::new());
        }
        let text = read_text(&file)?;
        Ok(disabled_entries(&text, &file)?
            .into_iter()
            .filter(|entry| is_shared_texture_path(entry))
            .collect())
    }

    fn write_shared_texture_disables(
        &self,
        textures: &[String],
    ) -> std::result::Result<(), yp_core::Error> {
        let file = self.game.disabled_mods_toml();
        let raw = if file.is_file() {
            read_text(&file)?
        } else {
            EMPTY_DISABLED_FILE.to_owned()
        };
        let current = disabled_entries(&raw, &file)?;
        let mut merged: IndexSet<String> = current
            .iter()
            .filter(|entry| !is_shared_texture_path(entry))
            .cloned()
            .collect();
        merged.extend(textures.iter().cloned());
        let updated: Vec<String> = merged.into_iter().collect();
        if updated == current {
            return Ok(());
        }
        let mut doc = parse_document(&raw, &file)?;
        set_root_value(&mut doc, "disabled", string_array(updated));
        self.write(&file, &doc.to_string())
    }

    fn restore_disabled_mods(&self, raw: Option<&str>) {
        let file = self.game.disabled_mods_toml();
        match raw {
            Some(raw) => {
                let _ = self.fs.write(&file, raw);
            }
            None if file.is_file() => {
                let _ = self.fs.remove_file(&file);
            }
            None => {}
        }
    }

    fn backup_orphan(&self, from: &str) -> std::result::Result<(), yp_core::Error> {
        let orphan = self.inactive_dir(from);
        let orphan_sidecars: Vec<(Sidecar, PathBuf)> = SIDECARS
            .iter()
            .map(|sidecar| (*sidecar, sidecar.inactive_path(self.game, from)))
            .filter(|(_, path)| path.is_file())
            .collect();
        if !orphan.is_dir() && orphan_sidecars.is_empty() {
            return Ok(());
        }
        let backup_root = self.game.nams_dir().join(ORPHAN_BACKUP_DIR);
        ensure_dir(&backup_root)?;

        if orphan.is_dir() {
            let mut n = 1;
            let mut dest = backup_root.join(format!("{PROFILE_DIR_PREFIX}{from}_{n}"));
            while dest.is_dir() {
                n += 1;
                dest = backup_root.join(format!("{PROFILE_DIR_PREFIX}{from}_{n}"));
            }
            self.rename(&orphan, &dest)?;
        }
        for (sidecar, path) in orphan_sidecars {
            let mut n = 1;
            let mut dest = backup_root.join(format!("{}_{from}_{n}.toml", sidecar.backup_prefix));
            while dest.is_file() {
                n += 1;
                dest = backup_root.join(format!("{}_{from}_{n}.toml", sidecar.backup_prefix));
            }
            self.rename(&path, &dest)?;
        }
        Ok(())
    }

    fn move_sidecars(
        &self,
        from: &dyn Fn(&Sidecar) -> PathBuf,
        to: &dyn Fn(&Sidecar) -> PathBuf,
    ) -> std::result::Result<Vec<Sidecar>, yp_core::Error> {
        let mut moved: Vec<Sidecar> = Vec::new();
        for sidecar in &SIDECARS {
            let src = from(sidecar);
            if !src.is_file() {
                continue;
            }
            if let Err(source) = self.fs.rename(&src, &to(sidecar)) {
                self.restore_sidecars(&moved, to, from);
                return Err(io_error(IoOp::Rename, &src, source));
            }
            moved.push(*sidecar);
        }
        Ok(moved)
    }

    fn restore_sidecars(
        &self,
        moved: &[Sidecar],
        from: &dyn Fn(&Sidecar) -> PathBuf,
        to: &dyn Fn(&Sidecar) -> PathBuf,
    ) {
        for sidecar in moved {
            let _ = self.fs.rename(&from(sidecar), &to(sidecar));
        }
    }

    fn mods_root_for_profile(&self, profile: &str) -> PathBuf {
        let inactive = self.inactive_dir(profile);
        if inactive.is_dir() {
            inactive
        } else {
            self.mods_dir()
        }
    }

    fn plan_pack_upgrade_renames(
        &self,
        profiles: &[ModProfile],
    ) -> IndexMap<String, Option<String>> {
        let mut result = IndexMap::new();
        for profile in profiles {
            let mods_root = self.mods_root_for_profile(&profile.name);
            if !mods_root.is_dir() {
                continue;
            }
            for mod_dir in fsx::child_dirs(&mods_root) {
                if is_hidden_name(&mod_dir.name) {
                    continue;
                }
                let sidecar = sidecar_path(&mod_dir.path);
                let packs = read_packs_file(&sidecar);
                if packs.is_empty() {
                    continue;
                }
                let mut new_packs = Vec::with_capacity(packs.len());
                let mut any_changed = false;
                for pack in packs {
                    let upgraded = pack_name::upgrade(&pack, &profile.name, &mod_dir.name);
                    if upgraded != pack {
                        result.insert(pack, Some(upgraded.clone()));
                        any_changed = true;
                    }
                    new_packs.push(upgraded);
                }
                if any_changed {
                    let _ = self.fs.write(&sidecar, &render_packs(&new_packs));
                }
            }
        }
        result
    }

    fn plan_profile_prefix_renames(
        &self,
        old_profile: &str,
        new_profile: &str,
        is_active: bool,
    ) -> IndexMap<String, Option<String>> {
        let mut result = IndexMap::new();
        let mods_root = if is_active {
            self.mods_dir()
        } else {
            self.inactive_dir(old_profile)
        };
        if !mods_root.is_dir() {
            return result;
        }
        for mod_dir in fsx::child_dirs(&mods_root) {
            let sidecar = sidecar_path(&mod_dir.path);
            let packs = read_packs_file(&sidecar);
            if packs.is_empty() {
                continue;
            }
            let mut new_packs = Vec::with_capacity(packs.len());
            let mut any_changed = false;
            for pack in packs {
                let renamed = pack_name::swap_profile(&pack, old_profile, new_profile);
                if renamed != pack {
                    result.insert(pack, Some(renamed.clone()));
                    any_changed = true;
                }
                new_packs.push(renamed);
            }
            if any_changed {
                let _ = self.fs.write(&sidecar, &render_packs(&new_packs));
            }
        }
        result
    }

    fn apply_pack_renames(&self, renames: &IndexMap<String, Option<String>>) {
        if renames.is_empty() {
            return;
        }
        let inject_root = self.game.inject_textures_dir();
        if !inject_root.is_dir() {
            return;
        }
        for (from_name, to_name) in renames {
            let from = inject_root.join(from_name);
            if !from.is_dir() {
                continue;
            }
            match to_name {
                None => {
                    let _ = std::fs::remove_dir_all(&from);
                }
                Some(new_name) => {
                    let to = inject_root.join(new_name);
                    if to.is_dir() {
                        continue;
                    }
                    let _ = self.fs.rename(&from, &to);
                }
            }
        }
    }

    fn collect_profile_bundled_packs(&self, profile: &str) -> IndexSet<String> {
        let mut out = IndexSet::new();
        let root = self.inactive_dir(profile);
        if !root.is_dir() {
            return out;
        }
        for mod_dir in fsx::child_dirs(&root) {
            out.extend(read_packs_file(&sidecar_path(&mod_dir.path)));
        }
        out
    }

    fn compute_disabled_packs(&self, profiles: &[ModProfile], active: &str) -> Vec<String> {
        let mut out: IndexSet<String> = IndexSet::new();
        for profile in profiles {
            if profile.name == active {
                continue;
            }
            out.extend(self.collect_profile_bundled_packs(&profile.name));
        }
        let mut list: Vec<String> = out.into_iter().collect();
        list.sort_by(|a, b| crate::text::utf16_cmp(a, b));
        list
    }

    fn write_disabled_packs(&self, packs: &[String]) -> std::result::Result<(), yp_core::Error> {
        let file = self.game.texture_injection_toml();
        if !file.is_file() {
            return Ok(());
        }
        let text = read_text(&file)?;
        match with_disabled_packs(&text, packs) {
            Some(updated) => self.write(&file, &updated),
            None => Ok(()),
        }
    }

    fn rewrite_load_order(
        &self,
        renames: &IndexMap<String, Option<String>>,
    ) -> std::result::Result<(), yp_core::Error> {
        if renames.is_empty() {
            return Ok(());
        }
        let file = self.game.texture_injection_toml();
        if !file.is_file() {
            return Ok(());
        }
        let text = read_text(&file)?;
        match with_renamed_load_order(&text, renames) {
            Some(updated) => self.write(&file, &updated),
            None => Ok(()),
        }
    }

    fn write_profiles_toml(
        &self,
        active: &str,
        profiles: &[ModProfile],
    ) -> std::result::Result<(), yp_core::Error> {
        let mut doc = DocumentMut::new();
        doc.insert("active", value(active));
        if let Some(mut key) = doc.as_table_mut().key_mut("active") {
            key.leaf_decor_mut().set_prefix(PROFILES_HEADER);
        }
        let mut entries = ArrayOfTables::new();
        for profile in profiles {
            let mut table = Table::new();
            table.insert("name", value(profile.name.as_str()));
            table.insert("created_at", value(format_timestamp(profile.created_at)));
            entries.push(table);
        }
        doc.insert("profile", Item::ArrayOfTables(entries));
        self.write(&self.game.profiles_toml(), &doc.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_profile_names() {
        assert_eq!(sanitize_name("  My Profile  "), "my_profile");
        assert_eq!(sanitize_name("__.Hidden"), "hidden");
        assert_eq!(sanitize_name("a!!b"), "a_b");
        assert_eq!(sanitize_name("x!"), "x_");
        assert_eq!(sanitize_name("Ünïcode"), "n_code");
        assert_eq!(sanitize_name(&"a".repeat(60)).len(), MAX_NAME_LEN);
        assert_eq!(sanitize_name("tab\t\tname"), "tab_name");
    }

    #[test]
    fn validates_profile_names() {
        assert!(is_valid_name("default"));
        assert!(is_valid_name("9s-run_2"));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name("mods"));
        assert!(!is_valid_name("_x"));
        assert!(!is_valid_name("-x"));
        assert!(!is_valid_name("A"));
    }

    #[test]
    fn timestamps_round_trip_with_millisecond_precision() {
        let text = format_timestamp("2026-01-02T03:04:05.678912Z".parse().unwrap());
        assert_eq!(text, "2026-01-02T03:04:05.678Z");
        assert!(parse_timestamp(&text).is_some());
        assert!(parse_timestamp("2026-01-02T03:04:05").is_some());
        assert!(parse_timestamp("2026-01-02").is_some());
        assert!(parse_timestamp("yesterday").is_none());
    }
}
