#[path = "../src/testing.rs"]
mod testing;

use std::path::PathBuf;
use std::time::Duration;

use futures::channel::oneshot;
use gpui::{
    AnyView, App, AppContext as _, Bounds, Context, Entity, Global, IntoElement,
    Render, SharedString, Subscription, TitlebarOptions, Window, WindowBounds, WindowOptions,
    point, prelude::*, px, size,
};
use testing::{CODEC_H264, CODEC_MPEG2, Sandbox};
use yp_mods::defaults::DefaultKind;
use yp_mods::{DataCategory, ModKind, ModVariant};
use yp_mods_ui::mods::dialogs::{
    BusyDialog, NamingRequest, OutfitGuideDialog, confirm_uninstall, prompt_mod_name,
};
use yp_mods_ui::mods::profile::{create_profile, delete_profile};
use yp_mods_ui::mods::tutorial::TutorialKind;
use yp_mods_ui::mods::variant::pick_variants;
use yp_mods_ui::{CutscenesView, ModsView, ThirdPartyView};
use yp_state::prelude::*;
use yp_state::{NotificationsEvent, Registry};
use yp_ui::prelude::*;
use yp_ui::{Assets, Card, Locale, Root, Toast, WindowExt};

const WINDOW_TITLE: &str = "yp_mods_ui preview";
const SETTLE: Duration = Duration::from_millis(900);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scene {
    Default,
    Empty,
    Installed,
    Found,
}

#[derive(Debug, Clone)]
struct Options {
    tab: TabId,
    theme: ThemeId,
    locale: Option<Locale>,
    width: f32,
    height: f32,
    scene: Scene,
    select: Option<String>,
    filter: Option<String>,
    tutorial: Option<TutorialKind>,
    dialog: Option<String>,
    toasts: bool,
    expand: bool,
    activity: bool,
    collapse: bool,
    hold: Option<u64>,
}

impl Options {
    fn from_args() -> Self {
        let mut options = Options {
            tab: TabId::Mods,
            theme: ThemeId::Dark,
            locale: None,
            width: 1300.,
            height: 750.,
            scene: Scene::Default,
            select: None,
            filter: None,
            tutorial: None,
            dialog: None,
            toasts: false,
            expand: false,
            activity: false,
            collapse: false,
            hold: None,
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            let mut value = || args.next().unwrap_or_default();
            match flag.as_str() {
                "--tab" => {
                    options.tab = match value().as_str() {
                        "cutscenes" => TabId::Cutscenes,
                        "thirdparty" => TabId::ThirdParty,
                        _ => TabId::Mods,
                    }
                }
                "--theme" => {
                    options.theme = ThemeId::from_code(&value()).unwrap_or(ThemeId::Dark);
                }
                "--locale" => options.locale = Locale::from_code(&value()),
                "--width" => options.width = value().parse().unwrap_or(1300.),
                "--height" => options.height = value().parse().unwrap_or(750.),
                "--scene" => {
                    options.scene = match value().as_str() {
                        "empty" => Scene::Empty,
                        "installed" => Scene::Installed,
                        "found" => Scene::Found,
                        _ => Scene::Default,
                    }
                }
                "--select" => options.select = Some(value()),
                "--filter" => options.filter = Some(value()),
                "--tutorial" => options.tutorial = TutorialKind::from_key(&value()),
                "--dialog" => options.dialog = Some(value()),
                "--hold" => options.hold = value().parse().ok(),
                "--toasts" => options.toasts = true,
                "--expand" => options.expand = true,
                "--activity" => options.activity = true,
                "--collapse" => options.collapse = true,
                _ => {}
            }
        }
        options
    }
}

struct Keep {
    _sandbox: Sandbox,
    _busy_owner: Option<oneshot::Sender<()>>,
}

impl Global for Keep {}

