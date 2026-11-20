use std::path::PathBuf;

use yp_core::{LaunchUnavailable, Os};
use yp_host::launch::failure::{
    parse, parse_nams_output, FailureCategory, FailureHint, FailureKind, HintLine, LaunchFailure,
};

#[test]
fn empty_or_unstructured_output_is_not_a_report() {
    assert_eq!(parse_nams_output(""), None);
    assert_eq!(parse_nams_output("   \n"), None);
    assert_eq!(parse_nams_output("just some log text"), None);
    assert_eq!(parse_nams_output("  ERROR[1] indented headers do not count"), None);
    assert_eq!(parse_nams_output("ERROR[x] not a number"), None);
    assert_eq!(parse_nams_output("ERROR[1]glued"), None);
    assert_eq!(parse_nams_output("ERROR[1]   "), None);
}

#[test]
fn reads_the_header_and_the_labelled_lines_with_crlf() {
    let report = parse_nams_output(
        "noise\r\nERROR[22]   Steam ownership check failed  \r\n    Phase:   steam\r\n OS error: none\r\nLikely cause: offline\r\nTry: restart Steam\r\n",
    )
    .unwrap();
    assert_eq!(report.code, Some(22));
    assert!(!report.panic);
    assert_eq!(report.headline, "Steam ownership check failed");
    assert_eq!(report.phase.as_deref(), Some("steam"));
    assert_eq!(report.os_error.as_deref(), Some("none"));
    assert_eq!(report.cause.as_deref(), Some("offline"));
    assert_eq!(report.fix.as_deref(), Some("restart Steam"));
}

#[test]
fn labelled_lines_are_optional() {
    let report = parse_nams_output("PANIC[7] boom").unwrap();
    assert!(report.panic);
    assert_eq!((report.phase, report.os_error, report.cause, report.fix), (None, None, None, None));
}

#[test]
fn parse_wraps_the_report_with_the_raw_output_and_log() {
    let failure = parse("ERROR[64] bad args", Some(PathBuf::from("log.txt"))).unwrap();
    assert_eq!(failure.raw_output, "ERROR[64] bad args");
    assert_eq!(failure.captured_log, Some(PathBuf::from("log.txt")));
    assert_eq!(failure.category(), FailureCategory::InvalidArgs);
    assert_eq!(failure.hints(), vec![HintLine::Known(FailureHint::PanicShare)]);
}

#[test]
fn codes_map_onto_categories() {
    let expected = [
        (1, FailureCategory::NamsFailure),
        (2, FailureCategory::InstallNotFound),
        (3, FailureCategory::FolderCreate),
        (4, FailureCategory::RuntimePrep),
        (5, FailureCategory::HostFailure),
        (9, FailureCategory::HostFailure),
        (20, FailureCategory::SteamNotRunning),
        (21, FailureCategory::SteamNotOwned),
        (22, FailureCategory::SteamCheckFailed),
        (64, FailureCategory::InvalidArgs),
        (10, FailureCategory::ExitedUnexpectedly),
        (-1, FailureCategory::ExitedUnexpectedly),
    ];
    for (code, category) in expected {
        assert_eq!(FailureCategory::for_code(code), category, "{code}");
    }
    assert_eq!(
        FailureCategory::HostFailure.hints(),
        [FailureHint::Reboot, FailureHint::AntivirusExclusions, FailureHint::PersistShare]
    );
    assert_eq!(
        FailureCategory::ExitedUnexpectedly.hints(),
        [FailureHint::CheckLogViewer, FailureHint::ShareReport]
    );
}

#[test]
fn launcher_side_failures_have_no_code_and_no_hints() {
    let failure = LaunchFailure::new(FailureKind::Unavailable(LaunchUnavailable::NoCompatLayer { os: Os::Linux }), "");
    assert_eq!(failure.code(), None);
    assert!(!failure.reported_by_nams());
    assert_eq!(failure.category(), FailureCategory::Unknown);
    assert!(failure.hints().is_empty());
}

#[test]
fn an_empty_fix_is_not_a_hint() {
    let failure = parse("ERROR[20] steam\nTry:   \n", None).unwrap();
    assert_eq!(failure.hints(), vec![HintLine::Known(FailureHint::StartSteam)]);
}
