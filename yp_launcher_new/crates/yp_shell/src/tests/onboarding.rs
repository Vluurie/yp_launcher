use gpui::{Entity, TestAppContext, VisualTestContext};
use yp_core::runtime::RuntimeStatus;
use yp_state::prelude::*;
use yp_state::{PlayState, Stores};

use super::{Harness, Opened, has_modal, open_shell};
use crate::Stage;
use crate::onboarding::{OnboardingWizard, Step};

fn open_onboarding(cx: &mut TestAppContext) -> (Opened<'_>, Entity<OnboardingWizard>) {
    let opened = open_shell(cx);
    let shell = opened.shell.clone();
    opened.cx.update(|window, cx| {
        shell.update(cx, |shell, cx| {
            shell.finish_startup(Ok(RuntimeStatus::Bundled), window, cx);
        });
    });
    opened.cx.run_until_parked();
    let wizard = shell
        .read_with(opened.cx, |shell, _| shell.onboarding().cloned())
        .unwrap();
    (opened, wizard)
}

fn step(wizard: &Entity<OnboardingWizard>, cx: &mut VisualTestContext) -> Step {
    wizard.read_with(cx, |wizard, _| wizard.step())
}

fn first_playthrough(stores: &Stores, cx: &mut VisualTestContext) -> Option<bool> {
    stores.nams_settings.read_with(cx, |settings, _| {
        settings
            .settings()
            .get("firstPlaythrough")
            .and_then(|value| value.as_bool())
    })
}

#[gpui::test]
fn wizard_waits_on_the_game_step_until_a_folder_is_selected(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.init(cx);
    let (opened, wizard) = open_onboarding(cx);
    let cx = opened.cx;
    assert_eq!(step(&wizard, cx), Step::Welcome);

    wizard.update(cx, |wizard, cx| wizard.advance(cx));
    assert_eq!(step(&wizard, cx), Step::SelectGame);
    wizard.update(cx, |wizard, cx| wizard.advance(cx));
    assert_eq!(step(&wizard, cx), Step::SelectGame);
    wizard.update(cx, |wizard, cx| wizard.go_back(cx));
    assert_eq!(step(&wizard, cx), Step::Welcome);
    assert!(!stores.session.read_with(cx, |session, _| session.is_directory_selected()));
}

#[gpui::test]
fn wizard_starts_with_the_saved_game_folder_and_walks_every_step(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    std::fs::create_dir_all(harness.game.mods_dir().join("outfit")).unwrap();
    let (opened, wizard) = open_onboarding(cx);
    let cx = opened.cx;
    wizard.read_with(cx, |wizard, _| {
        assert_eq!(wizard.flow().selected_path(), Some(harness.game.root()));
    });

    wizard.update(cx, |wizard, cx| wizard.advance(cx));
    wizard.update(cx, |wizard, cx| wizard.advance(cx));
    assert_eq!(step(&wizard, cx), Step::FirstPlaythrough);
    wizard.read_with(cx, |wizard, _| assert!(wizard.flow().first_playthrough()));

    wizard.update(cx, |wizard, cx| wizard.advance(cx));
    cx.run_until_parked();
    assert_eq!(step(&wizard, cx), Step::Migration);
    wizard.read_with(cx, |wizard, _| {
        let findings = wizard.flow().findings().expect("detection ran");
        assert_eq!(findings.existing_mods, 1);
    });

    wizard.update(cx, |wizard, cx| wizard.answer_had_mods(true, cx));
    assert_eq!(step(&wizard, cx), Step::Migration);
    wizard.update(cx, |wizard, cx| wizard.go_back(cx));
    wizard.read_with(cx, |wizard, _| {
        assert_eq!(wizard.flow().had_mods_before(), None);
    });
    wizard.update(cx, |wizard, cx| wizard.answer_had_mods(false, cx));
    assert_eq!(step(&wizard, cx), Step::Mods);

    wizard.update(cx, |wizard, cx| wizard.answer_wants_mod_install(false, cx));
    assert_eq!(step(&wizard, cx), Step::Ready);
    wizard.update(cx, |wizard, cx| wizard.advance(cx));
    assert_eq!(step(&wizard, cx), Step::Ready);
    assert_eq!(
        stores.session.read_with(cx, |session, _| session.play_state()),
        PlayState::Idle
    );
}

