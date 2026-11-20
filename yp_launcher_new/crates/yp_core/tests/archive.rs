mod support;

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use support::{Recorder, files, fixture, listing_mismatch, read, text, tree};
use yp_core::archive::{ARCHIVE_TEMP_PREFIX, ArchiveEntry, ArchiveFormat, BACKSLASH_IN_NAME, Extractor};
use yp_core::testing::archives::{
    RawRarEntry, RawZipEntry, ZIP_FLAG_ENCRYPTED, ZIP_HOST_UNIX, noise, raw_rar_bytes, raw_zip_bytes, write_encrypted_zip,
    write_rar, write_raw_rar, write_raw_zip, write_seven_z, write_solid_seven_z, write_zip, write_zip_with,
};
use yp_core::{ArchiveFailure, CancelToken, Error, HostEnv, NoProgress, Os};

const ZIP_METHOD_SHRINK: u16 = 1;
const ZIP_METHOD_AES: u16 = 99;
const ZIP_AES_EXTRA: [u8; 11] = [0x01, 0x99, 0x07, 0x00, 0x02, 0x00, b'A', b'E', 0x03, 0x08, 0x00];
const ZIP_EXTENDED_TIMESTAMP: u16 = 0x5455;
const ZIP_EXTENDED_TIMESTAMP_HAS_MTIME: u8 = 1;
const ZIP_NTFS_TIMES: u16 = 0x000a;
const ZIP_NTFS_TIMES_TAG: u16 = 0x0001;
const SHIFT_JIS: u32 = 932;
const BIG_ENTRY: usize = 1024 * 1024;
const PAYLOAD_TREE: [&str; 7] = [
    "payload/",
    "payload/empty.txt",
    "payload/hollow/",
    "payload/readme.txt",
    "payload/sub/",
    "payload/sub/code.bin",
    "payload/sub/data.bin",
];
const PAYLOAD_FILES: [&str; 3] = ["readme.txt", "sub/code.bin", "sub/data.bin"];
const NAMES_TREE: [&str; 12] = [
    "_aux.txt",
    "col_on.txt",
    "file ends dot_",
    "file ends space_",
    "plain/a.txt",
    "quest_ion.txt",
    "star_pipe_.txt",
    "sub/_NUL",
    "trailing dot_/c.txt",
    "trailing space_/b.txt",
    "two dots__/d.txt",
    "unicode/\u{30c6}\u{30b9}\u{30c8}.txt",
];
const NAMES: [(&str, &str); 12] = [
    ("plain/a.txt", "a"),
    ("trailing space /b.txt", "b"),
    ("trailing dot./c.txt", "c"),
    ("two dots../d.txt", "d"),
    ("file ends dot.", "e"),
    ("file ends space ", "f"),
    ("col:on.txt", "g"),
    ("quest?ion.txt", "h"),
    ("star*pipe|.txt", "i"),
    ("aux.txt", "j"),
    ("sub/NUL", "k"),
    ("unicode/\u{30c6}\u{30b9}\u{30c8}.txt", "l"),
];

struct Bench {
    dir: tempfile::TempDir,
}

impl Bench {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    fn out(&self) -> PathBuf {
        self.path("out")
    }

    fn extract(&self, archive: &Path) -> yp_core::Result<()> {
        Extractor::new().extract(archive, &self.out(), &NoProgress, &CancelToken::new())
    }

    fn extracted(&self, archive: &Path) -> Vec<String> {
        self.extract(archive).unwrap_or_else(|error| panic!("{}: {error}", archive.display()));
        let listing = Extractor::new().list(archive).unwrap();
        assert_eq!(listing_mismatch(&listing, &self.out()), None, "{}", archive.display());
        tree(&self.out())
    }

    fn leftovers(&self) -> Vec<String> {
        if self.out().is_dir() { files(&self.out()) } else { Vec::new() }
    }
}

fn failure_of<T: std::fmt::Debug>(result: yp_core::Result<T>) -> Option<ArchiveFailure> {
    match result {
        Err(error) => {
            let failure = error.archive_failure();
            assert!(failure.is_some(), "not an archive failure: {error:?}");
            failure
        }
        Ok(value) => panic!("expected a failure, got {value:?}"),
    }
}

fn entry(path: &str, is_dir: bool, size: u64) -> ArchiveEntry {
    ArchiveEntry {
        path: path.to_owned(),
        is_dir,
        size,
    }
}

fn sample() -> [(&'static str, &'static [u8]); 4] {
    [
        ("mod/", b""),
        ("mod/info.txt", b"PRJ_028"),
        ("mod/data/", b""),
        ("mod/data/a.bin", b"hello"),
    ]
}

fn assert_sample(archive: &Path) {
    let extractor = Extractor::new();
    let listing = extractor.list(archive).unwrap();
    assert_eq!(
        listing,
        vec![
            entry("mod", true, 0),
            entry("mod/info.txt", false, 7),
            entry("mod/data", true, 0),
            entry("mod/data/a.bin", false, 5),
        ],
        "{}",
        archive.display()
    );
    assert!(extractor.contains_extension(archive, &[".BIN"]).unwrap());
    assert!(!extractor.contains_extension(archive, &[".cpk"]).unwrap());
    assert!(extractor.contains_any(archive, &mut |path| path == "mod/data").unwrap());
    let bench = Bench::new();
    assert_eq!(
        bench.extracted(archive),
        ["mod/", "mod/data/", "mod/data/a.bin", "mod/info.txt"],
        "{}",
        archive.display()
    );
    assert_eq!(text(&bench.out(), "mod/info.txt"), "PRJ_028");
    assert_eq!(text(&bench.out(), "mod/data/a.bin"), "hello");
}

