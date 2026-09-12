use std::fs;
use std::path::PathBuf;

use crate::errors::{CodedError, ErrorCode};
use crate::texture::misctex;
use crate::texture::preview;

#[derive(Debug, Clone)]
pub struct MiscTexResult {
    pub dat_path: String,
    pub dtt_path: String,
    pub width: u32,
    pub height: u32,
    pub dat_bytes: u64,
    pub dtt_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct MiscTexError {
    pub code: u32,
    pub detail: String,
}

impl From<CodedError> for MiscTexError {
    fn from(error: CodedError) -> Self {
        Self {
            code: error.code,
            detail: error.detail,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TexturePreview {
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
}

pub fn read_misctex_preview(dtt_path: String) -> Result<TexturePreview, MiscTexError> {
    let decoded = preview::decode_misctex(PathBuf::from(dtt_path))?;
    Ok(TexturePreview {
        width: decoded.width,
        height: decoded.height,
        png: decoded.png,
    })
}

pub fn create_item_thumbnail(
    source_image_path: String,
    output_dir: String,
    texture_name: String,
    width: u32,
    height: u32,
) -> Result<MiscTexResult, MiscTexError> {
    let image_bytes = fs::read(&source_image_path).map_err(|e| {
        MiscTexError::from(CodedError::new(ErrorCode::FileReadFailed, e.to_string()))
    })?;

    let tex = misctex::create_misctex(&image_bytes, &texture_name, width, height)?;

    let dir = PathBuf::from(&output_dir);
    fs::create_dir_all(&dir).map_err(|e| {
        MiscTexError::from(CodedError::new(
            ErrorCode::DirectoryCreateFailed,
            e.to_string(),
        ))
    })?;

    let stem = format!("misctex_{texture_name}");
    let dat_path = dir.join(format!("{stem}.dat"));
    let dtt_path = dir.join(format!("{stem}.dtt"));

    fs::write(&dat_path, &tex.dat).map_err(|e| {
        MiscTexError::from(CodedError::new(ErrorCode::FileWriteFailed, e.to_string()))
    })?;
    fs::write(&dtt_path, &tex.dtt).map_err(|e| {
        MiscTexError::from(CodedError::new(ErrorCode::FileWriteFailed, e.to_string()))
    })?;

    Ok(MiscTexResult {
        dat_path: dat_path.to_string_lossy().into_owned(),
        dtt_path: dtt_path.to_string_lossy().into_owned(),
        width: tex.width,
        height: tex.height,
        dat_bytes: tex.dat.len() as u64,
        dtt_bytes: tex.dtt.len() as u64,
    })
}
