use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::bytes::{be_u16, be_u32, be_u64, nul_terminated, slice};
use crate::error::{FileOp, FormatError, Result, UsmFault};

pub const CRID: [u8; 4] = *b"CRID";
pub const VIDEO: [u8; 4] = *b"@SFV";
pub const AUDIO: [u8; 4] = *b"@SFA";
pub const UTF: [u8; 4] = *b"@UTF";

const CHUNK_HEADER_SIZE: u64 = 16;
const MIN_PAYLOAD_OFFSET: u8 = 8;
const MAX_HEADER_PAYLOAD: u64 = 16 * 1024 * 1024;
const UTF_HEADER_SIZE: usize = 24;
const MAX_ROWS: u32 = 1 << 20;
const HAS_NAME: u8 = 0x10;
const HAS_DEFAULT: u8 = 0x20;
const PER_ROW: u8 = 0x40;
const VANILLA_SIZES: [(u32, u32); 2] = [(1600, 904), (1920, 1080)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PayloadKind {
    Stream,
    Header,
    SectionEnd,
    Metadata,
    Other(u8),
}

impl PayloadKind {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => PayloadKind::Stream,
            1 => PayloadKind::Header,
            2 => PayloadKind::SectionEnd,
            3 => PayloadKind::Metadata,
            other => PayloadKind::Other(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkHeader {
    pub signature: [u8; 4],
    pub offset: u64,
    pub size: u32,
    pub payload_offset: u8,
    pub padding: u16,
    pub channel: u8,
    pub kind: PayloadKind,
}

impl ChunkHeader {
    pub fn end(&self) -> u64 {
        self.offset + 8 + u64::from(self.size)
    }

    pub fn payload_range(&self) -> Option<(u64, u64)> {
        if self.payload_offset < MIN_PAYLOAD_OFFSET || u32::from(self.payload_offset) > self.size {
            return None;
        }
        Some((self.offset + 8 + u64::from(self.payload_offset), self.end()))
    }
}

pub struct ChunkReader<R> {
    reader: R,
    origin: PathBuf,
    length: u64,
    position: u64,
}

impl<R: Read + Seek> ChunkReader<R> {
    pub fn new(reader: R) -> Result<Self> {
        Self::with_origin(reader, PathBuf::new())
    }

    fn with_origin(mut reader: R, origin: PathBuf) -> Result<Self> {
        let length = reader
            .seek(SeekFrom::End(0))
            .map_err(|error| FormatError::io(FileOp::Read, &origin, error))?;
        Ok(Self {
            reader,
            origin,
            length,
            position: 0,
        })
    }

    pub fn leading_signature(&mut self) -> Result<Option<[u8; 4]>> {
        if self.length < 4 {
            return Ok(None);
        }
        let mut signature = [0u8; 4];
        self.read_at(0, &mut signature)?;
        Ok(Some(signature))
    }

    pub fn next_chunk(&mut self) -> Result<Option<ChunkHeader>> {
        if self.position >= self.length {
            return Ok(None);
        }
        if self.length - self.position < CHUNK_HEADER_SIZE {
            return Err(UsmFault::Truncated.into());
        }
        let mut raw = [0u8; CHUNK_HEADER_SIZE as usize];
        self.read_at(self.position, &mut raw)?;
        let size = be_u32(&raw, 4).ok_or(UsmFault::Truncated)?;
        if size < 8 {
            return Err(UsmFault::InvalidChunk.into());
        }
        let header = ChunkHeader {
            signature: [raw[0], raw[1], raw[2], raw[3]],
            offset: self.position,
            size,
            payload_offset: raw[9],
            padding: be_u16(&raw, 10).ok_or(UsmFault::Truncated)?,
            channel: raw[12],
            kind: PayloadKind::from_code(raw[15]),
        };
        if header.end() > self.length {
            return Err(UsmFault::Truncated.into());
        }
        self.position = header.end();
        Ok(Some(header))
    }

    pub fn read_payload(&mut self, chunk: &ChunkHeader) -> Result<Vec<u8>> {
        let (start, end) = chunk.payload_range().ok_or(UsmFault::InvalidChunk)?;
        if end - start > MAX_HEADER_PAYLOAD {
            return Err(UsmFault::InvalidChunk.into());
        }
        if end > self.length {
            return Err(UsmFault::Truncated.into());
        }
        let mut payload = vec![0u8; (end - start) as usize];
        self.read_at(start, &mut payload)?;
        Ok(payload)
    }

    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        self.reader
            .seek(SeekFrom::Start(offset))
            .and_then(|_| self.reader.read_exact(buffer))
            .map_err(|error| FormatError::io(FileOp::Read, &self.origin, error))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
    Data,
}

impl ColumnType {
    pub fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            0x0 => ColumnType::U8,
            0x1 => ColumnType::I8,
            0x2 => ColumnType::U16,
            0x3 => ColumnType::I16,
            0x4 => ColumnType::U32,
            0x5 => ColumnType::I32,
            0x6 => ColumnType::U64,
            0x7 => ColumnType::I64,
            0x8 => ColumnType::F32,
            0x9 => ColumnType::F64,
            0xA => ColumnType::String,
            0xB => ColumnType::Data,
            _ => return None,
        })
    }

    pub fn width(self) -> usize {
        match self {
            ColumnType::U8 | ColumnType::I8 => 1,
            ColumnType::U16 | ColumnType::I16 => 2,
            ColumnType::U32 | ColumnType::I32 | ColumnType::F32 | ColumnType::String => 4,
            ColumnType::U64 | ColumnType::I64 | ColumnType::F64 | ColumnType::Data => 8,
        }
    }

    pub fn zero(self) -> UtfValue {
        match self {
            ColumnType::U8 => UtfValue::U8(0),
            ColumnType::I8 => UtfValue::I8(0),
            ColumnType::U16 => UtfValue::U16(0),
            ColumnType::I16 => UtfValue::I16(0),
            ColumnType::U32 => UtfValue::U32(0),
            ColumnType::I32 => UtfValue::I32(0),
            ColumnType::U64 => UtfValue::U64(0),
            ColumnType::I64 => UtfValue::I64(0),
            ColumnType::F32 => UtfValue::F32(0.0),
            ColumnType::F64 => UtfValue::F64(0.0),
            ColumnType::String => UtfValue::String(String::new()),
            ColumnType::Data => UtfValue::Data(Vec::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UtfValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Data(Vec<u8>),
}

impl UtfValue {
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            UtfValue::U8(value) => Some(u64::from(*value)),
            UtfValue::U16(value) => Some(u64::from(*value)),
            UtfValue::U32(value) => Some(u64::from(*value)),
            UtfValue::U64(value) => Some(*value),
            UtfValue::I8(value) => u64::try_from(*value).ok(),
            UtfValue::I16(value) => u64::try_from(*value).ok(),
            UtfValue::I32(value) => u64::try_from(*value).ok(),
            UtfValue::I64(value) => u64::try_from(*value).ok(),
            UtfValue::F32(_) | UtfValue::F64(_) | UtfValue::String(_) | UtfValue::Data(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            UtfValue::String(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnStorage {
    Zero,
    Constant(UtfValue),
    PerRow,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UtfColumn {
    pub name: String,
    pub kind: ColumnType,
    pub storage: ColumnStorage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UtfTable {
    pub name: String,
    pub columns: Vec<UtfColumn>,
    pub rows: Vec<Vec<UtfValue>>,
}

struct TableAreas<'a> {
    table: &'a [u8],
    strings: &'a [u8],
    data: &'a [u8],
}

impl TableAreas<'_> {
    fn string(&self, offset: u32) -> String {
        match self.strings.get(offset as usize..) {
            Some(rest) => String::from_utf8_lossy(nul_terminated(rest)).into_owned(),
            None => String::new(),
        }
    }

    fn value(
        &self,
        kind: ColumnType,
        cursor: &mut usize,
    ) -> std::result::Result<UtfValue, UsmFault> {
        let at = *cursor;
        let raw = slice(self.table, at, kind.width()).ok_or(UsmFault::InvalidTable)?;
        *cursor = at + kind.width();
        let word = || be_u32(raw, 0).unwrap_or(0);
        let wide = || be_u64(raw, 0).unwrap_or(0);
        Ok(match kind {
            ColumnType::U8 => UtfValue::U8(raw[0]),
            ColumnType::I8 => UtfValue::I8(raw[0] as i8),
            ColumnType::U16 => UtfValue::U16(be_u16(raw, 0).unwrap_or(0)),
            ColumnType::I16 => UtfValue::I16(be_u16(raw, 0).unwrap_or(0) as i16),
            ColumnType::U32 => UtfValue::U32(word()),
            ColumnType::I32 => UtfValue::I32(word() as i32),
            ColumnType::U64 => UtfValue::U64(wide()),
            ColumnType::I64 => UtfValue::I64(wide() as i64),
            ColumnType::F32 => UtfValue::F32(f32::from_bits(word())),
            ColumnType::F64 => UtfValue::F64(f64::from_bits(wide())),
            ColumnType::String => UtfValue::String(self.string(word())),
            ColumnType::Data => {
                let offset = word() as usize;
                let size = be_u32(raw, 4).unwrap_or(0) as usize;
                UtfValue::Data(
                    slice(self.data, offset, size)
                        .map(<[u8]>::to_vec)
                        .unwrap_or_default(),
                )
            }
        })
    }
}

impl UtfTable {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Ok(parse_table(bytes)?)
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().rposition(|column| column.name == name)
    }

    pub fn value(&self, row: usize, column: &str) -> Option<&UtfValue> {
        let index = self.column_index(column)?;
        self.rows.get(row)?.get(index)
    }

    pub fn unsigned(&self, row: usize, column: &str) -> Option<u64> {
        self.value(row, column)?.as_u64()
    }
}

fn parse_table(bytes: &[u8]) -> std::result::Result<UtfTable, UsmFault> {
    if bytes.get(0..4) != Some(&UTF[..]) {
        return Err(UsmFault::InvalidTable);
    }
    let table_size = be_u32(bytes, 4).ok_or(UsmFault::InvalidTable)? as usize;
    let table = slice(bytes, 8, table_size).ok_or(UsmFault::InvalidTable)?;
    if table.len() < UTF_HEADER_SIZE {
        return Err(UsmFault::InvalidTable);
    }
    let field = |offset| be_u32(table, offset).ok_or(UsmFault::InvalidTable);
    let rows_offset = be_u16(table, 2).ok_or(UsmFault::InvalidTable)? as usize;
    let strings_offset = field(4)? as usize;
    let data_offset = field(8)? as usize;
    let name_offset = field(12)?;
    let column_count = be_u16(table, 16).ok_or(UsmFault::InvalidTable)?;
    let row_width = be_u16(table, 18).ok_or(UsmFault::InvalidTable)? as usize;
    let row_count = field(20)?;
    if strings_offset > data_offset
        || data_offset > table.len()
        || rows_offset > table.len()
        || row_count > MAX_ROWS
    {
        return Err(UsmFault::InvalidTable);
    }
    let areas = TableAreas {
        table,
        strings: &table[strings_offset..data_offset],
        data: &table[data_offset..],
    };

    let mut cursor = UTF_HEADER_SIZE;
    let mut columns = Vec::with_capacity(usize::from(column_count));
    for _ in 0..column_count {
        let flags = *table.get(cursor).ok_or(UsmFault::InvalidTable)?;
        cursor += 1;
        let code = flags & 0x0F;
        let kind = ColumnType::from_code(code).ok_or(UsmFault::UnknownColumnType(code))?;
        let name = if flags & HAS_NAME != 0 {
            let offset = be_u32(table, cursor).ok_or(UsmFault::InvalidTable)?;
            cursor += 4;
            areas.string(offset)
        } else {
            String::new()
        };
        let default = if flags & HAS_DEFAULT != 0 {
            Some(areas.value(kind, &mut cursor)?)
        } else {
            None
        };
        let storage = match (flags & PER_ROW != 0, default) {
            (true, _) => ColumnStorage::PerRow,
            (false, Some(value)) => ColumnStorage::Constant(value),
            (false, None) => ColumnStorage::Zero,
        };
        columns.push(UtfColumn {
            name,
            kind,
            storage,
        });
    }

    let has_row_storage = columns
        .iter()
        .any(|column| column.storage == ColumnStorage::PerRow);
    if has_row_storage {
        let rows_end = (row_count as usize)
            .checked_mul(row_width)
            .and_then(|size| size.checked_add(rows_offset))
            .ok_or(UsmFault::InvalidTable)?;
        if rows_end > table.len() {
            return Err(UsmFault::InvalidTable);
        }
    }

    let mut rows = Vec::with_capacity(row_count as usize);
    for row in 0..row_count as usize {
        let mut cursor = rows_offset + row * row_width;
        let mut values = Vec::with_capacity(columns.len());
        for column in &columns {
            values.push(match &column.storage {
                ColumnStorage::PerRow => areas.value(column.kind, &mut cursor)?,
                ColumnStorage::Constant(value) => value.clone(),
                ColumnStorage::Zero => column.kind.zero(),
            });
        }
        rows.push(values);
    }

    Ok(UtfTable {
        name: areas.string(name_offset),
        columns,
        rows,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoCodec {
    SofdecPrime,
    H264,
    Vp9,
    Other(u32),
}

impl VideoCodec {
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => VideoCodec::SofdecPrime,
            5 => VideoCodec::H264,
            9 => VideoCodec::Vp9,
            other => VideoCodec::Other(other),
        }
    }

    pub fn code(self) -> u32 {
        match self {
            VideoCodec::SofdecPrime => 1,
            VideoCodec::H264 => 5,
            VideoCodec::Vp9 => 9,
            VideoCodec::Other(code) => code,
        }
    }

    pub fn is_h264(self) -> bool {
        self == VideoCodec::H264
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub codec: VideoCodec,
}

impl VideoInfo {
    pub fn from_header_table(table: &UtfTable) -> Result<Self> {
        if table.rows.is_empty() {
            return Err(UsmFault::EmptyTable.into());
        }
        let largest = |names: [&str; 3]| {
            names
                .iter()
                .map(|name| table.unsigned(0, name).unwrap_or(0))
                .max()
                .unwrap_or(0)
                .min(u64::from(u32::MAX)) as u32
        };
        let codec = table
            .unsigned(0, "mpeg_codec")
            .unwrap_or(0)
            .min(u64::from(u32::MAX)) as u32;
        Ok(VideoInfo {
            width: largest(["width", "disp_width", "mat_width"]),
            height: largest(["height", "disp_height", "mat_height"]),
            codec: VideoCodec::from_code(codec),
        })
    }

    pub fn pixels(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    pub fn is_vanilla_shape(&self) -> bool {
        self.codec == VideoCodec::SofdecPrime && VANILLA_SIZES.contains(&(self.width, self.height))
    }
}

pub fn read_video_info<R: Read + Seek>(reader: R) -> Result<VideoInfo> {
    video_info_from(ChunkReader::new(reader)?)
}

fn video_info_from<R: Read + Seek>(mut chunks: ChunkReader<R>) -> Result<VideoInfo> {
    if chunks.leading_signature()? != Some(CRID) {
        return Err(UsmFault::NotUsm.into());
    }
    while let Some(chunk) = chunks.next_chunk()? {
        if chunk.signature == VIDEO && chunk.kind == PayloadKind::Header {
            let payload = chunks.read_payload(&chunk)?;
            return VideoInfo::from_header_table(&UtfTable::parse(&payload)?);
        }
    }
    Err(UsmFault::NoVideoHeader.into())
}

pub fn is_usm_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("usm"))
}

pub fn probe(path: &Path) -> Result<VideoInfo> {
    let file = File::open(path).map_err(|error| FormatError::io(FileOp::Read, path, error))?;
    video_info_from(ChunkReader::with_origin(
        BufReader::new(file),
        path.to_path_buf(),
    )?)
}

pub fn probe_quiet(path: &Path) -> Option<VideoInfo> {
    probe(path).ok()
}