#[test]
fn a_zip_lists_and_extracts() {
    let bench = Bench::new();
    let archive = bench.path("sample.zip");
    write_zip(&archive, &sample());
    assert_sample(&archive);
}

#[test]
fn a_seven_z_lists_and_extracts() {
    let bench = Bench::new();
    let archive = bench.path("sample.7z");
    write_seven_z(&archive, &sample());
    let extractor = Extractor::new();
    let mut listing = extractor.list(&archive).unwrap();
    listing.sort_by(|a, b| a.path.cmp(&b.path));
    assert_eq!(
        listing,
        vec![
            entry("mod", true, 0),
            entry("mod/data", true, 0),
            entry("mod/data/a.bin", false, 5),
            entry("mod/info.txt", false, 7),
        ]
    );
    let out = Bench::new();
    assert_eq!(out.extracted(&archive), ["mod/", "mod/data/", "mod/data/a.bin", "mod/info.txt"]);
    assert_eq!(text(&out.out(), "mod/info.txt"), "PRJ_028");
    assert_eq!(text(&out.out(), "mod/data/a.bin"), "hello");
}

#[test]
fn a_rar_lists_and_extracts() {
    let bench = Bench::new();
    let archive = bench.path("sample.rar");
    write_rar(&archive, &sample());
    assert_sample(&archive);
}

#[test]
fn extracting_to_a_scoped_temp_folder() {
    let bench = Bench::new();
    let archive = bench.path("sample.zip");
    write_zip(&archive, &sample());
    let host = HostEnv::for_test(Os::Windows, &[], bench.dir.path());
    let recorder = Recorder::new();
    let out = Extractor::new()
        .extract_to_temp(&archive, None, &host, &recorder, &CancelToken::new())
        .unwrap();
    assert!(out.path().starts_with(host.temp_dir()));
    assert!(out.path().file_name().unwrap().to_string_lossy().starts_with(ARCHIVE_TEMP_PREFIX));
    assert_eq!(text(out.path(), "mod/info.txt"), "PRJ_028");
    assert_eq!(recorder.fractions().last(), Some(&1.0));

    let not_an_archive = bench.path("notes.tar.gz");
    std::fs::write(&not_an_archive, raw_zip_bytes(&[RawZipEntry::utf8("a.txt", b"a")])).unwrap();
    let refused = Extractor::new().extract_to_temp(&not_an_archive, None, &host, &NoProgress, &CancelToken::new());
    assert_eq!(failure_of(refused), Some(ArchiveFailure::UnsupportedFormat));

    let broken = bench.path("broken.zip");
    std::fs::write(&broken, b"this is not a zip file at all").unwrap();
    let failed = Extractor::new().extract_to_temp(&broken, None, &host, &NoProgress, &CancelToken::new());
    assert_eq!(failure_of(failed), Some(ArchiveFailure::Corrupt));
    let left: Vec<_> = std::fs::read_dir(host.temp_dir()).unwrap().filter_map(Result::ok).collect();
    assert_eq!(left.len(), 1, "{left:?}");
}

#[test]
fn every_zip_method_written_by_the_reference_tool_extracts() {
    for method in ["copy", "deflate", "deflate64", "bzip2", "lzma", "ppmd"] {
        let archive = fixture(&format!("methods/zip_{method}.zip"));
        let bench = Bench::new();
        assert_eq!(bench.extracted(&archive), PAYLOAD_TREE, "{method}");
        for file in PAYLOAD_FILES {
            assert_eq!(
                read(&bench.out(), &format!("payload/{file}")),
                read(&fixture("payload"), file),
                "{method}: {file}"
            );
        }
        assert!(read(&bench.out(), "payload/empty.txt").is_empty());
    }
}

#[test]
fn every_seven_z_method_written_by_the_reference_tool_extracts() {
    for method in ["copy", "lzma", "lzma2", "bcj", "bcj2", "delta", "ppmd", "bzip2", "nonsolid"] {
        let archive = fixture(&format!("methods/sevenz_{method}.7z"));
        let bench = Bench::new();
        assert_eq!(bench.extracted(&archive), PAYLOAD_TREE, "{method}");
        for file in PAYLOAD_FILES {
            assert_eq!(
                read(&bench.out(), &format!("payload/{file}")),
                read(&fixture("payload"), file),
                "{method}: {file}"
            );
        }
        assert!(read(&bench.out(), "payload/empty.txt").is_empty());
    }
}

#[test]
fn every_zip_method_the_writer_offers_round_trips() {
    use zip::CompressionMethod;
    let body = noise(40_000, 7);
    let text: Vec<u8> = b"compressible ".repeat(4_000);
    for method in [
        CompressionMethod::Stored,
        CompressionMethod::Deflated,
        CompressionMethod::Bzip2,
        CompressionMethod::Ppmd,
    ] {
        let bench = Bench::new();
        let archive = bench.path("methods.zip");
        write_zip_with(&archive, method, &[("a/noise.bin", &body), ("a/text.txt", &text)]);
        assert_eq!(bench.extracted(&archive), ["a/", "a/noise.bin", "a/text.txt"], "{method:?}");
        assert_eq!(read(&bench.out(), "a/noise.bin"), body, "{method:?}");
        assert_eq!(read(&bench.out(), "a/text.txt"), text, "{method:?}");
    }
}

