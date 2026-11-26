use std::collections::HashSet;
use std::path::Path;

use gpui::Pixels;
use yp_mods::grouping::{MOD_GROUP_ORDER, ModGroupKind, groups_for_mod};
use yp_mods::{DataSummary, InstalledMod, ModKind};

const MODS_PREFIX: &str = "mods";
pub use yp_mods::defaults::stem_rel_path;

pub fn mod_rel_path(mod_id: &str) -> String {
    format!("{MODS_PREFIX}/{mod_id}")
}

pub fn mod_id_of_entry(entry_path: &str) -> &str {
    entry_path.split('/').nth(1).unwrap_or(entry_path)
}

pub fn last_segment(entry_path: &str) -> &str {
    entry_path.rsplit('/').next().unwrap_or(entry_path)
}

pub fn shown_name<'a>(installed: &'a InstalledMod, custom: Option<&'a str>) -> &'a str {
    custom.unwrap_or(&installed.display_name)
}

pub fn sort_tier(installed: &InstalledMod) -> u8 {
    if installed.kind == ModKind::Native {
        0
    } else if installed
        .data
        .as_ref()
        .is_some_and(|data| data.has_compat_config)
    {
        1
    } else {
        2
    }
}

pub fn filter_and_sort<'a>(
    mods: &'a [InstalledMod],
    filter: &str,
    custom_name: &dyn Fn(&str) -> Option<String>,
) -> Vec<&'a InstalledMod> {
    let needle = filter.trim().to_lowercase();
    let mut shown: Vec<(&InstalledMod, String)> = mods
        .iter()
        .map(|installed| {
            let custom = custom_name(&installed.id);
            let name = shown_name(installed, custom.as_deref()).to_lowercase();
            (installed, name)
        })
        .filter(|(installed, name)| {
            needle.is_empty()
                || installed.id.to_lowercase().contains(&needle)
                || installed.display_name.to_lowercase().contains(&needle)
                || name.contains(&needle)
                || installed.data.as_ref().is_some_and(|data| {
                    data.outfits_by_stem.values().flatten().any(|outfit| {
                        outfit
                            .name
                            .as_ref()
                            .is_some_and(|name| name.to_lowercase().contains(&needle))
                            || outfit.outfit_id.to_string() == needle
                    }) || data.players.iter().any(|player| {
                        player.file_name.to_lowercase().contains(&needle)
                            || player.label.to_lowercase().contains(&needle)
                    }) || data
                        .archives
                        .iter()
                        .any(|archive| archive.stem.to_lowercase().contains(&needle))
                })
        })
        .collect();
    shown.sort_by(|(a, a_name), (b, b_name)| {
        sort_tier(a)
            .cmp(&sort_tier(b))
            .then_with(|| a_name.cmp(b_name))
    });
    shown.into_iter().map(|(installed, _)| installed).collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListRow {
    Header {
        kind: ModGroupKind,
        count: usize,
        collapsed: bool,
    },
    Mod {
        id: String,
        group: Option<ModGroupKind>,
    },
}

pub fn list_rows(
    mods: &[&InstalledMod],
    filter_active: bool,
    collapsed: &HashSet<ModGroupKind>,
) -> Vec<ListRow> {
    if filter_active {
        return mods
            .iter()
            .map(|installed| ListRow::Mod {
                id: installed.id.clone(),
                group: None,
            })
            .collect();
    }
    let mut buckets: Vec<(ModGroupKind, Vec<&str>)> = MOD_GROUP_ORDER
        .iter()
        .map(|kind| (*kind, Vec::new()))
        .collect();
    for installed in mods {
        for group in groups_for_mod(installed) {
            if let Some((_, members)) = buckets.iter_mut().find(|(kind, _)| *kind == group) {
                members.push(&installed.id);
            }
        }
    }
    let mut rows = Vec::new();
    for (kind, members) in buckets {
        if members.is_empty() {
            continue;
        }
        let is_collapsed = collapsed.contains(&kind);
        rows.push(ListRow::Header {
            kind,
            count: members.len(),
            collapsed: is_collapsed,
        });
        if is_collapsed {
            continue;
        }
        rows.extend(members.into_iter().map(|id| ListRow::Mod {
            id: id.to_owned(),
            group: Some(kind),
        }));
    }
    rows
}

