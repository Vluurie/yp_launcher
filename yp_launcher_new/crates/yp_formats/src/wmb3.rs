mod rig;

pub use rig::{ClothPlayer, Pose, Skeleton, Weights};

use std::collections::{BTreeSet, HashMap};

use image::ImageFormat;

use crate::bytes::{le_f32, le_u16, le_u32, slice};
use crate::error::{ModelFault, Result};

pub const MAGIC: [u8; 4] = *b"WMB3";
pub const GAME_LAYOUT_VERSION: u32 = 0x2016_0116;
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
const ALBEDO_SLOT: &str = "g_AlbedoMap";
const NORMAL_SLOT: &str = "g_NormalMap";
pub const MAX_TEXTURE_SIDE: u32 = 1024;
const MAX_DECODE_THREADS: usize = 8;
const DDS_HEADER_SIZE: usize = 128;
const DDS_DX10_HEADER_SIZE: usize = 148;
const DDS_CUBEMAP_FLAG: u32 = 0x200;

type Parsed<T> = std::result::Result<T, ModelFault>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureFiles {
    pub index: Vec<u8>,
    pub payload: Vec<u8>,
}

impl TextureFiles {
    pub fn combined(wtb: Vec<u8>) -> Self {
        Self {
            index: wtb.clone(),
            payload: wtb,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MaterialBlend {
    #[default]
    Opaque,
    Multiply,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub blend: MaterialBlend,
    pub id: u32,
    pub name: String,
    pub lod_name: String,
    pub lod_index: u32,
    pub weights: Vec<Weights>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub albedo_texture_id: Option<u32>,
    pub normal_texture_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub skeleton: Skeleton,
    pub meshes: Vec<Mesh>,
    pub textures: HashMap<u32, Texture>,
    pub center: [f32; 3],
    pub radius: f32,
    pub lod_names: Vec<String>,
}

struct VertexGroup {
    weights: Vec<[u8; 8]>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

struct Batch {
    palette: usize,
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
    blend: MaterialBlend,
    albedo_texture_id: Option<u32>,
    normal_texture_id: Option<u32>,
}

pub fn is_wmb3(bytes: &[u8]) -> bool {
    bytes.get(0..4) == Some(&MAGIC[..])
}

pub fn layout_version(bytes: &[u8]) -> Option<u32> {
    is_wmb3(bytes).then(|| le_u32(bytes, 4)).flatten()
}

pub fn is_game_layout(bytes: &[u8]) -> bool {
    layout_version(bytes) == Some(GAME_LAYOUT_VERSION)
}

impl Model {
    pub fn parse(bytes: &[u8], texture_files: Option<&TextureFiles>) -> Result<Self> {
        Ok(parse_model(bytes, texture_files)?)
    }

    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.positions.len()).sum()
    }

    pub fn triangle_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.indices.len() / 3).sum()
    }

    pub fn texture_ids(&self) -> BTreeSet<u32> {
        referenced_texture_ids(&self.meshes)
    }
}

