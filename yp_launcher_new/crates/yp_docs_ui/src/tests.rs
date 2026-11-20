use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyView, App, Context, Entity, IntoElement, Render, TestAppContext, VisualTestContext,
    Window, div, prelude::*,
};
use tempfile::TempDir;
use yp_builders::{MeshRule, MeshVisibility, OutfitCharacter, first_mod, item, thumbnail};
use yp_core::testing::FakeRunner;
use yp_core::{AppPaths, GamePaths, HostEnv, Os, SettingsStore};
use yp_host::nams_cli::VerifyStatus;
use yp_state::prelude::*;
use yp_state::{AppEnv, OverlayId, OverlayRequest};
use yp_ui::{ActiveStrings, Root, ThemeId, WindowExt};

use crate::builders::ToolKind;
use crate::builders::first_mod::{FirstModRunner, STEP_COUNT, STEP_FOLDER, STEP_ITEM, STEP_MANIFEST};
use crate::builders::forms::{DEFAULT_ITEM_MOD, DEFAULT_OUTFIT_MOD, ThumbnailChoice};
use crate::builders::item::ItemBuilder;
use crate::builders::outfit::OutfitBuilder;
use crate::builders::tests::{write_model, write_png};
use crate::builders::weapon::WeaponBuilder;
use crate::dialogs::{icon_name, markdown_editor, thumbnail as thumbnail_dialog, toml_editor};
use crate::docs::{DocsConfig, DocsOverlay};
use crate::markdown::tests::docs_dir;
use crate::tools::{ToolCard, ToolsView, adjust_target};

struct Harness {
    root: TempDir,
    game: GamePaths,
    env: AppEnv,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("temp dir");
        let host = HostEnv::for_test(Os::Windows, &[], root.path());
        let paths = AppPaths::for_test(root.path());
        std::fs::create_dir_all(paths.plugins_dir()).expect("plugins");
        std::fs::create_dir_all(&paths.logs_dir).expect("logs");
        std::fs::create_dir_all(host.temp_dir()).expect("temp");
        let game = GamePaths::new(root.path().join("game"));
        std::fs::create_dir_all(game.root()).expect("game");
        std::fs::write(game.exe(), b"exe").expect("exe");
        let env = AppEnv::new(
            host,
            paths,
            Arc::new(SettingsStore::in_memory()),
            Arc::new(FakeRunner::new()),
        )
        .with_launch_grace(Duration::ZERO)
        .with_file_watching(false)
        .with_gpu_registry(false);
        Self { root, game, env }
    }

    fn init(&self, cx: &mut TestAppContext) {
        self.init_with_docs(
            DocsConfig {
                docs_dir: docs_dir(),
                editable: false,
            },
            cx,
        );
    }

    fn init_with_docs(&self, config: DocsConfig, cx: &mut TestAppContext) {
        let env = self.env.clone();
        cx.update(|cx| {
            yp_ui::init_for_tests(cx);
            yp_state::init_with(env, cx);
            cx.set_global(config);
            crate::init(cx);
        });
    }

    fn select_game(&self, cx: &mut TestAppContext) {
        let directory = self.game.root().to_path_buf();
        cx.update(|cx| {
            let session = cx.stores().session.clone();
            session.update(cx, |session, cx| session.set_directory(directory, cx));
        });
        cx.run_until_parked();
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }
}

struct Stage {
    view: Option<AnyView>,
}

impl Render for Stage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().children(self.view.clone())
    }
}

fn open_stage(cx: &mut TestAppContext) -> (Entity<Stage>, &mut VisualTestContext) {
    let mut stage = None;
    let (_root, cx) = cx.add_window_view(|window, cx| {
        let created = cx.new(|_| Stage { view: None });
        stage = Some(created.clone());
        Root::new(created, window, cx)
    });
    (stage.expect("stage"), cx)
}

fn show<V: Render>(
    stage: &Entity<Stage>,
    cx: &mut VisualTestContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
) -> Entity<V> {
    let view = cx.update(|window, cx| {
        let view = cx.new(|cx| build(window, cx));
        let any: AnyView = view.clone().into();
        stage.update(cx, |stage, cx| {
            stage.view = Some(any);
            cx.notify();
        });
        view
    });
    cx.run_until_parked();
    view
}

fn open_docs(stage: &Entity<Stage>, cx: &mut VisualTestContext, page: Option<&str>) -> Entity<DocsOverlay> {
    let request = OverlayRequest::Docs {
        page: page.map(str::to_owned),
    };
    let overlay = cx.update(|window, cx| {
        let view = Registry::build_overlay(&request, window, cx).expect("docs overlay");
        stage.update(cx, |stage, cx| {
            stage.view = Some(view.clone());
            cx.notify();
        });
        view.downcast::<DocsOverlay>().expect("docs view")
    });
    cx.run_until_parked();
    overlay
}

fn toast_count(cx: &mut VisualTestContext) -> usize {
    cx.update(|window, cx: &mut App| {
        window
            .toast_stack(cx)
            .map_or(0, |stack| stack.read(cx).len())
    })
}

fn has_modal(cx: &mut VisualTestContext) -> bool {
    cx.update(|window, cx: &mut App| window.has_modal(cx))
}

