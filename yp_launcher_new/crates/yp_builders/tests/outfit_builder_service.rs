use std::collections::BTreeSet;
use std::path::Path;

use yp_builders::outfit::{self, MeshLists, MeshNames, MAX_SHOP_STATE, SHOP_NAMES};
use yp_builders::{MeshRule, MeshState, MeshVisibility, OutfitCharacter, OutfitDraft};

fn draft_with(character: OutfitCharacter, meshes: &[(MeshState, &[&str])], display_name: &str) -> OutfitDraft {
    let mut draft = OutfitDraft::new(display_name, "A dress", character, 10001, 9100);
    draft.meshes = meshes
        .iter()
        .map(|(state, names)| (*state, names.iter().collect::<MeshNames>()))
        .collect::<MeshLists>();
    draft
}

fn plain() -> OutfitDraft {
    draft_with(OutfitCharacter::TwoB, &[], "Black Dress")
}

fn names(lists: &MeshLists, state: MeshState) -> Vec<&str> {
    lists.get(&state).map(MeshNames::sorted).unwrap_or_default()
}

fn rule(visibility: MeshVisibility) -> MeshRule {
    MeshRule {
        visibility,
        ..MeshRule::default()
    }
}

mod to_toml {
    use super::*;

    #[test]
    fn writes_the_required_fields() {
        let toml = outfit::to_toml(&plain());

        assert!(toml.contains("outfit_id = 10001"));
        assert!(toml.contains("character = \"2B\""));
        assert!(toml.contains("item_id = 9100"));
        assert!(toml.contains("name = \"black_dress\""));
    }

    #[test]
    fn omits_mesh_lists_entirely_when_nothing_is_selected() {
        assert!(!outfit::to_toml(&plain()).contains("meshes"));
    }

    #[test]
    fn writes_only_the_states_that_have_meshes() {
        let toml = outfit::to_toml(&draft_with(
            OutfitCharacter::TwoB,
            &[
                (MeshState::Base, &["Body", "Hair"]),
                (MeshState::Combat, &["facial_serious"]),
                (MeshState::NonCombat, &[]),
            ],
            "Black Dress",
        ));

        assert!(toml.contains("meshes = [\"Body\", \"Hair\"]"));
        assert!(toml.contains("combat_meshes = [\"facial_serious\"]"));
        assert!(!toml.contains("non_combat_meshes"));
        assert!(toml.contains("name = \"black_dress\"\n\nmeshes = "), "{toml}");
    }

    #[test]
    fn drops_states_that_do_not_apply_to_the_character() {
        let toml = outfit::to_toml(&draft_with(
            OutfitCharacter::A2,
            &[
                (MeshState::Wig, &["Hair"]),
                (MeshState::Eyemask, &["Eyemask"]),
                (MeshState::Tower, &["Body"]),
            ],
            "Black Dress",
        ));

        assert!(toml.contains("wig_meshes = [\"Hair\"]"));
        assert!(!toml.contains("eyemask_meshes"));
        assert!(!toml.contains("tower_meshes"));
    }

    #[test]
    fn nine_s_keeps_the_story_states_two_b_does_not() {
        let tower: &[(MeshState, &[&str])] = &[(MeshState::Tower, &["mesh_es0202"])];
        let nine_s = outfit::to_toml(&draft_with(OutfitCharacter::NineS, tower, "Black Dress"));
        let two_b = outfit::to_toml(&draft_with(OutfitCharacter::TwoB, tower, "Black Dress"));

        assert!(nine_s.contains("tower_meshes"));
        assert!(!two_b.contains("tower_meshes"));
    }

