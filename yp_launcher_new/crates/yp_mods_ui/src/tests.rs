use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui::{AnyView, App, Entity, TestAppContext, VisualTestContext, px, size};
use yp_mods::camera::CameraField;
use yp_mods::grouping::ModGroupKind;
use yp_state::prefs::OUTFIT_GUIDE_SHOWN;
use yp_state::prelude::*;
use yp_state::{ModInstallMode, ModInstallReport, ModInstallRequest, Registry, Stores};
use yp_thirdparty::{RuntimePresence, ThirdPartyRuntime};
use yp_ui::theme::ThemeId;
use yp_ui::{ActiveStrings, Msg, Root, Severity, WindowExt};

use crate::cutscenes::CutscenesView;
use crate::cutscenes::flow::{delete_cutscene, install_from_path};
use crate::cutscenes::model::merge_targets;
use crate::mods::ModsView;
use crate::mods::install;
use crate::mods::list::ListRow;
use crate::mods::profile::{create_profile, delete_profile, rename_profile, switch_profile};
use crate::mods::tutorial::TutorialKind;
use crate::testing::{CODEC_H264, CODEC_MPEG2, Sandbox};
use crate::thirdparty::ThirdPartyView;

const SETTLE: Duration = Duration::from_millis(50);

fn boot_without_game(sandbox: &Sandbox, cx: &mut TestAppContext) -> Stores {
    let env = sandbox.env.clone();
    cx.update(|cx| {
        yp_ui::init_for_tests(cx);
        yp_state::init_with(env, cx);
        crate::init(cx);
        cx.stores().clone()
    })
}

fn boot(sandbox: &Sandbox, cx: &mut TestAppContext) -> Stores {
    let stores = boot_without_game(sandbox, cx);
    let dir = sandbox.game.root().to_path_buf();
    stores
        .session
        .update(cx, |session, cx| session.set_directory(dir, cx));
    cx.run_until_parked();
    stores
}

fn settle(cx: &mut VisualTestContext) {
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
}

fn open_tab<V: 'static>(
    tab: TabId,
    cx: &mut TestAppContext,
) -> (Entity<V>, &mut VisualTestContext) {
    let slot: Rc<RefCell<Option<AnyView>>> = Rc::default();
    let built = slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let view = Registry::build_tab(tab, window, cx).expect("the tab is registered");
        *built.borrow_mut() = Some(view.clone());
        Root::new(view, window, cx)
    });
    let view = slot
        .borrow_mut()
        .take()
        .and_then(|view| view.downcast::<V>().ok())
        .expect("the tab builds its own view");
    settle(cx);
    (view, cx)
}

fn has_modal(cx: &mut VisualTestContext) -> bool {
    cx.update(|window, cx: &mut App| window.has_modal(cx))
}

fn pushed(stores: &Stores, cx: &mut VisualTestContext) -> Vec<(Msg, Severity)> {
    stores.notifications.read_with(cx, |store, _| {
        store
            .items()
            .iter()
            .map(|notice| (notice.msg.clone(), notice.severity))
            .collect()
    })
}

fn was_told(stores: &Stores, msg: Msg, severity: Severity, cx: &mut VisualTestContext) -> bool {
    pushed(stores, cx).contains(&(msg, severity))
}

fn mod_ids(stores: &Stores, cx: &mut VisualTestContext) -> Vec<String> {
    stores.mods.read_with(cx, |mods, _| {
        mods.mods()
            .iter()
            .map(|installed| installed.id.clone())
            .collect()
    })
}

fn rows(view: &Entity<ModsView>, cx: &mut VisualTestContext) -> Vec<ListRow> {
    view.read_with(cx, |view, cx| view.visible_rows(cx))
}

fn row_ids(rows: &[ListRow]) -> Vec<&str> {
    rows.iter()
        .filter_map(|row| match row {
            ListRow::Mod { id, .. } => Some(id.as_str()),
            ListRow::Header { .. } => None,
        })
        .collect()
}

fn header_of(rows: &[ListRow], wanted: ModGroupKind) -> Option<(usize, bool)> {
    rows.iter().find_map(|row| match row {
        ListRow::Header {
            kind,
            count,
            collapsed,
        } if *kind == wanted => Some((*count, *collapsed)),
        _ => None,
    })
}

fn selected(view: &Entity<ModsView>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |view, _| view.selected_id().map(str::to_owned))
}

fn library(sandbox: &Sandbox) {
    sandbox.add_native_mod("bunker_free_cam", "Bunker Free-Cam", "1.0");
    sandbox.add_camera_settings("bunker_free_cam");
    sandbox.add_native_mod("grain_removal", "Grain Removal", "1.2");
    sandbox.add_outfit_mod("kimono", "pl000d");
    sandbox.add_bundled_textures("kimono", &["Kimono 4K (kimono@default)"]);
    sandbox.add_wax_mod("wax_alice", "pl000d");
    sandbox.add_data_file("gold_sword", "wp", "wp0000.dat", 64);
}

fn guided(sources: Vec<PathBuf>) -> ModInstallRequest {
    ModInstallRequest {
        sources,
        mode: ModInstallMode::Guided,
        texture_packs: false,
    }
}

fn start_install(
    request: ModInstallRequest,
    cx: &mut VisualTestContext,
) -> gpui::Task<ModInstallReport> {
    let installing = cx.update(|window, cx| install::run(request, window, cx));
    cx.run_until_parked();
    installing
}

#[gpui::test]
fn init_registers_the_tabs_and_the_install_flow(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    cx.update(|cx| {
        let registry = cx.registry();
        for tab in [TabId::Mods, TabId::Cutscenes, TabId::ThirdParty] {
            assert!(registry.tabs.contains(tab), "{tab:?}");
        }
        assert!(registry.flows.mod_install().is_some());
    });
}