fn populate(sandbox: &Sandbox, scene: Scene) {
    if scene == Scene::Empty {
        return;
    }
    sandbox.add_native_mod("bunker_free_cam", "Bunker Free-Cam", "1.0");
    sandbox.add_camera_settings("bunker_free_cam");
    sandbox.add_data_file("bunker_free_cam", "wd1", "wd1_bunker.dat", 64);
    sandbox.add_native_mod("bunker_color", "Bunker Color", "1.0");
    sandbox.add_native_mod("grain_removal", "Bunker Grain Removal", "1.2");
    sandbox.add_outfit_mod("2b_pl000d_stellar_blade_edition", "pl000d");
    sandbox.add_bundled_textures(
        "2b_pl000d_stellar_blade_edition",
        &["Stellar Blade 4K (2b_pl000d_stellar_blade_edition@default)"],
    );
    sandbox.add_bundled_cutscene("2b_pl000d_stellar_blade_edition", "Stellar Intro");
    sandbox.add_wax_mod("wax_alice", "pl000d");
    sandbox.add_outfit_mod("nines_casual", "pl0200");
    sandbox.add_data_file("virtuous_contract_gold", "wp", "wp0000.dat", 64);
    sandbox.add_data_file("virtuous_contract_neon", "wp", "wp0000.dat", 128);
    sandbox.add_data_file("desert_fog", "effect", "fog.dat", 32);
    sandbox.add_data_file(
        "type_40_blade_ultra_hd_remaster_with_extra_glow_and_particles_v2_final",
        "wp",
        "wp0400.dat",
        48,
    );
    sandbox.add_outfit_mod("a2_wanderer", "pl0100");
    sandbox.add_requirements(
        "a2_wanderer",
        &["nams_core", "wax_alice"],
        &["yorha_protocol"],
    );
    sandbox.add_cutscene_mod("NA AI 1440p30 Cutscenes 1.1", 6, 2560, 1440, CODEC_MPEG2);
    sandbox.add_cutscene_mod("Ending E Remaster", 2, 3840, 2160, CODEC_H264);
    if scene == Scene::Installed {
        sandbox.install_reshade();
        sandbox.install_migoto("NieRAutomata.exe");
        sandbox.add_game_mod("SpeedrunTimer", "timer.dll", 180_000);
        sandbox.add_game_mod("FreeCamTool", "freecam.dll", 2_400_000);
        sandbox.add_graphic_mod("ColorWrapper", "d3d11.dll", 900_000);
    }
    if scene == Scene::Found {
        sandbox.add_game_root_reshade();
        sandbox.write_nams_toml("disable_reshade_loading = true\n");
    }
}

struct PreviewSlot {
    label: SharedString,
}

impl Render for PreviewSlot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        v_flex().w_full().child(
            Card::new().title(self.label.clone()).child(
                h_flex()
                    .w_full()
                    .justify_center()
                    .py(sizes.padding_xl)
                    .child(
                        Icon::new(IconName::ViewInAr)
                            .size(IconSize::Custom(sizes.icon_lg * 3.))
                            .color(Color::Muted),
                    ),
            ),
        )
    }
}

struct PreviewShell {
    tab: AnyView,
    _subscriptions: Vec<Subscription>,
}

impl PreviewShell {
    fn new(tab: AnyView, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let notifications = cx.stores().notifications.clone();
        let subscription = cx.subscribe_in(
            &notifications,
            window,
            |_, _, event: &NotificationsEvent, window, cx| {
                if let NotificationsEvent::Pushed(notice) = event {
                    let toast = Toast::new(cx.tr(&notice.msg))
                        .id(notice.id.clone())
                        .severity(notice.severity);
                    window.push_toast(toast, cx);
                }
            },
        );
        Self {
            tab,
            _subscriptions: vec![subscription],
        }
    }
}

impl Render for PreviewShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.tab.clone())
    }
}

fn sample_variants() -> Vec<ModVariant> {
    let variant = |label: &str, category, texture_only| ModVariant {
        sub_path: PathBuf::from(label),
        label: label.to_owned(),
        kind: ModKind::Data,
        texture_only,
        category,
    };
    vec![
        variant("2P - Classic", Some(DataCategory::Player), false),
        variant("2P - Final Fantasy", Some(DataCategory::Player), false),
        variant("2P - Soulcalibur", Some(DataCategory::Player), false),
        variant(
            "Virtuous Contract / Gold",
            Some(DataCategory::Weapon),
            false,
        ),
        variant(
            "Virtuous Contract / Neon",
            Some(DataCategory::Weapon),
            false,
        ),
        variant("Textures / 2P I Shared Textures", None, true),
    ]
}

