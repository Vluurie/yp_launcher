use crate::atlas::{Theme, Tint};
use crate::{GridNoise, Lens};
use gpui::{
    App, Bounds, ContentMask, Global, IntoElement, Pixels, RenderImage, RenderOnce, Window, canvas,
    fill, point, prelude::*, px, rgba, size,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub(crate) fn from_bounds(bounds: Bounds<Pixels>) -> Self {
        Self::new(
            bounds.left().into(),
            bounds.top().into(),
            bounds.size.width.into(),
            bounds.size.height.into(),
        )
    }
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
    pub(crate) fn intersect(self, other: Rect) -> Option<Self> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let w = (self.x + self.w).min(other.x + other.w) - x;
        let h = (self.y + self.h).min(other.y + other.h) - y;
        (w > 0. && h > 0.).then_some(Rect::new(x, y, w, h))
    }
    pub(crate) fn mapped(self, from: Rect, to: Rect) -> Rect {
        Rect::new(
            to.x + (self.x - from.x) * to.w / from.w,
            to.y + (self.y - from.y) * to.h / from.h,
            self.w * to.w / from.w,
            self.h * to.h / from.h,
        )
    }
    pub(crate) fn bounds(self) -> Bounds<Pixels> {
        Bounds::new(point(px(self.x), px(self.y)), size(px(self.w), px(self.h)))
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
enum Asset {
    Sprite(String, Tint),
    Text(String, u16, Tint, u32, u8),
}

#[derive(Default)]
struct Images {
    clock: u64,
    entries: HashMap<Asset, (Arc<RenderImage>, u64)>,
}
impl Global for Images {}

pub(crate) fn forget_text(cx: &mut App) {
    if !cx.has_global::<Images>() {
        return;
    }
    let cache = cx.global_mut::<Images>();
    let stale: Vec<Asset> = cache
        .entries
        .keys()
        .filter(|key| matches!(key, Asset::Text(..)))
        .cloned()
        .collect();
    let dropped: Vec<Arc<RenderImage>> = stale
        .iter()
        .filter_map(|key| cache.entries.remove(key))
        .map(|(image, _)| image)
        .collect();
    cx.defer(move |cx| {
        for image in dropped {
            cx.drop_image(image, None);
        }
    });
}

fn texture(key: Asset, cx: &mut App) -> Option<Arc<RenderImage>> {
    if !cx.has_global::<Images>() {
        cx.set_global(Images::default());
    }
    let cache = cx.global_mut::<Images>();
    cache.clock += 1;
    if let Some((image, last)) = cache.entries.get_mut(&key) {
        *last = cache.clock;
        return Some(image.clone());
    }
    let data = match &key {
        Asset::Sprite(name, tint) => Theme::bundled().sprite_image(name, *tint)?,
        Asset::Text(text, font, tint, spacing, alpha) => {
            Theme::bundled().run_image(text, *font, *tint, f32::from_bits(*spacing), *alpha)?
        }
    };
    let old = if cache.entries.len() >= 256 {
        let lru = cache
            .entries
            .iter()
            .min_by_key(|(_, (_, time))| time)
            .map(|(k, _)| k.clone());
        lru.and_then(|k| cache.entries.remove(&k))
    } else {
        None
    };
    cache.entries.insert(key, (data.clone(), cache.clock));
    if let Some((old, _)) = old {
        cx.defer(move |cx| cx.drop_image(old, None));
    }
    Some(data)
}

#[derive(Clone)]
enum Brush {
    Solid(Tint, u8),
    Image(Asset, Option<Rect>),
    Raster(Arc<RenderImage>, Option<Rect>),
}
struct Draw {
    additive: bool,
    rect: Rect,
    clip: Rect,
    brush: Brush,
}

#[derive(IntoElement)]
pub struct Surface {
    pub width: f32,
    pub height: f32,
    clip: Rect,
    draws: Vec<Draw>,
    pub(crate) additive: bool,
    lens: Option<Lens>,
    noise: Option<GridNoise>,
    pub(crate) blur_strength: f32,
    pub(crate) static_blocks: Option<Vec<Rect>>,
}

impl Surface {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            clip: Rect::new(0., 0., width, height),
            draws: Vec::new(),
            additive: false,
            lens: None,
            noise: None,
            blur_strength: 0.,
            static_blocks: None,
        }
    }
    pub fn clip(&mut self, bounds: Rect, draw: impl FnOnce(&mut Self)) {
        let saved = self.clip;
        if let Some(clip) = saved.intersect(bounds) {
            self.clip = clip;
            draw(self);
            self.clip = saved;
        }
    }
    fn push(&mut self, rect: Rect, brush: Brush) {
        if rect.intersect(self.clip).is_some() {
            self.draws.push(Draw {
                additive: self.additive,
                rect,
                clip: self.clip,
                brush,
            });
        }
    }
    pub fn fill(&mut self, rect: Rect, tint: Tint, alpha: u8) {
        self.fill_raw(rect, tint, alpha);
    }
    pub(crate) fn fill_raw(&mut self, rect: Rect, tint: Tint, alpha: u8) {
        self.push(rect, Brush::Solid(tint, alpha));
    }
    pub fn raster(&mut self, rect: Rect, image: Arc<RenderImage>, region: Option<Rect>) {
        self.push(rect, Brush::Raster(image, region));
    }
    pub fn sprite(&mut self, name: &str, rect: Rect, tint: Tint) {
        self.push(
            rect,
            Brush::Image(Asset::Sprite(name.to_owned(), tint), None),
        );
    }
    pub fn text(&mut self, text: &str, font: u16, scale: f32, x: f32, center_y: f32, tint: Tint) {
        self.lettering(
            text,
            font,
            scale,
            0.,
            [x, center_y],
            tint,
            255,
        );
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn lettering(
        &mut self,
        text: &str,
        font: u16,
        scale: f32,
        spacing: f32,
        position: [f32; 2],
        tint: Tint,
        alpha: u8,
    ) {
        let [x, center_y] = position;
        let [w, h] = Theme::bundled().measure_spaced(text, font, spacing);
        if w <= 0. {
            return;
        }
        let rect = Rect::new(
            x,
            center_y - h * scale / 2.,
            (w.ceil() + 8.) * scale,
            h.ceil() * scale,
        );
        self.push(
            rect,
            Brush::Image(
                Asset::Text(text.to_owned(), font, tint, spacing.to_bits(), alpha),
                None,
            ),
        );
    }
    pub fn nine_slice(
        &mut self,
        name: &str,
        rect: Rect,
        source_border: [f32; 2],
        border: [f32; 2],
        tint: Tint,
    ) {
        let Some([w, h]) = Theme::bundled().sprite_size(name) else {
            return;
        };
        let axis = |length: f32, b: f32| {
            let b = b.clamp(0., length / 2.);
            [0., b, length - b, length]
        };
        let [sx, sy] = [
            axis(w as f32, source_border[0]),
            axis(h as f32, source_border[1]),
        ];
        let [dx, dy] = [axis(rect.w, border[0]), axis(rect.h, border[1])];
        for row in 0..3 {
            for col in 0..3 {
                let dest = Rect::new(
                    rect.x + dx[col],
                    rect.y + dy[row],
                    dx[col + 1] - dx[col],
                    dy[row + 1] - dy[row],
                );
                let src = Rect::new(
                    sx[col],
                    sy[row],
                    sx[col + 1] - sx[col],
                    sy[row + 1] - sy[row],
                );
                if src.w > 0. && src.h > 0. {
                    self.push(
                        dest,
                        Brush::Image(Asset::Sprite(name.to_owned(), tint), Some(src)),
                    );
                }
            }
        }
    }
    pub fn lens(&mut self, lens: Lens) {
        self.lens = Some(lens);
    }
    pub fn grid_noise(&mut self, noise: GridNoise) {
        self.noise = Some(noise);
    }

    pub(crate) fn paint(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        if self.width <= 0. || self.height <= 0. {
            return;
        }
        let src = Rect::new(0., 0., self.width, self.height);
        let dst = Rect::new(
            bounds.origin.x.into(),
            bounds.origin.y.into(),
            bounds.size.width.into(),
            bounds.size.height.into(),
        );
        if dst.w <= 0. || dst.h <= 0. {
            return;
        }
        if crate::filter_layer::collect(
            self,
            src,
            dst,
            Rect::from_bounds(window.content_mask().bounds),
            cx,
        ) {
            return;
        }
        let draws: Vec<_> = self
            .draws
            .iter()
            .filter_map(|op| {
                let image = match &op.brush {
                    Brush::Solid(..) => None,
                    Brush::Image(asset, _) => Some(texture(asset.clone(), cx)?),
                    Brush::Raster(image, _) => Some(image.clone()),
                };
                Some(Prepared { op, image })
            })
            .collect();
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            if let Some(noise) = &self.noise {
                noise.paint(&draws, self.lens.as_ref(), src, dst, window, cx);
            } else if self.blur_strength > 0. {
                crate::grid_noise::paint_blur(self.blur_strength, &draws, src, dst, window, cx);
            } else if let Some(lens) = &self.lens {
                lens.paint(&draws, src, dst, window, cx);
            } else {
                for (i, draw) in draws.iter().enumerate() {
                    if draw.additive() {
                        let Some(area) = draw.bounds().intersect(src) else {
                            continue;
                        };
                        let dest = area.mapped(src, dst);
                        let (dims, _) = crate::capture::dimensions(dest, window.scale_factor());
                        let image = crate::capture::filtered(
                            &draws[..=i],
                            area,
                            dims,
                            "additive",
                            cx,
                            |rgba, _, _| {
                                rgba.iter().flat_map(|p| [p[2], p[1], p[0], p[3]]).collect()
                            },
                        );
                        let bounds = dest.bounds();
                        let _ =
                            window.paint_image(bounds, bounds, Default::default(), image, 0, false);
                    } else {
                        draw.paint(src, dst, window);
                    }
                }
            }
            if let Some(regions) = &self.static_blocks {
                crate::static_noise::paint(regions, &draws, src, dst, window, cx);
            }
        });
    }
}