#[gpui::test]
fn every_tab_draws_in_both_themes_down_to_the_smallest_window(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    library(&sandbox);
    sandbox.add_requirements("gold_sword", &["missing_core"], &["missing_plugin"]);
    sandbox.add_cutscene_mod("Ending E Remaster", 2, 3840, 2160, CODEC_H264);
    sandbox.add_cutscene_mod("Old Intro", 1, 1280, 720, CODEC_MPEG2);
    sandbox.add_bundled_cutscene("kimono", "Kimono Intro");
    sandbox.install_reshade();
    sandbox.install_migoto("NieRAutomata.exe");
    sandbox.add_game_mod("FreeCamTool", "freecam.dll", 4096);
    sandbox.add_graphic_mod("ColorWrapper", "d3d11.dll", 4096);
    boot(&sandbox, cx);
    let sizes = [(1280., 720.), (1000., 680.), (780., 560.), (600., 420.)];
    for theme in [ThemeId::Dark, ThemeId::Nier] {
        cx.update(|cx| yp_ui::theme::set_theme(theme, cx));

        let (mods, cx) = open_tab::<ModsView>(TabId::Mods, cx);
        for id in ["bunker_free_cam", "wax_alice", "kimono", "gold_sword"] {
            mods.update(cx, |view, cx| view.select(Some(id.to_owned()), cx));
            for (width, height) in sizes {
                cx.simulate_resize(size(px(width), px(height)));
                settle(cx);
            }
        }
        for kind in TutorialKind::ALL {
            mods.update(cx, |view, cx| view.start_tutorial(kind, cx));
            settle(cx);
        }
        mods.update(cx, |view, cx| view.close_tutorial(cx));
        settle(cx);

        let (_, cx) = open_tab::<CutscenesView>(TabId::Cutscenes, cx);
        for (width, height) in sizes {
            cx.simulate_resize(size(px(width), px(height)));
            settle(cx);
        }

        let (thirdparty, cx) = open_tab::<ThirdPartyView>(TabId::ThirdParty, cx);
        thirdparty.update(cx, |view, cx| {
            view.toggle_expanded("FreeCamTool/freecam.dll", cx)
        });
        for (width, height) in sizes {
            cx.simulate_resize(size(px(width), px(height)));
            settle(cx);
        }
    }
}

#[gpui::test]
fn the_mod_list_is_grouped_and_a_filter_flattens_it(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    library(&sandbox);
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    let grouped = rows(&view, cx);
    assert_eq!(header_of(&grouped, ModGroupKind::Native), Some((2, false)));
    assert_eq!(header_of(&grouped, ModGroupKind::Wax), Some((1, false)));
    assert_eq!(
        header_of(&grouped, ModGroupKind::Outfit2b),
        Some((1, false))
    );
    assert_eq!(header_of(&grouped, ModGroupKind::Weapon), Some((1, false)));
    assert!(matches!(
        grouped.first(),
        Some(ListRow::Header {
            kind: ModGroupKind::Native,
            ..
        })
    ));
    for id in [
        "bunker_free_cam",
        "grain_removal",
        "kimono",
        "wax_alice",
        "gold_sword",
    ] {
        assert!(row_ids(&grouped).contains(&id), "{id}");
    }

    view.update_in(cx, |view, window, cx| view.set_filter("BUNKER", window, cx));
    settle(cx);
    assert_eq!(
        rows(&view, cx),
        vec![ListRow::Mod {
            id: "bunker_free_cam".to_owned(),
            group: None
        }]
    );

    stores.mod_names.update(cx, |names, cx| {
        names.rename("kimono", Some("Festival Dress"), cx);
    });
    view.update_in(cx, |view, window, cx| {
        view.set_filter("festival", window, cx)
    });
    settle(cx);
    assert_eq!(row_ids(&rows(&view, cx)), vec!["kimono"]);

    view.update_in(cx, |view, window, cx| view.set_filter("", window, cx));
    view.update(cx, |view, cx| view.toggle_group(ModGroupKind::Native, cx));
    settle(cx);
    let collapsed = rows(&view, cx);
    assert_eq!(header_of(&collapsed, ModGroupKind::Native), Some((2, true)));
    assert!(!row_ids(&collapsed).contains(&"bunker_free_cam"));
    assert!(row_ids(&collapsed).contains(&"kimono"));
    assert!(view.read_with(cx, |view, _| view.is_group_collapsed(ModGroupKind::Native)));
}

#[gpui::test]
fn selection_follows_navigation_requests_and_drops_removed_mods(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    library(&sandbox);
    let stores = boot(&sandbox, cx);
    stores.navigation.update(cx, |navigation, cx| {
        navigation.request_tab(TabId::Mods, Some("kimono".to_owned()), cx);
    });
    let (view, cx) = open_tab::<ModsView>(TabId::Mods, cx);
    assert_eq!(selected(&view, cx).as_deref(), Some("kimono"));

    stores.navigation.update(cx, |navigation, cx| {
        navigation.request_tab(TabId::Mods, Some("late_arrival".to_owned()), cx);
    });
    settle(cx);
    assert_eq!(selected(&view, cx).as_deref(), Some("kimono"));
    sandbox.add_data_file("late_arrival", "wp", "wp0100.dat", 16);
    let reloading = stores.mods.update(cx, |mods, cx| mods.reload(cx));
    settle(cx);
    drop(reloading);
    assert_eq!(selected(&view, cx).as_deref(), Some("late_arrival"));

    std::fs::remove_dir_all(sandbox.mod_dir("late_arrival")).unwrap();
    let reloading = stores.mods.update(cx, |mods, cx| mods.reload(cx));
    settle(cx);
    drop(reloading);
    assert_eq!(selected(&view, cx), None);

    view.update(cx, |view, cx| {
        view.select(Some("gold_sword".to_owned()), cx)
    });
    let removing = stores
        .mods
        .update(cx, |mods, cx| mods.uninstall("gold_sword", false, cx));
    settle(cx);
    drop(removing);
    assert_eq!(selected(&view, cx), None);
    assert!(!mod_ids(&stores, cx).contains(&"gold_sword".to_owned()));
}

