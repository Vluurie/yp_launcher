use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::defaults::{self, DefaultKind, FEATURE_KEY};
use yp_mods::state::{DefaultModsData, WatchTarget};

fn game() -> (TempDir, GamePaths) {
    let dir = tempfile::Builder::new()
        .prefix("yp_defoutfit_")
        .tempdir()
        .unwrap();
    let game = GamePaths::new(dir.path());
    std::fs::create_dir_all(game.nams_dir()).unwrap();
    (dir, game)
}

fn set_feature(game: &GamePaths, enabled: bool) {
    std::fs::write(
        game.nams_toml(),
        format!("# NAMS settings\nvalidate_scripts = false\n{FEATURE_KEY} = {enabled}\n"),
    )
    .unwrap();
}

fn star_one_mod(game: &GamePaths) {
    defaults::set_default(
        game,
        "mods/kimono/pl/pl000d",
        Some(DefaultKind::OutfitConfig),
        None,
    )
    .unwrap();
}

fn paths(data: &DefaultModsData) -> Vec<&str> {
    data.entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect()
}

#[test]
fn an_enabled_feature_shows_the_stars() {
    let (_dir, game) = game();
    set_feature(&game, true);
    star_one_mod(&game);
    let mut data = DefaultModsData::default();
    data.reload(&game).unwrap();
    assert!(data.default_outfits_enabled);
    assert_eq!(paths(&data), vec!["mods/kimono/pl/pl000d"]);
}

#[test]
fn turning_the_feature_on_refreshes_the_stars_without_a_restart() {
    let (_dir, game) = game();
    set_feature(&game, false);
    star_one_mod(&game);
    let mut data = DefaultModsData::default();
    data.reload(&game).unwrap();
    assert!(
        !data.default_outfits_enabled,
        "feature is off, so no stars are shown"
    );
    assert!(data.entries.is_empty());

    set_feature(&game, true);
    data.reload(&game).unwrap();
    assert!(
        data.default_outfits_enabled,
        "reloading after the toggle must pick it up"
    );
    assert_eq!(paths(&data), vec!["mods/kimono/pl/pl000d"]);
}

#[test]
fn turning_the_feature_off_clears_the_stars_right_away() {
    let (_dir, game) = game();
    set_feature(&game, true);
    star_one_mod(&game);
    let mut data = DefaultModsData::default();
    data.reload(&game).unwrap();
    assert!(!data.entries.is_empty());

    set_feature(&game, false);
    data.reload(&game).unwrap();
    assert!(!data.default_outfits_enabled);
    assert!(data.entries.is_empty());
}

#[test]
fn a_missing_or_non_boolean_toggle_counts_as_off() {
    let (_dir, game) = game();
    assert!(!defaults::is_feature_enabled(&game));
    std::fs::write(game.nams_toml(), format!("{FEATURE_KEY} = \"true\"\n")).unwrap();
    assert!(!defaults::is_feature_enabled(&game));
    std::fs::write(game.nams_toml(), format!("{FEATURE_KEY} = = true\n")).unwrap();
    assert!(!defaults::is_feature_enabled(&game));
}

#[test]
fn saving_an_unrelated_key_leaves_the_starred_entries_alone() {
    let (_dir, game) = game();
    set_feature(&game, true);
    star_one_mod(&game);
    let mut data = DefaultModsData::default();
    data.reload(&game).unwrap();
    let before: Vec<String> = data
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect();

    std::fs::write(
        game.nams_toml(),
        format!("# NAMS settings\nvalidate_scripts = true\n{FEATURE_KEY} = true\n"),
    )
    .unwrap();
    data.reload(&game).unwrap();

    assert_eq!(
        paths(&data),
        before.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert!(data.default_outfits_enabled);
}

#[test]
fn the_state_watches_nams_toml_so_toggles_trigger_a_reload() {
    let (_dir, game) = game();
    let targets = DefaultModsData::watch_paths(&game);
    assert!(targets.contains(&WatchTarget {
        path: game.nams_toml(),
        recursive: false
    }));
    assert!(targets.contains(&WatchTarget {
        path: game.default_mods_toml(),
        recursive: false
    }));
}
