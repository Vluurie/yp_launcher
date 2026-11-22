use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use yp_core::consts::RESHADE_MOD_DIR_NAME;
use yp_core::ini::{IniDoc, IniLine};
use yp_core::{GamePaths, HostEnv, IoOp, IoResultExt, Os};

use crate::dll_id::{self, GraphicsDll};
use crate::error::Result;
use crate::flags;
use crate::model::{
    Classification, DllSummary, InstallOutcome, InstallReport, ReShadeConfig, ReShadeInfo, RuntimeStatus,
    ThirdPartyKind, ThirdPartyRuntime, ThirdPartyUpdateInfo,
};
use crate::paths::{self, base_name, DllHit, DIRECTX_PROXY_NAMES, RESHADE_INI_NAME};
use crate::runtime::GraphicsRuntime;
use crate::tree::{self, ExistingFiles};

pub const D3D_COMPILER_NAME: &str = "d3dcompiler_47.dll";
pub const PRESET_FILE_NAME: &str = "ReShadePreset.ini";
pub const DLL_NAME: &str = "ReShade64.dll";
pub const ADDON_DLL_NAME: &str = "ReShade64_Addon.dll";

pub const GAME_ROOT_DLLS: [&str; 5] = ["ReShade64.dll", "ReShade64_Addon.dll", "reshade64.dll", "dxgi.dll", "d3d11.dll"];
pub const RUNTIME_DLLS: [&str; 3] = ["dxgi.dll", "ReShade64_Addon.dll", "ReShade64.dll"];

pub const SHADER_SEARCH_PATH: &str = r".\reshade-shaders\Shaders\**";
pub const TEXTURE_SEARCH_PATH: &str = r".\reshade-shaders\Textures\**";
pub const ADDON_SEARCH_PATH: &str = r".\reshade-addons";
pub const CACHE_PATH: &str = r".\reshade-cache";
pub const DEFAULT_PRESET_PATH: &str = r".\reshade-presets\ReShadePreset.ini";

const GENERAL: &str = "GENERAL";
const EFFECT_SEARCH_PATHS: &str = "EffectSearchPaths";
const TEXTURE_SEARCH_PATHS: &str = "TextureSearchPaths";

const VERSION_NEEDLE: &[u8] = b"ReShade ";

const REPLACED_DLL_NAMES: [&str; 4] = ["reshade64.dll", "reshade64_addon.dll", "dxgi.dll", "d3d11.dll"];

