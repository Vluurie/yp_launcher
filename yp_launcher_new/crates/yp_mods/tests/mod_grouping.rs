use indexmap::IndexSet;
use yp_mods::grouping::{ModGroupKind, group_mods, groups_for_mod};
use yp_mods::{DataCategory, DataDirEntry, DataSummary, InstalledMod, ModKind, PlayerModelEntry};

fn installed(
    id: &str,
    kind: ModKind,
    player_files: &[&str],
    categories: &[DataCategory],
) -> InstalledMod {
    let mut installed = InstalledMod::new(id, format!("mods/{id}"), kind);
    installed.data = Some(DataSummary {
        entries: categories
            .iter()
            .map(|category| DataDirEntry {
                dir_name: category.name().to_owned(),
                category: *category,
                file_count: 1,
            })
            .collect(),
        players: player_files
            .iter()
            .map(|file| PlayerModelEntry {
                file_name: (*file).to_owned(),
                label: (*file).to_owned(),
            })
            .collect(),
        ..DataSummary::default()
    });
    installed
}

fn data_mod(id: &str, player_files: &[&str], categories: &[DataCategory]) -> InstalledMod {
    installed(id, ModKind::Data, player_files, categories)
}

fn set(groups: &[ModGroupKind]) -> IndexSet<ModGroupKind> {
    groups.iter().copied().collect()
}

#[test]
fn outfits_group_by_player_character() {
    let cases = [
        ("pl0000.dat", ModGroupKind::Outfit2b),
        ("pl000d.dat", ModGroupKind::Outfit2b),
        ("plf00d.dat", ModGroupKind::Outfit2b),
        ("pl0200.dat", ModGroupKind::Outfit9s),
        ("pl020d.dat", ModGroupKind::Outfit9s),
        ("pl0100.dat", ModGroupKind::OutfitA2),
        ("pl0101.dat", ModGroupKind::OutfitA2),
        ("pl010d.dat", ModGroupKind::OutfitA2),
    ];
    for (file, expected) in cases {
        assert_eq!(
            groups_for_mod(&data_mod("m", &[file], &[])),
            set(&[expected]),
            "{file}"
        );
    }
}

#[test]
fn non_main_player_models_land_in_other_outfits() {
    assert_eq!(
        groups_for_mod(&data_mod("m", &["pl1010.dat"], &[])),
        set(&[ModGroupKind::OutfitOther])
    );
}

#[test]
fn a_mod_replacing_2b_and_9s_is_listed_under_both() {
    assert_eq!(
        groups_for_mod(&data_mod("m", &["pl0000.dat", "pl0200.dat"], &[])),
        set(&[ModGroupKind::Outfit2b, ModGroupKind::Outfit9s])
    );
}

#[test]
fn several_files_for_the_same_character_yield_one_group() {
    assert_eq!(
        groups_for_mod(&data_mod(
            "m",
            &["pl0000.dat", "pl0000.dtt", "plf000.dat"],
            &[]
        )),
        set(&[ModGroupKind::Outfit2b])
    );
}

#[test]
fn weapons_group_by_category() {
    assert_eq!(
        groups_for_mod(&data_mod("w", &[], &[DataCategory::Weapon])),
        set(&[ModGroupKind::Weapon])
    );
}

#[test]
fn effects_group_by_category() {
    assert_eq!(
        groups_for_mod(&data_mod("e", &[], &[DataCategory::Effects])),
        set(&[ModGroupKind::Effect])
    );
}

#[test]
fn native_mods_without_data_land_in_the_native_group() {
    let native = InstalledMod::new("n", "mods/n", ModKind::Native);
    assert_eq!(groups_for_mod(&native), set(&[ModGroupKind::Native]));
}

#[test]
fn a_native_mod_with_an_outfit_overlay_stays_native_only() {
    let native = installed(
        "n",
        ModKind::Native,
        &["pl0000.dat"],
        &[DataCategory::Player],
    );
    assert_eq!(groups_for_mod(&native), set(&[ModGroupKind::Native]));
}

#[test]
fn group_mods_lists_a_native_outfit_mod_once_under_native() {
    let mods = [installed("n", ModKind::Native, &["pl0000.dat"], &[])];
    let grouped = group_mods(&mods);
    assert_eq!(
        grouped.keys().copied().collect::<Vec<_>>(),
        vec![ModGroupKind::Native]
    );
    assert_eq!(grouped[&ModGroupKind::Native].len(), 1);
    assert_eq!(grouped[&ModGroupKind::Native][0].id, "n");
}

#[test]
fn player_data_without_recognised_slots_is_an_outfit() {
    assert_eq!(
        groups_for_mod(&data_mod("p", &[], &[DataCategory::Player])),
        set(&[ModGroupKind::OutfitOther])
    );
}

#[test]
fn compat_config_mods_are_wax() {
    let mut wax = data_mod("w", &["pl0000.dat"], &[DataCategory::Weapon]);
    if let Some(data) = wax.data.as_mut() {
        data.has_compat_config = true;
    }
    assert_eq!(groups_for_mod(&wax), set(&[ModGroupKind::Wax]));
}

#[test]
fn three_distinct_groups_collapse_into_mixed() {
    let mixed = data_mod(
        "x",
        &["pl0000.dat", "pl0200.dat"],
        &[DataCategory::Weapon, DataCategory::Effects],
    );
    assert_eq!(groups_for_mod(&mixed), set(&[ModGroupKind::Mixed]));
    let two = data_mod("y", &["pl0000.dat", "pl0200.dat"], &[DataCategory::Weapon]);
    assert_eq!(
        groups_for_mod(&two),
        set(&[
            ModGroupKind::Outfit2b,
            ModGroupKind::Outfit9s,
            ModGroupKind::Weapon
        ])
    );
}

#[test]
fn empty_data_falls_back_to_texture_or_other() {
    let texture = installed("t", ModKind::Texture, &[], &[]);
    assert_eq!(groups_for_mod(&texture), set(&[ModGroupKind::Texture]));
    let unknown = installed("u", ModKind::Data, &[], &[DataCategory::Other]);
    assert_eq!(groups_for_mod(&unknown), set(&[ModGroupKind::Other]));
}

#[test]
fn group_mods_orders_groups_by_mod_group_order_and_keeps_every_mod() {
    let mods = [
        data_mod("weapon", &[], &[DataCategory::Weapon]),
        data_mod("a2", &["pl0100.dat"], &[]),
        data_mod("twob", &["pl0000.dat"], &[]),
        InstalledMod::new("nams", "mods/nams", ModKind::Native),
    ];
    let grouped = group_mods(&mods);
    assert_eq!(
        grouped.keys().copied().collect::<Vec<_>>(),
        vec![
            ModGroupKind::Native,
            ModGroupKind::Outfit2b,
            ModGroupKind::OutfitA2,
            ModGroupKind::Weapon,
        ]
    );
    assert_eq!(grouped[&ModGroupKind::Outfit2b].len(), 1);
    assert_eq!(grouped[&ModGroupKind::Outfit2b][0].id, "twob");
}

#[test]
fn group_mods_a_multi_character_mod_appears_in_each_of_its_groups() {
    let mods = [data_mod("pack", &["pl0000.dat", "pl0200.dat"], &[])];
    let grouped = group_mods(&mods);
    assert_eq!(grouped[&ModGroupKind::Outfit2b][0].id, "pack");
    assert_eq!(grouped[&ModGroupKind::Outfit9s][0].id, "pack");
}

#[test]
fn group_mods_empty_input_yields_no_groups() {
    assert!(group_mods(&[]).is_empty());
}
