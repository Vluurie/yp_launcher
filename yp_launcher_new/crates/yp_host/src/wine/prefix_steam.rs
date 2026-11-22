use yp_core::consts::GAME_EXE_NAME;
use yp_core::wine_paths::{posix, to_posix};
use yp_core::NierInstallation;

use super::crossover::{list_bottles, nier_candidates_in_bottle, PROGRAM_FILES_VARIANTS};
use super::{installation, is_dir, is_file, push_unique, WineContext, NIER_DIR_NAME};
use crate::vdf;

const VERBATIM_PREFIX: &str = r"\\?\";

pub fn windows_drive_letter(path: &str) -> Option<char> {
    let mut chars = path.chars();
    let letter = chars.next().filter(char::is_ascii_alphabetic)?;
    (chars.next() == Some(':') && chars.next() == Some('\\')).then(|| letter.to_ascii_lowercase())
}

pub fn normalize_steam_library_path(library_path: &str, prefix: &str) -> String {
    let Some(drive) = windows_drive_letter(library_path) else {
        return library_path.to_owned();
    };
    let segments: Vec<&str> = library_path[3..].split('\\').filter(|part| !part.is_empty()).collect();
    let drive_root = if drive == 'c' {
        posix::join(prefix, "drive_c")
    } else {
        posix::join_all(prefix, &["dosdevices", &format!("{drive}:")])
    };
    posix::join_all(&drive_root, &segments)
}

pub fn parse_library_folders_in_prefix(prefix: &str) -> Vec<String> {
    let mut libraries = Vec::new();
    for program_files in PROGRAM_FILES_VARIANTS {
        let file = posix::join_all(prefix, &["drive_c", program_files, "Steam", "steamapps", "libraryfolders.vdf"]);
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        libraries.extend(
            vdf::path_entries(&text)
                .iter()
                .map(|path| normalize_steam_library_path(path, prefix)),
        );
    }
    libraries
}

pub fn nier_dirs_in_prefix(prefix: &str) -> Vec<String> {
    let mut dirs = Vec::new();
    for exe in nier_candidates_in_bottle(prefix) {
        if is_file(&exe) {
            push_unique(&mut dirs, posix::normalize(&posix::dirname(&exe)));
        }
    }
    for library in parse_library_folders_in_prefix(prefix) {
        let dir = posix::join_all(&library, &["steamapps", "common", NIER_DIR_NAME]);
        if is_file(&posix::join(&dir, GAME_EXE_NAME)) {
            push_unique(&mut dirs, posix::normalize(&dir));
        }
    }
    dirs
}

pub fn find_nier_in_prefixes(ctx: &WineContext) -> Vec<NierInstallation> {
    let mut seen = Vec::new();
    let mut installations = Vec::new();
    for bottle in list_bottles(ctx) {
        for dir in nier_dirs_in_prefix(&bottle.path) {
            let normalized = posix::normalize(&dir);
            let key = normalized.to_lowercase();
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            installations.push(installation(&normalized));
        }
    }
    installations
}

pub fn deep_scan_roots(ctx: &WineContext) -> Vec<String> {
    let mut roots = Vec::new();
    for bottle in list_bottles(ctx) {
        let drive_c = posix::join(&bottle.path, "drive_c");
        if is_dir(&drive_c) {
            push_unique(&mut roots, drive_c);
        }
        let dos_devices = posix::join(&bottle.path, "dosdevices");
        let Ok(entries) = std::fs::read_dir(&dos_devices) else {
            continue;
        };
        let mut links: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        links.sort();
        for name in links {
            let lower = name.to_lowercase();
            if lower == "c:" || lower == "z:" {
                continue;
            }
            let Ok(resolved) = std::fs::canonicalize(posix::join(&dos_devices, &name)) else {
                continue;
            };
            let resolved = to_posix(&resolved);
            let resolved = resolved.strip_prefix(&VERBATIM_PREFIX.replace('\\', "/")).map(str::to_owned).unwrap_or(resolved);
            if is_dir(&resolved) {
                push_unique(&mut roots, resolved);
            }
        }
    }
    roots
}