const GAME_ROOT_COMPANIONS: [&str; 6] = [
    RESHADE_INI_NAME,
    PRESET_FILE_NAME,
    "reshade-shaders",
    "reshade-presets",
    "reshade-addons",
    D3D_COMPILER_NAME,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReShadeRuntime {
    host_os: Os,
}

impl ReShadeRuntime {
    pub fn new(host: &HostEnv) -> Self {
        Self::for_os(host.os())
    }

    pub const fn for_os(host_os: Os) -> Self {
        Self { host_os }
    }

    pub fn find_game_root_dll(&self, game: &GamePaths) -> Option<DllHit> {
        GAME_ROOT_DLLS.iter().find_map(|name| {
            let file = game.root().join(name);
            (file.is_file() && dll_id::identify_file(&file) == GraphicsDll::ReShade).then(|| DllHit {
                file,
                is_addon: name.to_lowercase().contains("_addon"),
            })
        })
    }

    pub fn apply_config(&self, game: &GamePaths, config: &ReShadeConfig) -> Result<()> {
        let ini = install_dir(game).join(RESHADE_INI_NAME);
        let Some(text) = yp_core::fs::read_text_opt(&ini)? else {
            return Ok(());
        };
        let mut doc = IniDoc::parse(&text);
        doc.set(Some(GENERAL), "PerformanceMode", bool_text(config.performance_mode));
        doc.set(Some("OVERLAY"), "ShowFPS", bool_text(config.show_fps));
        doc.set(Some("OVERLAY"), "ShowClock", bool_text(config.show_clock));
        std::fs::write(&ini, doc.to_string()).io(IoOp::Write, &ini)?;
        Ok(())
    }

    fn collect_info(&self, dir: &Path, dll_hit: Option<&DllHit>) -> ReShadeInfo {
        let mut presets: Vec<String> = paths::find_presets(&dir.join("reshade-presets"))
            .iter()
            .map(|preset| tree::file_stem(preset))
            .collect();
        presets.sort();

        let shaders_dir = dir.join("reshade-shaders").join("Shaders");
        let mut repos = BTreeSet::new();
        let mut shader_count = 0;
        for file in tree::files_where(&shaders_dir, |rel, _| rel.ends_with(".fx")) {
            shader_count += 1;
            if let Ok(rel) = file.strip_prefix(&shaders_dir) {
                let parts: Vec<_> = rel.components().collect();
                if parts.len() > 1 {
                    repos.insert(parts[0].as_os_str().to_string_lossy().into_owned());
                }
            }
        }

        let mut addons: Vec<String> = tree::files_where(&dir.join("reshade-addons"), |rel, _| paths::is_addon_name(rel))
            .iter()
            .map(|addon| tree::file_name(addon))
            .collect();
        addons.sort();

        let config = match yp_core::fs::read_text_opt(&dir.join(RESHADE_INI_NAME)) {
            Ok(Some(text)) => parse_config(&text),
            _ => ReShadeConfig::default(),
        };

        ReShadeInfo {
            presets,
            shader_repos: repos.into_iter().collect(),
            shader_count,
            addons,
            is_addon_build: dll_hit.is_some_and(|hit| hit.is_addon),
            version: dll_hit.and_then(|hit| read_version(&hit.file)),
            dll_name: dll_hit.map(|hit| tree::file_name(&hit.file)),
            d3d_compiler_missing: self.host_os == Os::Linux && !dir.join(D3D_COMPILER_NAME).is_file(),
            config,
        }
    }
}

impl GraphicsRuntime for ReShadeRuntime {
    fn runtime(&self) -> ThirdPartyRuntime {
        ThirdPartyRuntime::ReShade
    }

    fn install_dir(&self, game: &GamePaths) -> PathBuf {
        install_dir(game)
    }

    fn can_install(&self, classification: &Classification) -> bool {
        matches!(classification.kind, ThirdPartyKind::ReShadeWholeInstall | ThirdPartyKind::ReShadePreset)
    }

    fn install(&self, game: &GamePaths, classification: &Classification) -> Result<InstallOutcome> {
        let src = &classification.source_root;
        let dest = install_dir(game);
        std::fs::create_dir_all(&dest).io(IoOp::Create, &dest)?;
        paths::ensure_reshade_tree(&dest)?;

        place_dll(src, &dest)?;
        place_shaders(src, &dest);
        place_addons(src, &dest)?;
        place_presets(src, &dest)?;
        place_ini(src, &dest)?;
        place_d3d_compiler(src, &dest)?;
        ensure_ini_paths(&dest);

        flags::set_disabled(game, self.loading_flag(), false)?;

        Ok(InstallOutcome::Installed(InstallReport {
            runtime: ThirdPartyRuntime::ReShade,
            shaders_missing: !paths::has_reshade_shaders(&dest),
            preset_count: paths::find_presets(src).len(),
        }))
    }

    fn would_update(&self, game: &GamePaths, classification: &Classification) -> Option<ThirdPartyUpdateInfo> {
        let incoming = pick_dll(&classification.source_root)?;
        let installed = paths::find_graphics_dll(&install_dir(game), GraphicsDll::ReShade)?;
        if !tree::files_differ(&incoming.file, &installed.file) {
            return None;
        }
        Some(ThirdPartyUpdateInfo {
            runtime: ThirdPartyRuntime::ReShade,
            installed: dll_summary(&installed.file),
            incoming: dll_summary(&incoming.file),
        })
    }

    fn repair(&self, game: &GamePaths) -> Result<()> {
        let dest = install_dir(game);
        if !dest.is_dir() {
            return Ok(());
        }
        paths::ensure_reshade_tree(&dest)?;
        ensure_ini_paths(&dest);
        Ok(())
    }

    fn import_from_game_root(&self, game: &GamePaths) -> Result<bool> {
        let Some(hit) = self.find_game_root_dll(game) else {
            return Ok(false);
        };
        let game_dir = game.root();
        let dest = install_dir(game);
        std::fs::create_dir_all(&dest).io(IoOp::Create, &dest)?;
        paths::ensure_reshade_tree(&dest)?;

        tree::copy_file_as(&hit.file, &dest, if hit.is_addon { ADDON_DLL_NAME } else { DLL_NAME })?;

        for name in GAME_ROOT_COMPANIONS {
            let src = game_dir.join(name);
            if src.is_dir() {
                tree::merge_tree(&src, &dest.join(name), ExistingFiles::Keep);
            } else if src.is_file() {
                let into = if name == PRESET_FILE_NAME { dest.join("reshade-presets") } else { dest.clone() };
                tree::copy_file_into(&src, &into)?;
            }
        }

        for (key, sub) in [(EFFECT_SEARCH_PATHS, "Shaders"), (TEXTURE_SEARCH_PATHS, "Textures")] {
            for dir in referenced_search_dirs(game_dir, key) {
                tree::merge_tree(&dir, &dest.join("reshade-shaders").join(sub), ExistingFiles::Keep);
            }
        }

        for entry in tree::list_dir(game_dir) {
            if paths::is_addon_name(&entry.name().to_lowercase()) && entry.path.is_file() {
                tree::copy_file_into(&entry.path, &dest.join("reshade-addons"))?;
            }
        }

        let ini = dest.join(RESHADE_INI_NAME);
        if let Some(text) = yp_core::fs::read_text_opt(&ini)? {
            std::fs::write(&ini, fix_ini_paths(&text)).io(IoOp::Write, &ini)?;
        }
        ensure_ini_paths(&dest);

        flags::set_disabled(game, self.loading_flag(), false)?;
        Ok(true)
    }

    fn status(&self, game: &GamePaths) -> RuntimeStatus {
        let dir = install_dir(game);
        let dll_hit = paths::find_graphics_dll(&dir, GraphicsDll::ReShade);
        let installed = dir.is_dir()
            && (dll_hit.is_some() || paths::has_reshade_shaders(&dir) || dir.join(RESHADE_INI_NAME).is_file());
        RuntimeStatus {
            installed,
            enabled: self.is_enabled(game),
            shaders_missing: installed && !paths::has_reshade_shaders(&dir),
            preset_count: paths::find_presets(&dir.join("reshade-presets")).len(),
            reshade_info: installed.then(|| self.collect_info(&dir, dll_hit.as_ref())),
            ..RuntimeStatus::default()
        }
    }

    fn remove(&self, game: &GamePaths) -> Result<()> {
        yp_core::fs::remove_dir_quiet(&paths::reshade(game));
        flags::set_disabled(game, self.loading_flag(), true)
    }
}

pub fn install_dir(game: &GamePaths) -> PathBuf {
    let kind_dir = paths::reshade(game);
    paths::mod_dir_with_dll(&kind_dir, &RUNTIME_DLLS)
        .or_else(|| paths::mod_dirs(&kind_dir).into_iter().next())
        .unwrap_or_else(|| kind_dir.join(RESHADE_MOD_DIR_NAME))
}

pub fn shaders_dir(game: &GamePaths) -> PathBuf {
    install_dir(game).join("reshade-shaders").join("Shaders")
}

pub fn ensure_ini_paths(dest: &Path) {
    let ini = dest.join(RESHADE_INI_NAME);
    let before = match yp_core::fs::read_text_opt(&ini) {
        Ok(text) => text.unwrap_or_default(),
        Err(_) => return,
    };
    let mut doc = IniDoc::parse(&before);
    with_search_path(&mut doc, dest, EFFECT_SEARCH_PATHS, SHADER_SEARCH_PATH);
    with_search_path(&mut doc, dest, TEXTURE_SEARCH_PATHS, TEXTURE_SEARCH_PATH);
    let addon_path = doc.get(Some("ADDON"), "AddonPath").unwrap_or_default().to_owned();
    let addon_dir = resolve_relative(dest, &search_dir(&addon_path));
    if !has_addons_directly_in(&addon_dir) && has_addons_directly_in(&dest.join("reshade-addons")) {
        doc.set(Some("ADDON"), "AddonPath", ADDON_SEARCH_PATH);
    }
    let after = doc.to_string();
    if after != before {
        let _ = std::fs::write(&ini, after);
    }
}

fn with_search_path(doc: &mut IniDoc, dest: &Path, key: &str, wanted: &str) {
    let current = doc.get(Some(GENERAL), key).unwrap_or_default().to_owned();
    let wanted_dir = search_dir(wanted).to_lowercase();
    let mut entries = vec![wanted.to_owned()];
    for raw in current.split(',') {
        let entry = raw.trim();
        if entry.is_empty() || is_absolute(entry) {
            continue;
        }
        let dir = search_dir(entry);
        if dir.to_lowercase() == wanted_dir || !resolve_relative(dest, &dir).is_dir() {
            continue;
        }
        entries.push(entry.to_owned());
    }
    let value = entries.join(",");
    if current != value {
        doc.set(Some(GENERAL), key, &value);
    }
}

pub fn search_dir(entry: &str) -> String {
    let mut dir = entry.trim().replace('\\', "/");
    if let Some(stripped) = dir.strip_suffix("**") {
        dir = stripped.to_owned();
    }
    let mut dir = dir.trim_end_matches('/');
    while let Some(rest) = dir.strip_prefix("./") {
        dir = rest;
    }
    if dir == "." { String::new() } else { dir.to_owned() }
}

fn resolve_relative(base: &Path, dir: &str) -> PathBuf {
    let joined = dir.split('/').filter(|segment| !segment.is_empty()).fold(base.to_path_buf(), |path, segment| path.join(segment));
    tree::normalize_lexically(&joined)
}

fn has_addons_directly_in(dir: &Path) -> bool {
    tree::list_dir(dir).iter().any(|entry| paths::is_addon_name(&entry.name().to_lowercase()))
}

pub fn referenced_search_dirs(game_dir: &Path, key: &str) -> Vec<PathBuf> {
    let Ok(Some(text)) = yp_core::fs::read_text_opt(&game_dir.join(RESHADE_INI_NAME)) else {
        return Vec::new();
    };
    let doc = IniDoc::parse(&text);
    let game = tree::normalize_lexically(&std::path::absolute(game_dir).unwrap_or_else(|_| game_dir.to_path_buf()));
    let shipped_shaders = game.join("reshade-shaders");
    let mut dirs: Vec<PathBuf> = Vec::new();
    for raw in doc.get(Some(GENERAL), key).unwrap_or_default().split(',') {
        let entry = raw.trim();
        if entry.is_empty() {
            continue;
        }
        let rel = search_dir(entry);
        let dir = if is_absolute(entry) {
            tree::normalize_lexically(Path::new(&rel))
        } else {
            resolve_relative(&game, &rel)
        };
        if tree::same_path(&dir, &game) || tree::is_within(&dir, &game) {
            continue;
        }
        if tree::same_path(&shipped_shaders, &dir) || tree::is_within(&shipped_shaders, &dir) {
            continue;
        }
        if !dir.is_dir() || dirs.iter().any(|known| tree::same_path(known, &dir)) {
            continue;
        }
        dirs.push(dir);
    }
    dirs
}

pub fn parse_config(content: &str) -> ReShadeConfig {
    let doc = IniDoc::parse(content);
    let get = |section: &str, key: &str| doc.get(Some(section), key).map(str::to_owned);
    ReShadeConfig {
        performance_mode: bool_from(get(GENERAL, "PerformanceMode").as_deref()),
        skip_loading_check: bool_from(get(GENERAL, "NoReloadOnInitException").as_deref()),
        active_preset: get(GENERAL, "PresetPath").map(|preset| windows_file_stem(&preset)),
        overlay_key: get("INPUT", "KeyOverlay").unwrap_or_default(),
        effect_toggle_key: get("INPUT", "KeyEffects").unwrap_or_default(),
        screenshot_key: get("INPUT", "KeyScreenshot").unwrap_or_default(),
        screenshot_path: get("SCREENSHOT", "SavePath"),
        screenshot_format: get("SCREENSHOT", "FileFormat").and_then(|format| format.parse().ok()).unwrap_or(1),
        show_fps: bool_from(get("OVERLAY", "ShowFPS").as_deref()),
        show_clock: bool_from(get("OVERLAY", "ShowClock").as_deref()),
    }
}

fn bool_from(value: Option<&str>) -> bool {
    value.is_some_and(|value| matches!(value.trim().to_lowercase().as_str(), "1" | "true"))
}

fn bool_text(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn last_segment(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn windows_file_stem(path: &str) -> String {
    let name = last_segment(path);
    match name.rfind('.') {
        Some(dot) if dot > 0 => name[..dot].to_owned(),
        _ => name.to_owned(),
    }
}

pub fn read_version(dll: &Path) -> Option<String> {
    let bytes = std::fs::read(dll).ok()?;
    let limit = bytes.len().saturating_sub(VERSION_NEEDLE.len() + 12);
    (0..limit)
        .filter(|&start| bytes[start..].starts_with(VERSION_NEEDLE))
        .find_map(|start| {
            let digits: String = bytes[start + VERSION_NEEDLE.len()..]
                .iter()
                .take_while(|&&byte| byte.is_ascii_digit() || byte == b'.')
                .map(|&byte| char::from(byte))
                .collect();
            (!digits.is_empty()).then(|| format!("ReShade {digits}"))
        })
}

fn dll_summary(dll: &Path) -> DllSummary {
    DllSummary {
        file_name: tree::file_name(dll),
        version: read_version(dll),
        size_bytes: tree::file_size(dll),
    }
}

fn pick_dll(src: &Path) -> Option<DllHit> {
    paths::find_graphics_dll(src, GraphicsDll::ReShade).or_else(|| {
        tree::first_file_where(src, |rel, _| matches!(base_name(rel), "reshade64.dll" | "reshade64_addon.dll")).map(|file| {
            let is_addon = tree::file_name_lower(&file).contains("_addon");
            DllHit { file, is_addon }
        })
    })
}

fn place_dll(src: &Path, dest: &Path) -> Result<()> {
    let Some(dll) = pick_dll(src) else {
        return Ok(());
    };
    let target = if dll.is_addon { ADDON_DLL_NAME } else { DLL_NAME };
    for name in REPLACED_DLL_NAMES {
        if name != target.to_lowercase() {
            yp_core::fs::remove_file_quiet(&dest.join(name));
        }
    }
    tree::copy_file_as(&dll.file, dest, target)?;
    Ok(())
}

fn place_shaders(src: &Path, dest: &Path) {
    if let Some(shaders) = paths::find_dir_named(src, "reshade-shaders") {
        tree::merge_tree(&shaders, &dest.join("reshade-shaders"), ExistingFiles::Keep);
        return;
    }
    if let Some(bare) = paths::find_dir_named(src, "shaders") {
        tree::merge_tree(&bare, &dest.join("reshade-shaders").join("Shaders"), ExistingFiles::Keep);
    }
    if let Some(bare) = paths::find_dir_named(src, "textures") {
        tree::merge_tree(&bare, &dest.join("reshade-shaders").join("Textures"), ExistingFiles::Keep);
    }
}

fn place_addons(src: &Path, dest: &Path) -> Result<()> {
    if let Some(addons) = paths::find_dir_named(src, "reshade-addons") {
        tree::merge_tree(&addons, &dest.join("reshade-addons"), ExistingFiles::Keep);
    }
    let addon_files = tree::files_where(src, |rel, _| paths::is_addon_name(rel));
    for addon in &addon_files {
        tree::copy_file_into(addon, &dest.join("reshade-addons"))?;
    }
    for dll in addon_companion_dlls(&addon_files) {
        tree::copy_file_into(&dll, dest)?;
    }
    Ok(())
}

pub fn addon_companion_dlls(addon_files: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs: Vec<&Path> = Vec::new();
    for parent in addon_files.iter().filter_map(|file| file.parent()) {
        if !dirs.contains(&parent) {
            dirs.push(parent);
        }
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for dir in dirs {
        for entry in tree::list_dir(dir) {
            let name = entry.name().to_lowercase();
            if !name.ends_with(".dll") || !entry.path.is_file() {
                continue;
            }
            if name == D3D_COMPILER_NAME || DIRECTX_PROXY_NAMES.contains(&name.as_str()) {
                continue;
            }
            if dll_id::identify_file(&entry.path) != GraphicsDll::Other {
                continue;
            }
            if seen.insert(name) {
                out.push(entry.path);
            }
        }
    }
    out
}

fn place_presets(src: &Path, dest: &Path) -> Result<()> {
    let presets_dir = dest.join("reshade-presets");
    std::fs::create_dir_all(&presets_dir).io(IoOp::Create, &presets_dir)?;
    for preset in paths::find_presets(src) {
        let name = paths::unique_file_name(&presets_dir, &tree::file_name(&preset));
        tree::copy_file_as(&preset, &presets_dir, &name)?;
    }
    Ok(())
}

fn place_ini(src: &Path, dest: &Path) -> Result<()> {
    let target = dest.join(RESHADE_INI_NAME);
    if target.is_file() {
        return Ok(());
    }
    let wanted = RESHADE_INI_NAME.to_lowercase();
    let Some(pack_ini) = tree::first_file_where(src, |rel, _| base_name(rel) == wanted) else {
        return Ok(());
    };
    let text = yp_core::fs::read_text_opt(&pack_ini)?.unwrap_or_default();
    std::fs::write(&target, fix_ini_paths(&text)).io(IoOp::Write, &target)?;
    Ok(())
}

fn place_d3d_compiler(src: &Path, dest: &Path) -> Result<()> {
    if dest.join(D3D_COMPILER_NAME).is_file() {
        return Ok(());
    }
    if let Some(dll) = tree::first_file_where(src, |rel, _| base_name(rel) == D3D_COMPILER_NAME) {
        tree::copy_file_as(&dll, dest, D3D_COMPILER_NAME)?;
    }
    Ok(())
}

pub fn fix_ini_paths(content: &str) -> String {
    let doc = IniDoc::parse(content);
    let newline = doc.newline();
    let mut out = String::with_capacity(content.len() + 64);
    if content.starts_with('\u{feff}') {
        out.push('\u{feff}');
    }
    for line in doc.lines() {
        match line {
            IniLine::Entry { key, value, .. } => match fixed_entry(key, value) {
                Some(fixed) => out.push_str(&fixed),
                None => out.push_str(line.raw()),
            },
            other => out.push_str(other.raw()),
        }
        out.push_str(newline);
    }
    out
}

fn fixed_entry(key: &str, value: &str) -> Option<String> {
    let fixed = match key.to_lowercase().as_str() {
        "effectsearchpaths" => format!("{EFFECT_SEARCH_PATHS}={}", fix_search(value, "Shaders")),
        "texturesearchpaths" => format!("{TEXTURE_SEARCH_PATHS}={}", fix_search(value, "Textures")),
        "intermediatecachepath" => format!("IntermediateCachePath={CACHE_PATH}"),
        "presetpath" | "currentpresetpath" => format!("PresetPath={}", fix_preset(value)),
        "startuppresetpath" => format!("StartupPresetPath={}", fix_preset(value)),
        _ if is_absolute(value.trim()) => format!("{key}="),
        _ => return None,
    };
    Some(fixed)
}

pub fn is_absolute(path: &str) -> bool {
    if path.starts_with('/') || path.starts_with('\\') {
        return true;
    }
    let bytes = path.as_bytes();
    bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn fix_search(value: &str, kind: &str) -> String {
    let mut cleaned: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty() && !is_absolute(part))
        .collect();
    let fallback = format!(r".\reshade-shaders\{kind}\**");
    if cleaned.is_empty() {
        cleaned.push(&fallback);
    }
    cleaned.join(",")
}

fn fix_preset(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() || is_absolute(value) {
        DEFAULT_PRESET_PATH.to_owned()
    } else {
        format!(r".\reshade-presets\{}", last_segment(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_dirs_are_normalised() {
        assert_eq!(search_dir(r".\reshade-shaders\Shaders\**"), "reshade-shaders/Shaders");
        assert_eq!(search_dir(r".\"), "");
        assert_eq!(search_dir("."), "");
        assert_eq!(search_dir(r"..\"), "..");
        assert_eq!(search_dir(r".\.\my-fx\**"), "my-fx");
    }

    #[test]
    fn absolute_paths() {
        assert!(is_absolute(r"C:\Games"));
        assert!(is_absolute("D:/x"));
        assert!(is_absolute("/home/u"));
        assert!(is_absolute(r"\\server\share"));
        assert!(!is_absolute(r".\reshade-shaders"));
        assert!(!is_absolute("C:"));
        assert!(!is_absolute(""));
    }

    #[test]
    fn preset_and_search_fixes() {
        assert_eq!(fix_preset(r"C:\Users\me\NieR.ini"), DEFAULT_PRESET_PATH);
        assert_eq!(fix_preset(r".\presets\Cinematic.ini"), r".\reshade-presets\Cinematic.ini");
        assert_eq!(fix_preset("   "), DEFAULT_PRESET_PATH);
        assert_eq!(fix_search(r"C:\a,.\b\**, ,", "Shaders"), r".\b\**");
        assert_eq!(fix_search(r"C:\only", "Textures"), r".\reshade-shaders\Textures\**");
    }

    #[test]
    fn version_is_read_from_the_binary() {
        let dir = tempfile::tempdir().unwrap();
        let dll = dir.path().join("ReShade64.dll");
        let mut bytes = b"MZ....ReShade 6.1.0\0".to_vec();
        bytes.extend_from_slice(&[0; 32]);
        std::fs::write(&dll, &bytes).unwrap();
        assert_eq!(read_version(&dll).as_deref(), Some("ReShade 6.1.0"));
        std::fs::write(&dll, b"ReShade x").unwrap();
        assert_eq!(read_version(&dll), None);
    }

    #[test]
    fn preset_stem_handles_windows_separators() {
        assert_eq!(windows_file_stem(r".\reshade-presets\Cinematic.ini"), "Cinematic");
        assert_eq!(windows_file_stem("a/b/c.ini"), "c");
        assert_eq!(windows_file_stem("plain"), "plain");
    }
}
