use std::path::Path;

use yp_core::{CancelToken, GamePaths, Os};
use yp_cutscenes::CutsceneDetection;
use yp_health::notifications::{self, DetectionEnv, NotificationCategory, NotificationKind};
use yp_host::game_probe::{self, ExeVariant};
use yp_i18n::{Locale, Msg};
use yp_thirdparty::ReShadeStatus;

use crate::env::AppEnv;
use crate::notifications::{Notice, NoticeKind, NoticeSeverity};

const HD_TEXTURE_DIR: &str = "SK_Res";
const DISABLED_PACKS_KEY: &str = "disabled_packs";
const ENTRY_SEPARATOR: &str = ", ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextureChip {
    pub active_packs: usize,
    pub has_sk_res: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionSnapshot {
    pub dlc: bool,
    pub exe_variant: ExeVariant,
    pub lodmod_enabled: bool,
    pub reshade: ReShadeStatus,
    pub textures: TextureChip,
    pub mods: usize,
    pub cutscenes: CutsceneDetection,
}

impl DetectionSnapshot {
    pub fn reshade_detected(&self) -> bool {
        self.reshade == ReShadeStatus::Detected
    }

    pub fn has_textures(&self) -> bool {
        self.textures.active_packs > 0 || self.textures.has_sk_res
    }

    pub fn has_mods(&self) -> bool {
        self.mods > 0
    }

    pub fn has_hd_cutscenes(&self) -> bool {
        self.cutscenes.files_scanned > 0 && self.cutscenes.has_hd_cutscenes
    }
}

fn os_label(os: Os) -> &'static str {
    let strings = Locale::FALLBACK.strings();
    match os {
        Os::Windows => strings.platform_name_windows(),
        Os::Linux => strings.platform_name_linux(),
        Os::MacOs => strings.platform_name_macos(),
    }
}

pub(crate) fn notice_for(found: NotificationKind) -> Notice {
    let id = found.id().into_owned();
    let kind = match found.category() {
        NotificationCategory::Migration => NoticeKind::Migration,
        NotificationCategory::ReShade => NoticeKind::Reshade,
        NotificationCategory::Textures => NoticeKind::Textures,
        NotificationCategory::Shortcut => NoticeKind::Shortcut,
        NotificationCategory::General => NoticeKind::General,
    };
    let (msg, severity) = match found {
        NotificationKind::FilesQuarantined => (Msg::NotifyFilesQuarantined, NoticeSeverity::Error),
        NotificationKind::LodModMigrated => (Msg::NotifyLodModMigrated, NoticeSeverity::Success),
        NotificationKind::NaiomMigrated => (Msg::NotifyNaiomMigrated, NoticeSeverity::Success),
        NotificationKind::NaiomSkipped { entries } => (
            Msg::NotifyNaiomSkipped {
                entries: entries
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(ENTRY_SEPARATOR),
            },
            NoticeSeverity::Warning,
        ),
        NotificationKind::ReShadeDetected => (Msg::NotifyReShadeDetected, NoticeSeverity::Accent),
        NotificationKind::ReShadeIncompatible => {
            (Msg::NotifyReShadeIncompatible, NoticeSeverity::Warning)
        }
        NotificationKind::TexturesDetected { folder } => (
            Msg::NotifyTexturesDetected { folder },
            NoticeSeverity::Accent,
        ),
        NotificationKind::PlatformUnsupported { os } => (
            Msg::NotifyPlatformUnsupported {
                platform: os_label(os).to_owned(),
            },
            NoticeSeverity::Warning,
        ),
    };
    Notice::new(id, msg, kind, severity)
}

pub(crate) fn detect_notices(env: &AppEnv, game: &GamePaths, force: bool) -> Option<Vec<Notice>> {
    let detection = DetectionEnv {
        settings: &env.settings,
        paths: &env.paths,
        host: &env.host,
    };
    let found = if force {
        Some(notifications::refresh_detections(&detection, game))
    } else {
        notifications::run_detections(&detection, game)
    };
    found.map(|found| found.into_iter().map(notice_for).collect())
}

pub(crate) fn platform_notice(env: &AppEnv) -> Option<Notice> {
    notifications::platform_support_notice(env.platform.as_ref()).map(notice_for)
}

fn lodmod_enabled(game: &GamePaths) -> bool {
    yp_core::fs::read_text_opt(&game.lodmod_toml())
        .ok()
        .flatten()
        .and_then(|raw| yp_core::toml_io::parse_values(&raw).ok())
        .is_some_and(|values| yp_config::fields::lodmod::ENABLED.value_in(&values))
}

fn entry_names(dir: &Path) -> Vec<(String, bool)> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| {
                    let is_dir = entry.file_type().is_ok_and(|kind| kind.is_dir());
                    (entry.file_name().to_string_lossy().into_owned(), is_dir)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn installed_mod_count(game: &GamePaths) -> usize {
    entry_names(&game.mods_dir())
        .into_iter()
        .filter(|(name, is_dir)| *is_dir && !name.starts_with('.') && !name.starts_with('_'))
        .count()
}

fn disabled_texture_packs(game: &GamePaths) -> Vec<String> {
    yp_core::fs::read_text_opt(&game.texture_injection_toml())
        .ok()
        .flatten()
        .and_then(|raw| yp_core::toml_io::parse_values(&raw).ok())
        .and_then(|values| values.get(DISABLED_PACKS_KEY).cloned())
        .and_then(|packs| packs.as_array().cloned())
        .map(|packs| {
            packs
                .iter()
                .filter_map(|pack| pack.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn texture_chip(game: &GamePaths) -> TextureChip {
    let disabled = disabled_texture_packs(game);
    let active_packs = entry_names(&game.inject_textures_dir())
        .into_iter()
        .filter(|(name, _)| !disabled.contains(name))
        .count();
    TextureChip {
        active_packs,
        has_sk_res: game.root().join(HD_TEXTURE_DIR).is_dir(),
    }
}

pub(crate) fn probe(game: &GamePaths, cancel: &CancelToken) -> DetectionSnapshot {
    DetectionSnapshot {
        dlc: game_probe::has_dlc(game.root()),
        exe_variant: game_probe::detect_exe_variant(game.root()),
        lodmod_enabled: lodmod_enabled(game),
        reshade: yp_thirdparty::detect_reshade(game),
        textures: texture_chip(game),
        mods: installed_mod_count(game),
        cutscenes: yp_cutscenes::detect(game, cancel).unwrap_or_default(),
    }
}
