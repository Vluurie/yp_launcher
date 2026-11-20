use std::path::Path;

use crate::dat::{self, DatArchive, DatEntry, DatKind};
use crate::error::{FileOp, FormatError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponClass {
    SmallSword,
    LargeSword,
    Spear,
    CombatBracer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamedWeapon {
    pub dat: Vec<u8>,
    pub dtt: Vec<u8>,
    pub from_stem: String,
    pub to_stem: String,
    pub renamed_entries: Vec<String>,
}

pub fn class_of_stem(stem: &str) -> Option<WeaponClass> {
    let prefix = stem.get(..2)?;
    if !prefix.eq_ignore_ascii_case("wp") {
        return None;
    }
    let digits = stem.get(2..6)?;
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let number = u32::from_str_radix(digits, 16).ok()?;
    match number {
        0x000..=0x1FF => Some(WeaponClass::SmallSword),
        0x200..=0x3FF => Some(WeaponClass::LargeSword),
        0x400..=0x5FF => Some(WeaponClass::Spear),
        0x600..=0x7FF => Some(WeaponClass::CombatBracer),
        _ => None,
    }
}

fn stem_of(path: &Path) -> Option<String> {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
}

fn has_stem_prefix(name: &str, stem: &str) -> bool {
    name.get(..stem.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(stem))
}

fn rename_entries(entries: &mut [DatEntry], from: &str, to: &str, renamed: &mut Vec<String>) {
    for entry in entries.iter_mut() {
        if !has_stem_prefix(&entry.name, from) {
            continue;
        }
        entry.name = format!("{to}{}", &entry.name[from.len()..]);
        renamed.push(entry.name.clone());
    }
}

fn rebuilt_or_original(entries: &[DatEntry], kind: DatKind, source: &Path) -> Result<Vec<u8>> {
    if entries.is_empty() {
        std::fs::read(source).map_err(|error| FormatError::io(FileOp::Read, source, error))
    } else {
        dat::build(entries, kind)
    }
}

pub fn rename_weapon(dat_path: &Path, dtt_path: &Path, to_stem: &str) -> Result<RenamedWeapon> {
    let from_stem = stem_of(dat_path).ok_or_else(|| FormatError::WeaponStemUnreadable {
        stem: String::new(),
    })?;

    if from_stem.len() != to_stem.len() {
        return Err(FormatError::WeaponStemLengthMismatch {
            from: from_stem,
            to: to_stem.to_string(),
        });
    }
    let from_class =
        class_of_stem(&from_stem).ok_or_else(|| FormatError::WeaponStemUnreadable {
            stem: from_stem.clone(),
        })?;
    let to_class = class_of_stem(to_stem).ok_or_else(|| FormatError::WeaponStemUnreadable {
        stem: to_stem.to_string(),
    })?;
    if from_class != to_class {
        return Err(FormatError::WeaponClassMismatch {
            from: from_class,
            to: to_class,
        });
    }
    if from_stem.eq_ignore_ascii_case(to_stem) {
        return Err(FormatError::WeaponSameStem {
            stem: to_stem.to_string(),
        });
    }

    let mut renamed = Vec::new();
    let mut dat_entries = DatArchive::open(dat_path)?.read_all()?;
    rename_entries(&mut dat_entries, &from_stem, to_stem, &mut renamed);
    let mut dtt_entries = DatArchive::open(dtt_path)?.read_all()?;
    rename_entries(&mut dtt_entries, &from_stem, to_stem, &mut renamed);

    if renamed.is_empty() {
        return Err(FormatError::WeaponNothingToRename { stem: from_stem });
    }

    Ok(RenamedWeapon {
        dat: rebuilt_or_original(&dat_entries, DatKind::Dat, dat_path)?,
        dtt: rebuilt_or_original(&dtt_entries, DatKind::Dtt, dtt_path)?,
        from_stem,
        to_stem: to_stem.to_string(),
        renamed_entries: renamed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_classes_follow_the_id_ranges() {
        assert_eq!(class_of_stem("wp0003"), Some(WeaponClass::SmallSword));
        assert_eq!(class_of_stem("wp0240"), Some(WeaponClass::LargeSword));
        assert_eq!(class_of_stem("WP0450"), Some(WeaponClass::Spear));
        assert_eq!(class_of_stem("wp0610"), Some(WeaponClass::CombatBracer));
        assert_eq!(class_of_stem("wp0800"), None);
        assert_eq!(class_of_stem("pl0000"), None);
        assert_eq!(class_of_stem("wp"), None);
        assert_eq!(class_of_stem("wp+123"), None);
        assert_eq!(class_of_stem("wpé12"), None);
    }

    #[test]
    fn stem_prefix_ignores_ascii_case() {
        assert!(has_stem_prefix("WP0070.wmb", "wp0070"));
        assert!(!has_stem_prefix("wp007", "wp0070"));
        assert!(!has_stem_prefix("é", "wp"));
    }
}
