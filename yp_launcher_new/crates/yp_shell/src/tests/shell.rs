use gpui::TestAppContext;
use yp_core::runtime::RuntimeStatus;
use yp_i18n::Msg;
use yp_state::prefs::SIDEBAR_SECTION_COLLAPSED;
use yp_state::prelude::*;
use yp_state::{
    BusyScope, GameError, LaunchRequest, Notice, NoticeKind, NoticeSeverity, OverlayId,
    OverlayRequest, SidebarSection, Stores,
};
use yp_ui::{ToastId, WindowExt};

use super::{Harness, has_modal, has_toast, open_main, open_shell};
use crate::Stage;
use crate::shell::overlay_key;
use crate::sidebar;

fn active(stores: &Stores, cx: &mut TestAppContext) -> TabId {
    stores
        .navigation
        .read_with(cx, |navigation, _| navigation.active())
}

#[gpui::test]
fn shell_shows_the_splash_until_startup_finishes(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_shell(cx);
    let shell = opened.shell.clone();
    assert_eq!(shell.read_with(opened.cx, |shell, _| shell.stage()), Stage::Splash);

    opened.cx.update(|window, cx| {
        shell.update(cx, |shell, cx| {
            shell.finish_startup(Ok(RuntimeStatus::Bundled), window, cx);
        });
    });
    opened.cx.run_until_parked();
    shell.read_with(opened.cx, |shell, _| {
        assert_eq!(shell.stage(), Stage::Onboarding);
        assert!(shell.onboarding().is_some());
    });
}

#[gpui::test]
fn finished_onboarding_goes_straight_to_the_main_screen(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_main(cx);
    opened.shell.read_with(opened.cx, |shell, _| {
        assert_eq!(shell.stage(), Stage::Main);
        assert!(shell.onboarding().is_none());
        assert_eq!(shell.cached_tabs(), vec![TabId::Launcher]);
    });
}

#[gpui::test]
fn failed_runtime_setup_is_reported_as_a_toast(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    cx.update(|cx| cx.prefs().set_onboarding_complete(true));
    let opened = open_shell(cx);
    let shell = opened.shell.clone();
    opened.cx.update(|window, cx| {
        shell.update(cx, |shell, cx| {
            shell.finish_startup(
                Err(yp_core::Error::invalid("runtime", "disk full")),
                window,
                cx,
            );
        });
    });
    opened.cx.run_until_parked();
    let notices = stores
        .notifications
        .read_with(opened.cx, |store, _| store.items().to_vec());
    assert_eq!(notices.len(), 1);
    assert!(matches!(notices[0].msg, Msg::ShellRuntimeSetupFailed { .. }));
    assert_eq!(notices[0].severity, NoticeSeverity::Error);
    let id = notices[0].id.clone();
    let shown = opened.cx.update(|window, cx| {
        window
            .toast_stack(cx)
            .is_some_and(|stack| stack.read(cx).contains(&ToastId::from(id.clone())))
    });
    assert!(shown);
}

#[gpui::test]
fn tabs_are_built_once_and_missing_features_show_a_placeholder(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;

    cx.dispatch_action(OpenTab::new(TabId::LauncherSettings));
    cx.run_until_parked();
    assert_eq!(active(&stores, cx), TabId::LauncherSettings);
    opened.shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.cached_tabs(),
            vec![TabId::Launcher, TabId::LauncherSettings]
        );
    });

    cx.dispatch_action(OpenTab::new(TabId::Mods));
    cx.run_until_parked();
    assert_eq!(active(&stores, cx), TabId::Mods);
    cx.dispatch_action(OpenTab::new(TabId::Launcher));
    cx.run_until_parked();
    opened.shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.cached_tabs(),
            vec![TabId::Launcher, TabId::LauncherSettings]
        );
    });
}

#[gpui::test]
fn texture_installs_block_navigation_with_a_toast(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let busy = stores
        .busy
        .update(cx, |busy, cx| busy.enter(BusyScope::Textures, cx));

    cx.dispatch_action(OpenTab::new(TabId::LauncherSettings));
    cx.run_until_parked();
    assert_eq!(active(&stores, cx), TabId::Launcher);
    assert!(has_toast("textures_busy", cx));
    assert!(!has_modal(&opened.root, cx));

    drop(busy);
    cx.run_until_parked();
    cx.dispatch_action(OpenTab::new(TabId::LauncherSettings));
    cx.run_until_parked();
    assert_eq!(active(&stores, cx), TabId::LauncherSettings);
}

