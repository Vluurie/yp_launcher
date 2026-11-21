mod support;

use std::borrow::Cow;

use serde_json::json;
use support::{FakeGame, FakePlatform, Rig, fake_cli, write};
use yp_config::testing::FakeNamsCli;
use yp_config::legacy::SkippedNaiomSetting;
use yp_core::toml_io::parse_values;
use yp_core::{GamePaths, Os};
use yp_health::notifications::{
    DETECTED_NOTIFICATIONS_DIR, DetectionEnv, NotificationCategory, NotificationEntry,
    NotificationKind, NotificationList, PLATFORM_UNSUPPORTED_ID, already_detected, detect,
    platform_support_notice, refresh_detections, run_detections,
};
use yp_thirdparty::LoadingFlag;

fn env<'a>(rig: &'a Rig, cli: &'a FakeNamsCli) -> DetectionEnv<'a> {
    DetectionEnv {
        settings: &rig.settings,
        paths: &rig.paths,
        host: &rig.host,
        cli: Some(cli),
    }
}

fn configured_game() -> (FakeGame, GamePaths) {
    let game = FakeGame::create();
    let paths = game.paths();
    std::fs::remove_dir_all(paths.nams_dir()).unwrap();
    let flags: String = LoadingFlag::ALL
        .iter()
        .map(|flag| format!("{} = false\n", flag.key()))
        .collect();
    write(&paths.nams_toml(), flags);
    (game, paths)
}

fn clean_game(rig: &Rig) -> (FakeGame, GamePaths) {
    rig.install_runtime_files();
    configured_game()
}

fn nams_values(game: &GamePaths) -> toml::Table {
    parse_values(&std::fs::read_to_string(game.nams_toml()).unwrap()).unwrap()
}

#[test]
fn clean_install_raises_nothing_and_is_only_detected_once_per_folder() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (_game, paths) = clean_game(&rig);

    assert!(!already_detected(&rig.settings, &paths));
    assert_eq!(run_detections(&env(&rig, &cli), &paths), Some(Vec::new()));
    assert!(already_detected(&rig.settings, &paths));
    assert_eq!(
        rig.settings.get(&DETECTED_NOTIFICATIONS_DIR),
        Some(paths.root().to_string_lossy().into_owned())
    );
    assert_eq!(run_detections(&env(&rig, &cli), &paths), None);

    let (_other, other_paths) = configured_game();
    assert_eq!(run_detections(&env(&rig, &cli), &other_paths), Some(Vec::new()));
    assert_eq!(run_detections(&env(&rig, &cli), &paths), Some(Vec::new()));
}

#[test]
fn refresh_runs_again_for_the_remembered_folder() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = clean_game(&rig);
    assert_eq!(run_detections(&env(&rig, &cli), &paths), Some(Vec::new()));

    write(
        &game.path().join("SK_Res").join("inject").join("x.dds"),
        [0u8],
    );
    assert_eq!(run_detections(&env(&rig, &cli), &paths), None);
    assert_eq!(
        refresh_detections(&env(&rig, &cli), &paths),
        vec![NotificationKind::TexturesDetected {
            folder: "SK_Res/".to_owned()
        }]
    );
    assert!(already_detected(&rig.settings, &paths));
}

#[test]
fn missing_launcher_files_raise_the_quarantine_notice() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let game = FakeGame::create();
    assert_eq!(
        detect(&env(&rig, &cli), &game.paths()),
        vec![NotificationKind::FilesQuarantined]
    );

    rig.install_runtime_files();
    std::fs::remove_file(rig.paths.yorha_dll()).unwrap();
    assert_eq!(
        detect(&env(&rig, &cli), &game.paths()),
        vec![NotificationKind::FilesQuarantined]
    );
}

#[test]
fn legacy_lodmod_ini_is_imported_once() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = clean_game(&rig);
    write(
        &game.path().join("LodMod.ini"),
        "[LodMod]\nShadowResolution = 4096\nLODMultiplier = 0\n",
    );

    assert_eq!(
        detect(&env(&rig, &cli), &paths),
        vec![NotificationKind::LodModMigrated]
    );
    assert_eq!(cli.value("lodmod.enabled"), Some(json!(true)));
    assert_eq!(cli.value("lodmod.shadow_resolution"), Some(json!(4096)));
    assert!(detect(&env(&rig, &cli), &paths).is_empty());
}

