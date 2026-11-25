use super::{drive_frames, view::ModManager};
use gpui::{
    AnyWindowHandle, App, AsyncApp, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PlatformInput, Window, WindowHandle, point, px,
};
use std::time::{Duration, Instant};
use yp_nier_ui::Focus;

pub(super) fn save_opening(handle: AnyWindowHandle, path: &std::path::Path, cx: &mut AsyncApp) {
    let first = frame(handle, cx);
    assert!(first.width() >= 640 && first.height() >= 360);
    let center = first.get_pixel(first.width() / 2, first.height() / 2);
    let edge = first.get_pixel(first.width() / 100, first.height() / 2);
    assert!(
        center[3] > 245
            && (85..145).contains(&center[0])
            && u16::from(edge[0]) + 8 < u16::from(center[0]),
        "expected the opaque opening background and shaded edges: {center:?} vs {edge:?}"
    );
    first.save(path).expect("write first frame");
    cx.update(|cx| cx.quit());
}

pub(super) fn save_opening_sequence(
    window: WindowHandle<ModManager>,
    path: &std::path::Path,
    cx: &mut AsyncApp,
) {
    let mut strip = image::RgbaImage::new(400 * 5, 225);
    let mut coverage = Vec::new();
    for (index, tick) in [0, 25, 30, 36, 43].into_iter().enumerate() {
        window
            .update(cx, |view, _, cx| {
                view.opened_at = Instant::now() - Duration::from_secs_f64(f64::from(tick) / 60.);
                cx.notify();
            })
            .unwrap();
        let sample = frame(window.into(), cx);
        let line = |y| {
            (450..1400)
                .map(|x| luminance(&sample, x as f32, y) > 155.)
                .collect::<Vec<_>>()
        };
        let upper = line(133.);
        coverage.push(upper.iter().filter(|&&light| light).count());
        if tick == 30 {
            let lower = line(168.);
            let slopes = upper.iter().zip(lower).filter(|(a, b)| **a != *b).count();
            assert!(
                slopes > 45,
                "expected diagonal triangle edges in the middle of the opening"
            );
        }
        let small =
            image::imageops::resize(&sample, 400, 225, image::imageops::FilterType::Triangle);
        image::imageops::replace(&mut strip, &small, index as i64 * 400, 0);
    }
    assert!(
        coverage[0] < 10 && coverage[4] > 900 && coverage[1] < coverage[3],
        "triangle coverage {coverage:?}"
    );
    strip.save(path).expect("write opening sequence");
    cx.update(|cx| cx.quit());
}

pub(super) async fn exercise_closing(
    window: WindowHandle<ModManager>,
    path: &std::path::Path,
    skip: bool,
    cx: &mut AsyncApp,
) {
    let handle = AnyWindowHandle::from(window);
    handle
        .update(cx, |_, w, cx| keystroke(w, cx, "escape"))
        .unwrap();
    drive_frames(handle, Duration::from_millis(140), cx).await;
    let mid = frame(handle, cx);
    let light = (450..1400)
        .filter(|&x| luminance(&mid, x as f32, 140.) > 155.)
        .count();
    assert!(
        (30..925).contains(&light),
        "expected a partially covered menu while closing: {light}"
    );
    if skip {
        handle.update(cx, |_, w, cx| keystroke(w, cx, "k")).unwrap();
    } else {
        cx.background_executor()
            .timer(yp_nier_ui::MenuFrame::CLOSE_DURATION)
            .await;
    }
    assert!(
        window.update(cx, |_, _, _| ()).is_err(),
        "menu did not close"
    );
    mid.save(path).expect("write closing frame");
    cx.update(|cx| cx.quit());
}

fn keystroke(w: &mut Window, cx: &mut App, key: &'static str) {
    w.dispatch_event(
        PlatformInput::KeyDown(gpui::KeyDownEvent {
            keystroke: gpui::Keystroke::parse(key).unwrap(),
            is_held: false,
            prefer_character_input: false,
        }),
        cx,
    );
}

