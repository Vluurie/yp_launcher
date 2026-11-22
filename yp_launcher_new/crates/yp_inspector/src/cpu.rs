use std::sync::Mutex;

use yp_formats::wmb3::{Mesh, Model, Texture};

use crate::error::{InspectorError, Result};

pub const MIN_SIDE: u32 = 16;
pub const MAX_PIXELS: usize = 2_073_600;
const MIN_DISTANCE: f32 = 0.9;
const MAX_DISTANCE: f32 = 12.0;
const NEAR_PLANE: f32 = 0.02;
const FOCAL_SCALE: f32 = 0.92;
const LIGHT: [f32; 3] = [-0.35, 0.75, 0.55];
const AMBIENT: f32 = 0.35;
const DIFFUSE: f32 = 0.65;
const ALPHA_CUTOFF: u8 = 128;
const ORBIT_SPEED: f32 = 0.01;
const ZOOM_SPEED: f32 = 0.001;
const MAX_PITCH: f32 = 1.45;
const MIN_AREA: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

impl Camera {
    pub const INSPECTOR: Camera = Camera {
        yaw: std::f32::consts::PI,
        pitch: -0.18,
        distance: 3.2,
        pan_x: 0.0,
        pan_y: 0.0,
    };

    pub const PREVIEW: Camera = Camera {
        distance: 2.6,
        ..Camera::INSPECTOR
    };

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * ORBIT_SPEED;
        self.pitch = (self.pitch + dy * ORBIT_SPEED).clamp(-MAX_PITCH, MAX_PITCH);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    pub fn zoom(&mut self, scroll_dy: f32) {
        self.distance =
            (self.distance * (scroll_dy * ZOOM_SPEED).exp()).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::INSPECTOR
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderStyle {
    pub background: [u8; 4],
    pub untextured: [u8; 4],
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            background: [25, 27, 26, 255],
            untextured: [196, 190, 154, 255],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl RenderFrame {
    pub fn into_bgra(mut self) -> Vec<u8> {
        for pixel in self.rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        self.rgba
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y as usize * self.width as usize + x as usize) * 4;
        self.rgba.get(index..index + 4)?.try_into().ok()
    }
}

type Projected = [f32; 8];

const HIDDEN: Projected = [0.0; 8];

struct Corner {
    x: f32,
    y: f32,
    inverse_depth: f32,
    normal: [f32; 3],
    uv: [f32; 2],
}

impl Corner {
    fn read(vertex: &Projected) -> Option<Self> {
        let [x, y, inverse_depth, normal_x, normal_y, normal_z, u, v] = *vertex;
        (inverse_depth > 0.0).then_some(Self {
            x,
            y,
            inverse_depth,
            normal: [normal_x, normal_y, normal_z],
            uv: [u, v],
        })
    }
}

fn corners(vertices: &[Projected], triangle: &[u32]) -> Option<[Corner; 3]> {
    let &[a, b, c] = triangle else {
        return None;
    };
    let corner = |index: u32| vertices.get(index as usize).and_then(Corner::read);
    Some([corner(a)?, corner(b)?, corner(c)?])
}

struct Orientation {
    sin_yaw: f32,
    cos_yaw: f32,
    sin_pitch: f32,
    cos_pitch: f32,
}

impl Orientation {
    fn new(camera: &Camera) -> Self {
        Self {
            sin_yaw: camera.yaw.sin(),
            cos_yaw: camera.yaw.cos(),
            sin_pitch: camera.pitch.sin(),
            cos_pitch: camera.pitch.cos(),
        }
    }

    fn rotate(&self, v: [f32; 3]) -> [f32; 3] {
        let x = self.cos_yaw * v[0] + self.sin_yaw * v[2];
        let yaw_z = -self.sin_yaw * v[0] + self.cos_yaw * v[2];
        let y = self.cos_pitch * v[1] - self.sin_pitch * yaw_z;
        let z = self.sin_pitch * v[1] + self.cos_pitch * yaw_z;
        [x, y, z]
    }
}

struct View {
    orientation: Orientation,
    center: [f32; 3],
    scale: f32,
    distance: f32,
    focal: f32,
    center_x: f32,
    center_y: f32,
}

impl View {
    fn new(model: &Model, camera: &Camera, width: u32, height: u32) -> Self {
        Self {
            orientation: Orientation::new(camera),
            center: model.center,
            scale: model.radius.recip(),
            distance: camera.distance.clamp(MIN_DISTANCE, MAX_DISTANCE),
            focal: width.min(height) as f32 * FOCAL_SCALE,
            center_x: width as f32 * 0.5 + camera.pan_x,
            center_y: height as f32 * 0.5 + camera.pan_y,
        }
    }

    fn project(&self, position: &[f32; 3], normal: &[f32; 3], uv: &[f32; 2]) -> Projected {
        let rotated = self.orientation.rotate([
            (position[0] - self.center[0]) * self.scale,
            (position[1] - self.center[1]) * self.scale,
            (position[2] - self.center[2]) * self.scale,
        ]);
        let camera_z = rotated[2] + self.distance;
        if camera_z.is_nan() || camera_z <= NEAR_PLANE {
            return HIDDEN;
        }
        let inverse_depth = camera_z.recip();
        let zoom = self.focal * inverse_depth;
        let [normal_x, normal_y, normal_z] = self.orientation.rotate(*normal);
        [
            self.center_x + rotated[0] * zoom,
            self.center_y - rotated[1] * zoom,
            inverse_depth,
            normal_x * inverse_depth,
            normal_y * inverse_depth,
            normal_z * inverse_depth,
            uv[0] * inverse_depth,
            uv[1] * inverse_depth,
        ]
    }
}

#[derive(Clone, Copy)]
struct Extent {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

impl Extent {
    const EMPTY: Extent = Extent {
        left: f32::INFINITY,
        right: f32::NEG_INFINITY,
        top: f32::INFINITY,
        bottom: f32::NEG_INFINITY,
    };

    fn around(indices: &[u32], vertices: &[Projected]) -> Self {
        let mut extent = Self::EMPTY;
        for triangle in indices.chunks_exact(3) {
            let Some(corners) = corners(vertices, triangle) else {
                continue;
            };
            for corner in &corners {
                extent.left = extent.left.min(corner.x);
                extent.right = extent.right.max(corner.x);
                extent.top = extent.top.min(corner.y);
                extent.bottom = extent.bottom.max(corner.y);
            }
        }
        extent
    }
}

fn sampled(low: f32, high: f32, (first, last): (f32, f32)) -> Option<(u32, u32)> {
    let low = (low - 0.5).max(first);
    let high = (high - 0.5).min(last);
    if low.is_nan() || high.is_nan() || low > high {
        return None;
    }
    let floor = low as u32;
    let start = if (floor as f32) < low { floor + 1 } else { floor };
    let end = high as u32;
    (start <= end).then_some((start, end))
}

struct Projection<'a> {
    vertices: &'a [Projected],
    indices: &'a [u32],
    extents: &'a [Extent],
    texture: Option<&'a Texture>,
}

const MAX_RENDER_THREADS: usize = 8;
const MIN_ROWS_PER_BAND: u32 = 32;
const BANDS_PER_THREAD: u32 = 4;
const VERTEX_CHUNK: usize = 16_384;
const BATCH_INDICES: usize = 3 * 256;
const BATCHES_PER_JOB: usize = 64;
const NARROW_REACH: u32 = 4;

pub fn validate_size(width: u32, height: u32) -> Result<usize> {
    if width < MIN_SIDE || height < MIN_SIDE {
        return Err(InspectorError::RenderTooSmall { width, height });
    }
    let pixels = (width as usize)
        .checked_mul(height as usize)
        .filter(|pixels| *pixels <= MAX_PIXELS)
        .ok_or(InspectorError::RenderTooLarge { width, height })?;
    Ok(pixels)
}

pub fn render(
    model: &Model,
    visible: &[bool],
    lod: u32,
    camera: &Camera,
    width: u32,
    height: u32,
    style: &RenderStyle,
) -> Result<RenderFrame> {
    render_in_bands(
        model,
        visible,
        lod,
        camera,
        (width, height),
        style,
        band_count(height),
    )
}

fn render_in_bands(
    model: &Model,
    visible: &[bool],
    lod: u32,
    camera: &Camera,
    (width, height): (u32, u32),
    style: &RenderStyle,
    bands: u32,
) -> Result<RenderFrame> {
    let pixel_count = validate_size(width, height)?;
    let threads = thread_count();
    let view = View::new(model, camera, width, height);
    let shown: Vec<&Mesh> = model
        .meshes
        .iter()
        .enumerate()
        .filter(|(position, mesh)| {
            visible.get(*position).copied().unwrap_or(false) && mesh.lod_index == lod
        })
        .map(|(_, mesh)| mesh)
        .collect();
    let vertices = project_vertices(&shown, &view, threads);
    let extents = measure_batches(&shown, &vertices, threads);
    let projections: Vec<Projection<'_>> = shown
        .iter()
        .zip(&vertices)
        .zip(&extents)
        .map(|((mesh, vertices), extents)| Projection {
            vertices,
            indices: &mesh.indices,
            extents,
            texture: mesh
                .albedo_texture_id
                .and_then(|id| model.textures.get(&id)),
        })
        .collect();

    let mut rgba = vec![0_u8; pixel_count * 4];
    let mut depth = vec![0.0_f32; pixel_count];
    let band_rows = height.div_ceil(bands.clamp(1, height)) as usize;
    let row_cells = width as usize;
    let jobs: Vec<_> = rgba
        .chunks_mut(band_rows * row_cells * 4)
        .zip(depth.chunks_mut(band_rows * row_cells))
        .enumerate()
        .collect();
    run_jobs(jobs, threads, |(band, (rgba, depth))| {
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.copy_from_slice(&style.background);
        }
        let first_row = (band * band_rows) as u32;
        let last_row = first_row + (depth.len() / row_cells) as u32 - 1;
        let mut target = Target {
            width: row_cells,
            first_row,
            rows: (first_row as f32, last_row as f32),
            columns: (0.0, (width - 1) as f32),
            rgba,
            depth,
            untextured: style.untextured,
        };
        for projection in &projections {
            target.draw_mesh(projection);
        }
    });

    Ok(RenderFrame {
        width,
        height,
        rgba,
    })
}

fn project_vertices(shown: &[&Mesh], view: &View, threads: usize) -> Vec<Vec<Projected>> {
    let mut buffers: Vec<Vec<Projected>> = shown
        .iter()
        .map(|mesh| {
            let count = mesh
                .positions
                .len()
                .min(mesh.normals.len())
                .min(mesh.uvs.len());
            vec![HIDDEN; count]
        })
        .collect();
    let jobs: Vec<_> = buffers
        .iter_mut()
        .zip(shown)
        .flat_map(|(buffer, mesh)| {
            buffer
                .chunks_mut(VERTEX_CHUNK)
                .zip(mesh.positions.chunks(VERTEX_CHUNK))
                .zip(mesh.normals.chunks(VERTEX_CHUNK))
                .zip(mesh.uvs.chunks(VERTEX_CHUNK))
        })
        .collect();
    run_jobs(jobs, threads, |(((slots, positions), normals), uvs)| {
        for (((slot, position), normal), uv) in
            slots.iter_mut().zip(positions).zip(normals).zip(uvs)
        {
            *slot = view.project(position, normal, uv);
        }
    });
    buffers
}

fn measure_batches(
    shown: &[&Mesh],
    vertices: &[Vec<Projected>],
    threads: usize,
) -> Vec<Vec<Extent>> {
    let mut extents: Vec<Vec<Extent>> = shown
        .iter()
        .map(|mesh| vec![Extent::EMPTY; mesh.indices.len().div_ceil(BATCH_INDICES)])
        .collect();
    let jobs: Vec<_> = extents
        .iter_mut()
        .zip(shown)
        .zip(vertices)
        .flat_map(|((extents, mesh), vertices)| {
            extents
                .chunks_mut(BATCHES_PER_JOB)
                .zip(mesh.indices.chunks(BATCH_INDICES * BATCHES_PER_JOB))
                .map(move |(slots, indices)| (slots, indices, vertices.as_slice()))
        })
        .collect();
    run_jobs(jobs, threads, |(slots, indices, vertices)| {
        for (slot, batch) in slots.iter_mut().zip(indices.chunks(BATCH_INDICES)) {
            *slot = Extent::around(batch, vertices);
        }
    });
    extents
}

fn run_jobs<T: Send>(jobs: Vec<T>, threads: usize, work: impl Fn(T) + Sync) {
    let helpers = threads.min(jobs.len()).saturating_sub(1);
    if helpers == 0 {
        jobs.into_iter().for_each(work);
        return;
    }
    let queue = Mutex::new(jobs.into_iter());
    let drain = || {
        while let Some(job) = queue.lock().ok().and_then(|mut jobs| jobs.next()) {
            work(job);
        }
    };
    std::thread::scope(|scope| {
        for _ in 0..helpers {
            scope.spawn(drain);
        }
        drain();
    });
}

fn thread_count() -> usize {
    std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(MAX_RENDER_THREADS)
}

fn band_count(height: u32) -> u32 {
    (thread_count() as u32 * BANDS_PER_THREAD)
        .min(height / MIN_ROWS_PER_BAND)
        .max(1)
}

fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn covered(weights: [f32; 3], steps: [f32; 3], reach: u32) -> Option<(u32, u32)> {
    let mut low = 0.0_f32;
    let mut high = reach as f32;
    for (weight, step) in weights.into_iter().zip(steps) {
        if step > 0.0 {
            low = low.max(-weight / step);
        } else if step < 0.0 {
            high = high.min(-weight / step);
        } else if weight.is_nan() || weight < 0.0 {
            return None;
        }
    }
    let first = (low - 1.0).max(0.0);
    let last = (high + 1.0).min(reach as f32);
    (first <= last).then_some((first as u32, last as u32))
}

fn blend<const N: usize>(weights: [f32; 3], values: &[[f32; N]; 3]) -> [f32; N] {
    std::array::from_fn(|slot| {
        weights[0] * values[0][slot] + weights[1] * values[1][slot] + weights[2] * values[2][slot]
    })
}

struct Surface<'a> {
    normals: [[f32; 3]; 3],
    uvs: [[f32; 2]; 3],
    texture: Option<&'a Texture>,
    untextured: [u8; 4],
}

