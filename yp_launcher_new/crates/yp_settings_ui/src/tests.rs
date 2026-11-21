use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Entity, Modifiers, TestAppContext, VisualTestContext, div, point, prelude::*, px, size,
};
use serde_json::{Value as Json, json};
use yp_config::Setting;
use yp_config::cli::NamsCli;
use yp_state::inspector::ArchivePaths;
use yp_state::prelude::*;
use yp_state::{CliFactory, NoticeSeverity, OverlayId, OverlayRequest, Registry, Stores};
use yp_ui::{ActiveStrings, Root, WindowExt};

use crate::fields::{Control, FieldRow, Marker};
use crate::inspector::model::{InspectorModel, ViewMode, mesh_groups};
use crate::nams::CheatTableNotice;
use crate::testing::Sandbox;
use crate::textures::model::{InstallOutcome, TexturesEvent, TexturesModel, pack_rel_path};
use crate::{InspectorPage, LodModTab, NaiomTab, NamsTab, TexturesTab, YorhaProtocolTab};

const SETTLE: Duration = Duration::from_millis(50);
const JUMP_ID: &str = "naiom.bindings.standard_jump";
const FIRE_ID: &str = "naiom.bindings.standard_fire";
const FORWARD_ID: &str = "naiom.bindings.standard_move_forward";
const CURSOR_ID: &str = "naiom.misc_custom_cursor_menu";
const DEBUG_KEY_ID: &str = "naiom.misc_open_debug_menu";
const PLUGIN_LOADING_ID: &str = "nams.disable_plugin_loading";
const OVERLAY_KEY_ID: &str = "plugins.yorha_protocol.keybinds.main.yorha_protocol";
const LODMOD_ENABLED_ID: &str = "lodmod.enabled";
const LOAD_ORDER_ID: &str = "texture_injection.load_order";

fn boot(sandbox: &Sandbox, cx: &mut TestAppContext) -> Stores {
    let env = sandbox.env.clone();
    let cli = sandbox.cli.clone();
    let stores = cx.update(|cx| {
        yp_ui::init_for_tests(cx);
        yp_state::init_with(env, cx);
        let factory: CliFactory = Arc::new(move |_| cli.clone() as Arc<dyn NamsCli>);
        let config = cx.stores().config.clone();
        config.update(cx, |config, cx| config.set_cli_factory(factory, cx));
        crate::init(cx);
        crate::set_cards_expanded(true, cx);
        cx.stores().clone()
    });
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

fn written(sandbox: &Sandbox, cx: &mut VisualTestContext, id: &str) -> Option<Json> {
    cx.executor().advance_clock(SETTLE * 4);
    cx.run_until_parked();
    sandbox.cli.value(id)
}

struct FieldHarness {
    settings: Vec<Setting>,
    seen: Rc<RefCell<Vec<(String, Json)>>>,
}

impl Render for FieldHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self.settings.iter().map(|setting| {
            let id = setting.id.clone();
            let selector = format!("field-{id}");
            let seen = self.seen.clone();
            div()
                .w_full()
                .debug_selector(move || selector.clone())
                .child(
                    FieldRow::new("test", setting, setting.default.clone())
                        .marker(Marker::Always)
                        .on_change(move |value, _, _| seen.borrow_mut().push((id.clone(), value))),
                )
        });
        div()
            .id("field-harness")
            .size_full()
            .overflow_y_scroll()
            .child(v_flex().w_full().children(rows))
    }
}

use gpui::{Context, IntoElement, Render, Window};
use yp_ui::v_flex;

#[gpui::test]
fn every_schema_setting_renders_and_switches_report_changes(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let settings = stores.config.read_with(cx, |store, _| store.schema().settings.clone());
    assert!(!settings.is_empty());
    let seen = Rc::new(RefCell::new(Vec::new()));
    let harness_seen = seen.clone();
    let (_, cx) = cx.add_window_view(|_, _| FieldHarness {
        settings,
        seen: harness_seen,
    });
    cx.simulate_resize(size(px(900.), px(40000.)));
    settle(cx);
    let bounds = cx
        .debug_bounds("field-nams.disable_plugin_loading")
        .expect("the switch row is laid out");
    cx.simulate_click(
        point(bounds.origin.x + px(40.), bounds.origin.y + px(16.)),
        Modifiers::none(),
    );
    settle(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        &[(PLUGIN_LOADING_ID.to_owned(), json!(true))]
    );
    for selector in [
        "field-lodmod.shadow_blur_scale",
        "field-lodmod.high_grids.room_rings",
        "field-nams.graphics_backend",
        "field-texture_injection.load_order",
    ] {
        let bounds = cx.debug_bounds(selector);
        assert!(bounds.is_some_and(|bounds| bounds.size.height > px(0.)), "{selector}");
    }
}

#[gpui::test]
fn init_registers_tabs_overlay_and_flows(cx: &mut TestAppContext) {

    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    cx.update(|cx| {
        let registry = cx.registry();
        for tab in [
            TabId::Nams,
            TabId::YorhaProtocol,
            TabId::LodMod,
            TabId::Naiom,
            TabId::Textures,
        ] {
            assert!(registry.tabs.contains(tab), "{tab:?}");
        }
        assert!(registry.overlays.contains(OverlayId::Inspector));
        assert!(registry.flows.texture_pack_install().is_some());
        assert!(registry.flows.model_preview().is_some());
    });
}

