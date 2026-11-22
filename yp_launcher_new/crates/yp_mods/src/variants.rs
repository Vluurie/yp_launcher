use std::path::PathBuf;

use crate::fsx;
use crate::model::{DetectedDrop, ModKind, ModVariant, TexturePack, VariantInstallRequest};
use crate::text::{dart_trim, is_regex_whitespace};

pub const OUTFIT_KEY_NOISE: [&str; 7] = ["2p", "i", "-", "/", "shared", "textures", "outfit"];
const SHARED_SET_MARKER: &str = "shared";

pub fn outfit_match_key(label: &str) -> String {
    let mut key = label.to_lowercase();
    for noise in OUTFIT_KEY_NOISE {
        key = key.replace(noise, " ");
    }
    let mut collapsed = String::with_capacity(key.len());
    let mut in_whitespace = false;
    for c in key.chars() {
        if is_regex_whitespace(c) {
            if !in_whitespace {
                collapsed.push(' ');
            }
            in_whitespace = true;
        } else {
            in_whitespace = false;
            collapsed.push(c);
        }
    }
    dart_trim(&collapsed).to_owned()
}

fn sub_path_in_drop(pack: &TexturePack, detected: &DetectedDrop) -> PathBuf {
    fsx::relative(&pack.path, &detected.unwrapped_root).unwrap_or_else(|| pack.path.clone())
}

pub fn textures_for_outfit(outfit: &ModVariant, detected: &DetectedDrop) -> Vec<PathBuf> {
    let outfit_key = outfit_match_key(&outfit.label);
    detected
        .texture_variants
        .iter()
        .filter(|pack| {
            pack.label.to_lowercase().contains(SHARED_SET_MARKER)
                || outfit_match_key(&pack.label) == outfit_key
        })
        .map(|pack| sub_path_in_drop(pack, detected))
        .collect()
}

pub fn texture_set_choices(detected: &DetectedDrop) -> Vec<ModVariant> {
    detected
        .texture_variants
        .iter()
        .map(|pack| ModVariant {
            sub_path: sub_path_in_drop(pack, detected),
            label: pack.label.clone(),
            kind: ModKind::Texture,
            texture_only: true,
            category: None,
        })
        .collect()
}

fn variant_name(base_name: &str, variant: &ModVariant) -> String {
    format!("{base_name} - {}", variant.label)
}

pub fn install_requests(
    base_name: &str,
    chosen: &[ModVariant],
    detected: &DetectedDrop,
) -> Vec<VariantInstallRequest> {
    chosen
        .iter()
        .map(|variant| VariantInstallRequest {
            requested_name: variant_name(base_name, variant),
            variant_sub_path: variant.sub_path.clone(),
            texture_pack_sub_paths: if variant.is_weapon() {
                Vec::new()
            } else {
                textures_for_outfit(variant, detected)
            },
        })
        .collect()
}

pub fn requests_without_textures(
    base_name: &str,
    variants: &[ModVariant],
) -> Vec<VariantInstallRequest> {
    variants
        .iter()
        .map(|variant| VariantInstallRequest {
            requested_name: variant_name(base_name, variant),
            variant_sub_path: variant.sub_path.clone(),
            texture_pack_sub_paths: Vec::new(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_keys_drop_the_noise_words_in_order() {
        assert_eq!(outfit_match_key("2P - Classic"), "class c");
        assert_eq!(outfit_match_key("2P I Classic"), "class c");
        assert_eq!(outfit_match_key("2P I Shared Textures"), "");
        assert_eq!(outfit_match_key("2P - Final Fantasy"), "f nal fantasy");
        assert_eq!(
            outfit_match_key("Textures / 2P I Soulcalibur"),
            "soulcal bur"
        );
        assert_eq!(outfit_match_key("Outfit Alpha"), "outf t alpha");
        assert_eq!(outfit_match_key("  Tab\tSeparated  "), "tab separated");
    }
}