#[test]
fn the_signature_decides_before_the_extension() {
    let bench = Bench::new();
    let seven_z_named_zip = bench.path("really_7z.zip");
    write_seven_z(&seven_z_named_zip, &sample());
    let zip_named_rar = bench.path("really_zip.rar");
    write_zip(&zip_named_rar, &sample());
    let rar_named_seven_z = bench.path("really_rar.7z");
    write_rar(&rar_named_seven_z, &sample());
    assert_eq!(ArchiveFormat::detect(&seven_z_named_zip).unwrap(), ArchiveFormat::SevenZ);
    assert_eq!(ArchiveFormat::detect(&zip_named_rar).unwrap(), ArchiveFormat::Zip);
    assert_eq!(ArchiveFormat::detect(&rar_named_seven_z).unwrap(), ArchiveFormat::Rar);
    for archive in [&seven_z_named_zip, &zip_named_rar, &rar_named_seven_z] {
        let out = Bench::new();
        assert_eq!(
            out.extracted(archive),
            ["mod/", "mod/data/", "mod/data/a.bin", "mod/info.txt"],
            "{}",
            archive.display()
        );
    }

    let prefixed = bench.path("prefixed.zip");
    let mut bytes = b"MZ stub in front of the archive ".to_vec();
    let stub = bytes.len();
    bytes.extend_from_slice(&std::fs::read(&zip_named_rar).unwrap());
    std::fs::write(&prefixed, bytes).unwrap();
    assert_eq!(ArchiveFormat::detect(&prefixed).unwrap(), ArchiveFormat::Zip);
    assert!(stub > 0 && Extractor::new().list(&prefixed).is_ok_and(|listing| listing.len() == 4));

    let unknown = bench.path("notes.txt");
    std::fs::write(&unknown, "plain text").unwrap();
    assert_eq!(failure_of(ArchiveFormat::detect(&unknown)), Some(ArchiveFailure::UnsupportedFormat));
    assert_eq!(failure_of(Extractor::new().list(&unknown)), Some(ArchiveFailure::UnsupportedFormat));
    assert_eq!(failure_of(bench.extract(&unknown)), Some(ArchiveFailure::UnsupportedFormat));
    let gzip = bench.path("pack.tar.gz");
    std::fs::write(&gzip, [0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00]).unwrap();
    assert_eq!(failure_of(Extractor::new().list(&gzip)), Some(ArchiveFailure::UnsupportedFormat));
    assert!(!bench.out().exists());
}

fn traversal_names(sandbox: &Path, tag: &str) -> (Vec<(String, String)>, String, String) {
    let target = sandbox.join("abs_target");
    std::fs::create_dir_all(&target).unwrap();
    let absolute = target.to_string_lossy().replace('\\', "/");
    let rooted = match absolute.split_once(':') {
        Some((_, rest)) => rest.to_owned(),
        None => absolute.clone(),
    };
    let names = vec![
        ("ok/normal.txt".to_owned(), "normal".to_owned()),
        (format!("../evil_dotdot_{tag}.txt"), "dotdot".to_owned()),
        (format!("sub/../../evil_dotdot2_{tag}.txt"), "dotdot2".to_owned()),
        (format!("..\\evil_backslash_{tag}.txt"), "backslash".to_owned()),
        (format!("{absolute}/evil_abs_{tag}.txt"), "abs".to_owned()),
        (format!("{rooted}/evil_rooted_{tag}.txt"), "rooted".to_owned()),
        ("ok/last.txt".to_owned(), "last".to_owned()),
    ];
    let inside_absolute = absolute.replace(':', "_").trim_start_matches('/').to_owned();
    let inside_rooted = rooted.trim_start_matches('/').to_owned();
    (names, inside_absolute, inside_rooted)
}

fn assert_traversal_is_contained(extension: &str, backslash_entry: &dyn Fn(&str) -> String) {
    let bench = Bench::new();
    let sandbox = bench.path("sandbox");
    std::fs::create_dir_all(&sandbox).unwrap();
    let (names, inside_absolute, inside_rooted) = traversal_names(&sandbox, extension);
    let borrowed: Vec<(&str, &[u8])> = names
        .iter()
        .map(|(name, body)| (name.as_str(), body.as_bytes()))
        .collect();
    let archive = bench.path(&format!("trav.{extension}"));
    match extension {
        "zip" => {
            let raw: Vec<RawZipEntry> = borrowed.iter().map(|(name, body)| RawZipEntry::utf8(name, body)).collect();
            write_raw_zip(&archive, &raw);
        }
        "7z" => write_seven_z(&archive, &borrowed),
        _ => write_rar(&archive, &borrowed),
    }
    let out = sandbox.join("deep").join("out");
    Extractor::new()
        .extract(&archive, &out, &NoProgress, &CancelToken::new())
        .unwrap();
    let mut expected = vec![
        format!("{inside_absolute}/evil_abs_{extension}.txt"),
        backslash_entry(extension),
        format!("evil_dotdot_{extension}.txt"),
        "ok/last.txt".to_owned(),
        "ok/normal.txt".to_owned(),
        format!("sub/evil_dotdot2_{extension}.txt"),
        format!("{inside_rooted}/evil_rooted_{extension}.txt"),
    ];
    expected.sort();
    assert_eq!(files(&out), expected, "{extension}");
    assert_eq!(text(&out, &format!("sub/evil_dotdot2_{extension}.txt")), "dotdot2");
    assert_eq!(text(&out, &format!("{inside_absolute}/evil_abs_{extension}.txt")), "abs");
    assert_eq!(text(&out, "ok/last.txt"), "last");
    assert_eq!(tree(&sandbox.join("abs_target")), Vec::<String>::new(), "{extension}");
    assert_eq!(tree(&sandbox.join("deep")).iter().filter(|path| !path.starts_with("out/")).count(), 0);
    let mut top: Vec<String> = std::fs::read_dir(&sandbox)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    top.sort();
    assert_eq!(top, ["abs_target", "deep"], "{extension}");
    let listing = Extractor::new().list(&archive).unwrap();
    assert_eq!(listing_mismatch(&listing, &out), None, "{extension}");
    let mut listed: Vec<String> = listing.into_iter().map(|entry| entry.path).collect();
    listed.sort();
    assert_eq!(listed, expected, "{extension}");
}

