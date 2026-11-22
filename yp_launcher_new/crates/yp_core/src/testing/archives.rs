use std::fs::File;
use std::io::Write;
use std::path::Path;

use sevenz_rust2::{ArchiveEntry, ArchiveWriter, SourceReader};
use zip::unstable::write::FileOptionsExt;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::archive::ArchiveFormat;

const CRC32_POLYNOMIAL: u32 = 0xedb8_8320;

const ZIP_LOCAL_HEADER: u32 = 0x0403_4b50;
const ZIP_CENTRAL_HEADER: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const ZIP_VERSION: u8 = 20;
const ZIP_DOS_DATE: u16 = 0x5821;
pub const ZIP_FLAG_ENCRYPTED: u16 = 0x0001;
pub const ZIP_FLAG_UTF8: u16 = 0x0800;
pub const ZIP_METHOD_STORED: u16 = 0;
pub const ZIP_HOST_DOS: u8 = 0;
pub const ZIP_HOST_UNIX: u8 = 3;

const RAR5_SIGNATURE: [u8; 8] = [0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01, 0x00];
const RAR5_MAIN_HEADER: u64 = 1;
const RAR5_FILE_HEADER: u64 = 2;
const RAR5_END_HEADER: u64 = 5;
const RAR5_HEADER_HAS_DATA: u64 = 0x0002;
const RAR5_HEADER_CONTINUES_IN_NEXT_VOLUME: u64 = 0x0010;
const RAR5_ARCHIVE_IS_VOLUME: u64 = 0x0001;
const RAR5_FILE_IS_DIRECTORY: u64 = 0x0001;
const RAR5_FILE_HAS_CRC: u64 = 0x0004;
const RAR5_END_NOT_LAST_VOLUME: u64 = 0x0001;
const RAR5_ATTRIBUTE_ARCHIVE: u64 = 0x20;
const RAR5_ATTRIBUTE_DIRECTORY: u64 = 0x10;

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 { (crc >> 1) ^ CRC32_POLYNOMIAL } else { crc >> 1 };
        }
    }
    !crc
}

pub fn noise(len: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    let mut bytes = Vec::with_capacity(len);
    while bytes.len() < len {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        bytes.extend_from_slice(&state.to_le_bytes());
    }
    bytes.truncate(len);
    bytes
}

pub fn write_zip_with(path: &Path, method: CompressionMethod, entries: &[(&str, &[u8])]) {
    let mut writer = ZipWriter::new(File::create(path).unwrap());
    let options = SimpleFileOptions::default().compression_method(method);
    for (name, bytes) in entries {
        if name.ends_with('/') {
            writer.add_directory(*name, options).unwrap();
        } else {
            writer.start_file(*name, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
    }
    writer.finish().unwrap();
}

pub fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
    write_zip_with(path, CompressionMethod::Deflated, entries);
}

pub fn write_encrypted_zip(path: &Path, password: &[u8], entries: &[(&str, &[u8])]) {
    let mut writer = ZipWriter::new(File::create(path).unwrap());
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .with_deprecated_encryption(password)
        .unwrap();
    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap();
}

pub fn write_seven_z(path: &Path, entries: &[(&str, &[u8])]) {
    let mut writer = ArchiveWriter::create(path).unwrap();
    for (name, bytes) in entries {
        match name.strip_suffix('/') {
            Some(dir) => writer
                .push_archive_entry::<&[u8]>(ArchiveEntry::new_directory(dir), None)
                .unwrap(),
            None => writer
                .push_archive_entry(ArchiveEntry::new_file(name), Some(*bytes))
                .unwrap(),
        };
    }
    writer.finish().unwrap();
}

pub fn write_solid_seven_z(path: &Path, entries: &[(&str, &[u8])]) {
    let mut writer = ArchiveWriter::create(path).unwrap();
    let files = entries.iter().map(|(name, _)| ArchiveEntry::new_file(name)).collect();
    let readers = entries.iter().map(|(_, bytes)| SourceReader::new(*bytes)).collect();
    writer.push_archive_entries(files, readers).unwrap();
    writer.finish().unwrap();
}

#[derive(Debug, Clone)]
pub struct RawZipEntry {
    pub name: Vec<u8>,
    pub flags: u16,
    pub method: u16,
    pub host: u8,
    pub extra: Vec<u8>,
    pub data: Vec<u8>,
}

impl RawZipEntry {
    pub fn stored(name: &[u8], data: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            flags: 0,
            method: ZIP_METHOD_STORED,
            host: ZIP_HOST_DOS,
            extra: Vec::new(),
            data: data.to_vec(),
        }
    }

    pub fn utf8(name: &str, data: &[u8]) -> Self {
        Self::stored(name.as_bytes(), data).flags(ZIP_FLAG_UTF8)
    }

    pub fn flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    pub fn method(mut self, method: u16) -> Self {
        self.method = method;
        self
    }

    pub fn host(mut self, host: u8) -> Self {
        self.host = host;
        self
    }

    pub fn extra(mut self, extra: &[u8]) -> Self {
        self.extra = extra.to_vec();
        self
    }
}

