use gpui::SharedString;
use yp_core::consts::GAME_EXE_NAME;
use yp_core::{LaunchUnavailable, Os, SelectionRejection};
use yp_host::launch::failure::{
    FailureCategory, FailureHint, FailureKind, HintLine, LaunchFailure,
};
use yp_host::nams_cli::verify::{VerifyOutcome, VerifyStatus};
use yp_host::platform::VerifyTimeoutHint;
use yp_i18n::Strings;
use yp_state::{GameError, LaunchStatus, NoticeKind, SelectionCheck, SessionError};
use yp_ui::{IconName, Severity};

const LIST_SEPARATOR: &str = ", ";
const SUBTITLE_SEPARATOR: &str = "  ·  ";

fn shown(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn failure_title(failure: &LaunchFailure, strings: &Strings) -> SharedString {
    let title = match failure.category() {
        FailureCategory::Panic => strings.fail_title_panic(),
        FailureCategory::Unknown => strings.fail_title_unknown(),
        FailureCategory::NamsFailure => strings.fail_title_nams_failure(),
        FailureCategory::InstallNotFound => strings.fail_title_install_not_found(),
        FailureCategory::FolderCreate => strings.fail_title_folder_create(),
        FailureCategory::RuntimePrep => strings.fail_title_runtime_prep(),
        FailureCategory::HostFailure => strings.fail_title_host_failure(),
        FailureCategory::SteamNotRunning => strings.fail_title_steam_not_running(),
        FailureCategory::SteamNotOwned => strings.fail_title_steam_not_owned(),
        FailureCategory::SteamCheckFailed => strings.fail_title_steam_check_failed(),
        FailureCategory::InvalidArgs => strings.fail_title_invalid_args(),
        FailureCategory::ExitedUnexpectedly => strings.fail_title_exited_unexpectedly(),
    };
    SharedString::new_static(title)
}

pub fn failure_headline(failure: &LaunchFailure, strings: &Strings) -> String {
    match &failure.kind {
        FailureKind::FilesMissing { missing, .. } => {
            strings.error_files_quarantined(&missing.join(LIST_SEPARATOR))
        }
        FailureKind::LauncherDirNotWritable { .. } => strings.error_dir_not_writable().to_owned(),
        FailureKind::ExeNotFound { game_dir, .. } => strings.error_exe_not_found(&shown(game_dir)),
        FailureKind::GameDirNotWritable { .. } => strings.error_game_dir_not_writable().to_owned(),
        FailureKind::NamsDllMissing { .. } => strings.shell_nams_dll_missing().to_owned(),
        FailureKind::NamsDllCopyFailed { .. } => strings.shell_nams_dll_copy_failed().to_owned(),
        FailureKind::Unavailable(LaunchUnavailable::NoCompatLayer { os }) => match os {
            Os::Linux => strings.error_no_compat_layer_linux().to_owned(),
            Os::Windows | Os::MacOs => strings.error_no_compat_layer().to_owned(),
        },
        FailureKind::Unavailable(LaunchUnavailable::ProtonMissing { .. }) => {
            strings.error_proton_missing().to_owned()
        }
        FailureKind::Unavailable(LaunchUnavailable::NoZDrive { .. }) => {
            strings.error_no_z_drive().to_owned()
        }
        FailureKind::Reported(report) => report.headline.clone(),
        FailureKind::Exited { code } => strings.shell_nams_exited_with_code(i64::from(*code)),
        FailureKind::Internal { .. } => strings.shell_internal_launcher_error().to_owned(),
    }
}

pub fn failure_explanation(failure: &LaunchFailure, strings: &Strings) -> String {
    let explanation = match failure.category() {
        FailureCategory::Unknown => return failure_headline(failure, strings),
        FailureCategory::Panic => strings.fail_explanation_panic(),
        FailureCategory::NamsFailure => strings.fail_explanation_nams_failure(),
        FailureCategory::InstallNotFound => strings.fail_explanation_install_not_found(),
        FailureCategory::FolderCreate => strings.fail_explanation_folder_create(),
        FailureCategory::RuntimePrep => strings.fail_explanation_runtime_prep(),
        FailureCategory::HostFailure => strings.fail_explanation_host_failure(),
        FailureCategory::SteamNotRunning => strings.fail_explanation_steam_not_running(),
        FailureCategory::SteamNotOwned => strings.fail_explanation_steam_not_owned(),
        FailureCategory::SteamCheckFailed => strings.fail_explanation_steam_check_failed(),
        FailureCategory::InvalidArgs => strings.fail_explanation_invalid_args(),
        FailureCategory::ExitedUnexpectedly => strings.fail_explanation_exited_unexpectedly(),
    };
    explanation.to_owned()
}

pub fn failure_detail(failure: &LaunchFailure, strings: &Strings) -> String {
    if !failure.raw_output.is_empty() {
        return failure.raw_output.clone();
    }
    match &failure.kind {
        FailureKind::FilesMissing {
            missing,
            launcher_dir,
        } => strings.shell_missing_files_detail(&shown(launcher_dir), &missing.join("\n")),
        FailureKind::LauncherDirNotWritable { dir } => {
            strings.error_dir_not_writable_body(&shown(dir))
        }
        FailureKind::ExeNotFound { exe, .. } => strings.shell_exe_not_found_detail(&shown(exe)),
        FailureKind::GameDirNotWritable { game_dir, nams_dir } => {
            strings.error_game_dir_not_writable_body(&shown(game_dir), &shown(nams_dir))
        }
        FailureKind::NamsDllMissing { dll } => strings.shell_nams_dll_missing_detail(&shown(dll)),
        FailureKind::NamsDllCopyFailed { dll, target, detail } => {
            strings.shell_nams_dll_copy_failed_detail(&shown(dll), &shown(target), detail)
        }
        FailureKind::Unavailable(LaunchUnavailable::NoCompatLayer { os }) => match os {
            Os::Linux => strings.error_no_compat_layer_linux_body().to_owned(),
            Os::Windows | Os::MacOs => strings.error_no_compat_layer_body().to_owned(),
        },
        FailureKind::Unavailable(LaunchUnavailable::ProtonMissing { proton }) => {
            strings.error_proton_missing_body(&shown(proton))
        }
        FailureKind::Unavailable(LaunchUnavailable::NoZDrive { prefix }) => {
            strings.error_no_z_drive_body(&shown(prefix))
        }
        FailureKind::Reported(report) => report.headline.clone(),
        FailureKind::Exited { .. } => failure_headline(failure, strings),
        FailureKind::Internal { detail } => detail.clone(),
    }
}

pub fn failure_subtitle(failure: &LaunchFailure) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(code) = failure.code() {
        let tag = if failure.is_panic() { "PANIC" } else { "ERROR" };
        parts.push(format!("{tag}[{code}]"));
    }
    if let Some(phase) = failure.report().and_then(|report| report.phase.clone()) {
        parts.push(phase);
    }
    (!parts.is_empty()).then(|| parts.join(SUBTITLE_SEPARATOR))
}

