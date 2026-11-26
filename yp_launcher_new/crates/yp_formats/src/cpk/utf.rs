use crate::bytes::{be_u16, be_u32, be_u64, slice};

#[derive(Clone, Copy)]
enum Storage {
    Zero,
    Constant(usize),
    Row(usize),
}

#[derive(Clone, Copy)]
pub(super) struct Column {
    kind: u8,
    storage: Storage,
}

pub(super) struct Table<'a> {
    bytes: &'a [u8],
    strings: &'a [u8],
    rows: usize,
    stride: usize,
    count: usize,
    columns: Vec<(&'a str, Column)>,
}

fn string_at(pool: &[u8], offset: usize) -> Option<&str> {
    let tail = pool.get(offset..)?;
    let end = tail.iter().take(4096).position(|&b| b == 0)?;
    std::str::from_utf8(&tail[..end]).ok()
}

impl<'a> Table<'a> {
    pub(super) fn parse(packet: &'a [u8]) -> Option<Self> {
        if packet.get(..4)? != b"@UTF" {
            return None;
        }
        let bytes = slice(packet, 8, be_u32(packet, 4)? as usize)?;
        let rows = usize::from(be_u16(bytes, 2)?);
        let strings_start = be_u32(bytes, 4)? as usize;
        let strings = bytes.get(strings_start..be_u32(bytes, 8)? as usize)?;
        let fields = usize::from(be_u16(bytes, 16)?);
        let stride = usize::from(be_u16(bytes, 18)?);
        let count = be_u32(bytes, 20)? as usize;
        if fields > 128
            || count > 200_000
            || rows.checked_add(count.checked_mul(stride)?)? > strings_start
        {
            return None;
        }
        let schema = bytes.get(..rows)?;
        let mut pos = 24;
        let mut row_pos = 0;
        let mut columns = Vec::with_capacity(fields);
        for _ in 0..fields {
            let flags = *schema.get(pos)?;
            let name = string_at(strings, be_u32(schema, pos + 1)? as usize)?;
            pos += 5;
            let kind = flags & 15;
            let width = match kind {
                0 | 1 => 1,
                2 | 3 => 2,
                4 | 5 | 8 | 10 => 4,
                6 | 7 | 9 | 11 => 8,
                12 => 16,
                _ => return None,
            };
            let storage = match flags >> 4 {
                1 => Storage::Zero,
                3 => {
                    slice(schema, pos, width)?;
                    pos += width;
                    Storage::Constant(pos - width)
                }
                5 => {
                    row_pos += width;
                    Storage::Row(row_pos - width)
                }
                _ => return None,
            };
            columns.push((name, Column { kind, storage }));
        }
        if row_pos > stride {
            return None;
        }
        Some(Self {
            bytes,
            strings,
            rows,
            stride,
            count,
            columns,
        })
    }

    pub(super) fn len(&self) -> usize {
        self.count
    }

    pub(super) fn column(&self, name: &str) -> Option<Column> {
        self.columns
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, column)| *column)
    }

    pub(super) fn number(&self, field: Column, row: usize) -> Option<u64> {
        if row >= self.count {
            return None;
        }
        let pos = match field.storage {
            Storage::Zero => return Some(0),
            Storage::Constant(pos) => pos,
            Storage::Row(pos) => self.rows + row * self.stride + pos,
        };
        match field.kind {
            0 | 1 => Some(u64::from(*self.bytes.get(pos)?)),
            2 | 3 => be_u16(self.bytes, pos).map(u64::from),
            4 | 5 | 10 => be_u32(self.bytes, pos).map(u64::from),
            6 | 7 => be_u64(self.bytes, pos),
            _ => None,
        }
    }

    pub(super) fn text(&self, field: Column, row: usize) -> Option<&str> {
        if field.kind != 10 {
            return None;
        }
        string_at(
            self.strings,
            usize::try_from(self.number(field, row)?).ok()?,
        )
    }
}
