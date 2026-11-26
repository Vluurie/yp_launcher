use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::{GrayImage, ImageFormat, load_from_memory_with_format};

use crate::bytes::{le_f32, le_i16, le_u16, le_u32};
use crate::cpk::{self, Member, MemberReader};
use crate::dat::{ArchiveEntry, DatArchive, crc32};
use crate::error::{FileOp, FormatError, Result};
use crate::texture::decode_dds_rgba;

pub const FACE: u16 = 0x8000;
const FACES: [(&str, u16); 4] = [
    ("font_01", 1),
    ("font_04", 4),
    ("font_05", 5),
    ("font_11", 11),
];
const MESSAGE_LANGUAGES: [&str; 5] = ["us", "fr", "de", "es", "it"];
const MEMBER_LIMIT: u64 = 512 * 1024 * 1024;
const MAX_ITEMS: usize = 1 << 20;
const FTB_COUNT_AT: usize = 0x7a;
const FTB_TABLE_AT: usize = 0x80;
const FTB_ENTRY: usize = 12;
const KTB_ENTRY: usize = 6;
const FACE_PADDING: u32 = 1;
const SYMBOL: usize = 8;
const GLYPH: usize = 40;
const FONT: usize = 20;
const MESSAGE: usize = 16;
const PARAGRAPH: usize = 20;
const LINE: usize = 24;
const LETTER: usize = 4;
const TERMINATOR: u16 = 0x8000;
const ATLAS_WIDTH: u32 = 2048;
const GAP: u32 = 1;
const CACHE_MAGIC: [u8; 4] = *b"YPGF";
const CACHE_VERSION: u32 = 1;
const CACHE_HEADER: usize = 20;
const GLYPH_RECORD: usize = 16;
const KERN_RECORD: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    pub font: u16,
    pub code: u16,
    pub rect: [u16; 4],
    pub advance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameFont {
    pub stamp: u32,
    pub width: u32,
    pub height: u32,
    pub coverage: Vec<u8>,
    pub glyphs: Vec<Glyph>,
    pub kerning: Vec<([u16; 3], i16)>,
}

pub fn in_common_ranges(ch: char) -> bool {
    matches!(
        u32::from(ch),
        0x20..=0x7e | 0xa0..=0x2fff | 0x3000..=0x30ff | 0xff00..=0xffef
    )
}

pub fn source_stamp(game_root: &Path) -> Result<u32> {
    let mut listing = Vec::new();
    for path in cpk_files(&game_root.join("data"))? {
        let size = fs::metadata(&path)
            .map_err(|error| FormatError::io(FileOp::Read, &path, error))?
            .len();
        listing.extend(path.file_name().unwrap_or_default().as_encoded_bytes());
        listing.push(0);
        listing.extend(size.to_le_bytes());
    }
    Ok(crc32(&listing))
}

pub fn build(game_root: &Path, wanted: impl Fn(char) -> bool) -> Result<GameFont> {
    let stamp = source_stamp(game_root)?;
    let archives = Archives::scan(&game_root.join("data"))?;
    let mut sheet = Sheet::default();
    for name in archives.message_archives() {
        sheet.add_messages(&archives, &name)?;
    }
    for (stem, id) in FACES {
        sheet.add_face(&archives, stem, id, &wanted)?;
    }
    sheet.pack(stamp)
}

fn cpk_files(data: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(data)
        .map_err(|error| FormatError::io(FileOp::Read, data, error))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("cpk"))
        })
        .collect();
    files.sort();
    Ok(files)
}

fn missing(name: &str) -> FormatError {
    FormatError::GameFileMissing {
        name: name.to_owned(),
    }
}

fn damaged(name: &str, detail: &'static str) -> FormatError {
    FormatError::GameFileDamaged {
        name: name.to_owned(),
        detail,
    }
}

struct Archives {
    members: Vec<(PathBuf, Member)>,
}