fn open_dialog(
    name: &str,
    tab: &AnyView,
    sandbox_drops: &Drops,
    window: &mut Window,
    cx: &mut App,
) {
    match name {
        "variant" => pick_variants(sample_variants(), window, cx).detach(),
        "naming" => prompt_mod_name(NamingRequest::new("Kimono Wax"), window, cx).detach(),
        "exists" => prompt_mod_name(
            NamingRequest::new("Kimono Wax").exists("kimono_wax"),
            window,
            cx,
        )
        .detach(),
        "uninstall" => {
            let installed = cx
                .stores()
                .mods
                .read(cx)
                .find("2b_pl000d_stellar_blade_edition")
                .cloned();
            if let Some(installed) = installed {
                confirm_uninstall(&installed, window, cx).detach();
            }
        }
        "guide" => {
            window.open_modal(cx, |_, cx| {
                OutfitGuideDialog::new(
                    "Stellar Blade Edition".to_owned(),
                    "F1".to_owned(),
                    true,
                    cx,
                )
            });
        }
        "busy" => {
            let (owner, alive) = oneshot::channel();
            let message = cx.t().extracting_archive_percent_file(42, "pl000d.dtt");
            window.open_modal(cx, |_, cx| BusyDialog::new(message, alive, cx));
            cx.global_mut::<Keep>()._busy_owner = Some(owner);
        }
        "profile-new" => create_profile(window, cx),
        "profile-delete" => delete_profile(window, cx),
        "conflict" => {
            if let Ok(view) = tab.clone().downcast::<ModsView>() {
                view.update(cx, |view, cx| view.resolve_selected_conflicts(window, cx));
            }
        }
        "cutscene-install" => {
            if let Ok(view) = tab.clone().downcast::<CutscenesView>() {
                let source = sandbox_drops.cutscene.clone();
                view.update(cx, |view, cx| view.install(source, window, cx));
            }
        }
        "update" => {
            if let Ok(view) = tab.clone().downcast::<ThirdPartyView>() {
                let source = sandbox_drops.game_mod.clone();
                view.update(cx, |view, cx| view.install_paths(vec![source], window, cx));
            }
        }
        _ => {}
    }
}

#[derive(Clone)]
struct Drops {
    cutscene: PathBuf,
    game_mod: PathBuf,
}

fn apply_state(tab: &AnyView, options: &Options, drops: &Drops, window: &mut Window, cx: &mut App) {
    if let Ok(view) = tab.clone().downcast::<ModsView>() {
        view.update(cx, |view, cx| {
            if let Some(id) = &options.select {
                view.select(Some(id.clone()), cx);
            }
            if let Some(filter) = &options.filter {
                view.set_filter(filter, window, cx);
            }
            if options.collapse {
                view.toggle_group(yp_mods::grouping::ModGroupKind::Native, cx);
            }
            if let Some(kind) = options.tutorial {
                view.start_tutorial(kind, cx);
            }
        });
    }
    if let Ok(view) = tab.clone().downcast::<CutscenesView>()
        && options.activity
    {
        let model: Entity<yp_mods_ui::cutscenes::CutscenesModel> = view.read(cx).model().clone();
        let text = cx
            .t()
            .cutscene_copying_file_bytes(3, 6, "ev0030.usm", "412", "980");
        model.update(cx, |model, cx| model.set_activity(text, Some(42.), cx));
    }
    if let Ok(view) = tab.clone().downcast::<ThirdPartyView>() {
        view.update(cx, |view, cx| {
            if options.expand {
                view.toggle_expanded("FreeCamTool/freecam.dll", cx);
            }
        });
    }
    if options.toasts {
        let notifications = cx.stores().notifications.clone();
        notifications.update(cx, |store, cx| {
            for (index, (msg, severity)) in [
                (
                    yp_ui::Msg::ModInstalled {
                        id: "kimono_wax".to_owned(),
                    },
                    Severity::Success,
                ),
                (yp_ui::Msg::ModDropMisroutedCutscenes, Severity::Error),
                (yp_ui::Msg::ModBulkInstallNone, Severity::Warning),
            ]
            .into_iter()
            .enumerate()
            {
                store.push(
                    yp_state::Notice::new(
                        format!("preview_{index}"),
                        msg,
                        yp_state::NoticeKind::General,
                        severity,
                    ),
                    cx,
                );
            }
        });
    }
    if let Some(dialog) = &options.dialog {
        open_dialog(dialog, tab, drops, window, cx);
    }
}