#[gpui::test]
fn every_tab_draws_expanded_at_wide_and_compact_sizes(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    sandbox.add_texture_pack("Pack A", &["a.dds"]);
    let stores = boot(&sandbox, cx);
    stores.config.update(cx, |store, cx| {
        store.set_now(LODMOD_ENABLED_ID, json!(true), cx);
    });
    for tab in [
        TabId::Nams,
        TabId::YorhaProtocol,
        TabId::LodMod,
        TabId::Naiom,
        TabId::Textures,
    ] {
        let (_, cx) = cx.add_window_view(|window, cx| {
            let view = Registry::build_tab(tab, window, cx).expect("registered");
            Root::new(view, window, cx)
        });
        settle(cx);
        cx.simulate_resize(size(px(764.), px(552.)));
        settle(cx);
        cx.simulate_resize(size(px(1284.), px(742.)));
        settle(cx);
    }
}

#[gpui::test]
fn nams_switches_write_through_nams_and_the_cheat_table_converts(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NamsTab::new);
    settle(cx);
    stores.config.update(cx, |store, cx| store.set(PLUGIN_LOADING_ID, json!(true), cx));
    assert!(stores.config.read_with(cx, |store, _| store.bool(PLUGIN_LOADING_ID)));
    assert_eq!(written(&sandbox, cx, PLUGIN_LOADING_ID), Some(json!(true)));
    assert!(!stores.config.read_with(cx, |store, _| store.has_pending_writes()));

    let table = sandbox.root.path().join("table.CT");
    std::fs::write(
        &table,
        "<CheatTable><Address>NieRAutomata.exe+10</Address></CheatTable>",
    )
    .unwrap();
    tab.update(cx, |tab, cx| tab.convert_cheat_table(table, cx));
    settle(cx);
    tab.read_with(cx, |tab, _| {
        assert_eq!(
            tab.cheat_table_notice(),
            Some(&CheatTableNotice::Converted("table-NAMS.CT".into()))
        );
    });
}

#[gpui::test]
fn yorha_keybinds_are_written_as_plugin_settings(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(YorhaProtocolTab::new);
    settle(cx);
    tab.update(cx, |tab, cx| tab.set_keybind(OVERLAY_KEY_ID, "Shift+K", cx));
    assert_eq!(written(&sandbox, cx, OVERLAY_KEY_ID), Some(json!("Shift+K")));
    assert_eq!(
        stores.config.read_with(cx, |store, _| store.text(OVERLAY_KEY_ID)),
        "Shift+K"
    );
}

#[gpui::test]
fn cheats_are_locked_while_first_playthrough_mode_is_on(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let cheat = "plugins.yorha_protocol.cheatsConfig.infiniteHealth";
    let locked = |stores: &Stores, cx: &mut TestAppContext| {
        stores.config.read_with(cx, |store, _| {
            let setting = store.setting(cheat).cloned().expect("cheat setting");
            crate::extras::lock(&setting, store).is_some()
        })
    };
    assert!(locked(&stores, cx));
    stores.config.update(cx, |store, cx| {
        store.set_now("plugins.yorha_protocol.firstPlaythrough", json!(false), cx)
    });
    cx.run_until_parked();
    assert!(!locked(&stores, cx));
}

#[gpui::test]
fn fixed_values_are_dropdowns_with_translated_labels(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    let options = |id: &str, cx: &mut TestAppContext| {
        cx.update(|cx| {
            let config = cx.stores().config.clone();
            let setting = config.read(cx).setting(id).cloned().expect("setting");
            match crate::schema_page::field("test", &setting, &config, Marker::None, cx).control() {
                Control::Select { options } => options,
                other => panic!("{id} is not a dropdown: {other:?}"),
            }
        })
    };
    let (auto, quality, vanilla) = cx.update(|cx| {
        let strings = cx.t();
        (
            strings.texture_auto_recommended(),
            strings.lod_multiplier_option_quality(),
            strings.lod_multiplier_option_vanilla(),
        )
    });
    let vram = options("texture_injection.vram_budget_mb", cx);
    assert_eq!(vram.len(), 8);
    assert_eq!(vram[0], (json!(0), auto.into()));
    assert_eq!(vram[1], (json!(1024), "1024".into()));
    let lod = options("lodmod.lod_multiplier", cx);
    assert_eq!(lod[0].1.as_ref(), quality);
    assert_eq!(lod[1].1.as_ref(), vanilla);
    let shadows = options("lodmod.shadow_resolution", cx);
    assert_eq!(shadows.len(), 5);
    let experimental = cx.update(|cx| cx.t().graphics_backend_d3d12_experimental());
    let backend = options("nams.graphics_backend", cx);
    assert_eq!(backend[0], (json!("d3d11"), "Direct3D 11".into()));
    assert_eq!(backend[1], (json!("d3d12"), experimental.into()));
}