pub fn should_show_search(
    mod_count: usize,
    row_height: Pixels,
    available_height: Pixels,
    filter_active: bool,
) -> bool {
    if mod_count == 0 {
        return false;
    }
    filter_active || row_height * mod_count as f32 > available_height
}

pub fn has_live_warnings(installed: &InstalledMod, is_disabled: &dyn Fn(&str) -> bool) -> bool {
    if is_disabled(&mod_rel_path(&installed.id)) {
        return false;
    }
    !installed.requires_missing.is_empty()
        || installed
            .conflicts
            .iter()
            .any(|conflict| !is_disabled(&mod_rel_path(&conflict.other_mod_id)))
}

pub fn conflict_candidates(
    installed: &InstalledMod,
    is_disabled: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut ids: Vec<String> = std::iter::once(installed.id.clone())
        .chain(
            installed
                .conflicts
                .iter()
                .map(|conflict| conflict.other_mod_id.clone()),
        )
        .filter(|id| !is_disabled(&mod_rel_path(id)))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

pub fn conflict_file_count(installed: &InstalledMod) -> usize {
    installed
        .conflicts
        .iter()
        .map(|conflict| conflict.detail.as_str())
        .collect::<HashSet<_>>()
        .len()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerStem {
    pub stem: String,
    pub label: String,
}

pub fn player_stems(data: &DataSummary) -> Vec<PlayerStem> {
    let mut seen = HashSet::new();
    data.players
        .iter()
        .filter_map(|player| {
            let stem = Path::new(&player.file_name)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_else(|| player.file_name.clone());
            seen.insert(stem.clone()).then(|| PlayerStem {
                stem,
                label: player.label.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;
    use std::path::PathBuf;
    use yp_mods::{
        DataCategory, DataDirEntry, ModConflict, ModConflictKind, NativeSummary, PlayerModelEntry,
    };

    fn data_mod(id: &str, dir: &str, category: DataCategory) -> InstalledMod {
        let mut installed = InstalledMod::new(id, PathBuf::from(id), ModKind::Data);
        installed.data = Some(DataSummary {
            entries: vec![DataDirEntry {
                dir_name: dir.to_owned(),
                category,
                file_count: 1,
            }],
            ..DataSummary::default()
        });
        installed
    }

    fn outfit(id: &str, file: &str) -> InstalledMod {
        let mut installed = data_mod(id, "pl", DataCategory::Player);
        if let Some(data) = &mut installed.data {
            data.players.push(PlayerModelEntry {
                file_name: file.to_owned(),
                label: "2B".to_owned(),
            });
        }
        installed
    }

    #[test]
    fn search_finds_a_later_outfit_by_name_and_id() {
        let mut wardrobe = outfit("wardrobe", "pl000d.dat");
        wardrobe.data.as_mut().unwrap().outfits_by_stem.insert(
            "pl000d".into(),
            [(7, "Uniform"), (15, "Evening Dress")]
                .map(|(id, label)| yp_mods::OutfitChoice {
                    outfit_id: id,
                    name: Some(label.to_owned()),
                    needs_item: false,
                })
                .to_vec(),
        );
        let mods = [wardrobe, native("other")];
        for query in [" EVENING dress ", "15"] {
            assert_eq!(ids(&filter_and_sort(&mods, query, &|_| None)), ["wardrobe"]);
        }
    }

    fn native(id: &str) -> InstalledMod {
        let mut installed = InstalledMod::new(id, PathBuf::from(id), ModKind::Native);
        installed.native = Some(NativeSummary::default());
        installed
    }

    fn wax(id: &str) -> InstalledMod {
        let mut installed = outfit(id, "pl000d.dat");
        if let Some(data) = &mut installed.data {
            data.has_compat_config = true;
        }
        installed
    }

    fn sample() -> Vec<InstalledMod> {
        vec![
            outfit("zeta_outfit", "pl000d.dat"),
            data_mod("sword", "wp", DataCategory::Weapon),
            wax("wax_alice"),
            native("Bunker Cam"),
            outfit("alpha_outfit", "pl0200.dat"),
        ]
    }

    fn ids(mods: &[&InstalledMod]) -> Vec<String> {
        mods.iter().map(|installed| installed.id.clone()).collect()
    }

    #[test]
    fn mods_sort_native_first_then_wax_then_by_shown_name() {
        let mods = sample();
        let sorted = filter_and_sort(&mods, "", &|_| None);
        assert_eq!(
            ids(&sorted),
            [
                "Bunker Cam",
                "wax_alice",
                "alpha_outfit",
                "sword",
                "zeta_outfit"
            ]
        );
    }

    #[test]
    fn custom_names_drive_both_the_order_and_the_search() {
        let mods = sample();
        let custom = |id: &str| (id == "zeta_outfit").then(|| "Aardvark Dress".to_owned());
        let sorted = filter_and_sort(&mods, "", &custom);
        assert_eq!(
            ids(&sorted),
            [
                "Bunker Cam",
                "wax_alice",
                "zeta_outfit",
                "alpha_outfit",
                "sword"
            ]
        );
        assert_eq!(
            ids(&filter_and_sort(&mods, "DRESS", &custom)),
            ["zeta_outfit"]
        );
        assert_eq!(
            ids(&filter_and_sort(&mods, "zeta", &custom)),
            ["zeta_outfit"]
        );
        assert_eq!(
            ids(&filter_and_sort(&mods, "outfit", &|_| None)),
            ["alpha_outfit", "zeta_outfit"]
        );
        assert!(filter_and_sort(&mods, "nothing", &|_| None).is_empty());
    }

    #[test]
    fn rows_are_grouped_in_the_fixed_group_order_with_counts() {
        let mods = sample();
        let sorted = filter_and_sort(&mods, "", &|_| None);
        let rows = list_rows(&sorted, false, &HashSet::new());
        let header = |kind, count| ListRow::Header {
            kind,
            count,
            collapsed: false,
        };
        let row = |id: &str, group| ListRow::Mod {
            id: id.to_owned(),
            group: Some(group),
        };
        assert_eq!(
            rows,
            vec![
                header(ModGroupKind::Native, 1),
                row("Bunker Cam", ModGroupKind::Native),
                header(ModGroupKind::Wax, 1),
                row("wax_alice", ModGroupKind::Wax),
                header(ModGroupKind::Outfit2b, 1),
                row("zeta_outfit", ModGroupKind::Outfit2b),
                header(ModGroupKind::Outfit9s, 1),
                row("alpha_outfit", ModGroupKind::Outfit9s),
                header(ModGroupKind::Weapon, 1),
                row("sword", ModGroupKind::Weapon),
            ]
        );
    }

    #[test]
    fn collapsed_groups_keep_their_header_and_hide_their_rows() {
        let mods = sample();
        let sorted = filter_and_sort(&mods, "", &|_| None);
        let collapsed: HashSet<ModGroupKind> = [ModGroupKind::Weapon].into_iter().collect();
        let rows = list_rows(&sorted, false, &collapsed);
        assert!(rows.contains(&ListRow::Header {
            kind: ModGroupKind::Weapon,
            count: 1,
            collapsed: true
        }));
        assert!(!rows.iter().any(|row| matches!(
            row,
            ListRow::Mod { id, .. } if id == "sword"
        )));
    }

    #[test]
    fn a_mod_for_two_characters_is_listed_under_both() {
        let mut both = outfit("duo", "pl000d.dat");
        if let Some(data) = &mut both.data {
            data.players.push(PlayerModelEntry {
                file_name: "pl0200.dtt".to_owned(),
                label: "9S".to_owned(),
            });
        }
        let mods = vec![both];
        let sorted = filter_and_sort(&mods, "", &|_| None);
        let rows = list_rows(&sorted, false, &HashSet::new());
        let listed: Vec<Option<ModGroupKind>> = rows
            .iter()
            .filter_map(|row| match row {
                ListRow::Mod { group, .. } => Some(*group),
                ListRow::Header { .. } => None,
            })
            .collect();
        assert_eq!(
            listed,
            vec![Some(ModGroupKind::Outfit2b), Some(ModGroupKind::Outfit9s)]
        );
    }

    #[test]
    fn searching_flattens_the_list() {
        let mods = sample();
        let found = filter_and_sort(&mods, "outfit", &|_| None);
        let rows = list_rows(&found, true, &HashSet::new());
        assert_eq!(
            rows,
            vec![
                ListRow::Mod {
                    id: "alpha_outfit".to_owned(),
                    group: None
                },
                ListRow::Mod {
                    id: "zeta_outfit".to_owned(),
                    group: None
                },
            ]
        );
    }

    #[test]
    fn the_search_field_appears_once_the_list_would_scroll_or_a_filter_is_set() {
        assert!(!should_show_search(0, px(40.), px(400.), true));
        assert!(!should_show_search(5, px(40.), px(400.), false));
        assert!(should_show_search(11, px(40.), px(400.), false));
        assert!(should_show_search(1, px(40.), px(400.), true));
    }

    fn conflict(other: &str, file: &str) -> ModConflict {
        ModConflict {
            other_mod_id: other.to_owned(),
            kind: ModConflictKind::OverlappingDataFile,
            detail: file.to_owned(),
        }
    }

    #[test]
    fn warnings_only_count_enabled_mods_on_both_sides() {
        let mut installed = outfit("kimono", "pl000d.dat");
        installed.conflicts = vec![conflict("dress", "pl/pl000d.dat")];
        assert!(has_live_warnings(&installed, &|_| false));
        assert!(!has_live_warnings(&installed, &|rel| rel == "mods/dress"));
        assert!(!has_live_warnings(&installed, &|rel| rel == "mods/kimono"));

        let mut needs = outfit("needs", "pl000d.dat");
        needs.requires_missing = vec!["base".to_owned()];
        assert!(has_live_warnings(&needs, &|_| false));
        assert!(!has_live_warnings(&outfit("clean", "pl000d.dat"), &|_| {
            false
        }));
    }

    #[test]
    fn conflict_candidates_are_the_enabled_mods_sorted_and_unique() {
        let mut installed = outfit("kimono", "pl000d.dat");
        installed.conflicts = vec![
            conflict("dress", "pl/pl000d.dat"),
            conflict("dress", "pl/pl000d.dtt"),
            conflict("armor", "pl/pl000d.dat"),
        ];
        assert_eq!(
            conflict_candidates(&installed, &|_| false),
            ["armor", "dress", "kimono"]
        );
        assert_eq!(
            conflict_candidates(&installed, &|rel| rel == "mods/armor"),
            ["dress", "kimono"]
        );
        assert_eq!(conflict_file_count(&installed), 2);
    }

    #[test]
    fn player_stems_drop_the_extension_and_duplicates() {
        let mut installed = outfit("kimono", "pl000d.dat");
        if let Some(data) = &mut installed.data {
            data.players.push(PlayerModelEntry {
                file_name: "pl000d.dtt".to_owned(),
                label: "2B".to_owned(),
            });
            data.players.push(PlayerModelEntry {
                file_name: "pl0200.dat".to_owned(),
                label: "9S".to_owned(),
            });
        }
        let stems = player_stems(installed.data.as_ref().unwrap());
        assert_eq!(
            stems,
            vec![
                PlayerStem {
                    stem: "pl000d".to_owned(),
                    label: "2B".to_owned()
                },
                PlayerStem {
                    stem: "pl0200".to_owned(),
                    label: "9S".to_owned()
                },
            ]
        );
    }

    #[test]
    fn default_entry_paths_resolve_to_their_mod_and_file() {
        assert_eq!(mod_rel_path("kimono"), "mods/kimono");
        assert_eq!(stem_rel_path("kimono", "pl000d"), "mods/kimono/pl/pl000d");
        assert_eq!(mod_id_of_entry("mods/kimono/pl/pl000d"), "kimono");
        assert_eq!(mod_id_of_entry("kimono"), "kimono");
        assert_eq!(last_segment("mods/kimono/pl/pl000d"), "pl000d");
    }
}