#[gpui::test]
fn init_registers_the_tools_tab_and_the_docs_overlay(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    cx.update(|cx| {
        assert!(cx.registry().tabs.contains(TabId::Tools));
        assert!(cx.registry().overlays.contains(OverlayId::Docs));
        assert!(!cx.registry().tabs.contains(TabId::Docs));
        assert!(!cx.global::<DocsConfig>().editable);
    });
}

#[gpui::test]
fn the_overlay_loads_the_index_and_shows_the_home_page(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    let overlay = open_docs(&stage, cx, None);
    let model = overlay.read_with(cx, |overlay, _| overlay.model().clone());
    model.read_with(cx, |model, _| {
        assert!(!model.is_loading());
        assert!(model.pages().len() > 50);
        assert!(model.shows_home());
        assert!(model.current().is_none());
        assert!(!model.can_edit());
        assert_eq!(model.load_error(), None);
    });
    let [nav, _, _, home] = overlay.read_with(cx, |overlay, _| overlay.panel_heights());
    assert!(f32::from(nav) > 200., "the nav panel is {nav:?} tall");
    assert!(f32::from(home) > 200., "the home page is {home:?} tall");
    model.update(cx, |model, cx| model.set_category("scripting", cx));
    cx.run_until_parked();
    assert_eq!(model.read_with(cx, |model, _| model.active_category().to_owned()), "scripting");
}

#[gpui::test]
fn pages_open_with_their_tool_panels_history_and_links(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    let overlay = open_docs(&stage, cx, None);
    let model = overlay.read_with(cx, |overlay, _| overlay.model().clone());

    model.update(cx, |model, cx| model.open_file_name("01_first_mod.md", cx));
    cx.run_until_parked();
    model.read_with(cx, |model, _| {
        let open = model.current().expect("page");
        assert_eq!(open.page.file_name, "01_first_mod.md");
        assert!(!model.shows_home());
        assert!(!model.has_history());
        assert_eq!(model.active_category(), "native");
    });
    overlay.read_with(cx, |overlay, cx| {
        let panel = overlay.tool(ToolKind::FirstMod).expect("first mod panel");
        assert!(panel.read(cx).is_open());
        assert!(overlay.tool(ToolKind::ItemBuilder).is_none());
    });

    overlay.update(cx, |overlay, cx| overlay.follow_link("03_item.md", cx));
    cx.run_until_parked();
    model.read_with(cx, |model, _| {
        assert_eq!(model.current().expect("page").page.file_name, "03_item.md");
        assert!(model.has_history());
    });
    overlay.read_with(cx, |overlay, cx| {
        assert!(overlay.tool(ToolKind::FirstMod).is_none());
        let panel = overlay.tool(ToolKind::ItemBuilder).expect("item panel");
        assert!(!panel.read(cx).is_open());
        assert!(panel.read(cx).content().clone().downcast::<ItemBuilder>().is_ok());
    });

    let [nav, content, outline, _] = overlay.read_with(cx, |overlay, _| overlay.panel_heights());
    for (panel, height) in [("nav", nav), ("content", content), ("outline", outline)] {
        assert!(f32::from(height) > 200., "the {panel} panel is {height:?} tall");
    }

    let anchor = model.read_with(cx, |model, _| {
        model.current().expect("page").outline[2].anchor.clone()
    });
    let scrolled = overlay.update(cx, |overlay, cx| overlay.scroll_to_anchor(&anchor, false, cx));
    assert!(scrolled);
    assert!(!overlay.update(cx, |overlay, cx| overlay.scroll_to_anchor("no-such-heading", false, cx)));

    let opened = Rc::new(RefCell::new(Vec::new()));
    let sink = opened.clone();
    cx.update(|_, cx| {
        yp_ui::set_link_opener(move |url, _| sink.borrow_mut().push(url.to_owned()), cx);
    });
    overlay.update(cx, |overlay, cx| overlay.follow_link("https://example.com/nams", cx));
    cx.run_until_parked();
    assert_eq!(*opened.borrow(), vec!["https://example.com/nams".to_owned()]);
    assert_eq!(
        model.read_with(cx, |model, _| model.current().expect("page").page.file_name.clone()),
        "03_item.md"
    );

    assert_eq!(toast_count(cx), 0);
    overlay.update(cx, |overlay, cx| overlay.follow_link("missing_page.md", cx));
    cx.run_until_parked();
    assert_eq!(toast_count(cx), 1);
    assert_eq!(
        model.read_with(cx, |model, _| model.current().expect("page").page.file_name.clone()),
        "03_item.md"
    );

    model.update(cx, |model, cx| {
        assert!(model.go_back(cx));
    });
    cx.run_until_parked();
    model.read_with(cx, |model, _| {
        assert_eq!(model.current().expect("page").page.file_name, "01_first_mod.md");
        assert!(!model.has_history());
    });
    model.update(cx, |model, cx| {
        assert!(!model.go_back(cx));
        model.go_home(cx);
    });
    cx.run_until_parked();
    assert!(model.read_with(cx, |model, _| model.shows_home()));
    assert!(overlay.read_with(cx, |overlay, _| overlay.tool(ToolKind::FirstMod).is_none()));
}

