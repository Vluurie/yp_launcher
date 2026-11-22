use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::load_order;

fn game() -> (TempDir, GamePaths) {
    let dir = tempfile::Builder::new()
        .prefix("yp_loadorder_")
        .tempdir()
        .unwrap();
    let game = GamePaths::new(dir.path());
    (dir, game)
}

fn order(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn write_raw(game: &GamePaths, text: &str) {
    std::fs::create_dir_all(game.nams_dir()).unwrap();
    std::fs::write(game.mod_load_order_toml(), text).unwrap();
}

#[test]
fn missing_file_yields_no_order() {
    let (_dir, game) = game();
    assert!(load_order::list(&game).unwrap().is_empty());
}

#[test]
fn an_order_round_trips_through_disk() {
    let (_dir, game) = game();
    load_order::save(&game, &order(&["mods/a", "mods/b"])).unwrap();
    assert_eq!(
        load_order::list(&game).unwrap(),
        order(&["mods/a", "mods/b"])
    );
    assert!(game.mod_load_order_toml().is_file());
}

#[test]
fn order_is_preserved_exactly_not_sorted() {
    let (_dir, game) = game();
    load_order::save(&game, &order(&["mods/z", "mods/a"])).unwrap();
    assert_eq!(
        load_order::list(&game).unwrap(),
        order(&["mods/z", "mods/a"])
    );
}

#[test]
fn saving_replaces_the_previous_order() {
    let (_dir, game) = game();
    load_order::save(&game, &order(&["mods/a", "mods/b"])).unwrap();
    load_order::save(&game, &order(&["mods/b", "mods/a"])).unwrap();
    assert_eq!(
        load_order::list(&game).unwrap(),
        order(&["mods/b", "mods/a"])
    );
}

#[test]
fn backslashes_are_normalized_to_forward_slashes() {
    let (_dir, game) = game();
    load_order::save(&game, &order(&[r"mods\a"])).unwrap();
    assert_eq!(load_order::list(&game).unwrap(), order(&["mods/a"]));
}

#[test]
fn blank_entries_are_dropped() {
    let (_dir, game) = game();
    load_order::save(&game, &order(&["mods/a", "", "   "])).unwrap();
    assert_eq!(load_order::list(&game).unwrap(), order(&["mods/a"]));
}

#[test]
fn an_empty_order_clears_the_list() {
    let (_dir, game) = game();
    load_order::save(&game, &order(&["mods/a"])).unwrap();
    load_order::save(&game, &[]).unwrap();
    assert!(load_order::list(&game).unwrap().is_empty());
}

#[test]
fn comments_in_the_file_survive_a_write() {
    let (_dir, game) = game();
    write_raw(&game, "# keep me\nload_order = []\n");
    load_order::save(&game, &order(&["mods/a"])).unwrap();
    assert!(
        std::fs::read_to_string(game.mod_load_order_toml())
            .unwrap()
            .contains("# keep me")
    );
}

#[test]
fn a_malformed_file_degrades_to_no_order_instead_of_throwing() {
    let (_dir, game) = game();
    write_raw(&game, "load_order = \"not a list\"\n");
    assert!(load_order::list(&game).unwrap().is_empty());
}
