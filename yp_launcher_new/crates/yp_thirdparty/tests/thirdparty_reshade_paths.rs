mod common;

use std::path::Path;

use common::{temp, write};
use yp_core::ini::IniDoc;
use yp_thirdparty::reshade::{
    addon_companion_dlls, ensure_ini_paths, referenced_search_dirs, ADDON_SEARCH_PATH, SHADER_SEARCH_PATH,
    TEXTURE_SEARCH_PATH,
};
use yp_thirdparty::tree::{self, ExistingFiles};
use yp_thirdparty::{classify, ThirdPartyKind};

fn ini(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("ReShade.ini")).unwrap()
}

fn key(text: &str, section: &str, name: &str) -> Option<String> {
    IniDoc::parse(text).get(Some(section), name).map(str::to_owned)
}

mod reshade_ini_search_paths {
    use super::*;

    #[test]
    fn a_missing_ini_gets_the_effect_and_texture_folders() {
        let dest = temp("yp_rs_ini_");

        ensure_ini_paths(dest.path());

        let text = ini(dest.path());
        assert_eq!(key(&text, "GENERAL", "EffectSearchPaths").as_deref(), Some(SHADER_SEARCH_PATH));
        assert_eq!(key(&text, "GENERAL", "TextureSearchPaths").as_deref(), Some(TEXTURE_SEARCH_PATH));
    }

    #[test]
    fn a_plain_dot_search_path_no_longer_hides_reshade_shaders() {
        let dest = temp("yp_rs_ini_");
        write(&dest.path().join("ReShade.ini"), "[GENERAL]\nEffectSearchPaths=.\\\nTextureSearchPaths=.\\\n");

        ensure_ini_paths(dest.path());

        let effects = key(&ini(dest.path()), "GENERAL", "EffectSearchPaths").unwrap();
        assert_eq!(effects.split(',').next(), Some(SHADER_SEARCH_PATH));
    }

    #[test]
    fn a_non_recursive_reshade_shaders_entry_becomes_recursive() {
        let dest = temp("yp_rs_ini_");
        std::fs::create_dir(dest.path().join("my-fx")).unwrap();
        write(
            &dest.path().join("ReShade.ini"),
            "[GENERAL]\nEffectSearchPaths=.\\reshade-shaders\\Shaders,.\\my-fx\\**,.\\ReShade_shaders\\Shaders,C:\\Old\\Shaders\n",
        );

        ensure_ini_paths(dest.path());

        assert_eq!(
            key(&ini(dest.path()), "GENERAL", "EffectSearchPaths"),
            Some(format!("{SHADER_SEARCH_PATH},.\\my-fx\\**"))
        );
    }

    #[test]
    fn a_correct_ini_is_left_as_it_is() {
        let dest = temp("yp_rs_ini_");
        let content =
            "[GENERAL]\nEffectSearchPaths=.\\reshade-shaders\\Shaders\\**\nTextureSearchPaths=.\\reshade-shaders\\Textures\\**\n";
        write(&dest.path().join("ReShade.ini"), content);

        ensure_ini_paths(dest.path());

        assert_eq!(ini(dest.path()), content);
    }

    #[test]
    fn add_ons_in_reshade_addons_get_an_addon_path() {
        let dest = temp("yp_rs_ini_");
        write(&dest.path().join("reshade-addons").join("Upscaler.addon64"), "x");

        ensure_ini_paths(dest.path());

        assert_eq!(key(&ini(dest.path()), "ADDON", "AddonPath").as_deref(), Some(ADDON_SEARCH_PATH));
    }

    #[test]
    fn add_ons_next_to_the_reshade_dll_keep_the_default_addon_path() {
        let dest = temp("yp_rs_ini_");
        write(&dest.path().join("Upscaler.addon64"), "x");
        write(&dest.path().join("reshade-addons").join("Other.addon64"), "x");

        ensure_ini_paths(dest.path());

        assert_eq!(key(&ini(dest.path()), "ADDON", "AddonPath"), None);
    }
}

mod importing_from_the_game_folder {
    use super::*;

