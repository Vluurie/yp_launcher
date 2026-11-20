mod support;

use serde_json::json;
use support::{put_bytes, put_json, temp};
use yp_core::GamePaths;
use yp_mods::detect::detect_texture_packs;
use yp_mods::scan::list_installed;
use yp_mods::{DataCategory, InstalledMod, ModConflictKind, ModKind};

fn find<'a>(mods: &'a [InstalledMod], id: &str) -> &'a InstalledMod {
    mods.iter().find(|installed| installed.id == id).unwrap()
}

#[test]
fn conflicts_need_a_shared_path_with_a_different_size() {
    let dir = temp("yp_scan_conflicts_");
    let game = GamePaths::new(dir.path());
    let mods = game.mods_dir();
    put_bytes(&mods.join("alpha"), "data/wp/wp0100.dat", 10, 1);
    put_bytes(&mods.join("beta"), "data/wp/wp0100.dat", 20, 2);
    put_bytes(&mods.join("alpha"), "data/em/em1000.dat", 5, 1);
    put_bytes(&mods.join("beta"), "data/em/em1000.dat", 5, 2);
    put_bytes(&mods.join("alpha"), "data/pl/pl0000.dat", 5, 1);
    put_bytes(&mods.join("beta"), "data/pl/pl0000.dat", 9, 2);
    std::fs::create_dir_all(mods.join("_hidden")).unwrap();
    std::fs::create_dir_all(mods.join(".git")).unwrap();
    std::fs::write(mods.join("stray.txt"), b"x").unwrap();

    let listed = list_installed(&game).unwrap();
    assert_eq!(listed.len(), 2);
    for (id, other) in [("alpha", "beta"), ("beta", "alpha")] {
        let installed = find(&listed, id);
        assert_eq!(
            installed.conflicts.len(),
            1,
            "{id}: {:?}",
            installed.conflicts
        );
        assert_eq!(installed.conflicts[0].other_mod_id, other);
        assert_eq!(
            installed.conflicts[0].kind,
            ModConflictKind::OverlappingDataFile
        );
        assert_eq!(installed.conflicts[0].detail, "data/wp/wp0100.dat");
        assert!(installed.has_warnings());
    }
}

#[test]
fn manifest_drives_names_and_missing_requirements() {
    let dir = temp("yp_scan_manifest_");
    let game = GamePaths::new(dir.path());
    let root = game.mods_dir().join("quest_pack");
    std::fs::create_dir_all(root.join("entities")).unwrap();
    std::fs::write(
        root.join("mod.toml"),
        "[mod]\nid = \"quest_pack\"\ndisplay_name = \"Quest Pack\"\nrequires = [\"base_lib\", \"other\"]\n",
    )
    .unwrap();
    std::fs::write(root.join("entities").join("quest_01.toml"), "x = 1\n").unwrap();
    std::fs::write(
        root.join("entities").join("weapons.json"),
        r#"{"kind":"Weapon_Bundle"}"#,
    )
    .unwrap();
    put_bytes(&game.mods_dir().join("other"), "data/wp/wp0100.dat", 4, 0);

    let listed = list_installed(&game).unwrap();
    let installed = find(&listed, "quest_pack");
    assert_eq!(installed.display_name, "Quest Pack");
    assert_eq!(installed.kind, ModKind::Native);
    assert_eq!(installed.requires_missing, vec!["base_lib".to_owned()]);
    let native = installed.native.as_ref().unwrap();
    assert_eq!(native.total_entity_files, 2);
    assert_eq!(native.bundles_by_kind.get("quest"), Some(&1));
    assert_eq!(native.bundles_by_kind.get("weapon"), Some(&1));
    assert_eq!(find(&listed, "other").display_name, "other");
}

