#[path = "../src/testing.rs"]
mod testing;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyView, App, AppContext as _, Bounds, Context, Global, IntoElement, Render,
    ScrollHandle, SharedString, Subscription, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, point, prelude::*, px, size,
};
use testing::Sandbox;
use yp_config::fields::bindings;
use yp_core::ProcessOutput;
use yp_core::testing::FakeRunner;
use yp_i18n::Locale;
use yp_settings_ui::{LodModTab, NaiomTab, NamsTab, TexturesTab, YorhaProtocolTab};
use yp_state::inspector::ArchivePaths;
use yp_state::prelude::*;
use yp_state::{
    ModelPreviewRequest, NavigationEvent, NotificationsEvent, OverlayRequest, Registry,
};
use yp_ui::prelude::*;
use yp_ui::{Assets, Root, ThemeId, Toast, WindowExt};

const OVERLAY_KEY: &str = "inspector";
const WINDOW_TITLE: &str = "yp_settings_ui preview";
const SAMPLE_CURSOR: &str = "C:\\Users\\Player\\Cursors\\yorha_pointer_extra_long_file_name.cur";
const SETTLE: Duration = Duration::from_millis(900);
const TEXTURE_LIST: &str = r#"{"packs":[{"name":"Kimono Textures","source":"mod","mod":"kimono_outfit","character":"2B","outfit_conditional":true,"enabled":true,"dds_count":24,"path":"nams/mods/kimono_outfit/textures"}]}"#;
const PLAYER_MESHES: [&str; 12] = [
    "Armor_Body",
    "Armor_Head",
    "Body",
    "DLC_Body",
    "DLC_Broken",
    "DLC_Skirt",
    "Eyelash",
    "Eyemask",
    "facial_normal",
    "Hair",
    "Skirt",
    "Feather",
];

#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    Nams,
    Yorha,
    LodMod,
    Naiom,
    Textures,
    Inspector,
    ModelPreview,
}

impl Screen {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "nams" => Screen::Nams,
            "yorha" => Screen::Yorha,
            "lodmod" => Screen::LodMod,
            "naiom" => Screen::Naiom,
            "textures" => Screen::Textures,
            "inspector" => Screen::Inspector,
            "preview" => Screen::ModelPreview,
            _ => return None,
        })
    }

    fn tab(&self) -> TabId {
        match self {
            Screen::Nams => TabId::Nams,
            Screen::Yorha => TabId::YorhaProtocol,
            Screen::LodMod => TabId::LodMod,
            Screen::Naiom => TabId::Naiom,
            Screen::Textures | Screen::Inspector | Screen::ModelPreview => TabId::Textures,
        }
    }
}

#[derive(Debug, Clone)]
struct Options {
    screen: Screen,
    theme: ThemeId,
    locale: Option<Locale>,
    width: f32,
    height: f32,
    expand: bool,
    scroll: f32,
    capture: Option<String>,
    dirty: bool,
    broken: bool,
    empty: bool,
    drop: bool,
    delete: Option<String>,
    reset: bool,
    input_off: bool,
    toasts: bool,
    lodmod_on: bool,
    bindings: bool,
    hold: Option<u64>,
}

impl Options {
    fn from_args() -> Self {
        let mut options = Options {
            screen: Screen::Nams,
            theme: ThemeId::Dark,
            locale: None,
            width: 1300.,
            height: 750.,
            expand: false,
            scroll: 0.,
            capture: None,
            dirty: false,
            broken: false,
            empty: false,
            drop: false,
            delete: None,
            reset: false,
            input_off: false,
            toasts: false,
            lodmod_on: false,
            bindings: false,
            hold: None,
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--tab" => {
                    if let Some(screen) = args.next().as_deref().and_then(Screen::parse) {
                        options.screen = screen;
                    }
                }
                "--theme" => {
                    if let Some(theme) = args.next().as_deref().and_then(ThemeId::from_code) {
                        options.theme = theme;
                    }
                }
                "--locale" => {
                    options.locale = args.next().as_deref().and_then(Locale::from_code);
                }
                "--size" => {
                    if let Some((width, height)) = args.next().as_deref().and_then(parse_size) {
                        options.width = width;
                        options.height = height;
                    }
                }
                "--scroll" => {
                    options.scroll = args
                        .next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(0.);
                }
                "--capture" => options.capture = args.next(),
                "--delete" => options.delete = args.next(),
                "--hold" => options.hold = args.next().and_then(|value| value.parse().ok()),
                "--expand" => options.expand = true,
                "--dirty" => options.dirty = true,
                "--broken" => options.broken = true,
                "--empty" => options.empty = true,
                "--drop" => options.drop = true,
                "--reset" => options.reset = true,
                "--input-off" => options.input_off = true,
                "--toasts" => options.toasts = true,
                "--lodmod-on" => options.lodmod_on = true,
                "--bindings" => options.bindings = true,
                _ => {}
            }
        }
        options
    }
}

