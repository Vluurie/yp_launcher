use std::path::{Path, PathBuf};

use crate::error::{FileOp, FormatError, Result};
use crate::texture::{misctex, preview};
use crate::weapon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiscTexResult {
    pub dat_path: PathBuf,
    pub dtt_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub dat_bytes: u64,
    pub dtt_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturePreview {
    pub width: u32,
    pub height: u32,
    pub png: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponRenameResult {
    pub dat_path: PathBuf,
    pub dtt_path: PathBuf,
    pub from_stem: String,
    pub to_stem: String,
    pub renamed_entries: Vec<String>,
}

pub fn misctex_paths(output_dir: &Path, texture_name: &str) -> (PathBuf, PathBuf) {
    let stem = misctex::misctex_stem(texture_name);
    (
        output_dir.join(format!("{stem}.dat")),
        output_dir.join(format!("{stem}.dtt")),
    )
}

pub fn weapon_paths(output_dir: &Path, stem: &str) -> (PathBuf, PathBuf) {
    (
        output_dir.join(format!("{stem}.dat")),
        output_dir.join(format!("{stem}.dtt")),
    )
}

pub fn read_misctex_preview(dtt_path: &Path) -> Result<TexturePreview> {
    let decoded = preview::decode_misctex(dtt_path)?;
    Ok(TexturePreview {
        width: decoded.width,
        height: decoded.height,
        png: decoded.png,
    })
}

pub fn create_item_thumbnail(
    source_image_path: &Path,
    output_dir: &Path,
    texture_name: &str,
    width: u32,
    height: u32,
) -> Result<MiscTexResult> {
    let image_bytes = std::fs::read(source_image_path)
        .map_err(|error| FormatError::io(FileOp::Read, source_image_path, error))?;
    let texture = misctex::create_misctex(&image_bytes, texture_name, width, height)?;

    create_dir(output_dir)?;
    let (dat_path, dtt_path) = misctex_paths(output_dir, texture_name);
    write_file(&dat_path, &texture.dat)?;
    write_file(&dtt_path, &texture.dtt)?;

    Ok(MiscTexResult {
        dat_path,
        dtt_path,
        width: texture.width,
        height: texture.height,
        dat_bytes: texture.dat.len() as u64,
        dtt_bytes: texture.dtt.len() as u64,
    })
}

pub fn rename_weapon(
    dat_path: &Path,
    dtt_path: &Path,
    to_stem: &str,
    out_dir: &Path,
) -> Result<WeaponRenameResult> {
    let renamed = weapon::rename_weapon(dat_path, dtt_path, to_stem)?;

    create_dir(out_dir)?;
    let (dat_out, dtt_out) = weapon_paths(out_dir, &renamed.to_stem);
    write_file(&dat_out, &renamed.dat)?;
    write_file(&dtt_out, &renamed.dtt)?;

    Ok(WeaponRenameResult {
        dat_path: dat_out,
        dtt_path: dtt_out,
        from_stem: renamed.from_stem,
        to_stem: renamed.to_stem,
        renamed_entries: renamed.renamed_entries,
    })
}

fn create_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(|error| FormatError::io(FileOp::CreateDir, dir, error))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    std::fs::write(path, bytes).map_err(|error| FormatError::io(FileOp::Write, path, error))
}
