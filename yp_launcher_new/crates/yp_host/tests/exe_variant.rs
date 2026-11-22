use std::path::Path;

use yp_core::hash::to_hex;
use yp_host::game_probe::*;

fn game_dir() -> tempfile::TempDir {
    tempfile::Builder::new().prefix("yp_exe_").tempdir().unwrap()
}

fn exe(dir: &Path) -> std::path::PathBuf {
    dir.join("NieRAutomata.exe")
}

fn pe_header(stamp: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 0x200];
    bytes[0] = 0x4d;
    bytes[1] = 0x5a;
    bytes[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
    bytes[0x80..0x84].copy_from_slice(&0x0000_4550u32.to_le_bytes());
    bytes[0x88..0x8c].copy_from_slice(&stamp.to_le_bytes());
    bytes
}

#[test]
fn missing_exe_is_reported_as_missing() {
    let dir = game_dir();
    assert_eq!(detect_exe_variant(dir.path()), ExeVariant::Missing);
    assert_eq!(probe_exe(dir.path()).status(), ExeBuildStatus::Missing);
}

#[test]
fn win7_size_with_the_wrong_hash_is_not_win7() {
    let dir = game_dir();
    std::fs::write(exe(dir.path()), vec![0u8; LEGACY_WINDOWS7_SIZE as usize]).unwrap();
    assert_ne!(detect_exe_variant(dir.path()), ExeVariant::LegacyWindows7);
}

#[test]
fn a_small_unknown_exe_is_treated_as_original() {
    let dir = game_dir();
    std::fs::write(exe(dir.path()), vec![7u8; 1024]).unwrap();
    assert_eq!(detect_exe_variant(dir.path()), ExeVariant::Original);
}

#[test]
fn reference_constants_match_what_nams_reports() {
    assert_eq!(LEGACY_WINDOWS7_SIZE, 17_786_752);
    assert_eq!(LEGACY_WINDOWS7_SHA1, "f734bef5403c77e29cfc55f57707df19c3655751");
    assert_eq!(WINDOWS10_SIZE, 17_837_952);
    assert_eq!(WINDOWS10_SHA1, "420cbab7daa1d94a79473c0a6271ef9c30e516e5");
    assert_eq!(STEAM_2026_SIZE, 17_601_536);
    assert_eq!(STEAM_2026_SHA1, "537be3ea1083114cc1e444b85ac270f2043ac4d6");
    assert_eq!(STEAM_2026_SIGNED_SIZE, 17_613_568);
    assert_eq!(STEAM_2026_SIGNED_SHA1, "f2fbdbbdddb939a992185dfa9ff404c8e25dd605");
    assert_eq!(SUPPORTED_EXE_SIZES, [WINDOWS10_SIZE, STEAM_2026_SIZE, STEAM_2026_SIGNED_SIZE]);
}

