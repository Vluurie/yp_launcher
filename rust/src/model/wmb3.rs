use image::ImageFormat;
use std::collections::{HashMap, HashSet};

const HEADER_SIZE: usize = 136;
const VERTEX_GROUP_SIZE: usize = 48;
const BATCH_SIZE: usize = 28;
const LOD_SIZE: usize = 20;
const BATCH_INFO_SIZE: usize = 24;
const MATERIAL_SIZE: usize = 48;
const MESH_SIZE: usize = 44;
const MAX_VERTEX_GROUPS: usize = 4_096;
const MAX_BATCHES: usize = 200_000;
const MAX_LODS: usize = 64;
const MAX_MATERIALS: usize = 65_536;
const MAX_MESHES: usize = 200_000;
const MAX_VERTICES: usize = 8_000_000;
const MAX_INDICES: usize = 24_000_000;
const MAX_STRING_LENGTH: usize = 4_096;

pub struct TextureFiles {
    pub index: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone)]
pub struct Mesh {
    pub id: u32,
    pub name: String,
    pub lod_name: String,
    pub lod_index: u32,
    pub enabled: bool,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub albedo_texture_id: Option<u32>,
    pub normal_texture_id: Option<u32>,
}

pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub textures: HashMap<u32, Texture>,
    pub center: [f32; 3],
    pub radius: f32,
    pub selected_lod: u32,
    pub lod_names: Vec<String>,
}

struct VertexGroup {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

struct Batch {
    index_start: usize,
    index_count: usize,
}

struct BatchInfo {
    vertex_group_index: usize,
    mesh_index: usize,
    material_index: usize,
}

struct Lod {
    name: String,
    batch_start: usize,
    batch_infos: Vec<BatchInfo>,
}

struct Material {
    albedo_texture_id: Option<u32>,
    normal_texture_id: Option<u32>,
}

impl Scene {
    pub fn parse(
        bytes: &[u8],
        texture_files: Option<TextureFiles>,
        is_player: bool,
    ) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE || bytes.get(0..4) != Some(b"WMB3") {
            return Err("Entry is not a NieR WMB3 model".to_string());
        }

        let flags = read_u16(bytes, 12)? as u32;
        let vertex_groups = parse_vertex_groups(bytes, flags)?;
        let batches = parse_batches(bytes)?;
        let lods = parse_lods(bytes)?;
        let materials = parse_materials(bytes)?;
        let mesh_names = parse_mesh_names(bytes)?;
        let mut meshes = Vec::new();
        let mut texture_ids = HashSet::new();
        let mut vertex_count = 0_usize;
        let mut index_count = 0_usize;