impl Archives {
    fn scan(data: &Path) -> Result<Self> {
        let members = cpk_files(data)?
            .into_iter()
            .filter_map(|path| Some((cpk::members(&path)?, path)))
            .flat_map(|(members, path)| members.into_iter().map(move |m| (path.clone(), m)))
            .collect();
        Ok(Self { members })
    }

    fn open(&self, directory: &str, name: &str) -> Result<DatArchive<MemberReader>> {
        let (path, member) = self
            .members
            .iter()
            .find(|(_, member)| member.is(directory, name))
            .ok_or_else(|| missing(name))?;
        let reader =
            cpk::open(path, member, MEMBER_LIMIT).ok_or_else(|| damaged(name, "archive member"))?;
        DatArchive::from_reader(reader)
    }

    fn message_archives(&self) -> Vec<String> {
        MESSAGE_LANGUAGES
            .iter()
            .flat_map(|language| {
                let suffix = format!("_{language}.dat");
                let mut names: Vec<String> = self
                    .members
                    .iter()
                    .map(|(_, member)| member)
                    .filter(|member| {
                        member.directory.eq_ignore_ascii_case("ui")
                            && member.name.to_ascii_lowercase().ends_with(&suffix)
                    })
                    .map(|member| member.name.clone())
                    .collect();
                names.sort();
                names.dedup();
                names
            })
            .collect()
    }
}

struct TextureTable {
    entries: Vec<(u32, u64, u64)>,
}

impl TextureTable {
    fn parse(wta: &[u8]) -> Option<Self> {
        let count = le_u32(wta, 8)? as usize;
        let offsets = le_u32(wta, 12)? as usize;
        let sizes = le_u32(wta, 16)? as usize;
        let ids = le_u32(wta, 24)? as usize;
        if count > MAX_ITEMS {
            return None;
        }
        let entries = (0..count)
            .map(|i| {
                Some((
                    le_u32(wta, ids + i * 4)?,
                    u64::from(le_u32(wta, offsets + i * 4)?),
                    u64::from(le_u32(wta, sizes + i * 4)?),
                ))
            })
            .collect::<Option<_>>()?;
        Some(Self { entries })
    }

    fn by_index(&self, index: usize) -> Option<(u64, u64)> {
        self.entries
            .get(index)
            .map(|&(_, offset, size)| (offset, size))
    }

    fn by_id(&self, id: u32) -> Option<(u64, u64)> {
        self.entries
            .iter()
            .find(|(own, _, _)| *own == id)
            .map(|&(_, offset, size)| (offset, size))
    }
}

struct Page {
    width: u32,
    height: u32,
    alpha: Vec<u8>,
}

impl Page {
    fn read(
        archive: &mut DatArchive<MemberReader>,
        wtp: &ArchiveEntry,
        (offset, size): (u64, u64),
    ) -> Result<Self> {
        let texture = decode_dds_rgba(&archive.read_entry_range(wtp, offset, size)?)?;
        Ok(Self {
            width: texture.width,
            height: texture.height,
            alpha: texture
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .map(|pixel| pixel[3])
                .collect(),
        })
    }

    fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Option<Vec<u8>> {
        if x >= self.width || y >= self.height || width == 0 || height == 0 {
            return None;
        }
        let span = width.min(self.width - x) as usize;
        let mut out = vec![0; width as usize * height as usize];
        for row in 0..height.min(self.height - y) {
            let from = ((y + row) * self.width + x) as usize;
            let to = (row * width) as usize;
            out[to..to + span].copy_from_slice(&self.alpha[from..from + span]);
        }
        Some(out)
    }
}

struct Cell {
    width: u32,
    height: u32,
    coverage: Vec<u8>,
    advance: f32,
}

struct Symbol {
    font: u16,
    code: u16,
    glyph: usize,
}

struct MessageGlyph {
    texture: u32,
    origin: [f32; 2],
    size: [f32; 2],
}

struct Mcd {
    symbols: Vec<Symbol>,
    glyphs: Vec<MessageGlyph>,
    spacing: Vec<(u16, f32)>,
    kerning: Vec<([u16; 3], i16)>,
}

