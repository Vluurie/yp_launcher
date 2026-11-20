use std::io::Cursor;
use std::path::{Path, PathBuf};

use yp_formats::dat::{self, DatArchive, DatEntry, DatKind};
use yp_formats::weapon::{self, WeaponClass};
use yp_formats::{FormatError, files};

struct Pair {
    _dir: tempfile::TempDir,
    dat: PathBuf,
    dtt: PathBuf,
}

fn write_pair(stem: &str, dat_entries: Vec<DatEntry>, dtt_entries: Vec<DatEntry>) -> Pair {
    let dir = tempfile::tempdir().unwrap();
    let dat_path = dir.path().join(format!("{stem}.dat"));
    let dtt_path = dir.path().join(format!("{stem}.dtt"));
    std::fs::write(&dat_path, dat::build(&dat_entries, DatKind::Dat).unwrap()).unwrap();
    std::fs::write(&dtt_path, dat::build(&dtt_entries, DatKind::Dtt).unwrap()).unwrap();
    Pair {
        _dir: dir,
        dat: dat_path,
        dtt: dtt_path,
    }
}

fn weapon_pair() -> Pair {
    write_pair(
        "wp0070",
        vec![
            DatEntry::new("wp0070.wta", vec![1; 40]),
            DatEntry::new("wp0070.bxm", vec![2; 17]),
            DatEntry::new("wp0070_e00.mot", (0..90u8).collect()),
            DatEntry::new("common.bin", vec![9; 5]),
        ],
        vec![
            DatEntry::new(
                "wp0070.wmb",
                b"WMB3".iter().copied().chain([0; 60]).collect(),
            ),
            DatEntry::new("WP0070.wtp", vec![3; 128]),
        ],
    )
}

fn names_of(bytes: &[u8]) -> Vec<String> {
    DatArchive::from_reader(Cursor::new(bytes.to_vec()))
        .unwrap()
        .entries()
        .iter()
        .map(|entry| entry.name.clone())
        .collect()
}

fn payloads_of(bytes: &[u8]) -> Vec<Vec<u8>> {
    DatArchive::from_reader(Cursor::new(bytes.to_vec()))
        .unwrap()
        .read_all()
        .unwrap()
        .into_iter()
        .map(|entry| entry.data)
        .collect()
}

#[test]
fn entry_names_take_the_new_stem() {
    let pair = weapon_pair();
    let out = weapon::rename_weapon(&pair.dat, &pair.dtt, "wp0100").unwrap();
    assert_eq!(out.from_stem, "wp0070");
    assert_eq!(out.to_stem, "wp0100");
    assert_eq!(
        names_of(&out.dat),
        ["wp0100.wta", "wp0100.bxm", "wp0100_e00.mot", "common.bin"]
    );
    assert_eq!(names_of(&out.dtt), ["wp0100.wmb", "wp0100.wtp"]);
    assert_eq!(
        out.renamed_entries,
        [
            "wp0100.wta",
            "wp0100.bxm",
            "wp0100_e00.mot",
            "wp0100.wmb",
            "wp0100.wtp"
        ]
    );
}

#[test]
fn file_contents_are_left_exactly_as_they_were() {
    let pair = weapon_pair();
    let original_dat = std::fs::read(&pair.dat).unwrap();
    let original_dtt = std::fs::read(&pair.dtt).unwrap();
    let out = weapon::rename_weapon(&pair.dat, &pair.dtt, "wp0100").unwrap();
    assert_eq!(payloads_of(&out.dat), payloads_of(&original_dat));
    assert_eq!(payloads_of(&out.dtt), payloads_of(&original_dtt));
}

#[test]
fn refuses_a_stem_of_a_different_length() {
    let pair = weapon_pair();
    let error = weapon::rename_weapon(&pair.dat, &pair.dtt, "wp010").unwrap_err();
    assert!(
        matches!(&error, FormatError::WeaponStemLengthMismatch { from, to } if from == "wp0070" && to == "wp010")
    );
    assert_eq!(error.code(), 1203);
}

#[test]
fn refuses_renaming_onto_itself() {
    let pair = weapon_pair();
    let error = weapon::rename_weapon(&pair.dat, &pair.dtt, "wp0070").unwrap_err();
    assert!(matches!(error, FormatError::WeaponSameStem { .. }));
    assert_eq!(error.code(), 1204);
}

