use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::load_order;
use yp_mods::state::ModLoadOrderData;

fn game() -> (TempDir, GamePaths) {
    let dir = tempfile::Builder::new()
        .prefix("yp_loadorder_state_")
        .tempdir()
        .unwrap();
    let game = GamePaths::new(dir.path());
    (dir, game)
}

fn order(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn the_ui_list_is_highest_priority_first() {
    let (_dir, game) = game();
    let mut state = ModLoadOrderData::default();
    state
        .save(&game, &order(&["mods/winner", "mods/loser"]))
        .unwrap();
    assert_eq!(state.order, order(&["mods/winner", "mods/loser"]));
}

#[test]
fn the_file_stores_lowest_first_so_nams_sees_the_winner_last() {
    let (_dir, game) = game();
    let mut state = ModLoadOrderData::default();
    state
        .save(&game, &order(&["mods/winner", "mods/loser"]))
        .unwrap();
    assert_eq!(
        load_order::list(&game).unwrap(),
        order(&["mods/loser", "mods/winner"])
    );
}

#[test]
fn save_then_load_is_stable_not_flipped_each_time() {
    let (_dir, game) = game();
    let mut state = ModLoadOrderData::default();
    state
        .save(&game, &order(&["mods/a", "mods/b", "mods/c"]))
        .unwrap();
    state.reload(&game).unwrap();
    state.reload(&game).unwrap();
    assert_eq!(state.order, order(&["mods/a", "mods/b", "mods/c"]));
}

#[test]
fn a_file_written_by_hand_is_shown_winner_first() {
    let (_dir, game) = game();
    std::fs::create_dir_all(game.nams_dir()).unwrap();
    std::fs::write(
        game.mod_load_order_toml(),
        "load_order = [\"mods/loser\", \"mods/winner\"]\n",
    )
    .unwrap();
    let mut state = ModLoadOrderData::default();
    state.reload(&game).unwrap();
    assert_eq!(state.order, order(&["mods/winner", "mods/loser"]));
}

#[test]
fn index_of_reports_ui_position() {
    let (_dir, game) = game();
    let mut state = ModLoadOrderData::default();
    state.save(&game, &order(&["mods/a", "mods/b"])).unwrap();
    assert_eq!(state.index_of("mods/a"), Some(0));
    assert_eq!(state.index_of("mods/b"), Some(1));
    assert_eq!(state.index_of("mods/missing"), None);
}

#[test]
fn an_empty_game_dir_is_a_no_op() {
    let mut state = ModLoadOrderData::default();
    state.reload(&GamePaths::new("")).unwrap();
    assert!(state.order.is_empty());
}
