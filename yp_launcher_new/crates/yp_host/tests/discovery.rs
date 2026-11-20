use std::path::Path;
use std::time::Duration;

use yp_core::{CancelToken, HostEnv, Os};
use yp_host::discovery::{
    installs_from_hints, installs_from_windows, installs_near_launcher, scan_roots, RegistryHints,
};

fn make_game(dir: &Path, with_data: bool) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("NieRAutomata.exe"), "PRJ_028").unwrap();
    if with_data {
        std::fs::create_dir_all(dir.join("data")).unwrap();
    }
}

#[test]
fn nearby_discovery_finds_the_game_in_the_launcher_directory() {
    let root = tempfile::tempdir().unwrap();
    make_game(root.path(), true);
    let found = installs_near_launcher(root.path());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, root.path());
    assert!(found[0].has_data);
}

#[test]
fn nearby_discovery_finds_the_game_above_a_nested_launcher_without_steam() {
    let root = tempfile::tempdir().unwrap();
    make_game(root.path(), false);
    let launcher = root.path().join("YoRHa Protocol Launcher").join("release");
    std::fs::create_dir_all(&launcher).unwrap();
    let found = installs_near_launcher(&launcher);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, root.path());
    assert!(!found[0].has_data);
}

#[test]
fn nearby_discovery_uses_the_closest_game() {
    let root = tempfile::tempdir().unwrap();
    make_game(root.path(), true);
    let inner = root.path().join("Other copy");
    make_game(&inner, true);
    let launcher = inner.join("YoRHa Protocol Launcher");
    std::fs::create_dir_all(&launcher).unwrap();
    let found = installs_near_launcher(&launcher);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, inner);
}

#[test]
fn nearby_discovery_requires_a_game_exe_file_and_does_not_scan_other_bottles() {
    let root = tempfile::tempdir().unwrap();
    let launcher = root.path().join("launcher bottle");
    std::fs::create_dir_all(launcher.join("NieRAutomata.exe")).unwrap();
    std::fs::write(launcher.join("NAMS.exe"), "PRJ_028").unwrap();
    make_game(&root.path().join("other bottle"), true);
    assert!(installs_near_launcher(&launcher).is_empty());
}

#[test]
fn registry_hints_combine_the_uninstall_entry_and_steam_libraries() {
    let dir = tempfile::tempdir().unwrap();
    let direct = dir.path().join("Direct").join("NieRAutomata");
    make_game(&direct, true);
    let steam = dir.path().join("Steam");
    let library = dir.path().join("Library");
    let in_library = library.join("steamapps").join("common").join("NieRAutomata");
    make_game(&in_library, false);
    std::fs::create_dir_all(steam.join("steamapps")).unwrap();
    let escape = |path: &Path| path.to_string_lossy().replace('\\', "\\\\");
    std::fs::write(
        steam.join("steamapps").join("libraryfolders.vdf"),
        format!(
            "\"libraryfolders\"\n{{\n\t\"0\" {{ \"path\" \"{}\" }}\n\t\"1\" {{ \"path\" \"{}\" }}\n\t\"2\" {{ \"path\" \"{}\" }}\n}}\n",
            escape(&steam),
            escape(&library),
            escape(&library)
        ),
    )
    .unwrap();
    let found = installs_from_hints(&RegistryHints {
        install_location: Some(format!("{}{}", direct.display(), std::path::MAIN_SEPARATOR)),
        steam_path: Some(steam.to_string_lossy().into_owned()),
    });
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].path, direct);
    assert!(found[0].has_data);
    assert_eq!(found[1].path, in_library);
    assert!(!found[1].has_data);
}

#[test]
fn missing_hints_find_nothing() {
    assert!(installs_from_hints(&RegistryHints::default()).is_empty());
    let dir = tempfile::tempdir().unwrap();
    let hints = RegistryHints {
        install_location: Some(dir.path().join("nope").to_string_lossy().into_owned()),
        steam_path: Some(dir.path().join("nosteam").to_string_lossy().into_owned()),
    };
    assert!(installs_from_hints(&hints).is_empty());
}

#[test]
fn a_bottle_library_is_found_without_registry_hints_or_a_nearby_launcher() {
    let root = tempfile::tempdir().unwrap();
    let programs = root.path().join("bottle").join("drive_c").join("Program Files (x86)");
    let steamapps = programs.join("Steam").join("steamapps");
    let game = steamapps.join("common").join("NieR Automata 日本語");
    make_game(&game, true);
    std::fs::write(
        steamapps.join("appmanifest_524220.acf"),
        "\"AppState\" { \"appid\" \"524220\" \"installdir\" \"NieR Automata 日本語\" }",
    )
    .unwrap();
    let host = HostEnv::for_test(
        Os::Windows,
        &[
            ("ProgramFiles(x86)", &programs.to_string_lossy()),
            ("ProgramFiles", &programs.to_string_lossy()),
        ],
        &root.path().join("separate tools"),
    );
    assert!(installs_near_launcher(host.exe_dir()).is_empty());
    let found = installs_from_windows(&host, &RegistryHints::default());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, game);
    assert!(found[0].has_data);
}

#[test]
fn steam_manifest_names_are_used_in_the_client_library_without_a_library_list() {
    let root = tempfile::tempdir().unwrap();
    let steam = root.path().join("steam-client");
    let game = steam.join("steamapps").join("common").join("custom game folder");
    make_game(&game, true);
    std::fs::write(
        steam.join("steamapps").join("appmanifest_524220.acf"),
        "\"AppState\" { \"installdir\" \"custom game folder\" }",
    )
    .unwrap();
    let found = installs_from_hints(&RegistryHints {
        steam_path: Some(steam.to_string_lossy().into_owned()),
        ..RegistryHints::default()
    });
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, game);
}