pub(super) async fn exercise_control(
    window: WindowHandle<ModManager>,
    name: &str,
    cx: &mut AsyncApp,
) {
    let handle = AnyWindowHandle::from(window);
    let (x, y) = match name {
        "toggle" => (875., 543.),
        "slider" => (875., 607.),
        "dropdown" => (875., 445.),
        _ => return,
    };
    let click = |cx: &mut AsyncApp| {
        handle
            .update(cx, |_, w, cx| pointer(w, cx, x, y, true))
            .unwrap();
    };
    let key = |cx: &mut AsyncApp, k| {
        handle.update(cx, |_, w, cx| keystroke(w, cx, k)).unwrap();
    };
    let values = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.settings_values()).unwrap();
    let before = values(cx);
    let closed = frame(handle, cx);
    let next = if name == "slider" { "right" } else { "down" };
    click(cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    if name != "slider" {
        check_dim(&closed, &frame(handle, cx));
    }
    if name == "dropdown" {
        key(cx, "up");
        key(cx, "enter");
        assert_eq!(
            values(cx),
            before,
            "keyboard applied the unavailable profile"
        );
        handle
            .update(cx, |_, w, cx| pointer(w, cx, 1400., 595., true))
            .unwrap();
        assert_eq!(values(cx), before, "mouse applied the unavailable profile");
        check_dim(&closed, &frame(handle, cx));
        key(cx, "down");
    }
    key(cx, next);
    key(cx, "escape");
    if name == "slider" {
        assert_eq!(values(cx).2, before.2 + 1, "Back did not apply the slider");
    } else {
        assert_eq!(values(cx), before, "cancel changed {name}");
    }
    drive_frames(handle, Duration::from_millis(220), cx).await;
    click(cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    key(cx, next);
    key(cx, "enter");
    let after = values(cx);
    match name {
        "toggle" => assert_eq!(after.0, !before.0, "toggle did not commit"),
        "dropdown" => assert_eq!(after.1, before.1 + 1, "dropdown did not commit"),
        "slider" => assert_eq!(after.2, before.2 + 2, "slider did not commit"),
        _ => unreachable!(),
    }
    drive_frames(handle, Duration::from_millis(250), cx).await;
    click(cx);
    drive_frames(handle, Duration::from_millis(360), cx).await;
    if name == "slider" {
        let aim = |x, click, cx: &mut AsyncApp| {
            handle
                .update(cx, |_, w, cx| pointer(w, cx, x, 607., click))
                .unwrap();
        };
        aim(1239., false, cx);
        drive_frames(handle, Duration::from_millis(270), cx).await;
        let low = frame(handle, cx);
        aim(1295., false, cx);
        drive_frames(handle, Duration::from_millis(270), cx).await;
        let high = frame(handle, cx);
        let fill = |im: &image::RgbaImage| {
            let sx = im.width() as f32 / 1600.;
            let y = (602. * im.height() as f32 / 900.) as u32;
            ((1249. * sx) as u32..(1293. * sx) as u32)
                .filter(|&x| im.get_pixel(x, y)[0] > 135)
                .count()
        };
        assert!(
            fill(&high) > fill(&low) + 8,
            "hover without a pressed button did not fill the segments"
        );
        assert_eq!(
            values(cx).2,
            after.2,
            "hover committed before the second click"
        );
        aim(1295., true, cx);
        assert_eq!(
            values(cx).2,
            9,
            "second click did not apply the hovered segment"
        );
        drive_frames(handle, Duration::from_millis(250), cx).await;
        let compact = frame(handle, cx);
        assert!(
            fill(&compact) < fill(&high),
            "the segment group did not retract after the second click"
        );
        click(cx);
        drive_frames(handle, Duration::from_millis(220), cx).await;
        aim(1231., false, cx);
        aim(1214., false, cx);
        click(cx);
        assert_eq!(
            values(cx).2,
            0,
            "leaving the first segment to the left did not select zero"
        );
        drive_frames(handle, Duration::from_millis(200), cx).await;
        click(cx);
        drive_frames(handle, Duration::from_millis(220), cx).await;
        aim(1279., false, cx);
        drive_frames(handle, Duration::from_millis(400), cx).await;
        let img = frame(handle, cx);
        let x = |dx: f32| ((1231. + dx) * img.width() as f32 / 1600.) as u32;
        let y = (607. * img.height() as f32 / 900.) as u32;
        let lit = (x(-3.)..x(78.))
            .filter(|&col| img.get_pixel(col, y)[0] > 135)
            .count();
        assert!(
            lit >= 21,
            "expanded slider is missing the segments on the row centerline ({lit} light pixels)"
        );
        let active = (x(45.)..=x(51.))
            .map(|col| img.get_pixel(col, y)[0])
            .max()
            .unwrap();
        assert!(active >= 190, "active segment is too dark ({active})");
        let physical_y = |design: f32| (design * img.height() as f32 / 900.) as u32;
        for yy in [591., 623.] {
            assert!(
                (x(-3.)..x(79.)).all(|xx| img.get_pixel(xx, physical_y(yy))[0] < 145),
                "unrequested marker above or below the slider"
            );
        }
        for yy in [582., 632.] {
            assert!(
                (x(128.)..x(138.)).all(|xx| img.get_pixel(xx, physical_y(yy))[0] > 145),
                "collapsed nine-slice borders left a dash outside the row"
            );
        }
    }
}

pub(super) async fn exercise_viewer(
    window: WindowHandle<ModManager>,
    expanded: bool,
    cx: &mut AsyncApp,
) {
    let handle = AnyWindowHandle::from(window);
    let state = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.viewer_state()).unwrap();
    let key = |k, cx: &mut AsyncApp| handle.update(cx, |_, w, cx| keystroke(w, cx, k)).unwrap();
    let click = |x, y, cx: &mut AsyncApp| {
        handle
            .update(cx, |_, w, cx| pointer(w, cx, x, y, true))
            .unwrap()
    };
    let detail = frame(handle, cx);
    assert!(!state(cx).0);
    click(812., 480., cx);
    assert!(state(cx).0);
    drive_frames(handle, Duration::from_millis(540), cx).await;
    let full = frame(handle, cx);
    assert!(
        luminance(&full, 600., 300.) + 40. < luminance(&detail, 600., 300.),
        "model viewport did not expand"
    );
    let yaw = state(cx).4;
    drag(handle, (810., 440.), (899., 475.), cx).await;
    assert!((state(cx).4 - yaw).abs() > 0.1, "model drag did not orbit");
    key("enter", cx);
    key("tab", cx);
    assert_eq!((state(cx).1, state(cx).2), (1, true));
    key("down", cx);
    assert_eq!((state(cx).1, state(cx).3), (0, 1));
    key("up", cx);
    key("escape", cx);
    assert!(!state(cx).0);
    drive_frames(handle, Duration::from_millis(470), cx).await;
    let returned = frame(handle, cx);
    assert!(luminance(&returned, 600., 300.) > luminance(&full, 600., 300.) + 40.);
    key("enter", cx);
    drive_frames(handle, Duration::from_millis(520), cx).await;
    assert!(state(cx).0);
    click(1490., 810., cx);
    drive_frames(handle, Duration::from_millis(470), cx).await;
    assert!(
        !state(cx).0,
        "viewer Back prompt did not return to Unit Data"
    );
    if expanded {
        key("enter", cx);
        drive_frames(handle, Duration::from_millis(520), cx).await;
    }
}

