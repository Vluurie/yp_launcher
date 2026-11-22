use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::host::HostEnv;

pub const NIER_STEAM_APP_ID: &str = "524220";
pub const WINE_DEBUG_VAR: &str = "WINEDEBUG";
pub const WINE_DEBUG_DEFAULT: &str = "-all";

const DRIVE_C: &str = "drive_c";
const PUBLIC_USER: &str = "public";
const CONVENTIONAL_USERS: [&str; 2] = ["steamuser", "crossover"];
const HOST_USER_VARS: [&str; 3] = ["USER", "USERNAME", "LOGNAME"];

pub mod posix {
    pub fn is_absolute(path: &str) -> bool {
        path.starts_with('/')
    }

    pub fn normalize(path: &str) -> String {
        let absolute = is_absolute(path);
        let mut parts: Vec<&str> = Vec::new();
        for segment in path.split('/') {
            match segment {
                "" | "." => {}
                ".." => match parts.last() {
                    Some(&last) if last != ".." => {
                        parts.pop();
                    }
                    _ if absolute => {}
                    _ => parts.push(".."),
                },
                other => parts.push(other),
            }
        }
        let joined = parts.join("/");
        match (absolute, joined.is_empty()) {
            (true, _) => format!("/{joined}"),
            (false, true) => ".".to_owned(),
            (false, false) => joined,
        }
    }

    pub fn join(base: &str, part: &str) -> String {
        if base.is_empty() || is_absolute(part) {
            return part.to_owned();
        }
        if part.is_empty() {
            return base.to_owned();
        }
        if base.ends_with('/') {
            format!("{base}{part}")
        } else {
            format!("{base}/{part}")
        }
    }

    pub fn join_all(base: &str, parts: &[&str]) -> String {
        parts.iter().fold(base.to_owned(), |acc, part| join(&acc, part))
    }

    pub fn basename(path: &str) -> &str {
        let trimmed = path.trim_end_matches('/');
        if trimmed.is_empty() {
            return if path.is_empty() { "" } else { "/" };
        }
        trimmed.rsplit('/').next().unwrap_or(trimmed)
    }

    pub fn dirname(path: &str) -> String {
        let trimmed = path.trim_end_matches('/');
        if trimmed.is_empty() {
            return if is_absolute(path) { "/".to_owned() } else { ".".to_owned() };
        }
        match trimmed.rfind('/') {
            None => ".".to_owned(),
            Some(0) => "/".to_owned(),
            Some(index) => trimmed[..index].trim_end_matches('/').to_owned(),
        }
    }

    pub fn segments(path: &str) -> Vec<&str> {
        path.split('/').filter(|segment| !segment.is_empty()).collect()
    }

    pub fn is_within(target: &str, parent: &str) -> bool {
        let target = normalize(target);
        let parent = normalize(parent);
        if target == parent {
            return true;
        }
        if is_absolute(&target) != is_absolute(&parent) {
            return false;
        }
        let target_parts = segments(&target);
        let parent_parts = segments(&parent);
        if parent == "." {
            return target_parts.first().is_none_or(|first| *first != "..");
        }
        target_parts.len() > parent_parts.len() && target_parts.starts_with(&parent_parts)
    }
}

