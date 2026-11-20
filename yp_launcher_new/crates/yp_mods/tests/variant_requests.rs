mod support;

use std::path::{Path, PathBuf};

use support::{detect, put_bytes, temp};
use yp_mods::naming::prettify_mod_id;
use yp_mods::variants::{
    install_requests, requests_without_textures, texture_set_choices, textures_for_outfit,
};
use yp_mods::{DataCategory, DetectedDrop, ModKind, ModVariant, TexturePack};

fn variant(label: &str, category: DataCategory) -> ModVariant {
    ModVariant {
        sub_path: PathBuf::from(label),
        label: label.to_owned(),
        kind: ModKind::Data,
        texture_only: false,
        category: Some(category),
    }
}

fn drop_with_texture_sets(root: &Path, sets: &[&str]) -> DetectedDrop {
    DetectedDrop {
        unwrapped_root: root.to_path_buf(),
        kind: ModKind::Unknown,
        manifest: None,
        native: None,
        data: None,
        suggested_id: "pack".to_owned(),
        rejection: None,
        variants: Vec::new(),
        texture_variants: sets
            .iter()
            .map(|label| TexturePack {
                path: root.join("Textures").join(label),
                label: (*label).to_owned(),
            })
            .collect(),
    }
}

fn sub_paths(items: &[&str]) -> Vec<PathBuf> {
    items
        .iter()
        .map(|item| Path::new("Textures").join(item))
        .collect()
}

const MOCK_MACHINE_SETS: [&str; 5] = [
    "2P I Classic",
    "2P I Final Fantasy",
    "2P I or not 2P",
    "2P I Shared Textures",
    "2P I Soulcalibur",
];

#[test]
fn an_outfit_gets_the_shared_set_and_its_own_set() {
    let root = Path::new("drop");
    let detected = drop_with_texture_sets(root, &MOCK_MACHINE_SETS);

    assert_eq!(
        textures_for_outfit(&variant("2P - Classic", DataCategory::Player), &detected),
        sub_paths(&["2P I Classic", "2P I Shared Textures"])
    );
    assert_eq!(
        textures_for_outfit(&variant("2P - or not 2P", DataCategory::Player), &detected),
        sub_paths(&["2P I or not 2P", "2P I Shared Textures"])
    );
    assert_eq!(
        textures_for_outfit(
            &variant("Unrelated Outfit", DataCategory::Player),
            &detected
        ),
        sub_paths(&["2P I Shared Textures"])
    );
}

#[test]
fn nothing_matches_without_texture_sets() {
    let detected = drop_with_texture_sets(Path::new("drop"), &[]);
    assert!(
        textures_for_outfit(&variant("2P - Classic", DataCategory::Player), &detected).is_empty()
    );
}

#[test]
fn install_requests_name_each_variant_and_skip_textures_for_weapons() {
    let detected = drop_with_texture_sets(Path::new("drop"), &MOCK_MACHINE_SETS);
    let chosen = [
        variant("2P - Soulcalibur", DataCategory::Player),
        variant("Mock Type-4O Sword", DataCategory::Weapon),
    ];

    let requests = install_requests("Mock Machine", &chosen, &detected);

    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0].requested_name,
        "Mock Machine - 2P - Soulcalibur"
    );
    assert_eq!(
        requests[0].variant_sub_path,
        PathBuf::from("2P - Soulcalibur")
    );
    assert_eq!(
        requests[0].texture_pack_sub_paths,
        sub_paths(&["2P I Shared Textures", "2P I Soulcalibur"])
    );
    assert_eq!(
        requests[1].requested_name,
        "Mock Machine - Mock Type-4O Sword"
    );
    assert!(requests[1].texture_pack_sub_paths.is_empty());

    let plain = requests_without_textures("Mock Machine", &chosen);
    assert_eq!(plain[0].requested_name, "Mock Machine - 2P - Soulcalibur");
    assert_eq!(
        plain[1].variant_sub_path,
        PathBuf::from("Mock Type-4O Sword")
    );
    assert!(
        plain
            .iter()
            .all(|request| request.texture_pack_sub_paths.is_empty())
    );
}

#[test]
fn texture_sets_become_selectable_choices() {
    let detected = drop_with_texture_sets(Path::new("drop"), &["City", "Desert"]);

    let choices = texture_set_choices(&detected);

    assert_eq!(
        choices,
        vec![
            ModVariant {
                sub_path: Path::new("Textures").join("City"),
                label: "City".to_owned(),
                kind: ModKind::Texture,
                texture_only: true,
                category: None,
            },
            ModVariant {
                sub_path: Path::new("Textures").join("Desert"),
                label: "Desert".to_owned(),
                kind: ModKind::Texture,
                texture_only: true,
                category: None,
            },
        ]
    );
}

#[test]
fn a_detected_folder_drop_feeds_the_request_builders() {
    let dir = temp("yp_variant_requests_");
    let pack = dir.path().join("mock_machine_1_2");
    put_bytes(&pack, "2P - Classic/data/pl/pl000d.dat", 8, 1);
    put_bytes(&pack, "2P - Classic/data/pl/pl000d.dtt", 8, 1);
    put_bytes(&pack, "2P - Soulcalibur/data/pl/pl000d.dat", 8, 2);
    put_bytes(&pack, "2P - Soulcalibur/data/pl/pl000d.dtt", 8, 2);
    put_bytes(&pack, "Textures/2P I Classic/a.dds", 8, 3);
    put_bytes(&pack, "Textures/2P I Shared Textures/b.dds", 8, 4);
    put_bytes(&pack, "Textures/2P I Soulcalibur/c.dds", 8, 5);

    let detected = detect(&pack);
    let base_name = prettify_mod_id(&detected.suggested_id);
    let outfits: Vec<ModVariant> = detected
        .variants
        .iter()
        .filter(|variant| variant.is_outfit())
        .cloned()
        .collect();
    let requests = install_requests(&base_name, &outfits, &detected);
    let results = support::install_batch(dir.path().join("game").as_path(), &pack, &requests);

    assert_eq!(base_name, "Mock Machine");
    assert_eq!(
        requests
            .iter()
            .map(|request| request.requested_name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Mock Machine - 2P - Classic",
            "Mock Machine - 2P - Soulcalibur"
        ]
    );
    assert_eq!(
        requests[0].texture_pack_sub_paths,
        sub_paths(&["2P I Classic", "2P I Shared Textures"])
    );
    let ids: Vec<String> = results
        .into_iter()
        .map(|result| result.unwrap().id)
        .collect();
    assert_eq!(
        ids,
        vec!["mock_machine_2p_classic", "mock_machine_2p_soulcalibur"]
    );
    let classic_textures = support::file_names(
        &support::mod_dir(dir.path().join("game").as_path(), &ids[0]).join("textures"),
    );
    assert_eq!(classic_textures, vec!["a.dds", "b.dds"]);
}
