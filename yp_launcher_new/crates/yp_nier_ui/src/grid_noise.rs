use crate::Lens;
use crate::{
    capture::pixels,
    draw::{Prepared, Rect},
};
use gpui::{App, Global, Window, fill, rgba};
use std::{
    collections::{VecDeque, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Clone, Copy, Debug)]
pub struct GridNoise {
    pub cell: [f32; 2],
    pub levels: u8,
    pub monochrome: f32,
    pub blur_strength: f32,
}

impl Default for GridNoise {
    fn default() -> Self {
        Self {
            cell: [6., 6.],
            levels: 8,
            monochrome: 0.,
            blur_strength: 2.,
        }
    }
}

type Strips = Rc<Vec<(Rect, u32)>>;
#[derive(Default)]
struct Cache(VecDeque<(u64, Strips)>);
impl Global for Cache {}

fn kernel(strength: f32) -> [f32; 8] {
    let sigma_squared = (1. + strength).powi(2);
    let taps: [f32; 8] = std::array::from_fn(|i| {
        let odd = (2 * i + 1) as f32;
        (-0.5 * odd * odd / sigma_squared).exp()
    });
    let sum = 2. * taps.iter().sum::<f32>();
    taps.map(|w| w / sum)
}

fn filter_at(
    position: f32,
    limit: usize,
    weights: &[f32; 8],
    mut pixel: impl FnMut(usize) -> [f32; 4],
) -> [f32; 4] {
    let mut result = [0.; 4];
    for (i, &weight) in weights.iter().enumerate() {
        let d = i as f32 + 0.5;
        let a = pixel((position - d).clamp(0., (limit - 1) as f32) as usize);
        let b = pixel((position + d).clamp(0., (limit - 1) as f32) as usize);
        for k in 0..4 {
            result[k] += (a[k] + b[k]) * weight;
        }
    }
    result
}

pub(crate) fn paint_blur(
    strength: f32,
    draws: &[Prepared<'_>],
    screen: Rect,
    dst: Rect,
    window: &mut Window,
    cx: &mut App,
) {
    let (dims, _) = crate::capture::dimensions(dst, window.scale_factor());
    let img = crate::capture::filtered(
        draws,
        screen,
        dims,
        ("gaussian", strength.to_bits()),
        cx,
        |pixels, w, h| {
            let weights = kernel(strength);
            let mut rows = Vec::with_capacity(w * h);
            for line in pixels.chunks_exact(w) {
                for x in 0..w {
                    rows.push(filter_at(x as f32, w, &weights, |i| line[i].map(f32::from)));
                }
            }
            let mut out = Vec::with_capacity(w * h * 4);
            for y in 0..h {
                for x in 0..w {
                    let [r, g, b, a] = filter_at(y as f32, h, &weights, |i| rows[i * w + x])
                        .map(|v| v.round() as u8);
                    out.extend_from_slice(&[b, g, r, a]);
                }
            }
            out
        },
    );
    let _ = window.paint_image(
        dst.bounds(),
        dst.bounds(),
        Default::default(),
        img,
        0,
        false,
    );
}

impl GridNoise {
    fn quantize(&self, pixel: [f32; 4]) -> u32 {
        let levels = if self.levels == 0 {
            255.
        } else {
            f32::from(self.levels)
        };
        let mut c = pixel.map(|x| (x * levels / 255.).round() * 255. / levels);
        if self.monochrome > 0. {
            let y = (0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]) * self.monochrome;
            c[..3].fill(y);
        }
        c[3] = pixel[3];
        u32::from_be_bytes(c.map(|v| v.round() as u8))
    }

    fn strips(
        &self,
        draws: &[Prepared<'_>],
        lens: Option<&Lens>,
        screen: Rect,
        dims: [usize; 2],
        cell: [f32; 2],
    ) -> Strips {
        let [width, height] = dims;
        let mut samples = pixels(draws, screen, width, height);
        if let Some(lens) = lens {
            samples = (0..width * height)
                .map(|i| {
                    lens.pixel(
                        &samples,
                        width,
                        height,
                        screen,
                        (i % width) as f32 * cell[0] + cell[0] * 0.5,
                        (i / width) as f32 * cell[1] + cell[1] * 0.5,
                    )
                })
                .collect();
        }
        let blend = (self.blur_strength * 0.075).clamp(0., 0.23);
        let mut output = Vec::new();
        for y in 0..height {
            let mut start = 0;
            let mut previous = None;
            for x in 0..width {
                let i = y * width + x;
                let base = samples[i];
                let value = if blend > 0. {
                    let near = [
                        y * width + x.saturating_sub(1),
                        y * width + (x + 1).min(width - 1),
                        y.saturating_sub(1) * width + x,
                        (y + 1).min(height - 1) * width + x,
                    ];
                    std::array::from_fn(|c| {
                        f32::from(base[c]) * (1. - 4. * blend)
                            + near
                                .iter()
                                .map(|&p| f32::from(samples[p][c]) * blend)
                                .sum::<f32>()
                    })
                } else {
                    base.map(f32::from)
                };
                let color = self.quantize(value);
                if previous != Some(color) {
                    if let Some(old) = previous {
                        output.push((
                            Rect::new(
                                start as f32 * cell[0],
                                y as f32 * cell[1],
                                (x - start) as f32 * cell[0],
                                cell[1],
                            ),
                            old,
                        ));
                    }
                    previous = Some(color);
                    start = x;
                }
            }
            if let Some(last) = previous {
                output.push((
                    Rect::new(
                        start as f32 * cell[0],
                        y as f32 * cell[1],
                        (width - start) as f32 * cell[0],
                        cell[1],
                    ),
                    last,
                ));
            }
        }
        Rc::new(output)
    }

    pub(crate) fn paint(
        &self,
        draws: &[Prepared<'_>],
        lens: Option<&Lens>,
        screen: Rect,
        dst: Rect,
        window: &mut Window,
        cx: &mut App,
    ) {
        let dims = [
            (screen.w / self.cell[0].max(1.)).ceil().clamp(1., 256.) as usize,
            (screen.h / self.cell[1].max(1.)).ceil().clamp(1., 144.) as usize,
        ];
        let cell = [screen.w / dims[0] as f32, screen.h / dims[1] as f32];
        let mut hash = DefaultHasher::new();
        (
            dims,
            cell.map(f32::to_bits),
            self.levels,
            self.monochrome.to_bits(),
            self.blur_strength.to_bits(),
            screen.w.to_bits(),
            screen.h.to_bits(),
        )
            .hash(&mut hash);
        lens.map(Lens::key).hash(&mut hash);
        for draw in draws {
            draw.fingerprint(&mut hash);
        }
        let key = hash.finish();
        if !cx.has_global::<Cache>() {
            cx.set_global(Cache::default());
        }
        let cache = &mut cx.global_mut::<Cache>().0;
        let strips = if let Some((_, strips)) = cache.iter().find(|(k, _)| *k == key) {
            strips.clone()
        } else {
            let strips = self.strips(draws, lens, screen, dims, cell);
            if cache.len() >= 4 {
                cache.pop_front();
            }
            cache.push_back((key, strips.clone()));
            strips
        };
        for &(rect, color) in strips.iter() {
            window.paint_quad(fill(rect.mapped(screen, dst).bounds(), rgba(color)));
        }
    }
}
