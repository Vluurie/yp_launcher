#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::io::{Read, Seek};
use yp_formats::cloth::ClothGroup;
use yp_formats::wmb3::{ClothPlayer, Skeleton};
use yp_formats::{ArchiveEntry, DatArchive};

pub(crate) fn load<R: Read + Seek>(
    dat: &mut Option<DatArchive<R>>,
    dtt: &mut Option<DatArchive<R>>,
    stem: &str,
    skeleton: &Skeleton,
) -> ClothPlayer {
    read_groups(dat, dtt, stem).map_or_else(ClothPlayer::default, |groups| {
        ClothPlayer::bind(skeleton, groups)
    })
}

fn read_groups<R: Read + Seek>(
    dat: &mut Option<DatArchive<R>>,
    dtt: &mut Option<DatArchive<R>>,
    stem: &str,
) -> Option<Vec<ClothGroup>> {
    let prefix = format!("{stem}_");
    let mut files: BTreeMap<String, [Option<(bool, ArchiveEntry)>; 3]> = BTreeMap::new();
    let mut total = 0u64;
    for (secondary, archive) in [(false, dat.as_ref()), (true, dtt.as_ref())] {
        for entry in archive.into_iter().flat_map(|a| a.entries()) {
            let name = entry.name.to_ascii_lowercase();
            let Some((group, part)) = name.rsplit_once('_') else {
                continue;
            };
            if group != stem && !group.starts_with(&prefix) {
                continue;
            }
            let slot = match part {
                "clp.bxm" => 0,
                "clh.bxm" => 1,
                "clw.bxm" => 2,
                _ => continue,
            };
            total = total.checked_add(entry.size)?;
            if entry.size > 4 * 1024 * 1024 || total > 32 * 1024 * 1024 {
                return None;
            }
            let previous = files.entry(group.to_owned()).or_default()[slot]
                .replace((secondary, entry.clone()));
            if previous.is_some() || files.len() > 128 {
                return None;
            }
        }
    }
    let mut groups = Vec::new();
    for [parameters, collisions, winds] in files.into_values() {
        let mut read = |(secondary, entry): (bool, ArchiveEntry)| {
            let archive = if secondary {
                dtt.as_mut()?
            } else {
                dat.as_mut()?
            };
            archive.read_entry(&entry).ok()
        };
        let parameters = read(parameters?)?;
        let collisions = match collisions {
            Some(e) => Some(read(e)?),
            None => None,
        };
        let winds = match winds {
            Some(e) => Some(read(e)?),
            None => None,
        };
        groups.push(ClothGroup::parse(
            &parameters,
            collisions.as_deref(),
            winds.as_deref(),
        )?);
    }
    Some(groups)
}
