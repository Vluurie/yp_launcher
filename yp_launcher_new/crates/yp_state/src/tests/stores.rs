use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::Ordering;

use futures::channel::oneshot;
use gpui::{AppContext as _, TestAppContext};
use yp_formats::wmb3::{Mesh, Model};
use yp_i18n::{Locale, Msg};
use yp_inspector::{ArchivePaths, InspectorError, Session};
use yp_thirdparty::{ThirdPartyKind, ThirdPartyRuntime};
use yp_ui::{I18n, ThemeId};

use super::{Harness, settle};
use crate::inspector::{ModelEvent, ModelSession};
use crate::logs::{LOG_POLL_INTERVAL, LogLevel, LogTab};
use crate::prefs::{
    APP_LOCALE, APP_THEME, DETECTED_NOTIFICATIONS_DIR, MINIMIZE_ON_LAUNCH, ONBOARDING_COMPLETE,
    PREFER_DEDICATED_GPU, SIDEBAR_SECTION_COLLAPSED,
};
use crate::registry::{ModInstallReport, ModInstallRequest, OverlayRequest};
use crate::{
    BusyEvent, BusyScope, DetectionEvent, Notice, NoticeKind, NoticeSeverity, OverlayId, Registry,
    StateExt, Stores, TabId,
};

fn notice_ids(stores: &Stores, cx: &mut TestAppContext) -> Vec<String> {
    stores.notifications.read_with(cx, |notifications, _| {
        notifications
            .items()
            .iter()
            .map(|notice| notice.id.to_string())
            .collect()
    })
}

#[gpui::test]
fn busy_tracking_counts_running_work(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let changes = Rc::new(RefCell::new(Vec::new()));
    let seen = changes.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.busy, move |_, event, _| {
            let BusyEvent::Changed { count, textures } = event;
            seen.borrow_mut().push((*count, *textures));
        })
    });

    let (finish, finished) = oneshot::channel::<u32>();
    let tracked = stores.busy.update(cx, |busy, cx| {
        busy.track(
            BusyScope::Textures,
            async move { finished.await.unwrap_or_default() },
            cx,
        )
    });
    let other = stores.busy.update(cx, |busy, cx| {
        busy.track(BusyScope::General, std::future::pending::<()>(), cx)
    });
    cx.run_until_parked();
    stores.busy.read_with(cx, |busy, _| {
        assert_eq!(busy.count(), 2);
        assert!(busy.is_busy());
        assert!(busy.textures());
    });

    finish.send(7).unwrap();
    cx.run_until_parked();
    assert_eq!(cx.foreground_executor().block_test(tracked), 7);
    cx.run_until_parked();
    stores.busy.read_with(cx, |busy, _| {
        assert_eq!(busy.count(), 1);
        assert!(!busy.textures());
    });

    drop(other);
    cx.run_until_parked();
    stores.busy.read_with(cx, |busy, _| {
        assert_eq!(busy.count(), 0);
        assert!(!busy.is_busy());
    });
    assert_eq!(
        *changes.borrow(),
        vec![(1, true), (2, true), (1, false), (0, false)]
    );
}

#[gpui::test]
fn detections_become_notifications_once_per_directory(cx: &mut TestAppContext) {
    let harness = Harness::new();
    std::fs::create_dir_all(harness.game.root().join("SK_Res").join("inject")).unwrap();
    std::fs::remove_file(harness.env.paths.yorha_dll()).unwrap();
    let stores = harness.start(cx);

    assert_eq!(
        notice_ids(&stores, cx),
        vec![
            "files_quarantined".to_owned(),
            "textures_SK_Res/".to_owned()
        ]
    );
    stores.notifications.read_with(cx, |notifications, _| {
        let quarantined = &notifications.items()[0];
        assert_eq!(quarantined.msg, Msg::NotifyFilesQuarantined);
        assert_eq!(quarantined.kind, NoticeKind::General);
        assert_eq!(quarantined.severity, NoticeSeverity::Error);
        assert_eq!(
            notifications.items()[1].msg,
            Msg::NotifyTexturesDetected {
                folder: "SK_Res/".to_owned()
            }
        );
        assert!(!notifications.is_detecting());
    });
    assert_eq!(
        harness.env.settings.get(&DETECTED_NOTIFICATIONS_DIR),
        Some(harness.game.root().to_string_lossy().into_owned())
    );

    stores.notifications.update(cx, |notifications, cx| {
        notifications.dismiss("files_quarantined", cx);
        notifications.dismiss("textures_SK_Res/", cx);
        notifications.run_detections(&harness.game, cx);
    });
    cx.run_until_parked();
    assert!(notice_ids(&stores, cx).is_empty());

    std::fs::write(harness.env.paths.yorha_dll(), b"yorha").unwrap();
    stores.notifications.update(cx, |notifications, cx| {
        notifications.refresh_detections(&harness.game, cx);
    });
    cx.run_until_parked();
    assert_eq!(notice_ids(&stores, cx), vec!["textures_SK_Res/".to_owned()]);
}