#[gpui::test]
fn a_clean_window_closes_without_asking(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_main(cx);
    let allowed = opened
        .shell
        .update_in(opened.cx, |shell, window, cx| shell.should_close(window, cx));
    assert!(allowed);
    assert!(!has_modal(&opened.root, opened.cx));
}

#[gpui::test]
fn closing_while_busy_asks_and_stay_keeps_the_window(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let _busy = stores
        .busy
        .update(cx, |busy, cx| busy.enter(BusyScope::General, cx));

    let allowed = opened
        .shell
        .update_in(cx, |shell, window, cx| shell.should_close(window, cx));
    cx.run_until_parked();
    assert!(!allowed);
    assert!(has_modal(&opened.root, cx));
    assert!(opened.shell.read_with(cx, |shell, _| shell.is_asking_to_close()));

    let again = opened
        .shell
        .update_in(cx, |shell, window, cx| shell.should_close(window, cx));
    cx.run_until_parked();
    assert!(!again);
    assert_eq!(opened.root.read_with(cx, |root, _| root.modal_count()), 1);

    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!has_modal(&opened.root, cx));
    assert!(!opened.shell.read_with(cx, |shell, _| shell.is_asking_to_close()));
    assert_eq!(cx.windows().len(), 1);
}

#[gpui::test]
fn force_close_removes_the_window(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let _busy = stores
        .busy
        .update(cx, |busy, cx| busy.enter(BusyScope::General, cx));

    let allowed = opened
        .shell
        .update_in(cx, |shell, window, cx| shell.should_close(window, cx));
    cx.run_until_parked();
    assert!(!allowed);
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(cx.windows().is_empty());
}

#[gpui::test]
fn sidebar_collapse_state_is_saved_and_restored(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let collapsed = |cx: &mut TestAppContext| {
        cx.update(|cx| {
            sidebar::is_collapsed(SidebarSection::Config, &sidebar::collapsed_sections(cx))
        })
    };
    assert!(collapsed(cx));

    cx.update(|cx| sidebar::toggle_section(SidebarSection::Config, cx));
    assert!(!collapsed(cx));
    let mut flushed = cx.update(|cx| cx.prefs().flush());
    cx.run_until_parked();
    assert_eq!(flushed.try_recv(), Ok(Some(())));
    let saved = harness.env.settings.get(&SIDEBAR_SECTION_COLLAPSED);
    assert_eq!(saved.get("Config"), Some(&false));

    cx.update(|cx| sidebar::toggle_section(SidebarSection::General, cx));
    assert_eq!(
        cx.update(|cx| sidebar::collapsed_sections(cx).len()),
        1
    );
    cx.update(|cx| sidebar::toggle_section(SidebarSection::Config, cx));
    assert!(collapsed(cx));
}

#[gpui::test]
fn log_panel_follows_the_navigation_store(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let has_sheet =
        |cx: &mut gpui::VisualTestContext| opened.root.read_with(cx, |root, _| root.has_sheet());

    cx.dispatch_action(OpenLogs);
    cx.run_until_parked();
    assert!(has_sheet(cx));
    assert!(opened.shell.read_with(cx, |shell, _| shell.is_log_panel_open()));
    assert!(stores.logs.read_with(cx, |logs, _| logs.is_streaming()));

    cx.dispatch_action(CloseLogs);
    cx.run_until_parked();
    assert!(!has_sheet(cx));
    assert!(!opened.shell.read_with(cx, |shell, _| shell.is_log_panel_open()));
    assert!(!stores.logs.read_with(cx, |logs, _| logs.is_streaming()));

    cx.dispatch_action(OpenLogs);
    cx.run_until_parked();
    assert!(has_sheet(cx));
    let slide_out = cx.update(|_, cx| yp_ui::ActiveTheme::theme(cx).motion.cross);
    cx.update(|window, cx| window.close_sheet(cx));
    cx.run_until_parked();
    assert!(!has_sheet(cx));
    cx.executor().advance_clock(slide_out * 2);
    cx.run_until_parked();
    assert!(!stores.navigation.read_with(cx, |navigation, _| navigation.log_open()));
    assert!(!stores.logs.read_with(cx, |logs, _| logs.is_streaming()));
}

