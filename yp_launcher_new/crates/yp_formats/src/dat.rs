use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::bytes::{le_u32, nul_terminated, put_u16, put_u32};
use crate::error::{DatFault, FileOp, FormatError, Result};

const HEADER_SIZE: u64 = 28;
const MAX_ENTRIES: usize = 100_000;
const MAX_NAME_LENGTH: usize = 4_096;
const MAX_ENTRY_SIZE: u64 = 512 * 1024 * 1024;
const MAX_PACKED_ENTRIES: usize = i16::MAX as usize;
const CRC32_POLY: u32 = 0xEDB8_8320;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatKind {
    Dat,
    Dtt,
}

impl DatKind {
    pub fn magic(self) -> [u8; 4] {
        match self {
            DatKind::Dat => *b"DAT\0",
            DatKind::Dtt => *b"DTT\0",
        }
    }

    pub fn from_magic(magic: [u8; 4]) -> Option<Self> {
        match &magic {
            b"DAT\0" => Some(DatKind::Dat),
            b"DTT\0" => Some(DatKind::Dtt),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub index: u32,
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

impl ArchiveEntry {
    pub fn extension(&self) -> Option<&str> {
        self.name.rsplit_once('.').map(|(_, extension)| extension)
    }

    pub fn stem(&self) -> &str {
        self.name
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(&self.name)
    }

    pub fn has_extension(&self, extension: &str) -> bool {
        self.extension()
            .is_some_and(|own| own.eq_ignore_ascii_case(extension))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatEntry {
    pub name: String,
    pub data: Vec<u8>,
}

impl DatEntry {
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
        }
    }
}

pub struct DatArchive<R = BufReader<File>> {
    reader: R,
    origin: PathBuf,
    kind: DatKind,
    length: u64,
    entries: Vec<ArchiveEntry>,
}

impl DatArchive<BufReader<File>> {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|error| FormatError::io(FileOp::Read, path, error))?;
        Self::load(BufReader::new(file), path.to_path_buf())
    }
}

impl<R: Read + Seek> DatArchive<R> {
    pub fn from_reader(reader: R) -> Result<Self> {
        Self::load(reader, PathBuf::new())
    }

    fn load(mut reader: R, origin: PathBuf) -> Result<Self> {
        let length = reader
            .seek(SeekFrom::End(0))
            .map_err(|error| FormatError::io(FileOp::Read, &origin, error))?;
        if length < HEADER_SIZE {
            return Err(DatFault::SmallerThanHeader.into());
        }
        let mut header = [0_u8; HEADER_SIZE as usize];
        read_at(&mut reader, &origin, 0, &mut header)?;
        let kind = DatKind::from_magic([header[0], header[1], header[2], header[3]])
            .ok_or(DatFault::UnsupportedMagic)?;

        let file_count = header_u32(&header, 4)? as usize;
        if file_count > MAX_ENTRIES {
            return Err(DatFault::TooManyEntries.into());
        }
        let offsets_offset = u64::from(header_u32(&header, 8)?);
        let names_offset = u64::from(header_u32(&header, 16)?);
        let sizes_offset = u64::from(header_u32(&header, 20)?);
        let table_size = (file_count as u64)
            .checked_mul(4)
            .ok_or(DatFault::TableOverflow)?;

        validate_range(offsets_offset, table_size, length)?;
        validate_range(sizes_offset, table_size, length)?;
        validate_range(names_offset, 4, length)?;

        let offsets = read_u32_table(&mut reader, &origin, offsets_offset, file_count)?;
        let sizes = read_u32_table(&mut reader, &origin, sizes_offset, file_count)?;

        let mut width_bytes = [0_u8; 4];
        read_at(&mut reader, &origin, names_offset, &mut width_bytes)?;
        let name_length = u32::from_le_bytes(width_bytes) as usize;
        if name_length == 0 || name_length > MAX_NAME_LENGTH {
            return Err(DatFault::InvalidNameWidth.into());
        }
        let names_size = (file_count as u64)
            .checked_mul(name_length as u64)
            .ok_or(DatFault::TableOverflow)?;
        let names_start = names_offset + 4;
        validate_range(names_start, names_size, length)?;
        let mut names = vec![0_u8; names_size as usize];
        read_at(&mut reader, &origin, names_start, &mut names)?;

        let mut entries = Vec::with_capacity(file_count);
        for (index, raw) in names.chunks_exact(name_length).enumerate() {
            let name = String::from_utf8_lossy(nul_terminated(raw)).into_owned();
            let offset = u64::from(offsets[index]);
            let size = u64::from(sizes[index]);
            validate_range(offset, size, length)?;
            entries.push(ArchiveEntry {
                index: index as u32,
                name,
                offset,
                size,
            });
        }

        Ok(Self {
            reader,
            origin,
            kind,
            length,
            entries,
        })
    }

    pub fn kind(&self) -> DatKind {
        self.kind
    }

