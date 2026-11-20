mod support;

use std::path::PathBuf;

use support::FakeBottleTree;
use yp_core::wine_paths::posix;
use yp_core::Os;
use yp_host::vdf::path_entries;
use yp_host::wine::prefix_steam::{
    find_nier_in_prefixes, nier_dirs_in_prefix, normalize_steam_library_path, parse_library_folders_in_prefix,
    windows_drive_letter,
};

const REAL_VDF: &str = "\"libraryfolders\"
{
\t\"0\"
\t{
\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"
\t\t\"label\"\t\t\"\"
\t\t\"apps\"
\t\t{
\t\t\t\"228980\"\t\t\"102931551\"
\t\t\t\"524220\"\t\t\"43881494773\"
\t\t}
\t}
}
";

#[test]
fn c_drive_resolves_to_drive_c() {
    assert_eq!(
        normalize_steam_library_path(r"C:\Program Files (x86)\Steam", "/b/Steam"),
        "/b/Steam/drive_c/Program Files (x86)/Steam"
    );
}

#[test]
fn another_drive_resolves_through_dosdevices() {
    assert_eq!(normalize_steam_library_path(r"D:\SteamLibrary", "/b/Steam"), "/b/Steam/dosdevices/d:/SteamLibrary");
}

#[test]
fn drive_letter_is_matched_case_insensitively() {
    assert_eq!(normalize_steam_library_path(r"c:\Games", "/b/Steam"), "/b/Steam/drive_c/Games");
    assert_eq!(windows_drive_letter(r"E:\x"), Some('e'));
    assert_eq!(windows_drive_letter("E:/x"), None);
}

#[test]
fn a_host_path_is_returned_untouched() {
    assert_eq!(normalize_steam_library_path("/mnt/games/SteamLibrary", "/b/Steam"), "/mnt/games/SteamLibrary");
}

#[test]
fn escaped_quotes_are_unescaped_by_the_vdf_reader() {
    let library = path_entries(r#""path" "C:\\a\"b""#);
    assert_eq!(normalize_steam_library_path(&library[0], "/b/Steam"), "/b/Steam/drive_c/a\"b");
}

#[test]
fn reads_the_real_world_vdf_shape() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    tree.add_library_folders("Steam", REAL_VDF);
    assert_eq!(
        parse_library_folders_in_prefix(&bottle),
        [posix::join_all(&bottle, &["drive_c", "Program Files (x86)", "Steam"])]
    );
}

#[test]
fn a_prefix_without_a_vdf_yields_nothing() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    assert!(parse_library_folders_in_prefix(&bottle).is_empty());
}

#[test]
fn finds_the_game_at_the_canonical_steam_location() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let exe = tree.add_nier("Steam");
    assert_eq!(nier_dirs_in_prefix(&bottle), [posix::dirname(&exe)]);
}

#[test]
fn an_empty_prefix_yields_nothing() {
    let tree = FakeBottleTree::create();
    assert!(nier_dirs_in_prefix(&tree.add_bottle("Steam")).is_empty());
}

#[test]
fn a_dir_found_by_both_probes_is_reported_once() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let exe = tree.add_nier("Steam");
    tree.add_library_folders("Steam", REAL_VDF);
    assert_eq!(nier_dirs_in_prefix(&bottle), [posix::dirname(&exe)]);
}

#[cfg(unix)]
#[test]
fn a_library_on_another_drive_is_found_through_dosdevices() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    tree.add_library_folders("Steam", "\"libraryfolders\" { \"1\" { \"path\" \"D:\\\\Games\" } }");
    let dir = posix::join_all(&bottle, &["dosdevices", "d:", "Games", "steamapps", "common", "NieRAutomata"]);
    support::write(&posix::join(&dir, "NieRAutomata.exe"), "PRJ_028");
    assert_eq!(nier_dirs_in_prefix(&bottle), [dir]);
}

#[test]
fn reports_the_install_and_whether_data_exists() {
    let tree = FakeBottleTree::create();
    tree.add_bottle("Steam");
    let exe = tree.add_nier("Steam");
    let found = find_nier_in_prefixes(&tree.ctx(Os::MacOs, None));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, PathBuf::from(posix::dirname(&exe)));
    assert!(!found[0].has_data);
}

#[test]
fn bottles_without_the_game_are_skipped() {
    let tree = FakeBottleTree::create();
    tree.add_bottle("Empty");
    assert!(find_nier_in_prefixes(&tree.ctx(Os::MacOs, None)).is_empty());
}

#[cfg(unix)]
#[test]
fn deep_scan_roots_follow_dosdevices_links() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let target = support::mkdirs(&posix::join(&tree.root, "external"));
    let devices = support::mkdirs(&posix::join(&bottle, "dosdevices"));
    std::os::unix::fs::symlink(&target, posix::join(&devices, "d:")).unwrap();
    std::os::unix::fs::symlink("/", posix::join(&devices, "z:")).unwrap();
    let roots = yp_host::wine::prefix_steam::deep_scan_roots(&tree.ctx(Os::Linux, None));
    let canonical = yp_core::wine_paths::to_posix(&std::fs::canonicalize(&target).unwrap());
    assert_eq!(roots, [posix::join(&bottle, "drive_c"), canonical]);
}