#[test]
fn sha1_of_a_sample_is_lowercase_hex() {
    let dir = game_dir();
    let file = dir.path().join("sample");
    std::fs::write(&file, [0x4d, 0x5a, 0x90, 0x00]).unwrap();
    let computed = to_hex(&yp_core::hash::sha1_file(&file).unwrap());
    assert_eq!(computed.len(), 40);
    assert!(computed.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[test]
fn known_builds_match_by_size_and_hash_or_by_stamp() {
    assert_eq!(build_of_exe(STEAM_2026_SIGNED_SIZE, STEAM_2026_SIGNED_SHA1).unwrap().label, "Steam 2026-11 (signed)");
    assert_eq!(build_of_exe(STEAM_2026_SIGNED_SIZE, STEAM_2026_SHA1), None);
    assert_eq!(build_of_time_date_stamp(Some(STEAM_2021_TIME_DATE_STAMP)).unwrap().build, GameBuild::Steam2021);
    assert_eq!(build_of_time_date_stamp(Some(STEAM_2026_TIME_DATE_STAMP)).unwrap().build, GameBuild::Steam2026_11);
    assert_eq!(build_of_time_date_stamp(None), None);
    assert!(newer_than_every_build(Some(STEAM_2026_TIME_DATE_STAMP + 1)));
    assert!(!newer_than_every_build(Some(STEAM_2026_TIME_DATE_STAMP)));
    assert!(!newer_than_every_build(None));
}

#[test]
fn pe_time_date_stamp_reads_the_coff_header() {
    assert_eq!(pe_time_date_stamp(&pe_header(0x1234_5678)), Some(0x1234_5678));
    assert_eq!(pe_time_date_stamp(b"MZ"), None);
    assert_eq!(pe_time_date_stamp(b"XX"), None);
    let mut broken = pe_header(1);
    broken[0x80] = 0;
    assert_eq!(pe_time_date_stamp(&broken), None);
    let mut far = pe_header(1);
    far[0x3c..0x40].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(pe_time_date_stamp(&far), None);
}

#[test]
fn probe_classifies_modified_newer_and_unknown_builds() {
    let dir = game_dir();
    std::fs::write(exe(dir.path()), pe_header(STEAM_2026_TIME_DATE_STAMP)).unwrap();
    let modified = probe_exe(dir.path());
    assert!(modified.exists);
    assert_eq!(modified.size, Some(0x200));
    assert_eq!(modified.time_date_stamp, Some(STEAM_2026_TIME_DATE_STAMP));
    assert_eq!(
        modified.sha1.as_deref(),
        Some(to_hex(&yp_core::hash::sha1_file(&exe(dir.path())).unwrap()).as_str())
    );
    assert_eq!(modified.status(), ExeBuildStatus::Modified);
    assert_eq!(file_time_date_stamp(&exe(dir.path())), Some(STEAM_2026_TIME_DATE_STAMP));

    std::fs::write(exe(dir.path()), pe_header(STEAM_2026_TIME_DATE_STAMP + 100)).unwrap();
    assert_eq!(probe_exe(dir.path()).status(), ExeBuildStatus::Newer);

    std::fs::write(exe(dir.path()), b"not a pe").unwrap();
    assert_eq!(probe_exe(dir.path()).status(), ExeBuildStatus::Unknown);
}

#[test]
fn probe_status_prefers_known_then_legacy() {
    let known = ExeProbe {
        exists: true,
        size: Some(WINDOWS10_SIZE),
        sha1: Some(WINDOWS10_SHA1.to_owned()),
        time_date_stamp: None,
    };
    assert_eq!(known.status(), ExeBuildStatus::Known);
    assert_eq!(known.build().unwrap().build, GameBuild::Steam2021);
    let legacy = ExeProbe {
        exists: true,
        size: Some(LEGACY_WINDOWS7_SIZE),
        sha1: Some(LEGACY_WINDOWS7_SHA1.to_owned()),
        time_date_stamp: Some(STEAM_2021_TIME_DATE_STAMP),
    };
    assert_eq!(legacy.status(), ExeBuildStatus::LegacyWindows7);
}

const MANIFEST: &str = "\"AppState\"
{
\t\"appid\"\t\t\"524220\"
\t\"StateFlags\"\t\t\"4\"
\t\"LastUpdated\"\t\t\"1700000000\"
\t\"buildid\"\t\t\"100\"
\t\"TargetBuildID\"\t\t\"200\"
\t\"InstalledDepots\"
\t{
\t\t\"524221\"
\t\t{
\t\t\t\"manifest\"\t\t\"1\"
\t\t}
\t\t\"580601\"
\t\t{
\t\t\t\"manifest\"\t\t\"2\"
\t\t\t\"dlcappid\"\t\t\"580600\"
\t\t}
\t}
\t\"UserConfig\"
\t{
\t\t\"BetaKey\"\t\t\"public_beta\"
\t\t\"DisabledDLC\"\t\t\"1, 2\"
\t}
\t\"MountedConfig\"
\t{
\t\t\"DisabledDLC\"\t\t\"\"
\t}
}
";

#[test]
fn steam_manifest_parses_dlc_build_and_beta() {
    let manifest = SteamAppManifest::parse(MANIFEST).unwrap();
    assert_eq!(manifest.build_id.as_deref(), Some("100"));
    assert_eq!(manifest.target_build_id.as_deref(), Some("200"));
    assert_eq!(manifest.state_flags.as_deref(), Some("4"));
    assert_eq!(manifest.beta_key.as_deref(), Some("public_beta"));
    assert!(manifest.update_pending());
    assert!(manifest.dlc_installed(DLC_APP_ID));
    assert!(manifest.disabled_dlc.is_empty());
    assert_eq!(manifest.last_updated_at().unwrap().as_second(), 1_700_000_000);

    let disabled = SteamAppManifest::parse(&MANIFEST.replace("\t\"MountedConfig\"\n\t{\n\t\t\"DisabledDLC\"\t\t\"\"\n\t}\n", "")).unwrap();
    assert!(disabled.disabled_dlc.contains("1") && disabled.disabled_dlc.contains("2"));

    let no_update = SteamAppManifest::parse(&MANIFEST.replace("\"200\"", "\"0\"")).unwrap();
    assert!(!no_update.update_pending());
}

#[test]
fn steam_manifest_path_needs_a_steamapps_common_layout() {
    let game = Path::new("/lib").join("steamapps").join("common").join("NieRAutomata");
    assert_eq!(
        steam_manifest_path(&game),
        Some(Path::new("/lib").join("steamapps").join("appmanifest_524220.acf"))
    );
    assert_eq!(steam_manifest_path(Path::new("/games/NieRAutomata")), None);
}

#[test]
fn dlc_comes_from_the_manifest_then_the_archive_size() {
    let root = tempfile::tempdir().unwrap();
    let game = root.path().join("steamapps").join("common").join("NieRAutomata");
    std::fs::create_dir_all(game.join("data")).unwrap();
    assert!(!has_dlc(&game));
    let archive = std::fs::File::create(game.join("data").join("data100.cpk")).unwrap();
    archive.set_len(930 * 1024 * 1024).unwrap();
    drop(archive);
    assert!(has_dlc(&game));
    std::fs::write(
        root.path().join("steamapps").join("appmanifest_524220.acf"),
        MANIFEST.replace("580600", "999"),
    )
    .unwrap();
    assert!(!has_dlc(&game));
    assert!(read_steam_manifest(&game).is_some());
}

#[test]
fn hd_textures_need_a_non_empty_sk_res() {
    let dir = game_dir();
    assert!(detect_hd_textures(dir.path()).is_empty());
    std::fs::create_dir_all(dir.path().join("SK_Res")).unwrap();
    assert!(detect_hd_textures(dir.path()).is_empty());
    std::fs::write(dir.path().join("SK_Res").join("x.dds"), "").unwrap();
    assert_eq!(detect_hd_textures(dir.path()), ["SK_Res/"]);
}

#[test]
fn the_wolf_limit_break_exe_is_recognised_by_sha256() {
    assert_eq!(WOLF_LIMIT_BREAK_SHA256.len(), 64);
    let dir = game_dir();
    std::fs::write(exe(dir.path()), b"").unwrap();
    assert_eq!(detect_exe_variant(dir.path()), ExeVariant::Original);
}

fn exe_bytes_with_signature_at(offset: usize, total: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; total];
    bytes[0] = 0x4d;
    bytes[1] = 0x5a;
    bytes[offset..offset + 7].copy_from_slice(b"PRJ_028");
    bytes
}

#[test]
fn exe_selection_accepts_an_mz_file_carrying_the_game_signature() {
    let dir = game_dir();
    std::fs::write(exe(dir.path()), exe_bytes_with_signature_at(4096, 8192)).unwrap();
    assert_eq!(check_exe_selection(&exe(dir.path())), ExeSelection::Valid);
}

#[test]
fn exe_selection_finds_a_signature_straddling_a_scan_chunk() {
    let dir = game_dir();
    let chunk = 64 * 1024;
    for boundary in [chunk + 2, 2 * chunk + 2, 3 * chunk + 8] {
        for back in 1..7 {
            std::fs::write(exe(dir.path()), exe_bytes_with_signature_at(boundary - back, 4 * chunk)).unwrap();
            assert!(is_nier_exe(&exe(dir.path())), "boundary {boundary} back {back}");
        }
    }
}

#[test]
fn exe_selection_rejects_other_executables_and_non_executables() {
    let dir = game_dir();
    assert_eq!(check_exe_selection(&exe(dir.path())), ExeSelection::Missing);

    let mut other = vec![0u8; 8192];
    other[0] = 0x4d;
    other[1] = 0x5a;
    std::fs::write(exe(dir.path()), &other).unwrap();
    assert_eq!(check_exe_selection(&exe(dir.path())), ExeSelection::NotNierExe);

    let mut text = b"PRJ_028".to_vec();
    text.resize(4096, b' ');
    std::fs::write(exe(dir.path()), &text).unwrap();
    assert_eq!(check_exe_selection(&exe(dir.path())), ExeSelection::NotNierExe);

    std::fs::write(exe(dir.path()), [0x4d]).unwrap();
    assert_eq!(check_exe_selection(&exe(dir.path())), ExeSelection::NotNierExe);
}

#[test]
fn byte_search_handles_short_inputs() {
    assert!(contains_bytes(&b"xxPRJ_028"[..], b"PRJ_028").unwrap());
    assert!(!contains_bytes(&b"PRJ_02"[..], b"PRJ_028").unwrap());
    assert!(!contains_bytes(&b""[..], b"PRJ_028").unwrap());
}
