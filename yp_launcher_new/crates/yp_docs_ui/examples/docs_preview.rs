use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    AnyView, AnyWindowHandle, App, AppContext as _, AsyncApp, Bounds, Context, Entity,
    IntoElement, Render, SharedString, Subscription, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, point, prelude::*, px, size,
};
use yp_builders::weapon::EFFECT_PATHS;
use yp_builders::{ItemDraft, MeshRule, MeshVisibility, first_mod, item, thumbnail};
use yp_core::testing::FakeRunner;
use yp_core::{AppPaths, GamePaths, HostEnv, Os, SettingsStore};
use yp_docs_ui::builders::ToolKind;
use yp_docs_ui::builders::first_mod::FirstModRunner;
use yp_docs_ui::builders::forms::ThumbnailChoice;
use yp_docs_ui::builders::item::ItemBuilder;
use yp_docs_ui::builders::outfit::OutfitBuilder;
use yp_docs_ui::builders::weapon::WeaponBuilder;
use yp_docs_ui::dialogs::icon_name::pick_icon_target;
use yp_docs_ui::dialogs::markdown_editor::{MarkdownEditorOptions, edit_markdown};
use yp_docs_ui::dialogs::thumbnail::{ThumbnailOptions, pick_thumbnail};
use yp_docs_ui::dialogs::toml_editor::{TomlEditorOptions, edit_toml};
use yp_docs_ui::{DocsConfig, DocsOverlay, ToolCard, ToolsView};
use yp_formats::testing::{cube_spec, dds_dxt1_solid, model_archives};
use yp_state::navigation::NavigationEvent;
use yp_state::prelude::*;
use yp_state::{AppEnv, OverlayRequest};
use yp_ui::prelude::*;
use yp_ui::{Assets, Locale, Root, TitleBar, WindowExt};

const FIXTURE_PREFIX: &str = "yp-docs-ui-preview-";
const OVERLAY_KEY: &str = "docs";
const POLL: Duration = Duration::from_millis(50);
const SETTLE: Duration = Duration::from_millis(350);
const MAX_POLLS: usize = 200;
const STALE_AFTER: Duration = Duration::from_secs(1800);

#[derive(Debug, Clone, PartialEq)]
struct Options {
    screen: String,
    page: Option<String>,
    anchor: Option<String>,
    scroll: Option<f32>,
    query: Option<String>,
    tool: bool,
    fill: bool,
    dialog: Option<String>,
    card: Option<String>,
    theme: ThemeId,
    locale: Locale,
    width: f32,
    height: f32,
    single: bool,
    editable: bool,
    no_game: bool,
    visible: bool,
}

impl Options {
    fn from_args() -> Self {
        let mut options = Self {
            screen: "home".to_owned(),
            page: None,
            anchor: None,
            scroll: None,
            query: None,
            tool: false,
            fill: false,
            dialog: None,
            card: None,
            theme: ThemeId::Dark,
            locale: Locale::En,
            width: 1300.,
            height: 750.,
            single: false,
            editable: false,
            no_game: false,
            visible: false,
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--tool" => options.tool = true,
                "--fill" => options.fill = true,
                "--single" => options.single = true,
                "--editable" => options.editable = true,
                "--no-game" => options.no_game = true,
                "--visible" => options.visible = true,
                _ => {
                    let Some(value) = args.next() else {
                        break;
                    };
                    match flag.as_str() {
                        "--screen" => options.screen = value,
                        "--page" => options.page = Some(value),
                        "--anchor" => options.anchor = Some(value),
                        "--scroll" => options.scroll = value.parse().ok(),
                        "--query" => options.query = Some(value),
                        "--dialog" => options.dialog = Some(value),
                        "--card" => options.card = Some(value),
                        "--theme" => {
                            options.theme = ThemeId::from_code(&value).unwrap_or(ThemeId::Dark);
                        }
                        "--locale" => {
                            options.locale = Locale::from_code(&value).unwrap_or(Locale::En);
                        }
                        "--width" => options.width = value.parse().unwrap_or(options.width),
                        "--height" => options.height = value.parse().unwrap_or(options.height),
                        _ => {}
                    }
                }
            }
        }
        options
    }
}

struct Fixture {
    root: PathBuf,
    game: GamePaths,
    player_model: PathBuf,
    weapon_model: PathBuf,
    image: PathBuf,
    docs_dir: PathBuf,
}