fn parse_model(bytes: &[u8], texture_files: Option<&TextureFiles>) -> Parsed<Model> {
    if bytes.len() < HEADER_SIZE || !is_wmb3(bytes) {
        return Err(ModelFault::NotWmb3);
    }
    let version = read_u32(bytes, 4)?;
    if version != GAME_LAYOUT_VERSION {
        return Err(ModelFault::OlderLayout(version));
    }

    let flags = u32::from(read_u16(bytes, 12)?);
    let vertex_groups = parse_vertex_groups(bytes, flags)?;
    let skeleton = Skeleton::read(bytes).unwrap_or_default();
    let palettes = rig::palettes(bytes).unwrap_or_default();
    let batches = parse_batches(bytes)?;
    let lods = parse_lods(bytes)?;
    let materials = parse_materials(bytes)?;
    let mesh_names = parse_mesh_names(bytes)?;
    let mut meshes: Vec<Mesh> = Vec::new();
    let mut vertex_count = 0_usize;
    let mut index_count = 0_usize;

    for (lod_index, lod) in lods.iter().enumerate() {
        for (batch_index, batch_info) in lod.batch_infos.iter().enumerate() {
            let batch = batches
                .get(lod.batch_start.saturating_add(batch_index))
                .ok_or(ModelFault::InvalidBatchIndex)?;
            let group = vertex_groups
                .get(batch_info.vertex_group_index)
                .ok_or(ModelFault::InvalidVertexGroupIndex)?;
            let mesh_name = mesh_names
                .get(batch_info.mesh_index)
                .cloned()
                .unwrap_or_else(|| format!("Mesh {}", batch_info.mesh_index));
            let material = materials.get(batch_info.material_index);
            let end = batch
                .index_start
                .checked_add(batch.index_count)
                .ok_or(ModelFault::RangeOverflow)?;
            let source_indices = group
                .indices
                .get(batch.index_start..end)
                .ok_or(ModelFault::OutsideEntry)?;
            if source_indices.len() < 3 {
                continue;
            }
            let minimum = source_indices
                .iter()
                .copied()
                .min()
                .ok_or(ModelFault::OutsideEntry)? as usize;
            let maximum = source_indices
                .iter()
                .copied()
                .max()
                .ok_or(ModelFault::OutsideEntry)? as usize;
            let vertex_end = maximum.checked_add(1).ok_or(ModelFault::RangeOverflow)?;
            let positions = group
                .positions
                .get(minimum..vertex_end)
                .ok_or(ModelFault::OutsideEntry)?
                .to_vec();
            let uvs = group
                .uvs
                .get(minimum..vertex_end)
                .ok_or(ModelFault::OutsideEntry)?
                .to_vec();
            let normals = group
                .normals
                .get(minimum..vertex_end)
                .ok_or(ModelFault::OutsideEntry)?
                .to_vec();
            let tangents = group
                .tangents
                .get(minimum..vertex_end)
                .ok_or(ModelFault::OutsideEntry)?
                .to_vec();
            let weights = palettes.get(batch.palette)
                .zip(group.weights.get(minimum..vertex_end))
                .map(|(palette, vertices)| vertices.iter().map(|v| Weights {
                    bones: std::array::from_fn(|i| palette.get(usize::from(v[i])).copied().unwrap_or(u16::MAX)),
                    weights: [v[4], v[5], v[6], v[7]],
                }).collect())
                .unwrap_or_default();
            let base = minimum as u32;
            let mut indices = Vec::with_capacity(source_indices.len());
            for triangle in source_indices.chunks_exact(3) {
                indices.push(triangle[2] - base);
                indices.push(triangle[1] - base);
                indices.push(triangle[0] - base);
            }
            vertex_count = vertex_count.saturating_add(positions.len());
            index_count = index_count.saturating_add(indices.len());
            if vertex_count > MAX_VERTICES || index_count > MAX_INDICES {
                return Err(ModelFault::TooLarge);
            }
            meshes.push(Mesh {
                blend: material.map_or(MaterialBlend::Opaque, |value| value.blend),
                id: meshes.len() as u32,
                name: mesh_name,
                lod_name: lod.name.clone(),
                lod_index: lod_index as u32,
                weights,
                positions,
                normals,
                tangents,
                uvs,
                indices,
                albedo_texture_id: material.and_then(|value| value.albedo_texture_id),
                normal_texture_id: material.and_then(|value| value.normal_texture_id),
            });
        }
    }

    if meshes.is_empty() {
        return Err(ModelFault::NoRenderableBatches);
    }

    let textures = match texture_files {
        Some(files) => decode_textures(files, &rendered_texture_ids(&meshes)),
        None => HashMap::new(),
    };
    let (center, radius) = bounds(&meshes);
    Ok(Model {
        skeleton,
        meshes,
        textures,
        center,
        radius,
        lod_names: lods.into_iter().map(|lod| lod.name).collect(),
    })
}

fn rendered_texture_ids(meshes: &[Mesh]) -> BTreeSet<u32> {
    meshes
        .iter()
        .filter_map(|mesh| mesh.albedo_texture_id)
        .collect()
}

fn referenced_texture_ids(meshes: &[Mesh]) -> BTreeSet<u32> {
    meshes
        .iter()
        .flat_map(|mesh| [mesh.albedo_texture_id, mesh.normal_texture_id])
        .flatten()
        .collect()
}