#[gpui::test]
fn pushed_notices_replace_their_id_and_survive_detections(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let custom =
        |msg: Msg| Notice::new("shortcut", msg, NoticeKind::Shortcut, NoticeSeverity::Info);

    stores.notifications.update(cx, |notifications, cx| {
        notifications.push(custom(Msg::NotifyFilesQuarantined), cx);
        notifications.push(custom(Msg::NotifyLodModMigrated), cx);
    });
    stores.notifications.read_with(cx, |notifications, _| {
        assert_eq!(notifications.items().len(), 1);
        assert_eq!(notifications.items()[0].msg, Msg::NotifyLodModMigrated);
        assert!(notifications.contains("shortcut"));
    });

    harness.platform.can_launch.store(false, Ordering::SeqCst);
    stores.notifications.update(cx, |notifications, cx| {
        notifications.check_platform_support(cx);
        notifications.check_platform_support(cx);
        notifications.refresh_detections(&harness.game, cx);
    });
    cx.run_until_parked();

    assert_eq!(
        notice_ids(&stores, cx),
        vec!["shortcut".to_owned(), "platform_unsupported".to_owned()]
    );
    stores.notifications.read_with(cx, |notifications, _| {
        assert_eq!(
            notifications.items()[1].msg,
            Msg::NotifyPlatformUnsupported {
                platform: "Windows".to_owned()
            }
        );
        assert_eq!(notifications.items()[1].severity, NoticeSeverity::Warning);
    });
}

#[gpui::test]
fn legacy_third_party_files_move_and_notify(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let legacy = harness.env.paths.launcher_thirdparty_dir().join("reshade");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("ReShade.ini"), "[GENERAL]\n").unwrap();
    let stores = harness.start(cx);

    assert!(notice_ids(&stores, cx).contains(&"thirdparty_migrated".to_owned()));
    stores.notifications.read_with(cx, |notifications, _| {
        let notice = notifications
            .items()
            .iter()
            .find(|notice| notice.id.as_ref() == "thirdparty_migrated")
            .unwrap();
        assert!(matches!(notice.msg, Msg::NotifyThirdPartyMigrated { .. }));
        assert_eq!(notice.severity, NoticeSeverity::Success);
    });
    assert!(!legacy.join("ReShade.ini").exists());

    stores
        .thirdparty
        .update(cx, |thirdparty, cx| thirdparty.refresh(cx));
    cx.run_until_parked();
    stores.thirdparty.read_with(cx, |thirdparty, _| {
        assert!(thirdparty.reshade().installed);
        assert!(!thirdparty.is_loading());
        assert!(!thirdparty.is_busy());
        assert!(thirdparty.error().is_none());
    });
}

#[gpui::test]
fn third_party_switches_update_the_statuses(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    stores
        .thirdparty
        .update(cx, |thirdparty, cx| thirdparty.refresh(cx));
    cx.run_until_parked();
    let enabled = stores
        .thirdparty
        .read_with(cx, |thirdparty, _| thirdparty.migoto().enabled);

    stores.thirdparty.update(cx, |thirdparty, cx| {
        thirdparty.set_enabled(ThirdPartyRuntime::Migoto, !enabled, cx);
    });
    cx.run_until_parked();
    stores.thirdparty.read_with(cx, |thirdparty, _| {
        assert_eq!(thirdparty.migoto().enabled, !enabled);
        assert_eq!(
            thirdparty.status(ThirdPartyRuntime::Migoto).enabled,
            !enabled
        );
    });

    let empty = harness.root.path().join("empty_drop");
    std::fs::create_dir_all(&empty).unwrap();
    let classified = stores
        .thirdparty
        .read_with(cx, |thirdparty, cx| thirdparty.classify(empty, cx));
    cx.run_until_parked();
    let classification = cx.foreground_executor().block_test(classified);
    assert_eq!(classification.kind, ThirdPartyKind::Unknown);

    let update = stores.thirdparty.read_with(cx, |thirdparty, cx| {
        thirdparty.would_update(classification.clone(), cx)
    });
    cx.run_until_parked();
    assert!(cx.foreground_executor().block_test(update).is_none());

    let installed = stores
        .thirdparty
        .update(cx, |thirdparty, cx| thirdparty.install(classification, cx));
    cx.run_until_parked();
    assert!(cx.foreground_executor().block_test(installed).unwrap().is_none());
    assert!(
        !stores
            .thirdparty
            .read_with(cx, |thirdparty, _| thirdparty.is_busy())
    );
}

