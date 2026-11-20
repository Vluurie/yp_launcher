use std::cell::RefCell;
use std::rc::Rc;

use gpui::TestAppContext;
use yp_config::fields::nams;

use super::Harness;
use crate::registry::OverlayRequest;
use crate::{
    BusyScope, CloseGuard, NavigationBlock, NavigationEvent, OverlayId, Stores, TabId,
    TabRequestOutcome,
};

fn request(
    stores: &Stores,
    tab: TabId,
    select: Option<&str>,
    cx: &mut TestAppContext,
) -> TabRequestOutcome {
    stores.navigation.update(cx, |navigation, cx| {
        navigation.request_tab(tab, select.map(str::to_owned), cx)
    })
}

fn active(stores: &Stores, cx: &mut TestAppContext) -> TabId {
    stores
        .navigation
        .read_with(cx, |navigation, _| navigation.active())
}

fn make_config_dirty(stores: &Stores, cx: &mut TestAppContext) {
    stores.config.update(cx, |config, cx| {
        config.update_nams(
            nams::DISABLE_PLUGIN_LOADING.key(),
            toml::Value::Boolean(true),
            None,
            cx,
        );
    });
}

#[gpui::test]
fn tabs_switch_and_hand_over_their_selection_once(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    assert_eq!(active(&stores, cx), TabId::Launcher);

    assert_eq!(
        request(&stores, TabId::Textures, Some("pack_a"), cx),
        TabRequestOutcome::Switched
    );
    assert_eq!(active(&stores, cx), TabId::Textures);
    stores.navigation.update(cx, |navigation, _| {
        assert!(navigation.has_pending_selection(TabId::Textures));
        assert_eq!(navigation.take_selection(TabId::Mods), None);
        assert_eq!(
            navigation.take_selection(TabId::Textures).as_deref(),
            Some("pack_a")
        );
        assert_eq!(navigation.take_selection(TabId::Textures), None);
    });

    assert_eq!(
        request(&stores, TabId::Textures, None, cx),
        TabRequestOutcome::AlreadyActive
    );
    assert_eq!(
        request(&stores, TabId::Textures, Some("pack_b"), cx),
        TabRequestOutcome::AlreadyActive
    );
    assert_eq!(
        stores.navigation.update(cx, |navigation, _| navigation
            .take_selection(TabId::Textures)),
        Some("pack_b".to_owned())
    );
}

#[gpui::test]
fn docs_open_as_an_overlay_without_leaving_the_tab(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    request(&stores, TabId::Mods, None, cx);

    assert_eq!(
        request(&stores, TabId::Docs, Some("native/35_third_party"), cx),
        TabRequestOutcome::OverlayOpened(OverlayId::Docs)
    );

    assert_eq!(active(&stores, cx), TabId::Mods);
    stores.navigation.update(cx, |navigation, cx| {
        assert!(navigation.is_overlay_open(OverlayId::Docs));
        assert_eq!(
            navigation.top_overlay(),
            Some(&OverlayRequest::Docs {
                page: Some("native/35_third_party".to_owned())
            })
        );
        navigation.open_overlay(
            OverlayRequest::Inspector {
                archives: Default::default(),
            },
            cx,
        );
        assert_eq!(navigation.overlays().len(), 2);
        navigation.close_top_overlay(cx);
        navigation.close_overlay(OverlayId::Docs, cx);
        assert!(navigation.overlays().is_empty());
    });
}

#[gpui::test]
fn busy_textures_block_tab_changes(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    request(&stores, TabId::Textures, None, cx);
    let blocked = Rc::new(RefCell::new(Vec::new()));
    let seen = blocked.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.navigation, move |_, event, _| {
            if let NavigationEvent::Blocked(block) = event {
                seen.borrow_mut().push(*block);
            }
        })
    });

    let handle = stores
        .busy
        .update(cx, |busy, cx| busy.enter(BusyScope::Textures, cx));
    assert_eq!(
        request(&stores, TabId::Mods, None, cx),
        TabRequestOutcome::Blocked(NavigationBlock::TexturesBusy)
    );
    cx.run_until_parked();
    assert_eq!(active(&stores, cx), TabId::Textures);
    assert_eq!(*blocked.borrow(), vec![NavigationBlock::TexturesBusy]);

    drop(handle);
    cx.run_until_parked();
    assert_eq!(
        request(&stores, TabId::Mods, None, cx),
        TabRequestOutcome::Switched
    );
}

#[gpui::test]
fn unsaved_changes_ask_before_switching(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    request(&stores, TabId::Nams, None, cx);
    make_config_dirty(&stores, cx);

    assert_eq!(
        request(&stores, TabId::Mods, Some("kimono"), cx),
        TabRequestOutcome::ConfirmDiscard
    );
    assert_eq!(active(&stores, cx), TabId::Nams);
    stores.navigation.update(cx, |navigation, cx| {
        assert_eq!(
            navigation.awaiting_discard().map(|request| request.tab),
            Some(TabId::Mods)
        );
        navigation.cancel_discard(cx);
        assert!(navigation.awaiting_discard().is_none());
    });
    assert_eq!(active(&stores, cx), TabId::Nams);
    assert!(stores.config.read_with(cx, |config, _| config.is_dirty()));

    request(&stores, TabId::Mods, Some("kimono"), cx);
    stores
        .navigation
        .update(cx, |navigation, cx| navigation.confirm_discard(cx));
    cx.run_until_parked();

    assert_eq!(active(&stores, cx), TabId::Mods);
    assert!(!stores.config.read_with(cx, |config, _| config.is_dirty()));
    assert!(!stores.config.read_with(cx, |config, _| {
        nams::DISABLE_PLUGIN_LOADING.value_in(config.nams())
    }));
    assert_eq!(
        stores
            .navigation
            .update(cx, |navigation, _| navigation.take_selection(TabId::Mods)),
        Some("kimono".to_owned())
    );
}