#[gpui::test]
fn the_graphics_api_dropdown_shows_and_clears_the_in_game_switch(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let backend = "nams.graphics_backend";
    let switch = "lodmod.d3d12_backend";
    stores.config.update(cx, |store, cx| store.set_now(switch, json!(true), cx));
    cx.run_until_parked();
    let shown_backend = stores
        .config
        .read_with(cx, |store, _| crate::extras::shown_value(backend, store));
    assert_eq!(shown_backend, json!("d3d12"));
    let (_, cx) = cx.add_window_view(LodModTab::new);
    settle(cx);
    let sink = cx.update(|_, cx| crate::schema_page::sink(&cx.stores().config.clone(), backend));
    cx.update(|window, cx| sink(json!("d3d11"), window, cx));
    assert_eq!(written(&sandbox, cx, switch), Some(Json::Null));
    assert_eq!(written(&sandbox, cx, backend), Some(json!("d3d11")));
    let shown_backend = stores
        .config
        .read_with(cx, |store, _| crate::extras::shown_value(backend, store));
    assert_eq!(shown_backend, json!("d3d11"));
}

#[gpui::test]
fn the_nams_page_takes_only_dll_files_and_can_go_back_to_nams_exe(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NamsTab::new);
    settle(cx);
    let picked = |stores: &Stores, cx: &mut VisualTestContext| {
        stores
            .launch_options
            .read_with(cx, |options, _| options.nams_dll().map(std::path::Path::to_path_buf))
    };
    tab.update(cx, |tab, cx| tab.use_nams_dll("D:/builds/readme.txt".into(), cx));
    assert!(tab.read_with(cx, |tab, _| tab.dll_rejected()));
    assert_eq!(picked(&stores, cx), None);

    let dll = std::path::PathBuf::from("D:/builds/nams.dll");
    tab.update(cx, |tab, cx| tab.use_nams_dll(dll.clone(), cx));
    settle(cx);
    assert!(!tab.read_with(cx, |tab, _| tab.dll_rejected()));
    assert_eq!(picked(&stores, cx), Some(dll));

    tab.update(cx, |tab, cx| tab.stop_using_nams_dll(cx));
    settle(cx);
    assert_eq!(picked(&stores, cx), None);
}

#[gpui::test]
fn turning_lodmod_off_keeps_the_frame_rate_settings(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let shown = |id: &str, stores: &Stores, cx: &mut TestAppContext| {
        stores.config.read_with(cx, |store, _| {
            let setting = store.setting(id).cloned().expect("lodmod setting");
            crate::extras::is_shown(&setting, store)
        })
    };
    let uncap = "lodmod.fps_uncap_in_gameplay";
    let lod = "lodmod.lod_multiplier";
    assert!(shown(uncap, &stores, cx));
    assert!(!shown(lod, &stores, cx));
    stores.config.update(cx, |store, cx| store.set_now(LODMOD_ENABLED_ID, json!(true), cx));
    cx.run_until_parked();
    assert!(shown(uncap, &stores, cx));
    assert!(shown(lod, &stores, cx));
}

#[gpui::test]
fn direct3d_12_settings_follow_the_backend_not_lodmod(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let shown = |id: &str, stores: &Stores, cx: &mut TestAppContext| {
        stores.config.read_with(cx, |store, _| {
            let setting = store.setting(id).cloned().expect("d3d12 setting");
            crate::extras::is_shown(&setting, store)
        })
    };
    let switch = "lodmod.d3d12_backend";
    let rtao = "lodmod.d3d12.ambient_occlusion";
    let radius = "lodmod.d3d12.ao_radius";
    assert!(!shown(switch, &stores, cx), "the dropdown stands for the in-game switch");
    assert!(!shown(rtao, &stores, cx));
    stores.config.update(cx, |store, cx| store.set_now("nams.graphics_backend", json!("d3d12"), cx));
    cx.run_until_parked();
    assert!(shown(rtao, &stores, cx), "LodMod is off, D3D12 is on");
    assert!(!shown(radius, &stores, cx), "tuning waits for RTAO");
    assert!(!shown("lodmod.d3d12.sun_shadows", &stores, cx));
    assert!(!shown("lodmod.d3d12.cull_casters", &stores, cx));
    stores.config.update(cx, |store, cx| store.set_now(rtao, json!(true), cx));
    cx.run_until_parked();
    assert!(shown(radius, &stores, cx));
    stores.config.update(cx, |store, cx| store.set_now(switch, json!(false), cx));
    cx.run_until_parked();
    assert!(!shown(rtao, &stores, cx), "the lodmod.toml switch wins over nams.toml");
    stores.config.update(cx, |store, cx| store.set_now(switch, json!(true), cx));
    stores.config.update(cx, |store, cx| store.set_now("nams.graphics_backend", json!("d3d11"), cx));
    cx.run_until_parked();
    assert!(shown(rtao, &stores, cx));
}

#[gpui::test]
fn lodmod_values_save_on_their_own(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (_, cx) = cx.add_window_view(LodModTab::new);
    settle(cx);
    stores.config.update(cx, |store, cx| store.set(LODMOD_ENABLED_ID, json!(true), cx));
    assert_eq!(written(&sandbox, cx, LODMOD_ENABLED_ID), Some(json!(true)));
}