#[gpui::test]
fn a_link_with_an_anchor_opens_the_page_and_scrolls_to_the_heading(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    let overlay = open_docs(&stage, cx, Some("assets/docs/native/01_first_mod.md"));
    let model = overlay.read_with(cx, |overlay, _| overlay.model().clone());
    assert_eq!(
        model.read_with(cx, |model, _| model.current().expect("page").page.file_name.clone()),
        "01_first_mod.md"
    );
    let target = cx.update(|_, cx| {
        let mut library = yp_docs::DocsLibrary::new(cx.global::<DocsConfig>().docs_dir.clone());
        let pages = library.load_index().expect("index").to_vec();
        let page = crate::docs::find_by_file_name(&pages, "04_weapon.md").expect("weapon page").clone();
        let markdown = library.content(&page.path).expect("content");
        yp_docs::outline(&markdown).last().expect("heading").anchor.clone()
    });
    overlay.update(cx, |overlay, cx| {
        overlay.follow_link(&format!("04_weapon.md#{target}"), cx);
    });
    cx.run_until_parked();
    cx.run_until_parked();
    assert_eq!(
        model.read_with(cx, |model, _| model.current().expect("page").page.file_name.clone()),
        "04_weapon.md"
    );
    let offset = overlay.read_with(cx, |overlay, _| overlay.content_scroll().offset().y);
    assert!(f32::from(offset) < 0., "the page did not scroll to the heading");

    let middle = model.read_with(cx, |model, _| {
        let outline = &model.current().expect("page").outline;
        outline[outline.len() / 2].anchor.clone()
    });
    assert!(overlay.update(cx, |overlay, cx| overlay.scroll_to_anchor(&middle, false, cx)));
    cx.run_until_parked();
    let (heading, viewport) = overlay.read_with(cx, |overlay, _| {
        (
            overlay.anchor_bounds(&middle).expect("heading bounds"),
            overlay.content_scroll().bounds(),
        )
    });
    let from_top = f32::from(heading.top() - viewport.top());
    let wanted = f32::from(viewport.size.height) * 0.05;
    assert!(
        (from_top - wanted).abs() < 1.5,
        "the heading sits {from_top}px below the top of the page area, wanted {wanted}px"
    );
}

#[gpui::test]
fn a_page_request_from_another_screen_opens_that_page(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    for (request, file_name) in [
        ("docs/native/04_weapon.md", Some("04_weapon.md")),
        ("assets/docs/native/35_third_party.md", Some("35_third_party.md")),
        ("05_outfit.md", Some("05_outfit.md")),
        ("docs/native/no_such_page.md", None),
    ] {
        let overlay = open_docs(&stage, cx, Some(request));
        overlay.read_with(cx, |overlay, cx| {
            let model = overlay.model().read(cx);
            let opened = model.current().map(|open| open.page.file_name.clone());
            assert_eq!(opened.as_deref(), file_name, "request {request}");
            assert_eq!(model.shows_home(), file_name.is_none());
            assert!(!model.has_history());
        });
    }
}

#[gpui::test]
fn searching_lists_hits_with_excerpts_and_clears_again(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    let overlay = open_docs(&stage, cx, None);
    let model = overlay.read_with(cx, |overlay, _| overlay.model().clone());
    cx.update(|window, cx| {
        overlay.update(cx, |overlay, cx| overlay.set_query("pebble", window, cx));
    });
    cx.run_until_parked();
    model.read_with(cx, |model, _| {
        assert!(model.is_searching());
        assert!(!model.hits().is_empty());
        assert!(model.hits().iter().any(|hit| hit.excerpt.is_some()));
    });
    assert_eq!(overlay.read_with(cx, |overlay, cx| overlay.search().read(cx).value().to_owned()), "pebble");

    cx.update(|window, cx| {
        overlay.update(cx, |overlay, cx| overlay.set_query("zzzz no such text", window, cx));
    });
    cx.run_until_parked();
    assert!(model.read_with(cx, |model, _| model.hits().is_empty() && model.is_searching()));

    let search = overlay.read_with(cx, |overlay, _| overlay.search().clone());
    cx.update(|window, cx| search.update(cx, |search, cx| search.clear(window, cx)));
    cx.run_until_parked();
    assert!(!model.read_with(cx, |model, _| model.is_searching()));
}

#[gpui::test]
fn single_page_mode_and_a_missing_docs_folder_render(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    let single = show(&stage, cx, |window, cx| {
        DocsOverlay::new(
            Some("assets/docs/native/05_outfit.md".to_owned()),
            true,
            window,
            cx,
        )
    });
    cx.run_until_parked();
    single.read_with(cx, |overlay, cx| {
        let model = overlay.model().read(cx);
        assert!(model.is_single_page());
        assert_eq!(model.current().expect("page").page.file_name, "05_outfit.md");
        assert!(overlay.tool(ToolKind::OutfitBuilder).is_some());
    });

    cx.update(|_, cx| {
        cx.set_global(DocsConfig {
            docs_dir: PathBuf::from("Z:/no/such/docs"),
            editable: true,
        });
    });
    let missing = show(&stage, cx, |window, cx| DocsOverlay::new(None, false, window, cx));
    cx.run_until_parked();
    missing.read_with(cx, |overlay, cx| {
        let model = overlay.model().read(cx);
        assert!(!model.is_loading());
        assert!(model.pages().is_empty());
        assert!(!model.can_edit());
    });
}

