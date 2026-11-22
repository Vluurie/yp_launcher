use serde_json::json;
use yp_host::nams_cli::verify::outcome_from_output;
use yp_host::nams_cli::{VerifyCheck, VerifyResult, VerifyStatus};

#[test]
fn parses_the_real_all_pass_payload() {
    let payload: serde_json::Value = serde_json::from_str(
        r#"{ "ok": true, "checks": [
          { "name": "steam_install", "status": "ok", "detail": "F:/SteamLibrary/steamapps/common/NieRAutomata" },
          { "name": "nier_exe", "status": "ok", "detail": null },
          { "name": "steam_api64_source", "status": "ok", "detail": null },
          { "name": "runtime_writable", "status": "ok", "detail": "C:/x/runtime" },
          { "name": "runtime_library_cached", "status": "ok", "detail": null }
        ] }"#,
    )
    .unwrap();
    let result = VerifyResult::from_json(&payload);
    assert!(result.ok);
    assert_eq!(result.checks.len(), 5);
    assert_eq!(result.checks[0].name, "steam_install");
    assert!(result.checks[0].ok());
    assert!(result.checks[0].detail.as_deref().unwrap().contains("NieRAutomata"));
    assert_eq!(result.checks[1].detail, None);
}

#[test]
fn parses_a_failing_payload_with_a_detail_message() {
    let payload = json!({ "ok": false, "checks": [
        { "name": "steam_install", "status": "ok", "detail": null },
        { "name": "nier_exe", "status": "fail", "detail": "unsupported NieRAutomata.exe build (Windows 7)" }
    ]});
    let result = VerifyResult::from_json(&payload);
    assert!(!result.ok);
    let nier_exe = result.check("nier_exe").unwrap();
    assert!(!nier_exe.ok());
    assert!(nier_exe.detail.as_deref().unwrap().contains("Windows 7"));
}

#[test]
fn missing_checks_yield_an_empty_list() {
    let result = VerifyResult::from_json(&json!({ "ok": false }));
    assert!(result.checks.is_empty());
    assert!(!result.ok);
}

#[test]
fn unknown_check_names_are_preserved() {
    let result = VerifyResult::from_json(&json!({ "ok": true, "checks": [ { "name": "future_check", "status": "ok" } ] }));
    assert_eq!(result.checks[0].name, "future_check");
}

#[test]
fn a_check_missing_status_defaults_to_not_ok() {
    assert!(!VerifyCheck::from_json(&json!({ "name": "x" })).ok());
}

#[test]
fn exit_codes_and_output_map_onto_statuses() {
    assert_eq!(outcome_from_output(Some(21), "", "").status, VerifyStatus::SteamNotRunning);
    assert_eq!(outcome_from_output(Some(21), "", "").exit_code, Some(21));
    assert_eq!(outcome_from_output(Some(64), "", "").status, VerifyStatus::Error);
    let garbage = outcome_from_output(Some(0), "not json", "oops");
    assert_eq!(garbage.status, VerifyStatus::CannotParse);
    assert_eq!(garbage.raw.as_deref(), Some("not jsonoops"));
    assert_eq!(outcome_from_output(Some(0), "{\"ok\":true}", "").status, VerifyStatus::CannotParse);
    let ok = outcome_from_output(Some(0), "{\"ok\":true,\"checks\":[]}", "");
    assert_eq!(ok.status, VerifyStatus::Ok);
    let failed = outcome_from_output(Some(1), "{\"ok\":false,\"checks\":[{\"name\":\"a\",\"status\":\"fail\"}]}", "");
    assert_eq!(failed.status, VerifyStatus::Failed);
    assert_eq!(failed.result.unwrap().checks.len(), 1);
}
