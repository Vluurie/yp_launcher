use std::path::{Path, PathBuf};

use yp_core::consts::{
    GAME_MODS_DIR_NAME, GRAPHIC_MODS_DIR_NAME, MIGOTO_DIR_NAME, RESHADE_DIR_NAME, THIRD_PARTY_DIR_NAME,
    THIRD_PARTY_DISABLED_DIR_NAME,
};
use yp_core::ini::IniDoc;
use yp_core::{GamePaths, IoOp, IoResultExt};

use crate::dll_id::{self, GraphicsDll};
use crate::tree::{self, EntryKind};

pub const RESHADE_INI_NAME: &str = "ReShade.ini";
pub const MIGOTO_INI_NAME: &str = "d3dx.ini";
pub const PRESET_TECHNIQUES_KEY: &str = "Techniques";

pub const DIRECTX_PROXY_NAMES: [&str; 5] = ["d3d9.dll", "d3d10.dll", "d3d11.dll", "d3d12.dll", "dxgi.dll"];

const SPECIAL_K_MARKER: &[u8] = b"SpecialK";

const RESHADE_TREE: [&[&str]; 5] = [
    &["reshade-presets"],
    &["reshade-shaders", "Shaders"],
    &["reshade-shaders", "Textures"],
    &["reshade-addons"],
    &["reshade-cache"],
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DllHit {
    pub file: PathBuf,
    pub is_addon: bool,
}

pub fn root(game: &GamePaths) -> PathBuf {
    game.thirdparty_dir()
}

pub fn legacy_root(launcher_dir: &Path) -> PathBuf {
    launcher_dir.join(THIRD_PARTY_DIR_NAME)
}

pub fn reshade(game: &GamePaths) -> PathBuf {
    root(game).join(RESHADE_DIR_NAME)
}

pub fn migoto(game: &GamePaths) -> PathBuf {
    root(game).join(MIGOTO_DIR_NAME)
}

pub fn game_mods(game: &GamePaths) -> PathBuf {
    root(game).join(GAME_MODS_DIR_NAME)
}

pub fn graphic_mods(game: &GamePaths) -> PathBuf {
    root(game).join(GRAPHIC_MODS_DIR_NAME)
}

pub fn graphic_mods_disabled(game: &GamePaths) -> PathBuf {
    root(game).join(THIRD_PARTY_DISABLED_DIR_NAME).join(GRAPHIC_MODS_DIR_NAME)
}

pub fn mod_dirs(kind_dir: &Path) -> Vec<PathBuf> {
    tree::list_dir(kind_dir)
        .into_iter()
        .filter(|entry| entry.kind == EntryKind::Dir)
        .map(|entry| entry.path)
        .collect()
}

pub fn dlls_in(dir: &Path) -> Vec<PathBuf> {
    tree::list_dir(dir)
        .into_iter()
        .filter(|entry| entry.kind == EntryKind::File && entry.name().to_lowercase().ends_with(".dll"))
        .map(|entry| entry.path)
        .collect()
}

pub fn mod_dir_with_dll<S: AsRef<str>>(kind_dir: &Path, dll_names: &[S]) -> Option<PathBuf> {
    let wanted: Vec<String> = dll_names.iter().map(|name| name.as_ref().to_lowercase()).collect();
    mod_dirs(kind_dir)
        .into_iter()
        .find(|dir| dlls_in(dir).iter().any(|dll| wanted.contains(&tree::file_name_lower(dll))))
}

pub fn find_dll(kind_dir: &Path, file_name: &str) -> Option<PathBuf> {
    let wanted = file_name.to_lowercase();
    mod_dirs(kind_dir)
        .into_iter()
        .flat_map(|dir| dlls_in(&dir))
        .find(|dll| tree::file_name_lower(dll) == wanted)
}

pub fn mod_dir_name(raw: &str, fallback: &str) -> String {
    let mut collapsed = String::with_capacity(raw.len());
    let mut pending_space = false;
    for c in raw.chars() {
        let blank =
            c.is_whitespace() || u32::from(c) < 0x20 || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*');
        if blank {
            pending_space = true;
            continue;
        }
        if pending_space {
            collapsed.push(' ');
            pending_space = false;
        }
        collapsed.push(c);
    }
    let name = collapsed.trim_end_matches(['.', ' ']).trim();
    if name.is_empty() { fallback.to_owned() } else { name.to_owned() }
}

pub fn unique_dir_name(parent: &Path, name: &str) -> String {
    let taken = |candidate: &str| std::fs::symlink_metadata(parent.join(candidate)).is_ok();
    if !taken(name) {
        return name.to_owned();
    }
    let mut index = 2;
    while taken(&format!("{name} ({index})")) {
        index += 1;
    }
    format!("{name} ({index})")
}

pub fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && Path::new(name).file_name().is_some_and(|base| base == name)
}