fn workspace_docs() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("docs")
}

fn sweep_old_fixtures(temp: &Path) {
    let Ok(entries) = std::fs::read_dir(temp) else {
        return;
    };
    for entry in entries.flatten() {
        let stale = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|modified| modified.elapsed().ok())
            .is_some_and(|age| age > STALE_AFTER);
        if stale
            && entry
                .file_name()
                .to_string_lossy()
                .starts_with(FIXTURE_PREFIX)
        {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

fn write_model(dir: &Path, stem: &str, meshes: &[&str], color: u16) -> PathBuf {
    std::fs::create_dir_all(dir).expect("model folder");
    let archives = model_archives(
        stem,
        cube_spec(meshes).build(),
        &[(0xC0BE, dds_dxt1_solid(8, 8, color))],
    );
    let dat = dir.join(format!("{stem}.dat"));
    std::fs::write(&dat, archives.dat).expect("model dat");
    std::fs::write(dir.join(format!("{stem}.dtt")), archives.dtt).expect("model dtt");
    dat
}

fn write_image(path: &Path, width: u32, height: u32, tint: [u8; 3]) {
    let radius = width.min(height) as f32 * 0.36;
    let image = image::RgbaImage::from_fn(width, height, |x, y| {
        let shade = ((x * 255 / width.max(1)) as u8) / 2 + ((y * 255 / height.max(1)) as u8) / 4;
        let dx = x as f32 - width as f32 / 2.;
        let dy = y as f32 - height as f32 / 2.;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance <= radius && distance >= radius * 0.82 {
            return image::Rgba([240, 236, 220, 255]);
        }
        image::Rgba([
            tint[0].saturating_add(shade / 2),
            tint[1].saturating_add(shade / 3),
            tint[2].saturating_add(shade),
            255,
        ])
    });
    image.save(path).expect("fixture image");
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("copy target");
    for entry in std::fs::read_dir(from).expect("copy source").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

impl Fixture {
    fn create(editable_docs: bool) -> Self {
        let temp = std::env::temp_dir();
        sweep_old_fixtures(&temp);
        let root = temp.join(format!("{FIXTURE_PREFIX}{}", std::process::id()));
        let game = GamePaths::new(root.join("game"));
        std::fs::create_dir_all(game.root()).expect("game folder");
        std::fs::write(game.exe(), b"MZ preview").expect("fake exe");
        std::fs::create_dir_all(game.nams_dir()).expect("nams folder");

        let sources = root.join("sources");
        std::fs::create_dir_all(&sources).expect("sources folder");
        let image = sources.join("icon.png");
        write_image(&image, 256, 128, [150, 90, 30]);
        let second = sources.join("badge.png");
        write_image(&second, 128, 128, [40, 110, 90]);

        for (index, (name, description)) in [
            ("Rusty Bolt", "A bolt. It has seen better days."),
            ("Test Pebble", "A small stone."),
        ]
        .into_iter()
        .enumerate()
        {
            let draft = ItemDraft::new(name, description, item::FIRST_FREE_ID + index as i64);
            item::write(&game, "sample_mod", &draft).expect("sample item");
        }
        let icons = thumbnail::output_dir_for(&game, "sample_mod");
        thumbnail::create_named(&image, &icons, "item_thumb_8000", 256, 128).expect("icon");
        thumbnail::create_named(&second, &icons, "wpi_1100", 128, 128).expect("icon");
        let other = thumbnail::output_dir_for(&game, "outfit_pack");
        thumbnail::create_named(&second, &other, "item_thumb_8200", 64, 64).expect("icon");

        let models = root.join("models");
        let player_model = write_model(
            &models,
            "pl000d",
            &["Body", "Skirt", "Hair", "Eyemask", "Feather", "Broken"],
            0xD6BA,
        );
        let weapon_model = write_model(&models, "wp0003", &["Blade", "Guard"], 0x8C71);

        let docs_dir = if editable_docs {
            let copy = root.join("docs");
            copy_tree(&workspace_docs(), &copy);
            copy
        } else {
            workspace_docs()
        };
        Self {
            root,
            game,
            player_model,
            weapon_model,
            image,
            docs_dir,
        }
    }
}

struct PreviewTitleBar;

impl Render for PreviewTitleBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new(cx.t().app_title()).icon(IconName::SportsEsports)
    }
}

struct PreviewHost {
    tab: Option<AnyView>,
    overlay: Option<AnyView>,
    _subscriptions: Vec<Subscription>,
}

impl PreviewHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let navigation = cx.stores().navigation.clone();
        let subscription = cx.subscribe_in(
            &navigation,
            window,
            |this, _, event: &NavigationEvent, window, cx| match event {
                NavigationEvent::OverlayOpened(request) => {
                    if let Some(view) = Registry::build_overlay(request, window, cx) {
                        this.overlay = Some(view.clone());
                        window.show_overlay(OVERLAY_KEY, view, cx);
                    }
                }
                NavigationEvent::OverlayClosed(_) => {
                    this.overlay = None;
                    window.hide_overlay(OVERLAY_KEY, cx);
                }
                _ => {}
            },
        );
        Self {
            tab: None,
            overlay: None,
            _subscriptions: vec![subscription],
        }
    }

    fn show_tab(&mut self, tab: TabId, window: &mut Window, cx: &mut Context<Self>) {
        self.tab = Registry::build_tab(tab, window, cx);
        cx.notify();
    }

    fn show_overlay_view(&mut self, view: AnyView, window: &mut Window, cx: &mut Context<Self>) {
        self.overlay = Some(view.clone());
        window.show_overlay(OVERLAY_KEY, view, cx);
        cx.notify();
    }

    fn docs(&self) -> Option<Entity<DocsOverlay>> {
        self.overlay.clone()?.downcast::<DocsOverlay>().ok()
    }

    fn tools(&self) -> Option<Entity<ToolsView>> {
        self.tab.clone()?.downcast::<ToolsView>().ok()
    }
}