#[gpui::test]
fn rows_disable_rename_and_uninstall_mods(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    library(&sandbox);
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    view.update(cx, |view, cx| view.set_mod_disabled("kimono", true, cx));
    settle(cx);
    assert!(
        stores
            .disabled_mods
            .read_with(cx, |disabled, _| disabled.is_disabled("mods/kimono"))
    );
    view.update(cx, |view, cx| view.set_mod_disabled("kimono", false, cx));
    settle(cx);
    assert!(
        !stores
            .disabled_mods
            .read_with(cx, |disabled, _| disabled.is_disabled("mods/kimono"))
    );

    view.update_in(cx, |view, window, cx| {
        view.rename_mod("kimono".to_owned(), window, cx)
    });
    cx.run_until_parked();
    assert!(has_modal(cx));
    cx.simulate_input("Festival Dress!");
    cx.simulate_keystrokes("enter");
    assert!(has_modal(cx));
    cx.simulate_keystrokes("backspace enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert_eq!(
        stores
            .mod_names
            .read_with(cx, |names, _| names
                .custom_name_of("kimono")
                .map(str::to_owned))
            .as_deref(),
        Some("Festival Dress")
    );

    view.update(cx, |view, cx| view.select(Some("kimono".to_owned()), cx));
    view.update_in(cx, |view, window, cx| {
        view.uninstall_mod("kimono".to_owned(), window, cx)
    });
    cx.run_until_parked();
    assert!(has_modal(cx));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(mod_ids(&stores, cx).contains(&"kimono".to_owned()));

    view.update_in(cx, |view, window, cx| {
        view.uninstall_mod("kimono".to_owned(), window, cx)
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!mod_ids(&stores, cx).contains(&"kimono".to_owned()));
    assert!(!sandbox.mod_dir("kimono").exists());
    assert_eq!(selected(&view, cx), None);
    assert!(was_told(
        &stores,
        Msg::ModUninstalled {
            id: "kimono".to_owned()
        },
        Severity::Info,
        cx
    ));
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
fn camera_sliders_save_into_the_mod_after_a_short_pause(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    library(&sandbox);
    boot(&sandbox, cx);
    let (view, cx) = open_tab::<ModsView>(TabId::Mods, cx);
    view.update(cx, |view, cx| {
        view.select(Some("bunker_free_cam".to_owned()), cx)
    });
    settle(cx);
    let loaded = view.read_with(cx, |view, _| {
        view.camera
            .as_ref()
            .map(|editor| editor.settings.get(CameraField::SideOffset))
    });
    assert_eq!(loaded, Some(0.5));

    view.update(cx, |view, cx| {
        view.set_camera_value(CameraField::SideOffset, 0.749, cx)
    });
    cx.executor().advance_clock(Duration::from_millis(200));
    cx.run_until_parked();
    let saved = std::fs::read_to_string(
        sandbox
            .mod_dir("bunker_free_cam")
            .join("entities")
            .join("camera.toml"),
    )
    .unwrap();
    assert!(saved.contains("side_offset = 0.75"), "{saved}");

    view.update(cx, |view, cx| view.select(Some("kimono".to_owned()), cx));
    settle(cx);
    assert!(view.read_with(cx, |view, _| view.camera.is_none()));
}

#[gpui::test]
async fn a_native_drop_with_a_manifest_id_installs_without_asking(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<ModsView>(TabId::Mods, cx);
    let source = sandbox.write_native_drop("Bunker Thing", "bunker_thing");

    let report = start_install(guided(vec![source]), cx).await;
    settle(cx);
    assert_eq!(report.installed, vec!["bunker_thing".to_owned()]);
    assert!(report.failed.is_empty());
    assert!(!report.cancelled);
    assert!(!has_modal(cx));
    assert!(sandbox.mod_dir("bunker_thing").join("mod.toml").is_file());
    assert!(was_told(
        &stores,
        Msg::ModInstalled {
            id: "bunker_thing".to_owned()
        },
        Severity::Success,
        cx
    ));
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));

    let source = sandbox.write_native_drop("Second Thing", "second_thing");
    view.update_in(cx, |view, window, cx| {
        view.install_paths(vec![source], ModInstallMode::Guided, window, cx)
    });
    settle(cx);
    assert_eq!(selected(&view, cx).as_deref(), Some("second_thing"));
}