fn table<T>(count: usize, item: impl Fn(usize) -> Option<T>) -> Option<Vec<T>> {
    if count > MAX_ITEMS {
        return None;
    }
    (0..count).map(item).collect()
}

fn letters(bytes: &[u8], line: usize) -> Option<Vec<(u16, i16)>> {
    let start = le_u32(bytes, line)? as usize;
    let words = le_u32(bytes, line + 8)? as usize;
    table(words.saturating_sub(1) / 2, |k| {
        let at = start + k * LETTER;
        Some((le_u16(bytes, at)?, le_i16(bytes, at + 2)?))
    })
}

impl Mcd {
    fn parse(bytes: &[u8]) -> Option<Self> {
        let field = |i: usize| le_u32(bytes, i * 4).map(|value| value as usize);
        let symbols = table(field(3)?, |i| {
            let at = field(2)? + i * SYMBOL;
            Some(Symbol {
                font: le_u16(bytes, at)?,
                code: le_u16(bytes, at + 2)?,
                glyph: le_u32(bytes, at + 4)? as usize,
            })
        })?;
        let glyphs = table(field(5)?, |i| {
            let at = field(4)? + i * GLYPH;
            Some(MessageGlyph {
                texture: le_u32(bytes, at)?,
                origin: [le_f32(bytes, at + 4)?, le_f32(bytes, at + 8)?],
                size: [le_f32(bytes, at + 20)?, le_f32(bytes, at + 24)?],
            })
        })?;
        let spacing = table(field(7)?, |i| {
            let at = field(6)? + i * FONT;
            Some((
                u16::try_from(le_u32(bytes, at)?).ok()?,
                le_f32(bytes, at + 12)?,
            ))
        })?;
        let mut mcd = Self {
            symbols,
            glyphs,
            spacing,
            kerning: Vec::new(),
        };
        mcd.kerning = mcd.message_kerning(bytes, field(0)?, field(1)?)?;
        Some(mcd)
    }

    fn character(&self, code: u16) -> Option<u16> {
        self.symbols
            .get(usize::from(code))
            .map(|symbol| symbol.code)
    }

    fn message_kerning(
        &self,
        bytes: &[u8],
        start: usize,
        count: usize,
    ) -> Option<Vec<([u16; 3], i16)>> {
        let mut pairs = Vec::new();
        let mut previous: Option<u16> = None;
        for message in table(count, |m| Some(start + m * MESSAGE))? {
            let paragraphs = le_u32(bytes, message)? as usize;
            let mut last = None;
            for p in 0..le_u32(bytes, message + 4)? as usize {
                let paragraph = paragraphs + p * PARAGRAPH;
                let lines = le_u32(bytes, paragraph)? as usize;
                let font = u16::try_from(le_u32(bytes, paragraph + 16)?).ok()?;
                for l in 0..le_u32(bytes, paragraph + 4)? as usize {
                    let letters = letters(bytes, lines + l * LINE)?;
                    if p == 0
                        && l == 0
                        && let (Some(prev), Some(&(first, kerning))) = (previous, letters.first())
                        && first < TERMINATOR
                        && let Some(ch) = self.character(first)
                    {
                        pairs.push(([font, prev, ch], kerning));
                    }
                    for pair in letters.windows(2) {
                        let [(left, _), (right, kerning)] = [pair[0], pair[1]];
                        if left >= TERMINATOR || right >= TERMINATOR {
                            continue;
                        }
                        pairs.push((
                            [font, self.character(left)?, self.character(right)?],
                            kerning,
                        ));
                    }
                    last = letters
                        .last()
                        .filter(|(code, _)| *code < TERMINATOR)
                        .and_then(|(code, _)| self.character(*code));
                }
            }
            previous = last;
        }
        Some(pairs)
    }
}

