use crate::model::{Scene, Texture};

const MAX_PIXELS: usize = 2_073_600;

pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy)]
struct Projected {
    x: f32,
    y: f32,
    inverse_depth: f32,
    normal: [f32; 3],
    uv: [f32; 2],
    visible: bool,
}

pub fn render(
    scene: &Scene,
    width: u32,
    height: u32,
    yaw: f32,
    pitch: f32,
    distance: f32,
    pan_x: f32,
    pan_y: f32,
) -> Result<Frame, String> {
    if width < 16 || height < 16 {
        return Err("Render size is too small".to_string());
    }
    let pixel_count = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| "Render size overflowed".to_string())?;
    if pixel_count > MAX_PIXELS {
        return Err("Render size is too large".to_string());
    }
    let mut rgba = vec![0_u8; pixel_count * 4];
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[25, 27, 26, 255]);
    }
    let mut depth = vec![f32::INFINITY; pixel_count];
    let sin_yaw = yaw.sin();
    let cos_yaw = yaw.cos();
    let sin_pitch = pitch.sin();
    let cos_pitch = pitch.cos();
    let distance = distance.clamp(0.9, 12.0);
    let focal = width.min(height) as f32 * 0.92;
    let center_x = width as f32 * 0.5 + pan_x;
    let center_y = height as f32 * 0.5 + pan_y;

    for mesh in scene
        .meshes
        .iter()
        .filter(|mesh| mesh.enabled && mesh.lod_index == scene.selected_lod)
    {
        let projected = mesh
            .positions
            .iter()
            .zip(&mesh.uvs)
            .zip(&mesh.normals)
            .map(|((position, uv), normal)| {
                let x = (position[0] - scene.center[0]) / scene.radius;
                let y = (position[1] - scene.center[1]) / scene.radius;
                let z = (position[2] - scene.center[2]) / scene.radius;
                let rotated_x = cos_yaw * x + sin_yaw * z;
                let yaw_z = -sin_yaw * x + cos_yaw * z;
                let rotated_y = cos_pitch * y - sin_pitch * yaw_z;
                let rotated_z = sin_pitch * y + cos_pitch * yaw_z;
                let normal_x = cos_yaw * normal[0] + sin_yaw * normal[2];
                let normal_yaw_z = -sin_yaw * normal[0] + cos_yaw * normal[2];
                let normal_y = cos_pitch * normal[1] - sin_pitch * normal_yaw_z;
                let normal_z = sin_pitch * normal[1] + cos_pitch * normal_yaw_z;
                let camera_z = rotated_z + distance;
                let visible = camera_z > 0.02;
                Projected {
                    x: center_x + rotated_x * focal / camera_z,
                    y: center_y - rotated_y * focal / camera_z,
                    inverse_depth: camera_z.recip(),
                    normal: [normal_x, normal_y, normal_z],
                    uv: *uv,
                    visible,
                }
            })
            .collect::<Vec<_>>();
        let texture = mesh
            .albedo_texture_id
            .and_then(|id| scene.textures.get(&id));
        for triangle in mesh.indices.chunks_exact(3) {
            let Some(a) = projected.get(triangle[0] as usize).copied() else {
                continue;
            };
            let Some(b) = projected.get(triangle[1] as usize).copied() else {
                continue;
            };
            let Some(c) = projected.get(triangle[2] as usize).copied() else {
                continue;
            };
            if !a.visible || !b.visible || !c.visible {
                continue;
            }
            draw_triangle(&mut rgba, &mut depth, width, height, a, b, c, texture);
        }
    }

    Ok(Frame {
        width,
        height,
        rgba,
    })
}