fn parse_size(text: &str) -> Option<(f32, f32)> {
    let (width, height) = text.split_once('x')?;
    Some((width.parse().ok()?, height.parse().ok()?))
}

fn populate(sandbox: &Sandbox, options: &Options) -> ArchivePaths {
    let model = sandbox.write_model("outfit", "pl000d", &PLAYER_MESHES);
    if options.empty {
        return model;
    }
    sandbox.add_texture_pack(
        "NieR Automata - Texture Pack 0.75",
        &["body_albedo.dds", "hair_albedo.dds", "menu_bg.dds"],
    );
    sandbox.add_texture_pack("Main Menu-151-1-0-1626789755(1)", &["menu_bg.dds"]);
    sandbox.add_texture_pack(
        "BlurColorFogLensOverlay-785-1-1-1700000000",
        &["lens_overlay.dds", "fog.dds"],
    );
    sandbox.add_texture_pack("0_AI4K_Re[P]acked", &["body_albedo.dds", "fog.dds"]);
    sandbox.add_texture_pack("Remove Everything-149-1-0", &["lens_overlay.dds"]);
    let special_k = sandbox
        .game
        .special_k_textures_dir()
        .join("NieRAutomata.exe")
        .join("HD Texture Pack");
    std::fs::create_dir_all(&special_k).expect("sk dir");
    let wax = sandbox.game.wax_mods_dir().join("WaxTextures");
    std::fs::create_dir_all(&wax).expect("wax dir");
    std::fs::write(wax.join("grass.dds"), Sandbox::dds_bytes()).expect("wax file");
    model
}

struct PreviewShell {
    content: AnyView,
    _subscriptions: Vec<Subscription>,
}

impl PreviewShell {
    fn new(content: AnyView, toasts: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let subscriptions = vec![
            cx.subscribe_in(
                &stores.navigation,
                window,
                |_, _, event: &NavigationEvent, window, cx| match event {
                    NavigationEvent::OverlayOpened(request) => {
                        if let Some(view) = Registry::build_overlay(request, window, cx) {
                            window.show_overlay(OVERLAY_KEY, view, cx);
                        }
                    }
                    NavigationEvent::OverlayClosed(_) => window.hide_overlay(OVERLAY_KEY, cx),
                    _ => {}
                },
            ),
            cx.subscribe_in(
                &stores.notifications,
                window,
                move |_, _, event: &NotificationsEvent, window, cx| {
                    if !toasts {
                        return;
                    }
                    if let NotificationsEvent::Pushed(notice) = event {
                        let text = cx.tr(&notice.msg);
                        window.push_toast(
                            Toast::new(text)
                                .id(notice.id.clone())
                                .severity(notice.severity),
                            cx,
                        );
                    }
                },
            ),
        ];
        Self {
            content,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for PreviewShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        h_flex()
            .size_full()
            .items_start()
            .bg(theme.colors.background_primary)
            .text_color(theme.colors.text_primary)
            .font_ui(cx)
            .child(
                div()
                    .flex_none()
                    .w(sizes.sidebar_width)
                    .h_full()
                    .bg(theme.colors.background_secondary)
                    .border_r_1()
                    .border_color(theme.colors.border_light),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(self.content.clone()),
            )
    }
}

struct Harness {
    slot: Option<AnyView>,
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        v_flex()
            .size_full()
            .p(sizes.content_padding)
            .child(
                v_flex()
                    .w(sizes.dialog_width)
                    .children(self.slot.clone()),
            )
    }
}