impl Render for PreviewHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        div().relative().size_full().child(div().absolute().inset_0().child(
            match &self.tab {
                Some(tab) => tab.clone().into_any_element(),
                None => EmptyState::new(strings.app_title())
                    .icon(IconName::MenuBookOutlined)
                    .into_any_element(),
            },
        ))
    }
}

type Preview = AnyWindowHandle;

async fn pause(cx: &mut AsyncApp, duration: Duration) {
    cx.background_executor().timer(duration).await;
}

async fn wait_until(
    cx: &mut AsyncApp,
    mut ready: impl FnMut(&mut AsyncApp) -> bool,
) -> bool {
    for _ in 0..MAX_POLLS {
        if ready(cx) {
            return true;
        }
        pause(cx, POLL).await;
    }
    false
}

fn with_docs<R>(
    host: &Entity<PreviewHost>,
    window: Preview,
    cx: &mut AsyncApp,
    action: impl FnOnce(&mut DocsOverlay, &mut Window, &mut Context<DocsOverlay>) -> R,
) -> Option<R> {
    window
        .update(cx, |_, window, cx| {
            let docs = host.read(cx).docs()?;
            Some(docs.update(cx, |docs, cx| action(docs, window, cx)))
        })
        .ok()
        .flatten()
}

fn fill_item(builder: &Entity<ItemBuilder>, fixture: &Fixture, window: &mut Window, cx: &mut App) {
    let image = fixture.image.clone();
    builder.update(cx, |builder, cx| {
        builder.set_name("Rusty Bolt", window, cx);
        builder.set_description("A bolt. It has seen better days.", window, cx);
        let form = builder.form_mut();
        form.carry = 12;
        form.sellable = true;
        form.sell_price = 250;
        form.icon = Some(ThumbnailChoice {
            image_path: image,
            width: 256,
            height: 128,
        });
        cx.notify();
    });
}

fn fill_outfit(
    builder: &Entity<OutfitBuilder>,
    fixture: &Fixture,
    window: &mut Window,
    cx: &mut App,
) {
    let model = fixture.player_model.clone();
    builder.update(cx, |builder, cx| {
        builder.accept_model(&model, window, cx);
        builder.set_name("Black Dress", window, cx);
        builder.set_description("Formal wear for the end of the world.", window, cx);
        let form = builder.form_mut();
        form.shop.enabled = true;
        form.shop.toggle_state(2);
        cx.notify();
    });
}

fn tune_outfit(builder: &Entity<OutfitBuilder>, cx: &mut App) {
    let Some(tuner) = builder.read(cx).tuner().cloned() else {
        return;
    };
    tuner.update(cx, |tuner, cx| {
        tuner.set_rule(
            "Skirt",
            MeshRule {
                visibility: MeshVisibility::OutOfCombatOnly,
                hide_on_self_destruct: true,
                ..MeshRule::default()
            },
            cx,
        );
        tuner.set_rule(
            "Eyemask",
            MeshRule {
                visibility: MeshVisibility::EyemaskOnly,
                ..MeshRule::default()
            },
            cx,
        );
    });
}