#[gpui::test]
fn detection_snapshot_counts_what_is_installed(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    stores.detection.read_with(cx, |detection, _| {
        let snapshot = detection.snapshot().unwrap();
        assert_eq!(snapshot.mods, 0);
        assert!(!snapshot.has_textures());
        assert!(!snapshot.lodmod_enabled);
        assert!(!snapshot.dlc);
        assert!(!snapshot.reshade_detected());
        assert!(!snapshot.has_hd_cutscenes());
    });

    for name in ["kimono", "armor", ".hidden", "_parked"] {
        std::fs::create_dir_all(harness.game.mods_dir().join(name)).unwrap();
    }
    for pack in ["hd_pack", "off_pack"] {
        std::fs::create_dir_all(harness.game.inject_textures_dir().join(pack)).unwrap();
    }
    std::fs::write(
        harness.game.texture_injection_toml(),
        "disabled_packs = [\"off_pack\"]\n",
    )
    .unwrap();
    let lodmod = std::fs::read_to_string(harness.game.lodmod_toml())
        .unwrap()
        .replace("enabled = false", "enabled = true");
    std::fs::write(harness.game.lodmod_toml(), lodmod).unwrap();

    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.detection, move |_, event: &DetectionEvent, _| {
            seen.borrow_mut().push(*event);
        })
    });
    let before = stores
        .detection
        .read_with(cx, |detection, _| detection.generation());
    stores.detection.update(cx, |detection, cx| {
        detection.invalidate(cx);
        assert!(detection.is_checking());
    });
    cx.run_until_parked();

    stores.detection.read_with(cx, |detection, _| {
        assert_eq!(detection.generation(), before + 1);
        assert!(!detection.is_checking());
        let snapshot = detection.snapshot().unwrap();
        assert_eq!(snapshot.mods, 2);
        assert_eq!(snapshot.textures.active_packs, 1);
        assert!(snapshot.has_textures());
        assert!(snapshot.lodmod_enabled);
    });
    assert_eq!(
        *events.borrow(),
        vec![
            DetectionEvent::Invalidated {
                generation: before + 1
            },
            DetectionEvent::Updated {
                generation: before + 1
            },
        ]
    );
}

fn log_line(second: u32, level: &str, module: &str, message: &str) -> String {
    format!("2026-01-01T10:00:{second:02}.123456Z {level} {module}: {message}\n")
}