    #[test]
    fn shader_folders_named_in_the_game_reshade_ini_are_found() {
        let game = temp("yp_rs_game_");
        std::fs::create_dir_all(game.path().join("ReShade_shaders").join("Shaders")).unwrap();
        std::fs::create_dir_all(game.path().join("reshade-shaders").join("Shaders")).unwrap();
        write(
            &game.path().join("ReShade.ini"),
            "[GENERAL]\nEffectSearchPaths=.\\ReShade_shaders\\Shaders\\**,.\\reshade-shaders\\Shaders\\**,.\\,..\\,.\\missing\n",
        );

        let dirs = referenced_search_dirs(game.path(), "EffectSearchPaths");

        assert_eq!(dirs.len(), 1);
        assert!(tree::same_path(&dirs[0], &game.path().join("ReShade_shaders").join("Shaders")));
    }
}

mod symbolic_links {
    use super::*;

    #[cfg(windows)]
    fn link_dir(target: &Path, link: &Path) -> bool {
        std::os::windows::fs::symlink_dir(target, link).is_ok()
    }

    #[cfg(not(windows))]
    fn link_dir(target: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(target, link).is_ok()
    }

    #[cfg(windows)]
    fn link_file(target: &Path, link: &Path) -> bool {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }

    #[cfg(not(windows))]
    fn link_file(target: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(target, link).is_ok()
    }

    #[test]
    fn merging_copies_the_content_of_a_linked_shader_folder() {
        let root = temp("yp_rs_link_");
        let library = root.path().join("library").join("SweetFX");
        write(&library.join("Clarity.fx"), "technique Clarity {}");
        let src = root.path().join("linked");
        std::fs::create_dir(&src).unwrap();
        if !link_dir(&library, &src.join("SweetFX")) {
            return;
        }

        let dest = root.path().join("out");
        tree::merge_tree(&src, &dest, ExistingFiles::Overwrite);

        let copied = dest.join("SweetFX").join("Clarity.fx");
        assert_eq!(std::fs::read_to_string(&copied).unwrap(), "technique Clarity {}");
        assert!(
            !std::fs::symlink_metadata(dest.join("SweetFX")).unwrap().file_type().is_symlink(),
            "the copy is a real folder, not another link"
        );
    }

    #[test]
    fn merging_copies_what_a_linked_shader_file_points_to() {
        let root = temp("yp_rs_link_");
        let real = root.path().join("library").join("Clarity.fx");
        write(&real, "technique Clarity {}");
        let src = root.path().join("linked");
        std::fs::create_dir(&src).unwrap();
        if !link_file(&real, &src.join("Clarity.fx")) {
            return;
        }

        let dest = root.path().join("out");
        tree::merge_tree(&src, &dest, ExistingFiles::Overwrite);

        let copied = dest.join("Clarity.fx");
        assert!(copied.is_file());
        assert!(
            !std::fs::symlink_metadata(&copied).unwrap().file_type().is_symlink(),
            "the copy is a real file, not another link"
        );
        assert_eq!(std::fs::read_to_string(&copied).unwrap(), "technique Clarity {}");
    }
}

mod reshade_add_on_archives {
    use super::*;

    #[test]
    fn an_add_on_with_upscaler_dlls_is_a_reshade_install() {
        let root = temp("yp_rs_addon_");
        let folder = root.path().join("NierASR");
        write(&folder.join("NierASR.addon64"), "addon");
        write(&folder.join("sl.interposer.dll"), "dll");
        write(&folder.join("nvngx_dlss.dll"), "dll");
        write(&folder.join("d3dcompiler_47.dll"), "dll");
        write(&folder.join("dxgi.dll"), "dll");

        let c = classify(root.path());
        assert_eq!(c.kind, ThirdPartyKind::ReShadeWholeInstall);
        assert!(c.is_addon_build);

        let mut companions = common::file_names(&addon_companion_dlls(&[folder.join("NierASR.addon64")]));
        companions.sort();
        assert_eq!(companions, ["nvngx_dlss.dll", "sl.interposer.dll"]);
    }
}