fn parse_vertex_groups(bytes: &[u8], header_flags: u32) -> Parsed<Vec<VertexGroup>> {
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
            return Err(ModelFault::TooLarge);
        }
        let stride = vertex_stride(vertex_flags)?;
        checked_slice(
            bytes,
            vertex_offset,
            vertex_count
                .checked_mul(stride)
                .ok_or(ModelFault::RangeOverflow)?,
        )?;
        let extended = if vertex_flags == 0 {
            None
        } else {
            let ex_stride = vertex_ex_stride(vertex_flags)?;
            checked_slice(
                bytes,
                vertex_ex_offset,
                vertex_count
                    .checked_mul(ex_stride)
                    .ok_or(ModelFault::RangeOverflow)?,
            )?;
            Some((ex_stride, vertex_ex_normal_offset(vertex_flags)?))
        };
        let has_weights = matches!(vertex_flags, 7 | 8 | 10 | 11);
        let mut weights = Vec::new();
        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut tangents = Vec::with_capacity(vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count);
        for vertex_index in 0..vertex_count {
            let vertex = vertex_offset + vertex_index * stride;
            if has_weights {
                weights.push(crate::bytes::array(bytes, vertex + 20).ok_or(ModelFault::OutsideEntry)?);
            }
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
            let normal_offset = match extended {
                None => vertex + 20,
                Some((ex_stride, normal_at)) => {
                    vertex_ex_offset + vertex_index * ex_stride + normal_at
                }
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
                .ok_or(ModelFault::RangeOverflow)?,
        )?;
        let mut indices = Vec::with_capacity(index_count);
        for item in 0..index_count {
            let source = index_offset + item * index_width;
            indices.push(if index_width == 4 {
                read_u32(bytes, source)?
            } else {
                u32::from(read_u16(bytes, source)?)
            });
        }
        groups.push(VertexGroup {
            weights,
            positions,
            normals,
            tangents,
            uvs,
            indices,
        });
    }
    Ok(groups)
}

fn parse_batches(bytes: &[u8]) -> Parsed<Vec<Batch>> {
    let offset = read_u32(bytes, 64)? as usize;
    let count = read_u32(bytes, 68)? as usize;
    validate_table(bytes, offset, count, BATCH_SIZE, MAX_BATCHES)?;
    (0..count)
        .map(|index| {
            let item = offset + index * BATCH_SIZE;
            Ok(Batch {
                palette: read_u32(bytes, item + 4)? as usize,
                index_start: read_u32(bytes, item + 12)? as usize,
                index_count: read_u32(bytes, item + 20)? as usize,
            })
        })
        .collect()
}

