use crate::{
    capture,
    draw::{Prepared, Rect},
};
use gpui::{App, Window, fill, rgba};

#[derive(Clone, Copy, Debug)]
pub struct Lens {
    pub distortion: [f32; 3],
    pub stretch: f32,
    pub shift: [[f32; 2]; 3],
    pub dark_edges: bool,
    pub monochrome: f32,
}

impl Default for Lens {
    fn default() -> Self {
        Self {
            distortion: [-7.; 3],
            stretch: 3.,
            shift: [[0.; 2]; 3],
            dark_edges: false,
            monochrome: 0.,
        }
    }
}

impl Lens {
    pub fn eye_loupe() -> Self {
        Self {
            distortion: [-500.; 3],
            stretch: 1.,
            dark_edges: true,
            ..Self::default()
        }
    }
    pub fn cathode_ray_tube() -> Self {
        Self {
            distortion: [-700.; 3],
            stretch: 10.,
            dark_edges: true,
            ..Self::default()
        }
    }
    pub fn chromatic() -> Self {
        Self {
            distortion: [80., 20., 30.],
            stretch: 4.,
            shift: [[2., 2.], [0., 0.], [0., 0.]],
            monochrome: 1.3,
            dark_edges: false,
        }
    }
    pub(crate) fn key(&self) -> impl std::hash::Hash + use<> {
        (
            self.distortion.map(f32::to_bits),
            self.stretch.to_bits(),
            self.shift.map(|v| v.map(f32::to_bits)),
            self.dark_edges,
            self.monochrome.to_bits(),
        )
    }
    fn offset(&self, x: f32, y: f32, screen: Rect) -> [f32; 2] {
        let dx = x - screen.w / 2.;
        let dy = y - screen.h / 2.;
        let r = dx.hypot(dy);
        if r == 0. {
            return [0.; 2];
        }
        let radius = (screen.w / 2.).hypot(screen.h / 2.);
        let offset = (r / radius).powf(self.stretch) * screen.h / screen.w / r;
        [dx * offset, dy * offset]
    }
    fn source(&self, x: f32, y: f32, offset: [f32; 2], channel: usize) -> [f32; 2] {
        [
            x - offset[0] * self.distortion[channel] - self.shift[channel][0],
            y - offset[1] * self.distortion[channel] - self.shift[channel][1],
        ]
    }
    pub(crate) fn pixel(
        &self,
        pixels: &[[u8; 4]],
        width: usize,
        height: usize,
        screen: Rect,
        x: f32,
        y: f32,
    ) -> [u8; 4] {
        let offset = self.offset(x, y, screen);
        let mut result = [0, 0, 0, 255];
        for c in 0..3 {
            let [u, v] = self.source(x, y, offset, c);
            if self.dark_edges && (u < 0. || u > screen.w || v < 0. || v > screen.h) {
                continue;
            }
            let color = capture::bilinear(
                pixels,
                width,
                height,
                u * width as f32 / screen.w - 0.5,
                v * height as f32 / screen.h - 0.5,
            );
            let channel = if self.monochrome != 0. {
                (0.299 * color[0] + 0.587 * color[1] + 0.114 * color[2]) * self.monochrome
            } else {
                color[c]
            };
            result[c] = channel.round() as u8;
        }
        result
    }
    pub(crate) fn paint(
        &self,
        draws: &[Prepared<'_>],
        screen: Rect,
        dst: Rect,
        window: &mut Window,
        cx: &mut App,
    ) {
        let fringe = self.distortion[0] != self.distortion[1] || self.shift[0] != self.shift[1];
        let tints = if fringe {
            draws
                .iter()
                .map(|draw| {
                    [
                        draw.tinted([93, 129, 132], cx),
                        draw.tinted([154, 96, 84], cx),
                    ]
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        if self.dark_edges {
            window.paint_quad(fill(dst.bounds(), rgba(0x262821ff)));
        }
        let linear = (self.stretch - 1.).abs() < 0.001;
        let cols = if linear {
            1
        } else {
            (dst.w / 66.).ceil().clamp(6., 24.) as usize
        };
        let rows = if linear {
            1
        } else {
            (dst.h / 58.).ceil().clamp(4., 16.) as usize
        };
        let source = |x: f32, y: f32| self.source(x, y, self.offset(x, y, screen), 1);
        for row in 0..rows {
            for col in 0..cols {
                let tile = Rect::new(
                    screen.x + screen.w * col as f32 / cols as f32,
                    screen.y + screen.h * row as f32 / rows as f32,
                    screen.w / cols as f32,
                    screen.h / rows as f32,
                );
                let center = [tile.x + tile.w / 2., tile.y + tile.h / 2.];
                let left = source(tile.x, center[1])[0];
                let right = source(tile.x + tile.w, center[1])[0];
                let top = source(center[0], tile.y)[1];
                let bottom = source(center[0], tile.y + tile.h)[1];
                if right <= left || bottom <= top {
                    continue;
                }
                let region = Rect::new(left, top, right - left, bottom - top);
                let dest = tile.mapped(screen, dst);
                let direction = self.offset(center[0], center[1], screen);
                for (i, draw) in draws.iter().enumerate() {
                    if fringe {
                        for (tint_index, c) in [2, 0].into_iter().enumerate() {
                            if let Some(img) = &tints[i][tint_index] {
                                let displacement = |axis: usize| {
                                    direction[axis] * (self.distortion[c] - self.distortion[1])
                                        + self.shift[c][axis]
                                        - self.shift[1][axis]
                                };
                                let shifted = Rect::new(
                                    left - displacement(0),
                                    top - displacement(1),
                                    region.w,
                                    region.h,
                                );
                                draw.paint_tinted(shifted, dest, window, img.clone());
                            }
                        }
                    }
                    draw.paint(region, dest, window);
                }
            }
        }
    }
}