#[gpui::test]
fn naiom_capture_binds_keys_with_modifiers(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    settle(cx);
    tab.update_in(cx, |tab, window, cx| tab.start_capture(JUMP_ID.into(), window, cx));
    settle(cx);
    assert_eq!(
        tab.read_with(cx, |tab, _| tab.capturing().map(|id| id.to_string())),
        Some(JUMP_ID.to_owned())
    );
    cx.simulate_keystrokes("shift-e");
    settle(cx);
    assert_eq!(tab.read_with(cx, |tab, _| tab.capturing().cloned()), None);
    assert_eq!(
        stores.config.read_with(cx, |store, _| store.text(JUMP_ID)),
        "SHIFT+E"
    );
    assert_eq!(written(&sandbox, cx, JUMP_ID), Some(json!("SHIFT+E")));

    tab.update_in(cx, |tab, window, cx| tab.start_capture(JUMP_ID.into(), window, cx));
    settle(cx);
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert_eq!(stores.config.read_with(cx, |store, _| store.text(JUMP_ID)), "");
}

#[gpui::test]
fn naiom_capture_takes_mouse_buttons_only_where_the_schema_allows(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    cx.simulate_resize(size(px(1284.), px(6000.)));
    settle(cx);
    let capturing = |tab: &Entity<NaiomTab>, cx: &mut VisualTestContext| {
        tab.read_with(cx, |tab, _| tab.capturing().map(|id| id.to_string()))
    };

    let fire = cx
        .debug_bounds("naiom-key-naiom.bindings.standard_fire")
        .expect("fire button")
        .center();
    cx.simulate_click(fire, Modifiers::none());
    settle(cx);
    assert_eq!(capturing(&tab, cx), Some(FIRE_ID.to_owned()));
    cx.simulate_click(fire, Modifiers::none());
    settle(cx);
    assert_eq!(capturing(&tab, cx), None);
    assert_eq!(stores.config.read_with(cx, |store, _| store.text(FIRE_ID)), "MOUSE1");

    let forward = cx
        .debug_bounds("naiom-key-naiom.bindings.standard_move_forward")
        .expect("forward button")
        .center();
    cx.simulate_click(forward, Modifiers::none());
    settle(cx);
    assert_eq!(capturing(&tab, cx), Some(FORWARD_ID.to_owned()));
    cx.simulate_click(forward, Modifiers::none());
    settle(cx);
    assert_eq!(capturing(&tab, cx), Some(FORWARD_ID.to_owned()));
    cx.simulate_keystrokes("w");
    settle(cx);
    assert_eq!(stores.config.read_with(cx, |store, _| store.text(FORWARD_ID)), "W");
}

#[gpui::test]
fn naiom_reports_conflicts_and_the_debug_key_is_a_virtual_key(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    settle(cx);
    tab.update(cx, |tab, cx| {
        tab.set_binding(JUMP_ID, "SPACE", cx);
        tab.set_binding(FIRE_ID, "SPACE", cx);
    });
    tab.update_in(cx, |tab, window, cx| tab.start_capture(DEBUG_KEY_ID.into(), window, cx));
    tab.update(cx, |tab, cx| tab.capture_virtual_key(DEBUG_KEY_ID, 0x70, cx));
    settle(cx);
    stores.config.read_with(cx, |store, _| {
        let all = crate::naiom::binding_values(store);
        assert_eq!(
            crate::naiom::conflict_label(JUMP_ID, &all, store),
            Some("Fire".into())
        );
        assert_eq!(store.number(DEBUG_KEY_ID), Some(112.0));
    });
}

#[gpui::test]
fn naiom_cursor_files_are_validated_before_they_are_stored(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    settle(cx);
    let fake = sandbox.root.path().join("fake.cur");
    std::fs::write(&fake, b"nope").unwrap();
    tab.update(cx, |tab, cx| tab.apply_cursor(CURSOR_ID.into(), fake, cx));
    settle(cx);
    assert!(tab.read_with(cx, |tab, _| tab.is_cursor_invalid(CURSOR_ID)));
    let real = sandbox.root.path().join("real.cur");
    std::fs::write(&real, [0x00, 0x00, 0x02, 0x00, 0x01]).unwrap();
    tab.update(cx, |tab, cx| tab.apply_cursor(CURSOR_ID.into(), real.clone(), cx));
    settle(cx);
    assert!(!tab.read_with(cx, |tab, _| tab.is_cursor_invalid(CURSOR_ID)));
    assert_eq!(
        stores.config.read_with(cx, |store, _| store.text(CURSOR_ID)),
        real.to_string_lossy()
    );
}

fn textures(cx: &mut TestAppContext) -> Entity<TexturesModel> {
    let model = cx.update(crate::textures::shared_model);
    cx.run_until_parked();
    model
}