fn parse_lods(bytes: &[u8]) -> Parsed<Vec<Lod>> {
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

fn parse_materials(bytes: &[u8]) -> Parsed<Vec<Material>> {
    let offset = read_u32(bytes, 104)? as usize;
    let count = read_u32(bytes, 108)? as usize;
    validate_table(bytes, offset, count, MATERIAL_SIZE, MAX_MATERIALS)?;
    let mut materials = Vec::with_capacity(count);
    for index in 0..count {
        let item = offset + index * MATERIAL_SIZE;
        let technique_offset = read_u32(bytes, item + 16)? as usize;
        let blend = if technique_offset != 0
            && read_string(bytes, technique_offset)?.eq_ignore_ascii_case("Multiplication")
        {
            MaterialBlend::Multiply
        } else {
            MaterialBlend::Opaque
        };
        let texture_offset = read_u32(bytes, item + 24)? as usize;
        let texture_count = read_u32(bytes, item + 28)? as usize;
        validate_table(bytes, texture_offset, texture_count, 8, MAX_MATERIALS)?;
        let mut albedo_texture_id = None;
        let mut normal_texture_id = None;
        for texture_index in 0..texture_count {
            let texture = texture_offset + texture_index * 8;
            let slot = read_string(bytes, read_u32(bytes, texture)? as usize)?;
            if slot.starts_with(ALBEDO_SLOT) {
                albedo_texture_id = Some(read_u32(bytes, texture + 4)?);
            } else if slot.starts_with(NORMAL_SLOT) {
                normal_texture_id = Some(read_u32(bytes, texture + 4)?);
            }
        }
        materials.push(Material {
            blend,
            albedo_texture_id,
            normal_texture_id,
        });
    }
    Ok(materials)
}

fn parse_mesh_names(bytes: &[u8]) -> Parsed<Vec<String>> {
    let offset = read_u32(bytes, 112)? as usize;
    let count = read_u32(bytes, 116)? as usize;
    validate_table(bytes, offset, count, MESH_SIZE, MAX_MESHES)?;
    (0..count)
        .map(|index| read_string(bytes, read_u32(bytes, offset + index * MESH_SIZE)? as usize))
        .collect()
}

fn decode_textures(files: &TextureFiles, requested: &BTreeSet<u32>) -> HashMap<u32, Texture> {
    let mut textures = HashMap::new();
    let index = &files.index;
    if index.len() < 28 {
        return textures;
    }
    let (Some(count), Some(offsets_offset), Some(sizes_offset), Some(ids_offset)) = (
        le_u32(index, 8).map(|value| value as usize),
        le_u32(index, 12).map(|value| value as usize),
        le_u32(index, 16).map(|value| value as usize),
        le_u32(index, 24).map(|value| value as usize),
    ) else {
        return textures;
    };
    if validate_table(index, offsets_offset, count, 4, MAX_MATERIALS).is_err()
        || validate_table(index, sizes_offset, count, 4, MAX_MATERIALS).is_err()
        || validate_table(index, ids_offset, count, 4, MAX_MATERIALS).is_err()
    {
        return textures;
    }
    let mut jobs: Vec<(u32, &[u8])> = Vec::new();
    for item in 0..count {
        let Some(id) = le_u32(index, ids_offset + item * 4) else {
            continue;
        };
        if !requested.contains(&id) {
            continue;
        }
        let (Some(offset), Some(size)) = (
            le_u32(index, offsets_offset + item * 4),
            le_u32(index, sizes_offset + item * 4),
        ) else {
            continue;
        };
        let Some(dds) = slice(&files.payload, offset as usize, size as usize) else {
            continue;
        };
        jobs.push((id, dds));
    }
    for (id, texture) in decode_jobs(&jobs) {
        textures.insert(id, texture);
    }
    textures
}

fn decode_batch(batch: &[(u32, &[u8])]) -> Vec<(u32, Texture)> {
    batch
        .iter()
        .filter_map(|(id, dds)| decode_dds_texture(dds).map(|texture| (*id, texture)))
        .collect()
}

pub(crate) fn decode_jobs(jobs: &[(u32, &[u8])]) -> Vec<(u32, Texture)> {
    let workers = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(MAX_DECODE_THREADS)
        .min(jobs.len());
    if workers <= 1 {
        return decode_batch(jobs);
    }
    let batch_size = jobs.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .chunks(batch_size)
            .map(|batch| scope.spawn(move || decode_batch(batch)))
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap_or_default())
            .collect()
    })
}

pub fn decode_dds_texture(dds: &[u8]) -> Option<Texture> {
    let reduced = reduced_mip(dds, MAX_TEXTURE_SIDE);
    reduced
        .as_deref()
        .and_then(decode_dds_level)
        .or_else(|| decode_dds_level(dds))
}

fn decode_dds_level(dds: &[u8]) -> Option<Texture> {
    let image = image::load_from_memory_with_format(dds, ImageFormat::Dds).ok()?;
    let (width, height) = (image.width(), image.height());
    let mut rgba = image.to_rgba8().into_raw();
    restore_bc1_alpha(dds, &mut rgba, width, height);
    let (mut rgba, width, height) = shrunk_to(rgba, width, height, MAX_TEXTURE_SIDE);
    dilate_transparent_rgb(&mut rgba, width, height);
    Some(Texture {
        width,
        height,
        rgba,
    })
}

fn block_count(side: u32) -> usize {
    (side as usize).div_ceil(4).max(1)
}