fn parse_ftb(ftb: &[u8]) -> Option<Vec<[u16; 6]>> {
    let count = usize::from(le_u16(ftb, FTB_COUNT_AT)?);
    let start = le_u32(ftb, FTB_TABLE_AT)? as usize;
    table(count, |i| {
        let at = start + i * FTB_ENTRY;
        let word = |n: usize| le_u16(ftb, at + n * 2);
        Some([word(0)?, word(1)?, word(2)?, word(3)?, word(4)?, word(5)?])
    })
}

fn parse_ktb(ktb: &[u8]) -> Vec<(u16, u16, i16)> {
    let count = le_u16(ktb, 0).map_or(0, usize::from);
    (0..count)
        .map_while(|i| {
            let at = 2 + i * KTB_ENTRY;
            Some((le_u16(ktb, at)?, le_u16(ktb, at + 2)?, le_i16(ktb, at + 4)?))
        })
        .collect()
}

#[derive(Default)]
struct Sheet {
    cells: BTreeMap<(u16, u16), Cell>,
    kerning: BTreeMap<[u16; 3], i16>,
    spacing: HashMap<u16, f32>,
}

impl Sheet {
    fn add_messages(&mut self, archives: &Archives, name: &str) -> Result<()> {
        let mut dat = archives.open("ui", name)?;
        let sources: Vec<ArchiveEntry> = dat
            .entries()
            .iter()
            .filter(|entry| entry.has_extension("mcd"))
            .cloned()
            .collect();
        let mut dtt = None;
        for entry in sources {
            let mcd = Mcd::parse(&dat.read_entry(&entry)?)
                .ok_or_else(|| damaged(&entry.name, "message table"))?;
            for &(font, spacing) in &mcd.spacing {
                self.spacing.entry(font).or_insert(spacing);
            }
            for &(key, value) in &mcd.kerning {
                self.kerning.entry(key).or_insert(value);
            }
            let fresh: Vec<&Symbol> = mcd
                .symbols
                .iter()
                .filter(|symbol| {
                    char::from_u32(u32::from(symbol.code)).is_some()
                        && !self.cells.contains_key(&(symbol.font, symbol.code))
                })
                .collect();
            if fresh.is_empty() {
                continue;
            }
            let stem = entry.stem();
            let wta_name = format!("{stem}.wta");
            let wta = dat
                .read_named(&wta_name)?
                .ok_or_else(|| missing(&wta_name))?;
            let textures =
                TextureTable::parse(&wta).ok_or_else(|| damaged(&wta_name, "texture table"))?;
            let dtt = match &mut dtt {
                Some(dtt) => dtt,
                None => {
                    let dtt_name = format!("{}.dtt", name.rsplit_once('.').map_or(name, |n| n.0));
                    dtt.insert(archives.open("ui", &dtt_name)?)
                }
            };
            let wtp_name = format!("{stem}.wtp");
            let wtp = dtt
                .find(&wtp_name)
                .cloned()
                .ok_or_else(|| missing(&wtp_name))?;
            let mut pages: HashMap<u32, Page> = HashMap::new();
            for symbol in fresh {
                let Some(glyph) = mcd.glyphs.get(symbol.glyph) else {
                    continue;
                };
                let page = match pages.entry(glyph.texture) {
                    Entry::Occupied(slot) => slot.into_mut(),
                    Entry::Vacant(slot) => {
                        let Some(location) = textures.by_id(glyph.texture) else {
                            continue;
                        };
                        slot.insert(Page::read(dtt, &wtp, location)?)
                    }
                };
                let [x, y] = [
                    glyph.origin[0] * page.width as f32,
                    glyph.origin[1] * page.height as f32,
                ]
                .map(|v| v.round().max(0.) as u32);
                let [width, height] = glyph.size.map(|v| v.round().max(0.) as u32);
                let Some(coverage) = page.crop(x, y, width, height) else {
                    continue;
                };
                let spacing = mcd
                    .spacing
                    .iter()
                    .find(|(font, _)| *font == symbol.font)
                    .map(|(_, spacing)| *spacing)
                    .or_else(|| self.spacing.get(&symbol.font).copied())
                    .unwrap_or_default();
                self.cells.insert(
                    (symbol.font, symbol.code),
                    Cell {
                        width,
                        height,
                        coverage,
                        advance: width as f32 + spacing,
                    },
                );
            }
        }
        Ok(())
    }

