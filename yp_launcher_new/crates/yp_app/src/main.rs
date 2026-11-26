#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod args;
mod boot;

use gpui::App;
use yp_state::StateExt;

fn init_features(cx: &mut App) {
    #[cfg(feature = "settings")]
    yp_settings_ui::init(cx);
    #[cfg(feature = "mods")]
    yp_mods_ui::init(cx);
    #[cfg(any(feature = "mods", feature = "docs"))]
    yp_docs_ui::init_docs(cx);
    #[cfg(feature = "tools")]
    yp_docs_ui::init_tools(cx);
    let _ = cx;
}

fn main() {
    let args = args::Args::from_process();
    let boot = boot::boot(&args);
    let platform = gpui_platform::current_platform(false);
    yp_ui::nier::Theme::with_text_system(platform.text_system());
    gpui::Application::with_platform(platform)
        .with_assets(yp_ui::Assets::with_folder(&boot.env.paths.assets_dir))
        .run(move |cx: &mut App| {
            yp_ui::init(cx);
            if let Err(error) =
                yp_core::logging::init_logging(&boot.env.paths.data_dir, log::LevelFilter::Info)
            {
                eprintln!("launcher log is unavailable: {error}");
            }
            let focus = boot.options.focus_window;
            yp_state::init_with(boot.env, cx);
            yp_shell::init_with(boot.options, cx);
            init_features(cx);
            let notifications = cx.stores().notifications.clone();
            for notice in boot.notices {
                notifications.update(cx, |store, cx| store.push(notice, cx));
            }
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            if let Err(error) = yp_shell::open_window(cx) {
                log::error!("could not open the launcher window: {error}");
                cx.quit();
                return;
            }
            let startup = yp_state::startup(cx);
            yp_shell::attach_startup(startup, cx);
            if focus {
                cx.activate(true);
            }
        });
}
