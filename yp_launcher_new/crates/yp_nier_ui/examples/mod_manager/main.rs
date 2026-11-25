#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod capture;
mod catalog;
mod profile;
mod view;

use gpui::{
    AnyWindowHandle, App, AsyncApp, Bounds, KeyDownEvent, Keystroke, MouseButton, MouseDownEvent,
    MouseUpEvent, PlatformInput, TitlebarOptions, WindowBounds, WindowOptions, point, prelude::*,
    px, size,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use view::ModManager;
use yp_nier_ui::Focus;

async fn drive_frames(handle: AnyWindowHandle, duration: Duration, cx: &mut AsyncApp) {
    let start = Instant::now();
    loop {
        let last_frame = start.elapsed() >= duration;
        handle
            .update(cx, |_, w, cx| {
                w.simulate_next_frame(cx);
                w.refresh();
                w.draw(cx).clear(cx);
            })
            .expect("advance GPUI frame");
        if last_frame {
            break;
        }
        cx.background_executor()
            .timer(Duration::from_millis(16))
            .await;
    }
}

async fn first_frame(window: gpui::WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let handle = AnyWindowHandle::from(window);
    handle
        .update(cx, |_, w, _| w.resize(size(px(1280.), px(720.))))
        .expect("size preview");
    let started = Instant::now();
    let mut sized_frame = false;
    loop {
        cx.background_executor()
            .timer(Duration::from_millis(4))
            .await;
        let ready = handle
            .update(cx, |_, w, cx| {
                let size = w.viewport_size();
                let ready = size.width > px(100.) && size.height > px(100.);
                if ready {
                    w.refresh();
                    w.draw(cx).clear(cx);
                }
                ready
            })
            .unwrap();
        if (sized_frame && ready) || started.elapsed() > Duration::from_secs(1) {
            break;
        }
        sized_frame = ready;
    }
    window
        .update(cx, |view, w, cx| {
            view.opened_at = Instant::now();
            cx.notify();
            w.refresh();
        })
        .expect("begin preview opening");
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let flag = args.next();
    let profile = flag.as_deref() == Some(std::ffi::OsStr::new("--profile-effects"));
    let capture = flag.filter(|_| !profile).map(|flag| {
        assert_eq!(
            flag, "--capture",
            "expected --capture <image.png> [menu|dialog|loupe|grid]"
        );
        PathBuf::from(args.next().expect("expected output PNG path"))
    });
    let state = args.next().unwrap_or_default();
    let tabs = state == "tabs" || state == "tab-fill";
    let (keys, effect, expected_focus): (&[&'static str], _, _) = match state.to_str().unwrap_or("")
    {
        "" | "menu" | "reveal" | "startup" | "opening" | "closing" | "closing-skip" => {
            (&[], 0, Focus::Tabs)
        }
        "packages" => (&["left", "left", "enter"], 0, Focus::Rows),
        "loading" => (&["left", "left", "enter", "down"], 0, Focus::Rows),
        "saving" => (&["left", "left", "enter", "down", "down"], 0, Focus::Rows),
        "unit-data" | "model-view" => (&["left", "enter"], 0, Focus::Rows),
        "profiles" | "dropdown" | "field" => (&["right", "enter"], 0, Focus::Rows),
        "settings" | "resolution" | "large-slider" => {
            (&["right", "right", "enter"], 0, Focus::Rows)
        }
        "toggle" | "slider" | "backdrop" | "scrollbar" | "disabled" | "actions" => {
            (&["enter"], 0, Focus::Rows)
        }
        "tabs" | "tab-fill" => (&["enter"], 0, Focus::Rows),
        "dialog" | "dialog-close" => (&["enter", "enter"], 0, Focus::Dialog),
        "loupe" => (
            &["right", "right", "enter", "down", "enter"],
            1,
            Focus::Rows,
        ),
        "grid" => (
            &["right", "right", "enter", "down", "down", "enter"],
            2,
            Focus::Rows,
        ),
        "static" => (
            &["right", "right", "enter", "down", "down", "down", "enter"],
            3,
            Focus::Rows,
        ),
        "crt" => (
            &["right", "right", "enter", "up", "up", "enter"],
            4,
            Focus::Rows,
        ),
        "chromatic" => (&["right", "right", "enter", "up", "enter"], 5, Focus::Rows),
        other => panic!("unknown preview state {other}"),
    };
    let exit = args.next();

    let platform = gpui_platform::current_platform(false);
    yp_nier_ui::Theme::with_text_system(platform.text_system());
    gpui::Application::with_platform(platform).run(move |cx: &mut App| {
        let check_close = state == "closing" || state == "closing-skip";
        if check_close {
            cx.set_quit_mode(gpui::QuitMode::Explicit);
        }
        cx.on_window_closed(move |cx, _| {
            if !check_close && cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let initial_size = if capture.is_some() {
            size(px(640.), px(360.))
        } else {
            size(px(1280.), px(720.))
        };
        let bounds = Bounds::centered(None, initial_size, cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("YoRHa Protocol · Mod Manager".into()),
                        ..Default::default()
                    }),
                    show: !cfg!(windows) && capture.is_none() && !profile,
                    focus: !cfg!(windows) && capture.is_none() && !profile,
                    window_background: gpui::WindowBackgroundAppearance::Opaque,
                    ..Default::default()
                },
                |w, cx| cx.new(|cx| ModManager::new(w, cx)),
            )
            .expect("open GPUI preview");

        if profile {
            cx.spawn(async move |cx| {
                first_frame(window, cx).await;
                profile::effects(window, cx).await;
                cx.update(|cx| cx.quit());
            })
            .detach();
        } else if let Some(path) = capture {
            let handle = AnyWindowHandle::from(window);
            cx.spawn(async move |cx| {
                first_frame(window, cx).await;
                if state == "startup" {
                    capture::save_opening(handle, &path, cx);
                    return;
                }
                if state == "opening" {
                    capture::save_opening_sequence(window, &path, cx);
                    return;
                }
                drive_frames(handle, Duration::from_millis(1650), cx).await;
                if check_close {
                    capture::exercise_closing(window, &path, state == "closing-skip", cx).await;
                    return;
                }
                handle
                    .update(cx, |_, w, cx| {
                        for &key in keys {
                            w.dispatch_event(
                                PlatformInput::KeyDown(KeyDownEvent {
                                    keystroke: Keystroke::parse(key).unwrap(),
                                    is_held: false,
                                    prefer_character_input: false,
                                }),
                                cx,
                            );
                        }
                    })
                    .expect("set preview state");
                window
                    .update(cx, |view, _, _| {
                        assert_eq!(view.selection(), (effect, expected_focus))
                    })
                    .expect("preview selection");
                let settle = if state == "packages" { 1650 } else { 850 };
                drive_frames(handle, Duration::from_millis(settle), cx).await;
                let tab_frames = if tabs {
                    Some(capture::exercise_tabs(window, cx).await)
                } else {
                    None
                };
                let scenario = state.to_str().unwrap_or_default();
                capture::exercise_control(window, scenario, cx).await;
                capture::exercise_motion(handle, scenario, cx).await;
                let timeline = if scenario == "dialog-close" {
                    Some(capture::exercise_dialog(window, cx).await)
                } else if scenario == "tab-fill" {
                    tab_frames
                } else {
                    None
                };
                if scenario == "reveal" {
                    capture::exercise_reveal(window, cx).await;
                }
                if scenario == "resolution" {
                    capture::exercise_resolution(window, cx).await;
                }
                if scenario == "scrollbar" {
                    capture::exercise_scroll(window, cx).await;
                }
                if scenario == "large-slider" {
                    capture::exercise_large_slider(window, cx).await;
                }
                if matches!(scenario, "unit-data" | "model-view") {
                    capture::exercise_viewer(window, scenario == "model-view", cx).await;
                }
                if scenario == "actions" {
                    capture::exercise_actions(window, cx).await;
                }
                if scenario == "disabled" {
                    capture::exercise_disabled(window, cx).await;
                }
                if scenario == "field" {
                    capture::exercise_field(window, cx).await;
                }
                if let Some(exit) = exit {
                    handle
                        .update(cx, |_, w, cx| {
                            w.draw(cx).clear(cx);
                            match exit.to_str() {
                                Some("--back") => {
                                    w.dispatch_event(
                                        PlatformInput::KeyDown(KeyDownEvent {
                                            keystroke: Keystroke::parse("escape").unwrap(),
                                            is_held: false,
                                            prefer_character_input: false,
                                        }),
                                        cx,
                                    );
                                }
                                Some("--click-back") => {
                                    let viewport = w.viewport_size();
                                    let s = (viewport.width / px(1600.))
                                        .min(viewport.height / px(900.));
                                    let position = point(
                                        viewport.width / 2. + px(680. * s),
                                        viewport.height / 2. + px(343. * s),
                                    );
                                    w.dispatch_event(
                                        PlatformInput::MouseDown(MouseDownEvent {
                                            position,
                                            button: MouseButton::Left,
                                            modifiers: Default::default(),
                                            click_count: 1,
                                            first_mouse: false,
                                        }),
                                        cx,
                                    );
                                    w.dispatch_event(
                                        PlatformInput::MouseUp(MouseUpEvent {
                                            position,
                                            button: MouseButton::Left,
                                            modifiers: Default::default(),
                                            click_count: 1,
                                        }),
                                        cx,
                                    );
                                }
                                _ => panic!("expected --back or --click-back"),
                            }
                        })
                        .expect("dispatch Back");
                    window
                        .update(cx, |view, _, _| {
                            assert_eq!(view.selection(), (0, Focus::Rows))
                        })
                        .expect("effect exit");
                }
                handle
                    .update(cx, |_, w, cx| {
                        w.draw(cx).clear(cx);
                        let image = timeline
                            .unwrap_or_else(|| w.render_to_image().expect("read GPUI frame"));
                        assert!(
                            image.width() >= 640
                                && image.height()
                                    >= match scenario {
                                        "dialog-close" => 180,
                                        "tab-fill" => 30,
                                        _ => 360,
                                    },
                            "capture window has no render target"
                        );
                        image.save(&path).expect("write preview PNG");
                        cx.quit();
                    })
                    .expect("capture GPUI frame");
            })
            .detach();
        } else {
            cx.spawn(async move |cx| {
                first_frame(window, cx).await;
                window
                    .update(cx, |_, w, cx| {
                        w.refresh();
                        w.activate_window();
                        cx.activate(true);
                    })
                    .unwrap();
            })
            .detach();
        }
    });
}