#[gpui::test]
fn texture_packs_are_listed_reordered_and_toggled(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    sandbox.add_texture_pack("Alpha", &["shared.dds", "a.dds"]);
    sandbox.add_texture_pack("Beta", &["shared.dds"]);
    sandbox.add_texture_pack("Gamma", &["g.dds"]);
    let stores = boot(&sandbox, cx);
    let model = textures(cx);
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
    let order = model.read_with(cx, |model, cx| model.order(cx));
    assert_eq!(order, vec!["Alpha", "Beta", "Gamma"]);
    assert_eq!(
        sandbox.cli.value(LOAD_ORDER_ID),
        Some(json!(["Alpha", "Beta", "Gamma"]))
    );
    let rows = model.read_with(cx, |model, cx| model.rows(cx));
    assert_eq!(rows[0].conflicts, vec!["Beta"]);
    assert!(rows[2].conflicts.is_empty());
    assert!(rows.iter().all(|row| row.exists && !row.disabled));

    model.update(cx, |model, cx| model.reorder(0, 2, cx));
    cx.run_until_parked();
    let order = model.read_with(cx, |model, cx| model.order(cx));
    assert_eq!(order, vec!["Beta", "Gamma", "Alpha"]);
    assert_eq!(
        sandbox.cli.value(LOAD_ORDER_ID),
        Some(json!(["Beta", "Gamma", "Alpha"]))
    );

    model.update(cx, |model, cx| model.set_pack_enabled("Gamma", false, cx));
    cx.run_until_parked();
    assert!(stores
        .disabled_mods
        .read_with(cx, |store, _| store.is_disabled(&pack_rel_path("Gamma"))));
    let rows = model.read_with(cx, |model, cx| model.rows(cx));
    assert!(rows.iter().find(|row| row.name == "Gamma").unwrap().disabled);
    model.update(cx, |model, cx| model.set_pack_enabled("Gamma", true, cx));
    cx.run_until_parked();
    assert!(!stores
        .disabled_mods
        .read_with(cx, |store, _| store.is_disabled(&pack_rel_path("Gamma"))));
}

#[gpui::test]
fn dropping_a_folder_installs_it_and_reports_the_result(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let model = textures(cx);
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    cx.update(|cx| {
        cx.subscribe(&model, move |_, event: &TexturesEvent, _| {
            seen.borrow_mut().push(event.clone());
        })
        .detach();
    });
    let dropped = sandbox.write_drop_folder("Fresh Pack", &["one.dds", "two.DDS", "readme.txt"]);
    let (_, cx) = cx.add_window_view(TexturesTab::new);
    model.update_in(cx, |model, window, cx| model.install(vec![dropped], window, cx));
    assert!(model.read_with(cx, |model, _| model.is_installing()));
    assert!(stores.busy.read_with(cx, |busy, _| busy.textures()));
    settle(cx);
    assert!(!model.read_with(cx, |model, _| model.is_installing()));
    assert!(!stores.busy.read_with(cx, |busy, _| busy.textures()));
    assert!(events
        .borrow()
        .contains(&TexturesEvent::InstallFinished(InstallOutcome::Installed(2))));
    let installed = sandbox.game.inject_textures_dir().join("Fresh Pack");
    assert!(installed.join("one.dds").is_file());
    assert!(!installed.join("readme.txt").exists());
    let order = model.read_with(cx, |model, cx| model.order(cx));
    assert_eq!(order, vec!["Fresh Pack"]);
    let notices = stores.notifications.read_with(cx, |store, _| {
        store
            .items()
            .iter()
            .map(|notice| notice.severity)
            .collect::<Vec<_>>()
    });
    assert!(notices.contains(&NoticeSeverity::Success));
}

#[gpui::test]
fn a_drop_without_textures_is_refused(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let model = textures(cx);
    let dropped = sandbox.write_drop_folder("Docs", &["readme.txt"]);
    let (_, cx) = cx.add_window_view(TexturesTab::new);
    model.update_in(cx, |model, window, cx| model.install(vec![dropped], window, cx));
    settle(cx);
    assert!(!model.read_with(cx, |model, _| model.is_installing()));
    assert!(model.read_with(cx, |model, cx| model.order(cx)).is_empty());
    assert!(!sandbox.game.inject_textures_dir().join("Docs").exists());
    let errors = stores.notifications.read_with(cx, |store, _| {
        store
            .items()
            .iter()
            .filter(|notice| notice.severity == NoticeSeverity::Error)
            .count()
    });
    assert_eq!(errors, 1);
}

#[gpui::test]
fn deleting_a_pack_removes_the_folder_and_its_load_order_entry(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    sandbox.add_texture_pack("Keep", &["k.dds"]);
    let doomed = sandbox.add_texture_pack("Doomed", &["d.dds"]);
    boot(&sandbox, cx);
    let model = textures(cx);
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
    model.update(cx, |model, cx| model.delete_confirmed("Doomed".into(), cx));
    assert!(model.read_with(cx, |model, _| model.is_deleting()));
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
    assert!(!model.read_with(cx, |model, _| model.is_deleting()));
    assert!(!doomed.exists());
    let order = model.read_with(cx, |model, cx| model.order(cx));
    assert_eq!(order, vec!["Keep"]);
}

#[gpui::test]
fn delete_asks_first_and_cancel_keeps_the_pack(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let pack = sandbox.add_texture_pack("Careful", &["c.dds"]);
    boot(&sandbox, cx);
    let model = textures(cx);
    let (_, cx) = cx.add_window_view(|window, cx| {
        let view = Registry::build_tab(TabId::Textures, window, cx).expect("registered");
        Root::new(view, window, cx)
    });
    settle(cx);
    model.update_in(cx, |model, window, cx| {
        model.delete("Careful".into(), window, cx)
    });
    settle(cx);
    assert!(cx.update(|window, cx| window.has_modal(cx)));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(!cx.update(|window, cx| window.has_modal(cx)));
    assert!(pack.exists());
    assert!(!model.read_with(cx, |model, _| model.is_deleting()));
}

