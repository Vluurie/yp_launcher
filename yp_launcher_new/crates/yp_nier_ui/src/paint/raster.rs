use super::{Graphic, SpriteKey, SpriteSource, geometry::Transform};
use crate::{Rect, Surface, Theme, atlas::image};
use gpui::{App, Global, RenderImage};
use image::{Rgba, RgbaImage};
use std::{
    collections::VecDeque,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

#[derive(Default)]
struct Cache(VecDeque<(u64, Arc<RenderImage>)>);
impl Global for Cache {}
fn cached(
    key: impl Hash,
    cx: &mut App,
    create: impl FnOnce() -> Arc<RenderImage>,
) -> Arc<RenderImage> {
    let mut h = DefaultHasher::new();
    key.hash(&mut h);
    let k = h.finish();
    if !cx.has_global::<Cache>() {
        cx.set_global(Cache::default());
    }
    let store = &mut cx.global_mut::<Cache>().0;
    if let Some(i) = store.iter().position(|(key, _)| *key == k) {
        let entry = store.remove(i).unwrap();
        let result = entry.1.clone();
        store.push_back(entry);
        return result;
    }
    let item = create();
    let old = if store.len() >= 192 {
        store.pop_front()
    } else {
        None
    };
    store.push_back((k, item.clone()));
    if let Some((_, old)) = old {
        cx.defer(move |cx| cx.drop_image(old, None));
    }
    item
}

pub(crate) struct Ink {
    pub color: [f32; 4],
    pub gradient: [f32; 4],
    pub mode: [i16; 3],
}
fn byte(v: f32) -> u8 {
    (v.clamp(0., 1.) * 255.).round() as u8
}

pub(crate) fn sprite(
    res: &dyn SpriteSource,
    g: &Graphic,
    key: SpriteKey,
    ink: &Ink,
    cx: &mut App,
) -> Option<Arc<RenderImage>> {
    let raw = res.sprite(key)?;
    let c = ink.color.map(byte);
    let gcol = ink.gradient.map(byte);
    let gradient = ink.mode;
    let cache_key = (0u8, res.id(), key, g.channels, g.flip, c, gcol, gradient);
    Some(cached(cache_key, cx, || {
        let (w, h) = raw.dimensions();
        image(RgbaImage::from_fn(w, h, |x, y| {
            let p = raw
                .get_pixel(
                    if g.flip[0] { w - 1 - x } else { x },
                    if g.flip[1] { h - 1 - y } else { y },
                )
                .0;
            let f = match gradient[0] {
                0 => 0.,
                1 => y as f32 / 2.,
                _ => x as f32 / 2.,
            };
            let t = if gradient[0] != 0 && gradient[2] != 0 {
                ((f - f32::from(gradient[1])) / f32::from(gradient[2])).clamp(0., 1.)
            } else {
                0.
            };
            let shade =
                |i: usize| (f32::from(c[i]) + (f32::from(gcol[i]) - f32::from(c[i])) * t) / 255.;
            Rgba(std::array::from_fn(|i| {
                let src = if i == 3 && g.channels == 1 {
                    (f32::from(p[0]) + f32::from(p[1]) + f32::from(p[2])) / 3.
                } else if i != 3 && matches!(g.channels, 1 | 2) {
                    255.
                } else {
                    f32::from(p[i])
                };
                (src * shade(i)).round() as u8
            }))
        }))
    }))
}

#[derive(Clone, Copy)]
pub(crate) struct Mask {
    pub transform: Transform,
    pub rect: Rect,
    pub inverse: bool,
}

pub(crate) struct Painter<'a> {
    pub surface: &'a mut Surface,
    pub cx: &'a mut App,
    pub density: f32,
}
fn coords(r: Rect) -> [u32; 4] {
    [r.x, r.y, r.w, r.h].map(f32::to_bits)
}

