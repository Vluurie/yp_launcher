use crate::dat::{self, DatEntry, DatKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl TestVertex {
    pub fn new(position: [f32; 3], uv: [f32; 2], normal: [f32; 3]) -> Self {
        Self {
            position,
            uv,
            normal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestLod {
    pub name: String,
    pub batch_start: u32,
    pub infos: Vec<[u32; 3]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WmbSpec {
    pub vertex_flags: u32,
    pub wide_indices: bool,
    pub vertices: Vec<TestVertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<[u32; 2]>,
    pub lods: Vec<TestLod>,
    pub materials: Vec<Vec<(String, u32)>>,
    pub mesh_names: Vec<String>,
}

pub fn f32_to_half(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x007f_ffff;
    if exponent == 0xff {
        return sign | 0x7c00 | if mantissa != 0 { 0x200 } else { 0 };
    }
    let half_exponent = exponent - 127 + 15;
    if half_exponent >= 0x1f {
        return sign | 0x7c00;
    }
    if half_exponent <= 0 {
        if half_exponent < -10 {
            return sign;
        }
        let full = mantissa | 0x0080_0000;
        let shift = (14 - half_exponent) as u32;
        return sign | (full >> shift) as u16;
    }
    sign | ((half_exponent as u16) << 10) | (mantissa >> 13) as u16
}

struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn align(&mut self, alignment: usize) {
        while !self.bytes.len().is_multiple_of(alignment) {
            self.bytes.push(0);
        }
    }

    fn here(&mut self) -> u32 {
        self.align(4);
        self.bytes.len() as u32
    }

    fn put_u32_at(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn string(&mut self, value: &str) -> u32 {
        let offset = self.bytes.len() as u32;
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(0);
        offset
    }

    fn zeros(&mut self, count: usize) {
        self.bytes.extend(std::iter::repeat_n(0u8, count));
    }
}

fn extended_layout(flags: u32) -> Option<(usize, usize)> {
    match flags {
        0 => None,
        1 | 4 => Some((8, 0)),
        5 => Some((12, 0)),
        7 => Some((12, 4)),
        10 => Some((16, 8)),
        14 => Some((16, 0)),
        11 => Some((20, 8)),
        12 => Some((20, 0)),
        _ => Some((8, 0)),
    }
}

impl WmbSpec {
    pub fn build(&self) -> Vec<u8> {
        let mut w = Writer {
            bytes: vec![0u8; 136],
        };
        w.bytes[0..4].copy_from_slice(b"WMB3");
        w.bytes[4..8].copy_from_slice(&crate::wmb3::GAME_LAYOUT_VERSION.to_le_bytes());
        let header_flags: u16 = if self.wide_indices { 0x8 } else { 0 };
        w.bytes[12..14].copy_from_slice(&header_flags.to_le_bytes());

        let stride = if self.vertex_flags == 1 { 24 } else { 28 };
        let vertex_offset = w.here();
        for vertex in &self.vertices {
            let start = w.bytes.len();
            for axis in vertex.position {
                w.f32(axis);
            }
            w.bytes.extend_from_slice(&[127, 127, 254, 0]);
            w.u16(f32_to_half(vertex.uv[0]));
            w.u16(f32_to_half(vertex.uv[1]));
            if self.vertex_flags == 0 {
                for axis in vertex.normal {
                    w.u16(f32_to_half(axis));
                }
            }
            let written = w.bytes.len() - start;
            w.zeros(stride - written);
        }

        let ex_offset = w.here();
        if let Some((ex_stride, normal_at)) = extended_layout(self.vertex_flags) {
            for vertex in &self.vertices {
                let start = w.bytes.len();
                w.zeros(normal_at);
                for axis in vertex.normal {
                    w.u16(f32_to_half(axis));
                }
                let written = w.bytes.len() - start;
                w.zeros(ex_stride - written);
            }
        }

        let index_offset = w.here();
        for index in &self.indices {
            if self.wide_indices {
                w.u32(*index);
            } else {
                w.u16(*index as u16);
            }
        }

        let group_offset = w.here();
        w.u32(vertex_offset);
        w.u32(ex_offset);
        w.zeros(24);
        w.u32(self.vertices.len() as u32);
        w.u32(self.vertex_flags);
        w.u32(index_offset);
        w.u32(self.indices.len() as u32);

        let batch_offset = w.here();
        for [start, count] in &self.batches {
            w.zeros(12);
            w.u32(*start);
            w.zeros(4);
            w.u32(*count);
            w.zeros(4);
        }

        let mut lod_info_offsets = Vec::new();
        for lod in &self.lods {
            let offset = w.here();
            for [group, mesh, material] in &lod.infos {
                w.u32(*group);
                w.u32(*mesh);
                w.u32(*material);
                w.zeros(12);
            }
            lod_info_offsets.push(offset);
        }
        let lod_offset = w.here();
        let lod_records: Vec<usize> = self
            .lods
            .iter()
            .map(|_| {
                let at = w.bytes.len();
                w.zeros(20);
                at
            })
            .collect();

        let mut texture_tables = Vec::new();
        for material in &self.materials {
            let offset = w.here();
            let slots: Vec<usize> = material
                .iter()
                .map(|(_, id)| {
                    let at = w.bytes.len();
                    w.u32(0);
                    w.u32(*id);
                    at
                })
                .collect();
            texture_tables.push((offset, slots));
        }
        let material_offset = w.here();
        for (offset, slots) in &texture_tables {
            w.zeros(24);
            w.u32(*offset);
            w.u32(slots.len() as u32);
            w.zeros(16);
        }

        let mesh_offset = w.here();
        let mesh_records: Vec<usize> = self
            .mesh_names
            .iter()
            .map(|_| {
                let at = w.bytes.len();
                w.zeros(44);
                at
            })
            .collect();

        for (index, lod) in self.lods.iter().enumerate() {
            let name_at = w.string(&lod.name);
            let record = lod_records[index];
            w.put_u32_at(record, name_at);
            w.put_u32_at(record + 8, lod.batch_start);
            w.put_u32_at(record + 12, lod_info_offsets[index]);
            w.put_u32_at(record + 16, lod.infos.len() as u32);
        }
        for (material, (_, slots)) in self.materials.iter().zip(&texture_tables) {
            for ((slot_name, _), at) in material.iter().zip(slots) {
                let name_at = w.string(slot_name);
                w.put_u32_at(*at, name_at);
            }
        }
        for (name, record) in self.mesh_names.iter().zip(&mesh_records) {
            let name_at = w.string(name);
            w.put_u32_at(*record, name_at);
        }

        w.put_u32_at(56, group_offset);
        w.put_u32_at(60, 1);
        w.put_u32_at(64, batch_offset);
        w.put_u32_at(68, self.batches.len() as u32);
        w.put_u32_at(72, lod_offset);
        w.put_u32_at(76, self.lods.len() as u32);
        w.put_u32_at(104, material_offset);
        w.put_u32_at(108, self.materials.len() as u32);
        w.put_u32_at(112, mesh_offset);
        w.put_u32_at(116, self.mesh_names.len() as u32);
        w.bytes
    }
}

pub fn quad_spec() -> WmbSpec {
    WmbSpec {
        vertex_flags: 0,
        wide_indices: false,
        vertices: vec![
            TestVertex::new([-1.0, -1.0, 0.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
            TestVertex::new([1.0, -1.0, 0.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
            TestVertex::new([1.0, 1.0, 0.0], [1.0, 1.0], [0.0, 0.0, 1.0]),
            TestVertex::new([-1.0, 1.0, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0]),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        batches: vec![[0, 6]],
        lods: vec![TestLod {
            name: "LOD0".to_string(),
            batch_start: 0,
            infos: vec![[0, 0, 0]],
        }],
        materials: vec![vec![("g_AlbedoMap".to_string(), 0xA1)]],
        mesh_names: vec!["Body".to_string()],
    }
}

pub fn cube_spec(mesh_names: &[&str]) -> WmbSpec {
    let corners = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
    ];
    let vertices = corners
        .iter()
        .map(|corner: &[f32; 3]| {
            let length = (3.0_f32).sqrt();
            TestVertex::new(
                *corner,
                [(corner[0] + 1.0) * 0.5, (corner[1] + 1.0) * 0.5],
                [corner[0] / length, corner[1] / length, corner[2] / length],
            )
        })
        .collect();
    let faces: [[u32; 4]; 6] = [
        [0, 1, 2, 3],
        [5, 4, 7, 6],
        [4, 0, 3, 7],
        [1, 5, 6, 2],
        [3, 2, 6, 7],
        [4, 5, 1, 0],
    ];
    let mut indices = Vec::new();
    for face in faces {
        indices.extend_from_slice(&[face[0], face[1], face[2], face[0], face[2], face[3]]);
    }
    let per_mesh = 36 / mesh_names.len().max(1) as u32 / 3 * 3;
    let batches: Vec<[u32; 2]> = (0..mesh_names.len() as u32)
        .map(|index| {
            let start = index * per_mesh;
            let count = if index as usize == mesh_names.len() - 1 {
                36 - start
            } else {
                per_mesh
            };
            [start, count]
        })
        .collect();
    WmbSpec {
        vertex_flags: 0,
        wide_indices: false,
        vertices,
        indices,
        lods: vec![TestLod {
            name: "LOD0".to_string(),
            batch_start: 0,
            infos: (0..batches.len() as u32)
                .map(|index| [0, index, 0])
                .collect(),
        }],
        batches,
        materials: vec![vec![("g_AlbedoMap".to_string(), 0xC0BE)]],
        mesh_names: mesh_names.iter().map(|name| name.to_string()).collect(),
    }
}

pub fn dds_dxt1_solid(width: u32, height: u32, color_565: u16) -> Vec<u8> {
    let mut out = vec![0u8; 128];
    out[0..4].copy_from_slice(b"DDS ");
    out[4..8].copy_from_slice(&124u32.to_le_bytes());
    out[8..12].copy_from_slice(&0x0008_1007u32.to_le_bytes());
    out[12..16].copy_from_slice(&height.to_le_bytes());
    out[16..20].copy_from_slice(&width.to_le_bytes());
    let blocks = width.div_ceil(4) * height.div_ceil(4);
    out[20..24].copy_from_slice(&(blocks * 8).to_le_bytes());
    out[28..32].copy_from_slice(&1u32.to_le_bytes());
    out[76..80].copy_from_slice(&32u32.to_le_bytes());
    out[80..84].copy_from_slice(&4u32.to_le_bytes());
    out[84..88].copy_from_slice(b"DXT1");
    out[108..112].copy_from_slice(&0x1000u32.to_le_bytes());
    for _ in 0..blocks {
        out.extend_from_slice(&color_565.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&[0, 0, 0, 0]);
    }
    out
}

pub fn texture_index(textures: &[(u32, u32, u32)]) -> Vec<u8> {
    let count = textures.len() as u32;
    let offsets_at = 32u32;
    let sizes_at = offsets_at + count * 4;
    let flags_at = sizes_at + count * 4;
    let ids_at = flags_at + count * 4;
    let mut out = vec![0u8; (ids_at + count * 4) as usize];
    out[0..4].copy_from_slice(b"WTB\0");
    out[4..8].copy_from_slice(&1u32.to_le_bytes());
    out[8..12].copy_from_slice(&count.to_le_bytes());
    out[12..16].copy_from_slice(&offsets_at.to_le_bytes());
    out[16..20].copy_from_slice(&sizes_at.to_le_bytes());
    out[20..24].copy_from_slice(&flags_at.to_le_bytes());
    out[24..28].copy_from_slice(&ids_at.to_le_bytes());
    for (index, (id, offset, size)) in textures.iter().enumerate() {
        let index = index as u32;
        let put = |out: &mut Vec<u8>, at: u32, value: u32| {
            out[at as usize..at as usize + 4].copy_from_slice(&value.to_le_bytes());
        };
        put(&mut out, offsets_at + index * 4, *offset);
        put(&mut out, sizes_at + index * 4, *size);
        put(&mut out, ids_at + index * 4, *id);
    }
    out
}

pub fn texture_pair(textures: &[(u32, Vec<u8>)]) -> (Vec<u8>, Vec<u8>) {
    let mut payload = Vec::new();
    let mut records = Vec::new();
    for (id, dds) in textures {
        while payload.len() % 16 != 0 {
            payload.push(0);
        }
        records.push((*id, payload.len() as u32, dds.len() as u32));
        payload.extend_from_slice(dds);
    }
    (texture_index(&records), payload)
}

pub struct ModelArchives {
    pub dat: Vec<u8>,
    pub dtt: Vec<u8>,
}

pub fn model_archives(stem: &str, wmb: Vec<u8>, textures: &[(u32, Vec<u8>)]) -> ModelArchives {
    let (wta, wtp) = texture_pair(textures);
    let dat = dat::build(
        &[
            DatEntry::new(format!("{stem}.wta"), wta),
            DatEntry::new(format!("{stem}.bxm"), vec![0; 8]),
        ],
        DatKind::Dat,
    )
    .expect("model dat");
    let dtt = dat::build(
        &[
            DatEntry::new(format!("{stem}.wmb"), wmb),
            DatEntry::new(format!("{stem}.wtp"), wtp),
        ],
        DatKind::Dtt,
    )
    .expect("model dtt");
    ModelArchives { dat, dtt }
}

fn utf_header_table(width: u32, height: u32, codec: u32) -> Vec<u8> {
    let names = ["width", "height", "disp_width", "disp_height", "mpeg_codec"];
    let values = [width, height, width, height, codec];
    let mut strings = b"<NULL>\0CRIUSF_VIDEOHDR_TABLE\0".to_vec();
    let mut descriptors = Vec::new();
    for name in names {
        descriptors.push(0x54);
        descriptors.extend_from_slice(&(strings.len() as u32).to_be_bytes());
        strings.extend_from_slice(name.as_bytes());
        strings.push(0);
    }
    let mut row = Vec::new();
    for value in values {
        row.extend_from_slice(&value.to_be_bytes());
    }
    let rows_offset = 24 + descriptors.len();
    let strings_offset = rows_offset + row.len();
    let data_offset = strings_offset + strings.len();
    let mut out = Vec::new();
    out.extend_from_slice(b"@UTF");
    out.extend_from_slice(&(data_offset as u32).to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&(rows_offset as u16).to_be_bytes());
    out.extend_from_slice(&(strings_offset as u32).to_be_bytes());
    out.extend_from_slice(&(data_offset as u32).to_be_bytes());
    out.extend_from_slice(&7u32.to_be_bytes());
    out.extend_from_slice(&(names.len() as u16).to_be_bytes());
    out.extend_from_slice(&(row.len() as u16).to_be_bytes());
    out.extend_from_slice(&1u32.to_be_bytes());
    out.extend_from_slice(&descriptors);
    out.extend_from_slice(&row);
    out.extend_from_slice(&strings);
    out
}

fn usm_chunk(signature: &[u8; 4], kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(signature);
    out.extend_from_slice(&((0x18 + payload.len()) as u32).to_be_bytes());
    out.extend_from_slice(&[0, 0x18, 0, 0, 0, 0, 0, kind]);
    out.extend_from_slice(&[0; 16]);
    out.extend_from_slice(payload);
    out
}

pub fn usm_file(width: u32, height: u32, codec: u32) -> Vec<u8> {
    let mut out = usm_chunk(b"CRID", 1, &[0; 32]);
    out.extend(usm_chunk(
        b"@SFV",
        1,
        &utf_header_table(width, height, codec),
    ));
    out.extend(usm_chunk(b"@SFV", 0, &[0x5A; 48]));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_conversion_round_trips_simple_values() {
        for value in [0.0f32, 1.0, -1.0, 0.5, 0.25, 0.577_35, 2.0, 1e-6, 65504.0] {
            let back = crate::wmb3::half_to_f32(f32_to_half(value));
            assert!(
                (back - value).abs() <= value.abs() * 0.001 + 1e-6,
                "{value} -> {back}"
            );
        }
        assert!(crate::wmb3::half_to_f32(f32_to_half(1e9)).is_infinite());
    }

    #[test]
    fn test_usm_parses() {
        let info =
            crate::usm::read_video_info(std::io::Cursor::new(usm_file(2560, 1440, 5))).unwrap();
        assert_eq!(
            (info.width, info.height, info.codec.code()),
            (2560, 1440, 5)
        );
    }
}
