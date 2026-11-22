use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use yp_core::GamePaths;
use yp_formats::files::WeaponRenameResult;

use crate::error::Result;
use crate::item::mod_files;
use crate::outfit::dotted_extension;
use crate::weapon::{self, WeaponType};

pub const WEAPON_OBJ_BASE: i64 = 0x30000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponSource {
    pub dat_path: PathBuf,
    pub dtt_path: PathBuf,
    pub stem: String,
    pub weapon_type: WeaponType,
}

fn stem_number(stem: &str) -> Option<i64> {
    let lower = stem.to_lowercase();
    let digits = lower.strip_prefix("wp")?.get(..4)?;
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    i64::from_str_radix(digits, 16).ok()
}

pub fn stem_type(stem: &str) -> Option<WeaponType> {
    weapon::type_from_obj_id(WEAPON_OBJ_BASE + stem_number(stem)?)
}

pub fn is_weapon_file(path: &Path) -> bool {
    matches!(dotted_extension(path).as_str(), ".dat" | ".dtt") && stem_type(&file_stem(path)).is_some()
}

fn file_stem(path: &Path) -> String {
    path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn inspect(path: &Path) -> Option<WeaponSource> {
    let stem = file_stem(path);
    let weapon_type = stem_type(&stem)?;
    let dat_path = path.with_file_name(format!("{stem}.dat"));
    let dtt_path = path.with_file_name(format!("{stem}.dtt"));
    (dat_path.is_file() && dtt_path.is_file()).then_some(WeaponSource {
        dat_path,
        dtt_path,
        stem,
        weapon_type,
    })
}

pub fn stem_for_obj_id(obj_id: i64) -> String {
    format!("wp{:04x}", obj_id - WEAPON_OBJ_BASE)
}

pub fn taken_obj_ids(game: &GamePaths) -> BTreeSet<i64> {
    let mut taken = weapon::taken_obj_ids(game);
    for path in mod_files(&game.mods_dir()) {
        if dotted_extension(&path) != ".dat" {
            continue;
        }
        if let Some(number) = stem_number(&file_stem(&path)) {
            taken.insert(WEAPON_OBJ_BASE + number);
        }
    }
    taken
}

pub fn model_dir_for(game: &GamePaths, mod_id: &str) -> PathBuf {
    game.mods_dir().join(mod_id).join("data").join("wp")
}

pub fn convert(source: &WeaponSource, target_obj_id: i64, game: &GamePaths, mod_id: &str) -> Result<WeaponRenameResult> {
    Ok(yp_formats::files::rename_weapon(
        &source.dat_path,
        &source.dtt_path,
        &stem_for_obj_id(target_obj_id),
        &model_dir_for(game, mod_id),
    )?)
}
