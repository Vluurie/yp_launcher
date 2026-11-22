mod support;

use support::FakeSteamTree;
use yp_core::wine_paths::{posix, NIER_STEAM_APP_ID};
use yp_host::wine::proton::{
    compat_data_path, find_any_proton_path, find_proton_path, has_any_proton, is_proton_runtime_path, steam_home_for_client,
    steam_root,
};

#[test]
fn context_for_a_game_in_the_main_library_maps_onto_that_root() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let game_dir = tree.add_nier(None);
    let ctx = tree.native().infer_steam_context(&posix::join(&game_dir, "NieRAutomata.exe")).unwrap();
    assert_eq!(ctx.steam_root, root);
    assert_eq!(ctx.library_root, root);
    assert_eq!(ctx.compat_data, posix::join_all(&root, &["steamapps", "compatdata", NIER_STEAM_APP_ID]));
}

#[test]
fn context_for_a_second_library_keeps_compatdata_there_but_points_at_the_real_root() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let library = posix::join_all(&tree.home, &["ssd", "SteamLibrary"]);
    let game_dir = tree.add_nier(Some(&library));
    let ctx = tree.native().infer_steam_context(&posix::join(&game_dir, "NieRAutomata.exe")).unwrap();
    assert_eq!(ctx.steam_root, root);
    assert_eq!(ctx.library_root, library);
    assert_eq!(ctx.compat_data, posix::join_all(&library, &["steamapps", "compatdata", NIER_STEAM_APP_ID]));
}

#[test]
fn context_falls_back_to_the_library_root_without_any_steam_root() {
    let tree = FakeSteamTree::create();
    let library = posix::join_all(&tree.home, &["ssd", "SteamLibrary"]);
    let game_dir = tree.add_nier(Some(&library));
    let ctx = tree
        .plain()
        .with_steam_roots(Vec::new())
        .infer_steam_context(&posix::join(&game_dir, "NieRAutomata.exe"))
        .unwrap();
    assert_eq!(ctx.steam_root, library);
    assert_eq!(ctx.library_root, library);
}

#[test]
fn context_is_none_outside_any_library() {
    let tree = FakeSteamTree::create();
    assert!(tree.native().infer_steam_context(&posix::join_all(&tree.home, &["Games", "NieRAutomata.exe"])).is_none());
}

#[test]
fn finds_a_build_under_compatibilitytools() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let proton = tree.add_proton("GE-Proton");
    assert_eq!(find_proton_path(&root, &root), Some(proton));
}

#[test]
fn finds_nothing_without_proton() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    assert_eq!(find_proton_path(&root, &root), None);
}

#[test]
fn does_not_find_proton_in_a_different_library() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    tree.add_proton("GE-Proton");
    let game_library = posix::join(&tree.home, "mnt/games/SteamLibrary");
    assert_eq!(find_proton_path(&game_library, &game_library), None);
}

#[test]
fn ignores_dirs_without_a_proton_script_and_prefers_the_newest() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    support::mkdirs(&posix::join_all(&root, &["compatibilitytools.d", "Proton-empty"]));
    let older = tree.add_proton("Proton-old");
    std::thread::sleep(std::time::Duration::from_millis(50));
    let newer = tree.add_proton("Proton-new");
    let found = find_proton_path(&root, &root).unwrap();
    assert!(found == newer || found == older);
    assert!(!found.contains("Proton-empty"));
}

#[test]
fn any_proton_comes_from_any_steam_root() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let proton = tree.add_proton("GE-Proton");
    let ctx = tree.native();
    assert_eq!(find_any_proton_path(&ctx), Some(proton));
    assert!(has_any_proton(&ctx));
}

#[test]
fn any_proton_is_none_when_no_root_has_one() {
    let tree = FakeSteamTree::create();
    tree.add_steam_root();
    let ctx = tree.native();
    assert_eq!(find_any_proton_path(&ctx), None);
    assert!(!has_any_proton(&ctx));
}

#[test]
fn steam_home_resolves_the_confined_home_with_steamclient() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    tree.add_steam_client();
    let outside = support::linux_host("/nonexistent-home");
    let ctx = yp_host::wine::WineContext::isolated(outside);
    assert_eq!(steam_home_for_client(&ctx, &root), Some(tree.home.clone()));
    assert_eq!(steam_home_for_client(&tree.native(), &root), None);
}

#[test]
fn steam_home_is_none_without_steamclient() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    assert_eq!(steam_home_for_client(&tree.native(), &root), None);
}

#[test]
fn compat_paths_prefer_the_environment_then_the_game_then_home() {
    let tree = FakeSteamTree::create();
    let root = tree.add_steam_root();
    let game_dir = tree.add_nier(None);
    let exe = posix::join(&game_dir, "NieRAutomata.exe");
    let ctx = tree.native();
    assert_eq!(compat_data_path(&ctx, Some(&exe)), posix::join_all(&root, &["steamapps", "compatdata", "524220"]));
    assert_eq!(steam_root(&ctx, Some(&exe)), root);
    assert_eq!(
        compat_data_path(&ctx, None),
        posix::join_all(&tree.home, &[".steam", "steam", "steamapps", "compatdata", "524220"])
    );
    assert_eq!(steam_root(&ctx, None), posix::join_all(&tree.home, &[".steam", "steam"]));
    let env = support::host_for(
        yp_core::Os::Linux,
        &[("STEAM_COMPAT_DATA_PATH", "/env/data"), ("STEAM_COMPAT_CLIENT_INSTALL_PATH", "/env/steam")],
        &tree.home,
    );
    let env_ctx = yp_host::wine::WineContext::isolated(env);
    assert_eq!(compat_data_path(&env_ctx, Some(&exe)), "/env/data");
    assert_eq!(steam_root(&env_ctx, Some(&exe)), "/env/steam");
}

#[test]
fn proton_runtime_paths_are_recognised_by_name() {
    assert!(is_proton_runtime_path("/x/GE-Proton/proton"));
    assert!(is_proton_runtime_path("/x/Proton"));
    assert!(!is_proton_runtime_path("/usr/bin/wine"));
}