    fn add_face(
        &mut self,
        archives: &Archives,
        stem: &str,
        id: u16,
        wanted: &impl Fn(char) -> bool,
    ) -> Result<()> {
        let face = FACE | id;
        let mut dat = archives.open("font", &format!("{stem}.dat"))?;
        let [ftb, wta] = ["ftb", "wta"].map(|extension| format!("{stem}.{extension}"));
        let entries = parse_ftb(&dat.read_named(&ftb)?.ok_or_else(|| missing(&ftb))?)
            .ok_or_else(|| damaged(&ftb, "glyph table"))?;
        let textures = TextureTable::parse(&dat.read_named(&wta)?.ok_or_else(|| missing(&wta))?)
            .ok_or_else(|| damaged(&wta, "texture table"))?;
        let mut pages: BTreeMap<u16, Vec<[u16; 6]>> = BTreeMap::new();
        for entry in entries {
            let ch = char::from_u32(u32::from(entry[0]));
            if ch.is_some_and(wanted) && !self.cells.contains_key(&(face, entry[0])) {
                pages.entry(entry[1]).or_default().push(entry);
            }
        }
        let spacing = self.spacing.get(&id).copied().unwrap_or_default();
        let mut dtt = archives.open("font", &format!("{stem}.dtt"))?;
        let wtp_name = format!("{stem}.wtp");
        let wtp = dtt
            .find(&wtp_name)
            .cloned()
            .ok_or_else(|| missing(&wtp_name))?;
        for (texture, glyphs) in pages {
            let location = textures
                .by_index(usize::from(texture))
                .ok_or_else(|| damaged(&wta, "glyph page"))?;
            let page = Page::read(&mut dtt, &wtp, location)?;
            for [code, _, width, height, u, v] in glyphs {
                let width = u32::from(width) + FACE_PADDING;
                let height = u32::from(height) + FACE_PADDING;
                let Some(coverage) = page.crop(u32::from(u), u32::from(v), width, height) else {
                    continue;
                };
                self.cells.insert(
                    (face, code),
                    Cell {
                        width,
                        height,
                        coverage,
                        advance: width as f32 + spacing,
                    },
                );
            }
        }
        if let Some(ktb) = dat.read_named(&format!("{stem}.ktb"))? {
            for (left, right, value) in parse_ktb(&ktb) {
                let usable = |code: u16| char::from_u32(u32::from(code)).is_some_and(wanted);
                if usable(left) && usable(right) {
                    self.kerning.entry([face, left, right]).or_insert(value);
                }
            }
        }
        Ok(())
    }

    fn pack(self, stamp: u32) -> Result<GameFont> {
        let mut order: Vec<(&(u16, u16), &Cell)> = self.cells.iter().collect();
        order.sort_by_key(|(key, cell)| (Reverse(cell.height), **key));
        let (mut x, mut y, mut row) = (0, 0, 0);
        let mut placed = Vec::with_capacity(order.len());
        for (key, cell) in order {
            if cell.width > ATLAS_WIDTH {
                continue;
            }
            if x + cell.width > ATLAS_WIDTH {
                (x, y, row) = (0, y + row + GAP, 0);
            }
            placed.push((*key, cell, x, y));
            x += cell.width + GAP;
            row = row.max(cell.height);
        }
        let height = y + row;
        let fits = |value: u32| u16::try_from(value).ok();
        if fits(height).is_none() {
            return Err(damaged("font atlas", "too many glyphs"));
        }
        let mut coverage = vec![0; (ATLAS_WIDTH * height) as usize];
        let mut glyphs = Vec::with_capacity(placed.len());
        for ((font, code), cell, x, y) in placed {
            for row in 0..cell.height {
                let from = (row * cell.width) as usize;
                let to = ((y + row) * ATLAS_WIDTH + x) as usize;
                let span = cell.width as usize;
                coverage[to..to + span].copy_from_slice(&cell.coverage[from..from + span]);
            }
            let rect = [x, y, cell.width, cell.height].map(|v| fits(v).unwrap_or_default());
            glyphs.push(Glyph {
                font,
                code,
                rect,
                advance: cell.advance,
            });
        }
        glyphs.sort_by_key(|glyph| (glyph.font, glyph.code));
        Ok(GameFont {
            stamp,
            width: ATLAS_WIDTH,
            height,
            coverage,
            glyphs,
            kerning: self.kerning.into_iter().collect(),
        })
    }
}