        for (lod_index, lod) in lods.iter().enumerate() {
            for (batch_index, batch_info) in lod.batch_infos.iter().enumerate() {
                let batch = batches
                    .get(lod.batch_start.saturating_add(batch_index))
                    .ok_or_else(|| "WMB LOD batch index is invalid".to_string())?;
                let group = vertex_groups
                    .get(batch_info.vertex_group_index)
                    .ok_or_else(|| "WMB vertex group index is invalid".to_string())?;
                let mesh_name = mesh_names
                    .get(batch_info.mesh_index)
                    .cloned()
                    .unwrap_or_else(|| format!("Mesh {}", batch_info.mesh_index));
                let material = materials.get(batch_info.material_index);
                let end = batch
                    .index_start
                    .checked_add(batch.index_count)
                    .ok_or_else(|| "WMB batch index range overflowed".to_string())?;
                let source_indices = group
                    .indices
                    .get(batch.index_start..end)
                    .ok_or_else(|| "WMB batch points outside its index buffer".to_string())?;
                if source_indices.len() < 3 {
                    continue;
                }
                let minimum = source_indices
                    .iter()
                    .copied()
                    .min()
                    .ok_or_else(|| "WMB batch has no indices".to_string())?
                    as usize;
                let maximum = source_indices
                    .iter()
                    .copied()
                    .max()
                    .ok_or_else(|| "WMB batch has no indices".to_string())?
                    as usize;
                let vertex_end = maximum
                    .checked_add(1)
                    .ok_or_else(|| "WMB batch vertex range overflowed".to_string())?;
                let positions = group
                    .positions
                    .get(minimum..vertex_end)
                    .ok_or_else(|| "WMB batch points outside its vertex buffer".to_string())?
                    .to_vec();
                let uvs = group
                    .uvs
                    .get(minimum..vertex_end)
                    .ok_or_else(|| "WMB batch points outside its UV buffer".to_string())?
                    .to_vec();
                let normals = group
                    .normals
                    .get(minimum..vertex_end)
                    .ok_or_else(|| "WMB batch points outside its normal buffer".to_string())?
                    .to_vec();
                let tangents = group
                    .tangents
                    .get(minimum..vertex_end)
                    .ok_or_else(|| "WMB batch points outside its tangent buffer".to_string())?
                    .to_vec();
                let mut indices = Vec::with_capacity(source_indices.len());
                for triangle in source_indices.chunks_exact(3) {
                    indices.push(triangle[2] - minimum as u32);
                    indices.push(triangle[1] - minimum as u32);
                    indices.push(triangle[0] - minimum as u32);
                }
                vertex_count = vertex_count.saturating_add(positions.len());
                index_count = index_count.saturating_add(indices.len());
                if vertex_count > MAX_VERTICES || index_count > MAX_INDICES {
                    return Err("WMB scene is too large".to_string());
                }
                let lowercase_name = mesh_name.to_ascii_lowercase();
                let enabled = !is_player
                    || !["armor", "serious", "broken", "dlc"]
                        .iter()
                        .any(|part| lowercase_name.contains(part));
                let albedo_texture_id = material.and_then(|value| value.albedo_texture_id);
                let normal_texture_id = material.and_then(|value| value.normal_texture_id);
                if let Some(texture_id) = albedo_texture_id {
                    texture_ids.insert(texture_id);
                }
                if let Some(texture_id) = normal_texture_id {
                    texture_ids.insert(texture_id);
                }
                meshes.push(Mesh {
                    id: meshes.len() as u32,
                    name: mesh_name,
                    lod_name: lod.name.clone(),
                    lod_index: lod_index as u32,
                    enabled,
                    positions,
                    normals,
                    tangents,
                    uvs,
                    indices,
                    albedo_texture_id,
                    normal_texture_id,
                });
            }
        }

        if meshes.is_empty() {
            return Err("WMB contains no renderable batches".to_string());
        }

        let textures = match texture_files {
            Some(files) => decode_textures(&files, &texture_ids),
            None => HashMap::new(),
        };
        let (center, radius) = bounds(&meshes);
        Ok(Self {
            meshes,
            textures,
            center,
            radius,
            selected_lod: 0,
            lod_names: lods.into_iter().map(|lod| lod.name).collect(),
        })
    }

    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.positions.len()).sum()
    }

    pub fn triangle_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.indices.len() / 3).sum()
    }

    pub fn set_mesh_visibility(&mut self, mesh_id: u32, visible: bool) -> Result<(), String> {
        let mesh = self
            .meshes
            .iter_mut()
            .find(|mesh| mesh.id == mesh_id)
            .ok_or_else(|| "Model mesh no longer exists".to_string())?;
        mesh.enabled = visible;
        Ok(())
    }

    pub fn set_lod(&mut self, lod_index: u32) -> Result<(), String> {
        if lod_index as usize >= self.lod_names.len() {
            return Err("Model LOD index is invalid".to_string());
        }
        self.selected_lod = lod_index;
        Ok(())
    }
}

