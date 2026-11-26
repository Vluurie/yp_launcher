use crate::bytes::{le_u32, slice};

struct Bits<'a> {
    unread: &'a [u8],
    pool: u32,
    available: u32,
}

impl Bits<'_> {
    fn read(&mut self, n: u32) -> Option<usize> {
        while self.available < n {
            let (&byte, rest) = self.unread.split_last()?;
            self.unread = rest;
            self.pool = (self.pool << 8) | u32::from(byte);
            self.available += 8;
        }
        self.available -= n;
        Some(((self.pool >> self.available) & ((1 << n) - 1)) as usize)
    }

    fn copy_len(&mut self) -> Option<usize> {
        let mut length = 3;
        for width in [2, 3, 5, 8].into_iter().chain(std::iter::repeat(8)) {
            let value = self.read(width)?;
            length = usize::checked_add(length, value)?;
            if value < (1 << width) - 1 {
                return Some(length);
            }
        }
        None
    }
}

pub(super) fn decompress(bytes: &[u8], expected: usize) -> Option<Vec<u8>> {
    let body = le_u32(bytes, 8)? as usize;
    let packed = le_u32(bytes, 12)? as usize;
    if bytes.get(..8)? != b"CRILAYLA" || body.checked_add(256)? != expected {
        return None;
    }
    let mut bits = Bits {
        unread: slice(bytes, 16, packed)?,
        pool: 0,
        available: 0,
    };
    let prefix = slice(bytes, 16usize.checked_add(packed)?, 256)?;
    let mut output = vec![0; expected];
    output[..256].copy_from_slice(prefix);
    let mut end = output.len();
    while end > 256 {
        if bits.read(1)? == 0 {
            end -= 1;
            output[end] = bits.read(8)? as u8;
        } else {
            let distance = 3 + bits.read(13)?;
            let count = bits.copy_len()?;
            if distance > output.len() - end || count > end - 256 {
                return None;
            }
            for at in (end - count..end).rev() {
                output[at] = output[at + distance];
            }
            end -= count;
        }
    }
    Some(output)
}