#[test]
fn legacy_naiom_ini_is_imported_and_unsupported_keys_are_listed() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = clean_game(&rig);
    write(
        &game.path().join("NAIOM.ini"),
        "MiscOpenDebugMenu PAUSE\nAimMode Disabled\n",
    );

    let found = detect(&env(&rig, &cli), &paths);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0], NotificationKind::NaiomMigrated);
    let NotificationKind::NaiomSkipped { entries } = &found[1] else {
        panic!("expected skipped NAIOM entries, got {:?}", found[1]);
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].setting, SkippedNaiomSetting::DebugMenuKey);
    assert_eq!(entries[0].to_string(), "MiscOpenDebugMenu (PAUSE)");
    assert_eq!(cli.value("naiom.aim_crosshair"), Some(json!(false)));
    assert_eq!(cli.value("naiom.misc_open_debug_menu"), None);
}

#[test]
fn naiom_import_without_skipped_keys_raises_one_notice() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = clean_game(&rig);
    write(
        &game.path().join("NAIOM.ini"),
        "ThirdPersonSensitivityX 1.500000\n",
    );

    assert_eq!(
        detect(&env(&rig, &cli), &paths),
        vec![NotificationKind::NaiomMigrated]
    );
}

#[test]
fn reshade_in_the_game_folder_is_switched_off_and_announced() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = clean_game(&rig);
    write(&game.path().join("dxgi.dll"), b"MZ.. reshade.me ..");

    assert_eq!(
        detect(&env(&rig, &cli), &paths),
        vec![NotificationKind::ReShadeDetected]
    );
    assert_eq!(
        nams_values(&paths)
            .get(LoadingFlag::ReShade.key())
            .and_then(toml::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        detect(&env(&rig, &cli), &paths),
        vec![NotificationKind::ReShadeDetected]
    );
}

#[test]
fn plain_proxy_dll_is_not_reported_as_reshade() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = clean_game(&rig);
    write(&game.path().join("dxgi.dll"), b"MZ plain proxy");

    assert!(detect(&env(&rig, &cli), &paths).is_empty());
    assert_eq!(
        nams_values(&paths)
            .get(LoadingFlag::ReShade.key())
            .and_then(toml::Value::as_bool),
        Some(false)
    );
}

#[test]
fn special_k_texture_folder_is_announced_only_when_it_has_content() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = clean_game(&rig);
    std::fs::create_dir_all(game.path().join("SK_Res")).unwrap();
    assert!(detect(&env(&rig, &cli), &paths).is_empty());

    write(
        &game.path().join("SK_Res").join("inject").join("x.dds"),
        [0u8],
    );
    let found = detect(&env(&rig, &cli), &paths);
    assert_eq!(
        found,
        vec![NotificationKind::TexturesDetected {
            folder: "SK_Res/".to_owned()
        }]
    );
    assert_eq!(found[0].id(), "textures_SK_Res/");
}

#[test]
fn every_detection_shows_up_in_startup_order() {
    let rig = Rig::windows();
    let cli = fake_cli();
    let (game, paths) = configured_game();
    write(
        &game.path().join("LodMod.ini"),
        "[LodMod]\nShadowResolution = 4096\n",
    );
    write(
        &game.path().join("NAIOM.ini"),
        "MiscOpenDebugMenu PAUSE\nAimMode Disabled\n",
    );
    write(&game.path().join("ReShade64.dll"), b"MZ crosire");
    write(&game.path().join("SK_Res").join("x.dds"), [0u8]);

    let ids: Vec<String> = run_detections(&env(&rig, &cli), &paths)
        .unwrap()
        .iter()
        .map(|kind| kind.id().into_owned())
        .collect();
    assert_eq!(
        ids,
        [
            "files_quarantined",
            "lodmod_migration",
            "naiom_migration",
            "naiom_migration_skipped",
            "reshade",
            "textures_SK_Res/",
        ]
    );
}