impl Surface {
    pub(crate) fn append(&mut self, child: &Surface, src: Rect, placement: Rect, clip: Rect) {
        for draw in &child.draws {
            if let Some(mask) = draw.clip.mapped(src, placement).intersect(clip) {
                self.draws.push(Draw {
                    additive: draw.additive,
                    rect: draw.rect.mapped(src, placement),
                    clip: mask,
                    brush: draw.brush.clone(),
                });
            }
        }
    }
}

pub(crate) struct Prepared<'a> {
    op: &'a Draw,
    image: Option<Arc<RenderImage>>,
}

impl Prepared<'_> {
    pub(crate) fn tinted(&self, tint: Tint, cx: &mut App) -> Option<Arc<RenderImage>> {
        let key = match &self.op.brush {
            Brush::Image(Asset::Text(text, font, _, spacing, alpha), _) => {
                Asset::Text(text.clone(), *font, tint, *spacing, *alpha)
            }
            Brush::Image(Asset::Sprite(name, _), _)
                if self.op.rect.w < 120. && self.op.rect.h < 120. =>
            {
                Asset::Sprite(name.clone(), tint)
            }
            _ => return None,
        };
        texture(key, cx)
    }
    pub(crate) fn paint_tinted(
        &self,
        src: Rect,
        dst: Rect,
        window: &mut Window,
        image: Arc<RenderImage>,
    ) {
        Prepared {
            op: self.op,
            image: Some(image),
        }
        .paint(src, dst, window);
    }
    pub(crate) fn additive(&self) -> bool {
        self.op.additive
    }
    pub(crate) fn fingerprint(&self, hash: &mut impl std::hash::Hasher) {
        use std::hash::Hash;
        let rect = |r: Rect| [r.x, r.y, r.w, r.h].map(f32::to_bits);
        self.op.additive.hash(hash);
        rect(self.op.rect).hash(hash);
        rect(self.op.clip).hash(hash);
        match &self.op.brush {
            Brush::Solid(rgb, alpha) => (0u8, rgb, alpha).hash(hash),
            Brush::Image(asset, region) => (1u8, asset, region.map(rect)).hash(hash),
            Brush::Raster(image, region) => (2u8, image.id, region.map(rect)).hash(hash),
        }
    }
    pub(crate) fn bounds(&self) -> Rect {
        self.op
            .rect
            .intersect(self.op.clip)
            .unwrap_or(Rect::new(0., 0., 0., 0.))
    }
    pub(crate) fn paint(&self, src: Rect, dst: Rect, window: &mut Window) {
        let Some(visible) = self.bounds().intersect(src) else {
            return;
        };
        let bounds = visible.mapped(src, dst).bounds();
        match (&self.op.brush, &self.image) {
            (Brush::Solid([r, g, b], a), _) => {
                window.paint_quad(fill(bounds, rgba(u32::from_be_bytes([*r, *g, *b, *a]))))
            }
            (Brush::Image(_, region) | Brush::Raster(_, region), Some(image)) => {
                let dims = image.size(0);
                let full = Rect::new(0., 0., dims.width.0 as f32, dims.height.0 as f32);
                let total = full
                    .mapped(region.unwrap_or(full), self.op.rect)
                    .mapped(src, dst)
                    .bounds();
                let _ =
                    window.paint_image(bounds, total, Default::default(), image.clone(), 0, false);
            }
            _ => {}
        }
    }
    pub(crate) fn sample(&self, x: f32, y: f32) -> [u8; 4] {
        match (&self.op.brush, &self.image) {
            (Brush::Solid([r, g, b], a), _) => [*r, *g, *b, *a],
            (Brush::Image(_, region) | Brush::Raster(_, region), Some(image)) => {
                let dims = image.size(0);
                let (w, h) = (dims.width.0 as f32, dims.height.0 as f32);
                let r = region.unwrap_or(Rect::new(0., 0., w, h));
                let u =
                    (r.x + (x - self.op.rect.x) / self.op.rect.w * r.w).clamp(0., w - 1.) as usize;
                let v =
                    (r.y + (y - self.op.rect.y) / self.op.rect.h * r.h).clamp(0., h - 1.) as usize;
                let i = (v * w as usize + u) * 4;
                let bytes = image.as_bytes(0).unwrap();
                [bytes[i + 2], bytes[i + 1], bytes[i], bytes[i + 3]]
            }
            _ => [0; 4],
        }
    }
}

impl RenderOnce for Surface {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        canvas(
            |_, _, _| (),
            move |bounds, (), window, cx| self.paint(bounds, window, cx),
        )
        .size_full()
    }
}