impl GameFont {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(
            CACHE_HEADER + self.glyphs.len() * GLYPH_RECORD + self.kerning.len() * KERN_RECORD,
        );
        out.extend(CACHE_MAGIC);
        for value in [
            CACHE_VERSION,
            self.stamp,
            u32::try_from(self.glyphs.len()).map_err(|_| FormatError::FontCacheDamaged)?,
            u32::try_from(self.kerning.len()).map_err(|_| FormatError::FontCacheDamaged)?,
        ] {
            out.extend(value.to_le_bytes());
        }
        for glyph in &self.glyphs {
            for value in [glyph.font, glyph.code].iter().chain(&glyph.rect) {
                out.extend(value.to_le_bytes());
            }
            out.extend(glyph.advance.to_le_bytes());
        }
        for (key, value) in &self.kerning {
            for part in key {
                out.extend(part.to_le_bytes());
            }
            out.extend(value.to_le_bytes());
        }
        let image = GrayImage::from_raw(self.width, self.height, self.coverage.clone())
            .ok_or(FormatError::FontCacheDamaged)?;
        let mut png = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
            .map_err(|error| FormatError::TextureEncodeFailed {
                detail: error.to_string(),
            })?;
        out.extend(png);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::parse(bytes).ok_or(FormatError::FontCacheDamaged)
    }

    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.get(..4)? != CACHE_MAGIC || le_u32(bytes, 4)? != CACHE_VERSION {
            return None;
        }
        let stamp = le_u32(bytes, 8)?;
        let glyph_count = le_u32(bytes, 12)? as usize;
        let kern_count = le_u32(bytes, 16)? as usize;
        let glyphs = table(glyph_count, |i| {
            let at = CACHE_HEADER + i * GLYPH_RECORD;
            let word = |n: usize| le_u16(bytes, at + n * 2);
            Some(Glyph {
                font: word(0)?,
                code: word(1)?,
                rect: [word(2)?, word(3)?, word(4)?, word(5)?],
                advance: le_f32(bytes, at + 12)?,
            })
        })?;
        let kerning_at = CACHE_HEADER + glyph_count * GLYPH_RECORD;
        let kerning = table(kern_count, |i| {
            let at = kerning_at + i * KERN_RECORD;
            let word = |n: usize| le_u16(bytes, at + n * 2);
            Some(([word(0)?, word(1)?, word(2)?], le_i16(bytes, at + 6)?))
        })?;
        let png = bytes.get(kerning_at + kern_count * KERN_RECORD..)?;
        let image = load_from_memory_with_format(png, ImageFormat::Png)
            .ok()?
            .into_luma8();
        let (width, height) = image.dimensions();
        let inside = |glyph: &Glyph| {
            let [x, y, w, h] = glyph.rect.map(u32::from);
            x + w <= width && y + h <= height
        };
        if !glyphs.iter().all(inside) {
            return None;
        }
        Some(Self {
            stamp,
            width,
            height,
            coverage: image.into_raw(),
            glyphs,
            kerning,
        })
    }
}

#[cfg(test)]
#[path = "font_tests.rs"]
mod tests;