fn hint_text(hint: FailureHint, strings: &Strings) -> &'static str {
    match hint {
        FailureHint::ShareReport => strings.fail_hint_share_report(),
        FailureHint::RepickDirectory => strings.fail_hint_repick_directory(),
        FailureHint::VerifyFiles => strings.fail_hint_verify_files(),
        FailureHint::WritableFolder => strings.fail_hint_writable_folder(),
        FailureHint::ProgramFiles => strings.fail_hint_program_files(),
        FailureHint::AntivirusExclusions => strings.fail_hint_antivirus_exclusions(),
        FailureHint::WritableCache => strings.fail_hint_writable_cache(),
        FailureHint::Reboot => strings.fail_hint_reboot(),
        FailureHint::PersistShare => strings.fail_hint_persist_share(),
        FailureHint::StartSteam => strings.fail_hint_start_steam(),
        FailureHint::SignInOwner => strings.fail_hint_sign_in_owner(),
        FailureHint::RestartSteam => strings.fail_hint_restart_steam(),
        FailureHint::PanicShare => strings.fail_hint_panic_share(),
        FailureHint::PanicReboot => strings.fail_hint_panic_reboot(),
        FailureHint::CheckLogViewer => strings.fail_hint_check_log_viewer(),
    }
}

pub fn failure_hints(failure: &LaunchFailure, strings: &Strings) -> Vec<SharedString> {
    failure
        .hints()
        .into_iter()
        .map(|line| match line {
            HintLine::FromNams(text) => SharedString::from(text),
            HintLine::Known(hint) => SharedString::new_static(hint_text(hint, strings)),
        })
        .collect()
}

pub fn rejection_text(rejection: &SelectionRejection, strings: &Strings) -> String {
    match rejection {
        SelectionRejection::OutsidePrefix { exe } => {
            strings.error_exe_outside_prefix_body(GAME_EXE_NAME, &shown(exe))
        }
    }
}