fn scroll_handle(view: &AnyView, cx: &App) -> Option<ScrollHandle> {
    if let Ok(tab) = view.clone().downcast::<NamsTab>() {
        return Some(tab.read(cx).scroll_handle().clone());
    }
    if let Ok(tab) = view.clone().downcast::<YorhaProtocolTab>() {
        return Some(tab.read(cx).scroll_handle().clone());
    }
    if let Ok(tab) = view.clone().downcast::<LodModTab>() {
        return Some(tab.read(cx).scroll_handle().clone());
    }
    if let Ok(tab) = view.clone().downcast::<NaiomTab>() {
        return Some(tab.read(cx).scroll_handle().clone());
    }
    if let Ok(tab) = view.clone().downcast::<TexturesTab>() {
        return Some(tab.read(cx).scroll_handle().clone());
    }
    None
}

fn apply_state(
    view: &AnyView,
    options: &Options,
    sandbox_drop: Option<std::path::PathBuf>,
    model: &ArchivePaths,
    window: &mut Window,
    cx: &mut App,
) {
    let stores = cx.stores().clone();
    if options.dirty {
        stores.config.update(cx, |store, cx| {
            store.update_nams("validate_scripts", toml::Value::Boolean(true), None, cx)
        });
    }
    if options.lodmod_on {
        stores.config.update(cx, |store, cx| {
            store.update_lodmod_live("enabled", toml::Value::Boolean(true), None, cx);
            store.update_lodmod_live(
                "enabled",
                toml::Value::Boolean(true),
                Some("high_grids"),
                cx,
            );
            store.update_lodmod_live("msaa_per_pixel", toml::Value::Boolean(true), None, cx);
            store.update_lodmod_live("shadow_cascades", toml::Value::Integer(8), None, cx);
        });
    }
    if options.bindings {
        stores.config.update(cx, |store, cx| {
            let section = bindings::SECTION;
            for (key, value) in [
                ("standard_jump", "SPACE"),
                ("standard_fire", "SPACE"),
                ("standard_light_attack", "CONTROLLER_X"),
                ("standard_heavy_attack", "SHIFT+MOUSE2"),
                ("standard_move_forward", "W"),
            ] {
                store.update_naiom(key, toml::Value::String(value.to_owned()), section, cx);
            }
            store.update_naiom(
                "misc_custom_cursor_menu",
                toml::Value::String(SAMPLE_CURSOR.to_owned()),
                "mouse",
                cx,
            );
            store.update_naiom("misc_open_debug_menu", toml::Value::Integer(0x70), "mouse", cx);
            store.update_naiom(
                "movement_disable_tap_evade",
                toml::Value::Boolean(true),
                "mouse",
                cx,
            );
        });
    }
    if options.input_off {
        stores.config.update(cx, |store, cx| {
            store.update_nams_now("disable_input_features", toml::Value::Boolean(true), None, cx)
        });
    }
    if let Some(key) = options.capture.as_deref()
        && let Ok(tab) = view.clone().downcast::<NaiomTab>()
    {
        let known = bindings::ALL
            .iter()
            .map(|field| field.key())
            .chain([yp_config::fields::nams::MISC_OPEN_DEBUG_MENU.key()])
            .find(|candidate| *candidate == key);
        if let Some(key) = known {
            tab.update(cx, |tab, cx| tab.start_capture(key, window, cx));
        }
    }
    if options.reset {
        if let Ok(tab) = view.clone().downcast::<NaiomTab>() {
            tab.update(cx, |tab, cx| tab.confirm_reset(window, cx));
        }
        if let Ok(tab) = view.clone().downcast::<LodModTab>() {
            tab.update(cx, |tab, cx| tab.confirm_reset(window, cx));
        }
    }
    if let Ok(tab) = view.clone().downcast::<TexturesTab>() {
        let textures = tab.read(cx).model().clone();
        if let Some(name) = options.delete.clone() {
            textures.update(cx, |model, cx| model.delete(name, window, cx));
        }
        if let Some(path) = sandbox_drop {
            textures.update(cx, |model, cx| model.install(vec![path], window, cx));
        }
    }
    if options.screen == Screen::Inspector {
        let request = OverlayRequest::Inspector {
            archives: model.clone(),
        };
        stores
            .navigation
            .update(cx, |navigation, cx| navigation.open_overlay(request, cx));
    }
    if options.scroll > 0.
        && let Some(handle) = scroll_handle(view, cx)
    {
        handle.set_offset(point(px(0.), px(-options.scroll)));
        window.refresh();
    }
}

