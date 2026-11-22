mod support;

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::{
    detect, dir_names, file_names, install_named, put_bytes, put_json, temp,
};
use tempfile::TempDir;
use yp_core::GamePaths;
use yp_mods::grouping::{ModGroupKind, groups_for_mod};
use yp_mods::scan::list_installed;
use yp_mods::{InstalledMod, ModKind, OutfitChoice};

const CONFIG_SUBDIRS: [&str; 6] = [
    "accessories",
    "behaviors",
    "inventory",
    "item_info",
    "mcd",
    "outfits",
];

fn outfits_json() -> Value {
    json!({
        "pl_2b_outfits": [
            {"item_id": 90210, "outfit_id": 9010, "outfit_name": "YoRHa Uniform 1",
             "meshes": ["NS_KIMONO_Normal", "NS_KIMONO_Body", "Hair"], "effects": []},
            {"item_id": 90213, "outfit_id": 9013, "outfit_name": "2P's Body Replica",
             "meshes": ["NS_2P_Body", "NS_2P_Skirt", "Hair"], "effects": []}
        ],
        "pl_9s_outfits": [
            {"item_id": 90211, "outfit_id": 9011, "outfit_name": "YoRHa Uniform 2",
             "meshes": ["NS_KIMONO_Pants", "NS_KIMONO_Body", "Hair"], "effects": []}
        ],
        "pl_a2_outfits": [
            {"item_id": 90212, "outfit_id": 9012, "outfit_name": "YoRHa Uniform Prototype",
             "meshes": ["NS_KIMONO_Body", "NS_KIMONO_Cloth", "Hair"], "effects": []}
        ]
    })
}

fn items_json() -> Value {
    json!([
        {"item_id": 90210, "name": "item_uq_kimono2B", "filename": "it0000", "max_num": 1, "no_trash": true, "category": 6},
        {"item_id": 90216, "name": "item_uq_whiteFoxMask", "filename": "ete00d", "max_num": 1, "no_trash": true, "category": 6}
    ])
}

fn write_kimono_payload(root: &Path) {
    for stem in ["pl000d", "pl010d", "pl020d"] {
        put_bytes(root, &format!("data/pl/{stem}.dat"), 32, 0);
        put_bytes(root, &format!("data/pl/{stem}.dtt"), 32, 0);
    }
    for stem in ["ete00d", "ete00e", "ete0fa"] {
        put_bytes(root, &format!("data/et/{stem}.dat"), 32, 0);
        put_bytes(root, &format!("data/et/{stem}.dtt"), 32, 0);
    }
    for id in [90210, 90216] {
        put_bytes(root, &format!("data/misctex/misctex_wax_{id}.dat"), 32, 0);
        put_bytes(root, &format!("data/misctex/misctex_wax_{id}.dtt"), 32, 0);
    }
    put_json(root, "Kimono/outfits/kimono_outfits.json", &outfits_json());
    put_json(root, "Kimono/item_info/kimono_items.json", &items_json());
    put_json(
        root,
        "Kimono/accessories/kimono_accessories.json",
        &json!([{"item_name": "item_uq_whiteFoxMask", "item_id": 90216, "accessory_id": 20,
                 "filename": "ete00d", "mesh_names_2b": ["_2BA2_Body"], "effects": []}]),
    );
    put_json(
        root,
        "Kimono/behaviors/kimono_behaviors.json",
        &json!([{"filename": "ete00d", "behavior": "PlFaceMask", "group_id": 8}]),
    );
    put_json(
        root,
        "Kimono/inventory/kimono_give.json",
        &json!({"auto_give": {"90210": 1, "90216": 1}}),
    );
    put_json(
        root,
        "Kimono/mcd/kimono_mcd.json",
        &json!([{"filename": "messcore.mcd", "messages": [{"event_name": "WAX_ITEM_NAME_90210",
                 "locale": "*", "font_id": 36, "paragraphs": [["YoRHa Uniform 1"]]}]}]),
    );
}

fn kimono_drop() -> (TempDir, PathBuf) {
    let dir = temp("yp_kimono_");
    let root = dir.path().join("kimono_wax_0.0.3");
    write_kimono_payload(&root);
    (dir, root)
}