impl Surface<'_> {
    fn shade(&self, weights: [f32; 3], inverse_depth: f32) -> Option<[u8; 4]> {
        let sampled = match self.texture {
            Some(texture) => {
                let depth = inverse_depth.recip();
                let [u, v] = blend(weights, &self.uvs);
                sample_texture(texture, u * depth, v * depth, self.untextured)
            }
            None => self.untextured,
        };
        if sampled[3] < ALPHA_CUTOFF {
            return None;
        }
        let normal = blend(weights, &self.normals);
        let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let light = if length > f32::EPSILON {
            ((normal[0] * LIGHT[0] + normal[1] * LIGHT[1] + normal[2] * LIGHT[2]) / length)
                .abs()
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        let intensity = AMBIENT + light * DIFFUSE;
        let lit = |channel: u8| (f32::from(channel) * intensity).min(255.0) as u8;
        Some([lit(sampled[0]), lit(sampled[1]), lit(sampled[2]), 255])
    }
}

struct Target<'a> {
    width: usize,
    first_row: u32,
    rows: (f32, f32),
    columns: (f32, f32),
    rgba: &'a mut [u8],
    depth: &'a mut [f32],
    untextured: [u8; 4],
}

impl Target<'_> {
    fn draw_mesh(&mut self, projection: &Projection<'_>) {
        let batches = projection.indices.chunks(BATCH_INDICES);
        for (batch, extent) in batches.zip(projection.extents) {
            if sampled(extent.top, extent.bottom, self.rows).is_none()
                || sampled(extent.left, extent.right, self.columns).is_none()
            {
                continue;
            }
            for triangle in batch.chunks_exact(3) {
                if let Some(corners) = corners(projection.vertices, triangle) {
                    self.draw_triangle(corners, projection.texture);
                }
            }
        }
    }

    fn draw_triangle(&mut self, [a, b, c]: [Corner; 3], texture: Option<&Texture>) {
        let top = a.y.min(b.y).min(c.y);
        let bottom = a.y.max(b.y).max(c.y);
        let Some((first_row, last_row)) = sampled(top, bottom, self.rows) else {
            return;
        };
        let left = a.x.min(b.x).min(c.x);
        let right = a.x.max(b.x).max(c.x);
        let Some((first_column, last_column)) = sampled(left, right, self.columns) else {
            return;
        };
        let area = edge(a.x, a.y, b.x, b.y, c.x, c.y);
        if !area.is_finite() || area.abs() < MIN_AREA {
            return;
        }
        let reach = last_column - first_column;
        let inverse_area = area.recip();
        let across = [(c.y - b.y) * inverse_area, (a.y - c.y) * inverse_area];
        let down = [(b.x - c.x) * inverse_area, (c.x - a.x) * inverse_area];
        let origin = first_column as f32 + 0.5;
        let start = [(origin - b.x) * across[0], (origin - c.x) * across[1]];
        let steps = [across[0], across[1], -(across[0] + across[1])];
        let inverse_depths = [[a.inverse_depth], [b.inverse_depth], [c.inverse_depth]];
        let surface = Surface {
            normals: [a.normal, b.normal, c.normal],
            uvs: [a.uv, b.uv, c.uv],
            texture,
            untextured: self.untextured,
        };
        for row in first_row..=last_row {
            let sample = row as f32 + 0.5;
            let row_a = start[0] + (sample - b.y) * down[0];
            let row_b = start[1] + (sample - c.y) * down[1];
            let span = if reach < NARROW_REACH {
                Some((0, reach))
            } else {
                covered([row_a, row_b, 1.0 - row_a - row_b], steps, reach)
            };
            let Some((first, last)) = span else {
                continue;
            };
            let offset =
                (row - self.first_row) as usize * self.width + (first_column + first) as usize;
            let count = (last - first) as usize + 1;
            let depths = &mut self.depth[offset..offset + count];
            let colors = self.rgba[offset * 4..(offset + count) * 4].chunks_exact_mut(4);
            for ((column, depth), color) in (first..last + 1).zip(depths).zip(colors) {
                let along = column as f32;
                let weight_a = row_a + along * across[0];
                let weight_b = row_b + along * across[1];
                let weight_c = 1.0 - weight_a - weight_b;
                let inside = weight_a >= 0.0 && weight_b >= 0.0 && weight_c >= 0.0;
                if !inside {
                    continue;
                }
                let weights = [weight_a, weight_b, weight_c];
                let [inverse_depth] = blend(weights, &inverse_depths);
                let nearer = inverse_depth > *depth;
                if !nearer {
                    continue;
                }
                if let Some(pixel) = surface.shade(weights, inverse_depth) {
                    *depth = inverse_depth;
                    color.copy_from_slice(&pixel);
                }
            }
        }
    }
}