pub fn session_error_text(error: &SessionError, strings: &Strings) -> SharedString {
    match error {
        SessionError::InvalidExe => SharedString::new_static(strings.error_invalid_exe()),
        SessionError::FileNotExist => SharedString::new_static(strings.error_file_not_exist()),
        SessionError::Rejected(rejection) => rejection_text(rejection, strings).into(),
        SessionError::Message(message) => SharedString::from(message.clone()),
    }
}

pub fn game_error_text(error: &GameError, strings: &Strings) -> SharedString {
    match error {
        GameError::Session(error) => session_error_text(error, strings),
        GameError::LaunchFailed(failure) => failure_title(failure, strings),
        GameError::StopFailed => SharedString::new_static(strings.error_stop_failed()),
    }
}

pub fn selection_error(check: &SelectionCheck) -> Option<SessionError> {
    match check {
        SelectionCheck::Accepted => None,
        SelectionCheck::Missing => Some(SessionError::FileNotExist),
        SelectionCheck::NotNierExe => Some(SessionError::InvalidExe),
        SelectionCheck::Rejected(rejection) => Some(SessionError::Rejected(rejection.clone())),
    }
}

pub fn launch_status_text(status: LaunchStatus, strings: &Strings) -> &'static str {
    match status {
        LaunchStatus::Launching => strings.status_launching(),
        LaunchStatus::Running => strings.status_running(),
        LaunchStatus::Stopped => strings.status_stopped(),
        LaunchStatus::Stopping => strings.status_stopping(),
    }
}

pub fn platform_unsupported_text(os: Os, strings: &Strings) -> &'static str {
    match os {
        Os::MacOs => strings.platform_unsupported_macos(),
        Os::Linux | Os::Windows => strings.platform_unsupported_linux(),
    }
}

pub fn verify_summary(
    outcome: &VerifyOutcome,
    timeout_hint: VerifyTimeoutHint,
    strings: &Strings,
) -> &'static str {
    match outcome.status {
        VerifyStatus::Ok => strings.verify_install_ok(),
        VerifyStatus::Failed => strings.verify_install_failed(),
        VerifyStatus::NoRuntime => strings.verify_no_runtime(),
        VerifyStatus::SteamNotRunning => strings.verify_steam_not_running(),
        VerifyStatus::TimedOut => match timeout_hint {
            VerifyTimeoutHint::Generic => strings.verify_timed_out(),
            VerifyTimeoutHint::Linux => strings.verify_timed_out_linux(),
        },
        VerifyStatus::CannotParse | VerifyStatus::Error => strings.verify_install_error(),
    }
}

pub fn verify_check_label(name: &str, strings: &Strings) -> SharedString {
    let label = match name {
        "steam_install" => strings.verify_check_steam_install(),
        "nier_exe" => strings.verify_check_nier_exe(),
        "steam_api64_source" => strings.verify_check_steam_api64(),
        "runtime_writable" => strings.verify_check_runtime_writable(),
        "runtime_library_cached" => strings.verify_check_runtime_cached(),
        other => return SharedString::from(other.to_owned()),
    };
    SharedString::new_static(label)
}

