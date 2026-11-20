use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    Entity, Modifiers, TestAppContext, VisualTestContext, div, point, prelude::*, px, size,
};
use yp_config::fields::{bindings, lodmod, nams, overlay, texture_injection};
use yp_config::{AnyField, ConfigFile};
use yp_state::inspector::ArchivePaths;
use yp_state::prelude::*;
use yp_state::{
    NoticeSeverity, OverlayId, OverlayRequest, Registry, Stores, config::NAIOM_AUTOSAVE_DELAY,
};
use yp_ui::{Root, WindowExt};

use crate::fields::{FieldRef, FieldRow, Marker, value_in};
use crate::inspector::model::{InspectorModel, ViewMode, mesh_groups};
use crate::naiom::binding_value;
use crate::nams::CheatTableNotice;
use crate::testing::Sandbox;
use crate::textures::model::{InstallOutcome, TexturesEvent, TexturesModel, pack_rel_path};
use crate::{InspectorPage, LodModTab, NaiomTab, NamsTab, TexturesTab, YorhaProtocolTab};

const SETTLE: Duration = Duration::from_millis(50);

fn boot(sandbox: &Sandbox, cx: &mut TestAppContext) -> Stores {
    let env = sandbox.env.clone();
    let stores = cx.update(|cx| {
        yp_ui::init_for_tests(cx);
        yp_state::init_with(env, cx);
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

fn read_text(path: std::path::PathBuf) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

struct FieldHarness {
    fields: Vec<FieldRef>,
    values: toml::Table,
    seen: Rc<RefCell<Vec<(&'static str, toml::Value)>>>,
}

impl Render for FieldHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self.fields.iter().map(|field| {
            let field: FieldRef = *field;
            let key = field.meta().key;
            let seen = self.seen.clone();
            div()
                .w_full()
                .debug_selector(move || format!("field-{key}"))
                .child(
                    FieldRow::toml("test", field, &self.values)
                        .marker(Marker::Always)
                        .on_change(move |value, _, _| seen.borrow_mut().push((key, value))),
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
fn every_registry_field_renders_and_switches_report_changes(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    boot(&sandbox, cx);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let fields: Vec<FieldRef> = yp_config::fields::all();
    let harness_seen = seen.clone();
    let (_, cx) = cx.add_window_view(|_, _| FieldHarness {
        fields,
        values: toml::Table::new(),
        seen: harness_seen,
    });
    cx.simulate_resize(size(px(900.), px(40000.)));
    settle(cx);
    let key = nams::VALIDATE_SCRIPTS.key();
    let bounds = cx
        .debug_bounds(Box::leak(format!("field-{key}").into_boxed_str()))
        .expect("the switch row is laid out");
    cx.simulate_click(
        point(bounds.origin.x + px(40.), bounds.origin.y + px(16.)),
        Modifiers::none(),
    );
    settle(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        &[(key, toml::Value::Boolean(true))]
    );
    for field in yp_config::fields::all() {
        let selector: &'static str =
            Box::leak(format!("field-{}", field.meta().key).into_boxed_str());
        let bounds = cx.debug_bounds(selector);
        assert!(
            bounds.is_some_and(|bounds| bounds.size.height > px(0.)),
            "{} was not rendered",
            field.meta().key
        );
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
        store.update_lodmod_live("enabled", toml::Value::Boolean(true), None, cx);
        store.update_lodmod_live(
            "enabled",
            toml::Value::Boolean(true),
            Some(lodmod::HIGH_GRIDS_SECTION),
            cx,
        );
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
fn nams_switches_mark_the_page_dirty_until_saved(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NamsTab::new);
    settle(cx);
    assert!(!stores.config.read_with(cx, |store, _| store.is_dirty()));
    stores.config.update(cx, |store, cx| {
        store.update_nams(
            nams::VALIDATE_SCRIPTS.key(),
            toml::Value::Boolean(true),
            None,
            cx,
        )
    });
    settle(cx);
    assert!(stores.config.read_with(cx, |store, _| store.is_dirty()));
    stores.config.update(cx, |store, cx| store.save(cx));
    settle(cx);
    assert!(!stores.config.read_with(cx, |store, _| store.is_dirty()));
    assert!(read_text(sandbox.game.nams_toml()).contains("validate_scripts = true"));

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
fn yorha_keybinds_and_toggles_edit_the_overlay_settings(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(YorhaProtocolTab::new);
    settle(cx);
    tab.update(cx, |tab, cx| {
        tab.set_keybind(&overlay::keybinds::FREE_CAM, "Shift+K", cx)
    });
    settle(cx);
    stores.nams_settings.read_with(cx, |store, _| {
        assert!(store.is_dirty());
        let binds = store.keybinds();
        let value = value_in_json_text(&overlay::keybinds::FREE_CAM, &binds);
        assert_eq!(value, "Shift+K");
    });
    stores.nams_settings.update(cx, |store, cx| store.discard(cx));
    settle(cx);
    stores.nams_settings.read_with(cx, |store, _| {
        assert!(!store.is_dirty());
        let binds = store.keybinds();
        assert_eq!(value_in_json_text(&overlay::keybinds::FREE_CAM, &binds), "F7");
    });
}

fn value_in_json_text(
    field: &dyn AnyField,
    values: &serde_json::Map<String, serde_json::Value>,
) -> String {
    crate::fields::text_of(&crate::fields::value_in_json(field, values))
}

#[gpui::test]
fn lodmod_values_save_on_their_own(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (_, cx) = cx.add_window_view(LodModTab::new);
    settle(cx);
    stores.config.update(cx, |store, cx| {
        store.update_lodmod_live(
            lodmod::AO_FULL_RESOLUTION.key(),
            toml::Value::Boolean(!lodmod::AO_FULL_RESOLUTION.default_value()),
            None,
            cx,
        )
    });
    cx.executor().advance_clock(Duration::from_millis(400));
    cx.run_until_parked();
    let saved = read_text(sandbox.game.lodmod_toml());
    let expected = format!(
        "{} = {}",
        lodmod::AO_FULL_RESOLUTION.key(),
        !lodmod::AO_FULL_RESOLUTION.default_value()
    );
    assert!(saved.contains(&expected), "{saved}");
    assert!(!stores.config.read_with(cx, |store, _| store.is_dirty()));
}

#[gpui::test]
fn naiom_capture_binds_keys_with_modifiers_and_autosaves(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    settle(cx);
    let jump = bindings::STANDARD_JUMP.key();
    tab.update_in(cx, |tab, window, cx| tab.start_capture(jump, window, cx));
    settle(cx);
    assert_eq!(tab.read_with(cx, |tab, _| tab.capturing()), Some(jump));
    cx.simulate_keystrokes("shift-e");
    settle(cx);
    assert_eq!(tab.read_with(cx, |tab, _| tab.capturing()), None);
    let bound = stores
        .config
        .read_with(cx, |store, _| binding_value(&bindings::STANDARD_JUMP, store.nams()));
    assert_eq!(bound, "SHIFT+E");
    assert!(!stores.config.read_with(cx, |store, _| store.is_dirty()));
    cx.executor().advance_clock(NAIOM_AUTOSAVE_DELAY * 2);
    cx.run_until_parked();
    assert!(read_text(sandbox.game.naiom_toml()).contains("standard_jump = \"SHIFT+E\""));

    tab.update_in(cx, |tab, window, cx| tab.start_capture(jump, window, cx));
    settle(cx);
    cx.simulate_keystrokes("escape");
    settle(cx);
    let bound = stores
        .config
        .read_with(cx, |store, _| binding_value(&bindings::STANDARD_JUMP, store.nams()));
    assert_eq!(bound, "");
}

#[gpui::test]
fn naiom_capture_takes_mouse_buttons_only_where_allowed(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    cx.simulate_resize(size(px(1284.), px(6000.)));
    settle(cx);
    let fire = bindings::STANDARD_FIRE.key();
    let forward = bindings::STANDARD_MOVE_FORWARD.key();
    let center = |cx: &mut VisualTestContext, key: &str| {
        let selector: &'static str = Box::leak(format!("naiom-key-{key}").into_boxed_str());
        cx.debug_bounds(selector).expect("binding button").center()
    };

    let fire_button = center(cx, fire);
    cx.simulate_click(fire_button, Modifiers::none());
    settle(cx);
    assert_eq!(tab.read_with(cx, |tab, _| tab.capturing()), Some(fire));
    cx.simulate_click(fire_button, Modifiers::none());
    settle(cx);
    assert_eq!(tab.read_with(cx, |tab, _| tab.capturing()), None);
    let bound = stores
        .config
        .read_with(cx, |store, _| binding_value(&bindings::STANDARD_FIRE, store.nams()));
    assert_eq!(bound, "MOUSE1");

    let forward_button = center(cx, forward);
    cx.simulate_click(forward_button, Modifiers::none());
    settle(cx);
    assert_eq!(tab.read_with(cx, |tab, _| tab.capturing()), Some(forward));
    cx.simulate_click(forward_button, Modifiers::none());
    settle(cx);
    assert_eq!(tab.read_with(cx, |tab, _| tab.capturing()), Some(forward));
    cx.simulate_keystrokes("w");
    settle(cx);
    let bound = stores.config.read_with(cx, |store, _| {
        binding_value(&bindings::STANDARD_MOVE_FORWARD, store.nams())
    });
    assert_eq!(bound, "W");
}

#[gpui::test]
fn naiom_reports_conflicts_and_reset_clears_everything(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    settle(cx);
    tab.update(cx, |tab, cx| {
        tab.set_binding(bindings::STANDARD_JUMP.key(), "SPACE", cx);
        tab.set_binding(bindings::STANDARD_USE.key(), "SPACE", cx);
        tab.capture_virtual_key(0x70, cx);
    });
    settle(cx);
    let strings = cx.update(|_, cx| {
        use yp_ui::ActiveStrings;
        cx.t()
    });
    let values = stores.config.read_with(cx, |store, _| store.nams().clone());
    let all = crate::naiom::binding_values(&values);
    assert_eq!(
        crate::naiom::conflict_label(&bindings::STANDARD_JUMP, &all, strings),
        Some(crate::fields::label_of(&bindings::STANDARD_USE, strings))
    );
    assert!(crate::naiom::has_non_defaults(&values));
    tab.update(cx, |tab, cx| tab.reset_all(cx));
    settle(cx);
    let values = stores.config.read_with(cx, |store, _| store.nams().clone());
    assert!(!crate::naiom::has_non_defaults(&values));
}

#[gpui::test]
fn naiom_cursor_files_are_validated_before_they_are_stored(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    let (tab, cx) = cx.add_window_view(NaiomTab::new);
    settle(cx);
    let field = &nams::MISC_CUSTOM_CURSOR_MENU;
    let fake = sandbox.root.path().join("fake.cur");
    std::fs::write(&fake, b"nope").unwrap();
    tab.update(cx, |tab, cx| tab.apply_cursor(field, fake, cx));
    settle(cx);
    assert!(tab.read_with(cx, |tab, _| tab.is_cursor_invalid(field.key())));
    let real = sandbox.root.path().join("real.cur");
    std::fs::write(&real, [0x00, 0x00, 0x02, 0x00, 0x01]).unwrap();
    tab.update(cx, |tab, cx| tab.apply_cursor(field, real.clone(), cx));
    settle(cx);
    assert!(!tab.read_with(cx, |tab, _| tab.is_cursor_invalid(field.key())));
    let stored = stores.config.read_with(cx, |store, _| {
        crate::fields::text_of(&value_in(field, store.nams()))
    });
    assert_eq!(stored, real.to_string_lossy());
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
    assert!(read_text(sandbox.game.texture_injection_toml()).contains("Gamma"));
    let rows = model.read_with(cx, |model, cx| model.rows(cx));
    assert_eq!(rows[0].conflicts, vec!["Beta"]);
    assert!(rows[2].conflicts.is_empty());
    assert!(rows.iter().all(|row| row.exists && !row.disabled));

    model.update(cx, |model, cx| model.reorder(0, 2, cx));
    cx.run_until_parked();
    let order = model.read_with(cx, |model, cx| model.order(cx));
    assert_eq!(order, vec!["Beta", "Gamma", "Alpha"]);
    let saved: toml::Table = read_text(sandbox.game.texture_injection_toml())
        .parse()
        .unwrap();
    assert_eq!(
        crate::textures::model::string_list(&saved, texture_injection::LOAD_ORDER.key()),
        vec!["Beta", "Gamma", "Alpha"]
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
fn config_parse_errors_reach_the_page(cx: &mut TestAppContext) {
    let sandbox = Sandbox::new();
    let stores = boot(&sandbox, cx);
    std::fs::write(sandbox.game.lodmod_toml(), "broken = = 1\n").unwrap();
    stores.config.update(cx, |store, cx| store.load(cx));
    cx.run_until_parked();
    let (_, cx) = cx.add_window_view(LodModTab::new);
    settle(cx);
    let text = stores.config.read_with(cx, |store, cx| {
        use yp_ui::ActiveStrings;
        store
            .error(ConfigFile::LodMod)
            .map(|error| crate::toml_error_text(error, cx.t()))
    });
    assert!(text.is_some_and(|text| !text.is_empty()));
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
    let saved: toml::Table = read_text(sandbox.game.texture_injection_toml())
        .parse()
        .unwrap();
    saved
        .get("load_order")
        .and_then(toml::Value::as_array)
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
    let file = sandbox.game.texture_injection_toml();
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(
        &file,
        "load_order = [\"Gamma\", \"Alpha\", \"Beta\"]
disabled_packs = []
",
    )
    .unwrap();
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
