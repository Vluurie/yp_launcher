use std::collections::BTreeSet;

use yp_builders::outfit::{self, MeshLists, MeshNames};
use yp_builders::weapon::{self, WeaponEffect, WeaponLevel, WeaponMaterial, WeaponUpgrade, EFFECT_PATHS};
use yp_builders::{first_mod, item, ItemDraft, MeshState, OutfitCharacter, OutfitDraft, WeaponDraft, WeaponType};

const AWKWARD_TEXTS: [&str; 9] = [
    "",
    "plain",
    "with \"quotes\"",
    r"back\slash",
    "multi\nline\r\ntext  \n",
    "ends with a quote'",
    "ends with two quotes''",
    "has ''' triple",
    "control \u{1} and tab\t and bell\u{7}",
];

fn parse(text: &str) -> toml::Table {
    toml::from_str(text).unwrap_or_else(|err| panic!("{err}\n{text}"))
}

fn normalised(text: &str) -> String {
    let mut lines: Vec<&str> = text.split(['\n']).map(|line| line.trim_end_matches('\r').trim_end()).collect();
    if text.ends_with('\n') {
        lines.pop();
    }
    lines.join("\n")
}

#[test]
fn item_text_survives_any_input() {
    for text in AWKWARD_TEXTS {
        let draft = ItemDraft {
            misctex_name: Some(text.to_owned()),
            buy_price: Some(-1),
            ..ItemDraft::new(text, text, 8000)
        };
        let values = parse(&item::to_toml(&draft));
        assert_eq!(values["text"]["name"]["_us"].as_str(), Some(text));
        assert_eq!(values["text"]["help"]["_us"].as_str(), Some(normalised(text).as_str()), "{text:?}");
        assert_eq!(values["buy"].as_integer(), Some(-1));
        if !text.is_empty() {
            assert_eq!(values["icon"]["misctex_name"].as_str(), Some(text));
        }
    }
}

#[test]
fn outfit_and_shop_files_parse_back() {
    for text in AWKWARD_TEXTS {
        let mut draft = OutfitDraft::new(text, text, OutfitCharacter::NineS, 20001, 9200);
        let mut lists = MeshLists::new();
        lists.insert(MeshState::Base, [text, "Body"].into_iter().collect::<MeshNames>());
        lists.insert(MeshState::Tower, ["mesh_es0202"].into_iter().collect::<MeshNames>());
        draft.meshes = lists;
        draft.shop_name = Some(text.to_owned());
        draft.shop_states = BTreeSet::from([0, 4]);

        let values = parse(&outfit::to_toml(&draft));
        assert_eq!(values["character"].as_str(), Some("9S"));
        assert_eq!(values["meshes"][0].as_str(), Some(text));
        assert_eq!(values["tower_meshes"][0].as_str(), Some("mesh_es0202"));

        let shop = parse(&outfit::shop_item_toml(&draft));
        let entries = shop["shop_item"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1]["shop_name"].as_str(), Some(text));
        assert_eq!(entries[1]["state"].as_integer(), Some(4));
    }
}

#[test]
fn a_fully_loaded_weapon_parses_back() {
    for text in AWKWARD_TEXTS {
        let mut draft = WeaponDraft::new(text, text, WeaponType::CombatBracer, 1500, 0x30612);
        draft.levels = vec![
            WeaponLevel { min_damage: 1, max_damage: 2 },
            WeaponLevel { min_damage: 3, max_damage: 4 },
        ];
        draft.shop_name = Some("shop_Emile".to_owned());
        draft.icon_name = Some(weapon::icon_texture_name_for(1500));
        draft.upgrades = vec![WeaponUpgrade {
            lv: 2,
            cost: 300,
            materials: vec![WeaponMaterial { item_id: 1, quantity: 2 }],
        }];
        draft.story = vec![text.to_owned(), "second page".to_owned()];
        draft.skill1_name = "Skill".to_owned();
        draft.skill1_desc = text.to_owned();
        draft.skill2_name = text.to_owned();
        draft.help_short = text.to_owned();
        draft.trait_id = Some(5);
        draft.story_texture_suffix = Some(9);
        let mut gold = WeaponEffect::new(EFFECT_PATHS[0]);
        gold.levels = [(1, 1.25), (4, 2.0)].into_iter().collect();
        let mut wig = WeaponEffect::new(EFFECT_PATHS[12]);
        wig.when_player = Some("A2".to_owned());
        draft.effects = vec![gold, wig];

        let values = parse(&weapon::to_toml(&draft));
        assert_eq!(values["obj_id"].as_integer(), Some(0x30612));
        assert_eq!(values["category"].as_integer(), Some(3));
        assert_eq!(values["special1"].as_integer(), Some(15001));
        assert_eq!(values["story_texture_suffix"].as_integer(), Some(9));
        assert_eq!(values["item"]["buy"].as_integer(), Some(5000));
        assert_eq!(values["text"]["name"]["_us"].as_str(), Some(text));
        assert_eq!(values["story"].as_array().unwrap().len(), 2);
        assert_eq!(values["upgrades"][0]["materials"][0]["quantity"].as_integer(), Some(2));
        assert_eq!(values["params"]["levels"].as_array().unwrap().len(), 2);
        assert_eq!(values["params"]["trait_id"].as_integer(), Some(5));
        assert_eq!(values["shops"][0]["shop_name"].as_str(), Some("shop_Emile"));
        assert_eq!(values["effect"]["gold_gain"]["levels"]["4"].as_float(), Some(2.0));
        assert_eq!(values["effect"]["a2_wig_show_a2"]["value"].as_float(), Some(1.0));
        assert_eq!(values["effect"]["a2_wig_show_a2"]["when"]["player"].as_str(), Some("A2"));
    }
}

#[test]
fn the_shipped_first_mod_snippets_parse() {
    parse(first_mod::MANIFEST);
    parse(first_mod::ITEM);
}