#[gpui::test]
async fn drops_that_are_not_mods_are_refused_with_their_reason(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    let unknown = sandbox.write_unknown_drop("notes");
    let report = start_install(guided(vec![unknown.clone()]), cx).await;
    settle(cx);
    assert!(report.installed.is_empty());
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].0, unknown);
    assert!(was_told(&stores, Msg::ModDropNotAMod, Severity::Error, cx));

    let cutscenes = sandbox.write_cutscene_drop("HD Cutscenes", 2);
    let report = start_install(guided(vec![cutscenes]), cx).await;
    settle(cx);
    assert!(report.installed.is_empty());
    assert_eq!(report.failed.len(), 1);
    assert!(was_told(
        &stores,
        Msg::ModDropMisroutedCutscenes,
        Severity::Error,
        cx
    ));

    let textures = sandbox.write_texture_drop("only_textures");
    let report = start_install(guided(vec![textures]), cx).await;
    settle(cx);
    assert!(report.installed.is_empty());
    assert_eq!(report.failed.len(), 1);
    assert!(was_told(
        &stores,
        Msg::ModInstallReasonTextureOnly,
        Severity::Warning,
        cx
    ));

    assert!(!has_modal(cx));
    assert!(mod_ids(&stores, cx).is_empty());
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
async fn a_data_drop_asks_for_a_name_then_shows_the_outfit_guide_once(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    let source = sandbox.write_outfit_drop("kimono_wax");
    let installing = start_install(guided(vec![source]), cx);
    assert!(has_modal(cx));
    assert!(mod_ids(&stores, cx).is_empty());
    cx.simulate_input("Kimono");
    cx.simulate_keystrokes("enter");
    let report = installing.await;
    settle(cx);
    assert_eq!(report.installed.len(), 1);
    let first = report.installed[0].clone();
    assert!(sandbox.mod_dir(&first).join("data").join("pl").is_dir());
    assert!(has_modal(cx));
    assert!(cx.update(|_, cx| cx.prefs().get(&OUTFIT_GUIDE_SHOWN)));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(!has_modal(cx));

    let source = sandbox.write_outfit_drop("kimono_wax");
    let installing = start_install(guided(vec![source]), cx);
    assert!(has_modal(cx));
    cx.simulate_input("Kimono");
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(has_modal(cx));
    assert_eq!(mod_ids(&stores, cx), vec![first.clone()]);
    cx.simulate_input("Kimono Two");
    cx.simulate_keystrokes("enter");
    let report = installing.await;
    settle(cx);
    assert_eq!(report.installed.len(), 1);
    assert_ne!(report.installed[0], first);
    assert_eq!(mod_ids(&stores, cx).len(), 2);
    assert!(!has_modal(cx));

    let source = sandbox.write_outfit_drop("kimono_wax");
    let installing = start_install(guided(vec![source.clone()]), cx);
    assert!(has_modal(cx));
    cx.simulate_keystrokes("escape");
    let report = installing.await;
    settle(cx);
    assert!(report.cancelled);
    assert!(report.installed.is_empty());
    assert!(source.is_dir());
    assert_eq!(mod_ids(&stores, cx).len(), 2);
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
async fn a_pack_with_variants_asks_which_ones_and_installs_each(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    cx.update(|cx| cx.prefs().set(&OUTFIT_GUIDE_SHOWN, &true));
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    let source = sandbox.write_variant_drop("dress_pack", &["Red", "Blue"]);
    let installing = start_install(guided(vec![source]), cx);
    assert!(has_modal(cx));
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert!(has_modal(cx));
    assert!(mod_ids(&stores, cx).is_empty());
    cx.simulate_input("Dress");
    cx.simulate_keystrokes("enter");
    let report = installing.await;
    settle(cx);
    assert_eq!(report.installed.len(), 2, "{report:?}");
    assert!(report.failed.is_empty());
    assert_eq!(mod_ids(&stores, cx).len(), 2);
    assert!(was_told(
        &stores,
        Msg::ModVariantInstalledToast { count: 2 },
        Severity::Success,
        cx
    ));
    assert!(!has_modal(cx));

    let source = sandbox.write_variant_drop("dress_pack", &["Red", "Blue"]);
    let installing = start_install(guided(vec![source]), cx);
    assert!(has_modal(cx));
    cx.simulate_keystrokes("escape");
    let report = installing.await;
    assert!(report.cancelled);
    assert_eq!(mod_ids(&stores, cx).len(), 2);
}

#[gpui::test]
async fn bulk_and_loose_installs_report_empty_folders_and_install_real_ones(
    cx: &mut TestAppContext,
) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    cx.update(|cx| cx.prefs().set(&OUTFIT_GUIDE_SHOWN, &true));
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    let no_archives = sandbox.write_unknown_drop("downloads");
    let report = start_install(
        ModInstallRequest {
            sources: vec![no_archives],
            mode: ModInstallMode::Bulk,
            texture_packs: false,
        },
        cx,
    )
    .await;
    settle(cx);
    assert!(report.installed.is_empty());
    assert!(was_told(
        &stores,
        Msg::ModBulkInstallNone,
        Severity::Warning,
        cx
    ));

    let empty = sandbox.drop_dir("empty_folder");
    std::fs::create_dir_all(&empty).unwrap();
    let installing = cx.update(|window, cx| install::run_loose_folder(empty, window, cx));
    cx.run_until_parked();
    let report = installing.await;
    settle(cx);
    assert!(report.installed.is_empty());
    assert!(was_told(
        &stores,
        Msg::ModLooseInstallNone,
        Severity::Warning,
        cx
    ));

    let loose = sandbox.drop_dir("my_outfit");
    std::fs::create_dir_all(&loose).unwrap();
    std::fs::write(loose.join("pl000d.dat"), [0u8; 32]).unwrap();
    std::fs::write(loose.join("pl000d.dtt"), [0u8; 32]).unwrap();
    let installing = cx.update(|window, cx| install::run_loose_folder(loose, window, cx));
    cx.run_until_parked();
    let report = installing.await;
    settle(cx);
    assert_eq!(report.installed.len(), 1, "{report:?}");
    let id = report.installed[0].clone();
    assert!(was_told(
        &stores,
        Msg::ModLooseInstallDone {
            count: 2,
            id: id.clone()
        },
        Severity::Success,
        cx
    ));
    assert!(sandbox.mod_dir(&id).is_dir());
    assert!(!has_modal(cx));
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
async fn loose_files_dropped_together_become_one_named_mod(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    cx.update(|cx| cx.prefs().set(&OUTFIT_GUIDE_SHOWN, &true));
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    let folder = sandbox.drop_dir("desktop");
    std::fs::create_dir_all(&folder).unwrap();
    let dat = folder.join("pl000d.dat");
    let dtt = folder.join("pl000d.dtt");
    std::fs::write(&dat, [0u8; 32]).unwrap();
    std::fs::write(&dtt, [0u8; 32]).unwrap();

    let installing = start_install(guided(vec![dat, dtt]), cx);
    assert!(has_modal(cx));
    cx.simulate_input("Loose Outfit");
    cx.simulate_keystrokes("enter");
    let report = installing.await;
    settle(cx);
    assert_eq!(report.installed.len(), 1, "{report:?}");
    assert_eq!(mod_ids(&stores, cx), report.installed);
}

#[gpui::test]
async fn installing_without_a_game_folder_does_nothing(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot_without_game(&sandbox, cx);
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);
    let source = sandbox.write_native_drop("Bunker Thing", "bunker_thing");
    let report = start_install(guided(vec![source.clone()]), cx).await;
    assert!(report.cancelled);
    assert!(report.installed.is_empty());
    assert!(source.is_dir());
    assert!(pushed(&stores, cx).is_empty());
    assert!(!has_modal(cx));
}

fn profile_names(stores: &Stores, cx: &mut VisualTestContext) -> Vec<String> {
    stores.profiles.read_with(cx, |profiles, _| {
        profiles
            .profiles()
            .iter()
            .map(|profile| profile.name.clone())
            .collect()
    })
}

fn active_profile(stores: &Stores, cx: &mut VisualTestContext) -> String {
    stores
        .profiles
        .read_with(cx, |profiles, _| profiles.active_name().to_owned())
}

