use std::io::Cursor;
use std::path::PathBuf;

use yp_formats::dat::{self, DatArchive, DatEntry, DatKind, name_crc};
use yp_formats::{DatFault, FormatError};

fn sample_entries() -> Vec<DatEntry> {
    vec![
        DatEntry::new(
            "pl0000.wmb",
            b"WMB3".iter().copied().chain(0..200u8).collect(),
        ),
        DatEntry::new("pl0000.wta", vec![7; 33]),
        DatEntry::new("pl0000_scr.bxm", Vec::new()),
        DatEntry::new("README", b"no extension".to_vec()),
        DatEntry::new("pl0000.mot", (0..=255u8).cycle().take(5000).collect()),
    ]
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

#[test]
fn built_archives_read_back_identically() {
    for kind in [DatKind::Dat, DatKind::Dtt] {
        let entries = sample_entries();
        let bytes = dat::build(&entries, kind).unwrap();
        assert_eq!(bytes.len() % 16, 0);
        let mut archive = DatArchive::from_reader(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.kind(), kind);
        let names: Vec<&str> = archive
            .entries()
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        assert_eq!(
            names,
            entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>()
        );
        for entry in archive.entries() {
            assert_eq!(
                entry.offset % 32,
                0,
                "{} is not 32-byte aligned",
                entry.name
            );
        }
        assert_eq!(archive.read_all().unwrap(), entries);
    }
}

#[test]
fn round_trips_through_a_file_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pl0000.dat");
    let entries = sample_entries();
    std::fs::write(&path, dat::build(&entries, DatKind::Dat).unwrap()).unwrap();
    let mut archive = DatArchive::open(&path).unwrap();
    assert_eq!(archive.origin(), path.as_path());
    let rebuilt = dat::build(&archive.read_all().unwrap(), DatKind::Dat).unwrap();
    assert_eq!(rebuilt, std::fs::read(&path).unwrap());
}

#[test]
fn header_tables_point_at_the_written_sections() {
    let entries = sample_entries();
    let bytes = dat::build(&entries, DatKind::Dat).unwrap();
    assert_eq!(&bytes[0..4], b"DAT\0");
    assert_eq!(u32_at(&bytes, 4), entries.len() as u32);
    assert_eq!(u32_at(&bytes, 8), 32);
    let extensions_at = u32_at(&bytes, 12) as usize;
    assert_eq!(&bytes[extensions_at..extensions_at + 4], b"wmb\0");
    assert_eq!(&bytes[extensions_at + 12..extensions_at + 16], b"\0\0\0\0");
    let names_at = u32_at(&bytes, 16) as usize;
    assert_eq!(u32_at(&bytes, names_at), "pl0000_scr.bxm".len() as u32 + 1);
    let sizes_at = u32_at(&bytes, 20) as usize;
    assert_eq!(sizes_at % 4, 0);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(
            u32_at(&bytes, sizes_at + index * 4),
            entry.data.len() as u32
        );
    }
}

#[test]
fn hash_map_lookups_find_every_entry() {
    let names: Vec<String> = (0..300)
        .map(|index| format!("Entry_{index:03}.bin"))
        .collect();
    let entries: Vec<DatEntry> = names
        .iter()
        .map(|name| DatEntry::new(name.clone(), vec![1]))
        .collect();
    let bytes = dat::build(&entries, DatKind::Dat).unwrap();
    let map_at = u32_at(&bytes, 24) as usize;
    let shift = u32_at(&bytes, map_at);
    let buckets_at = map_at + u32_at(&bytes, map_at + 4) as usize;
    let hashes_at = map_at + u32_at(&bytes, map_at + 8) as usize;
    let indices_at = map_at + u32_at(&bytes, map_at + 12) as usize;
    let bucket_count = (hashes_at - buckets_at) / 2;
    assert_eq!(bucket_count, 1 << (31 - shift));
    for (index, name) in names.iter().enumerate() {
        let hash = name_crc(&name.to_lowercase());
        let bucket = (hash >> shift) as usize;
        let start = u16_at(&bytes, buckets_at + bucket * 2) as i16;
        assert!(start >= 0, "{name} has no bucket");
        let mut position = start as usize;
        let found = loop {
            assert!(position < names.len(), "{name} not found");
            let candidate = u32_at(&bytes, hashes_at + position * 4);
            assert_eq!(
                candidate >> shift,
                hash >> shift,
                "{name} walked out of its bucket"
            );
            if candidate == hash {
                break u16_at(&bytes, indices_at + position * 2) as usize;
            }
            position += 1;
        };
        assert_eq!(found, index);
    }
}