#[gpui::test]
fn overlay_settings_changes_also_guard_navigation(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    request(&stores, TabId::YorhaProtocol, None, cx);
    stores.nams_settings.update(cx, |settings, cx| {
        settings.update_toggle("impeller", true, cx);
    });

    assert_eq!(
        request(&stores, TabId::Launcher, None, cx),
        TabRequestOutcome::ConfirmDiscard
    );
    stores
        .navigation
        .update(cx, |navigation, cx| navigation.confirm_discard(cx));
    cx.run_until_parked();

    assert_eq!(active(&stores, cx), TabId::Launcher);
    assert!(
        !stores
            .nams_settings
            .read_with(cx, |settings, _| settings.is_dirty())
    );
}

#[gpui::test]
fn close_guard_prefers_busy_work_over_unsaved_changes(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let guard = |stores: &Stores, cx: &mut TestAppContext| {
        stores
            .navigation
            .read_with(cx, |navigation, cx| navigation.close_guard(cx))
    };
    assert_eq!(guard(&stores, cx), CloseGuard::Clear);

    make_config_dirty(&stores, cx);
    assert_eq!(guard(&stores, cx), CloseGuard::UnsavedChanges);

    let first = stores
        .busy
        .update(cx, |busy, cx| busy.enter(BusyScope::General, cx));
    let second = stores
        .busy
        .update(cx, |busy, cx| busy.enter(BusyScope::Textures, cx));
    assert_eq!(guard(&stores, cx), CloseGuard::Busy { operations: 2 });

    drop((first, second));
    assert_eq!(guard(&stores, cx), CloseGuard::UnsavedChanges);
}

#[gpui::test]
fn log_sheet_state_is_tracked(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let toggles = Rc::new(RefCell::new(Vec::new()));
    let seen = toggles.clone();
    let _subscription = cx.update(|cx| {
        cx.subscribe(&stores.navigation, move |_, event, _| {
            if let NavigationEvent::LogsToggled(open) = event {
                seen.borrow_mut().push(*open);
            }
        })
    });

    stores.navigation.update(cx, |navigation, cx| {
        navigation.open_logs(cx);
        navigation.open_logs(cx);
        navigation.toggle_logs(cx);
        navigation.toggle_logs(cx);
        navigation.close_logs(cx);
    });
    cx.run_until_parked();

    assert_eq!(*toggles.borrow(), vec![true, false, true, false]);
    assert!(
        !stores
            .navigation
            .read_with(cx, |navigation, _| navigation.log_open())
    );
}

#[gpui::test]
fn tab_ids_keep_the_flutter_indices(_cx: &mut TestAppContext) {
    let expected = [
        (TabId::Launcher, 0),
        (TabId::Nams, 1),
        (TabId::LodMod, 2),
        (TabId::Textures, 3),
        (TabId::YorhaProtocol, 4),
        (TabId::Mods, 5),
        (TabId::Naiom, 6),
        (TabId::Cutscenes, 7),
        (TabId::LauncherSettings, 8),
        (TabId::ThirdParty, 9),
        (TabId::Docs, 10),
        (TabId::Tools, 11),
    ];
    for (tab, index) in expected {
        assert_eq!(tab.index(), index);
        assert_eq!(TabId::from_index(index), Some(tab));
        assert!(tab.section().tabs().contains(&tab));
    }
    assert_eq!(TabId::from_index(12), None);
}

#[gpui::test]
fn default_action_handlers_drive_the_stores(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let generation = stores
        .detection
        .read_with(cx, |detection, _| detection.generation());

    cx.update(|cx| {
        crate::register_default_handlers(cx);
        cx.dispatch_action(&crate::OpenTab::selecting(TabId::Mods, "kimono"));
        cx.dispatch_action(&crate::OpenLogs);
        cx.dispatch_action(&crate::OpenDocs {
            page: Some("home".to_owned()),
        });
        cx.dispatch_action(&crate::OpenThreeDInspector::default());
        cx.dispatch_action(&crate::RefreshDetections);
    });
    cx.run_until_parked();

    assert_eq!(active(&stores, cx), TabId::Mods);
    stores.navigation.update(cx, |navigation, _| {
        assert!(navigation.log_open());
        assert!(navigation.is_overlay_open(OverlayId::Docs));
        assert!(navigation.is_overlay_open(OverlayId::Inspector));
        assert_eq!(
            navigation.take_selection(TabId::Mods).as_deref(),
            Some("kimono")
        );
    });
    assert!(
        stores
            .detection
            .read_with(cx, |detection, _| detection.generation())
            > generation
    );

    cx.update(|cx| {
        cx.dispatch_action(&crate::CloseLogs);
        cx.dispatch_action(&crate::Play);
    });
    cx.run_until_parked();
    assert!(
        !stores
            .navigation
            .read_with(cx, |navigation, _| navigation.log_open())
    );
    assert!(!harness.runner.calls().is_empty());
}