fn fill_weapon(
    builder: &Entity<WeaponBuilder>,
    fixture: &Fixture,
    window: &mut Window,
    cx: &mut App,
) {
    let model = fixture.weapon_model.clone();
    builder.update(cx, |builder, cx| {
        builder.accept_model(&model, window, cx);
        builder.set_name("Virtuous Blade", window, cx);
        builder.set_description("Sharp enough to cut a promise.", window, cx);
        let form = builder.form_mut();
        form.add_level();
        form.add_level();
        form.add_upgrade();
        form.add_upgrade();
        form.add_effect();
        form.set_effect_path(0, EFFECT_PATHS[3]);
        form.add_effect();
        form.set_effect_path(1, EFFECT_PATHS[9]);
        form.set_effect_when(1, Some("9S".to_owned()));
        form.shop.enabled = true;
        cx.notify();
    });
}

fn tool_for_page(page: &str) -> Option<ToolKind> {
    match page {
        "01_first_mod.md" => Some(ToolKind::FirstMod),
        "03_item.md" => Some(ToolKind::ItemBuilder),
        "04_weapon.md" => Some(ToolKind::WeaponBuilder),
        "05_outfit.md" => Some(ToolKind::OutfitBuilder),
        _ => None,
    }
}

fn open_dialog(kind: &str, fixture: &Fixture, window: &mut Window, cx: &mut App) {
    let strings = cx.t();
    match kind {
        "thumbnail" => pick_thumbnail(
            ThumbnailOptions::new("misctex_item_thumb_8000.dat / .dtt").initial(Some(
                ThumbnailChoice {
                    image_path: fixture.image.clone(),
                    width: 256,
                    height: 128,
                },
            )),
            window,
            cx,
        )
        .detach(),
        "thumbnail-empty" => {
            pick_thumbnail(ThumbnailOptions::new("misctex_wpi_1100.dat / .dtt"), window, cx)
                .detach();
        }
        "thumbnail-story" => pick_thumbnail(
            ThumbnailOptions::new("misctex_ws_1100.dat / .dtt")
                .fixed_size(thumbnail::STORY_WIDTH, thumbnail::STORY_HEIGHT)
                .initial(Some(ThumbnailChoice {
                    image_path: fixture.image.clone(),
                    width: thumbnail::STORY_WIDTH,
                    height: thumbnail::STORY_HEIGHT,
                })),
            window,
            cx,
        )
        .detach(),
        "icon-name" => pick_icon_target(fixture.game.clone(), window, cx).detach(),
        "toml" | "toml-invalid" => {
            let initial = if kind == "toml" {
                first_mod::ITEM.to_owned()
            } else {
                "name = \"item_test_pebble\"\nid_type = \n[text.name\n".to_owned()
            };
            edit_toml(
                TomlEditorOptions {
                    title: SharedString::new_static(strings.first_mod_step3()),
                    file_name: SharedString::new_static("entities/item_9999.toml"),
                    explanation: SharedString::new_static(strings.first_mod_explain_item()),
                    initial,
                    reset: first_mod::ITEM.to_owned(),
                },
                window,
                cx,
            )
            .detach();
        }
        "markdown" => {
            let source = std::fs::read_to_string(fixture.docs_dir.join("native").join("03_item.md"))
                .unwrap_or_default();
            edit_markdown(
                MarkdownEditorOptions {
                    file_name: SharedString::new_static("03_item.md"),
                    file_path: SharedString::new_static("assets/docs/native/03_item.md"),
                    initial: source,
                    save_label: SharedString::new_static(strings.docs_save_file()),
                },
                window,
                cx,
            )
            .detach();
        }
        _ => {}
    }
}

