use serde_json::json;
use yp_config::cli;
use yp_config::testing::FakeNamsCli;
use yp_config::{Kind, ResetTarget};

#[test]
fn the_sample_schema_parses_into_typed_settings() {
    let fake = FakeNamsCli::default();
    let schema = cli::schema(&fake, Some("de")).unwrap();
    assert_eq!(schema.nams_version, "4.0.0");
    let blur = schema.setting("lodmod.shadow_blur_scale").unwrap();
    assert_eq!(blur.kind, Kind::List);
    assert_eq!(blur.length, Some(4));
    assert_eq!(schema.setting("nams.graphics_backend").unwrap().choices.len(), 2);
    assert!(schema.setting("naiom.bindings.standard_jump").unwrap().is_key_binding());
    assert_eq!(fake.calls()[0], vec!["schema", "--lang", "de"]);
}

#[test]
fn set_writes_through_stdin_and_get_reads_it_back() {
    let fake = FakeNamsCli::default();
    let report = cli::set(&fake, &[("lodmod.enabled".to_owned(), json!(true))], false).unwrap();
    assert_eq!(report.changed.len(), 1);
    let values = cli::get(&fake).unwrap();
    assert_eq!(values.get("lodmod.enabled"), Some(&json!(true)));
    assert!(!values.not_in_file.contains("lodmod.enabled"));
    assert!(values.not_in_file.contains("nams.graphics_backend"));
}

#[test]
fn a_rejected_value_surfaces_as_a_nams_error_with_its_id() {
    let fake = FakeNamsCli::default();
    fake.reject("lodmod.shadow_resolution");
    let error = cli::set(&fake, &[("lodmod.shadow_resolution".to_owned(), json!(3))], false).unwrap_err();
    assert_eq!(error.rejected_ids(), vec!["lodmod.shadow_resolution".to_owned()]);
    let (report, problems) = cli::set_accepted(
        &fake,
        &[("lodmod.shadow_resolution".to_owned(), json!(3)), ("lodmod.enabled".to_owned(), json!(true))],
    )
    .unwrap();
    assert_eq!(report.changed.len(), 1);
    assert_eq!(problems.len(), 1);
}

#[test]
fn reset_and_empty_sets_build_the_right_arguments() {
    let fake = FakeNamsCli::default();
    cli::set(&fake, &[], false).unwrap();
    assert_eq!(fake.count("set"), 0);
    let ids = ResetTarget::Ids(Vec::new());
    cli::reset(&fake, &ids, false).unwrap();
    assert_eq!(fake.count("reset"), 0);
}
