use yp_core::consts::GAME_EXE_NAME;
use yp_core::wine_paths::posix;
use yp_core::NierInstallation;

use super::{installation, is_file, push_unique, WineContext, NIER_DIR_NAME};
use crate::vdf;

pub fn native_steam_libraries(ctx: &WineContext) -> Vec<String> {
    let mut libraries = Vec::new();
    for root in ctx.steam_roots() {
        push_unique(&mut libraries, root.clone());
        let Ok(text) = std::fs::read_to_string(posix::join_all(&root, &["steamapps", "libraryfolders.vdf"])) else {
            continue;
        };
        for path in vdf::path_entries(&text) {
            push_unique(&mut libraries, path);
        }
    }
    libraries
}

pub fn find_nier_in_native_steam(ctx: &WineContext) -> Vec<NierInstallation> {
    let mut seen = Vec::new();
    let mut installations = Vec::new();
    for library in native_steam_libraries(ctx) {
        let dir = posix::join_all(&library, &["steamapps", "common", NIER_DIR_NAME]);
        if !is_file(&posix::join(&dir, GAME_EXE_NAME)) {
            continue;
        }
        let normalized = posix::normalize(&dir);
        let key = normalized.to_lowercase();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        installations.push(installation(&normalized));
    }
    installations
}