#[gpui::test]
fn notifications_become_toasts_and_dismissal_goes_both_ways(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let push = |id: &'static str, cx: &mut gpui::VisualTestContext| {
        let notice = Notice::new(
            id,
            Msg::NotifyShortcutCreated,
            NoticeKind::Shortcut,
            NoticeSeverity::Success,
        );
        stores
            .notifications
            .update(cx, |store, cx| store.push(notice, cx));
        cx.run_until_parked();
    };

    push("first", cx);
    push("second", cx);
    assert!(has_toast("first", cx));
    assert!(has_toast("second", cx));

    stores
        .notifications
        .update(cx, |store, cx| store.dismiss("first", cx));
    cx.run_until_parked();
    cx.executor().advance_clock(std::time::Duration::from_secs(2));
    cx.run_until_parked();
    assert!(!has_toast("first", cx));

    cx.update(|window, cx| window.dismiss_toast(&ToastId::from("second"), cx));
    cx.run_until_parked();
    cx.executor().advance_clock(std::time::Duration::from_secs(2));
    cx.run_until_parked();
    assert!(
        !stores
            .notifications
            .read_with(cx, |store, _| store.contains("second"))
    );
}

#[gpui::test]
fn notices_from_before_the_window_are_shown_once_it_opens(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    stores.notifications.update(cx, |store, cx| {
        store.push(
            Notice::new(
                "early",
                Msg::NotifyShortcutFailed,
                NoticeKind::General,
                NoticeSeverity::Error,
            ),
            cx,
        );
    });
    let opened = open_main(cx);
    opened.cx.simulate_keystrokes("shift");
    opened.cx.run_until_parked();
    assert!(has_toast("early", opened.cx));
}

#[gpui::test]
fn launch_failure_opens_the_failure_dialog(cx: &mut TestAppContext) {
    let harness = Harness::without_runtime();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;

    stores
        .session
        .update(cx, |session, cx| session.launch(LaunchRequest::default(), cx));
    cx.run_until_parked();

    assert!(has_modal(&opened.root, cx));
    assert!(stores.session.read_with(cx, |session, _| matches!(
        session.error(),
        Some(GameError::LaunchFailed(_))
    )));
}

#[gpui::test]
fn overlays_without_a_feature_close_again_with_a_toast(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;

    cx.dispatch_action(OpenDocs::default());
    cx.run_until_parked();
    assert!(has_toast("overlay_unavailable", cx));
    assert!(
        !stores
            .navigation
            .read_with(cx, |navigation, _| navigation.is_overlay_open(OverlayId::Docs))
    );
}

#[gpui::test]
fn registered_overlays_are_shown_and_hidden_with_navigation(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    cx.update(|cx| {
        cx.registry_mut()
            .overlays
            .register(OverlayId::Docs, |_, _, cx| {
                gpui::AppContext::new(cx, |_| gpui::Empty).into()
            });
    });
    let opened = open_main(cx);
    let cx = opened.cx;
    let shown = |cx: &mut gpui::VisualTestContext| {
        opened
            .root
            .read_with(cx, |root, _| root.has_overlay(overlay_key(OverlayId::Docs)))
    };

    cx.dispatch_action(OpenDocs::default());
    cx.run_until_parked();
    assert!(shown(cx));
    stores.navigation.update(cx, |navigation, cx| {
        assert_eq!(
            navigation.top_overlay(),
            Some(&OverlayRequest::Docs { page: None })
        );
        navigation.close_overlay(OverlayId::Docs, cx);
    });
    cx.run_until_parked();
    assert!(!shown(cx));
}

#[gpui::test]
fn diagnostics_action_collects_and_opens_the_dialog(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;

    cx.dispatch_action(OpenDiagnostics);
    cx.run_until_parked();
    assert_eq!(opened.root.read_with(cx, |root, _| root.modal_count()), 1);
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!has_modal(&opened.root, cx));
}

#[gpui::test]
fn links_open_in_the_browser_outside_the_sandbox(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    cx.update(|cx| crate::folders::open_link(yp_core::consts::GUIDE_URL, cx));
    assert_eq!(
        cx.opened_url().as_deref(),
        Some(yp_core::consts::GUIDE_URL)
    );
}

#[gpui::test]
fn sandbox_never_opens_links(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    cx.update(|cx| {
        cx.set_global(crate::ShellOptions::sandboxed(harness.game.root()));
        crate::folders::open_link(yp_core::consts::GUIDE_URL, cx);
    });
    assert_eq!(cx.opened_url(), None);
}
