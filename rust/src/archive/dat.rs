use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

const HEADER_SIZE: u64 = 28;
const MAX_ENTRIES: usize = 100_000;
const MAX_NAME_LENGTH: usize = 4_096;
const MAX_ENTRY_SIZE: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct ArchiveEntry {
    pub index: u32,
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

pub struct DatArchive {
    file: File,
    length: u64,
    entries: Vec<ArchiveEntry>,
}

impl DatArchive {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let mut file = File::open(&path).map_err(|error| error.to_string())?;
        let length = file.metadata().map_err(|error| error.to_string())?.len();
        if length < HEADER_SIZE {
            return Err("Archive is smaller than its header".to_string());
        }

        let mut header = [0_u8; HEADER_SIZE as usize];
        file.read_exact(&mut header)
            .map_err(|error| error.to_string())?;

        let magic = &header[0..4];
        if magic != b"DAT\0" && magic != b"DTT\0" {
            return Err("Archive has an unsupported header".to_string());
        }

        let file_count = read_u32(&header, 4)? as usize;
        if file_count > MAX_ENTRIES {
            return Err("Archive contains too many entries".to_string());
        }

        let offsets_offset = read_u32(&header, 8)? as u64;
        let names_offset = read_u32(&header, 16)? as u64;
        let sizes_offset = read_u32(&header, 20)? as u64;
        let table_size = (file_count as u64)
            .checked_mul(4)
            .ok_or_else(|| "Archive table size overflowed".to_string())?;

        validate_range(offsets_offset, table_size, length)?;
        validate_range(sizes_offset, table_size, length)?;
        validate_range(names_offset, 4, length)?;

        let offsets = read_u32_table(&mut file, offsets_offset, file_count)?;
        let sizes = read_u32_table(&mut file, sizes_offset, file_count)?;

        file.seek(SeekFrom::Start(names_offset))
            .map_err(|error| error.to_string())?;
        let name_length = read_u32_from_file(&mut file)? as usize;
        if name_length == 0 || name_length > MAX_NAME_LENGTH {
            return Err("Archive name width is invalid".to_string());
        }

        let names_size = (file_count as u64)
            .checked_mul(name_length as u64)
            .ok_or_else(|| "Archive name table size overflowed".to_string())?;
        validate_range(names_offset + 4, names_size, length)?;

        let mut entries = Vec::with_capacity(file_count);
        let mut name_bytes = vec![0_u8; name_length];
        for index in 0..file_count {
            file.read_exact(&mut name_bytes)
                .map_err(|error| error.to_string())?;
            let end = name_bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(name_bytes.len());
            let name = String::from_utf8_lossy(&name_bytes[..end]).into_owned();
            let offset = offsets[index] as u64;
            let size = sizes[index] as u64;
            validate_range(offset, size, length)?;
            entries.push(ArchiveEntry {
                index: index as u32,
                name,
                offset,
                size,
            });
        }

        Ok(Self {
            file,
            length,
            entries,
        })
    }

    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    pub fn read_entry_prefix(
        &mut self,
        entry: &ArchiveEntry,
        requested: usize,
    ) -> Result<Vec<u8>, String> {
        let amount = requested.min(entry.size as usize);
        validate_range(entry.offset, amount as u64, self.length)?;
        self.file
            .seek(SeekFrom::Start(entry.offset))
            .map_err(|error| error.to_string())?;
        let mut bytes = vec![0_u8; amount];
        self.file
            .read_exact(bytes.as_mut_slice())
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }

    pub fn read_entry(&mut self, entry: &ArchiveEntry) -> Result<Vec<u8>, String> {
        if entry.size > MAX_ENTRY_SIZE {
            return Err("Archive entry is too large".to_string());
        }
        validate_range(entry.offset, entry.size, self.length)?;
        self.file
            .seek(SeekFrom::Start(entry.offset))
            .map_err(|error| error.to_string())?;
        let size = usize::try_from(entry.size)
            .map_err(|_| "Archive entry does not fit in memory".to_string())?;
        let mut bytes = vec![0_u8; size];
        self.file
            .read_exact(bytes.as_mut_slice())
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| "Integer offset overflowed".to_string())?;
    let value = bytes
        .get(offset..end)
        .ok_or_else(|| "Unexpected end of archive header".to_string())?;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| "Invalid integer width".to_string())?,
    ))
}

fn read_u32_from_file(file: &mut File) -> Result<u32, String> {
    let mut bytes = [0_u8; 4];
    file.read_exact(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u32_table(file: &mut File, offset: u64, count: usize) -> Result<Vec<u32>, String> {
    file.seek(SeekFrom::Start(offset))
        .map_err(|error| error.to_string())?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_u32_from_file(file)?);
    }
    Ok(values)
}

fn validate_range(offset: u64, size: u64, length: u64) -> Result<(), String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| "Archive range overflowed".to_string())?;
    if end > length {
        return Err("Archive range points outside the file".to_string());
    }
    Ok(())
}