fn open_model(
    archives: &ArchivePaths,
    cx: &mut TestAppContext,
) -> Entity<InspectorModel> {
    let archives = archives.clone();
    let model = cx.update(|cx| {
        cx.new(|cx| InspectorModel::new(archives, ViewMode::Page, None, cx))
    });
    cx.run_until_parked();
    model
}

#[gpui::test]
fn inspector_opens_the_model_and_toggles_mesh_groups(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    let archives = sandbox.write_model("prop", "wp0003", &["Blade", "Grip", "Blade"]);
    let model = open_model(&archives, cx);
    model.update(cx, |model, cx| model.set_viewport(320., 200., cx));
    cx.run_until_parked();
    let session = model
        .read_with(cx, |model, _| model.session().cloned())
        .expect("a session is open");
    session.read_with(cx, |session, _| {
        assert!(session.is_ready());
        assert!(session.latest_frame().is_some());
        assert_eq!(session.viewport(), (320, 200));
    });
    model.read_with(cx, |model, cx| {
        assert_eq!(model.candidates().len(), 1);
        assert!(model.error_text(cx).is_none());
        assert!(model.presets().is_empty());
    });
    let groups = session.read_with(cx, |session, _| {
        mesh_groups(session.meshes(), session.selected_lod())
    });
    let blade = groups.iter().find(|group| group.name == "Blade").unwrap().clone();
    assert!(blade.visible);
    model.update(cx, |model, cx| model.set_group_visibility(&blade.ids, false, cx));
    cx.run_until_parked();
    let groups = session.read_with(cx, |session, _| {
        mesh_groups(session.meshes(), session.selected_lod())
    });
    assert!(!groups.iter().find(|group| group.name == "Blade").unwrap().visible);
    assert!(groups.iter().find(|group| group.name == "Grip").unwrap().visible);
}

#[gpui::test]
fn inspector_infers_player_variants_and_applies_them(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    let archives = sandbox.write_model(
        "outfit",
        "pl000d",
        &["Body", "Hair", "Armor_Body", "DLC_Body", "Eyemask", "Skirt"],
    );
    let model = open_model(&archives, cx);
    let (count, selected) = model.read_with(cx, |model, _| {
        (model.presets().len(), model.selected_index())
    });
    assert!(count >= 2, "vanilla variants are inferred from the mesh names");
    assert_eq!(selected, Some(0));
    let session = model
        .read_with(cx, |model, _| model.session().cloned())
        .unwrap();
    let before: Vec<bool> =
        session.read_with(cx, |session, _| session.meshes().iter().map(|mesh| mesh.visible).collect());
    model.update(cx, |model, cx| model.select_preset(1, cx));
    cx.run_until_parked();
    let after: Vec<bool> =
        session.read_with(cx, |session, _| session.meshes().iter().map(|mesh| mesh.visible).collect());
    assert_ne!(before, after);
    assert_eq!(model.read_with(cx, |model, _| model.selected_index()), Some(1));
}

#[gpui::test]
fn inspector_reports_unreadable_archives(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    let missing = ArchivePaths::new(
        Some(sandbox.root.path().join("nope.dat")),
        Some(sandbox.root.path().join("nope.dtt")),
    );
    let model = open_model(&missing, cx);
    model.read_with(cx, |model, cx| {
        assert!(!model.is_probing());
        assert!(model.session().is_none());
        assert!(model.error_text(cx).is_some_and(|text| !text.is_empty()));
    });
}

#[gpui::test]
fn escape_closes_the_inspector_overlay(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let archives = sandbox.write_model("prop", "wp0003", &["Blade"]);
    let request = OverlayRequest::Inspector {
        archives: archives.clone(),
    };
    stores
        .navigation
        .update(cx, |navigation, cx| navigation.open_overlay(request, cx));
    let (page, cx) =
        cx.add_window_view(|window, cx| InspectorPage::new(archives, window, cx));
    settle(cx);
    assert!(stores
        .navigation
        .read_with(cx, |navigation, _| navigation.is_overlay_open(OverlayId::Inspector)));
    cx.simulate_keystrokes("escape");
    settle(cx);
    assert!(!stores
        .navigation
        .read_with(cx, |navigation, _| navigation.is_overlay_open(OverlayId::Inspector)));
    page.read_with(cx, |page, cx| {
        assert!(page.model().read(cx).error_text(cx).is_none());
    });
}

