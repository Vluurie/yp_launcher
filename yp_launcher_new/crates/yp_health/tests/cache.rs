mod support;

use support::{FakeGame, Rig, write};
use yp_health::cache::{CacheClearResult, clear_all};
use yp_health::notifications::{DETECTED_NOTIFICATIONS_DIR, DetectionEnv, run_detections};
use yp_host::platform::nams_settings_path;

#[test]
fn clearing_removes_logs_overlay_settings_stamp_and_the_detection_memory() {
    let rig = Rig::windows();
    rig.install_runtime_files();
    let game = FakeGame::create();
    let paths = game.paths();

    rig.write_log("nams.log", "line\n");
    rig.write_log("crashdump_1.txt", "boom\n");
    rig.write_log("last_launch.json", "{}");
    write(&rig.paths.logs_dir.join("archive").join("old.log"), "kept");
    let overlay = nams_settings_path(Some(game.path())).unwrap();
    write(&overlay, "{}");
    write(&rig.paths.runtime_stamp(), "v|hash");
    let env = DetectionEnv {
        settings: &rig.settings,
        paths: &rig.paths,
        host: &rig.host,
        cli: None,
    };
    assert!(run_detections(&env, &paths).is_some());
    assert!(rig.settings.contains(&DETECTED_NOTIFICATIONS_DIR));

    let result = clear_all(&rig.paths, &rig.settings, &rig.platform(), Some(&paths));
    assert_eq!(
        result,
        CacheClearResult {
            logs_deleted: 3,
            detection_reset: true,
            nams_settings_deleted: true,
            stamp_deleted: true,
        }
    );
    assert!(!rig.paths.nams_log().exists());
    assert!(rig.paths.logs_dir.join("archive").join("old.log").is_file());
    assert!(!overlay.exists());
    assert!(!rig.paths.runtime_stamp().exists());
    assert!(!rig.settings.contains(&DETECTED_NOTIFICATIONS_DIR));
    assert!(rig.paths.nams_exe().is_file());
    assert!(paths.nams_toml().is_file());
    assert!(run_detections(&env, &paths).is_some());
}

#[test]
fn clearing_twice_finds_nothing_left() {
    let rig = Rig::windows();
    let game = FakeGame::create();
    let paths = game.paths();
    rig.write_log("nams.log", "line\n");
    write(&nams_settings_path(Some(game.path())).unwrap(), "{}");

    let first = clear_all(&rig.paths, &rig.settings, &rig.platform(), Some(&paths));
    assert_eq!(first.logs_deleted, 1);
    assert!(first.nams_settings_deleted);
    assert!(!first.stamp_deleted);

    let second = clear_all(&rig.paths, &rig.settings, &rig.platform(), Some(&paths));
    assert_eq!(
        second,
        CacheClearResult {
            logs_deleted: 0,
            detection_reset: true,
            nams_settings_deleted: false,
            stamp_deleted: false,
        }
    );
}

#[test]
fn clearing_without_a_game_folder_leaves_overlay_settings_alone() {
    let rig = Rig::windows();
    let game = FakeGame::create();
    let overlay = nams_settings_path(Some(game.path())).unwrap();
    write(&overlay, "{}");

    let none = clear_all(&rig.paths, &rig.settings, &rig.platform(), None);
    assert_eq!(none.logs_deleted, 0);
    assert!(!none.nams_settings_deleted);
    let blank = yp_core::GamePaths::new("");
    let empty = clear_all(&rig.paths, &rig.settings, &rig.platform(), Some(&blank));
    assert!(!empty.nams_settings_deleted);
    assert!(overlay.is_file());
}

#[test]
fn missing_logs_folder_counts_as_nothing_deleted() {
    let rig = Rig::windows();
    assert!(!rig.paths.logs_dir.exists());
    let result = clear_all(&rig.paths, &rig.settings, &rig.platform(), None);
    assert_eq!(result.logs_deleted, 0);
    assert!(result.detection_reset);
}
