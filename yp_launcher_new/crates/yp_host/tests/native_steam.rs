mod support;

use std::path::PathBuf;

use support::FakeSteamTree;
use yp_host::wine::native_steam::{find_nier_in_native_steam, native_steam_libraries};

fn library_folders(paths: &[&str]) -> String {
    let mut text = String::from("\"libraryfolders\"\n{\n");
    for (index, path) in paths.iter().enumerate() {
        text.push_str(&format!("\t\"{index}\"\n\t{{\n\t\t\"path\"\t\t\"{path}\"\n\t}}\n"));
    }
    text.push_str("}\n");
    text
}

#[test]
fn libraries_include_the_root_and_its_vdf_entries() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    tree.add_library_folders(&library_folders(&[&root, "/mnt/games/SteamLibrary"]));
    let libraries = native_steam_libraries(&tree.native());
    assert!(libraries.contains(&root));
    assert!(libraries.contains(&"/mnt/games/SteamLibrary".to_owned()));
    assert_eq!(libraries.len(), 2);
}

#[test]
fn libraries_are_just_the_root_without_a_vdf() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    assert_eq!(native_steam_libraries(&tree.native()), [root]);
}

#[test]
fn finds_the_game_in_the_root_library() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let dir = tree.add_nier(None);
    let found = find_nier_in_native_steam(&tree.native());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, PathBuf::from(&dir));
    assert!(!found[0].has_data);
}

#[test]
fn finds_the_game_in_a_secondary_library_from_the_vdf() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let dir = tree.add_nier(Some(&root));
    tree.add_library_folders(&library_folders(&[&root]));
    let found = find_nier_in_native_steam(&tree.native());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, PathBuf::from(dir));
}

#[test]
fn finds_a_second_library_and_reports_data() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let library = yp_core::wine_paths::posix::join(&tree.home, "ssd/SteamLibrary");
    let dir = tree.add_nier(Some(&library));
    support::mkdirs(&format!("{dir}/data"));
    tree.add_library_folders(&library_folders(&[&root, &library]));
    let found = find_nier_in_native_steam(&tree.native());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, PathBuf::from(dir));
    assert!(found[0].has_data);
}

#[test]
fn reports_nothing_when_the_game_is_absent() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    assert!(find_nier_in_native_steam(&tree.native()).is_empty());
}

#[test]
fn default_roots_come_from_home() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let ctx = tree.plain().with_default_steam_roots();
    assert_eq!(ctx.steam_roots(), [root]);
}