#[gpui::test]
fn profile_dialogs_refuse_bad_names_and_apply_good_ones(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    library(&sandbox);
    let stores = boot(&sandbox, cx);
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);
    let original = active_profile(&stores, cx);
    assert_eq!(profile_names(&stores, cx), vec![original.clone()]);

    cx.update(delete_profile);
    cx.run_until_parked();
    assert!(!has_modal(cx));

    cx.update(create_profile);
    cx.run_until_parked();
    assert!(has_modal(cx));
    cx.simulate_keystrokes("enter");
    assert!(has_modal(cx));
    cx.simulate_input("run!");
    cx.simulate_keystrokes("enter");
    assert!(has_modal(cx));
    cx.simulate_keystrokes("backspace enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert!(profile_names(&stores, cx).contains(&"run".to_owned()));
    assert!(was_told(
        &stores,
        Msg::ModProfileCreatedToast {
            name: "run".to_owned()
        },
        Severity::Success,
        cx
    ));

    cx.update(create_profile);
    cx.run_until_parked();
    cx.simulate_input("run");
    cx.simulate_keystrokes("enter");
    assert!(has_modal(cx));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(!has_modal(cx));
    assert_eq!(profile_names(&stores, cx).len(), 2);

    assert_eq!(active_profile(&stores, cx), "run");
    cx.update(|window, cx| switch_profile(original.clone(), window, cx));
    settle(cx);
    assert_eq!(active_profile(&stores, cx), original);
    assert!(was_told(
        &stores,
        Msg::ModProfileSwitchedToast {
            name: original.clone()
        },
        Severity::Success,
        cx
    ));
    cx.update(|window, cx| switch_profile("run".to_owned(), window, cx));
    settle(cx);
    assert_eq!(active_profile(&stores, cx), "run");

    cx.update(rename_profile);
    cx.run_until_parked();
    assert!(has_modal(cx));
    cx.simulate_input(&original);
    cx.simulate_keystrokes("enter");
    assert!(has_modal(cx));
    for _ in 0..original.chars().count() {
        cx.simulate_keystrokes("backspace");
    }
    cx.simulate_input("speedrun");
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert_eq!(active_profile(&stores, cx), "speedrun");
    let names = profile_names(&stores, cx);
    assert!(names.contains(&"speedrun".to_owned()));
    assert!(!names.contains(&"run".to_owned()));

    cx.update(delete_profile);
    cx.run_until_parked();
    assert!(has_modal(cx));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(!has_modal(cx));
    assert_eq!(profile_names(&stores, cx).len(), 2);
}

fn presence(
    stores: &Stores,
    which: ThirdPartyRuntime,
    cx: &mut VisualTestContext,
) -> RuntimePresence {
    stores
        .thirdparty
        .read_with(cx, |store, _| store.presence(which))
}

fn game_mod_sizes(stores: &Stores, cx: &mut VisualTestContext) -> Vec<(String, Option<u64>)> {
    stores.thirdparty.read_with(cx, |store, _| {
        store
            .game_mods()
            .game_mods_info
            .as_ref()
            .map(|info| {
                info.mods
                    .iter()
                    .map(|entry| (entry.file_name.clone(), entry.size_bytes))
                    .collect()
            })
            .unwrap_or_default()
    })
}

#[gpui::test]
fn third_party_cards_follow_what_is_on_disk(cx: &mut TestAppContext) {
    let empty = Sandbox::new();
    let stores = boot(&empty, cx);
    let (_view, cx) = open_tab::<ThirdPartyView>(TabId::ThirdParty, cx);
    for which in [ThirdPartyRuntime::ReShade, ThirdPartyRuntime::Migoto] {
        assert_eq!(presence(&stores, which, cx), RuntimePresence::NotInstalled);
    }
    assert!(game_mod_sizes(&stores, cx).is_empty());

    let installed = Sandbox::new();
    installed.install_reshade();
    installed.install_migoto("NAMS.exe");
    installed.add_game_mod("FreeCamTool", "freecam.dll", 4096);
    installed.add_graphic_mod("ColorWrapper", "d3d11.dll", 2048);
    let dir = installed.game.root().to_path_buf();
    stores
        .session
        .update(cx, |session, cx| session.set_directory(dir, cx));
    settle(cx);
    for which in [ThirdPartyRuntime::ReShade, ThirdPartyRuntime::Migoto] {
        assert_eq!(presence(&stores, which, cx), RuntimePresence::Installed);
    }
    stores.thirdparty.read_with(cx, |store, _| {
        assert_eq!(store.reshade().preset_count, 2);
        assert!(store.migoto().has_shader_fixes);
        assert!(store.graphic_mods().installed);
    });
    assert_eq!(
        game_mod_sizes(&stores, cx),
        vec![("freecam.dll".to_owned(), Some(4096))]
    );
}

#[gpui::test]
fn reshade_left_in_the_game_folder_can_be_imported(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    sandbox.add_game_root_reshade();
    sandbox.write_nams_toml("disable_reshade_loading = true\n");
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<ThirdPartyView>(TabId::ThirdParty, cx);
    assert_eq!(
        presence(&stores, ThirdPartyRuntime::ReShade, cx),
        RuntimePresence::FoundInGameRoot
    );

    view.update(cx, |view, cx| {
        view.import_from_game_root(ThirdPartyRuntime::ReShade, cx)
    });
    settle(cx);
    assert_eq!(
        presence(&stores, ThirdPartyRuntime::ReShade, cx),
        RuntimePresence::Installed
    );
    assert!(
        sandbox
            .thirdparty_dir()
            .join("reshade")
            .join("ReShade")
            .join("ReShade64.dll")
            .is_file()
    );
    assert!(was_told(
        &stores,
        Msg::ThirdPartyImported,
        Severity::Success,
        cx
    ));
}

