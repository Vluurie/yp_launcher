use image::imageops::FilterType;
use texpresso::Format;

use crate::archive::dat_writer::{build_dat, name_crc, DatEntry};
use crate::errors::{CodedError, ErrorCode};

#[derive(Debug)]
pub struct MiscTex {
    pub dat: Vec<u8>,
    pub dtt: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

const DDS_HEADER_SIZE: usize = 128;

pub fn create_misctex(
    image_bytes: &[u8],
    texture_name: &str,
    width: u32,
    height: u32,
) -> Result<MiscTex, CodedError> {
    if !(4..=4096).contains(&width) || !(4..=4096).contains(&height) {
        return Err(CodedError::new(
            ErrorCode::ImageSizeOutOfRange,
            format!("{width}x{height}"),
        ));
    }
    if width % 4 != 0 || height % 4 != 0 {
        return Err(CodedError::new(
            ErrorCode::ImageSizeNotMultipleOfFour,
            format!("{width}x{height}"),
        ));
    }

    let source = image::load_from_memory(image_bytes)
        .map_err(|e| CodedError::new(ErrorCode::ImageUnreadable, e.to_string()))?;
    let resized = source
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgba8();

    let dds = encode_dds(&resized, width, height);
    let full_name = format!("misctex_{texture_name}");
    let crc = name_crc(texture_name);

    let wta = build_wta(dds.len() - DDS_HEADER_SIZE, crc, &dds);
    let uvd = build_uvd(texture_name, crc, width as f32, height as f32);

    let dat = build_dat(
        &[
            DatEntry {
                name: format!("{full_name}.wta"),
                data: wta,
            },
            DatEntry {
                name: format!("{full_name}.uvd"),
                data: uvd,
            },
        ],
        b"DAT\0",
    )?;

    let dtt = build_dat(
        &[DatEntry {
            name: format!("{full_name}.wtp"),
            data: dds.clone(),
        }],
        b"DTT\0",
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

fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
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

    let is_cube = dds.len() > 116
        && u32::from_le_bytes(dds[112..116].try_into().unwrap_or([0; 4])) == 0xFE00;
    put_u32(&mut out, 0xA0, 77);
    put_u32(&mut out, 0xA4, 3);
    put_u32(&mut out, 0xA8, if is_cube { 4 } else { 0 });
    put_u32(&mut out, 0xAC, 1);
    put_u32(&mut out, 0xB0, 0);

    out
}

fn build_uvd(texture_name: &str, tex_crc: u32, width: f32, height: f32) -> Vec<u8> {
    let temp_name = format!("{texture_name}_temp");
    let sprite_crc = name_crc(&temp_name);

    let mut out = vec![0u8; 0x94];

    put_u32(&mut out, 0, 1);
    put_u32(&mut out, 4, 1);
    put_u32(&mut out, 8, 0x34);
    put_u32(&mut out, 12, 0x10);

    put_fixed_string(&mut out, 0x10, texture_name, 32);
    put_u32(&mut out, 0x30, tex_crc);

    put_fixed_string(&mut out, 0x34, &temp_name, 32);
    put_u32(&mut out, 0x74, sprite_crc);
    put_u32(&mut out, 0x78, tex_crc);
    put_f32(&mut out, 0x7C, 0.0);
    put_f32(&mut out, 0x80, 0.0);
    put_f32(&mut out, 0x84, width);
    put_f32(&mut out, 0x88, height);
    put_f32(&mut out, 0x8C, 1.0 / width);
    put_f32(&mut out, 0x90, 1.0 / height);

    out
}

fn put_f32(buffer: &mut [u8], offset: usize, value: f32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_fixed_string(buffer: &mut [u8], offset: usize, value: &str, length: usize) {
    let bytes = value.as_bytes();
    let copy = bytes.len().min(length);
    buffer[offset..offset + copy].copy_from_slice(&bytes[..copy]);
}

