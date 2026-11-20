pub mod shaderfix_names;

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use yp_core::consts::{MIGOTO_MOD_DIR_NAME, NAMS_EXE_NAME};
use yp_core::ini::IniDoc;
use yp_core::{GamePaths, HostEnv, IoOp, IoResultExt};

use crate::dll_id::{self, GraphicsDll};
use crate::error::Result;
use crate::flags;
use crate::model::{
    Classification, DllSummary, InstallOutcome, MigotoConfig, MigotoHunting, MigotoInfo, MigotoMarking,
    RuntimeStatus, ThirdPartyKind, ThirdPartyRuntime, ThirdPartyUpdateInfo,
};
use crate::paths::{self, base_name, DllHit, MIGOTO_INI_NAME};
use crate::runtime::GraphicsRuntime;
use crate::tree::{self, ExistingFiles};

pub const DLL_NAME: &str = "3dmigoto.dll";
pub const PROXY_DLL_NAME: &str = "d3d11.dll";
pub const NVAPI_DLL_NAME: &str = "nvapi64.dll";
pub const COMPILER_DLL_NAME: &str = "d3dcompiler_46.dll";
pub const SHADER_FIXES_DIR_NAME: &str = "ShaderFixes";
pub const PARKED_SUFFIX: &str = ".disabled-on-wine";

pub const SUPPORT_DLLS: [&str; 2] = [NVAPI_DLL_NAME, COMPILER_DLL_NAME];
pub const RUNTIME_DLLS: [&str; 2] = [DLL_NAME, PROXY_DLL_NAME];

pub const LOADER_SECTION: &str = "Loader";
pub const LOADER_TARGET_KEY: &str = "target";
pub const SYSTEM_SECTION: &str = "System";

pub const NAMS_COMPAT_SYSTEM_KEYS: [(&str, &str); 5] = [
    ("hook", "recommended"),
    ("load_library_redirect", "1"),
    ("allow_check_interface", "1"),
    ("allow_create_device", "1"),
    ("allow_platform_update", "1"),
];

const STATUS_FILES: [&str; 5] = [PROXY_DLL_NAME, DLL_NAME, NVAPI_DLL_NAME, COMPILER_DLL_NAME, MIGOTO_INI_NAME];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigotoRuntime {
    loader_target: String,
    parks_nvapi: bool,
}

impl MigotoRuntime {
    pub fn new(host: &HostEnv) -> Self {
        Self::for_wine(host.runs_game_through_wine())
    }

    pub fn for_wine(runs_game_through_wine: bool) -> Self {
        Self {
            loader_target: NAMS_EXE_NAME.to_owned(),
            parks_nvapi: runs_game_through_wine,
        }
    }

    pub fn with_loader_target(mut self, target: impl Into<String>) -> Self {
        self.loader_target = target.into();
        self
    }

    pub fn loader_target(&self) -> &str {
        &self.loader_target
    }

    pub fn parks_nvapi(&self) -> bool {
        self.parks_nvapi
    }

    pub fn find_game_root_dll(&self, game: &GamePaths) -> Option<DllHit> {
        [PROXY_DLL_NAME, DLL_NAME].iter().find_map(|name| {
            let file = game.root().join(name);
            (file.is_file() && dll_id::identify_file(&file) == GraphicsDll::Migoto).then_some(DllHit {
                file,
                is_addon: false,
            })
        })
    }

    pub fn repair_in(&self, dest: &Path) -> Result<()> {
        if !dest.is_dir() {
            return Ok(());
        }
        self.patch_d3dx(dest)?;
        let proxy = dest.join(PROXY_DLL_NAME);
        let renamed = dest.join(DLL_NAME);
        if proxy.is_file() && !renamed.is_file() {
            std::fs::rename(&proxy, &renamed).io(IoOp::Rename, &proxy)?;
        }
        self.park_wine_incompatible_dlls(dest);
        Ok(())
    }