fn parse_vertex_groups(bytes: &[u8], header_flags: u32) -> Result<Vec<VertexGroup>, String> {
    let table_offset = read_u32(bytes, 56)? as usize;
    let count = read_u32(bytes, 60)? as usize;
    validate_table(
        bytes,
        table_offset,
        count,
        VERTEX_GROUP_SIZE,
        MAX_VERTEX_GROUPS,
    )?;
    let mut groups = Vec::with_capacity(count);
    for index in 0..count {
        let offset = table_offset + index * VERTEX_GROUP_SIZE;
        let vertex_offset = read_u32(bytes, offset)? as usize;
        let vertex_ex_offset = read_u32(bytes, offset + 4)? as usize;
        let vertex_count = read_u32(bytes, offset + 32)? as usize;
        let vertex_flags = read_u32(bytes, offset + 36)?;
        let index_offset = read_u32(bytes, offset + 40)? as usize;
        let index_count = read_u32(bytes, offset + 44)? as usize;
        if vertex_count > MAX_VERTICES || index_count > MAX_INDICES {
            return Err("WMB vertex group is too large".to_string());
        }
        let stride = vertex_stride(vertex_flags)?;
        checked_slice(
            bytes,
            vertex_offset,
            vertex_count
                .checked_mul(stride)
                .ok_or_else(|| "WMB vertex data size overflowed".to_string())?,
        )?;
        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut tangents = Vec::with_capacity(vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count);
        if vertex_flags != 0 {
            checked_slice(
                bytes,
                vertex_ex_offset,
                vertex_count
                    .checked_mul(vertex_ex_stride(vertex_flags)?)
                    .ok_or_else(|| "WMB extended vertex data size overflowed".to_string())?,
            )?;
        }
        for vertex_index in 0..vertex_count {
            let vertex = vertex_offset + vertex_index * stride;
            let position = [
                read_f32(bytes, vertex)?,
                read_f32(bytes, vertex + 4)?,
                read_f32(bytes, vertex + 8)?,
            ];
            positions.push(if position.iter().all(|value| value.is_finite()) {
                position
            } else {
                [0.0, 0.0, 0.0]
            });
            uvs.push([
                half_to_f32(read_u16(bytes, vertex + 16)?),
                half_to_f32(read_u16(bytes, vertex + 18)?),
            ]);
            tangents.push([
                normalized_byte(read_u8(bytes, vertex + 12)?),
                normalized_byte(read_u8(bytes, vertex + 13)?),
                normalized_byte(read_u8(bytes, vertex + 14)?),
                -normalized_byte(read_u8(bytes, vertex + 15)?),
            ]);
            let normal_offset = if vertex_flags == 0 {
                vertex + 20
            } else {
                vertex_ex_offset
                    + vertex_index * vertex_ex_stride(vertex_flags)?
                    + vertex_ex_normal_offset(vertex_flags)?
            };
            let normal = [
                half_to_f32(read_u16(bytes, normal_offset)?),
                half_to_f32(read_u16(bytes, normal_offset + 2)?),
                half_to_f32(read_u16(bytes, normal_offset + 4)?),
            ];
            normals.push(if normal.iter().all(|value| value.is_finite()) {
                normal
            } else {
                [0.0, 1.0, 0.0]
            });
        }
        let index_width = if header_flags & 0x8 != 0 { 4 } else { 2 };
        checked_slice(
            bytes,
            index_offset,
            index_count
                .checked_mul(index_width)
                .ok_or_else(|| "WMB index data size overflowed".to_string())?,
        )?;
        let mut indices = Vec::with_capacity(index_count);
        for item in 0..index_count {
            let source = index_offset + item * index_width;
            indices.push(if index_width == 4 {
                read_u32(bytes, source)?
            } else {
                read_u16(bytes, source)? as u32
            });
        }
        groups.push(VertexGroup {
            positions,
            normals,
            tangents,
            uvs,
            indices,
        });
    }
    Ok(groups)
}

fn parse_batches(bytes: &[u8]) -> Result<Vec<Batch>, String> {
    let offset = read_u32(bytes, 64)? as usize;
    let count = read_u32(bytes, 68)? as usize;
    validate_table(bytes, offset, count, BATCH_SIZE, MAX_BATCHES)?;
    let mut batches = Vec::with_capacity(count);
    for index in 0..count {
        let item = offset + index * BATCH_SIZE;
        batches.push(Batch {
            index_start: read_u32(bytes, item + 12)? as usize,
            index_count: read_u32(bytes, item + 20)? as usize,
        });
    }
    Ok(batches)
}