    #[test]
    fn escapes_quotes_in_mesh_names_and_the_outfit_name() {
        let toml = outfit::to_toml(&draft_with(
            OutfitCharacter::TwoB,
            &[(MeshState::Base, &["Od\"d"])],
            "The \"Best\" Dress",
        ));

        assert!(toml.contains(r#""Od\"d""#));
        assert!(toml.contains("name = \"the_best_dress\""));
        let values: toml::Table = toml::from_str(&toml).unwrap();
        assert_eq!(values["meshes"][0].as_str(), Some("Od\"d"));
    }
}

mod character_detection {
    use super::*;

    #[test]
    fn maps_every_known_model_stem_to_its_character() {
        assert_eq!(outfit::character_from_model_path(Path::new("mods/x/pl0000.dat")), Some(OutfitCharacter::TwoB));
        assert_eq!(outfit::character_from_model_path(Path::new("mods/x/pl020d.dtt")), Some(OutfitCharacter::NineS));
        assert_eq!(outfit::character_from_model_path(Path::new("mods/x/pl0100.dat")), Some(OutfitCharacter::A2));
    }

    #[test]
    fn is_case_insensitive() {
        assert_eq!(outfit::character_from_model_path(Path::new("X/PL000D.DAT")), Some(OutfitCharacter::TwoB));
    }

    #[test]
    fn returns_none_for_something_that_is_not_a_player_model() {
        assert_eq!(outfit::character_from_model_path(Path::new("mods/x/em0100.dat")), None);
    }
}

mod untouched_meshes {
    use super::*;

    #[test]
    fn write_no_lists_at_all_so_nams_keeps_its_own_assembly() {
        let lists = outfit::lists_from_rules(
            OutfitCharacter::TwoB,
            &[
                ("Body", MeshRule::default()),
                ("Hair", MeshRule::default()),
                ("Armor_Body", MeshRule::default()),
                ("DLC_Body", MeshRule::default()),
            ],
        );

        assert!(lists.is_empty());
    }

    #[test]
    fn one_changed_mesh_switches_the_whole_outfit_to_custom_lists() {
        let lists = outfit::lists_from_rules(
            OutfitCharacter::TwoB,
            &[("Body", MeshRule::default()), ("Armor_Body", rule(MeshVisibility::Never))],
        );

        assert_eq!(names(&lists, MeshState::Base), ["Body"]);
    }
}

mod rules_produce_the_vanilla_lists {
    use super::*;

    #[test]
    fn two_b_normal() {
        let lists = outfit::lists_from_rules(
            OutfitCharacter::TwoB,
            &[
                ("Body", MeshRule::default()),
                ("Hair", MeshRule { tinted_by_hairspray: true, ..MeshRule::default() }),
                (
                    "Skirt",
                    MeshRule {
                        visibility: MeshVisibility::FeatherOnly,
                        hide_on_self_destruct: true,
                        ..MeshRule::default()
                    },
                ),
                ("Feather", rule(MeshVisibility::FeatherOnly)),
                (
                    "Broken",
                    MeshRule {
                        visibility: MeshVisibility::BrokenOnly,
                        hide_on_self_destruct: true,
                        ..MeshRule::default()
                    },
                ),
                ("Eyemask", rule(MeshVisibility::EyemaskOnly)),
                ("Eyelash", rule(MeshVisibility::NoEyemaskOnly)),
                ("facial_serious", rule(MeshVisibility::CombatOnly)),
                ("facial_normal", rule(MeshVisibility::OutOfCombatOnly)),
            ],
        );

        assert_eq!(names(&lists, MeshState::Base), ["Body", "Hair"]);
        assert_eq!(names(&lists, MeshState::NonBroken), ["Feather", "Skirt"]);
        assert_eq!(names(&lists, MeshState::Broken), ["Broken"]);
        assert_eq!(names(&lists, MeshState::SelfDestructHide), ["Broken", "Skirt"]);
        assert_eq!(names(&lists, MeshState::Eyemask), ["Eyemask"]);
        assert_eq!(names(&lists, MeshState::NonEyemask), ["Eyelash"]);
        assert_eq!(names(&lists, MeshState::Combat), ["facial_serious"]);
        assert_eq!(names(&lists, MeshState::NonCombat), ["facial_normal"]);
        assert_eq!(names(&lists, MeshState::Hairspray), ["Hair"]);
    }

    #[test]
    fn a2_normal() {
        let lists = outfit::lists_from_rules(
            OutfitCharacter::A2,
            &[
                ("Body", MeshRule::default()),
                ("Cloth", MeshRule { hide_on_self_destruct: true, ..MeshRule::default() }),
                (
                    "Hair",
                    MeshRule {
                        tinted_by_hairspray: true,
                        replaced_by_wig: true,
                        ..MeshRule::default()
                    },
                ),
                ("facial_serious", rule(MeshVisibility::CombatOnly)),
                ("facial_normal", rule(MeshVisibility::OutOfCombatOnly)),
            ],
        );

        assert_eq!(names(&lists, MeshState::Base), ["Body", "Cloth", "Hair"]);
        assert_eq!(names(&lists, MeshState::SelfDestructHide), ["Cloth"]);
        assert_eq!(names(&lists, MeshState::Wig), ["Hair"]);
        assert_eq!(names(&lists, MeshState::Hairspray), ["Hair"]);
        assert_eq!(names(&lists, MeshState::Combat), ["facial_serious"]);
    }

    #[test]
    fn two_b_full_armor_needs_only_the_base_list() {
        let lists = outfit::lists_from_rules(
            OutfitCharacter::TwoB,
            &[
                ("Armor_Body", MeshRule::default()),
                ("Armor_Head", MeshRule::default()),
                ("Body", rule(MeshVisibility::Never)),
                ("Hair", rule(MeshVisibility::Never)),
            ],
        );

        assert_eq!(names(&lists, MeshState::Base), ["Armor_Body", "Armor_Head"]);
        assert_eq!(lists.len(), 1);
    }

    #[test]
    fn combat_and_non_combat_can_never_be_half_filled() {
        let lists = outfit::lists_from_rules(
            OutfitCharacter::TwoB,
            &[("a", rule(MeshVisibility::CombatOnly)), ("b", rule(MeshVisibility::OutOfCombatOnly))],
        );

        assert_eq!(names(&lists, MeshState::Combat), ["a"]);
        assert_eq!(names(&lists, MeshState::NonCombat), ["b"]);
    }

    #[test]
    fn a_mesh_set_to_never_appears_in_no_list_at_all() {
        let lists = outfit::lists_from_rules(OutfitCharacter::TwoB, &[("gone", rule(MeshVisibility::Never))]);

        assert!(lists.is_empty());
    }

    #[test]
    fn a_state_the_character_lacks_falls_back_to_always_visible() {
        let lists = outfit::lists_from_rules(OutfitCharacter::A2, &[("x", rule(MeshVisibility::EyemaskOnly))]);

        assert_eq!(names(&lists, MeshState::Base), ["x"]);
        assert!(!lists.contains_key(&MeshState::Eyemask));
    }
}

mod archive_pairing {
    use super::*;

    #[test]
    fn finds_the_sibling_of_the_other_extension() {
        assert!(outfit::sibling_path(Path::new(r"C:\mods\pl0000.dat"), ".dtt").to_string_lossy().ends_with("pl0000.dtt"));
        assert!(outfit::sibling_path(Path::new(r"C:\mods\pl0000.dtt"), ".dat").to_string_lossy().ends_with("pl0000.dat"));
    }

    #[test]
    fn keeps_the_sibling_in_the_same_folder() {
        let sibling = outfit::sibling_path(Path::new(r"C:\mods\deep\pl0000.dat"), ".dtt");
        assert!(sibling.to_string_lossy().contains("deep"));
    }

    #[test]
    fn existing_sibling_and_model_copy_use_the_pair() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("src");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("pl0000.dat"), b"dat").unwrap();
        std::fs::write(source.join("pl0000.dtt"), b"dtt").unwrap();
        assert_eq!(
            outfit::existing_sibling(&source.join("pl0000.dat"), ".dtt"),
            Some(source.join("pl0000.dtt"))
        );
        assert_eq!(outfit::existing_sibling(&source.join("pl0000.dat"), ".wmb"), None);

        let game = yp_core::GamePaths::new(dir.path().join("game"));
        let copied = outfit::copy_model(&game, "my_mod", &source.join("pl0000.dtt")).unwrap();

        let model_dir = outfit::model_dir_for(&game, "my_mod");
        assert_eq!(copied, [model_dir.join("pl0000.dat"), model_dir.join("pl0000.dtt")]);
        assert_eq!(std::fs::read(model_dir.join("pl0000.dtt")).unwrap(), b"dtt");
    }
}

