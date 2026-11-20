use std::path::Path;

use yp_builders::first_mod;
use yp_core::GamePaths;

fn game() -> (tempfile::TempDir, GamePaths) {
    let dir = tempfile::Builder::new().prefix("first_mod_test").tempdir().unwrap();
    let game = GamePaths::new(dir.path());
    (dir, game)
}

fn parse(path: &Path) -> toml::Table {
    toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn writes_the_mod_under_nams_mods_my_first_mod() {
    let (dir, game) = game();
    first_mod::create_folder(&game).unwrap();
    first_mod::write_manifest(&game, None).unwrap();
    first_mod::write_item(&game, None).unwrap();

    let root = dir.path().join("nams").join("mods").join("my_first_mod");
    assert!(root.is_dir());
    assert!(root.join("mod.toml").is_file());
    assert!(root.join("entities").join("item_9999.toml").is_file());
}

#[test]
fn the_manifest_parses_and_carries_the_documented_fields() {
    let (_dir, game) = game();
    first_mod::write_manifest(&game, None).unwrap();
    let values = parse(&first_mod::manifest_path(&game));

    assert_eq!(values["id"].as_str(), Some("my_first_mod"));
    assert_eq!(values["version"].as_str(), Some("1.0.0"));
    assert!(values.contains_key("display_name"));
}

#[test]
fn the_item_parses_with_its_text_tables() {
    let (_dir, game) = game();
    first_mod::write_item(&game, None).unwrap();
    let values = parse(&first_mod::item_path(&game));

    assert_eq!(values["name"].as_str(), Some("item_test_pebble"));
    assert_eq!(values["id_type"].as_integer(), Some(0));
    assert_eq!(values["no_trash"].as_bool(), Some(true));
    assert_eq!(values["auto_give"]["quantity"].as_integer(), Some(1));
    assert_eq!(values["text"]["name"]["_us"].as_str(), Some("Test Pebble"));
    assert!(values["text"]["help"]["_us"].as_str().is_some());
}

#[test]
fn progress_checks_report_what_is_on_disk() {
    let (_dir, game) = game();
    assert!(!first_mod::has_folder(&game));
    assert!(!first_mod::has_manifest(&game));
    assert!(!first_mod::has_item(&game));

    first_mod::create_folder(&game).unwrap();
    assert!(first_mod::has_folder(&game));
    assert!(!first_mod::has_manifest(&game));

    first_mod::write_manifest(&game, None).unwrap();
    first_mod::write_item(&game, None).unwrap();
    assert!(first_mod::has_manifest(&game));
    assert!(first_mod::has_item(&game));
}

#[test]
fn writing_twice_overwrites_instead_of_failing() {
    let (_dir, game) = game();
    first_mod::write_item(&game, None).unwrap();
    first_mod::write_item(&game, None).unwrap();
    let raw = std::fs::read_to_string(first_mod::item_path(&game)).unwrap();
    assert_eq!(raw.matches("Test Pebble").count(), 1);
}

#[test]
fn edited_content_gets_a_trailing_newline_and_reads_back() {
    let (_dir, game) = game();
    assert_eq!(first_mod::read_item(&game).unwrap(), first_mod::ITEM);
    first_mod::write_item(&game, Some("name = \"item_x\"")).unwrap();
    assert_eq!(first_mod::read_item(&game).unwrap(), "name = \"item_x\"\n");
    first_mod::write_manifest(&game, Some("id = \"x\"\n")).unwrap();
    assert_eq!(first_mod::read_manifest(&game).unwrap(), "id = \"x\"\n");
}

#[test]
fn reset_removes_the_whole_mod_folder() {
    let (_dir, game) = game();
    first_mod::write_item(&game, None).unwrap();
    first_mod::remove_all(&game).unwrap();
    assert!(!first_mod::mod_root(&game).exists());
}

#[test]
fn reset_on_a_missing_folder_does_not_fail() {
    let (_dir, game) = game();
    first_mod::remove_all(&game).unwrap();
}

#[test]
fn the_written_files_match_the_documented_snippets() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("docs")
        .join("native")
        .join("01_first_mod.md");
    let doc = std::fs::read_to_string(doc_path).unwrap();
    assert!(doc.contains("id = \"my_first_mod\""));
    assert!(doc.contains("name = \"item_test_pebble\""));
    assert!(doc.contains("item_9999.toml"));

    for line in first_mod::ITEM.lines().map(str::trim).filter(|line| !line.is_empty()) {
        assert!(doc.contains(line), "doc is missing: {line}");
    }
}