    pub fn apply_config(&self, game: &GamePaths, config: &MigotoConfig) -> Result<()> {
        let ini = install_dir(game).join(MIGOTO_INI_NAME);
        let Some(text) = yp_core::fs::read_text_opt(&ini)? else {
            return Ok(());
        };
        let mut doc = IniDoc::parse(&text);
        doc.set(Some("Hunting"), "hunting", hunting_text(config.hunting));
        doc.set(Some("Hunting"), "marking_mode", marking_text(config.marking_mode));
        doc.set(Some("Logging"), "verbose_overlay", bool_text(config.verbose_overlay));
        doc.set(Some("Rendering"), "cache_shaders", bool_text(config.cache_shaders));
        doc.set(Some("Device"), "check_foreground_window", bool_text(config.check_foreground_window));
        std::fs::write(&ini, doc.to_string()).io(IoOp::Write, &ini)?;
        Ok(())
    }

    fn patch_d3dx(&self, dest: &Path) -> Result<()> {
        let ini = dest.join(MIGOTO_INI_NAME);
        let current = yp_core::fs::read_text_opt(&ini)?.unwrap_or_default();
        let patched = patch_nams_compat(&patch_loader_target(&current, &self.loader_target));
        std::fs::write(&ini, patched).io(IoOp::Write, &ini)?;
        Ok(())
    }

    fn is_wine_incompatible(&self, name: &str) -> bool {
        name == NVAPI_DLL_NAME && self.parks_nvapi
    }

    fn park_wine_incompatible_dlls(&self, dest: &Path) {
        if !self.parks_nvapi {
            return;
        }
        for name in SUPPORT_DLLS.iter().filter(|name| self.is_wine_incompatible(name)) {
            let file = dest.join(name);
            if !file.is_file() {
                continue;
            }
            if std::fs::rename(&file, dest.join(format!("{name}{PARKED_SUFFIX}"))).is_err() {
                yp_core::fs::remove_file_quiet(&file);
            }
        }
    }

    fn place_support_dlls(&self, src: &Path, dest: &Path) -> Result<()> {
        for name in SUPPORT_DLLS {
            if self.is_wine_incompatible(name) {
                continue;
            }
            if let Some(found) = tree::first_file_where(src, |rel, _| base_name(rel) == name) {
                tree::copy_file_as(&found, dest, name)?;
            }
        }
        self.park_wine_incompatible_dlls(dest);
        Ok(())
    }

    fn write_d3dx(&self, src: &Path, dest: &Path) -> Result<()> {
        let target = dest.join(MIGOTO_INI_NAME);
        if !target.is_file()
            && let Some(pack_ini) = tree::first_file_where(src, |rel, _| base_name(rel) == MIGOTO_INI_NAME)
        {
            let text = yp_core::fs::read_text_opt(&pack_ini)?.unwrap_or_default();
            std::fs::write(&target, text).io(IoOp::Write, &target)?;
        }
        self.patch_d3dx(dest)
    }

    fn collect_info(&self, dir: &Path) -> MigotoInfo {
        let files = STATUS_FILES
            .iter()
            .filter(|name| dir.join(name).is_file())
            .map(|name| (*name).to_owned())
            .collect();

        let fix_files = shader_fix_files(&dir.join(SHADER_FIXES_DIR_NAME));
        let existing: HashSet<String> = fix_files.iter().cloned().collect();
        shaderfix_names::prune(dir, &existing);
        let name_map = shaderfix_names::read(dir);
        let mod_names: BTreeSet<String> =
            fix_files.iter().map(|file| name_map.get(file).cloned().unwrap_or_else(|| file.clone())).collect();

        let (loader_target, config) = match yp_core::fs::read_text_opt(&dir.join(MIGOTO_INI_NAME)) {
            Ok(Some(text)) => {
                let doc = IniDoc::parse(&text);
                (doc.get(Some(LOADER_SECTION), LOADER_TARGET_KEY).map(str::to_owned), parse_config_doc(&doc))
            }
            _ => (None, MigotoConfig::default()),
        };
        let loader_target_ok = loader_target
            .as_deref()
            .is_some_and(|target| target.to_lowercase().ends_with(&self.loader_target.to_lowercase()));

        MigotoInfo {
            files,
            shader_fix_count: fix_files.len(),
            shader_fix_names: mod_names.into_iter().collect(),
            loader_target,
            loader_target_ok,
            config,
        }
    }
}

impl GraphicsRuntime for MigotoRuntime {
    fn runtime(&self) -> ThirdPartyRuntime {
        ThirdPartyRuntime::Migoto
    }

    fn install_dir(&self, game: &GamePaths) -> PathBuf {
        install_dir(game)
    }