fn parse_lods(bytes: &[u8]) -> Result<Vec<Lod>, String> {
    let offset = read_u32(bytes, 72)? as usize;
    let count = read_u32(bytes, 76)? as usize;
    validate_table(bytes, offset, count, LOD_SIZE, MAX_LODS)?;
    let mut lods = Vec::with_capacity(count);
    for index in 0..count {
        let item = offset + index * LOD_SIZE;
        let name = read_string(bytes, read_u32(bytes, item)? as usize)?;
        let batch_start = read_u32(bytes, item + 8)? as usize;
        let info_offset = read_u32(bytes, item + 12)? as usize;
        let info_count = read_u32(bytes, item + 16)? as usize;
        validate_table(bytes, info_offset, info_count, BATCH_INFO_SIZE, MAX_BATCHES)?;
        let mut batch_infos = Vec::with_capacity(info_count);
        for info_index in 0..info_count {
            let info = info_offset + info_index * BATCH_INFO_SIZE;
            batch_infos.push(BatchInfo {
                vertex_group_index: read_u32(bytes, info)? as usize,
                mesh_index: read_u32(bytes, info + 4)? as usize,
                material_index: read_u32(bytes, info + 8)? as usize,
            });
        }
        lods.push(Lod {
            name: if name.is_empty() {
                format!("LOD {index}")
            } else {
                name
            },
            batch_start,
            batch_infos,
        });
    }
    Ok(lods)
}

fn parse_materials(bytes: &[u8]) -> Result<Vec<Material>, String> {
    let offset = read_u32(bytes, 104)? as usize;
    let count = read_u32(bytes, 108)? as usize;
    validate_table(bytes, offset, count, MATERIAL_SIZE, MAX_MATERIALS)?;
    let mut materials = Vec::with_capacity(count);
    for index in 0..count {
        let item = offset + index * MATERIAL_SIZE;
        let texture_offset = read_u32(bytes, item + 24)? as usize;
        let texture_count = read_u32(bytes, item + 28)? as usize;
        validate_table(bytes, texture_offset, texture_count, 8, MAX_MATERIALS)?;
        let mut albedo_texture_id = None;
        let mut normal_texture_id = None;
        for texture_index in 0..texture_count {
            let texture = texture_offset + texture_index * 8;
            let name = read_string(bytes, read_u32(bytes, texture)? as usize)?;
            if name.contains("g_AlbedoMap") {
                albedo_texture_id = Some(read_u32(bytes, texture + 4)?);
            } else if name.contains("g_NormalMap") {
                normal_texture_id = Some(read_u32(bytes, texture + 4)?);
            }
        }
        materials.push(Material {
            albedo_texture_id,
            normal_texture_id,
        });
    }
    Ok(materials)
}

fn parse_mesh_names(bytes: &[u8]) -> Result<Vec<String>, String> {
    let offset = read_u32(bytes, 112)? as usize;
    let count = read_u32(bytes, 116)? as usize;
    validate_table(bytes, offset, count, MESH_SIZE, MAX_MESHES)?;
    let mut names = Vec::with_capacity(count);
    for index in 0..count {
        let item = offset + index * MESH_SIZE;
        names.push(read_string(bytes, read_u32(bytes, item)? as usize)?);
    }
    Ok(names)
}

fn decode_textures(files: &TextureFiles, requested: &HashSet<u32>) -> HashMap<u32, Texture> {
    let mut textures = HashMap::new();
    if files.index.len() < 28 {
        return textures;
    }
    let Ok(count) = read_u32(&files.index, 8).map(|value| value as usize) else {
        return textures;
    };
    let Ok(offsets_offset) = read_u32(&files.index, 12).map(|value| value as usize) else {
        return textures;
    };
    let Ok(sizes_offset) = read_u32(&files.index, 16).map(|value| value as usize) else {
        return textures;
    };
    let Ok(ids_offset) = read_u32(&files.index, 24).map(|value| value as usize) else {
        return textures;
    };
    if validate_table(&files.index, offsets_offset, count, 4, MAX_MATERIALS).is_err()
        || validate_table(&files.index, sizes_offset, count, 4, MAX_MATERIALS).is_err()
        || validate_table(&files.index, ids_offset, count, 4, MAX_MATERIALS).is_err()
    {
        return textures;
    }
    for index in 0..count {
        let Ok(id) = read_u32(&files.index, ids_offset + index * 4) else {
            continue;
        };
        if !requested.contains(&id) {
            continue;
        }
        let Ok(offset) =
            read_u32(&files.index, offsets_offset + index * 4).map(|value| value as usize)
        else {
            continue;
        };
        let Ok(size) = read_u32(&files.index, sizes_offset + index * 4).map(|value| value as usize)
        else {
            continue;
        };
        let Ok(dds) = checked_slice(&files.payload, offset, size) else {
            continue;
        };
        let Ok(image) = image::load_from_memory_with_format(dds, ImageFormat::Dds) else {
            continue;
        };
        let mut rgba = image.to_rgba8().into_raw();
        restore_bc1_alpha(dds, &mut rgba, image.width(), image.height());
        dilate_transparent_rgb(&mut rgba, image.width(), image.height());
        textures.insert(
            id,
            Texture {
                width: image.width(),
                height: image.height(),
                rgba,
            },
        );
    }
    textures
}