fn ids(choices: Option<&Vec<OutfitChoice>>) -> Vec<i64> {
    choices
        .map(|list| list.iter().map(|choice| choice.outfit_id).collect())
        .unwrap_or_default()
}

fn names(choices: Option<&Vec<OutfitChoice>>) -> Vec<String> {
    choices
        .map(|list| {
            list.iter()
                .filter_map(|choice| choice.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn listed(game_dir: &Path, id: &str) -> InstalledMod {
    list_installed(&GamePaths::new(game_dir))
        .unwrap()
        .into_iter()
        .find(|installed| installed.id == id)
        .unwrap()
}

#[test]
fn detect_reports_the_wax_config_without_touching_the_drop() {
    let (drop, root) = kimono_drop();

    let detected = detect(drop.path());
    assert_eq!(
        detected.kind,
        ModKind::Data,
        "rejected: {:?}",
        detected.rejection
    );
    let data = detected.data.as_ref().unwrap();
    assert!(data.has_compat_config);
    assert!(data.has_outfit_config);

    let for_2b = data.outfits_by_stem.get("pl000d");
    assert!(ids(for_2b).contains(&9010) && ids(for_2b).contains(&9013));
    let labels = names(for_2b);
    assert!(labels.contains(&"YoRHa Uniform 1".to_owned()));
    assert!(labels.contains(&"2P's Body Replica".to_owned()));
    assert!(for_2b.unwrap().iter().all(|choice| choice.needs_item));
    assert_eq!(ids(data.outfits_by_stem.get("pl020d")), vec![9011]);
    assert_eq!(ids(data.outfits_by_stem.get("pl010d")), vec![9012]);

    assert!(root.join("Kimono").is_dir());
    assert!(!root.join("wax").exists());
}

#[test]
fn install_relocates_the_wrapper_to_wax_mods_and_keeps_data_intact() {
    let (drop, _root) = kimono_drop();
    let game_dir = temp("yp_kimono_g_");

    let installed = install_named(game_dir.path(), drop.path(), "kimono_wax").unwrap();
    let mod_root = support::mod_dir(game_dir.path(), &installed.id);
    let wax_mod = mod_root.join("wax").join("mods").join("Kimono");
    for sub in CONFIG_SUBDIRS {
        let json_files: Vec<String> = file_names(&wax_mod.join(sub))
            .into_iter()
            .filter(|name| name.ends_with(".json"))
            .collect();
        assert!(!json_files.is_empty(), "missing wax/mods/Kimono/{sub}");
    }
    assert!(!mod_root.join("Kimono").exists());
    assert!(
        mod_root
            .join("data")
            .join("pl")
            .join("pl000d.dat")
            .is_file()
    );
    assert!(
        mod_root
            .join("data")
            .join("et")
            .join("ete00d.dat")
            .is_file()
    );
    assert!(
        mod_root
            .join("data")
            .join("misctex")
            .join("misctex_wax_90210.dtt")
            .is_file()
    );
}

#[test]
fn list_installed_exposes_the_config_outfits_and_wax_grouping() {
    let (drop, _root) = kimono_drop();
    let game_dir = temp("yp_kimono_g_");
    install_named(game_dir.path(), drop.path(), "kimono_wax").unwrap();

    let installed = listed(game_dir.path(), "kimono_wax");
    assert_eq!(installed.kind, ModKind::Data);
    let data = installed.data.as_ref().unwrap();
    assert!(data.has_compat_config);
    assert!(data.has_outfit_config);
    let for_pl0000 = ids(data.outfits_by_stem.get("pl0000"));
    assert!(for_pl0000.contains(&9010) && for_pl0000.contains(&9013));
    assert_eq!(
        groups_for_mod(&installed).into_iter().collect::<Vec<_>>(),
        vec![ModGroupKind::Wax]
    );
}

#[test]
fn an_already_broken_install_is_healed_on_the_next_list() {
    let game_dir = temp("yp_kimono_g_");
    let mod_root = support::mod_dir(game_dir.path(), "kimono_old");
    write_kimono_payload(&mod_root);

    let installed = listed(game_dir.path(), "kimono_old");

    assert!(
        mod_root
            .join("wax")
            .join("mods")
            .join("Kimono")
            .join("outfits")
            .join("kimono_outfits.json")
            .is_file()
    );
    assert!(!mod_root.join("Kimono").exists());
    let data = installed.data.as_ref().unwrap();
    assert!(data.has_compat_config);
    let for_2b = ids(data.outfits_by_stem.get("pl000d"));
    assert!(for_2b.contains(&9010) && for_2b.contains(&9013));
}

#[test]
fn a_correctly_packaged_wax_mod_is_left_untouched() {
    let drop = temp("yp_kimono_ok_");
    let game_dir = temp("yp_kimono_g_");
    let root = drop.path().join("wax_alice");
    put_bytes(&root, "data/pl/pl000d.dat", 32, 0);
    put_bytes(&root, "data/pl/pl000d.dtt", 32, 0);
    put_json(
        &root,
        "wax/mods/NikkeAlice/outfits/NikkeAlice.json",
        &outfits_json(),
    );
    put_json(
        &root,
        "wax/mods/NikkeAlice/item_info/AliceHeadset.json",
        &items_json(),
    );

    install_named(game_dir.path(), drop.path(), "wax_alice").unwrap();

    let mod_root = support::mod_dir(game_dir.path(), "wax_alice");
    assert!(
        mod_root
            .join("wax")
            .join("mods")
            .join("NikkeAlice")
            .join("outfits")
            .join("NikkeAlice.json")
            .is_file()
    );
    assert_eq!(
        dir_names(&mod_root.join("wax").join("mods")),
        vec!["NikkeAlice"]
    );
}

#[test]
fn a_wrapper_carrying_its_own_data_payload_is_not_relocated() {
    let drop = temp("yp_kimono_var_");
    let game_dir = temp("yp_kimono_g_");
    let root = drop.path().join("pack");
    put_bytes(&root, "data/pl/pl000d.dat", 32, 0);
    put_bytes(&root, "VariantB/data/pl/pl000d.dat", 32, 0);
    put_json(&root, "VariantB/outfits/variant.json", &outfits_json());

    install_named(game_dir.path(), drop.path(), "variant_pack").unwrap();

    let mod_root = support::mod_dir(game_dir.path(), "variant_pack");
    assert!(mod_root.join("VariantB").is_dir());
    assert!(!mod_root.join("wax").exists());
}

#[test]
fn real_kimono_wax_archive_full_pipeline() {
    let archive = std::env::var("YP_KIMONO_ZIP")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lib/widgets")
                .join("Kimono Wax 0.0.3 812 0.0.3 2026-07-30T15-53Z MWGBtyVlH.zip")
        });
    if !archive.is_file() {
        eprintln!("skipped: kimono zip not present ({})", archive.display());
        return;
    }
    let game_dir = temp("yp_kimono_g_");

    let detected = detect(&archive);
    assert_eq!(
        detected.kind,
        ModKind::Data,
        "rejected: {:?}",
        detected.rejection
    );
    let data = detected.data.as_ref().unwrap();
    assert!(data.has_compat_config);
    assert!(data.has_outfit_config);
    assert!(names(data.outfits_by_stem.get("pl000d")).contains(&"YoRHa Uniform 1".to_owned()));

    let installed = install_named(game_dir.path(), &archive, "kimono_wax").unwrap();
    let mod_root = support::mod_dir(game_dir.path(), &installed.id);
    let wax_mod = mod_root.join("wax").join("mods").join("Kimono");
    for sub in CONFIG_SUBDIRS {
        assert!(wax_mod.join(sub).is_dir(), "missing wax/mods/Kimono/{sub}");
    }
    assert!(!mod_root.join("Kimono").exists());
    assert!(
        mod_root
            .join("data")
            .join("pl")
            .join("pl0000.dat")
            .is_file()
    );
    assert!(
        mod_root
            .join("data")
            .join("misctex")
            .join("misctex_wax_90221.dtt")
            .is_file()
    );

    let listed = listed(game_dir.path(), "kimono_wax");
    let labels = names(listed.data.as_ref().unwrap().outfits_by_stem.get("pl0000"));
    assert!(labels.contains(&"YoRHa Uniform 1".to_owned()));
    assert!(labels.contains(&"2P's Body Replica".to_owned()));
    assert_eq!(
        groups_for_mod(&listed).into_iter().collect::<Vec<_>>(),
        vec![ModGroupKind::Wax]
    );
}