fn reduced_mip(dds: &[u8], max_side: u32) -> Option<Vec<u8>> {
    if dds.get(0..4)? != b"DDS " {
        return None;
    }
    let height = le_u32(dds, 12)?;
    let width = le_u32(dds, 16)?;
    let levels = le_u32(dds, 28)?;
    if width.max(height) <= max_side || levels <= 1 {
        return None;
    }
    if le_u32(dds, 112)? & DDS_CUBEMAP_FLAG != 0 {
        return None;
    }
    let (header_size, block_bytes) = match dds.get(84..88)? {
        b"DXT1" => (DDS_HEADER_SIZE, 8),
        b"DXT3" | b"DXT5" => (DDS_HEADER_SIZE, 16),
        b"DX10" if le_u32(dds, 140)? == 1 => match le_u32(dds, 128)? {
            70..=72 => (DDS_DX10_HEADER_SIZE, 8),
            73..=78 => (DDS_DX10_HEADER_SIZE, 16),
            _ => return None,
        },
        _ => return None,
    };
    let level_bytes =
        |width: u32, height: u32| block_count(width) * block_count(height) * block_bytes;
    let mut offset = header_size;
    let (mut level_width, mut level_height) = (width, height);
    let mut level = 0;
    while level + 1 < levels && level_width.max(level_height) > max_side {
        offset = offset.checked_add(level_bytes(level_width, level_height))?;
        level_width = (level_width / 2).max(1);
        level_height = (level_height / 2).max(1);
        level += 1;
    }
    let size = level_bytes(level_width, level_height);
    let data = slice(dds, offset, size)?;
    let mut reduced = dds.get(..header_size)?.to_vec();
    reduced[12..16].copy_from_slice(&level_height.to_le_bytes());
    reduced[16..20].copy_from_slice(&level_width.to_le_bytes());
    reduced[20..24].copy_from_slice(&u32::try_from(size).ok()?.to_le_bytes());
    reduced[28..32].copy_from_slice(&1_u32.to_le_bytes());
    reduced.extend_from_slice(data);
    Some(reduced)
}

fn shrunk_to(rgba: Vec<u8>, width: u32, height: u32, max_side: u32) -> (Vec<u8>, u32, u32) {
    let mut factor = 1_u32;
    while (width.max(height) / factor) > max_side {
        factor *= 2;
    }
    let source_width = width as usize;
    let source_height = height as usize;
    if factor == 1 || rgba.len() != source_width * source_height * 4 {
        return (rgba, width, height);
    }
    let step = factor as usize;
    let target_width = (source_width / step).max(1);
    let target_height = (source_height / step).max(1);
    let mut shrunk = Vec::with_capacity(target_width * target_height * 4);
    for target_y in 0..target_height {
        let rows = (target_y * step..(target_y + 1) * step).take_while(|y| *y < source_height);
        let rows: Vec<usize> = rows.collect();
        for target_x in 0..target_width {
            let mut sum = [0_u32; 4];
            let mut samples = 0_u32;
            for source_y in &rows {
                let columns =
                    (target_x * step..(target_x + 1) * step).take_while(|x| *x < source_width);
                for source_x in columns {
                    let at = (source_y * source_width + source_x) * 4;
                    for (total, channel) in sum.iter_mut().zip(&rgba[at..at + 4]) {
                        *total += u32::from(*channel);
                    }
                    samples += 1;
                }
            }
            let samples = samples.max(1);
            shrunk.extend(sum.map(|total| (total / samples) as u8));
        }
    }
    (shrunk, target_width as u32, target_height as u32)
}

