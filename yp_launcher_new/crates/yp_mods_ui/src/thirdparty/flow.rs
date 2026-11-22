use std::path::PathBuf;

use yp_thirdparty::{
    Classification, DropRoute, GameModSettings, InstallOutcome, RuntimePresence, RuntimeStatus,
    StageFailure, ThirdPartyKind, ThirdPartyRuntime, ThirdPartyUpdateInfo,
};
use yp_state::archive_failure_key;
use yp_ui::{IconName, Msg, Severity, Strings};

use crate::text::{dll_label, seconds_label};

pub const PATCH_TIMEOUTS_MS: [u64; 5] = [5_000, 10_000, 20_000, 30_000, 60_000];
pub const REAL_IMAGE_SIZE: u64 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropNext {
    Install,
    OfferLodModImport(PathBuf),
    Tell(Msg, Severity),
}

pub fn drop_next(classification: &Classification) -> DropNext {
    if classification.kind == ThirdPartyKind::LodMod
        && let Some(ini) = &classification.lodmod_ini
    {
        return DropNext::OfferLodModImport(ini.clone());
    }
    if classification.kind == ThirdPartyKind::NamsDll {
        return DropNext::Tell(Msg::ThirdPartyNamsDll, Severity::Accent);
    }
    match classification.kind.drop_route() {
        DropRoute::Install => DropNext::Install,
        DropRoute::ModsTab => DropNext::Tell(Msg::ThirdPartyRedirectMods, Severity::Accent),
        DropRoute::TexturesTab => DropNext::Tell(Msg::ThirdPartyRedirectTextures, Severity::Accent),
        DropRoute::Unsupported => DropNext::Tell(Msg::ThirdPartyUnsupported, Severity::Warning),
    }
}

pub fn stage_failure_notice(failure: &StageFailure) -> Option<(Msg, Severity)> {
    match failure {
        StageFailure::Cancelled => None,
        StageFailure::UnsupportedFile => Some((Msg::ThirdPartyUnsupported, Severity::Warning)),
        StageFailure::ExtractFailed(error) => Some((
            error
                .archive_failure()
                .map_or(Msg::ThirdPartyInstallFailed, |failure| Msg::from(archive_failure_key(failure))),
            Severity::Error,
        )),
        StageFailure::ExecutableNotSupported | StageFailure::Io(_) => {
            Some((Msg::ThirdPartyInstallFailed, Severity::Error))
        }
    }
}

pub fn install_notice<E>(result: &Result<Option<InstallOutcome>, E>) -> (Msg, Severity) {
    match result {
        Ok(Some(outcome)) if outcome.is_ok() => (Msg::ThirdPartyInstalled, Severity::Success),
        _ => (Msg::ThirdPartyInstallFailed, Severity::Error),
    }
}

pub fn lodmod_import_notice(imported: Option<bool>) -> (Msg, Severity) {
    match imported {
        Some(true) => (Msg::ModsUiLodmodImported, Severity::Success),
        Some(false) => (Msg::ModsUiLodmodNotImported, Severity::Info),
        None => (Msg::ThirdPartyInstallFailed, Severity::Error),
    }
}

pub fn runtime_name(which: ThirdPartyRuntime, strings: &Strings) -> &'static str {
    match which {
        ThirdPartyRuntime::ReShade => strings.mods_ui_name_reshade(),
        ThirdPartyRuntime::Migoto => strings.mods_ui_name_migoto(),
        ThirdPartyRuntime::GameMods => strings.third_party_game_mods_header(),
        ThirdPartyRuntime::GraphicMods => strings.third_party_graphic_mods_header(),
    }
}

