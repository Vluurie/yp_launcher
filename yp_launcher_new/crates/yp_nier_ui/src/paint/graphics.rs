use super::{
    ImageLayer, NineGrid, SpriteKey, SpriteSource,
    geometry::{Transform, anchored},
    raster::{self, Ink, Mask, Painter},
};
use crate::Rect;
use gpui::RenderImage;

pub(crate) fn dimensions(image: &RenderImage) -> [f32; 2] {
    let size = image.size(0);
    [size.width.0 as f32, size.height.0 as f32]
}

pub(crate) struct GraphicPainter<'a, 'b> {
    pub painter: &'a mut Painter<'b>,
    pub resources: &'a dyn SpriteSource,
    pub transform: Transform,
    pub masks: &'a [Mask],
    pub ink: &'a Ink,
}

impl GraphicPainter<'_, '_> {
    pub fn image(
        &mut self,
        data: &ImageLayer,
        key: SpriteKey,
        size_override: Option<[f32; 2]>,
        fraction: Option<[f32; 2]>,
    ) -> Option<Rect> {
        self.painter.surface.additive = data.graphic.blend == 1;
        let (w, h) = (self.painter.surface.width, self.painter.surface.height);
        if key.texture == 0
            && key.sprite == 0
            && self.ink.mode[0] == 0
            && self.transform.axis_aligned()
            && self
                .masks
                .iter()
                .all(|m| m.transform.axis_aligned() && !m.inverse)
        {
            let mut dims = size_override.unwrap_or(if data.mode == 1 { [w, h] } else { data.size });
            if let Some([x, y]) = fraction {
                dims[0] *= x;
                dims[1] *= y;
            }
            let mut rect = self.transform.bounds(anchored(dims, data.graphic.anchor));
            for m in self.masks {
                rect = rect.intersect(m.transform.bounds(m.rect))?;
            }
            let [r, g, b, a] = self
                .ink
                .color
                .map(|v| (v.clamp(0., 1.) * 255.).round() as u8);
            self.painter.surface.fill_raw(rect, [r, g, b], a);
            return Some(rect);
        }
        let texture = if key.texture == 0 && key.sprite == 0 {
            raster::plain(self.ink.color, self.painter.cx)
        } else {
            raster::sprite(
                self.resources,
                &data.graphic,
                key,
                self.ink,
                self.painter.cx,
            )?
        };
        let [iw, ih] = dimensions(&texture);
        let mut size = data.size;
        let [l, t, r, b] = data.crop.unwrap_or([0.; 4]);
        let mut uv = Rect::new(l, t, (iw - l - r).max(0.), (ih - t - b).max(0.));
        if data.crop.is_some() && iw > 0. && ih > 0. {
            size = [size[0] * uv.w / iw, size[1] * uv.h / ih];
        }
        if data.mode == 1 {
            size = [w, h];
        }
        if let Some(s) = size_override {
            size = s;
        }
        let mut local = anchored(size, data.graphic.anchor);
        if let Some([x, y]) = fraction {
            local.w *= x;
            local.h *= y;
            uv.w *= x;
            uv.h *= y;
        }
        if data.tiled == 0 || uv.w <= 0. || uv.h <= 0. {
            self.painter
                .quad(self.transform, local, texture, uv, self.masks);
        } else {
            let nx = (size[0] / uv.w).ceil() as u32;
            let ny = (size[1] / uv.h).ceil() as u32;
            if nx.saturating_mul(ny) <= 65_536 {
                for j in 0..ny {
                    for i in 0..nx {
                        let dx = i as f32 * uv.w;
                        let dy = j as f32 * uv.h;
                        let target = Rect::new(
                            local.x + dx,
                            local.y + dy,
                            (local.w - dx).min(uv.w),
                            (local.h - dy).min(uv.h),
                        );
                        self.painter.quad(
                            self.transform,
                            target,
                            texture.clone(),
                            Rect::new(uv.x, uv.y, target.w, target.h),
                            self.masks,
                        );
                    }
                }
            }
        }
        Some(self.transform.bounds(local))
    }
    pub fn nine_grid(&mut self, data: &NineGrid, key: SpriteKey, extra_width: f32) -> Option<Rect> {
        self.painter.surface.additive = data.graphic.blend == 1;
        let img = raster::sprite(
            self.resources,
            &data.graphic,
            key,
            self.ink,
            self.painter.cx,
        )?;
        let source = dimensions(&img);
        let scale = self.transform.scale();
        if scale.iter().any(|&axis| axis < 0.00001) {
            return None;
        }
        let transform = self.transform.normalized();
        let mut center = if data.use_sprite_size {
            source
        } else {
            data.center
        };
        center[0] += extra_width;
        let mut dst = [[0.; 4]; 2];
        let mut uv = [[[0.; 2]; 3]; 2];
        for i in 0..2 {
            let raw = source[i];
            let border = data.border[i];
            let total = if data.scale_mode == 0 {
                center[i] * scale[i] + 2. * border
            } else {
                center[i] * scale[i]
            };
            let edge = border.min(if data.scale_mode == 0 {
                raw * 0.25
            } else {
                center[i] * 0.5
            });
            let src_edge = (border * 2.).min(raw * 0.5);
            dst[i] = [0., edge, total - edge, total];
            uv[i] = [
                [0., src_edge],
                [src_edge, (raw.min(total * 2.) - src_edge).max(src_edge)],
                [raw - src_edge, raw],
            ];
        }
        let bounds = anchored([dst[0][3], dst[1][3]], data.graphic.anchor);
        for row in 0..3 {
            for col in 0..3 {
                let rect = Rect::new(
                    bounds.x + dst[0][col],
                    bounds.y + dst[1][row],
                    dst[0][col + 1] - dst[0][col],
                    dst[1][row + 1] - dst[1][row],
                );
                if rect.w <= 0. || rect.h <= 0. {
                    continue;
                }
                let [x, u] = uv[0][col];
                let [y, v] = uv[1][row];
                let (x, w) = fixed_uv(x, u, source[0]);
                let (y, h) = fixed_uv(y, v, source[1]);
                self.painter.quad(
                    transform,
                    rect,
                    img.clone(),
                    Rect::new(x, y, w, h),
                    self.masks,
                );
            }
        }
        Some(transform.bounds(bounds))
    }
    #[cfg(feature = "uid-debug")]
    pub fn text(&mut self, text: &str, font: u16, anchor: [u8; 2]) -> Option<Rect> {
        self.label(text, font, anchor, 0., [0.; 2], self.ink.color)
    }
    pub fn label(
        &mut self,
        text: &str,
        font: u16,
        anchor: [u8; 2],
        spacing: f32,
        offset: [f32; 2],
        color: [f32; 4],
    ) -> Option<Rect> {
        self.painter.surface.additive = false;
        let img = raster::text_image_spaced(text, font, spacing, color, self.painter.cx)?;
        let [w, h] = dimensions(&img);
        let mut bounds = anchored([w - 8., h], anchor);
        bounds.w = w;
        bounds.x += offset[0];
        bounds.y += offset[1];
        self.painter.quad(
            self.transform,
            bounds,
            img,
            Rect::new(0., 0., w, h),
            self.masks,
        );
        Some(self.transform.bounds(bounds))
    }
}

fn fixed_uv(start: f32, end: f32, limit: f32) -> (f32, f32) {
    if end > start {
        (start, end - start)
    } else {
        ((start - 0.5).clamp(0., (limit - 1.).max(0.)), 1.)
    }
}
