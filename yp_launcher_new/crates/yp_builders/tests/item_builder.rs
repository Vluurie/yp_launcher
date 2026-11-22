use yp_builders::item::{self, internal_name_for};
use yp_builders::ItemDraft;
use yp_core::GamePaths;

fn parse(text: &str) -> toml::Table {
    toml::from_str(text).unwrap()
}

mod internal_name_for {
    use super::*;

    #[test]
    fn turns_a_display_name_into_a_safe_identifier() {
        assert_eq!(internal_name_for("My Thing"), "item_my_thing");
        assert_eq!(internal_name_for("Rusty Bolt"), "item_rusty_bolt");
    }

    #[test]
    fn drops_punctuation_and_collapses_separators() {
        assert_eq!(internal_name_for("Engineer's  Wrench!"), "item_engineer_s_wrench");
    }

    #[test]
    fn keeps_digits() {
        assert_eq!(internal_name_for("Type 40 Sword"), "item_type_40_sword");
    }

    #[test]
    fn falls_back_when_nothing_usable_is_left() {
        assert_eq!(internal_name_for("!!!"), "item_unnamed");
        assert_eq!(internal_name_for(""), "item_unnamed");
    }

    #[test]
    fn handles_non_latin_input_without_producing_a_bare_prefix() {
        assert_eq!(internal_name_for("わたしのもの"), "item_unnamed");
    }

    #[test]
    fn always_starts_with_item_which_the_engine_requires() {
        for input in ["My Thing", "item already prefixed", "   ", "???", "わたしのもの", "42"] {
            assert!(internal_name_for(input).starts_with("item_"), "input: {input:?}");
        }
    }

    #[test]
    fn never_leaves_a_bare_prefix_or_a_trailing_underscore() {
        for input in ["", "!", "x!", "a  b  "] {
            let name = internal_name_for(input);
            assert_ne!(name, "item_");
            assert!(!name.ends_with('_'), "input: {input:?}");
        }
    }
}

mod to_toml {
    use super::*;

    #[test]
    fn produces_valid_toml_with_the_required_fields() {
        let values = parse(&item::to_toml(&ItemDraft::new("My Thing", "Something useful.", 8500)));

        assert_eq!(values["name"].as_str(), Some("item_my_thing"));
        assert_eq!(values["id_type"].as_integer(), Some(0));
        assert_eq!(values["text"]["name"]["_us"].as_str(), Some("My Thing"));
        assert_eq!(values["text"]["help"]["_us"].as_str(), Some("Something useful."));
    }

    #[test]
    fn omits_max_num_when_it_is_the_default() {
        assert!(!item::to_toml(&ItemDraft::new("A", "B", 8000)).contains("max_num"));
    }

    #[test]
    fn writes_max_num_when_the_player_can_carry_more() {
        let draft = ItemDraft {
            max_carry: 99,
            ..ItemDraft::new("A", "B", 8000)
        };
        assert_eq!(parse(&item::to_toml(&draft))["max_num"].as_integer(), Some(99));
    }

    #[test]
    fn sellable_writes_a_price_otherwise_no_trash() {
        let sellable = ItemDraft {
            sellable: true,
            sell_price: 250,
            ..ItemDraft::new("A", "B", 8000)
        };
        let with_price = parse(&item::to_toml(&sellable));
        assert_eq!(with_price["sell"].as_integer(), Some(250));
        assert!(!with_price.contains_key("no_trash"));

        let locked = parse(&item::to_toml(&ItemDraft::new("A", "B", 8000)));
        assert_eq!(locked["no_trash"].as_bool(), Some(true));
    }

    #[test]
    fn auto_give_can_be_switched_off() {
        let draft = ItemDraft {
            auto_give: false,
            ..ItemDraft::new("A", "B", 8000)
        };
        assert!(!item::to_toml(&draft).contains("auto_give"));
    }

    #[test]
    fn quotes_in_user_text_do_not_break_the_file() {
        let draft = ItemDraft::new("The \"Real\" Thing", r#"Has a \ backslash and a " quote."#, 8000);
        let values = parse(&item::to_toml(&draft));
        assert_eq!(values["text"]["name"]["_us"].as_str(), Some("The \"Real\" Thing"));
        assert_eq!(values["text"]["help"]["_us"].as_str(), Some(r#"Has a \ backslash and a " quote."#));
    }

    #[test]
    fn writes_an_icon_section_only_when_a_texture_is_set() {
        assert!(!item::to_toml(&ItemDraft::new("A", "B", 8000)).contains("[icon]"));

        let with_icon = ItemDraft {
            misctex_name: Some("item_thumb_8500".to_owned()),
            ..ItemDraft::new("A", "B", 8500)
        };
        assert_eq!(parse(&item::to_toml(&with_icon))["icon"]["misctex_name"].as_str(), Some("item_thumb_8500"));
    }

    #[test]
    fn never_writes_an_id_since_the_filename_carries_it() {
        let draft = ItemDraft::new("A", "B", 8500);
        assert!(!parse(&item::to_toml(&draft)).contains_key("id"));
        assert_eq!(item::file_name_for(&draft), "item_8500.toml");
    }

    #[test]
    fn the_layout_matches_the_dart_builder() {
        let toml = item::to_toml(&ItemDraft::new("My Thing", "Line one  \r\nLine two", 8500));
        assert_eq!(
            toml,
            "name = \"item_my_thing\"\nid_type = 0\nno_trash = true\n\n[auto_give]\nquantity = 1\n\n\
             [text.name]\n_us = \"My Thing\"\n\n[text.help]\n_us = '''\nLine one\nLine two'''\n"
        );
    }
}

mod id_from_file_name {
    use super::*;

    #[test]
    fn reads_the_id_back_out() {
        assert_eq!(item::id_from_file_name("item_8500.toml"), Some(8500));
    }

    #[test]
    fn ignores_files_that_are_not_items() {
        assert_eq!(item::id_from_file_name("weapon_1500.toml"), None);
        assert_eq!(item::id_from_file_name("item_abc.toml"), None);
        assert_eq!(item::id_from_file_name("item_8500.md"), None);
    }
}

mod next_free_id {
    use super::*;

    #[test]
    fn starts_at_8000_on_an_empty_install() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(item::next_free_id(&GamePaths::new(dir.path())), 8000);
    }

    #[test]
    fn skips_ids_already_used_by_any_mod() {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path());
        let draft = ItemDraft::new("A", "B", 8000);
        item::write(&game, "mod_a", &draft).unwrap();
        item::write(&game, "mod_b", &ItemDraft { id: 8001, ..draft }).unwrap();

        assert_eq!(item::next_free_id(&game), 8002);
    }
}

#[test]
fn write_puts_the_file_under_entities() {
    let dir = tempfile::tempdir().unwrap();
    let game = GamePaths::new(dir.path());

    let file = item::write(&game, "my_mod", &ItemDraft::new("My Thing", "Something useful.", 8500)).unwrap();

    assert!(file.is_file());
    assert_eq!(file, dir.path().join("nams").join("mods").join("my_mod").join("entities").join("item_8500.toml"));
}