pub fn notice_icon(kind: NoticeKind, severity: Severity) -> IconName {
    match (kind, severity) {
        (_, Severity::Error) => IconName::ErrorOutline,
        (NoticeKind::Reshade, _) => IconName::AutoFixHigh,
        (NoticeKind::Textures, _) => IconName::Texture,
        (_, Severity::Warning) => IconName::WarningAmber,
        (_, Severity::Success) => IconName::CheckCircle,
        (NoticeKind::Migration, _) => IconName::CheckCircle,
        (NoticeKind::Shortcut | NoticeKind::General, _) => IconName::InfoOutline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use yp_host::launch::failure::NamsReport;
    use yp_i18n::Locale;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    fn reported(code: i32, panic: bool, fix: Option<&str>) -> LaunchFailure {
        LaunchFailure::new(
            FailureKind::Reported(NamsReport {
                code: Some(code),
                panic,
                headline: "steam is not running".to_owned(),
                phase: Some("steam".to_owned()),
                os_error: None,
                cause: None,
                fix: fix.map(str::to_owned),
            }),
            "raw",
        )
    }

    #[test]
    fn reported_failures_use_the_category_texts() {
        let failure = reported(20, false, Some("start steam first"));
        assert_eq!(
            failure_title(&failure, strings()).as_ref(),
            strings().fail_title_steam_not_running()
        );
        assert_eq!(
            failure_explanation(&failure, strings()),
            strings().fail_explanation_steam_not_running()
        );
        let hints = failure_hints(&failure, strings());
        assert_eq!(hints[0].as_ref(), "start steam first");
        assert_eq!(hints[1].as_ref(), strings().fail_hint_start_steam());
        assert_eq!(
            failure_subtitle(&failure).as_deref(),
            Some("ERROR[20]  ·  steam")
        );
    }

    #[test]
    fn panics_get_the_panic_texts_and_tag() {
        let failure = reported(1, true, Some("ignored"));
        assert_eq!(
            failure_title(&failure, strings()).as_ref(),
            strings().fail_title_panic()
        );
        let hints = failure_hints(&failure, strings());
        assert_eq!(hints.len(), 2);
        assert_eq!(hints[0].as_ref(), strings().fail_hint_panic_share());
        assert!(failure_subtitle(&failure).unwrap().starts_with("PANIC[1]"));
    }

    #[test]
    fn a_missing_or_uncopyable_nams_dll_explains_itself() {
        let missing = LaunchFailure::new(
            FailureKind::NamsDllMissing {
                dll: PathBuf::from("D:/builds/nams.dll"),
            },
            String::new(),
        );
        assert_eq!(failure_explanation(&missing, strings()), strings().shell_nams_dll_missing());
        assert!(failure_detail(&missing, strings()).contains("D:/builds/nams.dll"));
        let copy = LaunchFailure::new(
            FailureKind::NamsDllCopyFailed {
                dll: PathBuf::from("D:/builds/nams.dll"),
                target: PathBuf::from("G:/game/dinput8.dll"),
                detail: "Access is denied".to_owned(),
            },
            String::new(),
        );
        assert_eq!(failure_explanation(&copy, strings()), strings().shell_nams_dll_copy_failed());
        let detail = failure_detail(&copy, strings());
        assert!(detail.contains("G:/game/dinput8.dll"));
        assert!(detail.contains("Access is denied"));
    }

    #[test]
    fn local_failures_explain_themselves_without_raw_output() {
        let failure = LaunchFailure::new(
            FailureKind::ExeNotFound {
                game_dir: PathBuf::from("G:/game"),
                exe: PathBuf::from("G:/game/NieRAutomata.exe"),
            },
            String::new(),
        );
        assert_eq!(
            failure_title(&failure, strings()).as_ref(),
            strings().fail_title_unknown()
        );
        assert_eq!(
            failure_explanation(&failure, strings()),
            strings().error_exe_not_found("G:/game")
        );
        assert!(failure_detail(&failure, strings()).contains("G:/game/NieRAutomata.exe"));
        assert!(failure_hints(&failure, strings()).is_empty());
        assert_eq!(failure_subtitle(&failure), None);
    }

    #[test]
    fn selection_checks_map_to_session_errors() {
        assert_eq!(selection_error(&SelectionCheck::Accepted), None);
        assert_eq!(
            selection_error(&SelectionCheck::Missing),
            Some(SessionError::FileNotExist)
        );
        assert_eq!(
            selection_error(&SelectionCheck::NotNierExe),
            Some(SessionError::InvalidExe)
        );
        let rejection = SelectionRejection::OutsidePrefix {
            exe: PathBuf::from("/tmp/NieRAutomata.exe"),
        };
        let error = selection_error(&SelectionCheck::Rejected(rejection)).unwrap();
        let text = session_error_text(&error, strings());
        assert!(text.contains("/tmp/NieRAutomata.exe"));
        assert!(text.contains(GAME_EXE_NAME));
    }

    #[test]
    fn game_errors_resolve_to_text() {
        assert_eq!(
            game_error_text(&GameError::StopFailed, strings()).as_ref(),
            strings().error_stop_failed()
        );
        let failure = reported(21, false, None);
        assert_eq!(
            game_error_text(&GameError::LaunchFailed(failure), strings()).as_ref(),
            strings().fail_title_steam_not_owned()
        );
    }

    #[test]
    fn verify_timeout_text_follows_the_platform_hint() {
        let outcome = VerifyOutcome::status(VerifyStatus::TimedOut);
        assert_eq!(
            verify_summary(&outcome, VerifyTimeoutHint::Linux, strings()),
            strings().verify_timed_out_linux()
        );
        assert_eq!(
            verify_summary(&outcome, VerifyTimeoutHint::Generic, strings()),
            strings().verify_timed_out()
        );
        assert_eq!(
            verify_check_label("nier_exe", strings()).as_ref(),
            strings().verify_check_nier_exe()
        );
        assert_eq!(verify_check_label("custom", strings()).as_ref(), "custom");
    }
}
