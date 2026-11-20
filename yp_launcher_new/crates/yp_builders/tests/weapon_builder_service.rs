use std::collections::BTreeSet;

use yp_builders::weapon::{self, WeaponLevel, WPMASK_OBJ_IDS};
use yp_builders::{WeaponDraft, WeaponType};

fn draft_with(weapon_type: WeaponType, obj_id: i64, levels: Vec<WeaponLevel>, display_name: &str) -> WeaponDraft {
    let mut draft = WeaponDraft::new(display_name, "A blade", weapon_type, 8000, obj_id);
    draft.levels = levels;
    draft
}

fn plain() -> WeaponDraft {
    draft_with(WeaponType::SmallSword, 0x30000, Vec::new(), "Test Blade")
}

fn none() -> BTreeSet<i64> {
    BTreeSet::new()
}

mod obj_id_ranges {
    use super::*;

    #[test]
    fn each_type_maps_to_its_own_block() {
        assert_eq!(weapon::type_from_obj_id(0x30001), Some(WeaponType::SmallSword));
        assert_eq!(weapon::type_from_obj_id(0x30201), Some(WeaponType::LargeSword));
        assert_eq!(weapon::type_from_obj_id(0x30401), Some(WeaponType::Spear));
        assert_eq!(weapon::type_from_obj_id(0x30601), Some(WeaponType::CombatBracer));
    }

    #[test]
    fn anything_outside_every_range_has_no_type() {
        assert_eq!(weapon::type_from_obj_id(0x2FFFF), None);
        assert_eq!(weapon::type_from_obj_id(0x30800), None);
    }

    #[test]
    fn vanilla_ids_are_refused() {
        assert!(!weapon::is_obj_id_available(0x30003));
        assert!(!weapon::is_obj_id_available(0x30655));
    }

    #[test]
    fn wpmask_ids_are_refused() {
        for id in WPMASK_OBJ_IDS {
            assert!(!weapon::is_obj_id_available(id), "0x{id:x} is engine-reserved");
        }
    }

    #[test]
    fn a_free_id_inside_a_range_is_accepted() {
        assert!(weapon::is_obj_id_available(0x30100));
    }
}

mod allocation {
    use super::*;

    #[test]
    fn picks_a_usable_id_for_every_type() {
        for weapon_type in WeaponType::ALL {
            let id = weapon::allocate_obj_id(weapon_type, &none(), None).unwrap();
            assert!(weapon::is_obj_id_available(id));
            assert_eq!(weapon::type_from_obj_id(id), Some(weapon_type));
        }
    }

    #[test]
    fn different_mod_names_land_on_different_slots() {
        let names = ["awakened_protocol", "flaming_sword", "my_weapon", "rusty_blade", "moonlight", "test"];
        let picked: BTreeSet<i64> = names
            .iter()
            .map(|name| weapon::allocate_obj_id(WeaponType::SmallSword, &none(), Some(name)).unwrap())
            .collect();

        assert_eq!(picked.len(), names.len(), "two mods would ship the same weapon number");
    }

    #[test]
    fn the_same_mod_name_always_gives_the_same_slot() {
        let first = weapon::allocate_obj_id(WeaponType::Spear, &none(), Some("my_spear"));
        let second = weapon::allocate_obj_id(WeaponType::Spear, &none(), Some("my_spear"));

        assert_eq!(first, second);
    }

    #[test]
    fn a_seeded_slot_is_still_valid_and_of_the_right_class() {
        for weapon_type in WeaponType::ALL {
            let id = weapon::allocate_obj_id(weapon_type, &none(), Some("some_mod_name")).unwrap();
            assert!(weapon::is_obj_id_available(id));
            assert_eq!(weapon::type_from_obj_id(id), Some(weapon_type));
        }
    }

    #[test]
    fn a_taken_seeded_slot_moves_on_to_the_next_free_one() {
        let first = weapon::allocate_obj_id(WeaponType::LargeSword, &none(), Some("taken_test")).unwrap();
        let second =
            weapon::allocate_obj_id(WeaponType::LargeSword, &BTreeSet::from([first]), Some("taken_test")).unwrap();

        assert_ne!(second, first);
        assert_eq!(weapon::type_from_obj_id(second), Some(WeaponType::LargeSword));
    }

    #[test]
    fn skips_ids_already_used_by_another_mod() {
        let first = weapon::allocate_obj_id(WeaponType::Spear, &none(), None).unwrap();
        let second = weapon::allocate_obj_id(WeaponType::Spear, &BTreeSet::from([first]), None).unwrap();

        assert_ne!(second, first);
        assert_eq!(weapon::type_from_obj_id(second), Some(WeaponType::Spear));
    }

    #[test]
    fn the_seed_hash_matches_the_dart_fnv_variant() {
        assert_eq!(weapon::seed_hash(""), 0x811c_9dc5);
        assert_eq!(weapon::seed_hash("a"), ((0x811c_9dc5_i64 ^ 0x61) * 0x0100_0193) & 0x7fff_ffff);
        assert_eq!(weapon::seed_hash("MY_SPEAR"), weapon::seed_hash("my_spear"));
    }
}

mod to_toml {
    use super::*;

