use crate::errors::{CodedError, ErrorCode};

pub struct DatEntry {
    pub name: String,
    pub data: Vec<u8>,
}

const CRC32_POLY: u32 = 0xEDB8_8320;

fn crc32(input: &str) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in input.bytes() {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (CRC32_POLY & mask);
        }
    }
    !crc
}

fn align_to(value: usize, alignment: usize) -> usize {
    value.div_ceil(alignment) * alignment
}

struct HashTable {
    pre_hash_shift: u32,
    bucket_offsets: Vec<i16>,
    hashes: Vec<u32>,
    indices: Vec<u16>,
}

impl HashTable {
    fn build(names: &[String]) -> Self {
        let count = if names.len() <= 1 {
            names.len() + 1
        } else {
            names.len()
        };
        let bit_length = usize::BITS - count.leading_zeros();
        let pre_hash_shift = 24.max(32 - bit_length);

        let bucket_count = 1usize << (31 - pre_hash_shift);
        let mut bucket_offsets = vec![-1i16; bucket_count];

        let mut rows: Vec<(String, usize, u32)> = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let hash = crc32(&name.to_lowercase()) & !0x8000_0000;
                (name.clone(), index, hash)
            })
            .collect();

        rows.sort_by(|a, b| {
            let ka = a.2 >> pre_hash_shift;
            let kb = b.2 >> pre_hash_shift;
            ka.cmp(&kb).then_with(|| a.0.cmp(&b.0))
        });

        let hashes: Vec<u32> = rows.iter().map(|row| row.2).collect();
        let mut indices = Vec::with_capacity(rows.len());

        for (position, row) in rows.iter().enumerate() {
            let bucket = (row.2 >> pre_hash_shift) as usize;
            if bucket < bucket_offsets.len() && bucket_offsets[bucket] == -1 {
                bucket_offsets[bucket] = position as i16;
            }
            indices.push(row.1 as u16);
        }

        Self {
            pre_hash_shift,
            bucket_offsets,
            hashes,
            indices,
        }
    }

    fn buckets_size(&self) -> usize {
        self.bucket_offsets.len() * 2
    }

    fn hashes_size(&self) -> usize {
        self.hashes.len() * 4
    }

    fn indices_size(&self) -> usize {
        self.indices.len() * 2
    }

    fn table_size(&self) -> usize {
        16 + self.buckets_size() + self.hashes_size() + self.indices_size()
    }
}

pub fn build_dat(entries: &[DatEntry], magic: &[u8; 4]) -> Result<Vec<u8>, CodedError> {
    if entries.is_empty() {
        return Err(CodedError::bare(ErrorCode::ArchiveEmpty));
    }

    let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
    let hash_table = HashTable::build(&names);

    let mut extensions = Vec::with_capacity(entries.len());
    let mut extensions_size = 0usize;
    for name in &names {
        let extension = name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
        if extension.len() > 3 {
            return Err(CodedError::new(ErrorCode::ArchiveExtensionTooLong, name));
        }
        let mut padded = extension.as_bytes().to_vec();
        padded.resize(3, 0);
        extensions_size += 4;
        extensions.push(padded);
    }

    let name_length = names
        .iter()
        .map(|name| name.len() + 1)
        .max()
        .unwrap_or(1);

    let offsets_offset = 32usize;
    let extensions_offset = offsets_offset + entries.len() * 4;
    let names_offset = extensions_offset + extensions_size;
    let sizes_offset = align_to(names_offset + 4 + entries.len() * name_length, 4);
    let hash_map_offset = sizes_offset + entries.len() * 4;

    let mut file_offsets = Vec::with_capacity(entries.len());
    let mut cursor = hash_map_offset + hash_table.table_size();
    for entry in entries {
        cursor = align_to(cursor, 32);
        file_offsets.push(cursor);
        cursor += entry.data.len();
    }

    let total = align_to(cursor + 1, 16);
    let mut out = vec![0u8; total];

    out[0..4].copy_from_slice(magic);
    write_u32(&mut out, 4, entries.len() as u32);
    write_u32(&mut out, 8, offsets_offset as u32);
    write_u32(&mut out, 12, extensions_offset as u32);
    write_u32(&mut out, 16, names_offset as u32);
    write_u32(&mut out, 20, sizes_offset as u32);
    write_u32(&mut out, 24, hash_map_offset as u32);

    for (index, offset) in file_offsets.iter().enumerate() {
        write_u32(&mut out, offsets_offset + index * 4, *offset as u32);
    }

    let mut position = extensions_offset;
    for extension in &extensions {
        out[position..position + 3].copy_from_slice(extension);
        position += 4;
    }

    write_u32(&mut out, names_offset, name_length as u32);
    position = names_offset + 4;
    for name in &names {
        let bytes = name.as_bytes();
        out[position..position + bytes.len()].copy_from_slice(bytes);
        position += name_length;
    }

    for (index, entry) in entries.iter().enumerate() {
        write_u32(&mut out, sizes_offset + index * 4, entry.data.len() as u32);
    }

    position = hash_map_offset;
    write_u32(&mut out, position, hash_table.pre_hash_shift);
    write_u32(&mut out, position + 4, 16);
    write_u32(&mut out, position + 8, (16 + hash_table.buckets_size()) as u32);
    write_u32(
        &mut out,
        position + 12,
        (16 + hash_table.buckets_size() + hash_table.hashes_size()) as u32,
    );

    position = hash_map_offset + 16;
    for value in &hash_table.bucket_offsets {
        write_u16(&mut out, position, *value as u16);
        position += 2;
    }
    for value in &hash_table.hashes {
        write_u32(&mut out, position, *value);
        position += 4;
    }
    for value in &hash_table.indices {
        write_u16(&mut out, position, *value);
        position += 2;
    }

    for (index, entry) in entries.iter().enumerate() {
        let start = file_offsets[index];
        out[start..start + entry.data.len()].copy_from_slice(&entry.data);
    }

    Ok(out)
}

fn write_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u16(buffer: &mut [u8], offset: usize, value: u16) {
    buffer[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub fn name_crc(name: &str) -> u32 {
    crc32(name) & !0x8000_0000
}