fn restore_bc1_alpha(dds: &[u8], rgba: &mut [u8], width: u32, height: u32) {
    let data_offset: usize = if dds.get(84..88) == Some(b"DXT1") {
        128
    } else if dds.get(84..88) == Some(b"DX10")
        && matches!(read_u32(dds, 128), Ok(70..=72))
    {
        148
    } else {
        return;
    };
    let width = width as usize;
    let height = height as usize;
    if width == 0 || height == 0 || rgba.len() != width.saturating_mul(height).saturating_mul(4) {
        return;
    }
    let blocks_wide = width.div_ceil(4);
    let blocks_high = height.div_ceil(4);
    for block_y in 0..blocks_high {
        for block_x in 0..blocks_wide {
            let block_index = block_y * blocks_wide + block_x;
            let Some(block_start) = block_index
                .checked_mul(8)
                .and_then(|offset| data_offset.checked_add(offset))
            else {
                return;
            };
            let Some(block) = dds.get(block_start..block_start + 8) else {
                return;
            };
            let color_0 = u16::from_le_bytes([block[0], block[1]]);
            let color_1 = u16::from_le_bytes([block[2], block[3]]);
            if color_0 > color_1 {
                continue;
            }
            let selectors = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
            for pixel_y in 0..4 {
                for pixel_x in 0..4 {
                    let x = block_x * 4 + pixel_x;
                    let y = block_y * 4 + pixel_y;
                    if x >= width || y >= height {
                        continue;
                    }
                    let selector_index = (pixel_y * 4 + pixel_x) * 2;
                    if (selectors >> selector_index) & 3 == 3 {
                        rgba[(y * width + x) * 4 + 3] = 0;
                    }
                }
            }
        }
    }
}

