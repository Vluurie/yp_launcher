mod support;

use std::collections::BTreeSet;

use support::FakeBottleTree;
use yp_core::wine_paths::posix;
use yp_core::Os;
use yp_host::wine::crossover::{
    bottle_from_game_path, bottle_roots, find_crossover_wine, list_bottles, nier_candidates_in_bottle, resolve_bottle,
};
use yp_host::wine::WineContext;

const BOTTLE_EXE: &str = "/u/Library/Application Support/CrossOver/Bottles/Steam/drive_c/Program Files (x86)/Steam/steamapps/common/NieRAutomata/NieRAutomata.exe";

#[test]
fn derives_name_and_path_from_a_bottle_internal_exe() {
    let bottle = bottle_from_game_path(BOTTLE_EXE).unwrap();
    assert_eq!(bottle.name, "Steam");
    assert_eq!(bottle.path, "/u/Library/Application Support/CrossOver/Bottles/Steam");
    assert_eq!(bottle.game_path.as_deref(), Some(BOTTLE_EXE));
}

#[test]
fn rejects_an_exe_outside_any_prefix() {
    assert!(bottle_from_game_path("/Users/d/Games/NieRAutomata.exe").is_none());
}

#[test]
fn rejects_a_prefix_that_is_not_crossover() {
    assert!(
        bottle_from_game_path("/home/d/.steam/steam/steamapps/compatdata/524220/pfx/drive_c/game/NieRAutomata.exe")
            .is_none()
    );
}

#[test]
fn lists_dirs_that_contain_drive_c() {
    let tree = FakeBottleTree::create();
    tree.add_bottle("Steam");
    tree.add_bottle("Other");
    let names: BTreeSet<String> = list_bottles(&tree.ctx(Os::MacOs, None)).into_iter().map(|b| b.name).collect();
    assert_eq!(names, BTreeSet::from(["Steam".to_owned(), "Other".to_owned()]));
}

#[test]
fn ignores_dirs_without_drive_c() {
    let tree = FakeBottleTree::create();
    tree.add_bottle("Steam");
    support::mkdirs(&posix::join(&tree.root, "NotABottle"));
    let names: Vec<String> = list_bottles(&tree.ctx(Os::MacOs, None)).into_iter().map(|b| b.name).collect();
    assert_eq!(names, ["Steam"]);
}

#[test]
fn an_empty_root_yields_no_bottles() {
    let tree = FakeBottleTree::create();
    assert!(list_bottles(&tree.ctx(Os::MacOs, None)).is_empty());
}

#[test]
fn probes_both_program_files_variants() {
    assert_eq!(
        nier_candidates_in_bottle("/b/Steam"),
        [
            "/b/Steam/drive_c/Program Files (x86)/Steam/steamapps/common/NieRAutomata/NieRAutomata.exe",
            "/b/Steam/drive_c/Program Files/Steam/steamapps/common/NieRAutomata/NieRAutomata.exe",
        ]
    );
}

#[test]
fn honours_the_wine_override_when_the_file_exists() {
    let tree = FakeBottleTree::create();
    let script = tree.add_wine_script();
    assert_eq!(find_crossover_wine(&tree.ctx(Os::MacOs, Some(script.clone()))), Some(script));
}

#[test]
fn a_missing_override_resolves_to_nothing() {
    let tree = FakeBottleTree::create();
    let missing = posix::join(&tree.root, "does-not-exist");
    tree.add_wine_script();
    let ctx = tree
        .ctx(Os::MacOs, Some(missing))
        .with_crossover_wine_candidates(vec![posix::join(&tree.root, "fake-wine")]);
    assert_eq!(find_crossover_wine(&ctx), None);
}

#[test]
fn falls_back_to_the_known_wine_locations() {
    let tree = FakeBottleTree::create();
    let script = tree.add_wine_script();
    let ctx = tree
        .ctx(Os::MacOs, None)
        .with_crossover_wine_candidates(vec![posix::join(&tree.root, "nope"), script.clone()]);
    assert_eq!(find_crossover_wine(&ctx), Some(script));
}

#[test]
fn bottle_roots_follow_the_host_conventions() {
    let tree = FakeBottleTree::create();
    let home = posix::join(&tree.root, "home");
    let mac_root = support::mkdirs(&posix::join_all(&home, &["Library", "Application Support", "CrossOver", "Bottles"]));
    let linux_root = support::mkdirs(&posix::join(&home, ".cxoffice"));
    let mac = WineContext::new(support::host_for(Os::MacOs, &[("HOME", &home)], &tree.root));
    assert_eq!(bottle_roots(&mac), [mac_root]);
    let linux = WineContext::new(support::host_for(Os::Linux, &[("HOME", &home)], &tree.root));
    assert_eq!(bottle_roots(&linux), [linux_root]);
    let configured = WineContext::new(support::host_for(Os::Linux, &[("HOME", &home)], &tree.root))
        .with_bottle_roots(vec![String::new(), tree.root.clone(), posix::join(&tree.root, "missing")]);
    assert_eq!(bottle_roots(&configured), std::slice::from_ref(&tree.root));
}

#[test]
fn resolves_the_bottle_named_by_cx_bottle_and_finds_its_game() {
    let tree = FakeBottleTree::create();
    tree.add_bottle("Steam");
    tree.add_bottle("Games");
    let exe = tree.add_nier("Games");
    let host = support::host_for(Os::MacOs, &[("CX_BOTTLE", "games")], &tree.root);
    let ctx = WineContext::isolated(host).with_bottle_roots(vec![tree.root.clone()]);
    let bottle = resolve_bottle(&ctx, "/elsewhere/NieRAutomata.exe", false).unwrap();
    assert_eq!(bottle.name, "Games");
    assert_eq!(bottle.game_path.as_deref(), Some(exe.as_str()));
}

#[test]
fn resolves_the_single_bottle_that_holds_the_game() {
    let tree = FakeBottleTree::create();
    tree.add_bottle("Steam");
    tree.add_bottle("Empty");
    let exe = tree.add_nier("Steam");
    let ctx = tree.ctx(Os::MacOs, None);
    let bottle = resolve_bottle(&ctx, "/elsewhere/NieRAutomata.exe", true).unwrap();
    assert_eq!(bottle.name, "Steam");
    assert_eq!(bottle.game_path.as_deref(), Some(exe.as_str()));
    assert!(resolve_bottle(&ctx, "/elsewhere/NieRAutomata.exe", false).is_none());
    tree.add_nier("Empty");
    assert!(resolve_bottle(&ctx, "/elsewhere/NieRAutomata.exe", true).is_none());
}
