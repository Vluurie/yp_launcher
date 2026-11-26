pub(crate) fn slice(bytes: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    bytes.get(offset..offset.checked_add(len)?)
}

pub(crate) fn array<const N: usize>(bytes: &[u8], offset: usize) -> Option<[u8; N]> {
    slice(bytes, offset, N)?.try_into().ok()
}

pub(crate) fn le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    array(bytes, offset).map(u16::from_le_bytes)
}

pub(crate) fn le_i16(bytes: &[u8], offset: usize) -> Option<i16> {
    array(bytes, offset).map(i16::from_le_bytes)
}

pub(crate) fn le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    array(bytes, offset).map(u32::from_le_bytes)
}

pub(crate) fn le_f32(bytes: &[u8], offset: usize) -> Option<f32> {
    array(bytes, offset).map(f32::from_le_bytes)
}

pub(crate) fn be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    array(bytes, offset).map(u16::from_be_bytes)
}

pub(crate) fn be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    array(bytes, offset).map(u32::from_be_bytes)
}

pub(crate) fn be_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    array(bytes, offset).map(u64::from_be_bytes)
}

pub(crate) fn put_u16(buffer: &mut [u8], offset: usize, value: u16) {
    buffer[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn put_f32(buffer: &mut [u8], offset: usize, value: f32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn put_fixed_str(buffer: &mut [u8], offset: usize, value: &str, width: usize) {
    let bytes = value.as_bytes();
    let copy = bytes.len().min(width);
    buffer[offset..offset + copy].copy_from_slice(&bytes[..copy]);
}

pub(crate) fn nul_terminated(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_are_bounds_checked() {
        let bytes = [1, 0, 0, 0, 2];
        assert_eq!(le_u32(&bytes, 0), Some(1));
        assert_eq!(le_u32(&bytes, 2), None);
        assert_eq!(le_u32(&bytes, usize::MAX), None);
        assert_eq!(be_u16(&bytes, 3), Some(2));
        assert_eq!(nul_terminated(b"ab\0cd"), b"ab");
        assert_eq!(nul_terminated(b"ab"), b"ab");
    }
}
