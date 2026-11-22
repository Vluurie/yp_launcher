use std::time::SystemTime;

use yp_core::wine_paths::{posix, NIER_STEAM_APP_ID};

use super::{is_file, subdir_names, WineContext};

pub const COMPAT_DATA_VAR: &str = "STEAM_COMPAT_DATA_PATH";
pub const CLIENT_INSTALL_VAR: &str = "STEAM_COMPAT_CLIENT_INSTALL_PATH";
const STEAM_HOME_SEARCH_DEPTH: usize = 4;

pub fn is_proton_runtime_path(runtime_path: &str) -> bool {
    posix::basename(runtime_path).eq_ignore_ascii_case("proton")
}

pub fn compat_data_path(ctx: &WineContext, game_path: Option<&str>) -> String {
    if let Some(from_env) = ctx.var(COMPAT_DATA_VAR) {
        return from_env.to_owned();
    }
    if let Some(context) = game_path.and_then(|path| ctx.infer_steam_context(path)) {
        return context.compat_data;
    }
    posix::join_all(&ctx.home(), &[".steam", "steam", "steamapps", "compatdata", NIER_STEAM_APP_ID])
}

pub fn steam_root(ctx: &WineContext, game_path: Option<&str>) -> String {
    if let Some(from_env) = ctx.var(CLIENT_INSTALL_VAR) {
        return from_env.to_owned();
    }
    if let Some(context) = game_path.and_then(|path| ctx.infer_steam_context(path)) {
        return context.steam_root;
    }
    posix::join_all(&ctx.home(), &[".steam", "steam"])
}

pub fn find_proton_path(steam_root: &str, library_root: &str) -> Option<String> {
    [
        posix::join(steam_root, "compatibilitytools.d"),
        posix::join_all(steam_root, &["steamapps", "common"]),
        posix::join_all(library_root, &["steamapps", "common"]),
    ]
    .iter()
    .find_map(|parent| proton_candidates(parent).into_iter().next())
}

pub fn find_any_proton_path(ctx: &WineContext) -> Option<String> {
    ctx.steam_roots().iter().find_map(|root| find_proton_path(root, root))
}

pub fn has_any_proton(ctx: &WineContext) -> bool {
    find_any_proton_path(ctx).is_some()
}

pub fn steam_home_for_client(ctx: &WineContext, steam_client_install_path: &str) -> Option<String> {
    if has_steam_client(&ctx.home()) {
        return None;
    }
    steam_home_candidates(steam_client_install_path)
        .into_iter()
        .find(|candidate| has_steam_client(candidate))
}

fn has_steam_client(home: &str) -> bool {
    !home.is_empty() && is_file(&posix::join_all(home, &[".steam", "sdk64", "steamclient.so"]))
}

fn steam_home_candidates(steam_root: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut dir = steam_root.to_owned();
    for _ in 0..STEAM_HOME_SEARCH_DEPTH {
        let parent = posix::dirname(&dir);
        if parent == dir {
            break;
        }
        candidates.push(parent.clone());
        dir = parent;
    }
    candidates
}

pub fn proton_candidates(parent_dir: &str) -> Vec<String> {
    let mut builds: Vec<(SystemTime, String, String)> = subdir_names(parent_dir)
        .into_iter()
        .filter(|name| name.to_lowercase().contains("proton"))
        .filter_map(|name| {
            let dir = posix::join(parent_dir, &name);
            let proton = posix::join(&dir, "proton");
            if !is_file(&proton) {
                return None;
            }
            let modified = std::fs::metadata(&dir)
                .and_then(|meta| meta.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            Some((modified, name, proton))
        })
        .collect();
    builds.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    builds.into_iter().map(|(_, _, proton)| proton).collect()
}