#[gpui::test]
fn logs_load_stream_and_filter(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let log_file = harness.env.paths.nams_log();
    let first = [
        log_line(1, "INFO", "nams::boot", "starting"),
        log_line(2, "WARN", "nams::content", "slow texture"),
        log_line(3, "ERROR", "yorha_protocol::overlay", "overlay failed"),
    ]
    .concat();
    std::fs::write(&log_file, &first).unwrap();

    stores.logs.update(cx, |logs, cx| {
        logs.load(cx);
        assert!(logs.is_loading());
        assert!(logs.is_streaming());
    });
    cx.run_until_parked();
    stores.logs.read_with(cx, |logs, _| {
        assert!(!logs.is_loading());
        assert_eq!(logs.entries(LogTab::Modloader).len(), 2);
        assert_eq!(logs.entries(LogTab::Yorha).len(), 1);
        assert_eq!(logs.active_tab(), LogTab::Modloader);
        assert_eq!(logs.counts().get(&LogLevel::Info), Some(&1));
        assert!(!logs.is_level_shown(LogLevel::Info));
        assert_eq!(logs.filtered().len(), 1);
        assert_eq!(logs.filtered()[0].message, "slow texture");
        assert_eq!(logs.log_file(), log_file.as_path());
    });

    let mut appended = std::fs::OpenOptions::new()
        .append(true)
        .open(&log_file)
        .unwrap();
    std::io::Write::write_all(
        &mut appended,
        log_line(4, "ERROR", "nams::content", "missing file").as_bytes(),
    )
    .unwrap();
    drop(appended);
    cx.run_until_parked();
    assert_eq!(
        stores
            .logs
            .read_with(cx, |logs, _| logs.entries(LogTab::Modloader).len()),
        2
    );
    settle(cx, LOG_POLL_INTERVAL);
    assert_eq!(
        stores
            .logs
            .read_with(cx, |logs, _| logs.entries(LogTab::Modloader).len()),
        3
    );

    stores.logs.update(cx, |logs, cx| {
        logs.toggle_level(LogLevel::Info, cx);
        assert_eq!(logs.filtered().len(), 3);
        logs.set_search_query("MISSING", cx);
        assert_eq!(logs.query(), "MISSING");
        assert_eq!(logs.filtered().len(), 1);
        logs.set_search_query("", cx);
        logs.set_active_tab(LogTab::Yorha, cx);
        assert_eq!(logs.filtered().len(), 1);
        assert_eq!(logs.active_entries().len(), 1);
    });

    stores.logs.update(cx, |logs, cx| logs.stop_streaming(cx));
    let mut appended = std::fs::OpenOptions::new()
        .append(true)
        .open(&log_file)
        .unwrap();
    std::io::Write::write_all(
        &mut appended,
        log_line(5, "WARN", "nams::content", "late").as_bytes(),
    )
    .unwrap();
    drop(appended);
    settle(cx, LOG_POLL_INTERVAL * 2);
    stores.logs.read_with(cx, |logs, _| {
        assert!(!logs.is_streaming());
        assert_eq!(logs.entries(LogTab::Modloader).len(), 3);
    });

    stores.logs.update(cx, |logs, cx| logs.refresh(cx));
    cx.run_until_parked();
    stores.logs.read_with(cx, |logs, _| {
        assert!(logs.is_streaming());
        assert_eq!(logs.entries(LogTab::Modloader).len(), 4);
        assert_eq!(logs.entries(LogTab::Yorha).len(), 1);
    });

    std::fs::write(&log_file, log_line(6, "WARN", "nams::boot", "fresh run")).unwrap();
    settle(cx, LOG_POLL_INTERVAL);
    stores.logs.read_with(cx, |logs, _| {
        assert_eq!(logs.entries(LogTab::Modloader).len(), 5);
    });
}

#[gpui::test]
fn log_tabs_follow_their_files_when_the_plugin_starts_and_stops(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let nams = harness.env.paths.nams_log();
    let yorha = nams.with_file_name(yp_core::consts::YORHA_LOG_NAME);
    let raw = "2026-11-25T07:39:51.321337Z ERROR [Native] Invalid outfit/mod.toml: TOML parse error\n";
    std::fs::write(&nams, raw).unwrap();
    stores.logs.update(cx, |logs, cx| logs.load(cx));
    cx.run_until_parked();
    let counts = |cx: &mut TestAppContext| stores.logs.read_with(cx, |logs, _| {
        (logs.entries(LogTab::Modloader).len(), logs.entries(LogTab::Yorha).len())
    });
    assert_eq!(counts(cx), (1, 0));
    std::fs::write(&yorha, "2026-11-25T07:39:52.123Z INFO overlay started\n").unwrap();
    settle(cx, LOG_POLL_INTERVAL);
    assert_eq!(counts(cx), (1, 1));
    let mut loader = std::fs::OpenOptions::new().append(true).open(&nams).unwrap();
    let mut plugin = std::fs::OpenOptions::new().append(true).open(&yorha).unwrap();
    std::io::Write::write_all(&mut loader, b"2026-11-25T07:39:53.001Z INFO [NAMS] Loading yorha_protocol.dll\n").unwrap();
    std::io::Write::write_all(&mut plugin, b"2026-11-25T07:39:53.002Z WARN [Native] plugin warning\n").unwrap();
    drop((loader, plugin));
    settle(cx, LOG_POLL_INTERVAL);
    assert_eq!(counts(cx), (2, 2));
    stores.logs.read_with(cx, |logs, _| {
        assert!(logs.entries(LogTab::Modloader)[0].message.starts_with("[Native] Invalid"));
        assert_eq!(logs.entries(LogTab::Yorha)[1].message, "[Native] plugin warning");
    });
    std::fs::remove_file(&nams).unwrap();
    settle(cx, LOG_POLL_INTERVAL);
    assert_eq!(counts(cx), (0, 2));
    std::fs::remove_file(&yorha).unwrap();
    settle(cx, LOG_POLL_INTERVAL);
    assert_eq!(counts(cx), (0, 0));
}

