use std::path::PathBuf;

use crate::archive::dat::DatArchive;
use crate::archive::dat_writer::{DatEntry, build_dat};
use crate::errors::{CodedError, ErrorCode};

#[derive(Debug)]
pub struct RenamedWeapon {
    pub dat: Vec<u8>,
    pub dtt: Vec<u8>,
    pub from_stem: String,
    pub to_stem: String,
    pub renamed_entries: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponClass {
    SmallSword,
    LargeSword,
    Spear,
    CombatBracer,
}

pub fn class_of_stem(stem: &str) -> Option<WeaponClass> {
    let digits = stem.strip_prefix("wp").or_else(|| stem.strip_prefix("WP"))?;
    if digits.len() < 4 {
        return None;
    }
    let number = u32::from_str_radix(digits.get(..4)?, 16).ok()?;
    match number {
        0x000..=0x1FF => Some(WeaponClass::SmallSword),
        0x200..=0x3FF => Some(WeaponClass::LargeSword),
        0x400..=0x5FF => Some(WeaponClass::Spear),
        0x600..=0x7FF => Some(WeaponClass::CombatBracer),
        _ => None,
    }
}

fn stem_of(path: &str) -> Option<String> {
    PathBuf::from(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
}


fn entries_of(path: PathBuf) -> Result<Vec<DatEntry>, CodedError> {
    let mut archive = DatArchive::open(path)
        .map_err(|e| CodedError::new(ErrorCode::FileReadFailed, e))?;

    let listed: Vec<_> = archive.entries().to_vec();
    let mut out = Vec::with_capacity(listed.len());
    for entry in &listed {
        let data = archive
            .read_entry(entry)
            .map_err(|e| CodedError::new(ErrorCode::FileReadFailed, e))?;
        out.push(DatEntry {
            name: entry.name.clone(),
            data,
        });
    }
    Ok(out)
}

fn rename_entries(
    entries: &mut [DatEntry],
    from: &str,
    to: &str,
    renamed: &mut Vec<String>,
) {
    for entry in entries.iter_mut() {
        if !entry.name.starts_with(from) {
            continue;
        }
        entry.name = format!("{to}{}", &entry.name[from.len()..]);
        renamed.push(entry.name.clone());
    }
}

pub fn rename_weapon(
    dat_path: String,
    dtt_path: String,
    to_stem: String,
) -> Result<RenamedWeapon, CodedError> {
    let from_stem = stem_of(&dat_path)
        .ok_or_else(|| CodedError::bare(ErrorCode::WeaponStemUnreadable))?;

    if from_stem.len() != to_stem.len() {
        return Err(CodedError::new(
            ErrorCode::WeaponStemLengthMismatch,
            format!("{from_stem} -> {to_stem}"),
        ));
    }
    if class_of_stem(&from_stem).is_none() {
        return Err(CodedError::new(ErrorCode::WeaponStemUnreadable, from_stem));
    }
    if class_of_stem(&to_stem).is_none() {
        return Err(CodedError::new(ErrorCode::WeaponStemUnreadable, to_stem));
    }
    if from_stem == to_stem {
        return Err(CodedError::new(ErrorCode::WeaponSameStem, to_stem));
    }

    let mut renamed = Vec::new();

    let dat_source = dat_path.clone();
    let dtt_source = dtt_path.clone();

    let mut dat = entries_of(PathBuf::from(dat_path))?;
    rename_entries(&mut dat, &from_stem, &to_stem, &mut renamed);

    let mut dtt = entries_of(PathBuf::from(dtt_path))?;
    rename_entries(&mut dtt, &from_stem, &to_stem, &mut renamed);

    if renamed.is_empty() {
        return Err(CodedError::new(ErrorCode::WeaponNothingToRename, from_stem));
    }

    Ok(RenamedWeapon {
        dat: if dat.is_empty() {
            std::fs::read(&dat_source)
                .map_err(|e| CodedError::new(ErrorCode::FileReadFailed, e.to_string()))?
        } else {
            build_dat(&dat, b"DAT\0")?
        },
        dtt: if dtt.is_empty() {
            std::fs::read(&dtt_source)
                .map_err(|e| CodedError::new(ErrorCode::FileReadFailed, e.to_string()))?
        } else {
            build_dat(&dtt, b"DTT\0")?
        },
        from_stem,
        to_stem,
        renamed_entries: renamed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const WP: &str = r"F:\All_extracted\data006.cpk\wp";

    fn names_of(bytes: &[u8]) -> Vec<String> {
        let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let names_at = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
        let width = u32::from_le_bytes(bytes[names_at..names_at + 4].try_into().unwrap()) as usize;
        (0..count)
            .map(|i| {
                let at = names_at + 4 + i * width;
                let raw = &bytes[at..at + width];
                let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
                String::from_utf8_lossy(&raw[..end]).into_owned()
            })
            .collect()
    }

    fn mot_payload(bytes: &[u8], index: usize) -> Vec<u8> {
        let offsets_at = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let sizes_at = u32::from_le_bytes(bytes[20..24].try_into().unwrap()) as usize;
        let off = u32::from_le_bytes(
            bytes[offsets_at + index * 4..offsets_at + index * 4 + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let size = u32::from_le_bytes(
            bytes[sizes_at + index * 4..sizes_at + index * 4 + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        bytes[off..off + size].to_vec()
    }

    #[test]
    fn entry_names_take_the_new_stem() {
        assert!(std::path::Path::new(WP).is_dir(), "weapon dir not found");
        let out = rename_weapon(
            format!("{WP}/wp0070.dat"),
            format!("{WP}/wp0070.dtt"),
            "wp0100".to_string(),
        )
        .expect("rename failed");

        for name in names_of(&out.dat).iter().chain(names_of(&out.dtt).iter()) {
            assert!(name.starts_with("wp0100"), "entry not renamed: {name}");
        }
    }

    #[test]
    fn file_contents_are_left_exactly_as_they_were() {
        assert!(std::path::Path::new(WP).is_dir(), "weapon dir not found");
        let original = std::fs::read(format!("{WP}/wp0070.dat")).unwrap();
        let out = rename_weapon(
            format!("{WP}/wp0070.dat"),
            format!("{WP}/wp0070.dtt"),
            "wp0100".to_string(),
        )
        .expect("rename failed");

        for index in 0..names_of(&original).len() {
            assert_eq!(
                mot_payload(&original, index),
                mot_payload(&out.dat, index),
                "payload {index} was modified"
            );
        }
    }

    #[test]
    fn refuses_a_stem_of_a_different_length() {
        let err = rename_weapon(
            format!("{WP}/wp0070.dat"),
            format!("{WP}/wp0070.dtt"),
            "wp010".to_string(),
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::WeaponStemLengthMismatch as u32);
    }

    #[test]
    fn refuses_renaming_onto_itself() {
        let err = rename_weapon(
            format!("{WP}/wp0070.dat"),
            format!("{WP}/wp0070.dtt"),
            "wp0070".to_string(),
        )
        .unwrap_err();
        assert_eq!(err.code, ErrorCode::WeaponSameStem as u32);
    }
}