    #[test]
    fn category_always_matches_the_type_so_they_cannot_disagree() {
        for weapon_type in WeaponType::ALL {
            let obj_id = weapon::allocate_obj_id(weapon_type, &none(), None).unwrap();
            let toml = weapon::to_toml(&draft_with(weapon_type, obj_id, Vec::new(), "Test Blade"));

            assert!(toml.contains(&format!("category = {}", weapon_type.category())));
            assert_eq!(weapon::type_from_obj_id(obj_id), Some(weapon_type), "written obj_id must sit in the category range");
        }
    }

    #[test]
    fn writes_obj_id_in_hex_so_it_matches_the_documented_ranges() {
        let toml = weapon::to_toml(&draft_with(WeaponType::SmallSword, 0x30100, Vec::new(), "Test Blade"));
        assert!(toml.contains("obj_id = 0x30100"));
    }

    #[test]
    fn effect_ids_follow_the_weapon_type() {
        let toml = weapon::to_toml(&draft_with(WeaponType::Spear, 0x30401, Vec::new(), "Test Blade"));

        assert!(toml.contains("est_id_normal = 620"));
        assert!(toml.contains("est_id_berserk = 635"));
    }

    #[test]
    fn always_clones_stats_from_a_vanilla_weapon_of_the_same_class() {
        let toml = weapon::to_toml(&plain());
        assert!(toml.contains("[params]"));
        assert!(toml.contains("clone_from_obj_id = 0x30003"));
        assert!(!toml.contains("levels = ["));
    }

    #[test]
    fn writes_the_levels_as_one_inline_array() {
        let toml = weapon::to_toml(&draft_with(
            WeaponType::SmallSword,
            0x30000,
            vec![
                WeaponLevel { min_damage: 10, max_damage: 20 },
                WeaponLevel { min_damage: 30, max_damage: 40 },
            ],
            "Test Blade",
        ));

        assert!(toml.contains("levels = ["));
        assert!(toml.contains("min_damage = 10"));
        assert!(toml.contains("max_damage = 40"));
    }

    #[test]
    fn writes_one_shop_block_per_story_state() {
        let mut draft = plain();
        draft.shop_name = Some("shop_RC_Grocery".to_owned());
        draft.shop_states = BTreeSet::from([2, 0]);

        let toml = weapon::to_toml(&draft);

        assert_eq!(toml.matches("[[shops]]").count(), 2);
        assert!(toml.find("state = 0").unwrap() < toml.find("state = 2").unwrap());
        assert!(toml.contains("buy = 5000"));
    }

    #[test]
    fn internal_name_is_prefixed_and_cleaned() {
        let draft = draft_with(WeaponType::SmallSword, 0x30000, Vec::new(), "The \"Best\" Blade");
        assert_eq!(draft.internal_name(), "weapon_the_best_blade");
        assert_eq!(draft_with(WeaponType::SmallSword, 0x30000, Vec::new(), "!!!").internal_name(), "weapon_unnamed");
    }

    #[test]
    fn escapes_quotes() {
        let toml = weapon::to_toml(&draft_with(WeaponType::SmallSword, 0x30000, Vec::new(), "A \"Quoted\" Blade"));
        assert!(toml.contains(r#"\"Quoted\""#));
        let values: toml::Table = toml::from_str(&toml).unwrap();
        assert_eq!(values["text"]["name"]["_us"].as_str(), Some("A \"Quoted\" Blade"));
    }
}

mod est_ids {
    use super::*;

    #[test]
    fn blocked_and_out_of_range_ids_are_refused_and_clamped() {
        assert!(weapon::is_est_id_allowed(0));
        assert!(weapon::is_est_id_allowed(665));
        assert!(!weapon::is_est_id_allowed(435));
        assert!(!weapon::is_est_id_allowed(666));
        assert_eq!(weapon::clamp_est_id(435, true), 437);
        assert_eq!(weapon::clamp_est_id(436, false), 434);
        assert_eq!(weapon::clamp_est_id(900, true), 665);
        assert_eq!(weapon::clamp_est_id(-3, false), 0);
    }
}

#[test]
fn file_name_follows_the_weapon_id() {
    assert_eq!(weapon::file_name_for(&plain()), "weapon_8000.toml");
}

#[test]
fn taken_ids_come_from_written_files() {
    let dir = tempfile::tempdir().unwrap();
    let game = yp_core::GamePaths::new(dir.path());
    let mut first = draft_with(WeaponType::SmallSword, 0x30100, Vec::new(), "First");
    first.id = 1100;
    weapon::write(&game, "mod_a", &first).unwrap();
    yp_builders::item::write(&game, "mod_b", &yp_builders::ItemDraft::new("Nut", "A nut", 1101)).unwrap();

    assert_eq!(weapon::taken_obj_ids(&game), BTreeSet::from([0x30100]));
    assert_eq!(weapon::taken_ids(&game), BTreeSet::from([1100, 1101]));
    assert_eq!(weapon::next_free_id(&game, None), Some(1000));
    let seeded = weapon::next_free_id(&game, Some("first")).unwrap();
    assert!(!weapon::VANILLA_IDS.contains(&seeded));
    assert!((weapon::FIRST_ID..=weapon::LAST_ID).contains(&seeded));
}
