mod support;

use std::collections::HashMap;
use std::path::Path;

use serde_json::json;
use support::{put_json, temp};
use yp_mods::DataArchivePair;
use yp_mods::inspector_presets::{
    Character, STATE_COMBAT, STATE_EYEMASK, STATE_PROLOGUE, STATE_SELF_DESTRUCTED,
    infer_vanilla_variants, load,
};

fn archive(stem: &str, root: &Path) -> DataArchivePair {
    DataArchivePair {
        stem: stem.to_owned(),
        label: stem.to_owned(),
        config_root: root.to_path_buf(),
        dat_path: None,
        dtt_path: None,
    }
}

fn write(root: &Path, rel: &str, text: &str) {
    let path = rel
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn names(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn characters_follow_the_player_stem() {
    assert_eq!(Character::for_stem("PL000D"), Some(Character::TwoB));
    assert_eq!(Character::for_stem("pl0200"), Some(Character::NineS));
    assert_eq!(Character::for_stem("pl010d"), Some(Character::A2));
    assert_eq!(Character::for_stem("pl1010"), None);
}

#[test]
fn native_outfits_read_meshes_states_and_overrides() {
    let dir = temp("yp_inspector_native_");
    let root = dir.path();
    write(
        root,
        "entities/outfits/outfit_dress.toml",
        "character = \"2B\"\noutfit_id = 7\nname = \"Dress\"\nmeshes = [\"Body\", \"Hair\"]\n\
         combat_meshes = [\"Visor\", \"Eyelash\"]\nnon_combat_meshes = [\"Hat\"]\nprologue_meshes = [\"Proto\"]\n",
    );
    write(
        root,
        "entities/outfits/outfit_other.toml",
        "character = \"9S\"\noutfit_id = 1\nmeshes = [\"X\"]\n",
    );
    write(
        root,
        "entities/meshes.toml",
        "[[mesh_override]]\nmodel = \"pl2b\"\nmesh = \"Cape\"\nforce = \"show\"\n\
         [[mesh_override]]\nmodel = \"pl2b\"\nmesh = \"Hair\"\nforce = \"hide\"\noutfit_id = 7\n\
         [[mesh_override]]\nmodel = \"pl2b\"\nmesh = \"Skip\"\nforce = \"hide\"\noutfit_id = 8\n\
         [[mesh_override]]\nmodel = \"pl9s\"\nmesh = \"Other\"\nforce = \"show\"\n",
    );

    let presets = load(&archive("pl000d", root));
    assert_eq!(presets.len(), 1);
    let preset = &presets[0];
    assert_eq!(preset.outfit_id, 7);
    assert_eq!(preset.name, "Dress");
    assert!(preset.custom_assembly);
    assert!(preset.id.ends_with("outfit_dress.toml|7"));
    assert_eq!(preset.base_meshes, names(&["Body", "Hair"]));
    assert_eq!(preset.force_show_meshes, names(&["Cape"]));
    assert_eq!(preset.force_hide_meshes, names(&["Hair"]));
    let state_ids: Vec<&str> = preset
        .states
        .iter()
        .map(|state| state.id.as_str())
        .collect();
    assert_eq!(state_ids, vec![STATE_PROLOGUE, STATE_COMBAT]);
    let combat = &preset.states[1];
    assert_eq!(
        combat.active_show_meshes,
        names(&["Visor", "Eyelash", "Eyelash_serious"])
    );
    assert_eq!(combat.active_hide_meshes, names(&["Hat"]));
    assert_eq!(
        combat.inactive_hide_meshes,
        names(&["Visor", "Eyelash", "Eyelash_serious"])
    );

    let idle: Vec<String> = preset
        .visible_mesh_names(&HashMap::new())
        .into_iter()
        .collect();
    assert_eq!(idle, names(&["Body", "Hat", "Cape"]));

    let fighting = HashMap::from([(STATE_COMBAT.to_owned(), true)]);
    let mut visible: Vec<String> = preset.visible_mesh_names(&fighting).into_iter().collect();
    visible.sort();
    assert_eq!(
        visible,
        names(&["Body", "Cape", "Eyelash", "Eyelash_serious", "Visor"])
    );

    let prologue = HashMap::from([(STATE_PROLOGUE.to_owned(), true)]);
    let mut visible: Vec<String> = preset.visible_mesh_names(&prologue).into_iter().collect();
    visible.sort();
    assert_eq!(visible, names(&["Cape", "Hat", "Proto"]));
}

#[test]
fn wax_outfits_come_from_json_lists_and_sort_by_id_then_name() {
    let dir = temp("yp_inspector_wax_");
    let root = dir.path();
    put_json(
        root,
        "wax/mods/Pack/outfits/pack.json",
        &json!({
            "pl_2b_outfits": [
                {"outfit_id": 20, "outfit_name": "Zeta", "meshes": ["A"], "eyemask_meshes": ["Mask"], "non_eyemask_meshes": ["Eyes"]},
                {"outfit_id": 3, "outfit_name": "", "broken_meshes": ["Torn"], "self_destruct_hide_meshes": ["Skirt"]},
                {"outfit_id": 20, "outfit_name": "Alpha"},
                {"outfit_name": "No id"}
            ],
            "pl_9s_outfits": [{"outfit_id": 1}]
        }),
    );
    write(
        root,
        "wax/mods/Pack/readme.json",
        "{\"pl_2b_outfits\": [{\"outfit_id\": 99}]}",
    );

    let presets = load(&archive("pl0000", root));
    let summary: Vec<(i64, &str)> = presets
        .iter()
        .map(|preset| (preset.outfit_id, preset.name.as_str()))
        .collect();
    assert_eq!(summary, vec![(3, ""), (20, "Alpha"), (20, "Zeta")]);
    assert!(!presets[1].custom_assembly);
    assert!(presets[2].custom_assembly);
    assert_eq!(presets[0].states[0].id, STATE_SELF_DESTRUCTED);
    assert_eq!(presets[0].states[0].active_show_meshes, names(&["Torn"]));
    assert_eq!(presets[0].states[0].active_hide_meshes, names(&["Skirt"]));
    assert_eq!(presets[2].states[0].id, STATE_EYEMASK);
    assert!(load(&archive("pl1010", root)).is_empty());
}

#[test]
fn configured_mesh_names_cover_every_list() {
    let dir = temp("yp_inspector_cfg_");
    write(
        dir.path(),
        "entities/outfit_a.toml",
        "character = \"A2\"\noutfit_id = 1\nmeshes = [\"B\"]\nwig_meshes = [\"W\"]\nnon_broken_meshes = [\"N\"]\n",
    );
    let presets = load(&archive("pl0100", dir.path()));
    let configured: Vec<String> = presets[0].configured_mesh_names().into_iter().collect();
    assert_eq!(configured, names(&["B", "W", "N"]));
}

#[test]
fn vanilla_variants_are_inferred_from_available_meshes_and_deduplicated() {
    let meshes = ["Body", "Eyelash", "Hair", "Skirt", "Broken", "Armor_Body"];
    let variants = infer_vanilla_variants("pl0000", meshes);
    let summary: Vec<(&str, i64)> = variants
        .iter()
        .map(|preset| (preset.id.as_str(), preset.outfit_id))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("vanilla:normal", -1),
            ("vanilla:damaged", -2),
            ("vanilla:base", -3),
            ("vanilla:armor", -4),
            ("vanilla:dlc", -5),
        ]
    );
    assert_eq!(
        variants[0].base_meshes,
        names(&["Body", "Eyelash", "Hair", "Skirt"])
    );
    assert!(
        variants
            .iter()
            .all(|preset| preset.custom_assembly && preset.name.is_empty())
    );
    assert!(infer_vanilla_variants("pl1010", meshes).is_empty());
    assert!(infer_vanilla_variants("pl0100", ["Nothing"]).is_empty());
}
