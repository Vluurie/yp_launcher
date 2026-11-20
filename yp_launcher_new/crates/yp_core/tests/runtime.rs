use std::path::{Path, PathBuf};

use yp_core::consts::{APP_VERSION, NAMS_EXE_NAME, YORHA_DLL_NAME};
use yp_core::hash::sha256_file;
use yp_core::runtime::{bundled_assets, stamp_for, RuntimeSetup, RuntimeStatus};
use yp_core::{AppPaths, HostEnv, Os};

fn workspace_bins() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/bins")
}

const BUNDLED_PROGRAMS: [&str; 1] = [NAMS_EXE_NAME];
const PROGRAM_SUFFIXES: [&str; 2] = [".exe", ".dll"];

#[test]
fn the_runtime_binaries_are_bundled_under_assets_bins() {
    let assets = bundled_assets(&workspace_bins()).unwrap();
    assert!(assets.contains(&NAMS_EXE_NAME.to_owned()), "{assets:?}");
    assert!(assets.contains(&format!("plugins/{YORHA_DLL_NAME}")), "{assets:?}");
    let programs: Vec<&str> = assets
        .iter()
        .map(String::as_str)
        .filter(|asset| !asset.contains('/') && PROGRAM_SUFFIXES.iter().any(|suffix| asset.ends_with(suffix)))
        .collect();
    assert_eq!(programs, BUNDLED_PROGRAMS, "only these programs ship next to the launcher");
    let mut sorted = assets.clone();
    sorted.sort();
    assert_eq!(assets, sorted);
}

fn bundle(root: &Path, files: &[(&str, &[u8])]) -> AppPaths {
    let paths = AppPaths::for_test(root);
    for (relative, bytes) in files {
        let path = relative.split('/').fold(paths.bundled_bins_dir.clone(), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }
    paths
}

#[test]
fn extracting_a_bundled_binary_reproduces_it_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_test(dir.path());
    let plugin = format!("plugins/{YORHA_DLL_NAME}");
    let source = workspace_bins().join("plugins").join(YORHA_DLL_NAME);
    assert!(source.is_file(), "{}", source.display());
    std::fs::create_dir_all(paths.bundled_bins_dir.join("plugins")).unwrap();
    std::fs::copy(&source, paths.bundled_bins_dir.join("plugins").join(YORHA_DLL_NAME)).unwrap();
    let host = HostEnv::for_test(Os::Linux, &[], dir.path());
    let status = RuntimeSetup::new(&paths, &host).ensure_ready().unwrap();
    assert_eq!(status, RuntimeStatus::Extracted { files: 1 });
    assert_eq!(bundled_assets(&paths.bundled_bins_dir).unwrap(), vec![plugin]);
    assert_eq!(
        sha256_file(&source).unwrap(),
        sha256_file(&paths.runtime_dir.join("plugins").join(YORHA_DLL_NAME)).unwrap()
    );
}

#[test]
fn extraction_is_stamped_and_skipped_until_the_asset_list_changes() {
    let dir = tempfile::tempdir().unwrap();
    let paths = bundle(
        dir.path(),
        &[
            ("NAMS.exe", b"nams"),
            ("plugins/yorha_protocol.dll", b"dll"),
            (".DS_Store", b"finder"),
        ],
    );
    let host = HostEnv::for_test(Os::Linux, &[], dir.path());
    let setup = RuntimeSetup::new(&paths, &host);
    assert_eq!(setup.missing_files(), vec![NAMS_EXE_NAME, YORHA_DLL_NAME]);

    assert_eq!(setup.ensure_ready().unwrap(), RuntimeStatus::Extracted { files: 2 });
    assert!(setup.missing_files().is_empty());
    assert!(!paths.runtime_dir.join(".DS_Store").exists());
    let stamp = std::fs::read_to_string(paths.runtime_stamp()).unwrap();
    let expected = stamp_for(&["NAMS.exe".to_owned(), "plugins/yorha_protocol.dll".to_owned()]);
    assert_eq!(stamp, expected);
    assert!(stamp.starts_with(&format!("{APP_VERSION}|")));

    std::fs::write(paths.runtime_dir.join("NAMS.exe"), "locally changed").unwrap();
    assert_eq!(setup.ensure_ready().unwrap(), RuntimeStatus::UpToDate);
    assert_eq!(std::fs::read_to_string(paths.runtime_dir.join("NAMS.exe")).unwrap(), "locally changed");

    std::fs::write(paths.bundled_bins_dir.join("plugins").join("extra.dll"), "x").unwrap();
    assert_eq!(setup.ensure_ready().unwrap(), RuntimeStatus::Extracted { files: 3 });
    assert_eq!(std::fs::read_to_string(paths.runtime_dir.join("NAMS.exe")).unwrap(), "nams");
    assert!(setup.is_writable());
}

#[test]
fn windows_runs_from_the_bundle_and_sweeps_stale_temp_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let assets = dir.path().join("assets");
    std::fs::create_dir_all(assets.join("bins")).unwrap();
    let host = HostEnv::for_test(
        Os::Windows,
        &[("APPDATA", &dir.path().join("appdata").to_string_lossy()), ("YP_ASSETS_DIR", &assets.to_string_lossy())],
        dir.path(),
    );
    let paths = AppPaths::detect(&host).unwrap();
    std::fs::create_dir_all(host.temp_dir().join("archive_123")).unwrap();
    std::fs::create_dir_all(host.temp_dir().join("texture_9")).unwrap();
    std::fs::create_dir_all(host.temp_dir().join("keep_me")).unwrap();
    assert_eq!(RuntimeSetup::new(&paths, &host).ensure_ready().unwrap(), RuntimeStatus::Bundled);
    assert!(!host.temp_dir().join("archive_123").exists());
    assert!(!host.temp_dir().join("texture_9").exists());
    assert!(host.temp_dir().join("keep_me").exists());
}

#[test]
fn an_empty_bundle_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::for_test(dir.path());
    let host = HostEnv::for_test(Os::Linux, &[], dir.path());
    assert_eq!(RuntimeSetup::new(&paths, &host).ensure_ready().unwrap(), RuntimeStatus::NoBundledAssets);
}