#[gpui::test]
fn a_dropped_dll_mod_installs_and_a_different_build_asks_before_replacing(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<ThirdPartyView>(TabId::ThirdParty, cx);
    let installs = |stores: &Stores, cx: &mut VisualTestContext| {
        pushed(stores, cx)
            .iter()
            .filter(|told| **told == (Msg::ThirdPartyInstalled, Severity::Success))
            .count()
    };

    let source = sandbox.write_game_mod_drop("SpeedrunTimer", "timer.dll", 2048);
    view.update_in(cx, |view, window, cx| {
        view.install_paths(vec![source], window, cx)
    });
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!view.read_with(cx, |view, _| view.is_installing()));
    assert_eq!(installs(&stores, cx), 1);
    assert_eq!(
        game_mod_sizes(&stores, cx),
        vec![("timer.dll".to_owned(), Some(2048))]
    );

    let newer = sandbox.write_game_mod_drop("SpeedrunTimer", "timer.dll", 4096);
    view.update_in(cx, |view, window, cx| {
        view.install_paths(vec![newer.clone()], window, cx)
    });
    settle(cx);
    assert!(has_modal(cx));
    assert!(view.read_with(cx, |view, _| view.is_installing()));
    assert!(stores.busy.read_with(cx, |busy, _| busy.is_busy()));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!view.read_with(cx, |view, _| view.is_installing()));
    assert!(was_told(
        &stores,
        Msg::ThirdPartyUpdateSkipped,
        Severity::Info,
        cx
    ));
    assert_eq!(
        game_mod_sizes(&stores, cx),
        vec![("timer.dll".to_owned(), Some(2048))]
    );

    let newer = sandbox.write_game_mod_drop("SpeedrunTimer", "timer.dll", 4096);
    view.update_in(cx, |view, window, cx| {
        view.install_paths(vec![newer], window, cx)
    });
    settle(cx);
    assert!(has_modal(cx));
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert_eq!(installs(&stores, cx), 2);
    assert_eq!(
        game_mod_sizes(&stores, cx),
        vec![("timer.dll".to_owned(), Some(4096))]
    );
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
fn third_party_drops_for_other_tabs_are_redirected_or_refused(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<ThirdPartyView>(TabId::ThirdParty, cx);

    let unknown = sandbox.write_unknown_drop("notes");
    let outfit = sandbox.write_outfit_drop("kimono_wax");
    view.update_in(cx, |view, window, cx| {
        view.install_paths(vec![unknown, outfit], window, cx)
    });
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!view.read_with(cx, |view, _| view.is_installing()));
    assert!(was_told(
        &stores,
        Msg::ThirdPartyUnsupported,
        Severity::Warning,
        cx
    ));
    assert!(was_told(
        &stores,
        Msg::ThirdPartyRedirectMods,
        Severity::Accent,
        cx
    ));
    assert!(game_mod_sizes(&stores, cx).is_empty());

    view.update(cx, |view, cx| view.toggle_expanded("a/b.dll", cx));
    assert!(view.read_with(cx, |view, _| view.is_expanded("a/b.dll")));
    view.update(cx, |view, cx| view.toggle_expanded("a/b.dll", cx));
    assert!(!view.read_with(cx, |view, _| view.is_expanded("a/b.dll")));
}

fn cutscene_names(view: &Entity<CutscenesView>, cx: &mut VisualTestContext) -> Vec<String> {
    let model = view.read_with(cx, |view, _| view.model().clone());
    model.read_with(cx, |model, _| {
        model
            .mods()
            .iter()
            .map(|cutscene| cutscene.name.clone())
            .collect()
    })
}

#[gpui::test]
fn the_cutscene_list_holds_standalone_and_bundled_mods(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    sandbox.add_cutscene_mod("Ending E Remaster", 2, 3840, 2160, CODEC_H264);
    sandbox.add_outfit_mod("kimono", "pl000d");
    sandbox.add_bundled_cutscene("kimono", "Kimono Intro");
    boot(&sandbox, cx);
    let (view, cx) = open_tab::<CutscenesView>(TabId::Cutscenes, cx);

    let names = cutscene_names(&view, cx);
    assert!(names.contains(&"Ending E Remaster".to_owned()), "{names:?}");
    assert!(names.contains(&"Kimono Intro".to_owned()), "{names:?}");
    let model = view.read_with(cx, |view, _| view.model().clone());
    model.read_with(cx, |model, _| {
        assert!(model.has_scanned());
        assert!(!model.is_loading());
        assert_eq!(
            merge_targets(model.mods()),
            vec!["Ending E Remaster".to_owned()]
        );
    });
}

#[gpui::test]
fn a_cutscene_drop_is_named_installed_merged_and_deleted(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<CutscenesView>(TabId::Cutscenes, cx);
    let model = view.read_with(cx, |view, _| view.model().clone());
    assert!(cutscene_names(&view, cx).is_empty());

    let source = sandbox.write_cutscene_drop("HD Pack", 3);
    cx.update(|window, cx| install_from_path(model.clone(), source, window, cx))
        .detach();
    settle(cx);
    assert!(has_modal(cx));
    assert!(model.read_with(cx, |model, _| model.is_installing()));
    cx.simulate_input("HD Pack One");
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!model.read_with(cx, |model, _| model.is_installing()));
    let movie = sandbox
        .game
        .cutscenes_dir()
        .join("HD Pack One")
        .join("movie");
    assert!(movie.join("ev0000.usm").is_file());
    assert!(was_told(
        &stores,
        Msg::InstalledCutsceneMod {
            name: "HD Pack One".to_owned(),
            count: 3
        },
        Severity::Success,
        cx
    ));
    assert_eq!(cutscene_names(&view, cx), vec!["HD Pack One".to_owned()]);

    let more = sandbox.write_cutscene_drop("HD Pack Part 2", 5);
    cx.update(|window, cx| install_from_path(model.clone(), more, window, cx))
        .detach();
    settle(cx);
    assert!(has_modal(cx));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!model.read_with(cx, |model, _| model.is_installing()));
    assert_eq!(cutscene_names(&view, cx), vec!["HD Pack One".to_owned()]);

    let cutscene = model.read_with(cx, |model, _| model.mods()[0].clone());
    cx.update(|window, cx| delete_cutscene(model.clone(), cutscene.clone(), window, cx))
        .detach();
    settle(cx);
    assert!(has_modal(cx));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(movie.is_dir());

    cx.update(|window, cx| delete_cutscene(model.clone(), cutscene, window, cx))
        .detach();
    settle(cx);
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!movie.exists());
    assert!(was_told(
        &stores,
        Msg::RemovedItem {
            name: "HD Pack One".to_owned()
        },
        Severity::Info,
        cx
    ));
    assert!(cutscene_names(&view, cx).is_empty());
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
fn a_folder_without_cutscenes_is_refused_on_the_cutscenes_tab(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<CutscenesView>(TabId::Cutscenes, cx);
    let source = sandbox.write_unknown_drop("notes");
    view.update_in(cx, |view, window, cx| view.install(source, window, cx));
    settle(cx);
    assert!(!has_modal(cx));
    let model = view.read_with(cx, |view, _| view.model().clone());
    assert!(!model.read_with(cx, |model, _| model.is_installing()));
    assert!(
        pushed(&stores, cx)
            .iter()
            .any(|(_, severity)| matches!(severity, Severity::Error | Severity::Warning))
    );
    assert!(cutscene_names(&view, cx).is_empty());
}