fn dilate_transparent_rgb(rgba: &mut [u8], width: u32, height: u32) {
    let width = width as usize;
    let height = height as usize;
    if width == 0 || height == 0 || rgba.len() != width.saturating_mul(height).saturating_mul(4) {
        return;
    }
    let mut resolved = rgba
        .chunks_exact(4)
        .map(|pixel| pixel[3] >= 128)
        .collect::<Vec<_>>();
    for _ in 0..8 {
        let previous = resolved.clone();
        let source = rgba.to_vec();
        let mut changed = false;
        for y in 0..height {
            for x in 0..width {
                let pixel_index = y * width + x;
                if previous[pixel_index] {
                    continue;
                }
                let mut color = [0_u32; 3];
                let mut count = 0_u32;
                for offset_y in -1_i32..=1 {
                    for offset_x in -1_i32..=1 {
                        if offset_x == 0 && offset_y == 0 {
                            continue;
                        }
                        let neighbor_x = x as i32 + offset_x;
                        let neighbor_y = y as i32 + offset_y;
                        if neighbor_x < 0
                            || neighbor_y < 0
                            || neighbor_x >= width as i32
                            || neighbor_y >= height as i32
                        {
                            continue;
                        }
                        let neighbor_index =
                            neighbor_y as usize * width + neighbor_x as usize;
                        if !previous[neighbor_index] {
                            continue;
                        }
                        let source_index = neighbor_index * 4;
                        color[0] += source[source_index] as u32;
                        color[1] += source[source_index + 1] as u32;
                        color[2] += source[source_index + 2] as u32;
                        count += 1;
                    }
                }
                if count == 0 {
                    continue;
                }
                let target = pixel_index * 4;
                rgba[target] = (color[0] / count) as u8;
                rgba[target + 1] = (color[1] / count) as u8;
                rgba[target + 2] = (color[2] / count) as u8;
                resolved[pixel_index] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

fn bounds(meshes: &[Mesh]) -> ([f32; 3], f32) {
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for position in meshes.iter().flat_map(|mesh| mesh.positions.iter()) {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(position[axis]);
            maximum[axis] = maximum[axis].max(position[axis]);
        }
    }
    let center = [
        (minimum[0] + maximum[0]) * 0.5,
        (minimum[1] + maximum[1]) * 0.5,
        (minimum[2] + maximum[2]) * 0.5,
    ];
    let mut radius = 0.0_f32;
    for position in meshes.iter().flat_map(|mesh| mesh.positions.iter()) {
        let dx = position[0] - center[0];
        let dy = position[1] - center[1];
        let dz = position[2] - center[2];
        radius = radius.max((dx * dx + dy * dy + dz * dz).sqrt());
    }
    if !radius.is_finite() || radius <= f32::EPSILON {
        radius = 1.0;
    }
    (center, radius)
}

fn validate_table(
    bytes: &[u8],
    offset: usize,
    count: usize,
    stride: usize,
    maximum: usize,
) -> Result<(), String> {
    if count > maximum {
        return Err("WMB table contains too many entries".to_string());
    }
    checked_slice(
        bytes,
        offset,
        count
            .checked_mul(stride)
            .ok_or_else(|| "WMB table size overflowed".to_string())?,
    )?;
    Ok(())
}

fn checked_slice(bytes: &[u8], offset: usize, size: usize) -> Result<&[u8], String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| "WMB data range overflowed".to_string())?;
    bytes
        .get(offset..end)
        .ok_or_else(|| "WMB data points outside the entry".to_string())
}

fn read_string(bytes: &[u8], offset: usize) -> Result<String, String> {
    let remaining = bytes
        .get(offset..)
        .ok_or_else(|| "WMB string offset is invalid".to_string())?;
    let end = remaining
        .iter()
        .take(MAX_STRING_LENGTH)
        .position(|byte| *byte == 0)
        .ok_or_else(|| "WMB string is not terminated".to_string())?;
    Ok(String::from_utf8_lossy(&remaining[..end]).into_owned())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let value = checked_slice(bytes, offset, 2)?;
    Ok(u16::from_le_bytes(
        value
            .try_into()
            .map_err(|_| "Invalid WMB integer".to_string())?,
    ))
}

fn read_u8(bytes: &[u8], offset: usize) -> Result<u8, String> {
    checked_slice(bytes, offset, 1).map(|value| value[0])
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let value = checked_slice(bytes, offset, 4)?;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| "Invalid WMB integer".to_string())?,
    ))
}

fn read_f32(bytes: &[u8], offset: usize) -> Result<f32, String> {
    let value = checked_slice(bytes, offset, 4)?;
    Ok(f32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| "Invalid WMB float".to_string())?,
    ))
}

fn vertex_stride(flags: u32) -> Result<usize, String> {
    match flags {
        0 => Ok(28),
        1 => Ok(24),
        4 | 5 | 7 | 10 | 11 | 12 | 14 => Ok(28),
        _ => Err(format!("WMB vertex flags are unsupported: {flags}")),
    }
}

fn vertex_ex_stride(flags: u32) -> Result<usize, String> {
    match flags {
        1 | 4 => Ok(8),
        5 | 7 => Ok(12),
        10 | 14 => Ok(16),
        11 | 12 => Ok(20),
        _ => Err(format!("WMB extended vertex flags are unsupported: {flags}")),
    }
}

fn vertex_ex_normal_offset(flags: u32) -> Result<usize, String> {
    match flags {
        1 | 4 | 5 | 12 | 14 => Ok(0),
        7 => Ok(4),
        10 | 11 => Ok(8),
        _ => Err(format!("WMB extended vertex flags are unsupported: {flags}")),
    }
}

fn normalized_byte(value: u8) -> f32 {
    (value as f32 - 127.0) / 127.0
}

fn half_to_f32(value: u16) -> f32 {
    let sign = ((value & 0x8000) as u32) << 16;
    let exponent = (value >> 10) & 0x1f;
    let mantissa = value & 0x03ff;
    let bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut mantissa = mantissa as u32;
            let mut exponent = 113_u32;
            while mantissa & 0x0400 == 0 {
                mantissa <<= 1;
                exponent -= 1;
            }
            sign | (exponent << 23) | ((mantissa & 0x03ff) << 13)
        }
        31 => sign | 0x7f80_0000 | ((mantissa as u32) << 13),
        _ => sign | (((exponent as u32) + 112) << 23) | ((mantissa as u32) << 13),
    };
    f32::from_bits(bits)
}