#[test]
fn zip_entries_cannot_leave_the_destination() {
    assert_traversal_is_contained("zip", &|tag| format!("evil_backslash_{tag}.txt"));
}

#[test]
fn seven_z_entries_cannot_leave_the_destination() {
    assert_traversal_is_contained("7z", &|tag| format!("..{BACKSLASH_IN_NAME}evil_backslash_{tag}.txt"));
}

#[test]
fn rar_entries_cannot_leave_the_destination() {
    assert_traversal_is_contained("rar", &|tag| format!(".._evil_backslash_{tag}.txt"));
}

fn assert_names_tree(archive: &Path) {
    let bench = Bench::new();
    bench.extract(archive).unwrap_or_else(|error| panic!("{}: {error}", archive.display()));
    assert_eq!(files(&bench.out()), NAMES_TREE, "{}", archive.display());
    assert_eq!(text(&bench.out(), "quest_ion.txt"), "h");
    assert_eq!(text(&bench.out(), "star_pipe_.txt"), "i");
    assert_eq!(text(&bench.out(), "_aux.txt"), "j");
    assert_eq!(text(&bench.out(), "sub/_NUL"), "k");
    assert_eq!(text(&bench.out(), "two dots__/d.txt"), "d");
    assert_eq!(text(&bench.out(), "unicode/\u{30c6}\u{30b9}\u{30c8}.txt"), "l");
    let listing = Extractor::new().list(archive).unwrap();
    assert_eq!(listing_mismatch(&listing, &bench.out()), None, "{}", archive.display());
    let mut listed: Vec<String> = listing.into_iter().map(|entry| entry.path).collect();
    listed.sort();
    assert_eq!(listed, NAMES_TREE, "{}", archive.display());
}

#[test]
fn names_windows_refuses_are_mapped_in_a_zip() {
    let bench = Bench::new();
    let archive = bench.path("names.zip");
    let raw: Vec<RawZipEntry> = NAMES.iter().map(|(name, body)| RawZipEntry::utf8(name, body.as_bytes())).collect();
    write_raw_zip(&archive, &raw);
    assert_names_tree(&archive);
}

#[test]
fn names_windows_refuses_are_mapped_in_a_seven_z() {
    let bench = Bench::new();
    let archive = bench.path("names.7z");
    let entries: Vec<(&str, &[u8])> = NAMES.iter().map(|(name, body)| (*name, body.as_bytes())).collect();
    write_seven_z(&archive, &entries);
    assert_names_tree(&archive);
}

#[test]
fn names_windows_refuses_are_mapped_in_a_rar_and_later_entries_survive() {
    assert_names_tree(&fixture("names.rar"));
    let bench = Bench::new();
    let archive = bench.path("names.rar");
    let entries: Vec<(&str, &[u8])> = NAMES.iter().map(|(name, body)| (*name, body.as_bytes())).collect();
    write_rar(&archive, &entries);
    assert_names_tree(&archive);
}

fn legacy_zip(path: &Path) {
    write_raw_zip(
        path,
        &[
            RawZipEntry::stored(b"sjis/\x83\x65\x83\x58\x83\x67.txt", b"sjis"),
            RawZipEntry::stored(b"cp437/\x9a\x62\x75\x6e\x67.txt", b"cp437"),
            RawZipEntry::stored("utf8noflag/\u{dc}bung.txt".as_bytes(), b"utf8noflag"),
            RawZipEntry::stored(b"trail\x95\x5c/\x83\x5c.txt", b"trail"),
            RawZipEntry::utf8("flagged/\u{30c6}\u{30b9}\u{30c8}.txt", b"flagged"),
        ],
    );
}

#[test]
fn legacy_zip_names_use_the_oem_code_page() {
    let bench = Bench::new();
    let archive = bench.path("legacy.zip");
    legacy_zip(&archive);
    let extractor = Extractor::with_oem_code_page(Some(SHIFT_JIS));
    assert_eq!(extractor.legacy_name_encoding(), Some("Shift_JIS"));
    extractor
        .extract(&archive, &bench.out(), &NoProgress, &CancelToken::new())
        .unwrap();
    assert_eq!(
        files(&bench.out()),
        [
            "cp437/\u{5598}ung.txt",
            "flagged/\u{30c6}\u{30b9}\u{30c8}.txt",
            "sjis/\u{30c6}\u{30b9}\u{30c8}.txt",
            "trail\u{8868}/\u{30bd}.txt",
            "utf8noflag/\u{dc}bung.txt",
        ]
    );
    assert_eq!(text(&bench.out(), "sjis/\u{30c6}\u{30b9}\u{30c8}.txt"), "sjis");
    assert_eq!(text(&bench.out(), "trail\u{8868}/\u{30bd}.txt"), "trail");
    assert_eq!(listing_mismatch(&extractor.list(&archive).unwrap(), &bench.out()), None);
}