pub fn base_name(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

pub fn ensure_reshade_tree(dest: &Path) -> yp_core::Result<()> {
    for parts in RESHADE_TREE {
        let dir = parts.iter().fold(dest.to_path_buf(), |dir, part| dir.join(part));
        std::fs::create_dir_all(&dir).io(IoOp::Create, &dir)?;
    }
    Ok(())
}

pub fn has_file_matching(root: &Path, mut pred: impl FnMut(&str) -> bool) -> bool {
    tree::any_file(root, |rel, _| pred(rel))
}

pub fn has_dir_named(root: &Path, name_lower: &str) -> bool {
    find_dir_named(root, name_lower).is_some()
}

pub fn find_dir_named(root: &Path, name_lower: &str) -> Option<PathBuf> {
    let mut hit = None;
    tree::walk_entities(root, &mut |entry| {
        if entry.is_dir() && entry.name().to_lowercase() == name_lower {
            hit = Some(entry.path.clone());
            return false;
        }
        true
    });
    hit
}

pub fn has_reshade_shaders(root: &Path) -> bool {
    find_dir_named(root, "shaders").is_some_and(|shaders| tree::any_file(&shaders, |rel, _| rel.ends_with(".fx")))
}

pub fn is_addon_name(name_lower: &str) -> bool {
    name_lower.ends_with(".addon") || name_lower.ends_with(".addon32") || name_lower.ends_with(".addon64")
}

pub fn has_addon_files(root: &Path) -> bool {
    tree::any_file(root, |rel, _| is_addon_name(rel))
}

pub fn is_preset_file(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return false;
    };
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    IniDoc::parse(text)
        .entries()
        .iter()
        .any(|(_, key, _)| *key == PRESET_TECHNIQUES_KEY)
}

pub fn find_presets(root: &Path) -> Vec<PathBuf> {
    tree::files_where(root, |rel, full| rel.ends_with(".ini") && is_preset_file(full))
}

pub fn only_images(root: &Path) -> bool {
    let files = tree::files_where(root, |rel, _| {
        let base = base_name(rel);
        !base.ends_with(".txt") && !base.ends_with(".md") && !base.ends_with(".pdf") && !base.starts_with("readme")
    });
    !files.is_empty()
        && files.iter().all(|file| {
            let name = tree::file_name_lower(file);
            name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") || name.ends_with(".dds")
        })
}

pub fn find_graphics_dll(root: &Path, want: GraphicsDll) -> Option<DllHit> {
    let mut hit = None;
    tree::walk_entities(root, &mut |entry| {
        if !entry.is_file() {
            return true;
        }
        let name = entry.name().to_lowercase();
        if !name.ends_with(".dll") || name == "nvapi64.dll" || name == "d3dcompiler_46.dll" {
            return true;
        }
        if dll_id::identify_file(&entry.path) == want {
            hit = Some(DllHit {
                file: entry.path.clone(),
                is_addon: name.contains("_addon"),
            });
            return false;
        }
        true
    });
    hit
}

