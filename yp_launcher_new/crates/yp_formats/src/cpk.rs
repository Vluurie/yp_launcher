#![forbid(unsafe_code)]

mod layla;
#[cfg(test)]
mod tests;
mod utf;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use utf::Table;

const MAX_TABLE_SIZE: u64 = 16 * 1024 * 1024;

fn read_at(file: &mut File, offset: u64, size: u64, limit: u64) -> Option<Vec<u8>> {
    if size > limit || offset.checked_add(size)? > file.metadata().ok()?.len() {
        return None;
    }
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut bytes = vec![0; usize::try_from(size).ok()?];
    file.read_exact(&mut bytes).ok()?;
    Some(bytes)
}

fn packet(file: &mut File, at: u64, magic: &[u8; 4]) -> Option<Vec<u8>> {
    file.seek(SeekFrom::Start(at)).ok()?;
    let mut header = [0u8; 16];
    file.read_exact(&mut header).ok()?;
    if &header[..4] != magic {
        return None;
    }
    let size = u64::from_le_bytes(header[8..].try_into().ok()?);
    read_at(file, at.checked_add(16)?, size, MAX_TABLE_SIZE)
}

pub fn read_member(path: &Path, directory: &str, name: &str, limit: u64) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    let header = packet(&mut file, 0, b"CPK ")?;
    let header = Table::parse(&header)?;
    let toc_offset = header.number(header.column("TocOffset")?, 0)?;
    let content = header.number(header.column("ContentOffset")?, 0)?;
    let toc = packet(&mut file, toc_offset, b"TOC ")?;
    let toc = Table::parse(&toc)?;
    let filename = toc.column("FileName")?;
    let dirname = toc.column("DirName")?;
    let row = (0..toc.len()).find(|&i| {
        toc.text(filename, i)
            .is_some_and(|n| n.eq_ignore_ascii_case(name))
            && toc
                .text(dirname, i)
                .is_some_and(|d| d.eq_ignore_ascii_case(directory))
    })?;
    let offset = content
        .min(toc_offset)
        .checked_add(toc.number(toc.column("FileOffset")?, row)?)?;
    let stored = toc.number(toc.column("FileSize")?, row)?;
    let expanded = toc.number(toc.column("ExtractSize")?, row)?;
    if expanded == 0 || expanded > limit {
        return None;
    }
    let bytes = read_at(&mut file, offset, stored, limit)?;
    if stored == expanded {
        return Some(bytes);
    }
    layla::decompress(&bytes, usize::try_from(expanded).ok()?)
}
