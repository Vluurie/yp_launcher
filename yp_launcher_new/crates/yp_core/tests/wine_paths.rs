use std::path::Path;

use yp_core::wine_paths::{
    infer_steam_context, infer_wine_prefix, is_crossover_prefix, posix, resolve_wine_user, to_posix, to_wine_path,
    wine_env, wine_roaming_dir, NIER_STEAM_APP_ID,
};
use yp_core::{HostEnv, Os};

const BOTTLE: &str = "/Users/d/Library/Application Support/CrossOver/Bottles/Steam";

fn game_dir() -> String {
    format!("{BOTTLE}/drive_c/Program Files (x86)/Steam/steamapps/common/NieRAutomata")
}

fn host(vars: &[(&str, &str)]) -> HostEnv {
    HostEnv::for_test(Os::Linux, vars, Path::new("/nonexistent-root"))
}

fn temp_posix() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let root = to_posix(dir.path());
    (dir, root)
}

#[test]
fn infer_prefix_finds_the_root_above_drive_c() {
    assert_eq!(infer_wine_prefix(&game_dir()).as_deref(), Some(BOTTLE));
}

#[test]
fn infer_prefix_matches_drive_c_case_insensitively() {
    assert_eq!(infer_wine_prefix(&format!("{BOTTLE}/DRIVE_C/Foo")).as_deref(), Some(BOTTLE));
}

#[test]
fn infer_prefix_is_none_outside_any_prefix() {
    assert_eq!(
        infer_wine_prefix("/Users/d/Library/Application Support/com.vluurie.yplauncher/bins"),
        None
    );
}

#[test]
fn infer_prefix_at_filesystem_root_is_slash() {
    assert_eq!(infer_wine_prefix("/drive_c/x").as_deref(), Some("/"));
}

#[test]
fn infer_prefix_bare_relative_drive_c_has_no_prefix() {
    assert_eq!(infer_wine_prefix("drive_c/x"), None);
}

#[test]
fn infer_prefix_first_drive_c_wins_when_nested() {
    assert_eq!(infer_wine_prefix("/a/drive_c/b/drive_c/c").as_deref(), Some("/a"));
}

#[test]
fn to_wine_path_maps_inside_a_prefix_to_c() {
    assert_eq!(
        to_wine_path(&game_dir()),
        r"C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata"
    );
}

#[test]
fn to_wine_path_maps_outside_a_prefix_to_z() {
    assert_eq!(
        to_wine_path("/Users/d/Library/Application Support/x/bins/NAMS.exe"),
        r"Z:\Users\d\Library\Application Support\x\bins\NAMS.exe"
    );
}

#[test]
fn to_wine_path_drive_c_itself_is_bare_c() {
    assert_eq!(to_wine_path(&format!("{BOTTLE}/drive_c")), r"C:\");
}

#[test]
fn to_wine_path_matches_drive_c_case_insensitively() {
    assert_eq!(to_wine_path(&format!("{BOTTLE}/DRIVE_C/Foo")), r"C:\Foo");
}

#[test]
fn crossover_prefix_accepts_a_bottle() {
    assert!(is_crossover_prefix(BOTTLE));
}

#[test]
fn crossover_prefix_accepts_a_preview_bottle() {
    assert!(is_crossover_prefix(
        "/Users/d/Library/Application Support/CrossOver Preview/Bottles/Steam"
    ));
}

#[test]
fn crossover_prefix_accepts_a_linux_cxoffice_bottle() {
    assert!(is_crossover_prefix("/home/d/.cxoffice/Steam"));
}

#[test]
fn crossover_prefix_accepts_a_flatpak_bottle() {
    assert!(is_crossover_prefix(
        "/home/d/.var/app/com.codeweavers.CrossOver/data/crossover/bottles/Steam"
    ));
}

#[test]
fn crossover_prefix_rejects_a_proton_prefix() {
    assert!(!is_crossover_prefix("/home/d/.steam/steam/steamapps/compatdata/524220/pfx"));
}

#[test]
fn resolve_user_skips_public_and_picks_the_crossover_user() {
    let (_dir, root) = temp_posix();
    std::fs::create_dir_all(posix::join(&root, "Public")).unwrap();
    std::fs::create_dir_all(posix::join(&root, "crossover")).unwrap();
    assert_eq!(resolve_wine_user(&root, &host(&[])), "crossover");
}

#[test]
fn resolve_user_prefers_the_host_user_over_steamuser() {
    let (_dir, root) = temp_posix();
    std::fs::create_dir_all(posix::join(&root, "steamuser")).unwrap();
    std::fs::create_dir_all(posix::join(&root, "alice")).unwrap();
    assert_eq!(resolve_wine_user(&root, &host(&[("USER", "alice")])), "alice");
}

#[test]
fn resolve_user_falls_back_to_the_only_user_present() {
    let (_dir, root) = temp_posix();
    std::fs::create_dir_all(posix::join(&root, "someoneelse")).unwrap();
    assert_eq!(resolve_wine_user(&root, &host(&[])), "someoneelse");
}

#[test]
fn resolve_user_tolerates_a_missing_users_dir() {
    let (_dir, root) = temp_posix();
    assert_eq!(resolve_wine_user(&posix::join(&root, "nope"), &host(&[])), "steamuser");
    assert_eq!(resolve_wine_user(&posix::join(&root, "nope"), &host(&[("USER", "bob")])), "bob");
}

#[test]
fn roaming_dir_is_under_the_resolved_user() {
    let (_dir, root) = temp_posix();
    std::fs::create_dir_all(posix::join_all(&root, &["drive_c", "users", "crossover"])).unwrap();
    assert_eq!(
        wine_roaming_dir(&root, &host(&[])),
        posix::join_all(&root, &["drive_c", "users", "crossover", "AppData", "Roaming"])
    );
}

#[test]
fn wine_env_defaults_winedebug_and_honours_the_host() {
    assert_eq!(wine_env(&host(&[]))["WINEDEBUG"], "-all");
    assert_eq!(wine_env(&host(&[("WINEDEBUG", "+seh")]))["WINEDEBUG"], "+seh");
}

#[test]
fn steam_context_splits_library_and_root() {
    let roots = vec!["/home/u/.local/share/Steam".to_owned()];
    let ctx = infer_steam_context("/mnt/ssd/SteamLibrary/steamapps/common/NieRAutomata/NieRAutomata.exe", &roots).unwrap();
    assert_eq!(ctx.library_root, "/mnt/ssd/SteamLibrary");
    assert_eq!(ctx.steam_root, "/home/u/.local/share/Steam");
    assert_eq!(
        ctx.compat_data,
        format!("/mnt/ssd/SteamLibrary/steamapps/compatdata/{NIER_STEAM_APP_ID}")
    );
    assert_eq!(ctx.prefix(), format!("{}/pfx", ctx.compat_data));
}

#[test]
fn steam_context_matches_segments_case_insensitively_and_needs_a_child() {
    assert!(infer_steam_context("/a/SteamApps/Common/NieRAutomata", &[]).is_some());
    assert!(infer_steam_context("/a/steamapps/common", &[]).is_none());
    assert!(infer_steam_context("/a/mysteamapps/common/x", &[]).is_none());
    assert!(infer_steam_context("steamapps/common/x", &[]).is_none());
    assert_eq!(
        infer_steam_context(r"D:\Lib\steamapps\common\NieR", &[]).unwrap().library_root,
        "D:/Lib"
    );
}