async fn drive(
    options: Options,
    fixture: Arc<Fixture>,
    window: Preview,
    host: Entity<PreviewHost>,
    cx: &mut AsyncApp,
) {
    if !options.no_game {
        let directory = fixture.game.root().to_path_buf();
        cx.update(|cx| {
            let session = cx.stores().session.clone();
            session.update(cx, |session, cx| session.set_directory(directory, cx));
        });
        wait_until(cx, |cx| {
            cx.update(|cx| !cx.stores().session.read(cx).is_preparing())
        })
        .await;
    }
    pause(cx, SETTLE).await;

    if options.screen == "tools" {
        window
            .update(cx, |_, window, cx| {
                host.update(cx, |host, cx| host.show_tab(TabId::Tools, window, cx));
            })
            .ok();
        pause(cx, SETTLE).await;
        let card = options.card.clone().unwrap_or_default();
        let weapon = fixture.weapon_model.clone();
        window
            .update(cx, |_, window, cx| {
                let Some(tools) = host.read(cx).tools() else {
                    return;
                };
                tools.update(cx, |tools, cx| match card.as_str() {
                    "weapon" => {
                        tools.set_card_open(ToolCard::Icons, false, cx);
                        tools.accept_weapon(&weapon, window, cx);
                    }
                    "weapon-empty" => {
                        tools.set_card_open(ToolCard::Icons, false, cx);
                        tools.set_card_open(ToolCard::Weapon, true, cx);
                    }
                    "all" => {
                        tools.set_card_open(ToolCard::Weapon, true, cx);
                    }
                    "closed" => tools.set_card_open(ToolCard::Icons, false, cx),
                    _ => {}
                });
            })
            .ok();
    } else {
        let single = options.single;
        let single_page = options
            .page
            .clone()
            .filter(|_| single)
            .map(|page| format!("assets/docs/native/{page}"));
        window
            .update(cx, |_, window, cx| {
                if single {
                    let view: AnyView = cx
                        .new(|cx| DocsOverlay::new(single_page, true, window, cx))
                        .into();
                    host.update(cx, |host, cx| host.show_overlay_view(view, window, cx));
                } else {
                    let navigation = cx.stores().navigation.clone();
                    navigation.update(cx, |navigation, cx| {
                        navigation.open_overlay(OverlayRequest::Docs { page: None }, cx);
                    });
                }
            })
            .ok();
        wait_until(cx, |cx| {
            with_docs(&host, window, cx, |docs, _, cx| {
                !docs.model().read(cx).is_loading()
            })
            .unwrap_or(false)
        })
        .await;

        if let Some(page) = options.page.clone().filter(|_| !single) {
            with_docs(&host, window, cx, |docs, _, cx| {
                docs.model()
                    .update(cx, |model, cx| model.open_file_name(&page, cx));
            });
            wait_until(cx, |cx| {
                with_docs(&host, window, cx, |docs, _, cx| {
                    docs.model().read(cx).current().is_some()
                })
                .unwrap_or(false)
            })
            .await;
            pause(cx, SETTLE).await;
            if let Some(kind) = tool_for_page(&page).filter(|_| options.tool || options.fill) {
                let fill = options.fill;
                let fixture = fixture.clone();
                with_docs(&host, window, cx, |docs, window, cx| {
                    let Some(panel) = docs.tool(kind).cloned() else {
                        return;
                    };
                    panel.update(cx, |panel, cx| panel.set_open(true, cx));
                    if !fill {
                        return;
                    }
                    let content = panel.read(cx).content().clone();
                    if let Ok(builder) = content.clone().downcast::<ItemBuilder>() {
                        fill_item(&builder, &fixture, window, cx);
                    } else if let Ok(builder) = content.clone().downcast::<OutfitBuilder>() {
                        fill_outfit(&builder, &fixture, window, cx);
                    } else if let Ok(builder) = content.clone().downcast::<WeaponBuilder>() {
                        fill_weapon(&builder, &fixture, window, cx);
                    } else if let Ok(runner) = content.downcast::<FirstModRunner>() {
                        runner.update(cx, |runner, cx| runner.create_folder(cx));
                    }
                });
                if fill && kind == ToolKind::FirstMod {
                    for step in 0..3 {
                        pause(cx, Duration::from_millis(700)).await;
                        with_docs(&host, window, cx, |docs, _, cx| {
                            let Some(panel) = docs.tool(kind).cloned() else {
                                return;
                            };
                            let content = panel.read(cx).content().clone();
                            if let Ok(runner) = content.downcast::<FirstModRunner>() {
                                runner.update(cx, |runner, cx| match step {
                                    0 => runner.write_manifest(first_mod::MANIFEST.to_owned(), cx),
                                    1 => runner.write_item(first_mod::ITEM.to_owned(), cx),
                                    _ => runner.verify(cx),
                                });
                            }
                        });
                    }
                }
                if fill && kind == ToolKind::OutfitBuilder {
                    pause(cx, Duration::from_millis(1500)).await;
                    with_docs(&host, window, cx, |docs, _, cx| {
                        let Some(panel) = docs.tool(kind).cloned() else {
                            return;
                        };
                        let content = panel.read(cx).content().clone();
                        if let Ok(builder) = content.downcast::<OutfitBuilder>() {
                            tune_outfit(&builder, cx);
                        }
                    });
                }
                pause(cx, SETTLE).await;
            }
        }
        if let Some(query) = options.query.clone() {
            with_docs(&host, window, cx, |docs, window, cx| {
                docs.set_query(&query, window, cx);
            });
        }
        if let Some(anchor) = options.anchor.clone() {
            pause(cx, SETTLE).await;
            with_docs(&host, window, cx, |docs, _, cx| {
                docs.scroll_to_anchor(&anchor, false, cx);
            });
        }
        if let Some(scroll) = options.scroll {
            pause(cx, SETTLE).await;
            with_docs(&host, window, cx, |docs, _, cx| {
                docs.content_scroll().set_offset(point(px(0.), px(-scroll)));
                cx.notify();
            });
        }
    }

    if let Some(dialog) = options.dialog.clone() {
        pause(cx, SETTLE).await;
        let fixture = fixture.clone();
        window
            .update(cx, |_, window, cx| {
                open_dialog(&dialog, &fixture, window, cx);
            })
            .ok();
    }
}