#[test]
fn legacy_zip_names_fall_back_to_cp437() {
    let bench = Bench::new();
    let archive = bench.path("legacy.zip");
    legacy_zip(&archive);
    for extractor in [Extractor::new(), Extractor::with_oem_code_page(Some(437)), Extractor::with_oem_code_page(None)] {
        assert_eq!(extractor.legacy_name_encoding(), None);
        let out = Bench::new();
        extractor
            .extract(&archive, &out.out(), &NoProgress, &CancelToken::new())
            .unwrap();
        assert_eq!(
            files(&out.out()),
            [
                "cp437/\u{dc}bung.txt",
                "flagged/\u{30c6}\u{30b9}\u{30c8}.txt",
                "sjis/\u{e2}e\u{e2}X\u{e2}g.txt",
                "trail\u{f2}/\u{e2}/.txt",
                "utf8noflag/\u{dc}bung.txt",
            ]
        );
        assert_eq!(listing_mismatch(&extractor.list(&archive).unwrap(), &out.out()), None);
    }
}

#[test]
fn a_backslash_separates_folders_unless_the_zip_was_made_on_unix() {
    let bench = Bench::new();
    let dos = bench.path("dos.zip");
    write_raw_zip(&dos, &[RawZipEntry::utf8("mod\\data\\a.bin", b"dos")]);
    assert_eq!(bench.extracted(&dos), ["mod/", "mod/data/", "mod/data/a.bin"]);

    let unix = bench.path("unix.zip");
    write_raw_zip(&unix, &[RawZipEntry::utf8("mod/we\\ird.bin", b"unix").host(ZIP_HOST_UNIX)]);
    let out = Bench::new();
    assert_eq!(out.extracted(&unix), ["mod/".to_owned(), format!("mod/we{BACKSLASH_IN_NAME}ird.bin")]);
}

#[test]
fn a_later_entry_with_the_same_name_wins() {
    let bench = Bench::new();
    let archive = bench.path("twice.zip");
    write_raw_zip(
        &archive,
        &[
            RawZipEntry::utf8("mod/a.txt", b"first"),
            RawZipEntry::utf8("mod/a.txt", b"second one"),
        ],
    );
    assert_eq!(bench.extracted(&archive), ["mod/", "mod/a.txt"]);
    assert_eq!(text(&bench.out(), "mod/a.txt"), "second one");
}

#[test]
fn files_already_in_the_destination_are_kept() {
    let bench = Bench::new();
    let archive = bench.path("sample.zip");
    write_zip(&archive, &sample());
    std::fs::create_dir_all(bench.out().join("mod")).unwrap();
    std::fs::write(bench.out().join("mod").join("keep.txt"), "mine").unwrap();
    std::fs::write(bench.out().join("mod").join("info.txt"), "old").unwrap();
    bench.extract(&archive).unwrap();
    assert_eq!(text(&bench.out(), "mod/keep.txt"), "mine");
    assert_eq!(text(&bench.out(), "mod/info.txt"), "PRJ_028");
}

fn assert_progress_inside_one_entry(archive: &Path, name: &str) {
    let bench = Bench::new();
    let recorder = Recorder::new();
    Extractor::new()
        .extract(archive, &bench.out(), &recorder, &CancelToken::new())
        .unwrap();
    let fractions = recorder.fractions();
    assert!(fractions.windows(2).all(|pair| pair[0] <= pair[1]), "{fractions:?}");
    assert_eq!(fractions.first(), Some(&0.0), "{fractions:?}");
    assert_eq!(fractions.last(), Some(&1.0), "{fractions:?}");
    let partial = fractions.iter().filter(|fraction| **fraction > 0.0 && **fraction < 1.0).count();
    assert!(partial >= 2, "no progress inside the entry: {fractions:?}");
    assert_eq!(recorder.names(), [name]);
    assert_eq!(read(&bench.out(), name), noise(BIG_ENTRY, 3));
}

#[test]
fn zip_progress_counts_bytes_inside_an_entry() {
    let bench = Bench::new();
    let archive = bench.path("big.zip");
    write_zip(&archive, &[("big/blob.bin", &noise(BIG_ENTRY, 3))]);
    assert_progress_inside_one_entry(&archive, "big/blob.bin");
}

#[test]
fn seven_z_progress_counts_bytes_inside_an_entry() {
    let bench = Bench::new();
    let archive = bench.path("big.7z");
    write_seven_z(&archive, &[("big/blob.bin", &noise(BIG_ENTRY, 3))]);
    assert_progress_inside_one_entry(&archive, "big/blob.bin");
}

#[test]
fn rar_progress_moves_with_every_entry() {
    let bench = Bench::new();
    let archive = bench.path("parts.rar");
    let body = noise(200_000, 5);
    write_rar(&archive, &[("a/one.bin", &body), ("a/two.bin", &body), ("a/three.bin", &body), ("a/four.bin", &body)]);
    let recorder = Recorder::new();
    Extractor::new()
        .extract(&archive, &bench.out(), &recorder, &CancelToken::new())
        .unwrap();
    assert_eq!(recorder.fractions(), [0.0, 0.25, 0.5, 0.75, 1.0]);
    assert_eq!(recorder.names(), ["a/one.bin", "a/two.bin", "a/three.bin", "a/four.bin"]);
    assert_eq!(files(&bench.out()).len(), 4);
}