#[gpui::test]
fn editing_is_offered_only_for_an_editable_library_and_saves_the_page(cx: &mut TestAppContext) {
    let harness = Harness::new();
    let docs = harness.path("docs");
    std::fs::create_dir_all(docs.join("native")).expect("docs folder");
    std::fs::write(docs.join("native").join("01_intro.md"), "# Intro\n\n## One\n\ntext\n").expect("page");
    harness.init_with_docs(
        DocsConfig {
            docs_dir: docs.clone(),
            editable: true,
        },
        cx,
    );
    let (stage, cx) = open_stage(cx);
    let overlay = open_docs(&stage, cx, Some("assets/docs/native/01_intro.md"));
    let model = overlay.read_with(cx, |overlay, _| overlay.model().clone());
    model.read_with(cx, |model, _| {
        assert!(model.can_edit());
        assert!(model.current().expect("page").source_file.is_some());
    });
    model.update(cx, |model, cx| {
        model.save_current("# Renamed\n\n## One\n\n## Two\n\n## Three\n".to_owned(), cx);
    });
    cx.run_until_parked();
    model.read_with(cx, |model, _| {
        let open = model.current().expect("page");
        assert_eq!(open.page.title, "Renamed");
        assert_eq!(open.outline.len(), 3);
        assert_eq!(model.pages()[0].title, "Renamed");
    });
    assert!(std::fs::read_to_string(docs.join("native").join("01_intro.md"))
        .expect("saved")
        .starts_with("# Renamed"));

    std::fs::write(docs.join("native").join("01_intro.md"), "# From disk\n").expect("external edit");
    model.update(cx, |model, cx| model.reload_current(cx));
    cx.run_until_parked();
    assert_eq!(model.read_with(cx, |model, _| model.current().expect("page").page.title.clone()), "From disk");
}

#[gpui::test]
fn every_page_with_a_tool_renders_in_both_themes_with_the_panel_open(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    harness.select_game(cx);
    let (stage, cx) = open_stage(cx);
    let overlay = open_docs(&stage, cx, None);
    let model = overlay.read_with(cx, |overlay, _| overlay.model().clone());
    for (page, kind) in [
        ("01_first_mod.md", ToolKind::FirstMod),
        ("03_item.md", ToolKind::ItemBuilder),
        ("04_weapon.md", ToolKind::WeaponBuilder),
        ("05_outfit.md", ToolKind::OutfitBuilder),
    ] {
        model.update(cx, |model, cx| model.open_file_name(page, cx));
        cx.run_until_parked();
        let panel = overlay.read_with(cx, |overlay, _| overlay.tool(kind).cloned()).expect("panel");
        panel.update(cx, |panel, cx| panel.set_open(true, cx));
        for theme in ThemeId::ALL {
            cx.update(|_, cx| yp_ui::theme::set_theme(theme, cx));
            cx.run_until_parked();
        }
        assert_eq!(panel.read_with(cx, |panel, _| panel.kind()), kind);
    }
}

#[gpui::test]
async fn the_thumbnail_dialog_snaps_sizes_and_returns_the_choice(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let image = harness.path("icon.png");
    write_png(&image, 32, 32);
    let (_stage, cx) = open_stage(cx);
    let (dialog, task) = cx.update(|window, cx| {
        thumbnail_dialog::open_for_test(thumbnail_dialog::ThumbnailOptions::new("misctex_x.dat"), window, cx)
    });
    let dialog = dialog.expect("dialog");
    cx.run_until_parked();
    assert!(has_modal(cx));
    dialog.read_with(cx, |dialog, _| {
        assert_eq!(dialog.size(), (thumbnail::DEFAULT_WIDTH, thumbnail::DEFAULT_HEIGHT));
        assert_eq!(dialog.image_path(), None);
    });
    dialog.update(cx, |dialog, cx| dialog.confirm(cx));
    cx.run_until_parked();
    assert!(has_modal(cx));

    dialog.update(cx, |dialog, cx| dialog.accept_paths(&[harness.path("notes.txt")], cx));
    assert!(dialog.read_with(cx, |dialog, _| dialog.has_error()));
    dialog.update(cx, |dialog, cx| {
        dialog.accept_paths(&[harness.path("notes.txt"), image.clone()], cx);
    });
    dialog.read_with(cx, |dialog, _| {
        assert!(!dialog.has_error());
        assert_eq!(dialog.image_path(), Some(image.as_path()));
    });
    cx.update(|window, cx| dialog.update(cx, |dialog, cx| dialog.set_size(130, 5000, window, cx)));
    assert_eq!(dialog.read_with(cx, |dialog, _| dialog.size()), (132, 1024));
    dialog.update(cx, |dialog, cx| dialog.confirm(cx));
    cx.run_until_parked();
    assert_eq!(
        task.await,
        Some(ThumbnailChoice {
            image_path: image,
            width: 132,
            height: 1024,
        })
    );
    assert!(!has_modal(cx));
}

