use std::path::PathBuf;

use yp_builders::misctex_scan;
use yp_core::GamePaths;

struct Fixture {
    _dir: tempfile::TempDir,
    game: GamePaths,
}

fn fixture() -> Fixture {
    let dir = tempfile::Builder::new().prefix("misctex_scan_test").tempdir().unwrap();
    let game = GamePaths::new(dir.path());
    Fixture { _dir: dir, game }
}

impl Fixture {
    fn misctex_dir(&self, mod_id: &str) -> PathBuf {
        self.game.mods_dir().join(mod_id).join("data").join("misctex")
    }

    fn write_pair(&self, mod_id: &str, texture_name: &str, with_dtt: bool) {
        let dir = self.misctex_dir(mod_id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("misctex_{texture_name}.dat")), vec![0u8; 2048]).unwrap();
        if with_dtt {
            std::fs::write(dir.join(format!("misctex_{texture_name}.dtt")), vec![0u8; 1024]).unwrap();
        }
    }
}

#[test]
fn returns_nothing_when_there_are_no_mods() {
    assert!(misctex_scan::scan(&fixture().game).is_empty());
}

#[test]
fn returns_nothing_for_an_empty_game_dir() {
    assert!(misctex_scan::scan(&GamePaths::new("")).is_empty());
}

#[test]
fn finds_a_pair_and_reads_its_name_back() {
    let fixture = fixture();
    fixture.write_pair("my_mod", "item_thumb_8500", true);

    let found = misctex_scan::scan(&fixture.game);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].texture_name, "item_thumb_8500");
    assert_eq!(found[0].mod_id, "my_mod");
    assert_eq!(found[0].file_stem(), "misctex_item_thumb_8500");
    assert_eq!(found[0].directory(), fixture.misctex_dir("my_mod"));
}

#[test]
fn adds_up_the_size_of_both_files() {
    let fixture = fixture();
    fixture.write_pair("my_mod", "a", true);
    assert_eq!(misctex_scan::scan(&fixture.game)[0].total_bytes, 3072);
}

#[test]
fn counts_a_dat_without_its_dtt() {
    let fixture = fixture();
    fixture.write_pair("my_mod", "lonely", false);
    let found = misctex_scan::scan(&fixture.game);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].total_bytes, 2048);
}

#[test]
fn ignores_files_that_are_not_misctex_pairs() {
    let fixture = fixture();
    let dir = fixture.misctex_dir("my_mod");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notes.txt"), "x").unwrap();
    std::fs::write(dir.join("other_thing.dat"), "x").unwrap();
    std::fs::write(dir.join("misctex_a.dtt"), "x").unwrap();

    assert!(misctex_scan::scan(&fixture.game).is_empty());
}

#[test]
fn finds_icons_across_several_mods_sorted() {
    let fixture = fixture();
    fixture.write_pair("zeta_mod", "b", true);
    fixture.write_pair("alpha_mod", "z", true);
    fixture.write_pair("alpha_mod", "a", true);

    let found: Vec<String> = misctex_scan::scan(&fixture.game)
        .into_iter()
        .map(|entry| format!("{}/{}", entry.mod_id, entry.texture_name))
        .collect();
    assert_eq!(found, ["alpha_mod/a", "alpha_mod/z", "zeta_mod/b"]);
}

#[test]
fn remove_deletes_both_files() {
    let fixture = fixture();
    fixture.write_pair("my_mod", "gone", true);
    let entry = misctex_scan::scan(&fixture.game).remove(0);

    misctex_scan::remove(&entry).unwrap();

    assert!(!entry.dat_path.exists());
    assert!(!entry.dtt_path.exists());
    assert!(misctex_scan::scan(&fixture.game).is_empty());
}

#[test]
fn remove_does_not_fail_when_the_dtt_is_already_gone() {
    let fixture = fixture();
    fixture.write_pair("my_mod", "half", false);
    let entry = misctex_scan::scan(&fixture.game).remove(0);
    misctex_scan::remove(&entry).unwrap();
    assert!(!entry.dat_path.exists());
}

#[test]
fn watching_covers_the_mods_folder_and_texture_files() {
    let fixture = fixture();
    assert_eq!(misctex_scan::watch_path(&fixture.game), None);
    fixture.write_pair("my_mod", "a", true);
    assert_eq!(misctex_scan::watch_path(&fixture.game), Some(fixture.game.mods_dir()));
    assert!(misctex_scan::is_relevant_change(&fixture.misctex_dir("my_mod").join("misctex_a.dtt")));
    assert!(!misctex_scan::is_relevant_change(&fixture.misctex_dir("my_mod").join("notes.txt")));
}
