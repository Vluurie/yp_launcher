use super::{ModManager, drive_frames};
use gpui::{AnyWindowHandle, AsyncApp, WindowHandle};
use std::time::{Duration, Instant};
use yp_nier_ui::MenuInput;

pub(super) async fn effects(window: WindowHandle<ModManager>, cx: &mut AsyncApp) {
    let handle = AnyWindowHandle::from(window);
    drive_frames(handle, Duration::from_millis(800), cx).await;
    window
        .update(cx, |v, w, cx| {
            for k in [MenuInput::Right, MenuInput::Right, MenuInput::Confirm] {
                v.input(k, w, cx)
            }
        })
        .unwrap();
    let dimensions = window
        .update(cx, |_, w, _| (w.viewport_size(), w.scale_factor()))
        .unwrap();
    println!(
        "GPUI window {:?} at {:.2}x. CPU draw time in milliseconds.",
        dimensions.0, dimensions.1
    );
    println!("effect             first     median      p95");
    for (index, name) in [
        "Menu",
        "Eye loupe",
        "Grid noise",
        "Hacking static",
        "CRT",
        "Chromatic",
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            window
                .update(cx, |v, w, cx| {
                    if index > 1 {
                        v.input(MenuInput::Back, w, cx);
                    }
                    for _ in 0..index {
                        v.input(MenuInput::Down, w, cx);
                    }
                    v.input(MenuInput::Confirm, w, cx);
                    assert_eq!(v.selection().0, index);
                })
                .unwrap();
        }
        let mut samples = Vec::new();
        for _ in 0..90 {
            let elapsed = handle
                .update(cx, |_, w, cx| {
                    let at = Instant::now();
                    w.simulate_next_frame(cx);
                    w.refresh();
                    w.draw(cx).clear(cx);
                    at.elapsed().as_secs_f64() * 1000.
                })
                .unwrap();
            samples.push(elapsed);
            cx.background_executor()
                .timer(Duration::from_millis(15))
                .await;
        }
        let first = samples[0];
        samples.sort_by(|a, b| a.total_cmp(b));
        println!(
            "{name:16} {first:7.2}   {:7.2}   {:7.2}",
            samples[45], samples[85]
        );
    }
}
