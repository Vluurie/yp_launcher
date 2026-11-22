use indexmap::{IndexMap, IndexSet};

use crate::model::{DataCategory, InstalledMod, ModKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModGroupKind {
    Mixed,
    Wax,
    Outfit2b,
    Outfit9s,
    OutfitA2,
    OutfitOther,
    Weapon,
    Accessory,
    Item,
    Enemy,
    WorldProp,
    ModelVariant,
    Map,
    Effect,
    Scripting,
    Localization,
    Ui,
    Cutscene,
    Audio,
    Texture,
    Misc,
    Archive,
    Native,
    Other,
}

pub const MOD_GROUP_ORDER: [ModGroupKind; 24] = [
    ModGroupKind::Native,
    ModGroupKind::Mixed,
    ModGroupKind::Wax,
    ModGroupKind::Outfit2b,
    ModGroupKind::Outfit9s,
    ModGroupKind::OutfitA2,
    ModGroupKind::OutfitOther,
    ModGroupKind::Weapon,
    ModGroupKind::Accessory,
    ModGroupKind::Item,
    ModGroupKind::Enemy,
    ModGroupKind::WorldProp,
    ModGroupKind::ModelVariant,
    ModGroupKind::Map,
    ModGroupKind::Effect,
    ModGroupKind::Scripting,
    ModGroupKind::Localization,
    ModGroupKind::Ui,
    ModGroupKind::Cutscene,
    ModGroupKind::Audio,
    ModGroupKind::Texture,
    ModGroupKind::Misc,
    ModGroupKind::Archive,
    ModGroupKind::Other,
];

const PLAYER_SLOT_GROUPS: [(&str, ModGroupKind); 18] = [
    ("pl0000", ModGroupKind::Outfit2b),
    ("pl000d", ModGroupKind::Outfit2b),
    ("pl0001", ModGroupKind::Outfit2b),
    ("pl000e", ModGroupKind::Outfit2b),
    ("plf000", ModGroupKind::Outfit2b),
    ("plf00d", ModGroupKind::Outfit2b),
    ("pl0200", ModGroupKind::Outfit9s),
    ("pl0203", ModGroupKind::Outfit9s),
    ("pl020d", ModGroupKind::Outfit9s),
    ("pl020e", ModGroupKind::Outfit9s),
    ("plf200", ModGroupKind::Outfit9s),
    ("plf20d", ModGroupKind::Outfit9s),
    ("pl0100", ModGroupKind::OutfitA2),
    ("pl0101", ModGroupKind::OutfitA2),
    ("pl010d", ModGroupKind::OutfitA2),
    ("pl010e", ModGroupKind::OutfitA2),
    ("plf100", ModGroupKind::OutfitA2),
    ("plf10d", ModGroupKind::OutfitA2),
];

const CATEGORY_GROUPS: [(DataCategory, ModGroupKind); 15] = [
    (DataCategory::Weapon, ModGroupKind::Weapon),
    (DataCategory::Accessory, ModGroupKind::Accessory),
    (DataCategory::Item, ModGroupKind::Item),
    (DataCategory::Enemy, ModGroupKind::Enemy),
    (DataCategory::WorldProp, ModGroupKind::WorldProp),
    (DataCategory::ModelVariant, ModGroupKind::ModelVariant),
    (DataCategory::Map, ModGroupKind::Map),
    (DataCategory::Effects, ModGroupKind::Effect),
    (DataCategory::Scripting, ModGroupKind::Scripting),
    (DataCategory::Localization, ModGroupKind::Localization),
    (DataCategory::Ui, ModGroupKind::Ui),
    (DataCategory::Cutscenes, ModGroupKind::Cutscene),
    (DataCategory::Audio, ModGroupKind::Audio),
    (DataCategory::Misc, ModGroupKind::Misc),
    (DataCategory::Archive, ModGroupKind::Archive),
];

impl ModGroupKind {
    pub fn is_outfit(self) -> bool {
        matches!(
            self,
            ModGroupKind::Outfit2b
                | ModGroupKind::Outfit9s
                | ModGroupKind::OutfitA2
                | ModGroupKind::OutfitOther
        )
    }
}

pub fn player_slot_group(file_name: &str) -> Option<ModGroupKind> {
    let lower = file_name.to_lowercase();
    let stem: String = lower.chars().take(6).collect();
    PLAYER_SLOT_GROUPS
        .iter()
        .find(|(slot, _)| *slot == stem)
        .map(|(_, group)| *group)
}

pub fn groups_for_mod(installed: &InstalledMod) -> IndexSet<ModGroupKind> {
    if installed.kind == ModKind::Native {
        return IndexSet::from([ModGroupKind::Native]);
    }
    if installed
        .data
        .as_ref()
        .is_some_and(|data| data.has_compat_config)
    {
        return IndexSet::from([ModGroupKind::Wax]);
    }

    let mut groups: IndexSet<ModGroupKind> = IndexSet::new();
    if let Some(data) = &installed.data {
        for player in &data.players {
            groups
                .insert(player_slot_group(&player.file_name).unwrap_or(ModGroupKind::OutfitOther));
        }
    }
    let has_outfit_group = !groups.is_empty();

    let categories: IndexSet<DataCategory> = installed
        .data
        .iter()
        .flat_map(|data| data.entries.iter().map(|entry| entry.category))
        .collect();

    if !has_outfit_group && categories.contains(&DataCategory::Player) {
        groups.insert(ModGroupKind::OutfitOther);
    }
    for (category, group) in CATEGORY_GROUPS {
        if categories.contains(&category) {
            groups.insert(group);
        }
    }

    if distinct_group_kinds(&groups) >= 3 {
        return IndexSet::from([ModGroupKind::Mixed]);
    }
    if !groups.is_empty() {
        return groups;
    }
    if installed.kind == ModKind::Texture {
        return IndexSet::from([ModGroupKind::Texture]);
    }
    IndexSet::from([ModGroupKind::Other])
}

fn distinct_group_kinds(groups: &IndexSet<ModGroupKind>) -> usize {
    let non_outfit = groups.iter().filter(|group| !group.is_outfit()).count();
    let has_outfit = groups.iter().any(|group| group.is_outfit());
    non_outfit + usize::from(has_outfit)
}

pub fn group_mods(mods: &[InstalledMod]) -> IndexMap<ModGroupKind, Vec<&InstalledMod>> {
    let mut buckets: IndexMap<ModGroupKind, Vec<&InstalledMod>> = IndexMap::new();
    for installed in mods {
        for group in groups_for_mod(installed) {
            buckets.entry(group).or_default().push(installed);
        }
    }
    MOD_GROUP_ORDER
        .iter()
        .filter_map(|kind| buckets.shift_remove(kind).map(|list| (*kind, list)))
        .collect()
}