fn restore_bc1_alpha(dds: &[u8], rgba: &mut [u8], width: u32, height: u32) {
    let data_offset: usize = match dds.get(84..88) {
        Some(b"DXT1") => 128,
        Some(b"DX10") if matches!(le_u32(dds, 128), Some(70..=72)) => 148,
        _ => return,
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
            let Some(block) = slice(dds, block_start, 8) else {
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
    if resolved.iter().all(|opaque| *opaque) {
        return;
    }
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
                for offset_y in -1_i64..=1 {
                    for offset_x in -1_i64..=1 {
                        if offset_x == 0 && offset_y == 0 {
                            continue;
                        }
                        let neighbor_x = x as i64 + offset_x;
                        let neighbor_y = y as i64 + offset_y;
                        if neighbor_x < 0
                            || neighbor_y < 0
                            || neighbor_x >= width as i64
                            || neighbor_y >= height as i64
                        {
                            continue;
                        }
                        let neighbor_index = neighbor_y as usize * width + neighbor_x as usize;
                        if !previous[neighbor_index] {
                            continue;
                        }
                        let source_index = neighbor_index * 4;
                        color[0] += u32::from(source[source_index]);
                        color[1] += u32::from(source[source_index + 1]);
                        color[2] += u32::from(source[source_index + 2]);
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
) -> Parsed<()> {
    if count > maximum {
        return Err(ModelFault::TooManyEntries);
    }
    checked_slice(
        bytes,
        offset,
        count.checked_mul(stride).ok_or(ModelFault::RangeOverflow)?,
    )?;
    Ok(())
}

fn checked_slice(bytes: &[u8], offset: usize, size: usize) -> Parsed<&[u8]> {
    let end = offset.checked_add(size).ok_or(ModelFault::RangeOverflow)?;
    bytes.get(offset..end).ok_or(ModelFault::OutsideEntry)
}

fn read_string(bytes: &[u8], offset: usize) -> Parsed<String> {
    let remaining = bytes.get(offset..).ok_or(ModelFault::OutsideEntry)?;
    let end = remaining
        .iter()
        .take(MAX_STRING_LENGTH)
        .position(|byte| *byte == 0)
        .ok_or(ModelFault::UnterminatedString)?;
    Ok(String::from_utf8_lossy(&remaining[..end]).into_owned())
}

fn read_u8(bytes: &[u8], offset: usize) -> Parsed<u8> {
    bytes.get(offset).copied().ok_or(ModelFault::OutsideEntry)
}

fn read_u16(bytes: &[u8], offset: usize) -> Parsed<u16> {
    le_u16(bytes, offset).ok_or(ModelFault::OutsideEntry)
}

fn read_u32(bytes: &[u8], offset: usize) -> Parsed<u32> {
    le_u32(bytes, offset).ok_or(ModelFault::OutsideEntry)
}

fn read_f32(bytes: &[u8], offset: usize) -> Parsed<f32> {
    le_f32(bytes, offset).ok_or(ModelFault::OutsideEntry)
}

fn vertex_stride(flags: u32) -> Parsed<usize> {
    match flags {
        1 => Ok(24),
        0 | 4 | 5 | 7 | 8 | 10 | 11 | 12 | 14 => Ok(28),
        _ => Err(ModelFault::UnsupportedVertexFlags(flags)),
    }
}

fn vertex_ex_stride(flags: u32) -> Parsed<usize> {
    match flags {
        1 | 4 => Ok(8),
        5 | 7 => Ok(12),
        8 | 10 | 14 => Ok(16),
        11 | 12 => Ok(20),
        _ => Err(ModelFault::UnsupportedVertexFlags(flags)),
    }
}

fn vertex_ex_normal_offset(flags: u32) -> Parsed<usize> {
    match flags {
        1 | 4 | 5 | 12 | 14 => Ok(0),
        7 | 8 => Ok(4),
        10 | 11 => Ok(8),
        _ => Err(ModelFault::UnsupportedVertexFlags(flags)),
    }
}

fn normalized_byte(value: u8) -> f32 {
    (f32::from(value) - 127.0) / 127.0
}

pub fn half_to_f32(value: u16) -> f32 {
    let sign = u32::from(value & 0x8000) << 16;
    let exponent = (value >> 10) & 0x1f;
    let mantissa = value & 0x03ff;
    let bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut mantissa = u32::from(mantissa);
            let mut exponent = 113_u32;
            while mantissa & 0x0400 == 0 {
                mantissa <<= 1;
                exponent -= 1;
            }
            sign | (exponent << 23) | ((mantissa & 0x03ff) << 13)
        }
        31 => sign | 0x7f80_0000 | (u32::from(mantissa) << 13),
        _ => sign | ((u32::from(exponent) + 112) << 23) | (u32::from(mantissa) << 13),
    };
    f32::from_bits(bits)
}

#[cfg(test)]
#[path = "wmb3_tests.rs"]
mod synthetic;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_floats_convert() {
        assert_eq!(half_to_f32(0x3C00), 1.0);
        assert_eq!(half_to_f32(0xC000), -2.0);
        assert_eq!(half_to_f32(0x3800), 0.5);
        assert_eq!(half_to_f32(0x0000), 0.0);
        assert_eq!(half_to_f32(0x0001), 2.0_f32.powi(-24));
        assert!(half_to_f32(0x7C00).is_infinite());
        assert!(half_to_f32(0x7E00).is_nan());
    }

    #[test]
    fn rejects_non_models() {
        assert!(matches!(
            Model::parse(b"WMB4", None),
            Err(crate::FormatError::Model(ModelFault::NotWmb3))
        ));
        assert!(matches!(
            Model::parse(&[0u8; HEADER_SIZE], None),
            Err(crate::FormatError::Model(ModelFault::NotWmb3))
        ));
    }

    #[test]
    fn dilation_fills_transparent_pixels_from_neighbours() {
        let mut rgba = vec![200, 100, 50, 255, 0, 0, 0, 0];
        dilate_transparent_rgb(&mut rgba, 2, 1);
        assert_eq!(&rgba[4..8], &[200, 100, 50, 0]);
    }
}