pub fn is_special_k_dll(path: &Path) -> bool {
    std::fs::read(path).is_ok_and(|bytes| dll_id::contains_bytes(&bytes, SPECIAL_K_MARKER))
}

pub fn find_game_mod_dlls(root: &Path) -> Vec<PathBuf> {
    let mut out = tree::files_where(root, |rel, full| {
        base_name(rel).ends_with(".dll") && dll_id::identify_file(full) == GraphicsDll::Other
    });
    out.sort_by_cached_key(|path| (tree::file_name_lower(path), path.clone()));
    out
}

pub fn find_inner_archives(root: &Path) -> Vec<PathBuf> {
    tree::files_where(root, |rel, _| {
        let base = base_name(rel);
        base.ends_with(".zip") || base.ends_with(".7z") || base.ends_with(".rar")
    })
}

pub fn unique_file_name(dir: &Path, file_name: &str) -> String {
    if !dir.join(file_name).is_file() {
        return file_name.to_owned();
    }
    let path = Path::new(file_name);
    let stem = tree::file_stem(path);
    let ext = path.extension().map(|ext| format!(".{}", ext.to_string_lossy())).unwrap_or_default();
    let mut index = 2;
    loop {
        let candidate = format!("{stem} ({index}){ext}");
        if !dir.join(&candidate).is_file() {
            return candidate;
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_dir_names_are_windows_safe() {
        assert_eq!(mod_dir_name("Celestial Timer", "x"), "Celestial Timer");
        assert_eq!(mod_dir_name("a<b>c:d\"e/f\\g|h?i*j", "x"), "a b c d e f g h i j");
        assert_eq!(mod_dir_name("  spaced \t\n name...  ", "x"), "spaced name");
        assert_eq!(mod_dir_name("...", "fallback"), "fallback");
        assert_eq!(mod_dir_name("", "fallback"), "fallback");
        assert_eq!(mod_dir_name("tab\u{1}ctrl", "x"), "tab ctrl");
    }

    #[test]
    fn plain_names() {
        assert!(is_plain_name("celestial"));
        assert!(is_plain_name("Celestial Timer"));
        assert!(!is_plain_name(""));
        assert!(!is_plain_name("."));
        assert!(!is_plain_name(".."));
        assert!(!is_plain_name("a/b"));
        assert!(!is_plain_name("a\\b"));
    }

    #[test]
    fn unique_names_count_up() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(unique_dir_name(dir.path(), "mod"), "mod");
        std::fs::create_dir(dir.path().join("mod")).unwrap();
        std::fs::create_dir(dir.path().join("mod (2)")).unwrap();
        assert_eq!(unique_dir_name(dir.path(), "mod"), "mod (3)");
        std::fs::write(dir.path().join("p.ini"), "x").unwrap();
        assert_eq!(unique_file_name(dir.path(), "p.ini"), "p (2).ini");
        assert_eq!(unique_file_name(dir.path(), "q.ini"), "q.ini");
    }

    #[test]
    fn base_name_of_relative_paths() {
        assert_eq!(base_name("a/b/c.dll"), "c.dll");
        assert_eq!(base_name("c.dll"), "c.dll");
    }

    #[test]
    fn layout_matches_nams() {
        let game = GamePaths::new("G");
        assert_eq!(reshade(&game), PathBuf::from("G").join("nams").join("thirdparty").join("reshade"));
        assert_eq!(migoto(&game), PathBuf::from("G").join("nams").join("thirdparty").join("3dmigoto"));
        assert_eq!(
            graphic_mods_disabled(&game),
            PathBuf::from("G").join("nams").join("thirdparty").join("disabled").join("graphic")
        );
        assert_eq!(legacy_root(Path::new("L")), PathBuf::from("L").join("thirdparty"));
    }
}