    pub fn origin(&self) -> &Path {
        &self.origin
    }

    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    pub fn entry(&self, index: u32) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|entry| entry.index == index)
    }

    pub fn find(&self, name: &str) -> Option<&ArchiveEntry> {
        self.entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
    }

    pub fn read_entry_prefix(&mut self, entry: &ArchiveEntry, requested: usize) -> Result<Vec<u8>> {
        let amount = (requested as u64).min(entry.size);
        validate_range(entry.offset, amount, self.length)?;
        let mut bytes = vec![0_u8; amount as usize];
        read_at(&mut self.reader, &self.origin, entry.offset, &mut bytes)?;
        Ok(bytes)
    }

    pub fn read_entry_range(
        &mut self,
        entry: &ArchiveEntry,
        start: u64,
        len: u64,
    ) -> Result<Vec<u8>> {
        if start.checked_add(len).is_none_or(|end| end > entry.size) {
            return Err(DatFault::RangeOutsideFile.into());
        }
        if len > MAX_ENTRY_SIZE {
            return Err(DatFault::EntryTooLarge.into());
        }
        let offset = entry.offset + start;
        validate_range(offset, len, self.length)?;
        let mut bytes = vec![0_u8; usize::try_from(len).map_err(|_| DatFault::EntryTooLarge)?];
        read_at(&mut self.reader, &self.origin, offset, &mut bytes)?;
        Ok(bytes)
    }

    pub fn read_entry(&mut self, entry: &ArchiveEntry) -> Result<Vec<u8>> {
        if entry.size > MAX_ENTRY_SIZE {
            return Err(DatFault::EntryTooLarge.into());
        }
        validate_range(entry.offset, entry.size, self.length)?;
        let size = usize::try_from(entry.size).map_err(|_| DatFault::EntryTooLarge)?;
        let mut bytes = vec![0_u8; size];
        read_at(&mut self.reader, &self.origin, entry.offset, &mut bytes)?;
        Ok(bytes)
    }

    pub fn read_named(&mut self, name: &str) -> Result<Option<Vec<u8>>> {
        let Some(entry) = self.find(name).cloned() else {
            return Ok(None);
        };
        self.read_entry(&entry).map(Some)
    }

    pub fn read_all(&mut self) -> Result<Vec<DatEntry>> {
        let listed = self.entries.clone();
        let mut out = Vec::with_capacity(listed.len());
        for entry in &listed {
            let data = self.read_entry(entry)?;
            out.push(DatEntry {
                name: entry.name.clone(),
                data,
            });
        }
        Ok(out)
    }
}

fn header_u32(header: &[u8], offset: usize) -> Result<u32> {
    le_u32(header, offset).ok_or_else(|| DatFault::SmallerThanHeader.into())
}

fn read_at<R: Read + Seek>(
    reader: &mut R,
    origin: &Path,
    offset: u64,
    buffer: &mut [u8],
) -> Result<()> {
    reader
        .seek(SeekFrom::Start(offset))
        .and_then(|_| reader.read_exact(buffer))
        .map_err(|error| FormatError::io(FileOp::Read, origin, error))
}

fn read_u32_table<R: Read + Seek>(
    reader: &mut R,
    origin: &Path,
    offset: u64,
    count: usize,
) -> Result<Vec<u32>> {
    let mut raw = vec![0_u8; count * 4];
    read_at(reader, origin, offset, &mut raw)?;
    Ok(raw
        .chunks_exact(4)
        .map(|value| u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
        .collect())
}

fn validate_range(offset: u64, size: u64, length: u64) -> Result<()> {
    let end = offset.checked_add(size).ok_or(DatFault::TableOverflow)?;
    if end > length {
        return Err(DatFault::RangeOutsideFile.into());
    }
    Ok(())
}

pub fn crc32(input: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in input {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (CRC32_POLY & mask);
        }
    }
    !crc
}

