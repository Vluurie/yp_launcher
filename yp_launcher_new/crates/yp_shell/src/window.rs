use std::time::Duration;

use gpui::{
    App, Bounds, Pixels, Size, TitlebarOptions, Window, WindowBounds, WindowOptions, point, px,
    size,
};
use yp_core::window_geometry::{
    DEFAULT_HEIGHT, DEFAULT_WIDTH, DisplayArea, MIN_HEIGHT, MIN_WIDTH, WINDOW, WindowGeometry,
};
use yp_state::StateExt;
use yp_ui::ActiveStrings;

use crate::ShellOptions;

pub const APP_ID: &str = "yorha-protocol-launcher";
pub const SAVE_DELAY: Duration = Duration::from_millis(400);

pub fn min_size() -> Size<Pixels> {
    size(px(MIN_WIDTH as f32), px(MIN_HEIGHT as f32))
}

pub fn default_size() -> Size<Pixels> {
    size(px(DEFAULT_WIDTH as f32), px(DEFAULT_HEIGHT as f32))
}

pub fn capture(window: &Window) -> WindowGeometry {
    let (bounds, maximized, fullscreen) = match window.window_bounds() {
        WindowBounds::Windowed(bounds) => (bounds, false, false),
        WindowBounds::Maximized(bounds) => (bounds, true, false),
        WindowBounds::Fullscreen(bounds) => (bounds, false, true),
    };
    WindowGeometry {
        width: f64::from(f32::from(bounds.size.width)),
        height: f64::from(f32::from(bounds.size.height)),
        x: Some(f64::from(f32::from(bounds.origin.x))),
        y: Some(f64::from(f32::from(bounds.origin.y))),
        maximized,
        fullscreen,
    }
}

pub fn display_areas(cx: &App) -> Vec<DisplayArea> {
    cx.displays()
        .into_iter()
        .map(|display| {
            let bounds = display.bounds();
            DisplayArea {
                x: f64::from(f32::from(bounds.origin.x)),
                y: f64::from(f32::from(bounds.origin.y)),
                width: f64::from(f32::from(bounds.size.width)),
                height: f64::from(f32::from(bounds.size.height)),
            }
        })
        .collect()
}

pub fn bounds_for(geometry: WindowGeometry, cx: &App) -> WindowBounds {
    let extent = size(px(geometry.width as f32), px(geometry.height as f32));
    let bounds = match (geometry.x, geometry.y) {
        (Some(x), Some(y)) => Bounds::new(point(px(x as f32), px(y as f32)), extent),
        _ => Bounds::centered(None, extent, cx),
    };
    if geometry.fullscreen {
        WindowBounds::Fullscreen(bounds)
    } else if geometry.maximized {
        WindowBounds::Maximized(bounds)
    } else {
        WindowBounds::Windowed(bounds)
    }
}

pub fn saved_geometry(cx: &App) -> Option<WindowGeometry> {
    cx.prefs()
        .get(&WINDOW)
        .map(|geometry| geometry.sanitize(&display_areas(cx)))
}

pub fn save(window: &Window, cx: &App) {
    cx.prefs().set(&WINDOW, &Some(capture(window)));
}

pub fn window_options(options: &ShellOptions, cx: &App) -> WindowOptions {
    let window_bounds = match options.window_size {
        Some((width, height)) => {
            let extent = size(
                px(width.max(MIN_WIDTH as f32)),
                px(height.max(MIN_HEIGHT as f32)),
            );
            let center = Bounds::centered(None, extent, cx).center();
            WindowBounds::Windowed(Bounds::centered_at(center, extent))
        }
        None => match saved_geometry(cx).filter(|_| options.persist_window) {
            Some(geometry) => bounds_for(geometry, cx),
            None => WindowBounds::Windowed(Bounds::centered(None, default_size(), cx)),
        },
    };
    let native_title_bar = cx.env().platform.uses_native_title_bar();
    WindowOptions {
        window_bounds: Some(window_bounds),
        titlebar: Some(TitlebarOptions {
            title: Some(cx.t().app_title().into()),
            appears_transparent: !native_title_bar,
            traffic_light_position: None,
        }),
        focus: options.focus_window,
        show: options.focus_window,
        window_min_size: Some(min_size()),
        app_id: Some(APP_ID.to_owned()),
        ..Default::default()
    }
}