#[test]
fn data_summary_lists_dirs_players_and_archive_pairs() {
    let dir = temp("yp_scan_data_");
    let game = GamePaths::new(dir.path());
    let root = game.mods_dir().join("Wrapper Mod");
    let content = root.join("Inner");
    put_bytes(&content, "data/pl/pl0000.dat", 4, 0);
    put_bytes(&content, "data/pl/pl0000.dtt", 4, 0);
    put_bytes(&content, "data/pl/pl1234.DAT", 4, 0);
    put_bytes(&content, "data/wp/wp0100.dat", 4, 0);
    put_bytes(&content, "data/wp/wp0100.dtt", 4, 0);
    put_bytes(&content, "data/wp/readme.txt", 4, 0);
    put_bytes(&content, "data/custom/thing.bin", 4, 0);
    put_bytes(&content, "data/data100.cpk", 4, 0);

    let listed = list_installed(&game).unwrap();
    let installed = find(&listed, "Wrapper Mod");
    assert_eq!(installed.root_path, root);
    let data = installed.data.as_ref().unwrap();
    let entries: Vec<(&str, DataCategory, usize)> = data
        .entries
        .iter()
        .map(|entry| (entry.dir_name.as_str(), entry.category, entry.file_count))
        .collect();
    assert_eq!(
        entries,
        vec![
            ("cpk", DataCategory::Archive, 1),
            ("custom", DataCategory::Other, 1),
            ("pl", DataCategory::Player, 2),
            ("wp", DataCategory::Weapon, 2),
        ]
    );
    let players: Vec<(&str, &str)> = data
        .players
        .iter()
        .map(|player| (player.file_name.as_str(), player.label.as_str()))
        .collect();
    assert_eq!(
        players,
        vec![("pl0000.dat", "2B (vanilla)"), ("pl1234.DAT", "pl1234.DAT")]
    );
    let archives: Vec<(&str, bool, bool)> = data
        .archives
        .iter()
        .map(|pair| {
            (
                pair.label.as_str(),
                pair.dat_path.is_some(),
                pair.dtt_path.is_some(),
            )
        })
        .collect();
    assert_eq!(
        archives,
        vec![
            ("2B (vanilla)", true, true),
            ("pl1234", true, false),
            ("wp0100", true, true)
        ]
    );
    assert!(data.archives.iter().all(|pair| pair.config_root == content));
    assert!(installed.has_data_overlay());
}

#[test]
fn bundled_cutscenes_and_texture_packs_are_reported() {
    let dir = temp("yp_scan_bundled_");
    let game = GamePaths::new(dir.path());
    let root = game.mods_dir().join("movie_mod");
    put_bytes(&root, "data/wp/wp0100.dat", 4, 0);
    put_bytes(&root, "cutscenes/intro/movie/ev0010.usm", 4, 0);
    put_json(
        &root,
        ".bundled_textures.json",
        &json!({"packs": ["HD (movie_mod@default)"]}),
    );

    let listed = list_installed(&game).unwrap();
    let installed = find(&listed, "movie_mod");
    assert_eq!(installed.bundled_cutscenes, vec!["intro".to_owned()]);
    assert_eq!(
        installed.bundled_texture_packs,
        vec!["HD (movie_mod@default)".to_owned()]
    );
}

#[test]
fn texture_packs_are_labelled_by_their_named_folder() {
    let dir = temp("yp_scan_packs_");
    let root = dir.path().join("Download");
    put_bytes(&root, "2B Pack/SK_Res/inject/textures/a.dds", 4, 0);
    put_bytes(&root, "A2 Pack/textures/b.dds", 4, 0);
    put_bytes(&root, "Wrapped/inject/Desert/c.dds", 4, 0);
    put_bytes(&root, "Wrapped/inject/City/d.dds", 4, 0);
    put_bytes(&root, "_skip/e.dds", 4, 0);

    let labels: Vec<String> = detect_texture_packs(&root)
        .into_iter()
        .map(|pack| pack.label)
        .collect();
    assert_eq!(labels, vec!["2B Pack", "A2 Pack", "City", "Desert"]);

    let flat = dir.path().join("Flat");
    put_bytes(&flat, "x.dds", 4, 0);
    let packs = detect_texture_packs(&flat);
    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0].label, "Flat");

    let with_payload = dir.path().join("Mod");
    put_bytes(&with_payload, "data/pl/pl0000.dat", 4, 0);
    put_bytes(&with_payload, "textures/x.dds", 4, 0);
    assert!(detect_texture_packs(&with_payload).is_empty());
}