pub(super) async fn exercise_dialog(
    window: WindowHandle<ModManager>,
    cx: &mut AsyncApp,
) -> image::RgbaImage {
    let handle = AnyWindowHandle::from(window);
    let focus = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.selection().1).unwrap();
    let enabled = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.settings_values().0).unwrap();
    let key = |k, cx: &mut AsyncApp| handle.update(cx, |_, w, cx| keystroke(w, cx, k)).unwrap();
    let opened = frame(handle, cx);
    let before = enabled(cx);
    key("escape", cx);
    assert_eq!(focus(cx), Focus::Dialog);
    key("enter", cx);
    key("right", cx);
    let mut samples = Vec::new();
    for _ in 0..7 {
        let im = frame(handle, cx);
        samples.push(im);
        if focus(cx) == Focus::Rows {
            break;
        }
        cx.background_executor()
            .timer(Duration::from_millis(5))
            .await;
    }
    drive_frames(handle, Duration::from_millis(240), cx).await;
    let closed = frame(handle, cx);
    assert_eq!(focus(cx), Focus::Rows);
    assert_eq!(enabled(cx), before);
    let point = |im| luminance(im, 785., 167.);
    let intermediate: Vec<_> = samples
        .iter()
        .map(|im| (point(im), luminance(im, 800., 308.)))
        .collect();
    let between = |v: f32, a: f32, b: f32| v > a.min(b) + 0.4 && v < a.max(b) - 0.4;
    assert!(
        intermediate
            .iter()
            .any(|&(v, _)| between(v, point(&opened), point(&closed))),
        "backdrop samples {intermediate:?}: {} -> {}",
        point(&opened),
        point(&closed)
    );
    let (a, b) = (
        luminance(&opened, 800., 308.),
        luminance(&closed, 800., 308.),
    );
    assert!(
        intermediate.iter().any(|&(_, v)| between(v, a, b)),
        "dialog face samples {intermediate:?}: {a} -> {b}"
    );
    key("enter", cx);
    drive_frames(handle, Duration::from_millis(420), cx).await;
    handle
        .update(cx, |_, w, cx| pointer(w, cx, 662., 548., true))
        .unwrap();
    assert_eq!(focus(cx), Focus::Dialog);
    assert_eq!(enabled(cx), before);
    drive_frames(handle, Duration::from_millis(250), cx).await;
    assert_eq!(focus(cx), Focus::Rows);
    assert_ne!(enabled(cx), before);
    let midpoint = (point(&closed) + point(&opened)) * 0.5;
    let middle = samples
        .iter()
        .min_by(|a, b| {
            (point(*a) - midpoint)
                .abs()
                .total_cmp(&(point(*b) - midpoint).abs())
        })
        .unwrap();
    let w = opened.width();
    let h = opened.height();
    let (cw, ch) = (w / 2, h / 2);
    let mut strip = image::RgbaImage::new(3 * cw, ch);
    for (i, im) in [&opened, middle, &closed].into_iter().enumerate() {
        let panel = image::imageops::crop_imm(im, w / 4, h / 4, cw, ch).to_image();
        image::imageops::replace(&mut strip, &panel, i as i64 * i64::from(cw), 0);
    }
    strip
}

