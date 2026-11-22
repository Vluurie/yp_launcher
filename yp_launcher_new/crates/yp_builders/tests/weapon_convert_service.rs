use std::collections::BTreeSet;
use std::path::Path;

use yp_builders::weapon::{self, WeaponDraft};
use yp_builders::weapon_convert;
use yp_builders::WeaponType;
use yp_core::GamePaths;
use yp_formats::testing::{model_archives, quad_spec};

#[test]
fn stem_maps_to_the_weapon_class_the_ranges_define() {
    assert_eq!(weapon_convert::stem_type("wp0003"), Some(WeaponType::SmallSword));
    assert_eq!(weapon_convert::stem_type("wp0240"), Some(WeaponType::LargeSword));
    assert_eq!(weapon_convert::stem_type("wp0450"), Some(WeaponType::Spear));
    assert_eq!(weapon_convert::stem_type("wp0610"), Some(WeaponType::CombatBracer));
}

#[test]
fn non_weapon_stems_are_refused() {
    assert_eq!(weapon_convert::stem_type("pl0000"), None);
    assert_eq!(weapon_convert::stem_type("em1234"), None);
    assert_eq!(weapon_convert::stem_type("wp"), None);
    assert_eq!(weapon_convert::stem_type("wp0800"), None);
}

#[test]
fn obj_id_and_stem_convert_back_and_forth() {
    for stem in ["wp0003", "wp0100", "wp0450", "wp0610"] {
        let weapon_type = weapon_convert::stem_type(stem).unwrap();
        let obj_id = 0x30000 + i64::from_str_radix(&stem[2..], 16).unwrap();
        assert_eq!(weapon_convert::stem_for_obj_id(obj_id), stem);
        assert_eq!(weapon::type_from_obj_id(obj_id), Some(weapon_type));
    }
}

#[test]
fn the_target_slot_is_always_free_and_of_the_same_class() {
    for weapon_type in WeaponType::ALL {
        let id = weapon::allocate_obj_id(weapon_type, &BTreeSet::new(), None).unwrap();
        assert!(weapon::is_obj_id_available(id));
        assert_eq!(weapon::type_from_obj_id(id), Some(weapon_type));
        let stem = weapon_convert::stem_for_obj_id(id);
        assert_eq!(weapon_convert::stem_type(&stem), Some(weapon_type));
    }
}

#[test]
fn a_file_that_is_not_a_weapon_is_rejected() {
    assert!(weapon_convert::is_weapon_file(Path::new("a/wp0003.dat")));
    assert!(weapon_convert::is_weapon_file(Path::new("a/wp0003.dtt")));
    assert!(!weapon_convert::is_weapon_file(Path::new("a/pl0000.dat")));
    assert!(!weapon_convert::is_weapon_file(Path::new("a/wp0003.wmb")));
}

#[test]
fn a_vanilla_weapon_pair_converts_into_a_free_slot_of_the_mod() {
    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("source");
    std::fs::create_dir_all(&source_dir).unwrap();
    let archives = model_archives("wp0003", quad_spec().build(), &[]);
    std::fs::write(source_dir.join("wp0003.dat"), &archives.dat).unwrap();
    std::fs::write(source_dir.join("wp0003.dtt"), &archives.dtt).unwrap();
    let game = GamePaths::new(dir.path().join("game"));

    assert_eq!(weapon_convert::inspect(&source_dir.join("missing.dat")), None);
    let source = weapon_convert::inspect(&source_dir.join("wp0003.dtt")).unwrap();
    assert_eq!(source.stem, "wp0003");
    assert_eq!(source.weapon_type, WeaponType::SmallSword);

    let target = weapon::allocate_obj_id(source.weapon_type, &weapon_convert::taken_obj_ids(&game), None).unwrap();
    let renamed = weapon_convert::convert(&source, target, &game, "my_mod").unwrap();

    let model_dir = weapon_convert::model_dir_for(&game, "my_mod");
    assert_eq!(renamed.to_stem, weapon_convert::stem_for_obj_id(target));
    assert_eq!(renamed.dat_path, model_dir.join(format!("{}.dat", renamed.to_stem)));
    assert!(renamed.dtt_path.is_file());
    assert!(weapon_convert::taken_obj_ids(&game).contains(&target));

    let mut draft = WeaponDraft::new("Converted", "x", WeaponType::SmallSword, 1100, 0x30101);
    draft.id = 1100;
    weapon::write(&game, "my_mod", &draft).unwrap();
    let taken = weapon_convert::taken_obj_ids(&game);
    assert!(taken.contains(&0x30101));
    assert!(taken.contains(&target));
}

#[test]
fn converting_into_another_class_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let archives = model_archives("wp0003", quad_spec().build(), &[]);
    std::fs::write(dir.path().join("wp0003.dat"), &archives.dat).unwrap();
    std::fs::write(dir.path().join("wp0003.dtt"), &archives.dtt).unwrap();
    let game = GamePaths::new(dir.path().join("game"));
    let source = weapon_convert::inspect(&dir.path().join("wp0003.dat")).unwrap();

    let result = weapon_convert::convert(&source, 0x30401, &game, "my_mod");

    assert!(matches!(
        result,
        Err(yp_builders::Error::Format(yp_formats::FormatError::WeaponClassMismatch { .. }))
    ));
}
