#![forbid(unsafe_code)]

mod layla;
#[cfg(test)]
mod tests;
mod utf;

use std::fs::File;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use utf::Table;

const MAX_TABLE_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub directory: String,
    pub name: String,
    offset: u64,
    stored: u64,
    expanded: u64,
}

impl Member {
    pub fn is(&self, directory: &str, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name) && self.directory.eq_ignore_ascii_case(directory)
    }
}

pub struct Window {
    file: File,
    start: u64,
    len: u64,
    position: u64,
}

impl Read for Window {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let left = self.len.saturating_sub(self.position);
        let wanted = usize::try_from(left).map_or(buf.len(), |left| left.min(buf.len()));
        if wanted == 0 {
            return Ok(0);
        }
        self.file
            .seek(SeekFrom::Start(self.start + self.position))?;
        let read = self.file.read(&mut buf[..wanted])?;
        self.position += read as u64;
        Ok(read)
    }
}

impl Seek for Window {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let next = match to {
            SeekFrom::Start(at) => Some(at),
            SeekFrom::End(delta) => self.len.checked_add_signed(delta),
            SeekFrom::Current(delta) => self.position.checked_add_signed(delta),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before the member"))?;
        self.position = next;
        Ok(next)
    }
}

pub enum MemberReader {
    Stored(Window),
    Expanded(Cursor<Vec<u8>>),
}

impl Read for MemberReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            MemberReader::Stored(window) => window.read(buf),
            MemberReader::Expanded(cursor) => cursor.read(buf),
        }
    }
}

impl Seek for MemberReader {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        match self {
            MemberReader::Stored(window) => window.seek(to),
            MemberReader::Expanded(cursor) => cursor.seek(to),
        }
    }
}

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

fn listing(file: &mut File) -> Option<Vec<Member>> {
    let header = packet(file, 0, b"CPK ")?;
    let header = Table::parse(&header)?;
    let toc_offset = header.number(header.column("TocOffset")?, 0)?;
    let content = header.number(header.column("ContentOffset")?, 0)?;
    let toc = packet(file, toc_offset, b"TOC ")?;
    let toc = Table::parse(&toc)?;
    let filename = toc.column("FileName")?;
    let dirname = toc.column("DirName")?;
    let offset = toc.column("FileOffset")?;
    let stored = toc.column("FileSize")?;
    let expanded = toc.column("ExtractSize")?;
    let base = content.min(toc_offset);
    Some(
        (0..toc.len())
            .filter_map(|row| {
                Some(Member {
                    directory: toc.text(dirname, row).unwrap_or_default().to_owned(),
                    name: toc.text(filename, row)?.to_owned(),
                    offset: base.checked_add(toc.number(offset, row)?)?,
                    stored: toc.number(stored, row)?,
                    expanded: toc.number(expanded, row)?,
                })
            })
            .collect(),
    )
}

pub fn members(path: &Path) -> Option<Vec<Member>> {
    listing(&mut File::open(path).ok()?)
}

pub fn open(path: &Path, member: &Member, limit: u64) -> Option<MemberReader> {
    if member.expanded == 0 || member.expanded > limit {
        return None;
    }
    let mut file = File::open(path).ok()?;
    if member.stored == member.expanded {
        if member.offset.checked_add(member.stored)? > file.metadata().ok()?.len() {
            return None;
        }
        return Some(MemberReader::Stored(Window {
            file,
            start: member.offset,
            len: member.stored,
            position: 0,
        }));
    }
    let bytes = read_at(&mut file, member.offset, member.stored, limit)?;
    let expanded = layla::decompress(&bytes, usize::try_from(member.expanded).ok()?)?;
    Some(MemberReader::Expanded(Cursor::new(expanded)))
}

pub fn read_member(path: &Path, directory: &str, name: &str, limit: u64) -> Option<Vec<u8>> {
    let member = members(path)?
        .into_iter()
        .find(|member| member.is(directory, name))?;
    let mut reader = open(path, &member, limit)?;
    let mut bytes = Vec::with_capacity(usize::try_from(member.expanded).ok()?);
    reader.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}
