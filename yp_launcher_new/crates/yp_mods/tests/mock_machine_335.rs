mod support;

use std::path::PathBuf;

use support::{
    corpus_root, detect, file_names, has_pl_model, install_batch, install_with, mod_dir,
    temp,
};
use yp_mods::drop_gate::{DropGate, classify};
use yp_mods::variants::textures_for_outfit;
use yp_mods::{InstallRequest, VariantInstallRequest};

const TEXTURE_WRAPPERS: [&str; 3] = ["sk_res", "far_res", "inject"];

fn sample_archive() -> Option<PathBuf> {
    let archive = corpus_root()
        .join("335")
        .join("2P - The Mock Machine - All in One.7z");
    if !archive.is_file() {
        eprintln!("skipped: sample archive not present at {}", archive.display());
        return None;
    }
    Some(archive)
}

fn dds_count(dir: &std::path::Path) -> usize {
    file_names(dir)
        .iter()
        .filter(|name| name.to_lowercase().ends_with(".dds"))
        .count()
}

#[test]
fn detect_exposes_outfit_variants_and_several_texture_sets() {
    let Some(archive) = sample_archive() else {
        return;
    };
    let detected = detect(&archive);
    assert!(detected.has_variants(), "expected outfit variants");
    assert!(
        detected.has_texture_variants(),
        "expected several combinable texture sets"
    );
    assert!(detected.texture_variants.len() > 1);
}

#[test]
fn install_one_outfit_and_chosen_texture_sets_merges_flat_into_textures() {
    let Some(archive) = sample_archive() else {
        return;
    };
    let game_dir = temp("yp_multi_tex_");
    let detected = detect(&archive);

    let outfit = detected
        .variants
        .iter()
        .find(|variant| has_pl_model(&detected.unwrapped_root.join(&variant.sub_path)))
        .unwrap_or(&detected.variants[0]);
    let wanted: Vec<PathBuf> = detected
        .texture_variants
        .iter()
        .take(2)
        .map(|pack| {
            pack.path
                .strip_prefix(&detected.unwrapped_root)
                .unwrap()
                .to_path_buf()
        })
        .collect();
    assert_eq!(wanted.len(), 2);

    let installed = install_with(
        game_dir.path(),
        &archive,
        &InstallRequest {
            requested_name: Some("multi_tex_mod".to_owned()),
            variant_sub_path: Some(outfit.sub_path.clone()),
            texture_pack_sub_paths: wanted,
        },
    )
    .unwrap();

    let root = mod_dir(game_dir.path(), &installed.id);
    assert!(root.is_dir());
    assert!(has_pl_model(&root), "outfit model not installed");
    let textures = root.join("textures");
    assert!(textures.is_dir(), "textures/ not created");
    assert!(
        dds_count(&textures) > 0,
        "chosen texture sets were not merged into textures/"
    );

    for entry in std::fs::read_dir(&root).unwrap().filter_map(Result::ok) {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        assert!(
            name == "data" || name == "textures",
            "unexpected leftover dir in mod folder: {name}"
        );
        assert!(!TEXTURE_WRAPPERS.contains(&name.as_str()));
    }
}

#[test]
fn batch_install_one_extract_installs_several_outfits_as_own_mods() {
    let Some(archive) = sample_archive() else {
        return;
    };
    let game_dir = temp("yp_batch_");
    let detected = detect(&archive);
    let outfits: Vec<_> = detected
        .variants
        .iter()
        .filter(|variant| variant.is_outfit())
        .take(3)
        .collect();
    assert!(outfits.len() >= 2);

    let all_textures: Vec<PathBuf> = detected
        .texture_variants
        .iter()
        .map(|pack| {
            pack.path
                .strip_prefix(&detected.unwrapped_root)
                .unwrap()
                .to_path_buf()
        })
        .collect();
    let requests: Vec<VariantInstallRequest> = outfits
        .iter()
        .map(|outfit| VariantInstallRequest {
            requested_name: format!("batch - {}", outfit.label),
            variant_sub_path: outfit.sub_path.clone(),
            texture_pack_sub_paths: all_textures.clone(),
        })
        .collect();

    let results = install_batch(game_dir.path(), &archive, &requests);
    assert_eq!(results.len(), outfits.len());
    assert!(results.iter().all(Result::is_ok), "{results:?}");

    let mut ids: Vec<String> = results
        .iter()
        .map(|result| result.as_ref().unwrap().id.clone())
        .collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), outfits.len(), "ids not unique: {ids:?}");

    for id in &ids {
        let root = mod_dir(game_dir.path(), id);
        assert!(has_pl_model(&root), "no pl model for {id}");
        assert!(
            dds_count(&root.join("textures")) > 0,
            "no textures bundled for {id}"
        );
    }
}

#[test]
fn each_outfit_is_paired_with_the_shared_set_and_its_own_set() {
    let Some(archive) = sample_archive() else {
        return;
    };
    let detected = detect(&archive);
    let classic = detected
        .variants
        .iter()
        .find(|variant| variant.label == "2P - Classic")
        .expect("classic outfit variant");

    let textures = PathBuf::from("Textures");
    assert_eq!(
        textures_for_outfit(classic, &detected),
        vec![
            textures.join("2P I Classic"),
            textures.join("2P I Shared Textures")
        ]
    );
    assert_eq!(classify(&support::staging(), &archive), DropGate::Mod);
    for outfit in detected
        .variants
        .iter()
        .filter(|variant| variant.is_outfit())
    {
        let sets = textures_for_outfit(outfit, &detected);
        assert_eq!(sets.len(), 2, "{}: {sets:?}", outfit.label);
    }
}