fn wrap(coordinate: f32, size: u32) -> u32 {
    let scaled = coordinate * size as f32;
    let truncated = scaled as i32;
    let floored = if truncated as f32 > scaled {
        truncated.wrapping_sub(1)
    } else {
        truncated
    };
    if size.is_power_of_two() {
        floored as u32 & (size - 1)
    } else {
        floored.rem_euclid(size.min(i32::MAX as u32) as i32) as u32
    }
}

fn sample_texture(texture: &Texture, u: f32, v: f32, fallback: [u8; 4]) -> [u8; 4] {
    if texture.width == 0 || texture.height == 0 {
        return fallback;
    }
    let x = wrap(u, texture.width);
    let y = wrap(v, texture.height);
    let index = (y as usize * texture.width as usize + x as usize) * 4;
    texture
        .rgba
        .get(index..index + 4)
        .and_then(|pixel| pixel.try_into().ok())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_identical_however_many_bands_draw_them() {
        let bytes = yp_formats::testing::cube_spec(&["Body", "Skirt"]).build();
        let model = Model::parse(&bytes, None).unwrap();
        let visible = vec![true; model.meshes.len()];
        let style = RenderStyle::default();
        let mut turned = Camera::default();
        turned.orbit(40.0, -25.0);
        turned.zoom(-2.0);
        let mut close = turned;
        close.zoom(-100_000.0);
        close.pan(-30.0, 12.0);
        let size = (200, 150);
        for camera in [turned, close] {
            let single = render_in_bands(&model, &visible, 0, &camera, size, &style, 1).unwrap();
            assert!(single.rgba.chunks_exact(4).any(|pixel| pixel != style.background));
            for bands in [2, 3, 4, 7, 8, 150, 1000] {
                let banded =
                    render_in_bands(&model, &visible, 0, &camera, size, &style, bands).unwrap();
                assert_eq!(banded.rgba, single.rgba, "{bands} bands");
            }
        }
        assert!(band_count(16) == 1);
        assert!(band_count(4000) >= 1);
    }

    #[test]
    fn spans_bracket_the_covered_columns() {
        assert_eq!(covered([-4.0, 1.0, 4.0], [1.0, 0.0, -1.0], 100), Some((3, 5)));
        assert_eq!(covered([0.5, 0.25, 0.25], [0.0, 0.0, 0.0], 9), Some((0, 9)));
        assert_eq!(covered([-1.0, 1.0, 1.0], [0.0, 0.5, -0.5], 9), None);
        assert_eq!(covered([-500.0, 1.0, 1.0], [1.0, 0.0, -1.0], 9), None);
        assert_eq!(covered([f32::NAN, 1.0, 1.0], [0.0, 1.0, -1.0], 9), None);
    }

    #[test]
    fn size_limits() {
        assert!(matches!(
            validate_size(15, 100),
            Err(InspectorError::RenderTooSmall { .. })
        ));
        assert!(matches!(
            validate_size(4000, 4000),
            Err(InspectorError::RenderTooLarge { .. })
        ));
        assert_eq!(validate_size(1920, 1080).unwrap(), MAX_PIXELS);
    }

    #[test]
    fn textures_wrap_and_fall_back() {
        let texture = Texture {
            width: 2,
            height: 1,
            rgba: vec![1, 2, 3, 255, 4, 5, 6, 255],
        };
        let fallback = [9, 9, 9, 9];
        assert_eq!(sample_texture(&texture, 0.1, 0.0, fallback), [1, 2, 3, 255]);
        assert_eq!(sample_texture(&texture, 0.6, 0.0, fallback), [4, 5, 6, 255]);
        assert_eq!(
            sample_texture(&texture, -0.4, 3.0, fallback),
            [4, 5, 6, 255]
        );
        let odd = Texture {
            width: 3,
            height: 1,
            rgba: vec![1, 1, 1, 255, 2, 2, 2, 255, 3, 3, 3, 255],
        };
        assert_eq!(sample_texture(&odd, 0.5, 0.0, fallback), [2, 2, 2, 255]);
        assert_eq!(sample_texture(&odd, -0.1, 0.0, fallback), [3, 3, 3, 255]);
        assert_eq!(sample_texture(&odd, 1.4, 0.0, fallback), [2, 2, 2, 255]);
        let empty = Texture {
            width: 0,
            height: 0,
            rgba: Vec::new(),
        };
        assert_eq!(sample_texture(&empty, 0.5, 0.5, fallback), fallback);
    }

    #[test]
    fn camera_controls_follow_the_flutter_widgets() {
        let mut camera = Camera::default();
        assert_eq!(camera, Camera::INSPECTOR);
        assert_eq!(Camera::PREVIEW.distance, 2.6);
        assert_eq!(Camera::PREVIEW.yaw, std::f32::consts::PI);
        camera.orbit(100.0, 1000.0);
        assert!((camera.yaw - (std::f32::consts::PI + 1.0)).abs() < 1e-6);
        assert_eq!(camera.pitch, MAX_PITCH);
        camera.orbit(0.0, -10_000.0);
        assert_eq!(camera.pitch, -MAX_PITCH);
        camera.zoom(100_000.0);
        assert_eq!(camera.distance, MAX_DISTANCE);
        camera.zoom(-100_000.0);
        assert_eq!(camera.distance, MIN_DISTANCE);
        camera.pan(3.0, -2.0);
        camera.pan(1.0, 1.0);
        assert_eq!((camera.pan_x, camera.pan_y), (4.0, -1.0));
    }

    #[test]
    fn bgra_swaps_red_and_blue() {
        let frame = RenderFrame {
            width: 1,
            height: 1,
            rgba: vec![1, 2, 3, 4],
        };
        assert_eq!(frame.pixel(0, 0), Some([1, 2, 3, 4]));
        assert_eq!(frame.pixel(1, 0), None);
        assert_eq!(frame.into_bgra(), vec![3, 2, 1, 4]);
    }
}
