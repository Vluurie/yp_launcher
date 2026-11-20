use yp_builders::weapon::{self, WeaponLevel};
use yp_builders::{WeaponDraft, WeaponType};

const EXPECTED: &str = r#"id = 1100
obj_id = 0x30100
category = 0

[item]
name = "weapon_awakened_protocol"
max_num = 1
no_trash = true
die_discard = false
buy = -1
sell = 5000

[effects]
est_id_normal = 600
est_id_berserk = 633

[text.name]
_us = "Awakened Protocol"

[text.help]
_us = '''
x'''

[params]
clone_from_obj_id = 0x30003
levels = [
  { min_damage = 100, max_damage = 140 },
]
"#;

#[test]
fn awakened_protocol_matches_the_dart_builder_output() {
    let mut draft = WeaponDraft::new("Awakened Protocol", "x", WeaponType::SmallSword, 1100, 0x30100);
    draft.sell_price = 5000;
    draft.levels = vec![WeaponLevel {
        min_damage: 100,
        max_damage: 140,
    }];

    let toml = weapon::to_toml(&draft);

    assert_eq!(toml, EXPECTED);
    let values: toml::Table = toml::from_str(&toml).unwrap();
    assert_eq!(values["obj_id"].as_integer(), Some(0x30100));
    assert_eq!(values["params"]["levels"][0]["max_damage"].as_integer(), Some(140));
}