pub(super) async fn exercise_reveal(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let h = AnyWindowHandle::from(window);
    let switch = |k, cx: &mut AsyncApp| h.update(cx, |_, w, cx| keystroke(w, cx, k)).unwrap();
    let extent = |im: &image::RgbaImage, [left, top, right, bottom]: [f32; 4]| {
        let (sx, sy) = (im.width() as f32 / 1600., im.height() as f32 / 900.);
        ((left * sx) as u32..(right * sx) as u32)
            .rev()
            .find(|&x| {
                ((top * sy) as u32..(bottom * sy) as u32).any(|y| im.get_pixel(x, y)[0] < 115)
            })
            .unwrap_or(0)
    };
    let title = [53., 131., 440., 180.];
    let first_row = [140., 240., 420., 273.];
    switch("right", cx);
    drive_frames(h, Duration::from_millis(185), cx).await;
    let partial = extent(&frame(h, cx), title);
    drive_frames(h, Duration::from_millis(420), cx).await;
    let partial_row = extent(&frame(h, cx), first_row);
    drive_frames(h, Duration::from_millis(630), cx).await;
    let settled = frame(h, cx);
    let full = extent(&settled, title);
    let full_row = extent(&settled, first_row);
    assert!(
        partial_row > 0 && partial_row + 12 < full_row,
        "opening row reveal width {partial_row} -> {full_row}"
    );
    assert!(
        partial > 0 && partial + 25 < full,
        "title reveal width {partial} -> {full}"
    );
    switch("left", cx);
    drive_frames(h, Duration::from_millis(260), cx).await;
    switch("right", cx);
    drive_frames(h, Duration::from_millis(210), cx).await;
    let restarted = extent(&frame(h, cx), title);
    assert!(
        restarted > 0 && restarted + 20 < full,
        "switching tabs did not replay the title"
    );
}

fn check_dim(closed: &image::RgbaImage, open: &image::RgbaImage) {
    for (x, y) in [(785., 167.), (52., 765.)] {
        let ratio = luminance(open, x, y) / luminance(closed, x, y);
        assert!(
            (0.75..0.85).contains(&ratio),
            "modal backdrop at ({x}, {y}) has luminance ratio {ratio}"
        );
    }
}

