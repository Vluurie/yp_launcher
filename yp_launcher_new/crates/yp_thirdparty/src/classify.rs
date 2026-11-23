use std::path::Path;

use crate::dll_id::{self, GraphicsDll};
use crate::model::{Classification, ThirdPartyKind};
use crate::paths::{self, base_name, DIRECTX_PROXY_NAMES};
use crate::tree;

pub fn classify(extracted_root: &Path) -> Classification {
    let root = tree::unwrap_single_child(extracted_root);
    let found = |kind: ThirdPartyKind| Classification::new(kind, root.clone());

    let inner = paths::find_inner_archives(&root);
    if inner.len() >= 2 {
        return Classification {
            inner_archives: inner,
            ..found(ThirdPartyKind::MultiBundle)
        };
    }

    if paths::find_graphics_dll(&root, GraphicsDll::Nams).is_some() {
        return found(ThirdPartyKind::NamsDll);
    }

    if let Some(lodmod_ini) = tree::first_file_where(&root, |rel, _| rel == "lodmod.ini" || rel.ends_with("/lodmod.ini")) {
        return Classification {
            lodmod_ini: Some(lodmod_ini),
            ..found(ThirdPartyKind::LodMod)
        };
    }

    let migoto_dll = paths::find_graphics_dll(&root, GraphicsDll::Migoto);
    let has_d3dx = paths::has_file_matching(&root, |rel| base_name(rel) == paths::MIGOTO_INI_NAME);
    let has_shader_fixes = paths::has_dir_named(&root, "shaderfixes");
    if migoto_dll.is_some() || (has_d3dx && has_shader_fixes) {
        return found(ThirdPartyKind::Migoto);
    }

    let special_k_dll = tree::any_file(&root, |rel, full| {
        base_name(rel).ends_with(".dll")
            && dll_id::identify_file(full) == GraphicsDll::Other
            && paths::is_special_k_dll(full)
    });
    if special_k_dll || paths::has_file_matching(&root, |rel| rel.contains("specialk") || base_name(rel).starts_with("sk_")) {
        return found(ThirdPartyKind::SpecialK);
    }

    if paths::has_file_matching(&root, |rel| base_name(rel) == "d3d11_shaders.ini") {
        return found(ThirdPartyKind::SkShaderEdit);
    }

    let reshade_dll = paths::find_graphics_dll(&root, GraphicsDll::ReShade);
    let shaders = paths::has_reshade_shaders(&root);
    let presets = paths::find_presets(&root);
    let addons = paths::has_addon_files(&root);

    let has_dxgi_ini = paths::has_file_matching(&root, |rel| base_name(rel) == "dxgi.ini");
    if has_dxgi_ini && reshade_dll.is_none() && presets.is_empty() && !shaders && !addons {
        return found(ThirdPartyKind::BrokenLegacy);
    }

    if reshade_dll.is_some() || shaders || addons {
        return Classification {
            has_reshade_dll: reshade_dll.is_some(),
            is_addon_build: addons || reshade_dll.as_ref().is_some_and(|hit| hit.is_addon),
            has_shaders: shaders,
            preset_count: presets.len(),
            ..found(ThirdPartyKind::ReShadeWholeInstall)
        };
    }

    if !presets.is_empty() {
        return Classification {
            has_shaders: shaders,
            preset_count: presets.len(),
            ..found(ThirdPartyKind::ReShadePreset)
        };
    }

    if paths::has_file_matching(&root, |rel| {
        let base = base_name(rel);
        base.ends_with(".dtt") || base.ends_with(".dat") || base.ends_with(".cpk") || base.ends_with(".pak")
    }) {
        return found(ThirdPartyKind::GameData);
    }

    let game_mod_dlls = paths::find_game_mod_dlls(&root);
    let proxies: Vec<_> = game_mod_dlls
        .iter()
        .filter(|dll| DIRECTX_PROXY_NAMES.contains(&tree::file_name_lower(dll).as_str()))
        .cloned()
        .collect();
    if !proxies.is_empty() {
        return Classification {
            graphic_mod_dlls: proxies,
            ..found(ThirdPartyKind::GraphicMod)
        };
    }
    if !game_mod_dlls.is_empty() {
        return Classification {
            game_mod_dlls,
            ..found(ThirdPartyKind::GameMod)
        };
    }

    if paths::has_file_matching(&root, |rel| base_name(rel).ends_with(".dds")) {
        return found(ThirdPartyKind::Textures);
    }

    if paths::has_file_matching(&root, |rel| base_name(rel).ends_with(".cfg")) || paths::only_images(&root) {
        return found(ThirdPartyKind::BrokenLegacy);
    }

    found(ThirdPartyKind::Unknown)
}

pub fn source_name_from_path(path: &Path) -> String {
    let stem = tree::file_stem(path);
    let mut spaced = String::with_capacity(stem.len());
    let mut in_underscores = false;
    for c in stem.chars() {
        if c == '_' {
            if !in_underscores {
                spaced.push(' ');
            }
            in_underscores = true;
        } else {
            spaced.push(c);
            in_underscores = false;
        }
    }
    let without_counter = strip_dash_counter(&spaced);
    let collapsed = without_counter.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() { tree::file_name(path) } else { collapsed }
}

fn strip_dash_counter(name: &str) -> &str {
    let digits = name.chars().rev().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return name;
    }
    let head = &name[..name.len() - digits];
    head.strip_suffix('-').unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_names_from_archive_paths() {
        assert_eq!(source_name_from_path(Path::new("dl/Bande_Desinee-123.zip")), "Bande Desinee");
        assert_eq!(source_name_from_path(Path::new("dl/Bloom__Fix.7z")), "Bloom Fix");
        assert_eq!(source_name_from_path(Path::new("dl/celestial.dll")), "celestial");
        assert_eq!(source_name_from_path(Path::new("dl/a-b-12.zip")), "a-b");
        assert_eq!(source_name_from_path(Path::new("dl/v-1.2.zip")), "v-1.2");
        assert_eq!(source_name_from_path(Path::new("dl/___.zip")), "___.zip");
        assert_eq!(source_name_from_path(Path::new("dl/  spaced   name .zip")), "spaced name");
    }
}