    fn can_install(&self, classification: &Classification) -> bool {
        classification.kind == ThirdPartyKind::Migoto
    }

    fn install(&self, game: &GamePaths, classification: &Classification) -> Result<InstallOutcome> {
        let src = &classification.source_root;
        let dest = install_dir(game);
        std::fs::create_dir_all(&dest).io(IoOp::Create, &dest)?;

        place_dll(src, &dest)?;
        self.place_support_dlls(src, &dest)?;
        let added = merge_shader_fixes(src, &dest);
        self.write_d3dx(src, &dest)?;

        if let Some(mod_name) = classification.source_name.as_deref().filter(|name| !name.is_empty())
            && !added.is_empty()
        {
            shaderfix_names::assign(&dest, &added, mod_name);
        }

        flags::set_disabled(game, self.loading_flag(), false)?;
        Ok(InstallOutcome::installed(ThirdPartyRuntime::Migoto))
    }

    fn would_update(&self, game: &GamePaths, classification: &Classification) -> Option<ThirdPartyUpdateInfo> {
        let incoming = paths::find_graphics_dll(&classification.source_root, GraphicsDll::Migoto)?;
        let dir = install_dir(game);
        let installed = [DLL_NAME, PROXY_DLL_NAME].iter().map(|name| dir.join(name)).find(|path| path.is_file())?;
        if !tree::files_differ(&incoming.file, &installed) {
            return None;
        }
        Some(ThirdPartyUpdateInfo {
            runtime: ThirdPartyRuntime::Migoto,
            installed: size_summary(&installed),
            incoming: size_summary(&incoming.file),
        })
    }

    fn repair(&self, game: &GamePaths) -> Result<()> {
        self.repair_in(&install_dir(game))
    }

    fn import_from_game_root(&self, game: &GamePaths) -> Result<bool> {
        let Some(hit) = self.find_game_root_dll(game) else {
            return Ok(false);
        };
        let dest = install_dir(game);
        std::fs::create_dir_all(&dest).io(IoOp::Create, &dest)?;

        tree::copy_file_as(&hit.file, &dest, DLL_NAME)?;
        yp_core::fs::remove_file_quiet(&dest.join(PROXY_DLL_NAME));
        for name in SUPPORT_DLLS.into_iter().chain([MIGOTO_INI_NAME]) {
            let file = game.root().join(name);
            if file.is_file() {
                tree::copy_file_as(&file, &dest, name)?;
            }
        }
        let shader_fixes = game.root().join(SHADER_FIXES_DIR_NAME);
        if shader_fixes.is_dir() {
            tree::merge_tree(&shader_fixes, &dest.join(SHADER_FIXES_DIR_NAME), ExistingFiles::Keep);
        }

        self.patch_d3dx(&dest)?;
        flags::set_disabled(game, self.loading_flag(), false)?;
        Ok(true)
    }

    fn status(&self, game: &GamePaths) -> RuntimeStatus {
        let dir = install_dir(game);
        let installed = dir.is_dir() && RUNTIME_DLLS.iter().any(|name| dir.join(name).is_file());
        RuntimeStatus {
            installed,
            enabled: self.is_enabled(game),
            has_shader_fixes: dir.join(SHADER_FIXES_DIR_NAME).is_dir(),
            migoto_info: installed.then(|| self.collect_info(&dir)),
            ..RuntimeStatus::default()
        }
    }

    fn remove(&self, game: &GamePaths) -> Result<()> {
        yp_core::fs::remove_dir_quiet(&paths::migoto(game));
        flags::set_disabled(game, self.loading_flag(), true)
    }
}

pub fn install_dir(game: &GamePaths) -> PathBuf {
    let kind_dir = paths::migoto(game);
    paths::mod_dir_with_dll(&kind_dir, &RUNTIME_DLLS)
        .or_else(|| paths::mod_dirs(&kind_dir).into_iter().next())
        .unwrap_or_else(|| kind_dir.join(MIGOTO_MOD_DIR_NAME))
}

pub fn patch_nams_compat(content: &str) -> String {
    let mut doc = IniDoc::parse(content);
    for (key, value) in NAMS_COMPAT_SYSTEM_KEYS {
        doc.set(Some(SYSTEM_SECTION), key, value);
    }
    doc.to_string()
}