pub fn update_body(update: &ThirdPartyUpdateInfo, strings: &Strings) -> String {
    let name = runtime_name(update.runtime, strings);
    let installed = strings.mods_ui_label_value(
        strings.third_party_update_installed_label(),
        &dll_label(&update.installed, strings),
    );
    let incoming = strings.mods_ui_label_value(
        strings.third_party_update_incoming_label(),
        &dll_label(&update.incoming, strings),
    );
    format!(
        "{}\n\n{installed}\n{incoming}",
        strings.third_party_update_body(name)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusLine {
    pub icon: IconName,
    pub severity: Option<Severity>,
    pub can_import: bool,
}

pub fn status_line(presence: RuntimePresence) -> StatusLine {
    match presence {
        RuntimePresence::Installed => StatusLine {
            icon: IconName::CheckCircleOutline,
            severity: Some(Severity::Success),
            can_import: false,
        },
        RuntimePresence::FoundInGameRoot => StatusLine {
            icon: IconName::InfoOutline,
            severity: Some(Severity::Warning),
            can_import: true,
        },
        RuntimePresence::NotInstalled => StatusLine {
            icon: IconName::RadioButtonUnchecked,
            severity: None,
            can_import: false,
        },
    }
}

pub fn status_text(presence: RuntimePresence, status: &RuntimeStatus, strings: &Strings) -> String {
    match presence {
        RuntimePresence::Installed if status.preset_count > 0 => strings.mods_ui_dot_join(
            strings.third_party_status_installed(),
            &strings
                .third_party_presets_count(i64::try_from(status.preset_count).unwrap_or(i64::MAX)),
        ),
        RuntimePresence::Installed => strings.third_party_status_installed().to_owned(),
        RuntimePresence::FoundInGameRoot => strings.third_party_status_found_in_game().to_owned(),
        RuntimePresence::NotInstalled => strings.third_party_status_not_installed().to_owned(),
    }
}

pub fn patch_timeout_options(current_ms: u64) -> Vec<u64> {
    let mut options = PATCH_TIMEOUTS_MS.to_vec();
    if !options.contains(&current_ms) {
        options.push(current_ms);
    }
    options
}

pub fn patch_timeout_label(milliseconds: u64, strings: &Strings) -> String {
    strings.third_party_game_mod_seconds(&seconds_label(milliseconds))
}

pub fn image_size_options(current: u64) -> Vec<u64> {
    let mut options = vec![
        GameModSettings::STEAM_2021_SIZE_OF_IMAGE,
        GameModSettings::STEAM_2026_SIZE_OF_IMAGE,
        REAL_IMAGE_SIZE,
    ];
    if !options.contains(&current) {
        options.push(current);
    }
    options
}

pub fn image_size_label(size: u64, strings: &Strings) -> String {
    if size == GameModSettings::STEAM_2021_SIZE_OF_IMAGE {
        strings.third_party_game_mod_size2021().to_owned()
    } else if size == GameModSettings::STEAM_2026_SIZE_OF_IMAGE {
        strings.third_party_game_mod_size2026().to_owned()
    } else if size == REAL_IMAGE_SIZE {
        strings.third_party_game_mod_size_real().to_owned()
    } else {
        strings.third_party_game_mod_size_custom(i64::try_from(size).unwrap_or(i64::MAX))
    }
}

pub fn dll_row_label(label: &str, size_bytes: Option<u64>, strings: &Strings) -> String {
    match size_bytes {
        Some(bytes) => strings.mods_ui_dot_join(label, &yp_core::fs::size_label(bytes)),
        None => label.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_thirdparty::{DllSummary, InstallFailure};
    use yp_ui::Locale;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    fn classified(kind: ThirdPartyKind) -> Classification {
        Classification::new(kind, PathBuf::from("staged"))
    }

    #[test]
    fn installable_kinds_install_and_other_tabs_are_named() {
        for kind in [
            ThirdPartyKind::ReShadeWholeInstall,
            ThirdPartyKind::ReShadePreset,
            ThirdPartyKind::Migoto,
            ThirdPartyKind::GameMod,
            ThirdPartyKind::GraphicMod,
        ] {
            assert_eq!(drop_next(&classified(kind)), DropNext::Install, "{kind:?}");
        }
        assert_eq!(
            drop_next(&classified(ThirdPartyKind::GameData)),
            DropNext::Tell(Msg::ThirdPartyRedirectMods, Severity::Accent)
        );
        assert_eq!(
            drop_next(&classified(ThirdPartyKind::Textures)),
            DropNext::Tell(Msg::ThirdPartyRedirectTextures, Severity::Accent)
        );
        assert_eq!(
            drop_next(&classified(ThirdPartyKind::NamsDll)),
            DropNext::Tell(Msg::ThirdPartyNamsDll, Severity::Accent)
        );
        for kind in [
            ThirdPartyKind::SpecialK,
            ThirdPartyKind::SkShaderEdit,
            ThirdPartyKind::BrokenLegacy,
            ThirdPartyKind::MultiBundle,
            ThirdPartyKind::Unknown,
            ThirdPartyKind::LodMod,
        ] {
            assert_eq!(
                drop_next(&classified(kind)),
                DropNext::Tell(Msg::ThirdPartyUnsupported, Severity::Warning),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn a_lodmod_drop_with_its_ini_offers_the_settings_import() {
        let mut classification = classified(ThirdPartyKind::LodMod);
        classification.lodmod_ini = Some(PathBuf::from("staged/LodMod.ini"));
        assert_eq!(
            drop_next(&classification),
            DropNext::OfferLodModImport(PathBuf::from("staged/LodMod.ini"))
        );
        assert_eq!(
            lodmod_import_notice(Some(true)),
            (Msg::ModsUiLodmodImported, Severity::Success)
        );
        assert_eq!(
            lodmod_import_notice(Some(false)),
            (Msg::ModsUiLodmodNotImported, Severity::Info)
        );
        assert_eq!(
            lodmod_import_notice(None),
            (Msg::ThirdPartyInstallFailed, Severity::Error)
        );
    }

    #[test]
    fn staging_failures_are_unsupported_failed_or_silent() {
        assert_eq!(stage_failure_notice(&StageFailure::Cancelled), None);
        assert_eq!(
            stage_failure_notice(&StageFailure::UnsupportedFile),
            Some((Msg::ThirdPartyUnsupported, Severity::Warning))
        );
        for failure in [
            StageFailure::ExecutableNotSupported,
            StageFailure::ExtractFailed(yp_core::Error::Cancelled),
            StageFailure::Io(yp_core::Error::Cancelled),
        ] {
            assert_eq!(
                stage_failure_notice(&failure),
                Some((Msg::ThirdPartyInstallFailed, Severity::Error))
            );
        }
    }

    #[test]
    fn a_failed_archive_drop_says_why() {
        let archive = std::path::Path::new("shaders.rar");
        for (failure, msg) in [
            (yp_core::ArchiveFailure::PasswordRequired, Msg::ArchivePasswordProtected),
            (yp_core::ArchiveFailure::Corrupt, Msg::ArchiveDamaged),
            (yp_core::ArchiveFailure::UnsupportedMethod, Msg::ArchiveUnsupportedMethod),
            (yp_core::ArchiveFailure::MissingVolume, Msg::ArchivePartMissing),
            (yp_core::ArchiveFailure::NoFiles, Msg::ArchiveHoldsNoFiles),
            (yp_core::ArchiveFailure::DiskFull, Msg::ExtractFailedDiskFull),
        ] {
            assert_eq!(
                stage_failure_notice(&StageFailure::ExtractFailed(yp_core::Error::archive(archive, failure))),
                Some((msg, Severity::Error))
            );
        }
    }

    #[test]
    fn install_results_only_succeed_for_an_installed_outcome() {
        let installed: Result<_, ()> =
            Ok(Some(InstallOutcome::installed(ThirdPartyRuntime::ReShade)));
        assert_eq!(
            install_notice(&installed),
            (Msg::ThirdPartyInstalled, Severity::Success)
        );
        let rejected: Result<_, ()> = Ok(Some(InstallOutcome::Rejected(InstallFailure::NoDll)));
        let nothing: Result<Option<InstallOutcome>, ()> = Ok(None);
        let failed: Result<Option<InstallOutcome>, ()> = Err(());
        for result in [rejected, nothing, failed] {
            assert_eq!(
                install_notice(&result),
                (Msg::ThirdPartyInstallFailed, Severity::Error)
            );
        }
    }

    #[test]
    fn card_status_follows_the_presence() {
        let strings = strings();
        let mut status = RuntimeStatus::default();
        assert!(!status_line(RuntimePresence::NotInstalled).can_import);
        assert!(status_line(RuntimePresence::FoundInGameRoot).can_import);
        assert_eq!(
            status_line(RuntimePresence::Installed).severity,
            Some(Severity::Success)
        );
        assert_eq!(
            status_text(RuntimePresence::NotInstalled, &status, strings),
            strings.third_party_status_not_installed()
        );
        assert_eq!(
            status_text(RuntimePresence::FoundInGameRoot, &status, strings),
            strings.third_party_status_found_in_game()
        );
        assert_eq!(
            status_text(RuntimePresence::Installed, &status, strings),
            strings.third_party_status_installed()
        );
        status.preset_count = 2;
        assert!(status_text(RuntimePresence::Installed, &status, strings).contains("2 presets"));
    }

    #[test]
    fn the_update_question_shows_both_builds() {
        let update = ThirdPartyUpdateInfo {
            runtime: ThirdPartyRuntime::ReShade,
            installed: DllSummary {
                file_name: "dxgi.dll".to_owned(),
                version: Some("6.2.0".to_owned()),
                size_bytes: Some(2 * 1024 * 1024),
            },
            incoming: DllSummary {
                file_name: "dxgi.dll".to_owned(),
                version: Some("6.3.1".to_owned()),
                size_bytes: None,
            },
        };
        let body = update_body(&update, strings());
        assert!(body.contains("ReShade"));
        assert!(body.contains("6.2.0 \u{b7} 2.0 MB"));
        assert!(body.contains("6.3.1"));
    }

    #[test]
    fn dropdown_options_keep_a_custom_current_value() {
        assert_eq!(patch_timeout_options(10_000), PATCH_TIMEOUTS_MS.to_vec());
        assert_eq!(patch_timeout_options(2_500).last(), Some(&2_500));
        assert_eq!(patch_timeout_label(2_500, strings()), "2.5 s");
        assert_eq!(patch_timeout_label(60_000, strings()), "60 s");

        assert_eq!(
            image_size_options(GameModSettings::STEAM_2026_SIZE_OF_IMAGE).len(),
            3
        );
        assert_eq!(image_size_options(123).last(), Some(&123));
        assert_eq!(
            image_size_label(REAL_IMAGE_SIZE, strings()),
            strings().third_party_game_mod_size_real()
        );
        assert_eq!(image_size_label(123, strings()), "Custom (123)");
        assert_eq!(
            dll_row_label("timer.dll", Some(2048), strings()),
            "timer.dll \u{b7} 2 KB"
        );
        assert_eq!(dll_row_label("timer.dll", None, strings()), "timer.dll");
    }
}