pub(super) async fn exercise_resolution(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let handle = AnyWindowHandle::from(window);
    let click = |x, y, cx: &mut AsyncApp| {
        handle
            .update(cx, |_, w, cx| pointer(w, cx, x, y, true))
            .unwrap()
    };
    let key = |k, cx: &mut AsyncApp| handle.update(cx, |_, w, cx| keystroke(w, cx, k)).unwrap();
    let value = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.resolution()).unwrap();
    let initial = value(cx);
    let closed = frame(handle, cx);
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(350), cx).await;
    let opened = frame(handle, cx);
    check_dim(&closed, &opened);
    key("down", cx);
    key("escape", cx);
    assert_eq!(value(cx), initial);
    drive_frames(handle, Duration::from_millis(240), cx).await;
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    check_dim(&closed, &frame(handle, cx));
    key("down", cx);
    key("enter", cx);
    assert_eq!(value(cx), initial + 1);
    drive_frames(handle, Duration::from_millis(240), cx).await;
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    key("up", cx);
    key("enter", cx);
    assert_eq!(value(cx), initial);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(240), cx).await;
    drag(handle, (1229., 665.), (1229., 310.), cx).await;
    key("enter", cx);
    assert_eq!(
        value(cx),
        0,
        "the compact scrollbar did not drag to the first option"
    );
    drive_frames(handle, Duration::from_millis(220), cx).await;
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    handle
        .update(cx, |_, w, cx| {
            let position = point_at(w, 1229., 310.);
            pointer(w, cx, 1229., 310., false);
            w.dispatch_event(
                PlatformInput::Scroll(gpui::ScrollEvent {
                    position,
                    delta: gpui::ScrollDelta::Lines(point(0., -1.)),
                    ..Default::default()
                }),
                cx,
            );
        })
        .unwrap();
    key("enter", cx);
    assert_eq!(value(cx), 1, "wheel did not advance the option");
    drive_frames(handle, Duration::from_millis(220), cx).await;
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    click(360., 50., cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    assert_eq!(value(cx), 1);
    assert_eq!(
        window.update(cx, |v, _, _| v.page()).unwrap(),
        2,
        "outside click reached the tab below"
    );
    let restored = frame(handle, cx);
    let before = luminance(&closed, 785., 167.);
    let after = luminance(&restored, 785., 167.);
    assert!(
        after > before * 0.91,
        "outside click left the backdrop visible ({before} -> {after})"
    );
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    for _ in 1..initial {
        key("down", cx);
    }
    key("enter", cx);
    assert_eq!(value(cx), initial);
    drive_frames(handle, Duration::from_millis(220), cx).await;
    click(875., 262., cx);
    drive_frames(handle, Duration::from_millis(400), cx).await;
}

pub(super) async fn exercise_large_slider(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let h = AnyWindowHandle::from(window);
    let aim = |x, click, cx: &mut AsyncApp| {
        h.update(cx, |_, w, cx| pointer(w, cx, x, 390., click))
            .unwrap()
    };
    aim(875., true, cx);
    drive_frames(h, Duration::from_millis(250), cx).await;
    aim(1236., false, cx);
    drive_frames(h, Duration::from_millis(320), cx).await;
    let image = frame(h, cx);
    for y in [374., 406.] {
        let bright = |x: f32| {
            let sx = image.width() as f32 / 1600.;
            let yy = (y * image.height() as f32 / 900.) as u32;
            ((x * sx) as u32..=((x + 6.) * sx) as u32)
                .map(|xx| image.get_pixel(xx, yy)[0])
                .max()
                .unwrap()
        };
        assert!(
            bright(1192.) > bright(1233.) + 45,
            "the default indicator moved with the current value"
        );
    }
    aim(1236., true, cx);
    assert_eq!(window.update(cx, |v, _, _| v.rotation_speed()).unwrap(), 13);
    drive_frames(h, Duration::from_millis(250), cx).await;
    aim(875., true, cx);
    drive_frames(h, Duration::from_millis(300), cx).await;
}