#[gpui::test]
async fn a_fixed_size_thumbnail_cannot_be_resized_and_escape_cancels(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (_stage, cx) = open_stage(cx);
    let (dialog, task) = cx.update(|window, cx| {
        thumbnail_dialog::open_for_test(
            thumbnail_dialog::ThumbnailOptions::new("misctex_ws_1100.dat")
                .fixed_size(thumbnail::STORY_WIDTH, thumbnail::STORY_HEIGHT),
            window,
            cx,
        )
    });
    let dialog = dialog.expect("dialog");
    cx.run_until_parked();
    cx.update(|window, cx| dialog.update(cx, |dialog, cx| dialog.set_size(64, 64, window, cx)));
    assert_eq!(
        dialog.read_with(cx, |dialog, _| dialog.size()),
        (thumbnail::STORY_WIDTH, thumbnail::STORY_HEIGHT)
    );
    cx.simulate_keystrokes("escape");
    assert_eq!(task.await, None);
    assert!(!has_modal(cx));
}

#[gpui::test]
async fn the_icon_name_dialog_needs_a_mod_and_a_clean_texture_name(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (_stage, cx) = open_stage(cx);
    let (dialog, task) = cx.update(|window, cx| {
        icon_name::open_for_test(vec!["alpha".to_owned(), "beta".to_owned()], window, cx)
    });
    let dialog = dialog.expect("dialog");
    cx.run_until_parked();
    assert!(!cx.update(|_, cx| dialog.read(cx).is_valid(cx)));
    cx.simulate_keystrokes("enter");
    assert!(has_modal(cx));
    cx.simulate_input("item thumb-8500!");
    assert!(cx.update(|_, cx| dialog.read(cx).is_valid(cx)));
    dialog.update(cx, |dialog, cx| {
        dialog.select_mod("gamma", cx);
        dialog.select_mod("beta", cx);
    });
    cx.simulate_keystrokes("enter");
    assert_eq!(task.await, Some(("beta".to_owned(), "itemthumb8500".to_owned())));

    let (empty, cancelled) = cx.update(|window, cx| icon_name::open_for_test(Vec::new(), window, cx));
    let empty = empty.expect("dialog");
    cx.run_until_parked();
    cx.simulate_input("name");
    assert!(!cx.update(|_, cx| empty.read(cx).is_valid(cx)));
    cx.simulate_keystrokes("escape");
    assert_eq!(cancelled.await, None);
}

#[gpui::test]
fn mod_folders_are_listed_sorted_and_the_icon_file_label_names_both_files(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    assert!(icon_name::list_mod_folders(&harness.game).is_empty());
    for name in ["zeta", "alpha"] {
        std::fs::create_dir_all(harness.game.mods_dir().join(name)).expect("mod folder");
    }
    std::fs::write(harness.game.mods_dir().join("notes.txt"), b"x").expect("file");
    assert_eq!(icon_name::list_mod_folders(&harness.game), vec!["alpha", "zeta"]);
    assert_eq!(icon_name::icon_file_label("wpi_1100"), "misctex_wpi_1100.dat / .dtt");
}

#[gpui::test]
async fn the_toml_editor_only_writes_valid_toml_and_can_reset(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (_stage, cx) = open_stage(cx);
    let options = toml_editor::TomlEditorOptions {
        title: "Write mod.toml".into(),
        file_name: "mod.toml".into(),
        explanation: "why".into(),
        initial: "id = \"x\"\n[text]\n".to_owned(),
        reset: first_mod::MANIFEST.to_owned(),
    };
    let (dialog, task) = cx.update(|window, cx| toml_editor::open_for_test(options, window, cx));
    let dialog = dialog.expect("dialog");
    cx.run_until_parked();
    assert!(dialog.read_with(cx, |dialog, _| dialog.check().is_valid()));

    cx.simulate_input("= broken");
    assert!(!dialog.read_with(cx, |dialog, _| dialog.check().is_valid()));
    dialog.update(cx, |dialog, cx| dialog.write(cx));
    cx.run_until_parked();
    assert!(has_modal(cx));

    cx.update(|window, cx| dialog.update(cx, |dialog, cx| dialog.reset(window, cx)));
    dialog.read_with(cx, |dialog, cx| {
        assert!(dialog.check().is_valid());
        assert_eq!(dialog.text(cx), first_mod::MANIFEST);
    });
    dialog.update(cx, |dialog, cx| dialog.write(cx));
    cx.run_until_parked();
    assert_eq!(task.await.as_deref(), Some(first_mod::MANIFEST));
    assert!(!has_modal(cx));
}

#[gpui::test]
async fn the_markdown_editor_tracks_changes_previews_and_saves_with_the_shortcut(
    cx: &mut TestAppContext,
) {
    let harness = Harness::new();
    harness.init(cx);
    let (_stage, cx) = open_stage(cx);
    let options = markdown_editor::MarkdownEditorOptions {
        file_name: "01_intro.md".into(),
        file_path: "assets/docs/native/01_intro.md".into(),
        initial: "# Intro\n".to_owned(),
        save_label: "Save".into(),
    };
    let (dialog, task) =
        cx.update(|window, cx| markdown_editor::open_for_test(options, window, cx));
    let dialog = dialog.expect("dialog");
    cx.run_until_parked();
    dialog.read_with(cx, |dialog, _| {
        assert!(!dialog.is_dirty());
        assert_eq!(dialog.preview().blocks.len(), 1);
    });
    cx.simulate_input("\n## More\n\ntext\n");
    assert!(dialog.read_with(cx, |dialog, _| dialog.is_dirty()));
    assert_eq!(dialog.read_with(cx, |dialog, _| dialog.preview().blocks.len()), 1);
    dialog.update(cx, |dialog, cx| dialog.refresh_preview(cx));
    cx.run_until_parked();
    assert_eq!(dialog.read_with(cx, |dialog, _| dialog.preview().blocks.len()), 3);
    cx.simulate_keystrokes("secondary-s");
    assert_eq!(task.await.as_deref(), Some("# Intro\n\n## More\n\ntext\n"));
    assert!(!has_modal(cx));
}