impl Painter<'_> {
    pub fn quad(
        &mut self,
        transform: Transform,
        local: Rect,
        texture: Arc<RenderImage>,
        uv: Rect,
        masks: &[Mask],
    ) {
        if local.w <= 0. || local.h <= 0. || uv.w <= 0. || uv.h <= 0. {
            return;
        }
        let full = transform.bounds(local);
        let screen = Rect::new(0., 0., self.surface.width, self.surface.height);
        let mut visible = match full.intersect(screen) {
            Some(r) => r,
            None => return,
        };
        for m in masks.iter().filter(|m| !m.inverse) {
            let Some(r) = visible.intersect(m.transform.bounds(m.rect)) else {
                return;
            };
            visible = r;
        }
        if transform.axis_aligned()
            && masks
                .iter()
                .all(|m| m.transform.axis_aligned() && !m.inverse)
        {
            self.surface
                .clip(visible, |s| s.raster(full, texture, Some(uv)));
            return;
        }
        let bounds =
            if masks.is_empty() && full.w * self.density < 4096. && full.h * self.density < 4096. {
                full
            } else {
                visible
            };
        let density = self
            .density
            .min((4_194_304. / (bounds.w * bounds.h)).sqrt());
        let (w, h) = (
            (bounds.w * density).ceil().clamp(1., 8192.) as u32,
            (bounds.h * density).ceil().clamp(1., 8192.) as u32,
        );
        let rel = |t: Transform| {
            [
                t.axes[0][0],
                t.axes[0][1],
                t.axes[1][0],
                t.axes[1][1],
                t.position[0] - bounds.x,
                t.position[1] - bounds.y,
            ]
            .map(f32::to_bits)
        };
        let clips: Vec<_> = masks
            .iter()
            .map(|m| (rel(m.transform), coords(m.rect), m.inverse))
            .collect();
        let key = (
            1u8,
            texture.id,
            rel(transform),
            coords(local),
            coords(uv),
            [bounds.w.to_bits(), bounds.h.to_bits()],
            w,
            h,
            clips,
        );
        let out = cached(key, self.cx, || {
            let bytes = texture.as_bytes(0).unwrap();
            let src = texture.size(0);
            let (sw, sh) = (src.width.0 as usize, src.height.0 as usize);
            let texels = bytes.as_chunks::<4>().0;
            let data = RgbaImage::from_fn(w, h, |x, y| {
                let p = [
                    bounds.x + (x as f32 + 0.5) * bounds.w / w as f32,
                    bounds.y + (y as f32 + 0.5) * bounds.h / h as f32,
                ];
                if masks.iter().any(|m| {
                    m.transform
                        .invert(p)
                        .is_some_and(|[x, y]| m.rect.contains(x, y))
                        == m.inverse
                }) {
                    return Rgba([0; 4]);
                }
                let Some([x, y]) = transform.invert(p) else {
                    return Rgba([0; 4]);
                };
                if !local.contains(x, y) {
                    return Rgba([0; 4]);
                }
                let (u, v) = (
                    uv.x + (x - local.x) * uv.w / local.w - 0.5,
                    uv.y + (y - local.y) * uv.h / local.h - 0.5,
                );
                Rgba(crate::capture::bilinear(texels, sw, sh, u, v).map(|v| v.round() as u8))
            });
            Arc::new(RenderImage::new(vec![image::Frame::new(data)]))
        });
        self.surface.raster(bounds, out, None);
    }
}

pub(crate) fn plain(color: [f32; 4], cx: &mut App) -> Arc<RenderImage> {
    let rgba = color.map(byte);
    cached((2u8, rgba), cx, || {
        image(RgbaImage::from_pixel(1, 1, Rgba(rgba)))
    })
}

#[cfg(feature = "uid-debug")]
pub(crate) fn reveal(
    mask: &RgbaImage,
    bank: u64,
    pattern: u32,
    level: f32,
    color: [f32; 4],
    cx: &mut App,
) -> Arc<RenderImage> {
    let threshold = ((1. - level) * 255.).ceil() as u8;
    let rgba = color.map(byte);
    cached((4u8, bank, pattern, threshold, rgba), cx, || {
        image(RgbaImage::from_fn(mask.width(), mask.height(), |x, y| {
            let mut p = rgba;
            if mask.get_pixel(x, y)[0] < threshold {
                p[3] = 0;
            }
            Rgba(p)
        }))
    })
}

pub(crate) fn text_image_spaced(
    text: &str,
    font: u16,
    spacing: f32,
    color: [f32; 4],
    cx: &mut App,
) -> Option<Arc<RenderImage>> {
    let [w, h] = Theme::bundled().measure_spaced(text, font, spacing);
    if w <= 0. || h <= 0. || w >= 16384. || h >= 8192. || w * h >= 16_777_216. {
        return None;
    }
    let [r, g, b, a] = color.map(byte);
    Some(cached(
        (3u8, text, font, spacing.to_bits(), [r, g, b, a]),
        cx,
        || {
            let img = Theme::bundled()
                .text_image_spaced(text, font, [r, g, b], spacing)
                .expect("measured glyph run");
            if a == 255 {
                return img;
            }
            let size = img.size(0);
            let mut bytes = img.as_bytes(0).unwrap().to_vec();
            for p in bytes.as_chunks_mut::<4>().0 {
                p[3] = (u16::from(p[3]) * u16::from(a) / 255) as u8;
            }
            let buf =
                RgbaImage::from_raw(size.width.0 as u32, size.height.0 as u32, bytes).unwrap();
            Arc::new(RenderImage::new(vec![image::Frame::new(buf)]))
        },
    ))
}