#[test]
fn progress_spans_many_small_entries() {
    let bench = Bench::new();
    let archive = bench.path("many.zip");
    let body = noise(40_000, 9);
    let names: Vec<String> = (0..20).map(|index| format!("many/file_{index:02}.bin")).collect();
    let entries: Vec<(&str, &[u8])> = names.iter().map(|name| (name.as_str(), body.as_slice())).collect();
    write_zip(&archive, &entries);
    let recorder = Recorder::new();
    Extractor::new()
        .extract(&archive, &bench.out(), &recorder, &CancelToken::new())
        .unwrap();
    let fractions = recorder.fractions();
    assert!(fractions.windows(2).all(|pair| pair[0] <= pair[1]), "{fractions:?}");
    assert_eq!(fractions.last(), Some(&1.0));
    assert!(fractions.len() >= 10 && fractions.len() <= 22, "{fractions:?}");
    assert_eq!(files(&bench.out()).len(), 20);
}

fn assert_cancel_inside_one_entry(archive: &Path) {
    let bench = Bench::new();
    let cancel = CancelToken::new();
    let recorder = Recorder::cancelling(&cancel, 0.0);
    let result = Extractor::new().extract(archive, &bench.out(), &recorder, &cancel);
    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
    let reached = recorder.fractions().last().copied().unwrap_or_default();
    assert!(reached > 0.0 && reached < 1.0, "{reached}");
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn zip_extraction_cancels_inside_an_entry_and_removes_the_partial_file() {
    let bench = Bench::new();
    let archive = bench.path("big.zip");
    write_zip(&archive, &[("big/blob.bin", &noise(BIG_ENTRY, 3))]);
    assert_cancel_inside_one_entry(&archive);
}

#[test]
fn seven_z_extraction_cancels_inside_an_entry_and_removes_the_partial_file() {
    let bench = Bench::new();
    let archive = bench.path("big.7z");
    write_seven_z(&archive, &[("big/blob.bin", &noise(BIG_ENTRY, 3))]);
    assert_cancel_inside_one_entry(&archive);
}

#[test]
fn a_solid_seven_z_stops_in_the_middle_of_its_block() {
    let bench = Bench::new();
    let archive = bench.path("solid.7z");
    let body = noise(300_000, 11);
    write_solid_seven_z(&archive, &[("s/one.bin", &body), ("s/two.bin", &body), ("s/three.bin", &body)]);
    let complete = Bench::new();
    assert_eq!(complete.extracted(&archive), ["s/", "s/one.bin", "s/three.bin", "s/two.bin"]);
    assert_eq!(read(&complete.out(), "s/three.bin"), body);

    let cancel = CancelToken::new();
    let recorder = Recorder::cancelling(&cancel, 0.5);
    let result = Extractor::new().extract(&archive, &bench.out(), &recorder, &cancel);
    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn rar_extraction_cancels_between_entries() {
    let bench = Bench::new();
    let archive = bench.path("parts.rar");
    let body = noise(200_000, 5);
    write_rar(&archive, &[("a/one.bin", &body), ("a/two.bin", &body), ("a/three.bin", &body), ("a/four.bin", &body)]);
    let cancel = CancelToken::new();
    let recorder = Recorder::cancelling(&cancel, 0.2);
    let result = Extractor::new().extract(&archive, &bench.out(), &recorder, &cancel);
    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
    assert_eq!(recorder.names(), ["a/one.bin", "a/two.bin"]);
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn a_cancelled_token_stops_before_anything_is_written() {
    let bench = Bench::new();
    let cancel = CancelToken::new();
    cancel.cancel();
    for (name, write) in [
        ("a.zip", write_zip as fn(&Path, &[(&str, &[u8])])),
        ("a.7z", write_seven_z),
        ("a.rar", write_rar),
    ] {
        let archive = bench.path(name);
        write(&archive, &sample());
        let result = Extractor::new().extract(&archive, &bench.out(), &NoProgress, &cancel);
        assert!(matches!(result, Err(Error::Cancelled)), "{name}");
        assert!(!bench.out().exists(), "{name}");
    }
}

fn flipped(bytes: &[u8], at: usize) -> Vec<u8> {
    let mut damaged = bytes.to_vec();
    damaged[at] ^= 0xff;
    damaged
}

#[test]
fn a_damaged_zip_is_reported_and_leaves_nothing_behind() {
    let bench = Bench::new();
    let good = bench.path("good.zip");
    let body = noise(300_000, 13);
    write_zip_with(&good, zip::CompressionMethod::Stored, &[("m/first.txt", b"fine"), ("m/blob.bin", &body)]);
    let bytes = std::fs::read(&good).unwrap();

    let bad_checksum = bench.path("bad_checksum.zip");
    std::fs::write(&bad_checksum, flipped(&bytes, bytes.len() / 2)).unwrap();
    assert!(Extractor::new().list(&bad_checksum).is_ok());
    assert_eq!(failure_of(bench.extract(&bad_checksum)), Some(ArchiveFailure::Corrupt));
    assert_eq!(bench.leftovers(), Vec::<String>::new());

    let truncated = bench.path("truncated.zip");
    std::fs::write(&truncated, &bytes[..bytes.len() - 40]).unwrap();
    assert_eq!(failure_of(Extractor::new().list(&truncated)), Some(ArchiveFailure::Corrupt));
    assert_eq!(failure_of(bench.extract(&truncated)), Some(ArchiveFailure::Corrupt));

    let garbage = bench.path("garbage.zip");
    std::fs::write(&garbage, noise(5_000, 17)).unwrap();
    assert_eq!(failure_of(Extractor::new().list(&garbage)), Some(ArchiveFailure::Corrupt));
    assert_eq!(failure_of(bench.extract(&garbage)), Some(ArchiveFailure::Corrupt));
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn a_damaged_seven_z_is_reported_and_leaves_nothing_behind() {
    let bench = Bench::new();
    let good = bench.path("good.7z");
    let body = noise(300_000, 13);
    write_seven_z(&good, &[("m/first.txt", b"fine"), ("m/blob.bin", &body)]);
    let bytes = std::fs::read(&good).unwrap();

    let bad_data = bench.path("bad_data.7z");
    std::fs::write(&bad_data, flipped(&bytes, bytes.len() / 2)).unwrap();
    assert!(Extractor::new().list(&bad_data).is_ok());
    assert_eq!(failure_of(bench.extract(&bad_data)), Some(ArchiveFailure::Corrupt));
    assert_eq!(bench.leftovers(), Vec::<String>::new());

    let truncated = bench.path("truncated.7z");
    std::fs::write(&truncated, &bytes[..bytes.len() - 40]).unwrap();
    assert_eq!(failure_of(Extractor::new().list(&truncated)), Some(ArchiveFailure::Corrupt));
    assert_eq!(failure_of(bench.extract(&truncated)), Some(ArchiveFailure::Corrupt));

    let bad_header = bench.path("bad_header.7z");
    std::fs::write(&bad_header, flipped(&bytes, bytes.len() - 6)).unwrap();
    assert_eq!(failure_of(Extractor::new().list(&bad_header)), Some(ArchiveFailure::Corrupt));

    let garbage = bench.path("garbage.7z");
    std::fs::write(&garbage, noise(5_000, 17)).unwrap();
    assert_eq!(failure_of(Extractor::new().list(&garbage)), Some(ArchiveFailure::Corrupt));
    assert_eq!(failure_of(bench.extract(&garbage)), Some(ArchiveFailure::Corrupt));
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn a_damaged_rar_is_reported_and_leaves_nothing_behind() {
    let bench = Bench::new();
    let body = noise(300_000, 13);
    let bytes = raw_rar_bytes(&[RawRarEntry::file("m/first.txt", b"fine"), RawRarEntry::file("m/blob.bin", &body)]);

    let bad_data = bench.path("bad_data.rar");
    std::fs::write(&bad_data, flipped(&bytes, bytes.len() / 2)).unwrap();
    assert!(Extractor::new().list(&bad_data).is_ok());
    assert_eq!(failure_of(bench.extract(&bad_data)), Some(ArchiveFailure::Corrupt));
    assert_eq!(bench.leftovers(), Vec::<String>::new());

    let garbage = bench.path("garbage.rar");
    std::fs::write(&garbage, noise(5_000, 17)).unwrap();
    assert_eq!(failure_of(Extractor::new().list(&garbage)), Some(ArchiveFailure::Corrupt));
    assert_eq!(failure_of(bench.extract(&garbage)), Some(ArchiveFailure::Corrupt));
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn compressed_rar_samples_made_by_the_real_tool_extract() {
    let classic = Bench::new();
    assert_eq!(classic.extracted(&fixture("version.rar")), ["VERSION"]);
    assert_eq!(text(&classic.out(), "VERSION"), "unrar-0.4.0");
    let solid = Bench::new();
    assert_eq!(solid.extracted(&fixture("solid.rar")), [".gitignore"]);
    assert_eq!(text(&solid.out(), ".gitignore"), "target\nCargo.lock\n");
}

#[test]
fn a_real_volume_set_without_its_other_parts_is_refused() {
    let first = Bench::new();
    assert_eq!(
        failure_of(first.extract(&fixture("archive.part1.rar"))),
        Some(ArchiveFailure::MissingVolume)
    );
    assert_eq!(first.leftovers(), Vec::<String>::new());

    let later = Bench::new();
    let second_part = fixture("100M.part00002.rar");
    assert_eq!(failure_of(Extractor::new().list(&second_part)), Some(ArchiveFailure::MissingVolume));
    assert_eq!(failure_of(later.extract(&second_part)), Some(ArchiveFailure::MissingVolume));
    assert_eq!(later.leftovers(), Vec::<String>::new());
}

#[test]
fn a_missing_rar_volume_is_its_own_failure() {
    let bench = Bench::new();
    let archive = bench.path("set.part1.rar");
    write_raw_rar(
        &archive,
        &[
            RawRarEntry::file("m/whole.txt", b"whole"),
            RawRarEntry::file("m/split.bin", &noise(50_000, 21)).continued(),
        ],
    );
    assert_eq!(failure_of(bench.extract(&archive)), Some(ArchiveFailure::MissingVolume));
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn password_protected_archives_are_refused_before_anything_is_written() {
    let bench = Bench::new();
    let zip_crypto = bench.path("zipcrypto.zip");
    write_encrypted_zip(&zip_crypto, b"secret", &[("mod/readme.txt", b"hidden")]);
    let aes_shaped = bench.path("aes_shaped.zip");
    write_raw_zip(
        &aes_shaped,
        &[
            RawZipEntry::utf8("mod/open.txt", b"open"),
            RawZipEntry::utf8("mod/locked.bin", &noise(64, 23))
                .flags(ZIP_FLAG_ENCRYPTED)
                .method(ZIP_METHOD_AES)
                .extra(&ZIP_AES_EXTRA),
        ],
    );
    for (archive, lists) in [
        (zip_crypto, true),
        (aes_shaped, true),
        (fixture("enc_aes256.zip"), true),
        (fixture("enc_data.7z"), true),
        (fixture("enc_headers.7z"), false),
        (fixture("enc_data.rar"), true),
        (fixture("enc_headers.rar"), false),
    ] {
        let listed = Extractor::new().list(&archive);
        if lists {
            assert!(listed.is_ok_and(|listing| !listing.is_empty()), "{}", archive.display());
        } else {
            assert_eq!(failure_of(listed), Some(ArchiveFailure::PasswordRequired), "{}", archive.display());
        }
        let out = Bench::new();
        assert_eq!(
            failure_of(out.extract(&archive)),
            Some(ArchiveFailure::PasswordRequired),
            "{}",
            archive.display()
        );
        assert_eq!(out.leftovers(), Vec::<String>::new(), "{}", archive.display());
    }
}

#[test]
fn an_unsupported_zip_method_is_its_own_failure() {
    let bench = Bench::new();
    let archive = bench.path("shrunk.zip");
    write_raw_zip(
        &archive,
        &[
            RawZipEntry::utf8("mod/plain.txt", b"plain"),
            RawZipEntry::utf8("mod/shrunk.bin", &noise(64, 29)).method(ZIP_METHOD_SHRINK),
        ],
    );
    assert_eq!(Extractor::new().list(&archive).unwrap().len(), 2);
    assert_eq!(failure_of(bench.extract(&archive)), Some(ArchiveFailure::UnsupportedMethod));
    assert_eq!(bench.leftovers(), Vec::<String>::new());
}

#[test]
fn an_archive_without_entries_is_its_own_failure() {
    let bench = Bench::new();
    let archive = bench.path("empty.zip");
    write_raw_zip(&archive, &[]);
    assert_eq!(Extractor::new().list(&archive).unwrap(), Vec::new());
    assert_eq!(failure_of(bench.extract(&archive)), Some(ArchiveFailure::NoFiles));
}

#[test]
fn file_system_failures_stay_io_errors() {
    let bench = Bench::new();
    let missing = bench.path("missing.zip");
    let error = Extractor::new().list(&missing).unwrap_err();
    assert!(error.is_not_found(), "{error:?}");
    let error = bench.extract(&missing).unwrap_err();
    assert!(error.is_not_found(), "{error:?}");

    for (name, write) in [
        ("blocked.zip", write_zip as fn(&Path, &[(&str, &[u8])])),
        ("blocked.7z", write_seven_z),
        ("blocked.rar", write_rar),
    ] {
        let archive = bench.path(name);
        write(&archive, &[("first.txt", b"first"), ("mod/info.txt", b"PRJ_028")]);
        let out = Bench::new();
        std::fs::create_dir_all(out.out()).unwrap();
        std::fs::write(out.out().join("mod"), "a file where a folder has to go").unwrap();
        let error = out.extract(&archive).unwrap_err();
        assert!(matches!(error, Error::Io { .. }), "{name}: {error:?}");
        assert!(!error.is_disk_full());
        assert_eq!(out.leftovers(), ["mod"], "{name}");
    }
}

fn zip_extra(id: u16, body: &[u8]) -> Vec<u8> {
    let mut extra = id.to_le_bytes().to_vec();
    extra.extend_from_slice(&u16::try_from(body.len()).unwrap().to_le_bytes());
    extra.extend_from_slice(body);
    extra
}

#[test]
fn modification_times_come_from_the_archive() {
    use sevenz_rust2::{ArchiveEntry as SevenZEntry, ArchiveWriter, NtTime};

    let bench = Bench::new();
    let seconds: u32 = 1_589_722_662;
    let stamp = SystemTime::UNIX_EPOCH + Duration::from_secs(u64::from(seconds));
    let seven_z = bench.path("timed.7z");
    let mut writer = ArchiveWriter::create(&seven_z).unwrap();
    let mut timed = SevenZEntry::new_file("timed.txt");
    timed.has_last_modified_date = true;
    timed.last_modified_date = NtTime::try_from(stamp).unwrap();
    writer.push_archive_entry(timed, Some(&b"timed"[..])).unwrap();
    writer.finish().unwrap();
    bench.extract(&seven_z).unwrap();
    let modified = |root: &Path, name: &str| std::fs::metadata(root.join(name)).unwrap().modified().unwrap();
    assert_eq!(modified(&bench.out(), "timed.txt"), stamp);

    let mut unix_body = vec![ZIP_EXTENDED_TIMESTAMP_HAS_MTIME];
    unix_body.extend_from_slice(&seconds.to_le_bytes());
    let ticks = u64::from(NtTime::try_from(stamp).unwrap());
    let mut ntfs_body = vec![0; 4];
    ntfs_body.extend_from_slice(&ZIP_NTFS_TIMES_TAG.to_le_bytes());
    ntfs_body.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..3 {
        ntfs_body.extend_from_slice(&ticks.to_le_bytes());
    }
    let zip_path = bench.path("timed.zip");
    write_raw_zip(
        &zip_path,
        &[
            RawZipEntry::utf8("dos.txt", b"dos"),
            RawZipEntry::utf8("unix.txt", b"unix").extra(&zip_extra(ZIP_EXTENDED_TIMESTAMP, &unix_body)),
            RawZipEntry::utf8("ntfs.txt", b"ntfs").extra(&zip_extra(ZIP_NTFS_TIMES, &ntfs_body)),
        ],
    );
    let out = Bench::new();
    out.extract(&zip_path).unwrap();
let local = jiff::tz::TimeZone::system()
    .to_timestamp(jiff::civil::date(2024, 1, 1).at(0, 0, 0, 0))
    .unwrap();

assert_eq!(
    modified(&out.out(), "dos.txt"),
    SystemTime::try_from(local).unwrap()
);
assert_eq!(modified(&out.out(), "unix.txt"), stamp);
assert_eq!(modified(&out.out(), "ntfs.txt"), stamp);
}