#[gpui::test]
fn inspector_viewport_follows_the_window_size_without_input(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    let archives = sandbox.write_model("prop", "wp0003", &["Blade", "Grip"]);
    let (page, cx) = cx.add_window_view(|window, cx| InspectorPage::new(archives, window, cx));
    cx.simulate_resize(size(px(900.), px(600.)));
    settle(cx);
    let session = page
        .read_with(cx, |page, cx| page.model().read(cx).session().cloned())
        .expect("a session is open");
    let frame_size = |cx: &mut VisualTestContext| {
        session.read_with(cx, |session, _| {
            let frame = session.latest_frame().expect("a frame was rendered");
            let size = frame.size(0);
            (i32::from(size.width), i32::from(size.height))
        })
    };
    assert_eq!(session.read_with(cx, |session, _| session.viewport()), (900, 600));
    assert_eq!(frame_size(cx), (900, 600));
    cx.simulate_resize(size(px(640.), px(480.)));
    settle(cx);
    assert_eq!(session.read_with(cx, |session, _| session.viewport()), (640, 480));
    assert_eq!(frame_size(cx), (640, 480));
}

#[gpui::test]
fn config_file_errors_reach_the_page(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    sandbox.cli.break_file("lodmod.toml", "expected a value at line 1");
    let stores = boot(&sandbox, cx);
    let (_, cx) = cx.add_window_view(LodModTab::new);
    settle(cx);
    let error = stores
        .config
        .read_with(cx, |store, _| store.file_error("lodmod.toml").map(str::to_owned));
    assert_eq!(error.as_deref(), Some("expected a value at line 1"));
}

fn pack_drop(sandbox: &Sandbox, folder: &std::path::Path, archive: &str) -> std::path::PathBuf {
    let path = sandbox.root.path().join("drops").join(archive);
    yp_core::testing::archives::write_archive_of_dir(folder, &path);
    path
}

fn locked_archive(sandbox: &Sandbox, name: &str, files: &[(&str, &[u8])]) -> std::path::PathBuf {
    let path = sandbox.root.path().join("drops").join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    yp_core::testing::archives::write_encrypted_zip(&path, b"secret", files);
    path
}

fn install_texture_archive(archive: &str, cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let model = textures(cx);
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    cx.update(|cx| {
        cx.subscribe(&model, move |_, event: &TexturesEvent, _| {
            seen.borrow_mut().push(event.clone());
        })
        .detach();
    });
    let folder = sandbox.write_drop_folder("outer/Fresh Pack", &["one.dds", "two.DDS", "readme.txt"]);
    let dropped = pack_drop(&sandbox, folder.parent().unwrap(), archive);
    let (_, cx) = cx.add_window_view(TexturesTab::new);
    model.update_in(cx, |model, window, cx| model.install(vec![dropped.clone()], window, cx));
    assert!(model.read_with(cx, |model, _| model.is_installing()));
    settle(cx);
    assert!(!cx.update(|window, cx| window.has_modal(cx)), "{archive}");
    assert!(!model.read_with(cx, |model, _| model.is_installing()), "{archive}");
    assert!(!stores.busy.read_with(cx, |busy, _| busy.textures()), "{archive}");
    assert!(
        events
            .borrow()
            .contains(&TexturesEvent::InstallFinished(InstallOutcome::Installed(2))),
        "{archive}: {:?}",
        events.borrow()
    );
    let installed = sandbox.game.inject_textures_dir().join("Fresh Pack");
    let mut files = yp_core::fs::files_where(&installed, &mut |_, _| true).unwrap();
    files.sort();
    let names: Vec<String> = files.iter().map(|file| yp_core::fs::file_name_lossy(file)).collect();
    assert_eq!(names, ["one.dds", "two.DDS"], "{archive}");
    for file in &files {
        assert_eq!(std::fs::read(file).unwrap(), Sandbox::dds_bytes(), "{archive}");
    }
    assert_eq!(model.read_with(cx, |model, cx| model.order(cx)), vec!["Fresh Pack"], "{archive}");
    assert!(dropped.is_file(), "{archive}");
    let leftovers = std::fs::read_dir(sandbox.env.host.temp_dir())
        .map(|entries| entries.count())
        .unwrap_or_default();
    assert_eq!(leftovers, 0, "{archive}");
}

#[gpui::test]
fn dropping_a_texture_zip_unpacks_it_and_installs_the_pack(cx: &mut TestAppContext) {
    install_texture_archive("Fresh Pack.zip", cx);
}

#[gpui::test]
fn dropping_a_texture_seven_z_unpacks_it_and_installs_the_pack(cx: &mut TestAppContext) {
    install_texture_archive("Fresh Pack.7z", cx);
}

#[gpui::test]
fn dropping_a_texture_rar_unpacks_it_and_installs_the_pack(cx: &mut TestAppContext) {
    install_texture_archive("Fresh Pack.rar", cx);
}

#[gpui::test]
fn a_locked_texture_archive_says_why_it_was_not_installed(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let model = textures(cx);
    let dds = Sandbox::dds_bytes();
    let locked = locked_archive(&sandbox, "Locked.zip", &[("Locked Pack/one.dds", &dds)]);
    let (_, cx) = cx.add_window_view(TexturesTab::new);
    model.update_in(cx, |model, window, cx| model.install(vec![locked], window, cx));
    settle(cx);
    assert!(!model.read_with(cx, |model, _| model.is_installing()));
    assert!(model.read_with(cx, |model, cx| model.order(cx)).is_empty());
    assert!(!sandbox.game.inject_textures_dir().join("Locked Pack").exists());
    let told = stores.notifications.read_with(cx, |store, _| {
        store
            .items()
            .iter()
            .map(|notice| (notice.msg.clone(), notice.severity))
            .collect::<Vec<_>>()
    });
    assert!(
        told.contains(&(yp_i18n::Msg::ArchivePasswordProtected, NoticeSeverity::Error)),
        "{told:?}"
    );
}

