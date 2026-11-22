use std::borrow::Cow;
use std::fmt::Display;

use yp_config::cli::NamsCli;
use yp_config::legacy::{
    SkippedNaiomEntry, detect_lodmod_ini, detect_naiom_ini, import_lodmod_ini, import_naiom_ini,
};
use yp_core::runtime::RuntimeSetup;
use yp_core::{AppPaths, GamePaths, HostEnv, Os, PlatformAdapter, SettingKey, SettingsStore};
use yp_host::game_probe::detect_hd_textures;
use yp_thirdparty::{ReShadeStatus, auto_disable_reshade_loading, detect_reshade};

pub const DETECTED_NOTIFICATIONS_DIR: SettingKey<Option<String>> =
    SettingKey::new("detected_notifications_dir", || None);

pub const FILES_QUARANTINED_ID: &str = "files_quarantined";
pub const LODMOD_MIGRATION_ID: &str = "lodmod_migration";
pub const NAIOM_MIGRATION_ID: &str = "naiom_migration";
pub const NAIOM_MIGRATION_SKIPPED_ID: &str = "naiom_migration_skipped";
pub const RESHADE_ID: &str = "reshade";
pub const RESHADE_INCOMPATIBLE_ID: &str = "reshade_incompatible";
pub const TEXTURES_ID_PREFIX: &str = "textures_";
pub const PLATFORM_UNSUPPORTED_ID: &str = "platform_unsupported";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationCategory {
    Migration,
    ReShade,
    Textures,
    Shortcut,
    General,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationKind {
    FilesQuarantined,
    LodModMigrated,
    NaiomMigrated,
    NaiomSkipped { entries: Vec<SkippedNaiomEntry> },
    ReShadeDetected,
    ReShadeIncompatible,
    TexturesDetected { folder: String },
    PlatformUnsupported { os: Os },
}

impl NotificationKind {
    pub fn id(&self) -> Cow<'static, str> {
        match self {
            NotificationKind::FilesQuarantined => Cow::Borrowed(FILES_QUARANTINED_ID),
            NotificationKind::LodModMigrated => Cow::Borrowed(LODMOD_MIGRATION_ID),
            NotificationKind::NaiomMigrated => Cow::Borrowed(NAIOM_MIGRATION_ID),
            NotificationKind::NaiomSkipped { .. } => Cow::Borrowed(NAIOM_MIGRATION_SKIPPED_ID),
            NotificationKind::ReShadeDetected => Cow::Borrowed(RESHADE_ID),
            NotificationKind::ReShadeIncompatible => Cow::Borrowed(RESHADE_INCOMPATIBLE_ID),
            NotificationKind::TexturesDetected { folder } => {
                Cow::Owned(format!("{TEXTURES_ID_PREFIX}{folder}"))
            }
            NotificationKind::PlatformUnsupported { .. } => Cow::Borrowed(PLATFORM_UNSUPPORTED_ID),
        }
    }

    pub fn category(&self) -> NotificationCategory {
        match self {
            NotificationKind::FilesQuarantined | NotificationKind::PlatformUnsupported { .. } => {
                NotificationCategory::General
            }
            NotificationKind::LodModMigrated
            | NotificationKind::NaiomMigrated
            | NotificationKind::NaiomSkipped { .. } => NotificationCategory::Migration,
            NotificationKind::ReShadeDetected | NotificationKind::ReShadeIncompatible => {
                NotificationCategory::ReShade
            }
            NotificationKind::TexturesDetected { .. } => NotificationCategory::Textures,
        }
    }
}

pub trait NotificationEntry {
    fn notification_id(&self) -> Cow<'_, str>;
}

impl NotificationEntry for NotificationKind {
    fn notification_id(&self) -> Cow<'_, str> {
        self.id()
    }
}

#[derive(Clone, Copy)]
pub struct DetectionEnv<'a> {
    pub settings: &'a SettingsStore,
    pub paths: &'a AppPaths,
    pub host: &'a HostEnv,
    pub cli: Option<&'a dyn NamsCli>,
}

fn quiet<T, E: Display>(step: &str, result: Result<T, E>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            log::warn!("startup detection step {step} failed: {error}");
            None
        }
    }
}

fn detect_lodmod_migration(env: &DetectionEnv<'_>, game: &GamePaths, found: &mut Vec<NotificationKind>) {
    let Some(cli) = env.cli else {
        return;
    };
    let Some(values) = quiet("LodMod.ini detection", detect_lodmod_ini(game)).flatten() else {
        return;
    };
    if quiet("LodMod.ini migration", import_lodmod_ini(cli, &values)) == Some(true) {
        found.push(NotificationKind::LodModMigrated);
    }
}

