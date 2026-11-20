use crate::model::DataCategory;

pub const DATA_DIR_CATEGORIES: [(&str, DataCategory); 34] = [
    ("pl", DataCategory::Player),
    ("wp", DataCategory::Weapon),
    ("em", DataCategory::Enemy),
    ("ba", DataCategory::WorldProp),
    ("bg", DataCategory::WorldProp),
    ("bh", DataCategory::WorldProp),
    ("et", DataCategory::Accessory),
    ("it", DataCategory::Item),
    ("um", DataCategory::ModelVariant),
    ("wd1", DataCategory::Map),
    ("wd2", DataCategory::Map),
    ("wd3", DataCategory::Map),
    ("wd4", DataCategory::Map),
    ("wd5", DataCategory::Map),
    ("wda", DataCategory::Map),
    ("core", DataCategory::Scripting),
    ("ph1", DataCategory::Scripting),
    ("ph2", DataCategory::Scripting),
    ("ph3", DataCategory::Scripting),
    ("phf", DataCategory::Scripting),
    ("quest", DataCategory::Scripting),
    ("st1", DataCategory::Scripting),
    ("st2", DataCategory::Scripting),
    ("st5", DataCategory::Scripting),
    ("novel", DataCategory::Localization),
    ("subtitle", DataCategory::Localization),
    ("txtmess", DataCategory::Localization),
    ("ui", DataCategory::Localization),
    ("effect", DataCategory::Effects),
    ("enlighten", DataCategory::Effects),
    ("font", DataCategory::Ui),
    ("misctex", DataCategory::Misc),
    ("movie", DataCategory::Cutscenes),
    ("sound", DataCategory::Audio),
];

pub const PLAYER_MODELS: [(&str, &str); 48] = [
    ("pl0000", "2B (vanilla)"),
    ("pl000d", "2B (DLC clothes)"),
    ("pl000f", "2B (animations)"),
    ("pl002f", "9S (animations)"),
    ("pl010f", "A2 (animations)"),
    ("pl020f", "9S (animations, DLC)"),
    ("pl0001", "2B (armoured)"),
    ("pl000e", "dead YoRHa (2B body)"),
    ("pl0010", "flight unit"),
    ("pl0013", "YoRHa flight unit"),
    ("pl0100", "A2 (long hair)"),
    ("pl010d", "A2 (DLC clothes, long hair)"),
    ("pl010e", "A2 short hair (DLC)"),
    ("pl0101", "A2 short hair"),
    ("pl0200", "9S (vanilla)"),
    ("pl0203", "9S"),
    ("pl020d", "9S (DLC clothes)"),
    ("pl020e", "dead YoRHa (9S body)"),
    ("pl0300", "YoRHa (background squad variants)"),
    ("pl0400", "YoRHa armour"),
    ("pl0500", "Devola"),
    ("pl0600", "Popola"),
    ("pl1000", "Pascal"),
    ("pl1001", "Pascal (flying)"),
    ("pl1010", "Emil"),
    ("pl1011", "Emil head"),
    ("pl1020", "Commander"),
    ("pl1030", "Anemone"),
    ("pl1040", "Devola (alt files)"),
    ("pl1050", "Popola (alt files)"),
    ("pl1100", "Engels' back"),
    ("pl2000", "YoRHa (background variants)"),
    ("pl2010", "YoRHa (armoured)"),
    ("pl2020", "operator base"),
    ("pl2021", "operator 21O hair"),
    ("pl2022", "operator 6O hair"),
    ("pl2023", "Adam hair"),
    ("pl2030", "Jackass"),
    ("pl2031", "strange resistance woman"),
    ("pl2040", "resistance androids"),
    ("pl2050", "resistance androids (more)"),
    ("pl2060", "4S"),
    ("plf000", "high-detail face"),
    ("plf00d", "high-detail face (DLC, 2B)"),
    ("plf10d", "high-detail face (DLC, A2)"),
    ("plf20d", "high-detail face (DLC, 9S)"),
    ("plf100", "high-detail face"),
    ("plf200", "high-detail face"),
];

pub const CONDITIONAL_OUTFIT_SLOTS: [&str; 6] =
    ["pl0000", "pl000d", "pl0100", "pl010d", "pl0200", "pl020d"];

pub const VARIANT_CATEGORY_ORDER: [DataCategory; 17] = [
    DataCategory::Player,
    DataCategory::Weapon,
    DataCategory::Accessory,
    DataCategory::Enemy,
    DataCategory::ModelVariant,
    DataCategory::Item,
    DataCategory::WorldProp,
    DataCategory::Map,
    DataCategory::Effects,
    DataCategory::Scripting,
    DataCategory::Localization,
    DataCategory::Ui,
    DataCategory::Cutscenes,
    DataCategory::Audio,
    DataCategory::Archive,
    DataCategory::Misc,
    DataCategory::Other,
];

pub const MUTUALLY_EXCLUSIVE_VARIANT_CATEGORIES: [DataCategory; 1] = [DataCategory::Weapon];

pub fn data_dir_category(dir_key: &str) -> Option<DataCategory> {
    DATA_DIR_CATEGORIES
        .iter()
        .find(|(key, _)| *key == dir_key)
        .map(|(_, category)| *category)
}

pub fn is_data_dir_key(dir_key: &str) -> bool {
    data_dir_category(dir_key).is_some()
}

pub fn player_model_label(stem: &str) -> Option<&'static str> {
    PLAYER_MODELS
        .iter()
        .find(|(key, _)| *key == stem)
        .map(|(_, label)| *label)
}

pub fn is_conditional_outfit_slot(stem: &str) -> bool {
    CONDITIONAL_OUTFIT_SLOTS.contains(&stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_keys_are_unique_and_lowercase() {
        for (index, (key, _)) in DATA_DIR_CATEGORIES.iter().enumerate() {
            assert_eq!(*key, key.to_lowercase());
            assert!(
                DATA_DIR_CATEGORIES[index + 1..]
                    .iter()
                    .all(|(other, _)| other != key)
            );
        }
        for (index, (key, _)) in PLAYER_MODELS.iter().enumerate() {
            assert!(
                PLAYER_MODELS[index + 1..]
                    .iter()
                    .all(|(other, _)| other != key)
            );
        }
    }

    #[test]
    fn lookups() {
        assert_eq!(data_dir_category("misctex"), Some(DataCategory::Misc));
        assert_eq!(data_dir_category("PL"), None);
        assert_eq!(player_model_label("pl0000"), Some("2B (vanilla)"));
        assert!(is_conditional_outfit_slot("pl020d"));
        assert!(!is_conditional_outfit_slot("pl000f"));
    }
}
