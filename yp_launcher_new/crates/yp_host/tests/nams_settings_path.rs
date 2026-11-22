mod support;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use support::FakeBottleTree;
use yp_core::testing::FakeRunner;
use yp_core::wine_paths::posix;
use yp_core::{Os, PlatformAdapter};
use yp_host::platform::adapter_for;
use yp_host::platform::macos::macos_adapter_with;
use yp_host::platform::nams_settings_path;

fn expected(game_dir: &Path) -> PathBuf {
    game_dir.join("nams").join("_internal").join("cache").join("settings.json")
}

#[test]
fn settings_land_in_the_game_dir_cache_where_nams_reads_them() {
    let tree = FakeBottleTree::create();
    let bottle = tree.add_bottle("Steam");
    let game_dir = PathBuf::from(posix::join_all(&bottle, &["drive_c", "game"]));
    let adapter = macos_adapter_with(tree.ctx(Os::MacOs, None), Arc::new(FakeRunner::new()));
    assert_eq!(adapter.resolve_nams_settings_path(Some(&game_dir)), Some(expected(&game_dir)));
}

#[test]
fn without_a_game_dir_there_is_no_location_to_write() {
    assert_eq!(nams_settings_path(None), None);
    assert_eq!(nams_settings_path(Some(Path::new(""))), None);
    for os in [Os::Windows, Os::Linux, Os::MacOs] {
        let adapter = adapter_for(support::host_for(os, &[], "/r"), Arc::new(FakeRunner::new()));
        assert_eq!(adapter.resolve_nams_settings_path(None), None);
    }
}

#[test]
fn a_game_dir_outside_any_prefix_resolves_next_to_the_game_too() {
    let tree = FakeBottleTree::create();
    let game_dir = PathBuf::from(posix::join_all(&tree.root, &["Games", "NieRAutomata"]));
    for os in [Os::Windows, Os::Linux, Os::MacOs] {
        let adapter = adapter_for(support::host_for(os, &[], &tree.root), Arc::new(FakeRunner::new()));
        assert_eq!(adapter.resolve_nams_settings_path(Some(&game_dir)), Some(expected(&game_dir)));
    }
}