#[gpui::test]
fn prefs_read_their_own_writes_and_persist_in_order(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);

    cx.update(|cx| {
        let prefs = cx.prefs();
        assert!(!prefs.onboarding_complete());
        prefs.set_onboarding_complete(true);
        assert!(prefs.onboarding_complete());
        for width in [1, 2, 3] {
            prefs.update(&SIDEBAR_SECTION_COLLAPSED, |sections| {
                sections.insert("config".to_owned(), width % 2 == 1);
            });
        }
        assert_eq!(
            prefs.get(&SIDEBAR_SECTION_COLLAPSED).get("config"),
            Some(&true)
        );
        prefs.set(&MINIMIZE_ON_LAUNCH, &true);
        prefs.remove(&MINIMIZE_ON_LAUNCH);
        assert!(!prefs.get(&MINIMIZE_ON_LAUNCH));
    });
    assert!(!harness.env.settings.get(&ONBOARDING_COMPLETE));

    let flushed = cx.update(|cx| cx.prefs().flush());
    cx.run_until_parked();
    cx.foreground_executor().block_test(flushed).unwrap();

    assert!(harness.env.settings.get(&ONBOARDING_COMPLETE));
    assert_eq!(
        harness
            .env
            .settings
            .get(&SIDEBAR_SECTION_COLLAPSED)
            .get("config"),
        Some(&true)
    );
    assert!(!harness.env.settings.contains(&MINIMIZE_ON_LAUNCH));
}

#[gpui::test]
fn launcher_uses_new_nier_theme_without_a_saved_choice(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    assert_eq!(cx.update(|cx| yp_ui::theme::active_theme_id(cx)), ThemeId::NierNew);
    assert!(!harness.env.settings.contains(&APP_THEME));
}

#[gpui::test]
fn theme_and_locale_are_restored_and_saved(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness
        .env
        .settings
        .set(&APP_THEME, &ThemeId::Nier.code().to_owned())
        .unwrap();
    harness.init(cx);

    cx.update(|cx| {
        assert_eq!(yp_ui::theme::active_theme_id(cx), ThemeId::Nier);
        assert_eq!(I18n::locale(cx), Locale::En);
        yp_ui::theme::toggle_theme(cx);
        I18n::set(Locale::Th, cx);
    });
    cx.run_until_parked();
    let flushed = cx.update(|cx| cx.prefs().flush());
    cx.run_until_parked();
    cx.foreground_executor().block_test(flushed).unwrap();

    assert_eq!(harness.env.settings.get(&APP_THEME), ThemeId::Dark.code());
    assert_eq!(harness.env.settings.get(&APP_LOCALE), Locale::Th.code());
    assert_eq!(
        cx.update(|cx| cx.prefs().locale(&harness.env.host)),
        Locale::Th
    );
}

#[gpui::test]
fn the_system_locale_is_followed_without_being_saved(cx: &mut TestAppContext) {
    let mut harness = Harness::new();
    harness.env.host = harness.env.host.clone().with_system_language(Some("de"));
    harness.init(cx);
    cx.run_until_parked();
    let flushed = cx.update(|cx| cx.prefs().flush());
    cx.run_until_parked();
    cx.foreground_executor().block_test(flushed).unwrap();

    assert_eq!(cx.update(|cx| I18n::locale(cx)), Locale::De);
    assert!(!harness.env.settings.contains(&APP_LOCALE));
    assert!(!harness.env.settings.contains(&APP_THEME));
}

