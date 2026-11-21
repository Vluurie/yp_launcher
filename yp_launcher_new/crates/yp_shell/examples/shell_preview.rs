use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::channel::oneshot;
use gpui::{App, AsyncApp};
use yp_config::cli::NamsCli;
use yp_config::testing::FakeNamsCli;
use yp_core::consts::{GAME_EXE_NAME, NAMS_EXE_NAME, NAMS_LOG_NAME, YORHA_DLL_NAME};
use yp_core::{GamePaths, NierInstallation, SettingKey};
use yp_host::launch::failure::{FailureKind, LaunchFailure, NamsReport};
use yp_host::session::NIER_DIRECTORY;
use yp_i18n::{Locale, Msg};
use yp_shell::onboarding::Step;
use yp_shell::sandbox::SandboxPlatform;
use yp_shell::{
    InstallChoice, InstallChoiceDialog, InstallDialogKind, LaunchFailureDialog, ShellHandle,
    ShellOptions,
};
use yp_state::prelude::*;
use yp_state::{AppEnv, Notice, NoticeKind, NoticeSeverity, SessionError};
use yp_ui::{ThemeId, WindowExt};

const SETTLE: Duration = Duration::from_millis(900);
const ONBOARDING_COMPLETE: SettingKey<bool> = SettingKey::new("onboarding_complete", || false);
const APP_THEME: SettingKey<String> = SettingKey::new("app_theme", String::new);
const APP_LOCALE: SettingKey<String> = SettingKey::new("app_locale", String::new);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Launcher,
    NoGame,
    Settings,
    Verify,
    Logs,
    Diagnostics,
    Failure,
    Toasts,
    Error,
    Warnings,
    Installs(InstallDialogKind),
    Tab(TabId),
    Onboarding(Step),
}

#[derive(Debug, Clone)]
struct PreviewArgs {
    dir: PathBuf,
    screen: Screen,
    theme: ThemeId,
    locale: Option<Locale>,
    size: (f32, f32),
    show: bool,
    missing_runtime: bool,
    unsupported: bool,
}

fn parse_size(text: &str) -> Option<(f32, f32)> {
    let (width, height) = text.split_once('x')?;
    Some((width.trim().parse().ok()?, height.trim().parse().ok()?))
}

fn parse_screen(text: &str) -> Option<Screen> {
    let screen = match text {
        "launcher" => Screen::Launcher,
        "nogame" => Screen::NoGame,
        "settings" => Screen::Settings,
        "verify" => Screen::Verify,
        "logs" => Screen::Logs,
        "diagnostics" => Screen::Diagnostics,
        "failure" => Screen::Failure,
        "toasts" => Screen::Toasts,
        "error" => Screen::Error,
        "warnings" => Screen::Warnings,
        "found" => Screen::Installs(InstallDialogKind::FastResults),
        "found-deep" => Screen::Installs(InstallDialogKind::DeepResults),
        "found-one" => Screen::Installs(InstallDialogKind::ConfirmSingle),
        "mods" => Screen::Tab(TabId::Mods),
        "nams" => Screen::Tab(TabId::Nams),
        "onboarding" => Screen::Onboarding(Step::Welcome),
        other => {
            let index: usize = other.strip_prefix("onboarding")?.parse().ok()?;
            Screen::Onboarding(Step::from_index(index)?)
        }
    };
    Some(screen)
}

fn parse_args() -> PreviewArgs {
    let mut parsed = PreviewArgs {
        dir: std::env::temp_dir().join("yp_shell_preview"),
        screen: Screen::Launcher,
        theme: ThemeId::Dark,
        locale: None,
        size: (1300., 750.),
        show: false,
        missing_runtime: false,
        unsupported: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dir" => {
                if let Some(dir) = args.next() {
                    parsed.dir = PathBuf::from(dir);
                }
            }
            "--screen" => {
                if let Some(screen) = args.next().as_deref().and_then(parse_screen) {
                    parsed.screen = screen;
                }
            }
            "--theme" => {
                if let Some(theme) = args.next().as_deref().and_then(ThemeId::from_code) {
                    parsed.theme = theme;
                }
            }
            "--locale" => {
                parsed.locale = args.next().as_deref().and_then(Locale::from_code);
            }
            "--size" => {
                if let Some(size) = args.next().as_deref().and_then(parse_size) {
                    parsed.size = size;
                }
            }
            "--show" => parsed.show = true,
            "--missing-runtime" => parsed.missing_runtime = true,
            "--unsupported" => parsed.unsupported = true,
            _ => {}
        }
    }
    parsed
}

fn write(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, content);
}

