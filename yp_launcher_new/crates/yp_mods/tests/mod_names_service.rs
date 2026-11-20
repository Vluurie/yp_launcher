use indexmap::IndexMap;
use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::names;
use yp_mods::state::ModNamesData;

fn game() -> (TempDir, GamePaths) {
    let dir = tempfile::Builder::new()
        .prefix("yp_names_")
        .tempdir()
        .unwrap();
    let game = GamePaths::new(dir.path());
    (dir, game)
}

fn map(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn missing_file_yields_no_names() {
    let (_dir, game) = game();
    assert!(names::load(&game).is_empty());
}

#[test]
fn a_name_round_trips_through_disk() {
    let (_dir, game) = game();
    names::set_name(&game, "my_mod", Some("Nicer Name"));
    assert_eq!(names::load(&game), map(&[("my_mod", "Nicer Name")]));
    assert!(game.mod_names_json().is_file());
}

#[test]
fn renaming_twice_keeps_only_the_latest_name() {
    let (_dir, game) = game();
    names::set_name(&game, "my_mod", Some("First"));
    names::set_name(&game, "my_mod", Some("Second"));
    assert_eq!(names::load(&game), map(&[("my_mod", "Second")]));
}

#[test]
fn names_of_other_mods_survive_an_unrelated_rename() {
    let (_dir, game) = game();
    names::set_name(&game, "a", Some("Alpha"));
    names::set_name(&game, "b", Some("Beta"));
    assert_eq!(names::load(&game), map(&[("a", "Alpha"), ("b", "Beta")]));
}

#[test]
fn clearing_a_name_drops_the_entry_and_removes_an_empty_file() {
    let (_dir, game) = game();
    names::set_name(&game, "my_mod", Some("Nicer Name"));
    names::set_name(&game, "my_mod", None);
    assert!(names::load(&game).is_empty());
    assert!(!game.mod_names_json().exists());
}

#[test]
fn blank_names_are_treated_as_a_reset() {
    let (_dir, game) = game();
    names::set_name(&game, "my_mod", Some("Nicer Name"));
    names::set_name(&game, "my_mod", Some("   "));
    assert!(names::load(&game).is_empty());
}

#[test]
fn corrupt_json_degrades_to_no_names_instead_of_throwing() {
    let (_dir, game) = game();
    std::fs::create_dir_all(game.nams_dir()).unwrap();
    std::fs::write(game.mod_names_json(), "{not json").unwrap();
    assert!(names::load(&game).is_empty());
}

#[test]
fn an_empty_game_dir_is_a_no_op() {
    let empty = GamePaths::new("");
    names::set_name(&empty, "my_mod", Some("X"));
    assert!(names::load(&empty).is_empty());
}

#[test]
fn state_exposes_custom_names() {
    let (_dir, game) = game();
    let mut state = ModNamesData::default();
    state.rename(&game, "a", Some("  Alpha  "));
    assert_eq!(state.custom_name_of("a"), Some("Alpha"));
    assert_eq!(state.custom_name_of("b"), None);
}