#[test]
fn refuses_a_different_weapon_class() {
    let pair = weapon_pair();
    let error = weapon::rename_weapon(&pair.dat, &pair.dtt, "wp0450").unwrap_err();
    assert!(matches!(
        error,
        FormatError::WeaponClassMismatch {
            from: WeaponClass::SmallSword,
            to: WeaponClass::Spear
        }
    ));
    assert_eq!(error.code(), 1206);
}

#[test]
fn refuses_stems_that_are_not_weapons() {
    let pair = weapon_pair();
    let error = weapon::rename_weapon(&pair.dat, &pair.dtt, "pl0100").unwrap_err();
    assert!(matches!(&error, FormatError::WeaponStemUnreadable { stem } if stem == "pl0100"));
    assert_eq!(error.code(), 1202);

    let player = write_pair(
        "pl0000",
        vec![DatEntry::new("pl0000.wmb", vec![1])],
        vec![DatEntry::new("pl0000.wtp", vec![1])],
    );
    let error = weapon::rename_weapon(&player.dat, &player.dtt, "wp0100").unwrap_err();
    assert!(matches!(&error, FormatError::WeaponStemUnreadable { stem } if stem == "pl0000"));
}

#[test]
fn refuses_archives_with_nothing_of_the_weapon_inside() {
    let pair = write_pair(
        "wp0070",
        vec![DatEntry::new("other.wta", vec![1])],
        vec![DatEntry::new("other.wtp", vec![1])],
    );
    let error = weapon::rename_weapon(&pair.dat, &pair.dtt, "wp0100").unwrap_err();
    assert!(matches!(error, FormatError::WeaponNothingToRename { .. }));
    assert_eq!(error.code(), 1205);
}

#[test]
fn files_api_writes_the_renamed_pair() {
    let pair = weapon_pair();
    let out_dir = tempfile::tempdir().unwrap();
    let target = out_dir
        .path()
        .join("mods")
        .join("my_mod")
        .join("data")
        .join("wp");
    let result = files::rename_weapon(&pair.dat, &pair.dtt, "wp0100", &target).unwrap();
    assert_eq!(result.dat_path, target.join("wp0100.dat"));
    assert_eq!(result.dtt_path, target.join("wp0100.dtt"));
    assert_eq!(
        files::weapon_paths(&target, "wp0100"),
        (result.dat_path.clone(), result.dtt_path.clone())
    );
    assert_eq!(result.from_stem, "wp0070");
    assert_eq!(result.renamed_entries.len(), 5);
    assert_eq!(
        names_of(&std::fs::read(&result.dat_path).unwrap())[0],
        "wp0100.wta"
    );
    assert_eq!(
        names_of(&std::fs::read(&result.dtt_path).unwrap())[0],
        "wp0100.wmb"
    );
}

#[test]
fn files_api_reports_a_missing_source() {
    let dir = tempfile::tempdir().unwrap();
    let error = files::rename_weapon(
        &dir.path().join("wp0070.dat"),
        &dir.path().join("wp0070.dtt"),
        "wp0100",
        dir.path(),
    )
    .unwrap_err();
    assert_eq!(error.code(), 1300);
}

fn extracted_wp_dir() -> PathBuf {
    let dir = PathBuf::from(std::env::var_os("YP_EXTRACTED_WP_DIR").expect("YP_EXTRACTED_WP_DIR"));
    assert!(dir.is_dir(), "weapon dir not found");
    dir
}

fn real_pair(dir: &Path) -> (PathBuf, PathBuf) {
    (dir.join("wp0070.dat"), dir.join("wp0070.dtt"))
}

#[test]
#[ignore = "needs YP_EXTRACTED_WP_DIR pointing at an extracted data006.cpk/wp folder"]
fn real_entry_names_take_the_new_stem() {
    let dir = extracted_wp_dir();
    let (dat_path, dtt_path) = real_pair(&dir);
    let out = weapon::rename_weapon(&dat_path, &dtt_path, "wp0100").unwrap();
    for name in names_of(&out.dat).iter().chain(names_of(&out.dtt).iter()) {
        assert!(name.starts_with("wp0100"), "entry not renamed: {name}");
    }
}

#[test]
#[ignore = "needs YP_EXTRACTED_WP_DIR pointing at an extracted data006.cpk/wp folder"]
fn real_file_contents_are_left_exactly_as_they_were() {
    let dir = extracted_wp_dir();
    let (dat_path, dtt_path) = real_pair(&dir);
    let original = std::fs::read(&dat_path).unwrap();
    let out = weapon::rename_weapon(&dat_path, &dtt_path, "wp0100").unwrap();
    assert_eq!(payloads_of(&out.dat), payloads_of(&original));
}
