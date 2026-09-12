use std::fs;
use std::path::PathBuf;

use crate::archive::weapon_convert;
use crate::errors::{CodedError, ErrorCode};

#[derive(Debug, Clone)]
pub struct WeaponRenameError {
    pub code: u32,
    pub detail: String,
}

impl From<CodedError> for WeaponRenameError {
    fn from(error: CodedError) -> Self {
        Self {
            code: error.code,
            detail: error.detail,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeaponRenameResult {
    pub dat_path: String,
    pub dtt_path: String,
    pub from_stem: String,
    pub to_stem: String,
    pub renamed_entries: Vec<String>,
}

pub fn rename_weapon(
    dat_path: String,
    dtt_path: String,
    to_stem: String,
    out_dir: String,
) -> Result<WeaponRenameResult, WeaponRenameError> {
    let renamed = weapon_convert::rename_weapon(dat_path, dtt_path, to_stem)?;

    let dir = PathBuf::from(out_dir);
    fs::create_dir_all(&dir)
        .map_err(|e| CodedError::new(ErrorCode::DirectoryCreateFailed, e.to_string()))?;

    let dat_path = dir.join(format!("{}.dat", renamed.to_stem));
    let dtt_path = dir.join(format!("{}.dtt", renamed.to_stem));

    fs::write(&dat_path, &renamed.dat)
        .map_err(|e| CodedError::new(ErrorCode::FileWriteFailed, e.to_string()))?;
    fs::write(&dtt_path, &renamed.dtt)
        .map_err(|e| CodedError::new(ErrorCode::FileWriteFailed, e.to_string()))?;

    Ok(WeaponRenameResult {
        dat_path: dat_path.to_string_lossy().into_owned(),
        dtt_path: dtt_path.to_string_lossy().into_owned(),
        from_stem: renamed.from_stem,
        to_stem: renamed.to_stem,
        renamed_entries: renamed.renamed_entries,
    })
}