#[test]
fn kinds_keep_the_flutter_ids_and_categories() {
    let cases = [
        (
            NotificationKind::FilesQuarantined,
            "files_quarantined",
            NotificationCategory::General,
        ),
        (
            NotificationKind::LodModMigrated,
            "lodmod_migration",
            NotificationCategory::Migration,
        ),
        (
            NotificationKind::NaiomMigrated,
            "naiom_migration",
            NotificationCategory::Migration,
        ),
        (
            NotificationKind::NaiomSkipped {
                entries: Vec::new(),
            },
            "naiom_migration_skipped",
            NotificationCategory::Migration,
        ),
        (
            NotificationKind::ReShadeDetected,
            "reshade",
            NotificationCategory::ReShade,
        ),
        (
            NotificationKind::ReShadeIncompatible,
            "reshade_incompatible",
            NotificationCategory::ReShade,
        ),
        (
            NotificationKind::TexturesDetected {
                folder: "wax/".to_owned(),
            },
            "textures_wax/",
            NotificationCategory::Textures,
        ),
        (
            NotificationKind::PlatformUnsupported { os: Os::Linux },
            "platform_unsupported",
            NotificationCategory::General,
        ),
    ];
    for (kind, id, category) in cases {
        assert_eq!(kind.id(), id);
        assert_eq!(kind.notification_id(), id);
        assert_eq!(kind.category(), category);
    }
}

#[test]
fn platform_notice_appears_once_while_the_game_cannot_be_launched() {
    let rig = Rig::new(Os::Linux);
    let able = rig.platform();
    let unable = FakePlatform::unable_to_launch(rig.host.clone());
    assert_eq!(platform_support_notice(&able), None);
    assert_eq!(
        platform_support_notice(&unable),
        Some(NotificationKind::PlatformUnsupported { os: Os::Linux })
    );

    let mut list: NotificationList = NotificationList::new();
    assert!(!list.check_platform_support(&able));
    assert!(list.is_empty());
    assert!(list.check_platform_support(&unable));
    assert!(!list.check_platform_support(&unable));
    assert_eq!(list.len(), 1);
    assert!(list.contains(PLATFORM_UNSUPPORTED_ID));

    assert!(list.dismiss(PLATFORM_UNSUPPORTED_ID));
    assert!(!list.dismiss(PLATFORM_UNSUPPORTED_ID));
    assert!(list.is_empty());
    assert!(list.check_platform_support(&unable));
}

#[test]
fn detections_replace_the_list_and_a_skipped_run_keeps_it() {
    let mut list: NotificationList = NotificationList::new();
    list.add(NotificationKind::ReShadeDetected);
    list.add(NotificationKind::PlatformUnsupported { os: Os::MacOs });

    assert!(!list.apply_detections(None));
    assert_eq!(list.len(), 2);

    assert!(list.apply_detections(Some(vec![NotificationKind::LodModMigrated])));
    assert_eq!(list.items(), [NotificationKind::LodModMigrated]);

    list.add(NotificationKind::NaiomMigrated);
    assert!(list.dismiss("lodmod_migration"));
    assert_eq!(list.items(), [NotificationKind::NaiomMigrated]);
    assert!(list.apply_detections(Some(Vec::new())));
    assert!(list.is_empty());
}

#[derive(Debug, Clone, PartialEq)]
enum AppNotice {
    Health(NotificationKind),
    Toast(&'static str),
}

impl From<NotificationKind> for AppNotice {
    fn from(kind: NotificationKind) -> Self {
        AppNotice::Health(kind)
    }
}

impl NotificationEntry for AppNotice {
    fn notification_id(&self) -> Cow<'_, str> {
        match self {
            AppNotice::Health(kind) => kind.id(),
            AppNotice::Toast(id) => Cow::Borrowed(id),
        }
    }
}

#[test]
fn list_carries_app_defined_notices_next_to_detected_ones() {
    let rig = Rig::new(Os::MacOs);
    let unable = FakePlatform::unable_to_launch(rig.host.clone());
    let mut list: NotificationList<AppNotice> = NotificationList::new();
    list.add(AppNotice::Toast("cache_cleared"));
    assert!(list.check_platform_support(&unable));
    assert_eq!(
        list.items(),
        [
            AppNotice::Toast("cache_cleared"),
            AppNotice::Health(NotificationKind::PlatformUnsupported { os: Os::MacOs }),
        ]
    );
    assert!(list.dismiss("cache_cleared"));
    assert!(list.apply_detections(Some(vec![NotificationKind::ReShadeIncompatible])));
    assert_eq!(
        list.items(),
        [AppNotice::Health(NotificationKind::ReShadeIncompatible)]
    );
}
