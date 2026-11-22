use gpui::{TestAppContext, WindowBounds, px};
use yp_core::window_geometry::{MIN_HEIGHT, MIN_WIDTH, WINDOW, WindowGeometry};
use yp_state::prelude::*;

use super::{Harness, open_main};
use crate::ShellOptions;
use crate::window;

fn windowed(bounds: WindowBounds) -> gpui::Bounds<gpui::Pixels> {
    match bounds {
        WindowBounds::Windowed(bounds) => bounds,
        other => panic!("expected a windowed placement, got {other:?}"),
    }
}

#[gpui::test]
fn saved_geometry_becomes_the_window_placement(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    cx.update(|cx| {
        let geometry = WindowGeometry {
            width: 1000.,
            height: 640.,
            x: Some(120.),
            y: Some(80.),
            maximized: false,
            fullscreen: false,
        };
        let bounds = windowed(window::bounds_for(geometry, cx));
        assert_eq!(bounds.origin.x, px(120.));
        assert_eq!(bounds.origin.y, px(80.));
        assert_eq!(bounds.size.width, px(1000.));
        assert_eq!(bounds.size.height, px(640.));

        let maximized = WindowGeometry {
            maximized: true,
            ..geometry
        };
        assert!(matches!(
            window::bounds_for(maximized, cx),
            WindowBounds::Maximized(_)
        ));
        let fullscreen = WindowGeometry {
            fullscreen: true,
            ..geometry
        };
        assert!(matches!(
            window::bounds_for(fullscreen, cx),
            WindowBounds::Fullscreen(_)
        ));
    });
}

#[gpui::test]
fn window_options_use_saved_size_unless_told_otherwise(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.init(cx);
    cx.update(|cx| {
        let fresh = window::window_options(&ShellOptions::default(), cx);
        let bounds = windowed(fresh.window_bounds.unwrap());
        assert_eq!(bounds.size, window::default_size());
        assert_eq!(fresh.window_min_size, Some(window::min_size()));
        assert!(fresh.show);

        cx.prefs().set(
            &WINDOW,
            &Some(WindowGeometry {
                width: 900.,
                height: 100.,
                ..WindowGeometry::default()
            }),
        );
        let restored = window::window_options(&ShellOptions::default(), cx);
        let bounds = windowed(restored.window_bounds.unwrap());
        assert_eq!(bounds.size.width, px(900.));
        assert_eq!(bounds.size.height, px(MIN_HEIGHT as f32));

        let ignoring = ShellOptions {
            persist_window: false,
            ..ShellOptions::default()
        };
        let bounds = windowed(window::window_options(&ignoring, cx).window_bounds.unwrap());
        assert_eq!(bounds.size, window::default_size());

        let sized = ShellOptions {
            window_size: Some((200., 2000.)),
            focus_window: false,
            ..ShellOptions::default()
        };
        let options = window::window_options(&sized, cx);
        let bounds = windowed(options.window_bounds.unwrap());
        assert_eq!(bounds.size.width, px(MIN_WIDTH as f32));
        assert_eq!(bounds.size.height, px(2000.));
        assert!(!options.show);
        assert!(!options.focus);
    });
}

#[gpui::test]
fn closing_saves_the_window_geometry(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;
    assert_eq!(cx.update(|_, cx| cx.prefs().get(&WINDOW)), None);

    let allowed = opened
        .shell
        .update_in(cx, |shell, window, cx| shell.should_close(window, cx));
    assert!(allowed);
    let saved = cx.update(|_, cx| cx.prefs().get(&WINDOW)).unwrap();
    assert!(saved.width >= MIN_WIDTH);
    assert!(saved.height >= MIN_HEIGHT);
    assert!(saved.x.is_some() && saved.y.is_some());
}

#[gpui::test]
fn window_moves_are_saved_after_a_pause(cx: &mut TestAppContext) {
    let harness = Harness::new();
    harness.start(cx);
    let opened = open_main(cx);
    let cx = opened.cx;

    cx.simulate_resize(gpui::size(px(1010.), px(700.)));
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| cx.prefs().get(&WINDOW)), None);
    cx.executor().advance_clock(window::SAVE_DELAY * 2);
    cx.run_until_parked();
    let saved = cx.update(|_, cx| cx.prefs().get(&WINDOW)).unwrap();
    assert_eq!(saved.width, 1010.);
    assert_eq!(saved.height, 700.);
}
