use std::sync::Arc;
use std::sync::atomic::Ordering;

use gpui::{AppContext as _, TestAppContext};
use yp_core::NierInstallation;
use yp_state::prelude::*;

use super::{Harness, has_modal, has_toast, open_main};
use crate::DirectorySelector;

#[gpui::test]
fn manual_selection_saves_the_game_folder_and_restores_it_from_disk(cx: &mut TestAppContext) {
    let mut harness = Harness::new();
    let game = harness.game.root().join("mounted game with spaces 日本語");
    std::fs::create_dir_all(game.join("data")).unwrap();
    let exe = game.join(yp_core::consts::GAME_EXE_NAME);
    std::fs::write(&exe, b"MZ\0\0PRJ_028").unwrap();
    let settings_file = harness.env.paths.settings_file.clone();
    harness.env.settings = Arc::new(
        yp_core::SettingsStore::open(settings_file.clone())
            .unwrap()
            .0,
    );
    let stores = harness.init(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let selector = cx.update(|window, cx| cx.new(|cx| DirectorySelector::new(window, cx)));
    selector.update_in(cx, |selector, window, cx| {
        selector.run_manual_select(window, cx)
    });
    cx.run_until_parked();
    assert!(cx.did_prompt_for_paths());
    cx.simulate_path_prompt_response(|_| Some(vec![exe]));
    cx.run_until_parked();
    assert_eq!(
        stores
            .session
            .read_with(cx, |session, _| session.selected_dir()),
        Some(game.clone())
    );
    assert!(
        stores
            .session
            .read_with(cx, |session, _| session.error().is_none())
    );
    let flush = cx.update(|_, cx| cx.prefs().flush());
    cx.foreground_executor().block_test(flush).unwrap();
    let settings = yp_core::SettingsStore::open(settings_file).unwrap().0;
    assert_eq!(
        yp_host::session::AppSession::load(&settings).directory(),
        Some(game)
    );
}

#[gpui::test]
fn manual_selection_reports_a_save_failure_and_can_retry_the_same_path(cx: &mut TestAppContext) {
    let mut harness = Harness::new();
    std::fs::write(harness.game.exe(), b"MZ\0\0PRJ_028").unwrap();
    let settings_file = harness.env.paths.settings_file.clone();
    harness.env.settings = Arc::new(
        yp_core::SettingsStore::open(settings_file.clone())
            .unwrap()
            .0,
    );
    std::fs::create_dir_all(&settings_file).unwrap();
    let stores = harness.init(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let selector = cx.update(|window, cx| cx.new(|cx| DirectorySelector::new(window, cx)));
    selector.update_in(cx, |selector, window, cx| {
        selector.run_manual_select(window, cx)
    });
    cx.run_until_parked();
    cx.simulate_path_prompt_response(|_| Some(vec![harness.game.exe()]));
    cx.run_until_parked();
    assert!(
        !stores
            .session
            .read_with(cx, |session, _| session.is_directory_selected())
    );
    assert!(
        stores
            .session
            .read_with(cx, |session, _| session.error().is_some())
    );
    assert!(
        harness
            .env
            .settings
            .get(&yp_state::prefs::NIER_DIRECTORY)
            .is_empty()
    );
    std::fs::remove_dir(&settings_file).unwrap();
    selector.update_in(cx, |selector, window, cx| {
        selector.run_manual_select(window, cx)
    });
    cx.run_until_parked();
    cx.simulate_path_prompt_response(|_| Some(vec![harness.game.exe()]));
    cx.run_until_parked();
    assert_eq!(
        stores
            .session
            .read_with(cx, |session, _| session.selected_dir()),
        Some(harness.game.root().to_path_buf())
    );
    assert!(
        stores
            .session
            .read_with(cx, |session, _| session.error().is_none())
    );
    let settings = yp_core::SettingsStore::open(settings_file).unwrap().0;
    assert_eq!(
        yp_host::session::AppSession::load(&settings).directory(),
        Some(harness.game.root().to_path_buf())
    );
}

#[gpui::test]
fn onboarding_reports_invalid_manual_selections_instead_of_discarding_them(
    cx: &mut TestAppContext,
) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let selecting = cx.update(|window, cx| {
        window.spawn(cx, async |cx| {
            crate::directory::select_manually(crate::directory::ManualRejection::ShowAsToast, cx)
                .await
        })
    });
    cx.run_until_parked();
    cx.simulate_path_prompt_response(|_| Some(vec![harness.game.exe()]));
    cx.run_until_parked();
    assert!(has_toast("game_selection_failed", cx));
    assert!(cx.foreground_executor().block_test(selecting).is_none());
    assert!(
        !stores
            .session
            .read_with(cx, |session, _| session.is_directory_selected())
    );
}

#[gpui::test]
fn auto_detect_scans_drives_when_fast_search_is_empty(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness
        .platform
        .deep_installs
        .lock()
        .unwrap()
        .push(NierInstallation {
            path: harness.game.root().to_path_buf(),
            has_data: true,
        });
    let stores = harness.init(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    let selector = cx.update(|window, cx| cx.new(|cx| DirectorySelector::new(window, cx)));
    selector.update_in(cx, |selector, window, cx| {
        selector.run_auto_search(window, cx)
    });
    cx.run_until_parked();
    assert_eq!(harness.platform.deep_searches.load(Ordering::SeqCst), 1);
    assert!(has_modal(&opened.root, cx));
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(!has_modal(&opened.root, cx));
    assert!(
        !stores
            .session
            .read_with(cx, |session, _| session.auto_searching())
    );
}