mod is_model_file {
    use super::*;

    #[test]
    fn accepts_dat_and_dtt_in_any_case() {
        assert!(outfit::is_model_file(Path::new("a/pl0000.dat")));
        assert!(outfit::is_model_file(Path::new("a/pl0000.DTT")));
    }

    #[test]
    fn rejects_anything_else() {
        assert!(!outfit::is_model_file(Path::new("a/pl0000.wmb")));
        assert!(!outfit::is_model_file(Path::new("a/icon.png")));
    }
}

mod shop_entry {
    use super::*;

    fn shop(states: &[i64]) -> OutfitDraft {
        let mut draft = plain();
        draft.shop_name = Some("shop_RC_Grocery".to_owned());
        draft.shop_states = states.iter().copied().collect();
        draft
    }

    #[test]
    fn writes_shop_name_item_id_and_state() {
        let toml = outfit::shop_item_toml(&shop(&[2]));

        assert!(toml.contains("shop_name = \"shop_RC_Grocery\""));
        assert!(toml.contains("item_id = 9100"));
        assert!(toml.contains("state = 2"));
    }

    #[test]
    fn writes_one_entry_per_story_state_in_order() {
        let toml = outfit::shop_item_toml(&shop(&[3, 0, 1]));

        assert_eq!(toml.matches("[[shop_item]]").count(), 3);
        assert!(toml.find("state = 0").unwrap() < toml.find("state = 1").unwrap());
        assert!(toml.find("state = 1").unwrap() < toml.find("state = 3").unwrap());
    }