#[test]
fn lookups_and_prefix_reads() {
    let bytes = dat::build(&sample_entries(), DatKind::Dtt).unwrap();
    let mut archive = DatArchive::from_reader(Cursor::new(bytes)).unwrap();
    let entry = archive.find("PL0000.WMB").cloned().unwrap();
    assert_eq!(entry.index, 0);
    assert_eq!(archive.read_entry_prefix(&entry, 4).unwrap(), b"WMB3");
    assert_eq!(
        archive.read_entry_prefix(&entry, 100_000).unwrap().len(),
        204
    );
    assert_eq!(archive.entry(4).unwrap().name, "pl0000.mot");
    assert!(archive.entry(9).is_none());
    assert_eq!(
        archive.read_named("readme").unwrap(),
        Some(b"no extension".to_vec())
    );
    assert_eq!(archive.read_named("missing.bin").unwrap(), None);
}

#[test]
fn builder_rejects_empty_and_long_extensions() {
    assert!(matches!(
        dat::build(&[], DatKind::Dat),
        Err(FormatError::ArchiveEmpty)
    ));
    let error = dat::build(&[DatEntry::new("model.wmbx", vec![1])], DatKind::Dat).unwrap_err();
    assert!(
        matches!(&error, FormatError::ArchiveExtensionTooLong { name } if name == "model.wmbx")
    );
    assert_eq!(error.code(), 1201);
}

fn dat_fault(bytes: Vec<u8>) -> DatFault {
    match DatArchive::from_reader(Cursor::new(bytes)) {
        Err(FormatError::Dat(fault)) => fault,
        Err(other) => panic!("expected a DAT fault, got {other}"),
        Ok(_) => panic!("expected a DAT fault, got an archive"),
    }
}

#[test]
fn reader_rejects_malformed_archives() {
    assert_eq!(dat_fault(b"DAT\0".to_vec()), DatFault::SmallerThanHeader);

    let good = dat::build(&sample_entries(), DatKind::Dat).unwrap();

    let mut magic = good.clone();
    magic[0..4].copy_from_slice(b"CPK ");
    assert_eq!(dat_fault(magic), DatFault::UnsupportedMagic);

    let mut count = good.clone();
    count[4..8].copy_from_slice(&200_000u32.to_le_bytes());
    assert_eq!(dat_fault(count), DatFault::TooManyEntries);

    let mut offsets = good.clone();
    offsets[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(dat_fault(offsets), DatFault::RangeOutsideFile);

    let mut width = good.clone();
    let names_at = u32_at(&good, 16) as usize;
    width[names_at..names_at + 4].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(dat_fault(width), DatFault::InvalidNameWidth);

    let mut entry = good.clone();
    let sizes_at = u32_at(&good, 20) as usize;
    entry[sizes_at..sizes_at + 4].copy_from_slice(&(good.len() as u32).to_le_bytes());
    assert_eq!(dat_fault(entry), DatFault::RangeOutsideFile);

    let mut truncated = good;
    truncated.truncate(names_at + 2);
    assert_eq!(dat_fault(truncated), DatFault::RangeOutsideFile);
}

#[test]
fn opening_a_missing_file_is_a_read_error() {
    let dir = tempfile::tempdir().unwrap();
    let error = DatArchive::open(&dir.path().join("missing.dat"))
        .err()
        .unwrap();
    assert_eq!(error.code(), 1300);
}

#[test]
#[ignore = "needs YP_EXTRACTED_WP_DIR pointing at an extracted data006.cpk/wp folder"]
fn real_weapon_archives_rebuild_losslessly() {
    let dir = PathBuf::from(std::env::var_os("YP_EXTRACTED_WP_DIR").expect("YP_EXTRACTED_WP_DIR"));
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension().is_some_and(|extension| {
                extension.eq_ignore_ascii_case("dat") || extension.eq_ignore_ascii_case("dtt")
            })
        })
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    for path in paths {
        let mut archive =
            DatArchive::open(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let entries = archive.read_all().unwrap();
        if entries.is_empty() {
            continue;
        }
        let rebuilt = dat::build(&entries, archive.kind()).unwrap();
        let mut reread = DatArchive::from_reader(Cursor::new(rebuilt)).unwrap();
        assert_eq!(reread.read_all().unwrap(), entries, "{}", path.display());
    }
}