pub(super) async fn exercise_actions(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let h = AnyWindowHandle::from(window);
    let click =
        |x, y, cx: &mut AsyncApp| h.update(cx, |_, w, cx| pointer(w, cx, x, y, true)).unwrap();
    let key = |k, cx: &mut AsyncApp| h.update(cx, |_, w, cx| keystroke(w, cx, k)).unwrap();
    let values = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.settings_values()).unwrap();
    let before = values(cx);
    click(830., 665., cx);
    drive_frames(h, Duration::from_millis(280), cx).await;
    key("down", cx);
    key("enter", cx);
    assert_eq!(values(cx), before);
    key("up", cx);
    key("enter", cx);
    assert_eq!(values(cx).0, !before.0);
    drive_frames(h, Duration::from_millis(230), cx).await;
    click(830., 665., cx);
    drive_frames(h, Duration::from_millis(230), cx).await;
    click(1170., 723., cx);
    assert_eq!(values(cx), before);
    drive_frames(h, Duration::from_millis(230), cx).await;
    click(830., 665., cx);
    drive_frames(h, Duration::from_millis(320), cx).await;
}

pub(super) async fn exercise_disabled(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let h = AnyWindowHandle::from(window);
    let point = |x, y, click, cx: &mut AsyncApp| {
        h.update(cx, |_, w, cx| pointer(w, cx, x, y, click))
            .unwrap()
    };
    let values = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.settings_values()).unwrap();
    let toggle = |cx: &mut AsyncApp| {
        h.update(cx, |_, w, cx| {
            keystroke(w, cx, "down");
            keystroke(w, cx, "enter");
        })
        .unwrap()
    };
    let normal = frame(h, cx);
    let original = values(cx);
    point(875., 543., true, cx);
    drive_frames(h, Duration::from_millis(230), cx).await;
    toggle(cx);
    point(1530., 160., false, cx);
    drive_frames(h, Duration::from_millis(280), cx).await;
    let disabled = frame(h, cx);
    let contrast = |img: &image::RgbaImage| {
        let sx = img.width() as f32 / 1600.;
        let sy = img.height() as f32 / 900.;
        let mut values = Vec::new();
        for y in (594. * sy) as u32..(620. * sy) as u32 {
            for x in (687. * sx) as u32..(814. * sx) as u32 {
                values.push(img.get_pixel(x, y)[0]);
            }
        }
        values.sort_unstable();
        f32::from(values[values.len() * 95 / 100] - values[values.len() * 5 / 100])
    };
    let ratio = contrast(&disabled) / contrast(&normal);
    assert!(
        (0.35..0.7).contains(&ratio),
        "disabled row contrast ratio {ratio}"
    );
    point(875., 607., true, cx);
    point(1295., 607., false, cx);
    point(1295., 607., true, cx);
    assert_eq!(values(cx), (false, original.1, original.2));
    point(1530., 160., false, cx);
    drive_frames(h, Duration::from_millis(250), cx).await;
}

pub(super) async fn exercise_motion(handle: AnyWindowHandle, name: &str, cx: &mut AsyncApp) {
    let (left, top, right, bottom) = match name {
        "backdrop" => (300., 100., 650., 218.),
        "loading" | "saving" => (1050., 472., 1100., 522.),
        _ => return,
    };
    let a = frame(handle, cx);
    for wait in [120, 175, 265] {
        drive_frames(handle, Duration::from_millis(wait), cx).await;
        let b = frame(handle, cx);
        let sx = b.width() as f32 / 1600.;
        let sy = b.height() as f32 / 900.;
        let mut changed = 0;
        for y in (top * sy) as u32..(bottom * sy) as u32 {
            for x in (left * sx) as u32..(right * sx) as u32 {
                if a.get_pixel(x, y)[0].abs_diff(b.get_pixel(x, y)[0]) > 5 {
                    changed += 1;
                }
            }
        }
        if changed > 20 {
            return;
        }
    }
    panic!("{name} stayed stationary between rendered frames");
}