pub fn name_crc(name: &str) -> u32 {
    crc32(name.as_bytes()) & !0x8000_0000
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
    fn build(names: &[&str]) -> Self {
        let count = if names.len() <= 1 {
            names.len() + 1
        } else {
            names.len()
        };
        let bit_length = usize::BITS - count.leading_zeros();
        let pre_hash_shift = 24.max(32 - bit_length);
        let bucket_count = 1usize << (31 - pre_hash_shift);
        let mut bucket_offsets = vec![-1i16; bucket_count];

        let mut rows: Vec<(&str, usize, u32)> = names
            .iter()
            .enumerate()
            .map(|(index, name)| (*name, index, name_crc(&name.to_lowercase())))
            .collect();
        rows.sort_by(|a, b| {
            (a.2 >> pre_hash_shift)
                .cmp(&(b.2 >> pre_hash_shift))
                .then_with(|| a.0.cmp(b.0))
        });

        let hashes = rows.iter().map(|row| row.2).collect();
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

pub fn build(entries: &[DatEntry], kind: DatKind) -> Result<Vec<u8>> {
    if entries.is_empty() {
        return Err(FormatError::ArchiveEmpty);
    }
    if entries.len() > MAX_PACKED_ENTRIES {
        return Err(FormatError::ArchiveTooManyEntries {
            count: entries.len(),
        });
    }

    let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
    let mut extensions = Vec::with_capacity(entries.len());
    for name in &names {
        let extension = name
            .rsplit_once('.')
            .map(|(_, extension)| extension)
            .unwrap_or("");
        if extension.len() > 3 {
            return Err(FormatError::ArchiveExtensionTooLong {
                name: (*name).to_string(),
            });
        }
        let mut padded = [0_u8; 3];
        padded[..extension.len()].copy_from_slice(extension.as_bytes());
        extensions.push(padded);
    }
    let hash_table = HashTable::build(&names);

    let name_length = names.iter().map(|name| name.len() + 1).max().unwrap_or(1);
    let offsets_offset = 32usize;
    let extensions_offset = offsets_offset + entries.len() * 4;
    let names_offset = extensions_offset + extensions.len() * 4;
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

    out[0..4].copy_from_slice(&kind.magic());
    put_u32(&mut out, 4, entries.len() as u32);
    put_u32(&mut out, 8, offsets_offset as u32);
    put_u32(&mut out, 12, extensions_offset as u32);
    put_u32(&mut out, 16, names_offset as u32);
    put_u32(&mut out, 20, sizes_offset as u32);
    put_u32(&mut out, 24, hash_map_offset as u32);

    for (index, offset) in file_offsets.iter().enumerate() {
        put_u32(&mut out, offsets_offset + index * 4, *offset as u32);
    }

    let mut position = extensions_offset;
    for extension in &extensions {
        out[position..position + 3].copy_from_slice(extension);
        position += 4;
    }

    put_u32(&mut out, names_offset, name_length as u32);
    position = names_offset + 4;
    for name in &names {
        let bytes = name.as_bytes();
        out[position..position + bytes.len()].copy_from_slice(bytes);
        position += name_length;
    }

    for (index, entry) in entries.iter().enumerate() {
        put_u32(&mut out, sizes_offset + index * 4, entry.data.len() as u32);
    }

    position = hash_map_offset;
    put_u32(&mut out, position, hash_table.pre_hash_shift);
    put_u32(&mut out, position + 4, 16);
    put_u32(
        &mut out,
        position + 8,
        (16 + hash_table.buckets_size()) as u32,
    );
    put_u32(
        &mut out,
        position + 12,
        (16 + hash_table.buckets_size() + hash_table.hashes_size()) as u32,
    );

    position = hash_map_offset + 16;
    for value in &hash_table.bucket_offsets {
        put_u16(&mut out, position, *value as u16);
        position += 2;
    }
    for value in &hash_table.hashes {
        put_u32(&mut out, position, *value);
        position += 4;
    }
    for value in &hash_table.indices {
        put_u16(&mut out, position, *value);
        position += 2;
    }

    for (index, entry) in entries.iter().enumerate() {
        let start = file_offsets[index];
        out[start..start + entry.data.len()].copy_from_slice(&entry.data);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn crc_matches_the_standard_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(name_crc("123456789"), 0xCBF4_3926 & 0x7FFF_FFFF);
    }

    #[test]
    fn entry_name_parts() {
        let entry = ArchiveEntry {
            index: 0,
            name: "pl0000.WTA".to_string(),
            offset: 0,
            size: 0,
        };
        assert_eq!(entry.stem(), "pl0000");
        assert!(entry.has_extension("wta"));
        assert!(!entry.has_extension("wtp"));
    }

    #[test]
    fn ranges_inside_an_entry_read_only_that_slice() {
        let bytes = build(&[DatEntry::new("a.wtp", (0..32).collect())], DatKind::Dtt).unwrap();
        let mut archive = DatArchive::from_reader(Cursor::new(bytes)).unwrap();
        let entry = archive.find("a.wtp").cloned().unwrap();
        assert_eq!(archive.read_entry_range(&entry, 4, 3).unwrap(), [4, 5, 6]);
        assert!(archive.read_entry_range(&entry, 32, 0).unwrap().is_empty());
        assert!(archive.read_entry_range(&entry, 30, 3).is_err());
        assert!(archive.read_entry_range(&entry, u64::MAX, 2).is_err());
    }

    #[test]
    fn single_entry_hashes_into_two_buckets() {
        let table = HashTable::build(&["a.wtp"]);
        assert_eq!(table.pre_hash_shift, 30);
        assert_eq!(table.bucket_offsets.len(), 2);
        assert_eq!(
            table
                .bucket_offsets
                .iter()
                .filter(|offset| **offset == 0)
                .count(),
            1
        );
        assert_eq!(table.indices, vec![0]);
    }

    #[test]
    fn large_archives_use_a_smaller_shift() {
        let names: Vec<String> = (0..600).map(|index| format!("e{index}.bin")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let table = HashTable::build(&refs);
        assert_eq!(
            table.pre_hash_shift,
            24.max(32 - (usize::BITS - 600usize.leading_zeros()))
        );
        assert_eq!(table.indices.len(), 600);
    }
}