fn detect_naiom_migration(
    env: &DetectionEnv<'_>,
    game: &GamePaths,
    found: &mut Vec<NotificationKind>,
) {
    let Some(cli) = env.cli else {
        return;
    };
    let Some(values) = quiet("NAIOM detection", detect_naiom_ini(game)).flatten() else {
        return;
    };
    let Some(migration) = quiet("NAIOM migration", import_naiom_ini(cli, &values)) else {
        return;
    };
    if !migration.migrated {
        return;
    }
    found.push(NotificationKind::NaiomMigrated);
    if !migration.skipped.is_empty() {
        found.push(NotificationKind::NaiomSkipped {
            entries: migration.skipped,
        });
    }
}

fn detect_reshade_install(game: &GamePaths, found: &mut Vec<NotificationKind>) {
    match detect_reshade(game) {
        ReShadeStatus::Detected => {
            if quiet("ReShade auto-disable", auto_disable_reshade_loading(game)).is_some() {
                found.push(NotificationKind::ReShadeDetected);
            }
        }
        ReShadeStatus::IncompatibleAddon => found.push(NotificationKind::ReShadeIncompatible),
        ReShadeStatus::NotFound => {}
    }
}

pub fn detect(env: &DetectionEnv<'_>, game: &GamePaths) -> Vec<NotificationKind> {
    let mut found = Vec::new();
    if !RuntimeSetup::new(env.paths, env.host)
        .missing_files()
        .is_empty()
    {
        found.push(NotificationKind::FilesQuarantined);
    }
    detect_lodmod_migration(env, game, &mut found);
    detect_naiom_migration(env, game, &mut found);
    detect_reshade_install(game, &mut found);
    found.extend(detect_hd_textures(game.root()).into_iter().map(|folder| {
        NotificationKind::TexturesDetected {
            folder: folder.to_owned(),
        }
    }));
    found
}

fn detection_key(game: &GamePaths) -> String {
    game.root().to_string_lossy().into_owned()
}

fn remember_detected(env: &DetectionEnv<'_>, game: &GamePaths) {
    quiet(
        "detection cache",
        env.settings
            .set(&DETECTED_NOTIFICATIONS_DIR, &Some(detection_key(game))),
    );
}

pub fn already_detected(settings: &SettingsStore, game: &GamePaths) -> bool {
    settings.get(&DETECTED_NOTIFICATIONS_DIR) == Some(detection_key(game))
}

pub fn run_detections(env: &DetectionEnv<'_>, game: &GamePaths) -> Option<Vec<NotificationKind>> {
    if already_detected(env.settings, game) {
        return None;
    }
    remember_detected(env, game);
    Some(detect(env, game))
}

pub fn refresh_detections(env: &DetectionEnv<'_>, game: &GamePaths) -> Vec<NotificationKind> {
    remember_detected(env, game);
    detect(env, game)
}

pub fn reset_detection_cache_quiet(settings: &SettingsStore) -> bool {
    settings.remove(&DETECTED_NOTIFICATIONS_DIR).is_ok()
}

pub fn platform_support_notice(platform: &dyn PlatformAdapter) -> Option<NotificationKind> {
    (!platform.can_launch_game()).then(|| NotificationKind::PlatformUnsupported {
        os: platform.host().os(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationList<T = NotificationKind> {
    items: Vec<T>,
}

impl<T> Default for NotificationList<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T: NotificationEntry> NotificationList<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.items.iter().any(|item| item.notification_id() == id)
    }

    pub fn replace_all(&mut self, items: impl IntoIterator<Item = T>) {
        self.items = items.into_iter().collect();
    }

    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn dismiss(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item.notification_id() != id);
        self.items.len() != before
    }
}

impl<T: NotificationEntry + From<NotificationKind>> NotificationList<T> {
    pub fn apply_detections(&mut self, detected: Option<Vec<NotificationKind>>) -> bool {
        let Some(detected) = detected else {
            return false;
        };
        self.replace_all(detected.into_iter().map(T::from));
        true
    }

    pub fn check_platform_support(&mut self, platform: &dyn PlatformAdapter) -> bool {
        if self.contains(PLATFORM_UNSUPPORTED_ID) {
            return false;
        }
        let Some(notice) = platform_support_notice(platform) else {
            return false;
        };
        self.add(T::from(notice));
        true
    }
}