async fn drag(handle: AnyWindowHandle, from: (f32, f32), to: (f32, f32), cx: &mut AsyncApp) {
    handle
        .update(cx, |_, w, cx| {
            let position = point_at(w, from.0, from.1);
            w.dispatch_event(
                PlatformInput::MouseMove(MouseMoveEvent {
                    position,
                    ..Default::default()
                }),
                cx,
            );
            w.dispatch_event(
                PlatformInput::MouseDown(MouseDownEvent {
                    button: MouseButton::Left,
                    position,
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
        })
        .unwrap();
    drive_frames(handle, Duration::from_millis(45), cx).await;
    handle
        .update(cx, |_, w, cx| {
            let position = point_at(w, to.0, to.1);
            w.dispatch_event(
                PlatformInput::MouseMove(MouseMoveEvent {
                    position,
                    pressed_button: Some(MouseButton::Left),
                    ..Default::default()
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
        })
        .unwrap();
    drive_frames(handle, Duration::from_millis(100), cx).await;
}

pub(super) async fn exercise_scroll(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let h = AnyWindowHandle::from(window);
    let offset = |cx: &mut AsyncApp| window.update(cx, |v, _, _| v.scroll_offset()).unwrap();
    let click =
        |x, y, cx: &mut AsyncApp| h.update(cx, |_, w, cx| pointer(w, cx, x, y, true)).unwrap();
    assert!(offset(cx) < 1.);
    click(536., 713., cx);
    let label_ink = |image: &image::RgbaImage| {
        let scale = image.width() as f32 / 1600.;
        let mut ink = Vec::new();
        for y in (688. * scale) as u32..(720. * scale) as u32 {
            for x in (140. * scale) as u32..(440. * scale) as u32 {
                if image.get_pixel(x, y)[0] < 115 {
                    ink.push((x, y));
                }
            }
        }
        ink
    };
    let first_frame = label_ink(&frame(h, cx));
    let bottom = offset(cx);
    assert!(bottom > 10., "down arrow did not scroll");
    drive_frames(h, Duration::from_millis(950), cx).await;
    let settled = label_ink(&frame(h, cx));
    assert!(
        !settled.is_empty() && first_frame == settled,
        "Texture Pack must be fully lettered on the first frame after scrolling"
    );
    click(536., 244., cx);
    drive_frames(h, Duration::from_millis(210), cx).await;
    assert!(offset(cx) < 1., "up arrow did not scroll");
    drag(h, (536., 417.), (536., 615.), cx).await;
    assert!(
        (offset(cx) - bottom).abs() < 1.,
        "thumb did not drag to last row"
    );
    drag(h, (536., 490.), (536., 250.), cx).await;
    assert!(offset(cx) < 1., "thumb did not drag back to first row");
    h.update(cx, |_, w, cx| {
        for _ in 0..8 {
            keystroke(w, cx, "down");
        }
    })
    .unwrap();
    drive_frames(h, Duration::from_millis(250), cx).await;
    assert!(
        offset(cx) > 10.,
        "keyboard navigation did not scroll the list"
    );
}

pub(super) async fn exercise_field(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let h = AnyWindowHandle::from(window);
    h.update(cx, |_, w, cx| pointer(w, cx, 1040., 507., true))
        .unwrap();
    drive_frames(h, Duration::from_millis(90), cx).await;
    h.update(cx, |_, w, cx| {
        for key in ["ctrl-a", "y", "o", "r", "h", "a", "enter"] {
            w.dispatch_keystroke(gpui::Keystroke::parse(key).unwrap(), cx);
        }
    })
    .unwrap();
    window
        .update(cx, |v, _, _| assert_eq!(v.profile_name(), "yorha"))
        .unwrap();
}

fn point_at(w: &Window, x: f32, y: f32) -> gpui::Point<gpui::Pixels> {
    let viewport = w.viewport_size();
    let scale = (viewport.width / px(1600.)).min(viewport.height / px(900.));
    point(
        viewport.width / 2. + px((x - 800.) * scale),
        viewport.height / 2. + px((y - 450.) * scale),
    )
}

fn pointer(w: &mut Window, cx: &mut App, x: f32, y: f32, click: bool) {
    let position = point_at(w, x, y);
    w.dispatch_event(
        PlatformInput::MouseMove(MouseMoveEvent {
            position,
            ..Default::default()
        }),
        cx,
    );
    if click {
        w.dispatch_event(
            PlatformInput::MouseDown(MouseDownEvent {
                position,
                button: MouseButton::Left,
                click_count: 1,
                modifiers: Default::default(),
                first_mouse: false,
            }),
            cx,
        );
        w.dispatch_event(
            PlatformInput::MouseUp(MouseUpEvent {
                position,
                button: MouseButton::Left,
                click_count: 1,
                modifiers: Default::default(),
            }),
            cx,
        );
    }
}

fn frame(handle: AnyWindowHandle, cx: &mut AsyncApp) -> image::RgbaImage {
    handle
        .update(cx, |_, w, cx| {
            w.simulate_next_frame(cx);
            w.refresh();
            w.draw(cx).clear(cx);
            w.render_to_image().expect("read preview frame")
        })
        .expect("capture preview frame")
}

fn luminance(image: &image::RgbaImage, x: f32, y: f32) -> f32 {
    let x = (x * image.width() as f32 / 1600.) as u32;
    let y = (y * image.height() as f32 / 900.) as u32;
    let mut sum = 0.;
    for j in y..y + 3 {
        for i in x..x + 3 {
            let p = image.get_pixel(i, j);
            sum += f32::from(p[0]);
        }
    }
    sum / 9.
}

pub(super) async fn exercise_tabs(
    window: WindowHandle<ModManager>,
    cx: &mut AsyncApp,
) -> image::RgbaImage {
    let handle = AnyWindowHandle::from(window);
    let before = frame(handle, cx);
    handle
        .update(cx, |_, w, cx| pointer(w, cx, 450., 50., false))
        .expect("hover second tab");
    let mut swept = None;
    for _ in 0..8 {
        let sample = frame(handle, cx);
        if luminance(&sample, 322., 66.) + 26. < luminance(&sample, 477., 66.) {
            swept = Some(sample);
            break;
        }
        cx.background_executor()
            .timer(Duration::from_millis(6))
            .await;
    }
    let swept = swept.expect("the incoming tab highlight must cross the face from left to right");
    drive_frames(handle, Duration::from_millis(420), cx).await;
    let hovered = frame(handle, cx);
    window
        .update(cx, |v, _, _| {
            assert_eq!((v.page(), v.selection()), (0, (0, Focus::Rows)))
        })
        .expect("keep current page while hovering");
    let value = |img: &image::RgbaImage, x, y| luminance(img, x, y);
    let point = (471., 51.);
    assert!(
        value(&before, point.0, point.1) - value(&hovered, point.0, point.1) > 25.,
        "hovered tab did not darken: {} -> {}",
        value(&before, point.0, point.1),
        value(&hovered, point.0, point.1)
    );
    assert!(
        value(&hovered, 246., 82.) + 25. < value(&hovered, 457., 82.),
        "open tab did not keep its extension: {} vs {}",
        value(&hovered, 246., 82.),
        value(&hovered, 457., 82.)
    );
    handle
        .update(cx, |_, w, cx| pointer(w, cx, 450., 50., true))
        .expect("open second tab");
    drive_frames(handle, Duration::from_millis(450), cx).await;
    let switched = frame(handle, cx);
    assert!(
        value(&switched, 457., 82.) + 25. < value(&switched, 246., 82.),
        "newly active tab did not expand: {} vs {}",
        value(&switched, 457., 82.),
        value(&switched, 246., 82.)
    );
    handle
        .update(cx, |_, w, cx| pointer(w, cx, 450., 50., true))
        .expect("reselect current tab");
    window
        .update(cx, |v, _, _| {
            assert_eq!((v.page(), v.selection()), (1, (0, Focus::Rows)))
        })
        .expect("retain current page");
    let sx = before.width() as f32 / 1600.;
    let sy = before.height() as f32 / 900.;
    let (w, h) = ((250. * sx) as u32, (90. * sy) as u32);
    let mut strip = image::RgbaImage::new(w * 4, h);
    for (i, frame) in [&before, &swept, &hovered, &switched]
        .into_iter()
        .enumerate()
    {
        let crop =
            image::imageops::crop_imm(frame, (263. * sx) as u32, (9. * sy) as u32, w, h).to_image();
        image::imageops::replace(&mut strip, &crop, i as i64 * i64::from(w), 0);
    }
    strip
}
