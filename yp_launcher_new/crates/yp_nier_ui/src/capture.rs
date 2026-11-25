use crate::draw::{Prepared, Rect};
use gpui::{App, Global, RenderImage};
use image::{Frame, RgbaImage};
use std::{
    collections::{VecDeque, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Default)]
struct Images(VecDeque<(u64, Arc<RenderImage>)>);
impl Global for Images {}

pub(crate) fn bilinear(data: &[[u8; 4]], w: usize, h: usize, u: f32, v: f32) -> [f32; 4] {
    let (u, v) = (u.clamp(0., (w - 1) as f32), v.clamp(0., (h - 1) as f32));
    let (x, y) = (u as usize, v as usize);
    let (nx, ny) = ((x + 1).min(w - 1), (y + 1).min(h - 1));
    let (fx, fy) = (u.fract(), v.fract());
    std::array::from_fn(|i| {
        let a = f32::from(data[y * w + x][i]) * (1. - fx) + f32::from(data[y * w + nx][i]) * fx;
        let b = f32::from(data[ny * w + x][i]) * (1. - fx) + f32::from(data[ny * w + nx][i]) * fx;
        a * (1. - fy) + b * fy
    })
}

pub(crate) fn dimensions(dst: Rect, density: f32) -> ([usize; 2], f32) {
    let physical = [dst.w * density, dst.h * density];
    let scale = (16_777_216. / (physical[0] * physical[1])).sqrt().min(1.);
    (
        physical.map(|v| (v * scale).round().max(1.) as usize),
        scale,
    )
}

pub(crate) fn filtered(
    draws: &[Prepared<'_>],
    screen: Rect,
    dims: [usize; 2],
    settings: impl Hash,
    cx: &mut App,
    transform: impl FnOnce(&[[u8; 4]], usize, usize) -> Vec<u8>,
) -> Arc<RenderImage> {
    let mut hash = DefaultHasher::new();
    (
        dims,
        [screen.x, screen.y, screen.w, screen.h].map(f32::to_bits),
        settings,
    )
        .hash(&mut hash);
    for draw in draws {
        draw.fingerprint(&mut hash);
    }
    let key = hash.finish();
    if !cx.has_global::<Images>() {
        cx.set_global(Images::default());
    }
    let cache = &mut cx.global_mut::<Images>().0;
    if let Some((_, image)) = cache.iter().find(|(k, _)| *k == key) {
        return image.clone();
    }
    let [w, h] = dims;
    let source = pixels(draws, screen, w, h);
    let bytes = transform(&source, w, h);
    let buf = RgbaImage::from_raw(w as u32, h as u32, bytes).expect("complete BGRA texture");
    let image = Arc::new(RenderImage::new(vec![Frame::new(buf)]));
    let old = if cache.len() >= 4 {
        cache.pop_front()
    } else {
        None
    };
    cache.push_back((key, image.clone()));
    if let Some((_, old)) = old {
        cx.defer(move |cx| cx.drop_image(old, None));
    }
    image
}

fn over(dst: &mut [u8; 4], src: [u8; 4]) {
    if src[3] == 0 {
        return;
    }
    if src[3] == 255 || dst[3] == 0 {
        *dst = src;
        return;
    }
    let a = u32::from(src[3]);
    let remaining = u32::from(dst[3]) * (255 - a) / 255;
    let alpha = a + remaining;
    for i in 0..3 {
        dst[i] = ((u32::from(src[i]) * a + u32::from(dst[i]) * remaining) / alpha) as u8;
    }
    dst[3] = alpha as u8;
}

pub(crate) fn pixels(
    draws: &[Prepared<'_>],
    screen: Rect,
    width: usize,
    height: usize,
) -> Vec<[u8; 4]> {
    let mut pixels = vec![[0u8; 4]; width * height];
    let dx = screen.w / width as f32;
    let dy = screen.h / height as f32;
    for draw in draws {
        let Some(visible) = draw.bounds().intersect(screen) else {
            continue;
        };
        let left = ((visible.x - screen.x) / dx - 0.5).ceil().max(0.) as usize;
        let right = ((visible.x + visible.w - screen.x) / dx - 0.5)
            .ceil()
            .min(width as f32) as usize;
        let top = ((visible.y - screen.y) / dy - 0.5).ceil().max(0.) as usize;
        let bottom = ((visible.y + visible.h - screen.y) / dy - 0.5)
            .ceil()
            .min(height as f32) as usize;
        for y in top..bottom {
            let sy = screen.y + (y as f32 + 0.5) * dy;
            let line = &mut pixels[y * width..(y + 1) * width];
            for (x, out) in line.iter_mut().enumerate().take(right).skip(left) {
                let pixel = draw.sample(screen.x + (x as f32 + 0.5) * dx, sy);
                if draw.additive() && out[3] != 0 {
                    for ch in 0..3 {
                        out[ch] = out[ch].saturating_add(
                            ((u16::from(pixel[ch]) * u16::from(pixel[3]) + 127) / 255) as u8,
                        );
                    }
                } else {
                    over(out, pixel);
                }
            }
        }
    }
    pixels
}