#[gpui::test]
fn launch_options_persist_only_the_saved_ones(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    stores.launch_options.read_with(cx, |options, _| {
        assert!(!options.vanilla());
        assert!(!options.minimize_on_launch());
        assert!(options.prefer_dedicated_gpu());
        assert!(!options.request().vanilla);
    });

    stores.launch_options.update(cx, |options, cx| {
        options.set_vanilla(true, cx);
        options.set_minimize_on_launch(true, cx);
        options.set_prefer_dedicated_gpu(false, cx);
        options.set_launch_wrapper("  gamemoderun  ", cx);
        assert!(options.request().vanilla);
        assert_eq!(options.launch_wrapper(), "gamemoderun");
    });
    let flushed = cx.update(|cx| cx.prefs().flush());
    cx.run_until_parked();
    cx.foreground_executor().block_test(flushed).unwrap();

    assert!(harness.env.settings.get(&MINIMIZE_ON_LAUNCH));
    assert!(!harness.env.settings.get(&PREFER_DEDICATED_GPU));
    assert_eq!(
        yp_host::launch::wrapper::read(&harness.env.settings),
        "gamemoderun"
    );
}

struct Placeholder;

impl gpui::Render for Placeholder {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        gpui::Empty
    }
}

#[gpui::test]
fn registry_hands_out_what_features_registered(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let built: Rc<RefCell<HashMap<&'static str, usize>>> = Rc::default();

    cx.update(|cx| {
        let tabs = built.clone();
        let overlays = built.clone();
        let registry = cx.registry_mut();
        registry.tabs.register(TabId::Mods, move |_, cx| {
            *tabs.borrow_mut().entry("tab").or_default() += 1;
            cx.new(|_| Placeholder).into()
        });
        registry
            .overlays
            .register(OverlayId::Docs, move |request, _, cx| {
                assert!(matches!(request, OverlayRequest::Docs { page: None }));
                *overlays.borrow_mut().entry("overlay").or_default() += 1;
                cx.new(|_| Placeholder).into()
            });
        registry.flows.set_mod_install(|request, _, _| {
            gpui::Task::ready(ModInstallReport {
                installed: request
                    .sources
                    .iter()
                    .map(|source| source.to_string_lossy().into_owned())
                    .collect(),
                ..ModInstallReport::default()
            })
        });
        assert!(cx.registry().tabs.contains(TabId::Mods));
        assert_eq!(cx.registry().tabs.registered(), vec![TabId::Mods]);
        assert!(cx.registry().overlays.contains(OverlayId::Docs));
        assert!(cx.registry().flows.texture_pack_install().is_none());
    });

    let window = cx.add_empty_window();
    let report = window.update(|window, cx| {
        assert!(Registry::build_tab(TabId::Mods, window, cx).is_some());
        assert!(Registry::build_tab(TabId::Textures, window, cx).is_none());
        assert!(
            Registry::build_overlay(&OverlayRequest::Docs { page: None }, window, cx).is_some()
        );
        assert!(
            Registry::build_overlay(
                &OverlayRequest::Inspector {
                    archives: ArchivePaths::default()
                },
                window,
                cx
            )
            .is_none()
        );
        assert!(Registry::install_texture_pack(vec!["pack.zip".into()], window, cx).is_none());
        Registry::install_mods(
            ModInstallRequest::guided(vec!["kimono.zip".into()]),
            window,
            cx,
        )
        .unwrap()
    });
    let report = cx.foreground_executor().block_test(report);

    assert_eq!(report.installed, vec!["kimono.zip".to_owned()]);
    assert!(report.any_installed());
    assert_eq!(built.borrow().get("tab"), Some(&1));
    assert_eq!(built.borrow().get("overlay"), Some(&1));
}

fn triangle(id: u32, name: &str, lod_index: u32) -> Mesh {
    Mesh {
        blend: Default::default(),
        id,
        name: name.to_owned(),
        lod_name: format!("LOD{lod_index}"),
        lod_index,
        weights: Vec::new(),
        positions: vec![[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
        uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]],
        indices: vec![0, 1, 2],
        albedo_texture_id: None,
        normal_texture_id: None,
    }
}

fn model() -> Model {
    Model {
        skeleton: Default::default(),
        meshes: vec![
            triangle(0, "Body", 0),
            triangle(1, "Hair", 0),
            triangle(2, "Body", 1),
        ],
        textures: HashMap::new(),
        center: [0.0, 0.0, 0.0],
        radius: 1.5,
        lod_names: vec!["LOD0".to_owned(), "LOD1".to_owned()],
    }
}