#[gpui::test]
fn a_burst_of_reload_requests_runs_the_texture_list_at_most_twice(cx: &mut TestAppContext) {
    let runner = yp_core::testing::FakeRunner::new();
    let sandbox = Sandbox::with_runner(runner.clone());
    sandbox.add_texture_pack("Alpha", &["a.dds"]);
    let _stores = boot(&sandbox, cx);
    let model = textures(cx);
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
    let listings = |runner: &yp_core::testing::FakeRunner| {
        runner
            .calls()
            .iter()
            .filter(|call| call.args_lossy().iter().any(|arg| arg == "textures"))
            .count()
    };
    let before = listings(&runner);

    model.update(cx, |model, cx| {
        for _ in 0..6 {
            model.reload(cx);
        }
    });
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();

    let burst = listings(&runner) - before;
    assert!((1..=2).contains(&burst), "the texture list ran {burst} times");
    let order = model.read_with(cx, |model, cx| model.order(cx));
    assert_eq!(order, vec!["Alpha"]);
}

fn saved_texture_order(sandbox: &Sandbox) -> Vec<String> {
    sandbox
        .cli
        .value(LOAD_ORDER_ID)
        .and_then(|value| value.as_array().cloned())
        .map(|packs| {
            packs
                .iter()
                .filter_map(|pack| pack.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[gpui::test]
fn a_custom_texture_load_order_survives_startup_and_refreshes(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    sandbox.add_texture_pack("Alpha", &["a.dds"]);
    sandbox.add_texture_pack("Beta", &["b.dds"]);
    sandbox.add_texture_pack("Gamma", &["g.dds"]);
    sandbox.cli.put(LOAD_ORDER_ID, json!(["Gamma", "Alpha", "Beta"]));
    let stores = boot(&sandbox, cx);
    let model = textures(cx);
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
    for _ in 0..3 {
        stores
            .detection
            .update(cx, |detection, cx| detection.invalidate(cx));
        model.update(cx, |model, cx| model.reload(cx));
        cx.executor().advance_clock(SETTLE);
        cx.run_until_parked();
    }
    assert_eq!(saved_texture_order(&sandbox), vec!["Gamma", "Alpha", "Beta"]);
    let order = model.read_with(cx, |model, cx| model.order(cx));
    assert_eq!(order, vec!["Gamma", "Alpha", "Beta"]);

    sandbox.add_texture_pack("Aardvark", &["z.dds"]);
    model.update(cx, |model, cx| model.reload(cx));
    cx.executor().advance_clock(SETTLE);
    cx.run_until_parked();
    assert_eq!(
        saved_texture_order(&sandbox),
        vec!["Gamma", "Alpha", "Beta", "Aardvark"]
    );
}

#[gpui::test]
fn the_embedded_preview_only_shows_for_files_that_hold_a_model(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let _stores = boot(&sandbox, cx);
    let with_model = sandbox.write_model("prop", "wp0003", &["Blade"]);
    let script_dir = sandbox.root.path().join("models").join("room");
    std::fs::create_dir_all(&script_dir).expect("script dir");
    let script = script_dir.join("r110.dat");
    let archive = yp_formats::dat::build(
        &[yp_formats::dat::DatEntry::new("r110_scp.bin", vec![1, 2, 3, 4])],
        yp_formats::dat::DatKind::Dat,
    )
    .expect("script archive");
    std::fs::write(&script, archive).expect("script dat");
    let without_model = ArchivePaths::new(Some(script), None);

    let (shown, cx) =
        cx.add_window_view(|window, cx| crate::inspector::ModelPreview::new(with_model, window, cx));
    settle(cx);
    assert!(shown.read_with(cx, |preview, cx| preview.has_model(cx)));

    let hidden = cx.update(|window, cx| {
        cx.new(|cx| crate::inspector::ModelPreview::new(without_model, window, cx))
    });
    settle(cx);
    assert!(!hidden.read_with(cx, |preview, cx| preview.has_model(cx)));
}

#[gpui::test]
fn the_texture_pack_flow_reports_how_the_install_ended(cx: &mut TestAppContext) {
    use futures::FutureExt;
    use yp_state::TexturePackOutcome;

    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    let dropped = sandbox.write_drop_folder("Flow Pack", &["one.dds", "notes.txt"]);
    let (_, cx) = cx.add_window_view(TexturesTab::new);

    let nothing = cx.update(|window, cx| Registry::install_texture_pack(Vec::new(), window, cx));
    assert_eq!(
        nothing.and_then(|task| task.now_or_never()),
        Some(TexturePackOutcome::Failed)
    );

    let installing = cx
        .update(|window, cx| Registry::install_texture_pack(vec![dropped], window, cx))
        .expect("the flow is registered");
    settle(cx);
    assert_eq!(installing.now_or_never(), Some(TexturePackOutcome::Installed));
    assert!(sandbox
        .game
        .inject_textures_dir()
        .join("Flow Pack")
        .join("one.dds")
        .is_file());
}