fn fake_log() -> String {
    let lines = [
        "2026-11-09T20:15:30.120000Z  INFO modloader::core: NAMS 3.1.0 starting",
        "2026-11-09T20:15:30.480000Z  INFO modloader::mods: loaded 3 mods from nams/mods",
        "2026-11-09T20:15:31.020000Z  WARN nams::textures: pack hd_faces overlaps pack world_4k in 12 textures, the higher one wins",
        "2026-11-09T20:15:31.300000Z DEBUG nier_rs::content::quests: registered 14 quest scripts",
        "2026-11-09T20:15:32.750000Z ERROR modloader::plugins: yorha_protocol.dll reported: could not hook the pause menu, retrying on the next frame so the overlay may appear late",
        "2026-11-09T20:15:33.010000Z  INFO yorha_protocol::workspace: workspace ready",
        "2026-11-09T20:15:33.400000Z  WARN yorha_protocol::randomizer: enemy table has 2 unknown ids",
        "2026-11-09T20:15:34.900000Z  INFO modloader::core: game window created",
    ];
    let mut text = String::new();
    for _ in 0..6 {
        for line in lines {
            text.push_str(line);
            text.push('\n');
        }
    }
    text
}

fn seed(env: &AppEnv, args: &PreviewArgs) -> GamePaths {
    let game = GamePaths::new(args.dir.join("game"));
    let mut exe = b"MZ".to_vec();
    exe.extend_from_slice(&[0u8; 64]);
    exe.extend_from_slice(b"PRJ_028");
    write(&game.exe(), &exe);
    write(&game.data_dir().join("data000.cpk"), b"cpk");
    if let Err(error) = std::fs::create_dir_all(game.nams_dir()) {
        eprintln!("could not create the preview nams folder: {error}");
    }
    for name in ["2b_kimono", "9s_casual_wear", "virtuous_contract_gold"] {
        write(
            &game.mods_dir().join(name).join("data").join("pl").join("pl000d.dat"),
            &[0u8; 32],
        );
        write(
            &game.mods_dir().join(name).join("data").join("pl").join("pl000d.dtt"),
            &[0u8; 32],
        );
    }
    for pack in ["hd_faces", "world_4k"] {
        write(
            &game.inject_textures_dir().join(pack).join("0a1b2c3d.dds"),
            b"DDS ",
        );
    }
    if args.missing_runtime {
        let _ = std::fs::remove_file(env.paths.nams_exe());
        let _ = std::fs::remove_file(env.paths.yorha_dll());
    } else {
        write(&env.paths.nams_exe(), b"MZ");
        write(&env.paths.yorha_dll(), b"MZ");
    }
    write(&env.paths.logs_dir.join(NAMS_LOG_NAME), fake_log().as_bytes());
    debug_assert_eq!(game.exe().file_name().unwrap(), GAME_EXE_NAME);
    debug_assert_eq!(env.paths.nams_exe().file_name().unwrap(), NAMS_EXE_NAME);
    debug_assert_eq!(env.paths.yorha_dll().file_name().unwrap(), YORHA_DLL_NAME);
    game
}

fn fake_failure() -> LaunchFailure {
    let mut failure = LaunchFailure::new(
        FailureKind::Reported(NamsReport {
            code: Some(20),
            panic: false,
            headline: "Steam is not running or no user is signed in".to_owned(),
            phase: Some("steam ownership check".to_owned()),
            os_error: Some("The system cannot find the file specified. (os error 2)".to_owned()),
            cause: Some("steam_api64.dll could not reach a Steam client".to_owned()),
            fix: Some("Start Steam, sign in, then press PLAY again.".to_owned()),
        }),
        "ERROR[20] Steam is not running or no user is signed in\n  Phase: steam ownership check\n  OS error: The system cannot find the file specified. (os error 2)\n  Likely cause: steam_api64.dll could not reach a Steam client\n  Try: Start Steam, sign in, then press PLAY again.",
    );
    failure.captured_log = None;
    failure
}

fn in_window(cx: &mut App, action: impl FnOnce(&mut gpui::Window, &mut App)) {
    let Some(handle) = cx.try_global::<ShellHandle>().cloned() else {
        return;
    };
    handle
        .window
        .update(cx, |_, window, cx| action(window, cx))
        .ok();
}