fn u16_len(len: usize) -> [u8; 2] {
    u16::try_from(len).unwrap().to_le_bytes()
}

fn u32_len(len: usize) -> [u8; 4] {
    u32::try_from(len).unwrap().to_le_bytes()
}

pub fn raw_zip_bytes(entries: &[RawZipEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for entry in entries {
        let offset = u32_len(out.len());
        let checksum = crc32(&entry.data).to_le_bytes();
        let size = u32_len(entry.data.len());
        out.extend_from_slice(&ZIP_LOCAL_HEADER.to_le_bytes());
        out.extend_from_slice(&[ZIP_VERSION, 0]);
        out.extend_from_slice(&entry.flags.to_le_bytes());
        out.extend_from_slice(&entry.method.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out.extend_from_slice(&ZIP_DOS_DATE.to_le_bytes());
        out.extend_from_slice(&checksum);
        out.extend_from_slice(&size);
        out.extend_from_slice(&size);
        out.extend_from_slice(&u16_len(entry.name.len()));
        out.extend_from_slice(&u16_len(entry.extra.len()));
        out.extend_from_slice(&entry.name);
        out.extend_from_slice(&entry.extra);
        out.extend_from_slice(&entry.data);

        central.extend_from_slice(&ZIP_CENTRAL_HEADER.to_le_bytes());
        central.extend_from_slice(&[ZIP_VERSION, entry.host]);
        central.extend_from_slice(&[ZIP_VERSION, 0]);
        central.extend_from_slice(&entry.flags.to_le_bytes());
        central.extend_from_slice(&entry.method.to_le_bytes());
        central.extend_from_slice(&[0, 0]);
        central.extend_from_slice(&ZIP_DOS_DATE.to_le_bytes());
        central.extend_from_slice(&checksum);
        central.extend_from_slice(&size);
        central.extend_from_slice(&size);
        central.extend_from_slice(&u16_len(entry.name.len()));
        central.extend_from_slice(&u16_len(entry.extra.len()));
        central.extend_from_slice(&[0; 10]);
        central.extend_from_slice(&offset);
        central.extend_from_slice(&entry.name);
        central.extend_from_slice(&entry.extra);
    }
    let central_offset = u32_len(out.len());
    let count = u16_len(entries.len());
    out.extend_from_slice(&central);
    out.extend_from_slice(&ZIP_END_OF_CENTRAL_DIRECTORY.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&count);
    out.extend_from_slice(&count);
    out.extend_from_slice(&u32_len(central.len()));
    out.extend_from_slice(&central_offset);
    out.extend_from_slice(&[0; 2]);
    out
}

pub fn write_raw_zip(path: &Path, entries: &[RawZipEntry]) {
    std::fs::write(path, raw_zip_bytes(entries)).unwrap();
}

#[derive(Debug, Clone)]
pub struct RawRarEntry {
    pub name: Vec<u8>,
    pub data: Vec<u8>,
    pub is_dir: bool,
    pub continues_in_next_volume: bool,
}

impl RawRarEntry {
    pub fn file(name: &str, data: &[u8]) -> Self {
        Self {
            name: name.as_bytes().to_vec(),
            data: data.to_vec(),
            is_dir: false,
            continues_in_next_volume: false,
        }
    }

    pub fn dir(name: &str) -> Self {
        Self {
            name: name.as_bytes().to_vec(),
            data: Vec::new(),
            is_dir: true,
            continues_in_next_volume: false,
        }
    }

    pub fn continued(mut self) -> Self {
        self.continues_in_next_volume = true;
        self
    }
}

fn vint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let low = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(low);
            return;
        }
        out.push(low | 0x80);
    }
}

fn rar_block(body: &[u8], out: &mut Vec<u8>) {
    let mut sized = Vec::new();
    vint(body.len() as u64, &mut sized);
    sized.extend_from_slice(body);
    out.extend_from_slice(&crc32(&sized).to_le_bytes());
    out.extend_from_slice(&sized);
}