fn english(cx: &mut VisualTestContext) -> &'static yp_ui::Strings {
    cx.update(|_, cx| cx.t())
}

fn temp_leftovers(sandbox: &Sandbox) -> usize {
    std::fs::read_dir(sandbox.game.scoped_temp_root())
        .map(|entries| entries.count())
        .unwrap_or_default()
}

#[gpui::test]
async fn a_downloads_folder_of_real_archives_installs_in_bulk(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    cx.update(|cx| cx.prefs().set(&OUTFIT_GUIDE_SHOWN, &true));
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);
    for (archive, id) in [
        ("downloads/Bunker Thing.zip", "bunker_thing"),
        ("downloads/Second Thing.7z", "second_thing"),
        ("downloads/335/Third Thing.rar", "third_thing"),
    ] {
        let folder = sandbox.write_native_drop(id, id);
        sandbox.pack_drop(&folder, archive);
    }
    let broken = sandbox.write_drop_file("downloads/Broken.zip", b"this is not an archive");
    sandbox.write_drop_file("downloads/notes.tar.gz", b"not something the launcher unpacks");

    let report = start_install(
        ModInstallRequest {
            sources: vec![sandbox.drop_dir("downloads")],
            mode: ModInstallMode::Bulk,
            texture_packs: false,
        },
        cx,
    )
    .await;
    settle(cx);

    let mut installed = report.installed.clone();
    installed.sort();
    assert_eq!(installed, ["bunker_thing", "second_thing", "third_thing"], "{report:?}");
    for id in &installed {
        assert!(sandbox.mod_dir(id).join("mod.toml").is_file(), "{id}");
        assert!(sandbox.mod_dir(id).join("entities").join("item_one.toml").is_file(), "{id}");
    }
    assert_eq!(
        report.failed,
        vec![(
            broken,
            yp_mods::InstallFailure::Rejected(yp_mods::DropRejection::ArchiveExtractFailed(
                yp_core::ArchiveFailure::Corrupt
            ))
        )]
    );
    let strings = english(cx);
    let told = pushed(&stores, cx);
    assert!(
        told.iter().any(|(msg, _)| *msg
            == Msg::ModBulkInstallDone {
                installed: 3,
                total: 4
            }),
        "{told:?}"
    );
    assert!(was_told(
        &stores,
        Msg::ModInstallFailed {
            reason: strings.mods_ui_label_value("Broken", strings.archive_damaged())
        },
        Severity::Error,
        cx
    ));
    assert_eq!(mod_ids(&stores, cx).len(), 3);
    assert_eq!(temp_leftovers(&sandbox), 0);
    assert!(!has_modal(cx));
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
async fn a_dropped_archive_installs_and_an_unreadable_one_says_why(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    cx.update(|cx| cx.prefs().set(&OUTFIT_GUIDE_SHOWN, &true));
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);
    let strings = english(cx);

    for (archive, id) in [
        ("Bunker Thing.zip", "bunker_thing"),
        ("Second Thing.7z", "second_thing"),
        ("Third Thing.rar", "third_thing"),
    ] {
        let folder = sandbox.write_native_drop(id, id);
        let source = sandbox.pack_drop(&folder, archive);
        let report = start_install(guided(vec![source.clone()]), cx).await;
        settle(cx);
        assert_eq!(report.installed, vec![id.to_owned()], "{archive}");
        assert!(report.failed.is_empty(), "{archive}");
        assert!(sandbox.mod_dir(id).join("mod.toml").is_file(), "{archive}");
        assert!(source.is_file(), "{archive}");
        assert!(was_told(
            &stores,
            Msg::ModInstalled { id: id.to_owned() },
            Severity::Success,
            cx
        ));
    }

    let locked = sandbox.write_locked_archive(
        "Locked.zip",
        &[
            ("mod.toml", b"[mod]\nid = \"locked\"\n"),
            ("entities/item_one.toml", b"[item]\nid = 1\n"),
        ],
    );
    let report = start_install(guided(vec![locked.clone()]), cx).await;
    settle(cx);
    assert!(report.installed.is_empty());
    assert_eq!(
        report.failed,
        vec![(
            locked,
            yp_mods::InstallFailure::Rejected(yp_mods::DropRejection::ArchiveExtractFailed(
                yp_core::ArchiveFailure::PasswordRequired
            ))
        )]
    );
    assert!(was_told(
        &stores,
        Msg::ModInstallFailed {
            reason: strings.archive_password_protected().to_owned()
        },
        Severity::Error,
        cx
    ));

    let broken = sandbox.write_drop_file("Broken.rar", b"this is not an archive");
    let report = start_install(guided(vec![broken.clone()]), cx).await;
    settle(cx);
    assert_eq!(
        report.failed,
        vec![(
            broken,
            yp_mods::InstallFailure::Rejected(yp_mods::DropRejection::ArchiveExtractFailed(
                yp_core::ArchiveFailure::Corrupt
            ))
        )]
    );
    assert!(was_told(&stores, Msg::ArchiveDamaged, Severity::Error, cx));

    let tarball = sandbox.write_drop_file("Pack.tar.gz", b"not something the launcher unpacks");
    let report = start_install(guided(vec![tarball.clone()]), cx).await;
    settle(cx);
    assert_eq!(
        report.failed,
        vec![(
            tarball,
            yp_mods::InstallFailure::Rejected(yp_mods::DropRejection::UnknownDrop)
        )]
    );
    assert!(was_told(&stores, Msg::ModDropNotAMod, Severity::Error, cx));
    assert_eq!(mod_ids(&stores, cx).len(), 3);
    assert_eq!(temp_leftovers(&sandbox), 0);
    assert!(!has_modal(cx));
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
fn a_dropped_third_party_archive_is_unpacked_and_installed(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<ThirdPartyView>(TabId::ThirdParty, cx);

    let folder = sandbox.write_game_mod_drop("SpeedrunTimer", "timer.dll", 2048);
    for archive in ["SpeedrunTimer.zip", "SpeedrunTimer.7z", "SpeedrunTimer.rar"] {
        let source = sandbox.pack_drop(&folder, &format!("packed/{archive}"));
        view.update_in(cx, |view, window, cx| {
            view.install_paths(vec![source], window, cx)
        });
        settle(cx);
        assert!(!has_modal(cx), "{archive}");
        assert!(!view.read_with(cx, |view, _| view.is_installing()), "{archive}");
        assert_eq!(
            game_mod_sizes(&stores, cx),
            vec![("timer.dll".to_owned(), Some(2048))],
            "{archive}"
        );
    }
    assert!(was_told(&stores, Msg::ThirdPartyInstalled, Severity::Success, cx));
    assert_eq!(temp_leftovers(&sandbox), 0);

    let locked = sandbox.write_locked_archive("Locked.zip", &[("Timer/timer.dll", b"MZ")]);
    let broken = sandbox.write_drop_file("Broken.7z", b"this is not an archive");
    let tarball = sandbox.write_drop_file("Shaders.tar.gz", b"not something the launcher unpacks");
    view.update_in(cx, |view, window, cx| {
        view.install_paths(vec![locked, broken, tarball], window, cx)
    });
    settle(cx);
    assert!(!has_modal(cx));
    assert!(was_told(&stores, Msg::ArchivePasswordProtected, Severity::Error, cx));
    assert!(was_told(&stores, Msg::ArchiveDamaged, Severity::Error, cx));
    assert!(was_told(&stores, Msg::ThirdPartyUnsupported, Severity::Warning, cx));
    assert_eq!(
        game_mod_sizes(&stores, cx),
        vec![("timer.dll".to_owned(), Some(2048))]
    );
    assert_eq!(temp_leftovers(&sandbox), 0);
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

fn install_cutscene_archive(archive: &str, cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<CutscenesView>(TabId::Cutscenes, cx);
    let model = view.read_with(cx, |view, _| view.model().clone());
    let folder = sandbox.write_cutscene_drop("HD Pack", 3);
    let source = sandbox.pack_drop(&folder, &format!("packed/{archive}"));

    cx.update(|window, cx| install_from_path(model.clone(), source.clone(), window, cx))
        .detach();
    settle(cx);
    assert!(has_modal(cx), "{:?}", pushed(&stores, cx));
    assert!(model.read_with(cx, |model, _| model.is_installing()));
    cx.simulate_input("From Archive");
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert!(!has_modal(cx));
    assert!(!model.read_with(cx, |model, _| model.is_installing()));
    let movie = sandbox.game.cutscenes_dir().join("From Archive").join("movie");
    for file in ["ev0000.usm", "ev0001.usm", "ev0002.usm"] {
        assert_eq!(
            std::fs::read(movie.join(file)).unwrap(),
            std::fs::read(folder.join("movie").join(file)).unwrap(),
            "{file}"
        );
    }
    assert!(was_told(
        &stores,
        Msg::InstalledCutsceneMod {
            name: "From Archive".to_owned(),
            count: 3
        },
        Severity::Success,
        cx
    ));
    assert_eq!(cutscene_names(&view, cx), vec!["From Archive".to_owned()]);
    assert!(source.is_file());
    assert_eq!(temp_leftovers(&sandbox), 0);
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
fn a_cutscene_zip_is_unpacked_named_and_installed(cx: &mut TestAppContext) {
    install_cutscene_archive("HD Pack.zip", cx);
}

#[gpui::test]
fn a_cutscene_seven_z_is_unpacked_named_and_installed(cx: &mut TestAppContext) {
    install_cutscene_archive("HD Pack.7z", cx);
}

#[gpui::test]
fn a_cutscene_rar_is_unpacked_named_and_installed(cx: &mut TestAppContext) {
    install_cutscene_archive("HD Pack.rar", cx);
}

#[gpui::test]
fn cutscene_archives_that_cannot_be_used_say_why(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (view, cx) = open_tab::<CutscenesView>(TabId::Cutscenes, cx);
    let model = view.read_with(cx, |view, _| view.model().clone());

    let locked = sandbox.write_locked_archive("Locked.zip", &[("movie/ev0000.usm", b"CRID")]);
    let broken = sandbox.write_drop_file("Broken.rar", b"this is not an archive");
    let plain = sandbox.write_unknown_drop("notes");
    let without_usm = sandbox.pack_drop(&plain, "packed/Notes.7z");
    let tarball = sandbox.write_drop_file("Cutscenes.tar.gz", b"not something the launcher unpacks");
    for (source, msg) in [
        (locked, Msg::ArchivePasswordProtected),
        (broken, Msg::ArchiveDamaged),
        (without_usm, Msg::CutsceneDropNoUsm),
        (tarball, Msg::CutsceneDropNoUsm),
    ] {
        let before = pushed(&stores, cx).len();
        cx.update(|window, cx| install_from_path(model.clone(), source, window, cx))
            .detach();
        settle(cx);
        assert!(!has_modal(cx), "{msg:?}");
        assert!(!model.read_with(cx, |model, _| model.is_installing()), "{msg:?}");
        let told = pushed(&stores, cx);
        assert!(told.len() > before, "{msg:?}");
        assert!(told.contains(&(msg.clone(), Severity::Error)), "{msg:?}: {told:?}");
    }
    assert!(cutscene_names(&view, cx).is_empty());
    assert_eq!(temp_leftovers(&sandbox), 0);
    assert!(!stores.busy.read_with(cx, |busy, _| busy.is_busy()));
}

#[gpui::test]
async fn a_texture_only_drop_is_installed_as_a_texture_pack_when_asked(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let handed_over: Rc<RefCell<Vec<PathBuf>>> = Rc::default();
    let seen = handed_over.clone();
    cx.update(|cx| {
        cx.registry_mut()
            .flows
            .set_texture_pack_install(move |sources, _, _| {
                seen.borrow_mut().extend(sources);
                gpui::Task::ready(yp_state::TexturePackOutcome::Installed)
            });
    });
    let (_view, cx) = open_tab::<ModsView>(TabId::Mods, cx);

    let textures = sandbox.write_texture_drop("hd_faces");
    let report = start_install(
        ModInstallRequest::guided(vec![textures.clone()]).with_texture_packs(),
        cx,
    )
    .await;
    settle(cx);
    assert_eq!(report.texture_packs, vec!["hd_faces".to_owned()]);
    assert!(report.installed.is_empty());
    assert!(report.failed.is_empty());
    assert!(report.any_installed());
    assert_eq!(*handed_over.borrow(), vec![textures]);
    assert!(!was_told(
        &stores,
        Msg::ModInstallReasonTextureOnly,
        Severity::Warning,
        cx
    ));
}
