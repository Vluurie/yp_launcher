use std::time::Duration;

use serde_json::json;
use yp_core::AppPaths;
use yp_host::launch::record::{format_timestamp, read, read_file, tail_of, write, LaunchRecord, TAIL_CHARS};

#[test]
fn round_trips_through_the_launch_record_file() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_test(dir.path());
    assert!(read(&paths).is_none());
    let started = LaunchRecord::started("Proton");
    write(&paths, &started).unwrap();
    let stored = read(&paths).unwrap();
    assert!(stored.still_running());
    assert_eq!(stored.label, "Proton");
    assert_eq!(stored.started_at.as_millisecond(), started.started_at.as_millisecond());
    let finished = stored.finished(1, "tail");
    write(&paths, &finished).unwrap();
    let stored = read(&paths).unwrap();
    assert_eq!(stored.exit_code, Some(1));
    assert_eq!(stored.output_tail, "tail");
    assert!(stored.duration().unwrap() < Duration::from_secs(60));
}

#[test]
fn json_shape_matches_the_flutter_launcher() {
    let record = LaunchRecord::from_json(&json!({
        "started_at": "2026-11-09T10:00:00.000Z",
        "ended_at": "2026-11-09T10:01:30.500Z",
        "exit_code": 20,
        "label": "Windows",
        "output_tail": "ERROR[20] steam"
    }))
    .unwrap();
    assert_eq!(record.duration(), Some(Duration::from_millis(90_500)));
    let json = record.to_json();
    assert_eq!(json["started_at"], "2026-11-09T10:00:00.000Z");
    assert_eq!(json["ended_at"], "2026-11-09T10:01:30.500Z");
    assert_eq!(json["exit_code"], 20);
    let keys: Vec<&String> = json.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["started_at", "ended_at", "exit_code", "label", "output_tail"]);
    let running = LaunchRecord::from_json(&json!({ "started_at": "2026-11-09T10:00:00.123456Z" })).unwrap();
    assert!(running.still_running());
    assert_eq!(running.label, "");
    assert!(!running.to_json().as_object().unwrap().contains_key("ended_at"));
    assert_eq!(format_timestamp(running.started_at), "2026-11-09T10:00:00.123Z");
}

#[test]
fn malformed_records_are_ignored() {
    assert!(LaunchRecord::from_json(&json!([])).is_none());
    assert!(LaunchRecord::from_json(&json!({ "started_at": "yesterday" })).is_none());
    let lenient = LaunchRecord::from_json(&json!({ "started_at": "2026-11-09T10:00:00Z", "exit_code": "x", "label": 5 })).unwrap();
    assert_eq!(lenient.exit_code, None);
    assert_eq!(lenient.label, "");
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("last_launch.json");
    std::fs::write(&file, "{not json").unwrap();
    assert!(read_file(&file).is_none());
}

#[test]
fn tails_keep_the_last_characters() {
    let long = "x".repeat(TAIL_CHARS + 10) + "end";
    let tail = tail_of(&long);
    assert_eq!(tail.chars().count(), TAIL_CHARS);
    assert!(tail.ends_with("end"));
    assert_eq!(tail_of("short"), "short");
}