pub fn patch_loader_target(content: &str, target: &str) -> String {
    if content.trim().is_empty() {
        return format!("[{LOADER_SECTION}]\n{LOADER_TARGET_KEY} = {target}\n");
    }
    let mut doc = IniDoc::parse(content);
    let has_loader = doc.sections().iter().any(|name| name.trim().eq_ignore_ascii_case(LOADER_SECTION));
    if !has_loader {
        return format!("[{LOADER_SECTION}]\n{LOADER_TARGET_KEY} = {target}\n\n{content}");
    }
    doc.set(Some(LOADER_SECTION), LOADER_TARGET_KEY, target);
    doc.to_string()
}

pub fn parse_config(content: &str) -> MigotoConfig {
    parse_config_doc(&IniDoc::parse(content))
}

fn parse_config_doc(doc: &IniDoc) -> MigotoConfig {
    let get = |section: &str, key: &str| doc.get(Some(section), key);
    MigotoConfig {
        hunting: match get("Hunting", "hunting").map(str::trim) {
            Some("1") => MigotoHunting::On,
            Some("2") => MigotoHunting::NoMarking,
            _ => MigotoHunting::Off,
        },
        marking_mode: match get("Hunting", "marking_mode").map(|value| value.trim().to_lowercase()).as_deref() {
            Some("original") => MigotoMarking::Original,
            Some("pink") => MigotoMarking::Pink,
            Some("mono") => MigotoMarking::Mono,
            _ => MigotoMarking::Skip,
        },
        verbose_overlay: bool_from(get("Logging", "verbose_overlay"), false),
        cache_shaders: bool_from(get("Rendering", "cache_shaders"), true),
        check_foreground_window: bool_from(get("Device", "check_foreground_window"), false),
        reload_fixes_key: get("Hunting", "reload_fixes").unwrap_or_default().to_owned(),
        wipe_cache_key: get("Hunting", "wipe_user_config").unwrap_or_default().to_owned(),
        toggle_hunt_key: get("Hunting", "toggle_hunting").unwrap_or_default().to_owned(),
    }
}

fn bool_from(value: Option<&str>, or_else: bool) -> bool {
    match value.map(|value| value.trim().to_lowercase()) {
        None => or_else,
        Some(value) if value.is_empty() => or_else,
        Some(value) => value == "1" || value == "true",
    }
}

fn bool_text(value: bool) -> &'static str {
    if value { "1" } else { "0" }
}

fn hunting_text(hunting: MigotoHunting) -> &'static str {
    match hunting {
        MigotoHunting::On => "1",
        MigotoHunting::NoMarking => "2",
        MigotoHunting::Off => "0",
    }
}

fn marking_text(marking: MigotoMarking) -> &'static str {
    match marking {
        MigotoMarking::Original => "original",
        MigotoMarking::Pink => "pink",
        MigotoMarking::Mono => "mono",
        MigotoMarking::Skip => "skip",
    }
}

fn size_summary(dll: &Path) -> DllSummary {
    DllSummary {
        file_name: tree::file_name(dll),
        version: None,
        size_bytes: tree::file_size(dll),
    }
}

fn place_dll(src: &Path, dest: &Path) -> Result<()> {
    let Some(dll) = paths::find_graphics_dll(src, GraphicsDll::Migoto) else {
        return Ok(());
    };
    tree::copy_file_as(&dll.file, dest, DLL_NAME)?;
    yp_core::fs::remove_file_quiet(&dest.join(PROXY_DLL_NAME));
    Ok(())
}

fn is_shader_fix(name_lower: &str) -> bool {
    name_lower.ends_with(".txt") || name_lower.ends_with(".hlsl")
}

fn shader_fix_files(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    tree::walk_entities(dir, &mut |entry| {
        if entry.is_file() && is_shader_fix(&entry.name().to_lowercase()) {
            names.push(entry.name());
        }
        true
    });
    names
}

fn merge_shader_fixes(src: &Path, dest: &Path) -> Vec<String> {
    let Some(shader_fixes) = paths::find_dir_named(src, "shaderfixes") else {
        return Vec::new();
    };
    tree::merge_tree(&shader_fixes, &dest.join(SHADER_FIXES_DIR_NAME), ExistingFiles::Keep);
    shader_fix_files(&shader_fixes)
}
