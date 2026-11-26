use crate::atlas::image;
use gpui::{
    App, Bounds, ContentMask, IntoElement, Pixels, RenderImage, Window, canvas, deferred, div,
    point, prelude::*, px, size,
};
use image::{Rgba, RgbaImage};
use std::sync::{Arc, LazyLock};

const TILE: u32 = 7 * 48;

static GRID: LazyLock<Arc<RenderImage>> = LazyLock::new(|| {
    image(RgbaImage::from_fn(TILE, TILE, |x, y| {
        if x % 7 == 0 || y % 7 == 0 {
            Rgba([26, 25, 18, 13])
        } else if x % 7 == 1 || y % 7 == 1 {
            Rgba([230, 226, 201, 5])
        } else {
            Rgba([0, 0, 0, 0])
        }
    }))
});

static VIGNETTE: LazyLock<Arc<RenderImage>> = LazyLock::new(|| {
    image(RgbaImage::from_fn(256, 144, |x, y| {
        let u = (2. * x as f32 / 255. - 1.).abs();
        let v = (2. * y as f32 / 143. - 1.).abs();
        let side = ((u - 0.45) / 0.55).max(0.).powi(2);
        let cap = ((v - 0.60) / 0.40).max(0.).powi(2);
        let alpha = 50. * side + 26. * cap + 13. * side * cap;
        Rgba([26, 25, 18, alpha.round() as u8])
    }))
});

fn paint_screen(screen: Bounds<Pixels>, clip: Bounds<Pixels>, window: &mut Window) {
    let clip = screen
        .intersect(&clip)
        .intersect(&window.content_mask().bounds);
    if clip.is_empty() {
        return;
    }
    let tile = px(TILE as f32);
    let columns = ((clip.left() - screen.left()) / tile).floor() as u32
        ..((clip.right() - screen.left()) / tile).ceil() as u32;
    let rows = ((clip.top() - screen.top()) / tile).floor() as u32
        ..((clip.bottom() - screen.top()) / tile).ceil() as u32;
    window.with_content_mask(Some(ContentMask { bounds: clip }), |window| {
        for row in rows {
            for col in columns.clone() {
                let origin = screen.origin + point(tile * col as f32, tile * row as f32);
                let bounds = Bounds::new(origin, size(tile, tile));
                let _ =
                    window.paint_image(bounds, bounds, Default::default(), GRID.clone(), 0, false);
            }
        }
        let _ = window.paint_image(
            screen,
            screen,
            Default::default(),
            VIGNETTE.clone(),
            0,
            false,
        );
    });
}

pub fn menu_screen_overlay() -> impl IntoElement {
    deferred(
        div()
            .absolute()
            .inset_0()
            .debug_selector(|| "menu-screen-overlay".to_owned())
            .child(
                canvas(
                    |_, _, _| (),
                    |bounds, _, window, _| {
                        paint_screen(bounds, bounds, window);
                    },
                )
                .size_full(),
            ),
    )
    .with_priority(usize::MAX)
}

pub fn menu_popup_overlay() -> impl IntoElement {
    canvas(
        |_, _, _| (),
        |bounds, _, window: &mut Window, _: &mut App| {
            paint_screen(
                Bounds::new(Default::default(), window.viewport_size()),
                bounds,
                window,
            );
        },
    )
    .absolute()
    .inset_0()
    .size_full()
}
