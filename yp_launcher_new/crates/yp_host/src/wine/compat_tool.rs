use yp_core::wine_paths::{posix, NIER_STEAM_APP_ID};

use super::native_steam::native_steam_libraries;
use super::{is_file, push_unique, subdir_names, WineContext, EXTRA_COMPAT_TOOLS_VAR};
use crate::vdf::{self, VdfObject};

pub const DEFAULT_TOOL_KEY: &str = "0";
const PROTON_PREFIX: &str = "proton";

pub fn find_steam_configured_proton(ctx: &WineContext, steam_root: &str, library_root: &str) -> Option<String> {
    let text = std::fs::read_to_string(posix::join_all(steam_root, &["config", "config.vdf"])).ok()?;
    let config = vdf::parse(&text).ok()?;
    [NIER_STEAM_APP_ID, DEFAULT_TOOL_KEY].iter().find_map(|app_id| {
        let name = steam_compat_tool_name(&config, app_id)?;
        resolve_compat_tool(ctx, &name, steam_root, &[library_root.to_owned()])
    })
}

pub fn steam_compat_tool_name(config: &VdfObject, app_id: &str) -> Option<String> {
    config
        .find_block("CompatToolMapping")?
        .block(app_id)?
        .string("name")
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

pub fn resolve_compat_tool(ctx: &WineContext, name: &str, steam_root: &str, library_roots: &[String]) -> Option<String> {
    let wanted = name.to_lowercase();
    if let Some(proton) = compat_tool_dirs(ctx, steam_root)
        .iter()
        .find_map(|tools_dir| custom_tool(tools_dir, &wanted))
    {
        return Some(proton);
    }
    let mut libraries = vec![steam_root.to_owned()];
    for library in library_roots.iter().cloned().chain(native_steam_libraries(ctx)) {
        push_unique(&mut libraries, library);
    }
    libraries
        .iter()
        .find_map(|library| official_tool(&posix::join_all(library, &["steamapps", "common"]), &wanted))
}

pub fn official_proton_tool_name(dir_name: &str) -> Option<String> {
    let lower = dir_name.to_lowercase();
    if !lower.starts_with(PROTON_PREFIX) {
        return None;
    }
    if lower.contains("experimental") {
        return Some("proton_experimental".to_owned());
    }
    if lower.contains("hotfix") {
        return Some("proton_hotfix".to_owned());
    }
    lower.split(' ').find_map(|word| {
        let (major, minor) = word.split_once('.')?;
        if minor.contains('.') {
            return None;
        }
        let major_number: u32 = major.parse().ok()?;
        let minor_number: u32 = minor.parse().ok()?;
        Some(if minor_number == 0 {
            format!("proton_{major_number}")
        } else {
            format!("proton_{major}{minor}")
        })
    })
}

fn compat_tool_dirs(ctx: &WineContext, steam_root: &str) -> Vec<String> {
    let mut dirs = vec![posix::join(steam_root, "compatibilitytools.d")];
    if let Some(extra) = ctx.var(EXTRA_COMPAT_TOOLS_VAR) {
        dirs.extend(extra.split(':').filter(|dir| !dir.is_empty()).map(str::to_owned));
    }
    dirs.extend(ctx.system_compat_tool_dirs().iter().cloned());
    dirs
}

fn custom_tool(tools_dir: &str, wanted: &str) -> Option<String> {
    subdir_names(tools_dir).into_iter().find_map(|name| {
        let dir = posix::join(tools_dir, &name);
        let text = std::fs::read_to_string(posix::join(&dir, "compatibilitytool.vdf")).ok()?;
        let manifest = vdf::parse(&text).ok()?;
        let tools = manifest.find_block("compat_tools")?;
        tools
            .blocks()
            .filter(|(key, _)| key.to_lowercase() == wanted)
            .find_map(|(_, tool)| {
                let install_path = tool.string("install_path").unwrap_or(".");
                let proton = posix::normalize(&posix::join_all(&dir, &[install_path, "proton"]));
                is_file(&proton).then_some(proton)
            })
    })
}

fn official_tool(common_dir: &str, wanted: &str) -> Option<String> {
    subdir_names(common_dir).into_iter().find_map(|name| {
        if official_proton_tool_name(&name).as_deref() != Some(wanted) {
            return None;
        }
        let proton = posix::join_all(common_dir, &[&name, "proton"]);
        is_file(&proton).then_some(proton)
    })
}
