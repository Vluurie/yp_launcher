use image::imageops::FilterType;
use texpresso::Format;

use crate::bytes::{le_u32, put_f32, put_fixed_str, put_u32};
use crate::dat::{self, DatEntry, DatKind, name_crc};
use crate::error::{FormatError, Result};
use crate::texture::{DDS_HEADER_SIZE, MAX_MISCTEX_SIDE, MIN_MISCTEX_SIDE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiscTex {
    pub dat: Vec<u8>,
    pub dtt: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn misctex_stem(texture_name: &str) -> String {
    format!("misctex_{texture_name}")
}

pub fn validate_size(width: u32, height: u32) -> Result<()> {
    let range = MIN_MISCTEX_SIDE..=MAX_MISCTEX_SIDE;
    if !range.contains(&width) || !range.contains(&height) {
        return Err(FormatError::ImageSizeOutOfRange { width, height });
    }
    if !width.is_multiple_of(4) || !height.is_multiple_of(4) {
        return Err(FormatError::ImageSizeNotMultipleOfFour { width, height });
    }
    Ok(())
}

pub fn create_misctex(
    image_bytes: &[u8],
    texture_name: &str,
    width: u32,
    height: u32,
) -> Result<MiscTex> {
    validate_size(width, height)?;

    let source =
        image::load_from_memory(image_bytes).map_err(|error| FormatError::ImageUnreadable {
            detail: error.to_string(),
        })?;
    let resized = source
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgba8();

    let dds = encode_dds(&resized, width, height);
    let full_name = misctex_stem(texture_name);
    let crc = name_crc(texture_name);

    let wta = build_wta(dds.len() - DDS_HEADER_SIZE, crc, &dds);
    let uvd = build_uvd(texture_name, crc, width as f32, height as f32);

    let dat = dat::build(
        &[
            DatEntry::new(format!("{full_name}.wta"), wta),
            DatEntry::new(format!("{full_name}.uvd"), uvd),
        ],
        DatKind::Dat,
    )?;
    let dtt = dat::build(
        &[DatEntry::new(format!("{full_name}.wtp"), dds)],
        DatKind::Dtt,
    )?;

    Ok(MiscTex {
        dat,
        dtt,
        width,
        height,
    })
}

fn encode_dds(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let format = Format::Bc3;
    let compressed_size = format.compressed_size(width as usize, height as usize);
    let mut out = vec![0u8; DDS_HEADER_SIZE + compressed_size];
    write_dds_header(&mut out, width, height, compressed_size);
    format.compress(
        rgba,
        width as usize,
        height as usize,
        texpresso::Params::default(),
        &mut out[DDS_HEADER_SIZE..],
    );
    out
}

fn write_dds_header(out: &mut [u8], width: u32, height: u32, data_size: usize) {
    out[0..4].copy_from_slice(b"DDS ");
    put_u32(out, 4, 124);
    put_u32(out, 8, 0x0000_1007);
    put_u32(out, 12, height);
    put_u32(out, 16, width);
    put_u32(out, 20, data_size as u32);
    put_u32(out, 28, 0);
    put_u32(out, 0x44, 0x5454_564e);
    put_u32(out, 0x48, 0x0002_0008);
    put_u32(out, 76, 32);
    put_u32(out, 80, 0x4);
    out[84..88].copy_from_slice(b"DXT5");
    put_u32(out, 108, 0x1000);
}

fn build_wta(dds_size: usize, hash: u32, dds: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 0xC0];
    out[0..4].copy_from_slice(b"WTB\0");
    put_u32(&mut out, 4, 3);
    put_u32(&mut out, 8, 1);
    put_u32(&mut out, 12, 0x20);
    put_u32(&mut out, 16, 0x40);
    put_u32(&mut out, 20, 0x60);
    put_u32(&mut out, 24, 0x80);
    put_u32(&mut out, 28, 0xA0);

    put_u32(&mut out, 0x20, 0);
    put_u32(&mut out, 0x40, (dds_size + DDS_HEADER_SIZE) as u32);
    put_u32(&mut out, 0x60, 0x2200_0022);
    put_u32(&mut out, 0x80, hash);

    let is_cube = dds.len() > 116 && le_u32(dds, 112) == Some(0xFE00);
    put_u32(&mut out, 0xA0, 77);
    put_u32(&mut out, 0xA4, 3);
    put_u32(&mut out, 0xA8, if is_cube { 4 } else { 0 });
    put_u32(&mut out, 0xAC, 1);
    put_u32(&mut out, 0xB0, 0);
    out
}

fn build_uvd(texture_name: &str, texture_crc: u32, width: f32, height: f32) -> Vec<u8> {
    let temp_name = format!("{texture_name}_temp");
    let sprite_crc = name_crc(&temp_name);
    let mut out = vec![0u8; 0x94];

    put_u32(&mut out, 0, 1);
    put_u32(&mut out, 4, 1);
    put_u32(&mut out, 8, 0x34);
    put_u32(&mut out, 12, 0x10);

    put_fixed_str(&mut out, 0x10, texture_name, 32);
    put_u32(&mut out, 0x30, texture_crc);

    put_fixed_str(&mut out, 0x34, &temp_name, 32);
    put_u32(&mut out, 0x74, sprite_crc);
    put_u32(&mut out, 0x78, texture_crc);
    put_f32(&mut out, 0x7C, 0.0);
    put_f32(&mut out, 0x80, 0.0);
    put_f32(&mut out, 0x84, width);
    put_f32(&mut out, 0x88, height);
    put_f32(&mut out, 0x8C, 1.0 / width);
    put_f32(&mut out, 0x90, 1.0 / height);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_rules() {
        assert!(validate_size(256, 128).is_ok());
        assert!(matches!(
            validate_size(2, 128),
            Err(FormatError::ImageSizeOutOfRange {
                width: 2,
                height: 128
            })
        ));
        assert!(matches!(
            validate_size(4100, 128),
            Err(FormatError::ImageSizeOutOfRange { .. })
        ));
        assert!(matches!(
            validate_size(250, 128),
            Err(FormatError::ImageSizeNotMultipleOfFour { .. })
        ));
    }

    #[test]
    fn unreadable_image_is_reported() {
        assert!(matches!(
            create_misctex(b"not an image", "x", 8, 8),
            Err(FormatError::ImageUnreadable { .. })
        ));
    }
}