struct KeepSandbox(#[allow(dead_code)] Sandbox);

impl Global for KeepSandbox {}

fn main() {
    let options = Options::from_args();
    let runner =
        FakeRunner::new().respond_program("NAMS.exe", ProcessOutput::success(TEXTURE_LIST));
    let sandbox = Sandbox::with_runner(runner);
    let model = populate(&sandbox, &options);
    let drop = options
        .drop
        .then(|| sandbox.write_drop_folder("Dropped Pack", &["new_albedo.dds", "new_normal.dds"]));
    let assets = Assets::with_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
    gpui_platform::application().with_assets(assets).run(move |cx| {
        yp_ui::init(cx);
        yp_state::init_with(sandbox.env.clone(), cx);
        yp_state::register_default_handlers(cx);
        yp_settings_ui::init(cx);
        yp_ui::theme::set_theme(options.theme, cx);
        if let Some(locale) = options.locale {
            yp_ui::I18n::set(locale, cx);
        }
        yp_settings_ui::set_cards_expanded(options.expand, cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let game = sandbox.game.clone();
        if options.broken {
            std::fs::create_dir_all(game.nams_dir()).expect("nams dir");
            for file in [
                game.nams_toml(),
                game.lodmod_toml(),
                game.naiom_toml(),
                game.texture_injection_toml(),
            ] {
                std::fs::write(file, "broken = = 1
").expect("broken config");
            }
        }
        let stores = cx.stores().clone();
        let game_dir = game.root().to_path_buf();
        stores
            .session
            .update(cx, |session, cx| session.set_directory(game_dir, cx));
        cx.set_global(KeepSandbox(sandbox));
        let bounds = Bounds::centered(None, size(px(options.width), px(options.height)), cx);
        let screen = options.screen.clone();
        let toasts = options.toasts || options.drop || options.delete.is_some();
        let preview_model = model.clone();
        let content: Rc<RefCell<Option<AnyView>>> = Rc::new(RefCell::new(None));
        let built = content.clone();
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(SharedString::new_static(WINDOW_TITLE)),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    focus: false,
                    show: false,
                    window_min_size: Some(size(px(600.), px(420.))),
                    ..Default::default()
                },
                move |window, cx| {
                    let view: AnyView = match screen {
                        Screen::ModelPreview => {
                            let slot = Registry::model_preview(
                                ModelPreviewRequest {
                                    archives: preview_model.clone(),
                                },
                                window,
                                cx,
                            );
                            cx.new(|_| Harness { slot }).into()
                        }
                        other => Registry::build_tab(other.tab(), window, cx)
                            .expect("tab is registered"),
                    };
                    *built.borrow_mut() = Some(view.clone());
                    let shell = cx.new(|cx| PreviewShell::new(view, toasts, window, cx));
                    cx.new(|cx| Root::new(shell, window, cx))
                },
            )
            .expect("preview window");
        let view = content.borrow().clone();
        cx.spawn(async move |cx| {
            cx.background_executor().timer(SETTLE).await;
            if let Some(view) = view {
                cx.update_window(handle.into(), |_, window, cx| {
                    apply_state(&view, &options, drop.clone(), &model, window, cx);
                })
                .ok();
            }
            if let Some(seconds) = options.hold {
                cx.background_executor()
                    .timer(Duration::from_secs(seconds))
                    .await;
                cx.update(|cx| cx.quit());
            }
        })
        .detach();
    });
}