    #[test]
    fn names_the_entity_kind_so_the_file_name_is_not_parsed_as_one() {
        assert!(outfit::shop_item_toml(&shop(&[0])).starts_with("[[shop_item]]"));
    }

    #[test]
    fn the_file_name_has_no_underscore_nams_could_split_on() {
        let name = outfit::shop_file_name_for(&plain());
        let stem = name.strip_suffix(".toml").unwrap();
        assert!(!stem.contains('_'));
    }

    #[test]
    fn is_only_requested_when_a_shop_is_set() {
        assert!(!plain().sells_in_shop());
        assert!(shop(&[0]).sells_in_shop());
    }

    #[test]
    fn clear_shop_removes_it_again() {
        let mut draft = shop(&[0]);
        draft.shop_name = None;
        assert!(!draft.sells_in_shop());
    }

    #[test]
    fn shop_file_is_named_after_the_outfit() {
        assert_eq!(outfit::shop_file_name_for(&plain()), "shop9100.toml");
    }

    #[test]
    fn every_offered_shop_name_is_a_real_vanilla_shop() {
        let vanilla: BTreeSet<&str> = [
            "shop_AP_Grocery", "shop_BK_Grocery", "shop_CD_Grocery", "shop_CF_Grocery", "shop_CS_Grocery",
            "shop_DC_Grocery", "shop_DO_Grocery", "shop_DP_Grocery", "shop_Emile", "shop_FC_Grocery", "shop_FC_Smith",
            "shop_GP_Grocery", "shop_PC_Grocery", "shop_PC_Grocery_first", "shop_PV_Grocery", "shop_PV_Smith",
            "shop_PV_UniqueShop", "shop_RC_Grocery", "shop_RC_Grocery_small", "shop_RC_Smith", "shop_TEST_Chip1",
            "shop_TEST_Chip2", "shop_TEST_Chip3", "shop_TEST_Chip4", "shop_TEST_Chip5", "shop_TEST_Chip6",
            "shop_TEST_Grocery", "shop_TEST_Smith", "shop_Upgrade", "shop_VF_Grocery", "shop_chipEmile",
            "shop_rareChipEmile", "shop_rareEmile",
        ]
        .into_iter()
        .collect();

        for name in SHOP_NAMES {
            assert!(vanilla.contains(name), "{name} is not a vanilla shop");
        }
    }

    #[test]
    fn state_stays_inside_the_range_nams_accepts() {
        assert_eq!(MAX_SHOP_STATE, 6);
    }

    #[test]
    fn write_shop_item_lands_under_entities() {
        let dir = tempfile::tempdir().unwrap();
        let game = yp_core::GamePaths::new(dir.path());
        let file = outfit::write_shop_item(&game, "my_mod", &shop(&[1])).unwrap();
        assert_eq!(file, game.mods_dir().join("my_mod").join("entities").join("shop9100.toml"));
        assert!(file.is_file());
    }
}

#[test]
fn file_name_follows_the_outfit_id() {
    assert_eq!(outfit::file_name_for(&plain()), "outfit_10001.toml");
}

#[test]
fn each_character_starts_in_its_own_id_block() {
    assert_eq!(OutfitCharacter::TwoB.first_outfit_id(), 10001);
    assert_eq!(OutfitCharacter::NineS.first_outfit_id(), 20001);
    assert_eq!(OutfitCharacter::A2.first_outfit_id(), 30001);
}

#[test]
fn next_free_outfit_id_skips_ids_of_the_same_character_only() {
    let dir = tempfile::tempdir().unwrap();
    let game = yp_core::GamePaths::new(dir.path());
    outfit::write(&game, "mod_a", &plain()).unwrap();
    let mut nine_s = draft_with(OutfitCharacter::NineS, &[], "Scanner Coat");
    nine_s.outfit_id = 20001;
    outfit::write(&game, "mod_b", &nine_s).unwrap();
    std::fs::write(game.mods_dir().join("mod_b").join("entities").join("outfit_broken.toml"), "outfit_id = = 3").unwrap();

    assert_eq!(outfit::taken_outfit_ids(&game, OutfitCharacter::TwoB), BTreeSet::from([10001]));
    assert_eq!(outfit::next_free_outfit_id(&game, OutfitCharacter::TwoB), 10002);
    assert_eq!(outfit::next_free_outfit_id(&game, OutfitCharacter::NineS), 20002);
    assert_eq!(outfit::next_free_outfit_id(&game, OutfitCharacter::A2), 30001);
}