fn draw_triangle(
    rgba: &mut [u8],
    depth_buffer: &mut [f32],
    width: u32,
    height: u32,
    a: Projected,
    b: Projected,
    c: Projected,
    texture: Option<&Texture>,
) {
    let area = edge(a.x, a.y, b.x, b.y, c.x, c.y);
    if !area.is_finite() || area.abs() < 0.01 {
        return;
    }
    let min_x =
        a.x.min(b.x)
            .min(c.x)
            .floor()
            .clamp(0.0, width.saturating_sub(1) as f32) as u32;
    let max_x =
        a.x.max(b.x)
            .max(c.x)
            .ceil()
            .clamp(0.0, width.saturating_sub(1) as f32) as u32;
    let min_y =
        a.y.min(b.y)
            .min(c.y)
            .floor()
            .clamp(0.0, height.saturating_sub(1) as f32) as u32;
    let max_y =
        a.y.max(b.y)
            .max(c.y)
            .ceil()
            .clamp(0.0, height.saturating_sub(1) as f32) as u32;
    if min_x > max_x || min_y > max_y {
        return;
    }
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let sample_x = x as f32 + 0.5;
            let sample_y = y as f32 + 0.5;
            let weight_a = edge(b.x, b.y, c.x, c.y, sample_x, sample_y) / area;
            let weight_b = edge(c.x, c.y, a.x, a.y, sample_x, sample_y) / area;
            let weight_c = 1.0 - weight_a - weight_b;
            if weight_a < 0.0 || weight_b < 0.0 || weight_c < 0.0 {
                continue;
            }
            let inverse_depth = weight_a * a.inverse_depth
                + weight_b * b.inverse_depth
                + weight_c * c.inverse_depth;
            if inverse_depth <= 0.0 {
                continue;
            }
            let pixel_depth = inverse_depth.recip();
            let pixel_index = y as usize * width as usize + x as usize;
            if pixel_depth >= depth_buffer[pixel_index] {
                continue;
            }
            let sampled = texture
                .map(|texture| {
                    let u = (weight_a * a.uv[0] * a.inverse_depth
                        + weight_b * b.uv[0] * b.inverse_depth
                        + weight_c * c.uv[0] * c.inverse_depth)
                        / inverse_depth;
                    let v = (weight_a * a.uv[1] * a.inverse_depth
                        + weight_b * b.uv[1] * b.inverse_depth
                        + weight_c * c.uv[1] * c.inverse_depth)
                        / inverse_depth;
                    sample_texture(texture, u, v)
                })
                .unwrap_or([196, 190, 154, 255]);
            if sampled[3] < 128 {
                continue;
            }
            let normal = [
                (weight_a * a.normal[0] * a.inverse_depth
                    + weight_b * b.normal[0] * b.inverse_depth
                    + weight_c * c.normal[0] * c.inverse_depth)
                    / inverse_depth,
                (weight_a * a.normal[1] * a.inverse_depth
                    + weight_b * b.normal[1] * b.inverse_depth
                    + weight_c * c.normal[1] * c.inverse_depth)
                    / inverse_depth,
                (weight_a * a.normal[2] * a.inverse_depth
                    + weight_b * b.normal[2] * b.inverse_depth
                    + weight_c * c.normal[2] * c.inverse_depth)
                    / inverse_depth,
            ];
            let normal_length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            let light = if normal_length > f32::EPSILON {
                ((normal[0] * -0.35 + normal[1] * 0.75 + normal[2] * 0.55)
                    / normal_length)
                    .abs()
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };
            let intensity = 0.35 + light * 0.65;
            depth_buffer[pixel_index] = pixel_depth;
            let color_index = pixel_index * 4;
            rgba[color_index] = (sampled[0] as f32 * intensity).min(255.0) as u8;
            rgba[color_index + 1] = (sampled[1] as f32 * intensity).min(255.0) as u8;
            rgba[color_index + 2] = (sampled[2] as f32 * intensity).min(255.0) as u8;
            rgba[color_index + 3] = 255;
        }
    }
}

fn sample_texture(texture: &Texture, u: f32, v: f32) -> [u8; 4] {
    if texture.width == 0 || texture.height == 0 {
        return [196, 190, 154, 255];
    }
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let x = (u * texture.width as f32) as u32 % texture.width;
    let y = (v * texture.height as f32) as u32 % texture.height;
    let index = (y as usize * texture.width as usize + x as usize) * 4;
    texture
        .rgba
        .get(index..index + 4)
        .and_then(|pixel| pixel.try_into().ok())
        .unwrap_or([196, 190, 154, 255])
}

fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}