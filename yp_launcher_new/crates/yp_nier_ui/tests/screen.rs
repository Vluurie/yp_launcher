#![cfg(target_os = "windows")]

use gpui::{
    AnyWindowHandle, AppContext, AsyncApp, Context, IntoElement, Render, Window, WindowOptions,
    deferred, div, prelude::*, px, relative, rgb, size,
};
use std::{cell::RefCell, rc::Rc, time::Duration};
use yp_nier_ui::{MenuWorkspace, ReducedMotion, menu_popup_overlay};

#[derive(Clone, Copy, PartialEq)]
enum Overlay {
    Off,
    Window,
    Popup,
}

struct Screen(Overlay);

impl Render for Screen {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let button = div()
            .id("control")
            .role(gpui::Role::Button)
            .size_full()
            .bg(rgb(0xcccccc));
        let popup = deferred(
            div()
                .absolute()
                .left(relative(0.025))
                .top(relative(0.35))
                .w(relative(0.95))
                .h(relative(0.3))
                .bg(rgb(0x888888))
                .when(self.0 == Overlay::Popup, |el| {
                    el.child(menu_popup_overlay())
                }),
        )
        .with_priority(9);
        MenuWorkspace::new(
            "screen",
            div().relative().size_full().child(button).child(popup),
        )
        .footer(div().w_full().h(px(36.)).bg(rgb(0xd8d8d8)))
        .screen_overlay(self.0 == Overlay::Window)
    }
}

#[test]
#[ignore = "requires a graphical desktop and GPU readback"]
fn grid_and_vignette_cover_controls_and_deferred_popups_and_both_borders_are_visible() {
    let captures = Rc::new(RefCell::new(Vec::new()));
    let frames = captures.clone();
    gpui_platform::application().run(move |app| {
        app.set_global(ReducedMotion(true));
        let handle = app
            .open_window(
                WindowOptions {
                    show: false,
                    focus: false,
                    ..Default::default()
                },
                |_, cx| cx.new(|_| Screen(Overlay::Off)),
            )
            .unwrap();
        app.spawn(async move |app| {
            let window = AnyWindowHandle::from(handle);
            for (width, height) in [(600., 420.), (1200., 720.)] {
                let extent = size(px(width), px(height));
                window.update(app, |_, w, _| w.resize(extent)).unwrap();
                for _ in 0..100 {
                    app.background_executor()
                        .timer(Duration::from_millis(10))
                        .await;
                    if window
                        .update(app, |_, w, _| w.viewport_size() == extent)
                        .unwrap()
                    {
                        break;
                    }
                }
                let capture = |app: &mut AsyncApp, enabled| {
                    handle
                        .update(app, |screen, _, cx| {
                            screen.0 = enabled;
                            cx.notify();
                        })
                        .unwrap();
                    window
                        .update(app, |_, window, cx| {
                            window.refresh();
                            window.draw(cx).clear(cx);
                            window.render_to_image().unwrap()
                        })
                        .unwrap()
                };
                let plain = capture(app, Overlay::Off);
                let filtered = capture(app, Overlay::Window);
                let popup = capture(app, Overlay::Popup);
                frames.borrow_mut().push((width, plain, filtered, popup));
            }
            app.update(|cx| cx.quit());
        })
        .detach();
    });
    assert_eq!(captures.borrow().len(), 2);
    for (width, plain, filtered, popup) in captures.take() {
        let (w, h) = plain.dimensions();
        assert!(
            w >= width as u32 - 32 && h >= 360,
            "unexpected capture size {w}x{h}"
        );
        let shade =
            |x, y| i16::from(plain.get_pixel(x, y)[0]) - i16::from(filtered.get_pixel(x, y)[0]);
        for (y, color) in [(h / 4, 204), (h / 2, 136)] {
            assert_eq!(plain.get_pixel(w / 2, y)[0], color);
            let mut max_shade = 0;
            let mut min_shade = i16::MAX;
            for row in y..y + 24 {
                for x in w / 2..w / 2 + 24 {
                    max_shade = max_shade.max(shade(x, row));
                    min_shade = min_shade.min(shade(x, row));
                }
            }
            assert!(
                max_shade - min_shade >= 4,
                "grid is missing on {color:#x} at {width}px"
            );
        }
        assert!(
            shade(3, h / 2) >= 20,
            "missing lens shading on the left edge"
        );
        assert!(
            shade(w - 4, h / 2) >= 20,
            "missing lens shading on the right edge"
        );
        assert_eq!(plain.get_pixel(w / 20, h / 2)[0], 136);
        assert!(
            shade(w / 20, h / 2) >= 9,
            "missing lens shading on the popup"
        );
        for x in w / 20..w * 19 / 20 {
            let local = popup.get_pixel(x, h / 2);
            let full = filtered.get_pixel(x, h / 2);
            assert!(
                (0..3).all(|c| local[c].abs_diff(full[c]) <= 1),
                "misaligned popup at x={x}: {local:?} vs {full:?}"
            );
        }
        let band = (14. * w as f32 / width).ceil() as u32;
        let footer_height = (36. * w as f32 / width).round() as u32;
        assert_eq!(plain.get_pixel(w / 2, h - 3)[0], 216);
        for top in [0, h - footer_height - band] {
            assert!(
                (top..top + band).any(|y| {
                    (w / 4..w * 3 / 4)
                        .filter(|&x| plain.get_pixel(x, y)[0] < 145)
                        .count()
                        > w as usize / 4
                }),
                "missing border at y={top} on the {width}px window"
            );
        }
    }
}