#[gpui::test]
fn first_playthrough_choice_is_written_to_the_overlay_settings(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let (opened, wizard) = open_onboarding(cx);
    let cx = opened.cx;

    wizard.update(cx, |wizard, cx| wizard.show_step(Step::FirstPlaythrough, cx));
    cx.simulate_keystrokes("shift");
    cx.run_until_parked();
    let saved = wizard.update(cx, |wizard, cx| {
        wizard.choose_first_playthrough(false, cx);
        wizard.save_first_playthrough(cx);
        wizard.flow().first_playthrough()
    });
    cx.run_until_parked();
    assert!(!saved);
    assert_eq!(step(&wizard, cx), Step::Migration);
    assert_eq!(first_playthrough(&stores, cx), Some(false));
    assert!(!stores.nams_settings.read_with(cx, |settings, _| settings.is_dirty()));
}

#[gpui::test]
fn finishing_marks_onboarding_done_and_shows_the_launcher(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let stores = harness.start(cx);
    let (opened, wizard) = open_onboarding(cx);
    let cx = opened.cx;
    assert!(!cx.update(|_, cx| cx.prefs().onboarding_complete()));

    wizard.update(cx, |wizard, cx| wizard.show_step(Step::Ready, cx));
    wizard.update(cx, |wizard, cx| wizard.finish(false, cx));
    cx.run_until_parked();

    assert!(cx.update(|_, cx| cx.prefs().onboarding_complete()));
    opened.shell.read_with(cx, |shell, _| {
        assert_eq!(shell.stage(), Stage::Main);
        assert!(shell.onboarding().is_none());
    });
    assert_eq!(
        stores.session.read_with(cx, |session, _| session.play_state()),
        PlayState::Idle
    );
    assert!(!has_modal(&opened.root, cx));
}

#[gpui::test]
fn play_from_the_last_step_launches_and_reports_a_failure(cx: &mut TestAppContext) {
    let harness = Harness::without_runtime();
    let stores = harness.start(cx);
    let (opened, wizard) = open_onboarding(cx);
    let cx = opened.cx;

    wizard.update(cx, |wizard, cx| wizard.show_step(Step::Ready, cx));
    wizard.update(cx, |wizard, cx| wizard.finish(true, cx));
    cx.run_until_parked();

    opened.shell.read_with(cx, |shell, _| assert_eq!(shell.stage(), Stage::Main));
    assert!(stores.session.read_with(cx, |session, _| session.launch_failure().is_some()));
    assert!(has_modal(&opened.root, cx));
}

#[gpui::test]
fn unsupported_platform_keeps_the_wizard_on_its_notice(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness
        .platform
        .can_launch
        .store(false, std::sync::atomic::Ordering::SeqCst);
    harness.start(cx);
    let (opened, wizard) = open_onboarding(cx);
    let cx = opened.cx;
    cx.simulate_keystrokes("shift");
    cx.run_until_parked();
    assert_eq!(step(&wizard, cx), Step::Welcome);
    assert!(!cx.update(|_, cx| cx.env().can_launch_game()));
}

#[gpui::test]
fn a_dropped_texture_pack_counts_as_installed(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let _stores = harness.start(cx);
    cx.update(|cx| {
        cx.registry_mut().flows.set_mod_install(|request, _, _| {
            assert!(request.texture_packs);
            gpui::Task::ready(yp_state::ModInstallReport {
                texture_packs: vec!["HD Pack".to_owned()],
                ..yp_state::ModInstallReport::default()
            })
        });
    });
    let (opened, wizard) = open_onboarding(cx);
    let cx = opened.cx;
    assert!(wizard.read_with(cx, |wizard, _| wizard.installed().is_empty()));

    wizard.update_in(cx, |wizard, window, cx| {
        wizard.install(vec!["HD Pack.zip".into()], window, cx);
    });
    cx.run_until_parked();
    let installed = wizard.read_with(cx, |wizard, _| wizard.installed().to_vec());
    assert_eq!(installed, ["HD Pack (texture pack)"]);
}