#[gpui::test]
fn the_item_builder_binds_its_fields_and_writes_the_item(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    let builder = show(&stage, cx, ItemBuilder::new);
    cx.update(|window, cx| builder.update(cx, |builder, cx| builder.create(window, cx)));
    let needs_game = cx.update(|_, cx| cx.t().item_builder_needs_game_dir());
    assert_eq!(builder.read_with(cx, |builder, _| builder.error().map(ToString::to_string)), Some(needs_game.to_owned()));

    harness.select_game(cx);
    cx.run_until_parked();
    cx.update(|window, cx| builder.update(cx, |builder, cx| builder.create(window, cx)));
    let needs_name = cx.update(|_, cx| cx.t().item_builder_needs_name());
    assert_eq!(builder.read_with(cx, |builder, _| builder.error().map(ToString::to_string)), Some(needs_name.to_owned()));

    let (name, id) = builder.read_with(cx, |builder, _| (builder.name_input().clone(), builder.id_input().clone()));
    cx.update(|window, cx| {
        name.update(cx, |input, cx| input.insert("Rusty Bolt", window, cx));
        id.update(cx, |input, cx| {
            input.select_all(window, cx);
            input.insert("85x00", window, cx);
        });
    });
    cx.run_until_parked();
    builder.read_with(cx, |builder, cx| {
        assert_eq!(builder.form().name, "Rusty Bolt");
        assert_eq!(builder.form().id, 8500);
        assert_eq!(builder.id_input().read(cx).value(), "8500");
    });
    cx.update(|window, cx| builder.update(cx, |builder, cx| builder.create(window, cx)));
    cx.run_until_parked();
    let written = builder.read_with(cx, |builder, _| {
        assert_eq!(builder.error(), None);
        assert!(!builder.is_busy());
        builder.written().cloned().expect("written file")
    });
    assert_eq!(written, item::entities_dir(&harness.game, DEFAULT_ITEM_MOD).join("item_8500.toml"));
    assert!(std::fs::read_to_string(&written).expect("toml").contains("Rusty Bolt"));
    assert_eq!(builder.read_with(cx, |builder, _| builder.form().id), item::FIRST_FREE_ID);

    cx.update(|window, cx| {
        builder.update(cx, |builder, cx| {
            builder.form_mut().id = 8500;
            builder.create(window, cx);
        });
    });
    cx.run_until_parked();
    let exists = cx.update(|_, cx| cx.t().item_builder_exists());
    assert_eq!(builder.read_with(cx, |builder, _| builder.error().map(ToString::to_string)), Some(exists.to_owned()));
}