pub fn raw_rar_bytes(entries: &[RawRarEntry]) -> Vec<u8> {
    let is_volume = entries.iter().any(|entry| entry.continues_in_next_volume);
    let mut out = RAR5_SIGNATURE.to_vec();
    let mut main = Vec::new();
    vint(RAR5_MAIN_HEADER, &mut main);
    vint(0, &mut main);
    vint(if is_volume { RAR5_ARCHIVE_IS_VOLUME } else { 0 }, &mut main);
    rar_block(&main, &mut out);
    for entry in entries {
        let mut header = Vec::new();
        vint(RAR5_FILE_HEADER, &mut header);
        if entry.is_dir {
            vint(0, &mut header);
            vint(RAR5_FILE_IS_DIRECTORY, &mut header);
            vint(0, &mut header);
            vint(RAR5_ATTRIBUTE_DIRECTORY, &mut header);
        } else {
            let mut flags = RAR5_HEADER_HAS_DATA;
            if entry.continues_in_next_volume {
                flags |= RAR5_HEADER_CONTINUES_IN_NEXT_VOLUME;
            }
            let unpacked = if entry.continues_in_next_volume {
                entry.data.len() as u64 * 2
            } else {
                entry.data.len() as u64
            };
            vint(flags, &mut header);
            vint(entry.data.len() as u64, &mut header);
            vint(RAR5_FILE_HAS_CRC, &mut header);
            vint(unpacked, &mut header);
            vint(RAR5_ATTRIBUTE_ARCHIVE, &mut header);
            header.extend_from_slice(&crc32(&entry.data).to_le_bytes());
        }
        vint(0, &mut header);
        vint(0, &mut header);
        vint(entry.name.len() as u64, &mut header);
        header.extend_from_slice(&entry.name);
        rar_block(&header, &mut out);
        out.extend_from_slice(&entry.data);
    }
    let mut end = Vec::new();
    vint(RAR5_END_HEADER, &mut end);
    vint(0, &mut end);
    vint(if is_volume { RAR5_END_NOT_LAST_VOLUME } else { 0 }, &mut end);
    rar_block(&end, &mut out);
    out
}

pub fn write_raw_rar(path: &Path, entries: &[RawRarEntry]) {
    std::fs::write(path, raw_rar_bytes(entries)).unwrap();
}

pub fn write_rar(path: &Path, entries: &[(&str, &[u8])]) {
    let raw: Vec<RawRarEntry> = entries
        .iter()
        .map(|(name, bytes)| match name.strip_suffix('/') {
            Some(dir) => RawRarEntry::dir(dir),
            None => RawRarEntry::file(name, bytes),
        })
        .collect();
    write_raw_rar(path, &raw);
}

pub fn write_archive(path: &Path, entries: &[(&str, &[u8])]) {
    match ArchiveFormat::from_extension(path).expect("a zip, 7z or rar file name") {
        ArchiveFormat::Zip => write_zip(path, entries),
        ArchiveFormat::SevenZ => write_seven_z(path, entries),
        ArchiveFormat::Rar => write_rar(path, entries),
    }
}

pub fn write_archive_with_folders(path: &Path, files: &[(&str, &[u8])]) {
    let mut entries: Vec<(String, &[u8])> = Vec::new();
    for (name, bytes) in files {
        for (index, _) in name.match_indices('/') {
            let folder = &name[..=index];
            if !entries.iter().any(|(listed, _)| listed == folder) {
                entries.push((folder.to_owned(), &[]));
            }
        }
        if !name.ends_with('/') {
            entries.push(((*name).to_owned(), bytes));
        }
    }
    let borrowed: Vec<(&str, &[u8])> = entries.iter().map(|(name, bytes)| (name.as_str(), *bytes)).collect();
    write_archive(path, &borrowed);
}

pub fn write_archive_of_dir(content_dir: &Path, path: &Path) {
    let mut found: Vec<(String, Vec<u8>)> = Vec::new();
    for entry in walkdir::WalkDir::new(content_dir).min_depth(1).sort_by_file_name() {
        let entry = entry.unwrap();
        let relative = crate::fs::rel_slash(content_dir, entry.path()).unwrap();
        if entry.file_type().is_dir() {
            found.push((format!("{relative}/"), Vec::new()));
        } else {
            found.push((relative, std::fs::read(entry.path()).unwrap()));
        }
    }
    let borrowed: Vec<(&str, &[u8])> = found.iter().map(|(name, bytes)| (name.as_str(), bytes.as_slice())).collect();
    write_archive_with_folders(path, &borrowed);
}