#[gpui::test]
fn model_sessions_render_in_the_background(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let frames = Rc::new(RefCell::new(0));
    let session = cx.new(|cx| ModelSession::from_session(Session::from_model(model(), false), cx));
    let seen = frames.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&session, move |_, event, _| {
            if matches!(event, ModelEvent::FrameReady) {
                *seen.borrow_mut() += 1;
            }
        })
    });
    session.read_with(cx, |session, _| {
        assert!(session.is_ready());
        assert!(session.is_rendering());
        assert!(session.latest_frame().is_none());
        assert_eq!(session.meshes().len(), 3);
        assert_eq!(session.lods(), ["LOD0".to_owned(), "LOD1".to_owned()]);
    });

    cx.run_until_parked();
    let first = session.read_with(cx, |session, _| {
        assert!(!session.is_rendering());
        session.latest_frame().unwrap()
    });
    assert_eq!(*frames.borrow(), 1);
    assert_eq!(first.size(0).width.0, 640);
    assert_eq!(first.size(0).height.0, 480);

    session.update(cx, |session, cx| {
        session.set_viewport_size(320, 200, cx);
        session.orbit(12.0, -4.0, cx);
        session.zoom(1.0, cx);
        session.pan(2.0, 2.0, cx);
        session.set_mesh_visibility(1, false, cx);
        session.set_lod(1, cx);
        assert!(!session.meshes()[1].visible);
        assert_eq!(session.selected_lod(), 1);
    });
    cx.run_until_parked();

    let latest = session.read_with(cx, |session, _| session.latest_frame().unwrap());
    assert_eq!(*frames.borrow(), 3);
    assert_eq!(latest.size(0).width.0, 320);
    assert_ne!(latest.id, first.id);

    session.update(cx, |session, cx| session.set_viewport_size(4, 4, cx));
    cx.run_until_parked();
    assert_eq!(
        session.read_with(cx, |session, _| session.latest_frame().unwrap().id),
        latest.id
    );
}

#[gpui::test]
fn probes_fail_typed_and_cache_nothing_on_error(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let missing = ArchivePaths::new(Some(harness.root.path().join("pl0000.dat")), None);

    let empty = stores.inspector.update(cx, |inspector, cx| {
        inspector.probe(ArchivePaths::default(), cx)
    });
    let unreadable = stores
        .inspector
        .update(cx, |inspector, cx| inspector.probe(missing.clone(), cx));
    cx.run_until_parked();

    assert!(matches!(
        cx.foreground_executor().block_test(empty),
        Err(InspectorError::NoArchives)
    ));
    assert!(matches!(
        cx.foreground_executor().block_test(unreadable),
        Err(InspectorError::NoReadableArchive { .. })
    ));
    assert!(
        stores
            .inspector
            .read_with(cx, |inspector, _| inspector.cached(&missing).is_none())
    );

    let opened = stores.inspector.update(cx, |inspector, cx| {
        inspector.open_session(
            missing,
            yp_inspector::CandidateId {
                side: yp_inspector::ArchiveSide::Dat,
                entry_index: 0,
            },
            cx,
        )
    });
    assert!(opened.read_with(cx, |session, _| session.is_loading()));
    cx.run_until_parked();
    assert!(opened.read_with(cx, |session, _| session.error().is_some()));
}

#[gpui::test]
fn the_log_view_follows_a_picked_nams_dll_into_the_game_folder(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let log_file = |stores: &crate::Stores, cx: &mut TestAppContext| {
        stores.logs.read_with(cx, |logs, _| logs.log_file().to_path_buf())
    };
    let launcher_log = harness.env.paths.nams_log();
    assert_eq!(log_file(&stores, cx), launcher_log);
    let dll = harness.game.root().join("dinput8.dll");
    stores
        .launch_options
        .update(cx, |options, cx| options.set_nams_dll(Some(dll), cx));
    cx.run_until_parked();
    assert_eq!(
        log_file(&stores, cx),
        harness.game.nams_dir().join("logs").join("nams.log")
    );
    stores
        .launch_options
        .update(cx, |options, cx| options.set_nams_dll(None, cx));
    cx.run_until_parked();
    assert_eq!(log_file(&stores, cx), launcher_log);
}