#[gpui::test]
fn the_outfit_builder_reads_the_model_meshes_and_writes_the_outfit(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    harness.select_game(cx);
    let model = write_model(&harness.path("models"), "pl000d", &["Body", "Skirt"]);
    let (stage, cx) = open_stage(cx);
    let builder = show(&stage, cx, OutfitBuilder::new);

    cx.update(|window, cx| {
        builder.update(cx, |builder, cx| {
            builder.accept_model(&harness.path("models").join("readme.txt"), window, cx);
        });
    });
    builder.read_with(cx, |builder, _| {
        assert!(builder.drop_error().is_some());
        assert!(builder.tuner().is_none());
    });

    cx.update(|window, cx| {
        builder.update(cx, |builder, cx| {
            builder.accept_model(&model, window, cx);
            builder.set_name("Black Dress", window, cx);
        });
    });
    cx.run_until_parked();
    let tuner = builder.read_with(cx, |builder, _| {
        assert_eq!(builder.drop_error(), None);
        assert_eq!(builder.form().character, OutfitCharacter::TwoB);
        builder.tuner().cloned().expect("tuner")
    });
    tuner.read_with(cx, |tuner, cx| {
        assert_eq!(tuner.mesh_names(), ["Body".to_owned(), "Skirt".to_owned()]);
        let view = tuner.view().read(cx);
        let measured = view.measured().expect("the preview was measured");
        assert!(measured.0 > 0 && measured.1 > 0);
        let session = view.session().expect("session").read(cx);
        assert_eq!(session.viewport(), measured, "the session renders at the measured size");
    });
    assert_eq!(builder.read_with(cx, |builder, _| builder.form().mesh_rules.len()), 2);

    tuner.update(cx, |tuner, cx| {
        tuner.set_rule(
            "Skirt",
            MeshRule {
                visibility: MeshVisibility::CombatOnly,
                ..MeshRule::default()
            },
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(
        builder.read_with(cx, |builder, _| builder.form().rule("Skirt").visibility),
        MeshVisibility::CombatOnly
    );

    cx.update(|window, cx| builder.update(cx, |builder, cx| builder.create(window, cx)));
    cx.run_until_parked();
    let written = builder.read_with(cx, |builder, _| {
        assert_eq!(builder.error(), None);
        builder.written().cloned().expect("written")
    });
    let toml = std::fs::read_to_string(&written).expect("outfit toml");
    assert!(toml.contains("combat_meshes = [\"Skirt\"]"));
    assert!(yp_builders::outfit::model_dir_for(&harness.game, DEFAULT_OUTFIT_MOD)
        .join("pl000d.dtt")
        .is_file());
    assert_eq!(
        builder.read_with(cx, |builder, _| builder.form().outfit_id),
        OutfitCharacter::TwoB.first_outfit_id() + 1
    );
}

#[gpui::test]
fn the_weapon_builder_accepts_a_model_and_writes_the_weapon(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    harness.select_game(cx);
    let model = write_model(&harness.path("models"), "wp0203", &["Blade"]);
    std::fs::write(harness.path("models").join("wp0003.dat"), b"x").expect("lonely dat");
    let (stage, cx) = open_stage(cx);
    let builder = show(&stage, cx, WeaponBuilder::new);

    cx.update(|window, cx| {
        builder.update(cx, |builder, cx| {
            builder.accept_model(&harness.path("models").join("wp0003.dat"), window, cx);
        });
    });
    cx.run_until_parked();
    let both = cx.update(|_, cx| cx.t().weapon_convert_needs_both_files());
    assert_eq!(builder.read_with(cx, |builder, _| builder.drop_error().map(ToString::to_string)), Some(both.to_owned()));

    cx.update(|window, cx| {
        builder.update(cx, |builder, cx| {
            builder.accept_model(&model, window, cx);
            builder.create(window, cx);
        });
    });
    cx.run_until_parked();
    let needs_name = cx.update(|_, cx| cx.t().weapon_builder_needs_name());
    builder.read_with(cx, |builder, _| {
        assert_eq!(builder.drop_error(), None);
        assert_eq!(builder.form().weapon_type, yp_builders::WeaponType::LargeSword);
        assert!(builder.form().obj_id.is_some());
        assert_eq!(builder.error().map(|error| error.as_ref()), Some(needs_name));
    });

    cx.update(|window, cx| {
        builder.update(cx, |builder, cx| {
            builder.set_name("Virtuous Blade", window, cx);
            builder.form_mut().add_level();
            builder.form_mut().add_effect();
            builder.create(window, cx);
        });
    });
    cx.run_until_parked();
    let written = builder.read_with(cx, |builder, _| {
        assert_eq!(builder.error(), None);
        builder.written().cloned().expect("written")
    });
    let toml = std::fs::read_to_string(&written).expect("weapon toml");
    assert!(toml.contains("_us = \"Virtuous Blade\""));
    assert!(toml.contains("[effect."));
}

#[gpui::test]
fn weapon_previews_go_through_the_registered_model_preview_flow(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    harness.select_game(cx);
    let requests = Rc::new(RefCell::new(Vec::new()));
    let sink = requests.clone();
    cx.update(|cx| {
        cx.registry_mut()
            .flows
            .set_model_preview(move |request, _, cx| {
                sink.borrow_mut().push(request.archives.clone());
                cx.new(|_| Stage { view: None }).into()
            });
    });
    let model = write_model(&harness.path("models"), "wp0203", &["Blade"]);
    let (stage, cx) = open_stage(cx);

    let builder = show(&stage, cx, WeaponBuilder::new);
    cx.update(|window, cx| {
        builder.update(cx, |builder, cx| builder.accept_model(&model, window, cx));
    });
    cx.run_until_parked();
    assert_eq!(builder.read_with(cx, |builder, _| builder.drop_error().cloned()), None);

    let tools = show(&stage, cx, ToolsView::new);
    cx.update(|window, cx| tools.update(cx, |tools, cx| tools.accept_weapon(&model, window, cx)));
    cx.run_until_parked();
    assert_eq!(tools.read_with(cx, |tools, _| tools.convert_drop_error().cloned()), None);

    let seen = requests.borrow().clone();
    assert_eq!(seen.len(), 2, "one request from the builder and one from the converter");
    for archives in seen {
        assert_eq!(archives.dat.as_deref(), Some(model.as_path()));
        assert_eq!(archives.dtt, Some(model.with_extension("dtt")));
    }
}

#[gpui::test]
fn the_first_mod_runner_walks_through_its_steps(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    harness.select_game(cx);
    let (stage, cx) = open_stage(cx);
    let runner = show(&stage, cx, FirstModRunner::new);
    assert!(runner.read_with(cx, |runner, _| runner.done().is_empty()));

    runner.update(cx, |runner, cx| runner.create_folder(cx));
    cx.run_until_parked();
    assert!(runner.read_with(cx, |runner, _| runner.done().contains(&STEP_FOLDER)));
    assert!(first_mod::has_folder(&harness.game));

    runner.update(cx, |runner, cx| runner.write_manifest("id = \"my_first_mod\"".to_owned(), cx));
    cx.run_until_parked();
    runner.update(cx, |runner, cx| runner.write_item(first_mod::ITEM.to_owned(), cx));
    cx.run_until_parked();
    runner.read_with(cx, |runner, _| {
        assert!(runner.done().contains(&STEP_MANIFEST));
        assert!(runner.done().contains(&STEP_ITEM));
        assert_eq!(runner.error(), None);
    });
    assert_eq!(
        std::fs::read_to_string(first_mod::manifest_path(&harness.game)).expect("manifest"),
        "id = \"my_first_mod\"\n"
    );

    runner.update(cx, |runner, cx| runner.verify(cx));
    cx.run_until_parked();
    runner.read_with(cx, |runner, _| {
        assert!(!runner.is_busy());
        assert!(runner.verify_status().is_some());
        assert_ne!(runner.verify_status(), Some(VerifyStatus::Ok));
        assert_eq!(runner.done().len(), 3);
    });
    runner.update(cx, |runner, cx| {
        runner.apply_verify(VerifyStatus::Ok);
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(runner.read_with(cx, |runner, _| runner.done().len()), STEP_COUNT);

    let fresh = show(&stage, cx, FirstModRunner::new);
    assert_eq!(fresh.read_with(cx, |runner, _| runner.done().len()), 3);
    fresh.update(cx, |runner, cx| runner.reset(cx));
    cx.run_until_parked();
    assert!(fresh.read_with(cx, |runner, _| runner.done().is_empty()));
    assert!(!first_mod::has_folder(&harness.game));
}

#[gpui::test]
fn the_tools_tab_lists_writes_and_converts(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let (stage, cx) = open_stage(cx);
    let tools = cx.update(|window, cx| {
        let view = Registry::build_tab(TabId::Tools, window, cx).expect("tools tab");
        stage.update(cx, |stage, cx| {
            stage.view = Some(view.clone());
            cx.notify();
        });
        view.downcast::<ToolsView>().expect("tools view")
    });
    cx.run_until_parked();
    assert!(tools.read_with(cx, |tools, _| tools.icons().is_empty()));

    harness.select_game(cx);
    let image = harness.path("icon.png");
    write_png(&image, 64, 32);
    let directory = thumbnail::output_dir_for(&harness.game, "sample");
    tools.update(cx, |tools, cx| {
        tools.write_icon(
            ThumbnailChoice {
                image_path: image.clone(),
                width: 64,
                height: 32,
            },
            directory.clone(),
            "item_thumb_8000".to_owned(),
            cx,
        );
    });
    cx.run_until_parked();
    tools.read_with(cx, |tools, _| {
        assert_eq!(tools.icon_error(), None);
        assert!(!tools.is_icon_busy());
        assert_eq!(tools.icons().len(), 1);
        assert_eq!(tools.icons()[0].mod_id, "sample");
        assert_eq!(tools.icons()[0].texture_name, "item_thumb_8000");
        assert!(f32::from(tools.viewport_height()) > 200.);
    });

    tools.update(cx, |tools, cx| {
        tools.write_icon(
            ThumbnailChoice {
                image_path: harness.path("missing.png"),
                width: 64,
                height: 32,
            },
            directory,
            "broken".to_owned(),
            cx,
        );
    });
    cx.run_until_parked();
    tools.read_with(cx, |tools, _| {
        assert!(tools.icon_error().is_some());
        assert_eq!(tools.icons().len(), 1);
    });

    let model = write_model(&harness.path("models"), "wp0403", &["Blade"]);
    assert!(!tools.read_with(cx, |tools, _| tools.is_card_open(ToolCard::Weapon)));
    cx.update(|window, cx| {
        tools.update(cx, |tools, cx| {
            tools.accept_weapon(&harness.path("models").join("pl0000.dat"), window, cx);
        });
    });
    cx.run_until_parked();
    assert!(tools.read_with(cx, |tools, _| tools.convert_drop_error().is_some()));
    cx.update(|window, cx| tools.update(cx, |tools, cx| tools.accept_weapon(&model, window, cx)));
    cx.run_until_parked();
    let target = tools.read_with(cx, |tools, _| {
        assert!(tools.is_card_open(ToolCard::Weapon));
        assert_eq!(tools.convert_drop_error(), None);
        tools.convert_form().target_obj_id.expect("slot")
    });
    tools.update(cx, |tools, cx| tools.run_convert(cx));
    cx.run_until_parked();
    tools.read_with(cx, |tools, _| {
        assert_eq!(tools.convert_error(), None);
        let done = tools.convert_done().expect("converted");
        assert_eq!(done.obj_id, target);
        assert_ne!(tools.convert_form().target_obj_id, Some(target));
    });
    let converted = tools.read_with(cx, |tools, _| tools.convert_done().expect("converted").stem.clone());
    assert!(yp_builders::weapon_convert::model_dir_for(&harness.game, "my_weapon")
        .join(format!("{converted}.dat"))
        .is_file());
    for theme in ThemeId::ALL {
        cx.update(|_, cx| yp_ui::theme::set_theme(theme, cx));
        cx.run_until_parked();
    }
}

#[gpui::test]
fn icon_files_are_checked_before_use(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    let target = adjust_target(&harness.path("mods/data/misctex/misctex_wpi_1100.DAT")).expect("icon");
    assert_eq!(target.texture_name, "wpi_1100");
    assert_eq!(target.file_stem, "misctex_wpi_1100");
    assert!(target.directory.ends_with("misctex"));
    assert_eq!(adjust_target(&harness.path("wp0003.dat")), None);
    assert_eq!(adjust_target(&harness.path("misctex_.dat")), None);
    assert_eq!(adjust_target(&harness.path("misctex_icon.dtt")), None);
}