#[test]
fn manifest_install_directories_cannot_escape_the_library() {
    let root = tempfile::tempdir().unwrap();
    let steam = root.path().join("Steam");
    let common = steam.join("steamapps").join("common");
    std::fs::create_dir_all(&common).unwrap();
    make_game(&steam.join("steamapps").join("outside"), false);
    for directory in ["../outside", "..\\outside", "C:/outside", ".", ".."] {
        std::fs::write(
            steam.join("steamapps").join("appmanifest_524220.acf"),
            format!(
                "\"AppState\" {{ \"installdir\" \"{}\" }}",
                directory.replace('\\', "\\\\")
            ),
        )
        .unwrap();
        assert!(installs_from_hints(&RegistryHints {
            steam_path: Some(steam.to_string_lossy().into_owned()),
            ..RegistryHints::default()
        })
        .is_empty());
    }
}

#[test]
fn drive_scan_checks_multiple_roots_and_skips_system_folders() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("drive");
    let found_game = root.join("Games").join("Steam").join("NieRAutomata");
    make_game(&found_game, true);
    make_game(&root.join("Windows").join("NieR"), false);
    make_game(&root.join("ProgramData").join("NieR"), false);
    make_game(&root.join(".hidden").join("NieR"), false);
    make_game(&root.join("Users").join("me").join("AppData").join("NieR"), false);
    std::fs::write(root.join("Games").join("nierautomata.EXE.txt"), "").unwrap();
    let second = dir.path().join("second");
    let other_game = second.join("NieRAutomata");
    make_game(&other_game, false);
    let found = scan_roots(&[root, second], &CancelToken::new(), Duration::from_secs(30));
    let paths: Vec<&Path> = found.iter().map(|install| install.path.as_path()).collect();
    assert_eq!(paths, [found_game.as_path(), other_game.as_path()]);
    assert!(found[0].has_data);
}

#[test]
fn drive_scan_stops_descending_after_finding_a_game() {
    let root = tempfile::tempdir().unwrap();
    let game = root.path().join("game");
    make_game(&game, true);
    make_game(&game.join("data").join("backup"), false);
    let found = scan_roots(
        &[root.path().to_path_buf()],
        &CancelToken::new(),
        Duration::from_secs(5),
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, game);
}

#[test]
fn drive_scan_deduplicates_overlapping_roots() {
    let root = tempfile::tempdir().unwrap();
    let game = root.path().join("storage").join("game");
    make_game(&game, true);
    let found = scan_roots(
        &[root.path().to_path_buf(), root.path().join("storage"), game.clone()],
        &CancelToken::new(),
        Duration::from_secs(5),
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, game);
}

#[cfg(windows)]
fn link_directory(target: &Path, link: &Path) {
    let output = std::process::Command::new("cmd")
        .args(["/D", "/C", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[cfg(unix)]
fn link_directory(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[test]
fn drive_scan_follows_linked_directories_without_loops_or_duplicate_games() {
    let root = tempfile::tempdir().unwrap();
    let drive = root.path().join("mapped-drive");
    std::fs::create_dir_all(&drive).unwrap();
    let game = root.path().join("external-storage").join("game");
    make_game(&game, true);
    link_directory(&game, &drive.join("game-link"));
    link_directory(&game, &drive.join("same-game-link"));
    link_directory(&drive, &drive.join("loop"));
    let found = scan_roots(&[drive], &CancelToken::new(), Duration::from_secs(5));
    assert_eq!(found.len(), 1);
    assert_eq!(
        std::fs::canonicalize(&found[0].path).unwrap(),
        std::fs::canonicalize(game).unwrap()
    );
}

#[test]
fn drive_scan_does_not_apply_git_or_ignore_file_rules() {
    let root = tempfile::tempdir().unwrap();
    let game = root.path().join("Games").join("NieR");
    make_game(&game, true);
    std::fs::create_dir_all(root.path().join(".git")).unwrap();
    std::fs::write(root.path().join(".gitignore"), "Games/\n").unwrap();
    std::fs::write(root.path().join(".ignore"), "Games/\n").unwrap();
    let found = scan_roots(
        &[root.path().to_path_buf()],
        &CancelToken::new(),
        Duration::from_secs(5),
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, game);
}

#[test]
fn drive_scan_handles_missing_and_empty_roots() {
    let root = tempfile::tempdir().unwrap();
    let game = root.path().join("game");
    make_game(&game, false);
    let cancel = CancelToken::new();
    assert!(scan_roots(&[], &cancel, Duration::from_secs(5)).is_empty());
    let found = scan_roots(
        &[root.path().join("missing-drive"), game.clone()],
        &cancel,
        Duration::from_secs(5),
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, game);
}

#[test]
fn drive_scan_stops_when_cancelled() {
    let dir = tempfile::tempdir().unwrap();
    make_game(&dir.path().join("a").join("NieR"), false);
    let cancel = CancelToken::new();
    cancel.cancel();
    assert!(scan_roots(&[dir.path().to_path_buf()], &cancel, Duration::from_secs(30)).is_empty());
    assert!(scan_roots(&[dir.path().to_path_buf()], &CancelToken::new(), Duration::ZERO).is_empty());
}