fn main() {
    let options = Options::from_args();
    let assets = Assets::with_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
    gpui_platform::application().with_assets(assets).run(move |cx| {
        yp_ui::init(cx);
        yp_ui::set_link_opener(|_, _| {}, cx);
        let temp = std::env::temp_dir().join(format!("{FIXTURE_PREFIX}{}", std::process::id()));
        let appdata = temp.join("appdata").to_string_lossy().into_owned();
        let host_env = HostEnv::for_test(Os::Windows, &[("APPDATA", appdata.as_str())], &temp);
        let fixture = Arc::new(Fixture::create(options.editable));
        std::fs::create_dir_all(host_env.temp_dir()).expect("temp folder");
        let paths = AppPaths::for_test(&fixture.root);
        std::fs::create_dir_all(paths.plugins_dir()).expect("plugins folder");
        std::fs::create_dir_all(&paths.logs_dir).expect("logs folder");
        let env = AppEnv::new(
            host_env,
            paths,
            Arc::new(SettingsStore::in_memory()),
            Arc::new(FakeRunner::new()),
        )
        .with_launch_grace(Duration::ZERO)
        .with_file_watching(false)
        .with_gpu_registry(false);
        yp_state::init_with(env, cx);
        yp_state::register_default_handlers(cx);
        yp_ui::theme::set_theme(options.theme, cx);
        yp_ui::I18n::set(options.locale, cx);
        cx.set_global(DocsConfig {
            docs_dir: fixture.docs_dir.clone(),
            editable: options.editable,
        });
        yp_docs_ui::init(cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let cleanup = fixture.root.clone();
        cx.on_app_quit(move |_| {
            let cleanup = cleanup.clone();
            async move {
                let _ = std::fs::remove_dir_all(cleanup);
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(options.width), px(options.height)), cx);
        let title = SharedString::new_static(cx.t().app_title());
        let mut host_slot = None;
        let window: Preview = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(title),
                        appears_transparent: cfg!(target_os = "windows"),
                        traffic_light_position: None,
                    }),
                    window_min_size: Some(size(px(600.), px(420.))),
                    focus: false,
                    show: options.visible,
                    ..Default::default()
                },
                |window, cx| {
                    let host = cx.new(|cx| PreviewHost::new(window, cx));
                    host_slot = Some(host.clone());
                    let title_bar = cx.new(|_| PreviewTitleBar);
                    cx.new(|cx| {
                        let mut root = Root::new(host, window, cx);
                        if cfg!(target_os = "windows") {
                            root.set_title_bar(Some(title_bar.into()), cx);
                        }
                        root
                    })
                },
            )
            .expect("preview window")
            .into();
        let host = host_slot.expect("preview host");
        cx.spawn(async move |cx| drive(options, fixture, window, host, cx).await)
            .detach();
    });
}