pub fn to_posix(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn from_posix(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn drive_c_index(parts: &[&str]) -> Option<usize> {
    parts.iter().position(|part| part.eq_ignore_ascii_case(DRIVE_C))
}

pub fn infer_wine_prefix(target: &str) -> Option<String> {
    let normalized = posix::normalize(target);
    let parts: Vec<&str> = normalized.split('/').collect();
    let index = drive_c_index(&parts)?;
    if index == 0 {
        return None;
    }
    let prefix = parts[..index].join("/");
    Some(if prefix.is_empty() { "/".to_owned() } else { prefix })
}

pub fn infer_wine_prefix_path(target: &Path) -> Option<PathBuf> {
    infer_wine_prefix(&to_posix(target)).map(PathBuf::from)
}

pub fn is_crossover_prefix(prefix: &str) -> bool {
    let lower = prefix.to_lowercase();
    let parts: Vec<&str> = lower.split('/').collect();
    let followed = |index: usize| index + 1 < parts.len();
    parts.iter().enumerate().any(|(index, part)| {
        let preceded = index > 0;
        if !preceded {
            return false;
        }
        match *part {
            "crossover" | "crossover preview" => {
                parts.get(index + 1) == Some(&"bottles") && followed(index + 1)
            }
            ".cxoffice" => followed(index),
            _ => false,
        }
    })
}

pub fn to_wine_path(target: &str) -> String {
    let normalized = posix::normalize(target);
    let parts: Vec<&str> = normalized.split('/').collect();
    match drive_c_index(&parts) {
        Some(index) => format!("C:\\{}", parts[index + 1..].join("\\")),
        None => format!("Z:{}", normalized.replace('/', "\\")),
    }
}

pub fn wine_users_dir(prefix: &str) -> String {
    posix::join_all(prefix, &[DRIVE_C, "users"])
}

pub fn resolve_wine_user(users_dir: &str, host: &HostEnv) -> String {
    let mut preferred: Vec<String> = HOST_USER_VARS
        .iter()
        .filter_map(|name| host.var(name).map(str::to_owned))
        .collect();
    preferred.extend(CONVENTIONAL_USERS.iter().map(|name| (*name).to_owned()));

    let mut existing = list_dir_names(Path::new(users_dir));
    existing.retain(|name| name.to_lowercase() != PUBLIC_USER);
    existing.sort();

    for candidate in &preferred {
        if let Some(found) = existing.iter().find(|name| name.to_lowercase() == candidate.to_lowercase()) {
            return found.clone();
        }
    }
    existing
        .into_iter()
        .next()
        .or_else(|| preferred.into_iter().next())
        .unwrap_or_default()
}

pub fn wine_roaming_dir(prefix: &str, host: &HostEnv) -> String {
    let users = wine_users_dir(prefix);
    let user = resolve_wine_user(&users, host);
    posix::join_all(&users, &[&user, "AppData", "Roaming"])
}

pub fn wine_env(host: &HostEnv) -> BTreeMap<String, String> {
    BTreeMap::from([(
        WINE_DEBUG_VAR.to_owned(),
        host.var(WINE_DEBUG_VAR).unwrap_or(WINE_DEBUG_DEFAULT).to_owned(),
    )])
}

fn list_dir_names(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamContext {
    pub steam_root: String,
    pub library_root: String,
    pub compat_data: String,
}

impl SteamContext {
    pub fn prefix(&self) -> String {
        posix::join(&self.compat_data, "pfx")
    }
}

pub fn host_home(host: &HostEnv) -> String {
    host.var("HOME")
        .map(str::to_owned)
        .or_else(|| host.home().map(|home| to_posix(&home)))
        .unwrap_or_default()
}

pub fn default_steam_roots(home: &str) -> Vec<String> {
    vec![
        posix::join_all(home, &[".steam", "steam"]),
        posix::join_all(home, &[".local", "share", "Steam"]),
        posix::join_all(home, &["snap", "steam", "common", ".local", "share", "Steam"]),
        posix::join_all(home, &[".var", "app", "com.valvesoftware.Steam", ".local", "share", "Steam"]),
    ]
}

pub fn steam_root_candidates(host: &HostEnv) -> Vec<String> {
    default_steam_roots(&host_home(host))
        .into_iter()
        .filter(|root| Path::new(root).is_dir())
        .collect()
}

pub fn infer_steam_context(game_path: &str, steam_roots: &[String]) -> Option<SteamContext> {
    let normalized = posix::normalize(&game_path.replace('\\', "/"));
    let parts: Vec<&str> = normalized.split('/').collect();
    let marker = (1..parts.len().saturating_sub(2)).find(|&index| {
        parts[index].eq_ignore_ascii_case("steamapps") && parts[index + 1].eq_ignore_ascii_case("common")
    })?;
    let library_root = parts[..marker].join("/");
    let target = posix::normalize(&library_root);
    let steam_root = steam_roots
        .iter()
        .find(|root| posix::normalize(root) == target)
        .or_else(|| steam_roots.first())
        .cloned()
        .unwrap_or_else(|| library_root.clone());
    Some(SteamContext {
        compat_data: posix::join_all(&library_root, &["steamapps", "compatdata", NIER_STEAM_APP_ID]),
        steam_root,
        library_root,
    })
}

pub fn infer_steam_context_for(host: &HostEnv, game_path: &Path) -> Option<SteamContext> {
    infer_steam_context(&to_posix(game_path), &steam_root_candidates(host))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posix_normalize_matches_dart() {
        assert_eq!(posix::normalize("/a//b/./c/../d/"), "/a/b/d");
        assert_eq!(posix::normalize("a/../../b"), "../b");
        assert_eq!(posix::normalize("/.."), "/");
        assert_eq!(posix::normalize(""), ".");
        assert_eq!(posix::normalize("C:/x/y/.."), "C:/x");
    }

    #[test]
    fn posix_join_and_names() {
        assert_eq!(posix::join("/a", "b"), "/a/b");
        assert_eq!(posix::join("/a/", "b"), "/a/b");
        assert_eq!(posix::join("/a", "/b"), "/b");
        assert_eq!(posix::basename("/a/b/"), "b");
        assert_eq!(posix::dirname("/a/b"), "/a");
        assert_eq!(posix::dirname("/a"), "/");
        assert_eq!(posix::dirname("a"), ".");
    }

    #[test]
    fn posix_is_within() {
        assert!(posix::is_within("/a/b/c", "/a/b"));
        assert!(posix::is_within("/a/b", "/a/b/"));
        assert!(!posix::is_within("/a/bc", "/a/b"));
        assert!(!posix::is_within("/a", "/a/b"));
    }
}