fn main() {
    let options = Options::from_args();
    let sandbox = Sandbox::new();
    populate(&sandbox, options.scene);
    let drops = Drops {
        cutscene: sandbox.write_cutscene_drop("HD Cutscene Pack Part 2", 3),
        game_mod: sandbox.write_game_mod_drop("SpeedrunTimer v2", "timer.dll", 220_000),
    };
    let assets = Assets::with_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
    gpui_platform::application().with_assets(assets).run(move |cx| {
        yp_ui::init(cx);
        yp_state::init_with(sandbox.env.clone(), cx);
        yp_state::register_default_handlers(cx);
        yp_mods_ui::init(cx);
        cx.registry_mut().flows.set_model_preview(|request, _, cx| {
            let label = request
                .archives
                .dat
                .as_deref()
                .or(request.archives.dtt.as_deref())
                .and_then(|path| path.file_stem())
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            cx.new(|_| PreviewSlot {
                label: label.into(),
            })
            .into()
        });
        yp_ui::theme::set_theme(options.theme, cx);
        if let Some(locale) = options.locale {
            yp_ui::I18n::set(locale, cx);
        }
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let stores = cx.stores().clone();
        let game_dir = sandbox.game.root().to_path_buf();
        stores
            .session
            .update(cx, |session, cx| session.set_directory(game_dir, cx));
        stores.navigation.update(cx, |navigation, cx| {
            navigation.request_tab(options.tab, None, cx);
        });
        cx.set_global(Keep {
            _sandbox: sandbox,
            _busy_owner: None,
        });

        let bounds = Bounds::new(
            point(px(40.), px(40.)),
            size(px(options.width), px(options.height)),
        );
        let tab_id = options.tab;
        let built: std::rc::Rc<std::cell::RefCell<Option<AnyView>>> = Default::default();
        let slot = built.clone();
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
                    let tab = Registry::build_tab(tab_id, window, cx).expect("tab is registered");
                    *slot.borrow_mut() = Some(tab.clone());
                    let shell = cx.new(|cx| PreviewShell::new(tab, window, cx));
                    cx.new(|cx| Root::new(shell, window, cx))
                },
            )
            .expect("preview window");

        let tab = built.borrow().clone();
        let scene = options.scene;
        cx.spawn(async move |cx| {
            cx.background_executor().timer(SETTLE).await;
            if scene != Scene::Empty {
                let profiles = cx.update(|cx| cx.stores().profiles.clone());
                let first = profiles.read_with(cx, |profiles, _| profiles.active_name().to_owned());
                profiles
                    .update(cx, |profiles, cx| profiles.create("speedrun", cx))
                    .await;
                profiles
                    .update(cx, |profiles, cx| profiles.switch_to(&first, cx))
                    .await;
                cx.update(|cx| {
                    let stores = cx.stores().clone();
                    stores.default_mods.update(cx, |defaults, cx| {
                        defaults.set_default(
                            "mods/wax_alice/pl/pl000d",
                            Some(DefaultKind::OutfitConfig),
                            Some(20),
                            cx,
                        );
                    });
                    stores.disabled_mods.update(cx, |disabled, cx| {
                        disabled.set_disabled("mods/desert_fog", true, cx);
                        disabled.set_disabled("cutscenes/Ending E Remaster", true, cx);
                    });
                    stores.mod_names.update(cx, |names, cx| {
                        names.rename("nines_casual", Some("9S Casual Wear"), cx);
                    });
                });
                cx.background_executor().timer(SETTLE).await;
            }
            if let Some(tab) = tab {
                let window_handle: gpui::AnyWindowHandle = handle.into();
                window_handle
                    .update(cx, |_, window, cx| {
                        apply_state(&tab, &options, &drops, window, cx);
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
