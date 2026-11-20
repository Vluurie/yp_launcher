use std::path::Path;

use texpresso::Format;

use crate::bytes::le_u32;
use crate::dat::DatArchive;
use crate::error::{FormatError, Result};
use crate::texture::{DDS_HEADER_SIZE, MAX_PREVIEW_SIDE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTexture {
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaTexture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn decode_misctex(dtt_path: &Path) -> Result<DecodedTexture> {
    let mut archive = DatArchive::open(dtt_path)?;
    let entry = archive
        .entries()
        .iter()
        .find(|entry| entry.has_extension("wtp"))
        .cloned()
        .ok_or(FormatError::TextureNoWtp)?;
    let wtp = archive.read_entry(&entry)?;
    decode_dds(&wtp)
}

pub fn decode_dds(buf: &[u8]) -> Result<DecodedTexture> {
    let texture = decode_dds_rgba(buf)?;
    let png = encode_png(&texture)?;
    Ok(DecodedTexture {
        width: texture.width,
        height: texture.height,
        png,
    })
}

pub fn decode_dds_rgba(buf: &[u8]) -> Result<RgbaTexture> {
    if buf.len() < DDS_HEADER_SIZE || &buf[0..4] != b"DDS " {
        return Err(FormatError::TextureNotDds);
    }
    let height = le_u32(buf, 12).ok_or(FormatError::TextureNotDds)?;
    let width = le_u32(buf, 16).ok_or(FormatError::TextureNotDds)?;
    if width == 0 || height == 0 || width > MAX_PREVIEW_SIDE || height > MAX_PREVIEW_SIDE {
        return Err(FormatError::TextureBadSize { width, height });
    }

    let fourcc = [buf[84], buf[85], buf[86], buf[87]];
    let format = match &fourcc {
        b"DXT1" => Format::Bc1,
        b"DXT3" => Format::Bc2,
        b"DXT5" => Format::Bc3,
        _ => return Err(FormatError::TextureUnsupportedFormat { fourcc }),
    };

    let expected = format.compressed_size(width as usize, height as usize);
    let payload = &buf[DDS_HEADER_SIZE..];
    if payload.len() < expected {
        return Err(FormatError::TextureTruncated {
            available: payload.len(),
            expected,
        });
    }

    let mut rgba = vec![0u8; (width as usize) * (height as usize) * 4];
    format.decompress(
        &payload[..expected],
        width as usize,
        height as usize,
        &mut rgba,
    );
    Ok(RgbaTexture {
        width,
        height,
        rgba,
    })
}

pub fn encode_png(texture: &RgbaTexture) -> Result<Vec<u8>> {
    let image = image::RgbaImage::from_raw(texture.width, texture.height, texture.rgba.clone())
        .ok_or(FormatError::TextureTruncated {
            available: texture.rgba.len(),
            expected: texture.width as usize * texture.height as usize * 4,
        })?;
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|error| FormatError::TextureEncodeFailed {
            detail: error.to_string(),
        })?;
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(width: u32, height: u32, fourcc: &[u8; 4]) -> Vec<u8> {
        let mut out = vec![0u8; DDS_HEADER_SIZE];
        out[0..4].copy_from_slice(b"DDS ");
        out[12..16].copy_from_slice(&height.to_le_bytes());
        out[16..20].copy_from_slice(&width.to_le_bytes());
        out[84..88].copy_from_slice(fourcc);
        out
    }

    #[test]
    fn rejects_non_dds() {
        assert!(matches!(
            decode_dds(b"PNG"),
            Err(FormatError::TextureNotDds)
        ));
        assert!(matches!(
            decode_dds(&[0u8; 200]),
            Err(FormatError::TextureNotDds)
        ));
    }

    #[test]
    fn rejects_bad_sizes_and_formats() {
        assert!(matches!(
            decode_dds(&header(0, 4, b"DXT1")),
            Err(FormatError::TextureBadSize {
                width: 0,
                height: 4
            })
        ));
        assert!(matches!(
            decode_dds(&header(4, 4, b"ATI2")),
            Err(FormatError::TextureUnsupportedFormat { fourcc }) if &fourcc == b"ATI2"
        ));
        assert!(matches!(
            decode_dds(&header(8, 8, b"DXT5")),
            Err(FormatError::TextureTruncated {
                available: 0,
                expected: 64
            })
        ));
    }

    #[test]
    fn decodes_a_solid_bc1_block() {
        let mut dds = header(4, 4, b"DXT1");
        dds.extend_from_slice(&[0x00, 0xF8, 0x00, 0xF8, 0, 0, 0, 0]);
        let texture = decode_dds_rgba(&dds).unwrap();
        assert_eq!((texture.width, texture.height), (4, 4));
        assert!(
            texture
                .rgba
                .chunks_exact(4)
                .all(|pixel| pixel[0] > 240 && pixel[1] < 10 && pixel[3] == 255)
        );
        let decoded = decode_dds(&dds).unwrap();
        assert_eq!(&decoded.png[1..4], b"PNG");
    }
}