fn drive(screen: Screen, cx: &mut App) {
    match screen {
        Screen::Launcher | Screen::NoGame => {}
        Screen::Settings => in_window(cx, |window, cx| {
            window.dispatch_action(Box::new(OpenTab::new(TabId::LauncherSettings)), cx);
        }),
        Screen::Verify => in_window(cx, |window, cx| {
            window.dispatch_action(Box::new(OpenTab::new(TabId::LauncherSettings)), cx);
        }),
        Screen::Tab(tab) => in_window(cx, |window, cx| {
            window.dispatch_action(Box::new(OpenTab::new(tab)), cx);
        }),
        Screen::Logs => in_window(cx, |window, cx| {
            window.dispatch_action(Box::new(OpenLogs), cx);
        }),
        Screen::Diagnostics => in_window(cx, |window, cx| {
            window.dispatch_action(Box::new(OpenTab::new(TabId::LauncherSettings)), cx);
            window.dispatch_action(Box::new(OpenDiagnostics), cx);
        }),
        Screen::Failure => in_window(cx, |window, cx| {
            LaunchFailureDialog::open(fake_failure(), window, cx);
        }),
        Screen::Error => {
            let session = cx.stores().session.clone();
            session.update(cx, |session, cx| {
                session.set_error(SessionError::InvalidExe, cx);
            });
        }
        Screen::Warnings => {
            let stores = cx.stores().clone();
            stores.config.update(cx, |config, cx| {
                config.set_many(
                    vec![
                        ("nams.disable_plugin_loading".to_owned(), serde_json::Value::Bool(true)),
                        ("nams.disable_reshade_loading".to_owned(), serde_json::Value::Bool(true)),
                    ],
                    cx,
                );
            });
            stores.launch_options.update(cx, |options, cx| {
                options.set_show_launch_warnings(true, cx);
            });
        }
        Screen::Installs(kind) => in_window(cx, |window, cx| {
            let mut installs = vec![
                NierInstallation {
                    path: PathBuf::from("D:/SteamLibrary/steamapps/common/NieRAutomata"),
                    has_data: true,
                },
                NierInstallation {
                    path: PathBuf::from("E:/Games/NieRAutomata"),
                    has_data: true,
                },
                NierInstallation {
                    path: PathBuf::from("C:/Old/NieRAutomata"),
                    has_data: false,
                },
            ];
            if kind == InstallDialogKind::ConfirmSingle {
                installs.truncate(1);
            }
            let (sender, receiver) = oneshot::channel::<InstallChoice>();
            window.open_modal(cx, |_, cx| {
                InstallChoiceDialog::new(kind, installs, sender, cx)
            });
            cx.spawn(async move |_| {
                let _ = receiver.await;
            })
            .detach();
        }),
        Screen::Toasts => {
            let notifications = cx.stores().notifications.clone();
            let notices = [
                Notice::new(
                    "preview_shortcut",
                    Msg::NotifyShortcutCreated,
                    NoticeKind::Shortcut,
                    NoticeSeverity::Success,
                ),
                Notice::new(
                    "preview_command",
                    Msg::NotificationCommandNotReady,
                    NoticeKind::General,
                    NoticeSeverity::Error,
                ),
                Notice::new(
                    "preview_textures",
                    Msg::NotifyTexturesDetected {
                        folder: "SK_Res/".to_owned(),
                    },
                    NoticeKind::Textures,
                    NoticeSeverity::Accent,
                ),
            ];
            for notice in notices {
                notifications.update(cx, |store, cx| store.push(notice, cx));
            }
        }
        Screen::Onboarding(step) => {
            let Some(shell) = yp_shell::shell_view(cx) else {
                return;
            };
            in_window(cx, |window, cx| {
                shell.update(cx, |shell, cx| shell.show_onboarding_step(step, window, cx));
            });
        }
    }
}

fn main() {
    let args = parse_args();
    let assets =
        yp_ui::Assets::with_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
    gpui_platform::application()
        .with_assets(assets)
        .run(move |cx: &mut App| {
            yp_ui::init(cx);
            let env = match yp_shell::sandbox::sandbox_env(&args.dir) {
                Ok(env) => env,
                Err(error) => {
                    eprintln!("could not prepare the preview folder: {error}");
                    cx.quit();
                    return;
                }
            };
            let env = if args.unsupported {
                let platform = SandboxPlatform::new(env.platform.clone()).without_launch();
                env.with_platform(Arc::new(platform))
            } else {
                env
            };
            let game = seed(&env, &args);
            let with_game = args.screen != Screen::NoGame;
            let directory = if with_game {
                game.root().to_string_lossy().into_owned()
            } else {
                String::new()
            };
            let onboarding = matches!(args.screen, Screen::Onboarding(_));
            let _ = env.settings.set(&NIER_DIRECTORY, &directory);
            let _ = env.settings.set(&ONBOARDING_COMPLETE, &!onboarding);
            let _ = env.settings.set(&APP_THEME, &args.theme.code().to_owned());
            if let Some(locale) = args.locale {
                let _ = env.settings.set(&APP_LOCALE, &locale.code().to_owned());
            }
            yp_state::init_with(env, cx);
            let fake: Arc<dyn NamsCli> = Arc::new(FakeNamsCli::default());
            let config = cx.stores().config.clone();
            config.update(cx, |config, cx| {
                config.set_cli_factory(Arc::new(move |_| fake.clone()), cx);
            });
            yp_shell::init_with(
                ShellOptions {
                    focus_window: args.show,
                    persist_window: false,
                    window_size: Some(args.size),
                    ..ShellOptions::sandboxed(args.dir.clone())
                },
                cx,
            );
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            if let Err(error) = yp_shell::open_window(cx) {
                eprintln!("could not open the preview window: {error}");
                cx.quit();
                return;
            }
            let startup = yp_state::startup(cx);
            yp_shell::attach_startup(startup, cx);
            let screen = args.screen;
            cx.spawn(async move |cx: &mut AsyncApp| {
                cx.background_executor().timer(SETTLE).await;
                cx.update(|cx| drive(screen, cx));
            })
            .detach();
        });
}
