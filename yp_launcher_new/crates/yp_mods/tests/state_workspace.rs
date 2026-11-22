mod support;

use support::{put_bytes, temp};
use yp_core::GamePaths;
use yp_mods::ModProfiles;
use yp_mods::state::{DisabledModsData, ModsData, ModsWorkspace, WatchTarget};

#[test]
fn workspace_loads_everything_and_follows_a_profile_switch() {
    let dir = temp("yp_workspace_");
    let game = GamePaths::new(dir.path());
    put_bytes(&game.mods_dir().join("alpha"), "data/wp/wp0100.dat", 4, 0);
    std::fs::write(
        game.nams_dir().join("disabled_mods.toml"),
        "disabled = [\"mods/alpha\"]\n",
    )
    .unwrap();

    let mut workspace = ModsWorkspace::load(&game).unwrap();
    assert_eq!(workspace.profiles.active_name, "default");
    assert!(workspace.mods.find("alpha").is_some());
    assert!(workspace.disabled.is_disabled("mods/alpha/data"));

    let created = ModProfiles::new(&game).create("beta").unwrap();
    workspace.apply_profile_change(&game, created).unwrap();
    assert_eq!(workspace.profiles.active_name, "beta");
    assert!(workspace.mods.mods.is_empty());
    assert!(workspace.disabled.entries.is_empty());

    let switched = ModProfiles::new(&game).switch_to("default").unwrap();
    workspace.apply_profile_change(&game, switched).unwrap();
    assert!(workspace.mods.find("alpha").is_some());
    assert!(workspace.disabled.is_disabled("mods/alpha"));
}

#[test]
fn an_empty_game_dir_loads_nothing_and_reloads_are_no_ops() {
    let empty = GamePaths::new("");
    assert_eq!(
        ModsWorkspace::load(&empty).unwrap(),
        ModsWorkspace::default()
    );
    let mut disabled = DisabledModsData {
        entries: vec!["mods/kept".to_owned()],
    };
    disabled.reload(&empty).unwrap();
    assert_eq!(disabled.entries, vec!["mods/kept".to_owned()]);
}

#[test]
fn watch_targets_cover_the_mods_tree_and_every_sidecar() {
    let game = GamePaths::new("G");
    let targets = ModsWorkspace::watch_paths(&game);
    assert!(targets.contains(&WatchTarget {
        path: game.mods_dir(),
        recursive: true
    }));
    for file in [
        game.disabled_mods_toml(),
        game.default_mods_toml(),
        game.mod_load_order_toml(),
        game.mod_names_json(),
        game.profiles_toml(),
        game.nams_toml(),
    ] {
        assert!(
            targets.contains(&WatchTarget {
                path: file.clone(),
                recursive: false
            }),
            "{}",
            file.display()
        );
    }
    assert_eq!(ModsData::watch_paths(&game).len(), 1);
}
