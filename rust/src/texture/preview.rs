use std::path::PathBuf;

use texpresso::Format;

use crate::archive::DatArchive;
use crate::errors::{CodedError, ErrorCode};

#[derive(Debug)]
pub struct DecodedTexture {
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
}

const DDS_HEADER_SIZE: usize = 128;

pub fn decode_misctex(dtt_path: PathBuf) -> Result<DecodedTexture, CodedError> {
    let mut archive = DatArchive::open(dtt_path)
        .map_err(|e| CodedError::new(ErrorCode::FileReadFailed, e))?;

    let entry = archive
        .entries()
        .iter()
        .find(|e| e.name.ends_with(".wtp"))
        .cloned()
        .ok_or_else(|| CodedError::bare(ErrorCode::TextureNoWtp))?;

    let wtp = archive
        .read_entry(&entry)
        .map_err(|e| CodedError::new(ErrorCode::FileReadFailed, e))?;

    decode_dds(&wtp)
}

pub fn decode_dds(buf: &[u8]) -> Result<DecodedTexture, CodedError> {
    if buf.len() < DDS_HEADER_SIZE || &buf[0..4] != b"DDS " {
        return Err(CodedError::bare(ErrorCode::TextureNotDds));
    }

    let height = read_u32(buf, 12);
    let width = read_u32(buf, 16);

    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return Err(CodedError::new(
            ErrorCode::TextureBadSize,
            format!("{width}x{height}"),
        ));
    }

    let fourcc = &buf[84..88];
    let format = match fourcc {
        b"DXT1" => Format::Bc1,
        b"DXT3" => Format::Bc2,
        b"DXT5" => Format::Bc3,
        _ => {
            return Err(CodedError::new(
                ErrorCode::TextureUnsupportedFormat,
                String::from_utf8_lossy(fourcc).into_owned(),
            ))
        }
    };

    let expected = format.compressed_size(width as usize, height as usize);
    let payload = &buf[DDS_HEADER_SIZE..];
    if payload.len() < expected {
        return Err(CodedError::new(
            ErrorCode::TextureTruncated,
            format!("{} < {expected}", payload.len()),
        ));
    }

    let mut rgba = vec![0u8; (width as usize) * (height as usize) * 4];
    format.decompress(
        &payload[..expected],
        width as usize,
        height as usize,
        &mut rgba,
    );

    let image = image::RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| CodedError::bare(ErrorCode::TextureTruncated))?;

    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(
            &mut std::io::Cursor::new(&mut png),
            image::ImageFormat::Png,
        )
        .map_err(|e| CodedError::new(ErrorCode::TextureEncodeFailed, e.to_string()))?;

    Ok(DecodedTexture {
        width,
        height,
        png,
    })
}

fn read_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ])
}
