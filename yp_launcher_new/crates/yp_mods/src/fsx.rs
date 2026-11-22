use std::cmp::Ordering;
use std::path::{Component, Path, PathBuf};

use jiff::Timestamp;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Clone)]
pub(crate) struct Child {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_file: bool,
}

pub(crate) fn name_order(a: &str, b: &str) -> Ordering {
    a.chars()
        .flat_map(char::to_uppercase)
        .cmp(b.chars().flat_map(char::to_uppercase))
        .then_with(|| a.cmp(b))
}

pub(crate) fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub(crate) fn children(dir: &Path) -> Vec<Child> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Child> = read
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            let meta = std::fs::metadata(&path).ok();
            Child {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: meta.as_ref().is_some_and(|meta| meta.is_dir()),
                is_file: meta.as_ref().is_some_and(|meta| meta.is_file()),
                path,
            }
        })
        .collect();
    out.sort_by(|a, b| name_order(&a.name, &b.name));
    out
}

pub(crate) fn children_no_follow(dir: &Path) -> Vec<Child> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<Child> = read
        .filter_map(Result::ok)
        .map(|entry| {
            let kind = entry.file_type().ok();
            Child {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: kind.is_some_and(|kind| kind.is_dir()),
                is_file: kind.is_some_and(|kind| kind.is_file()),
                path: entry.path(),
            }
        })
        .collect();
    out.sort_by(|a, b| name_order(&a.name, &b.name));
    out
}

pub(crate) fn child_dirs(dir: &Path) -> Vec<Child> {
    children(dir)
        .into_iter()
        .filter(|child| child.is_dir)
        .collect()
}

pub(crate) fn child_files(dir: &Path) -> Vec<Child> {
    children(dir)
        .into_iter()
        .filter(|child| child.is_file)
        .collect()
}

fn walker(dir: &Path) -> impl Iterator<Item = DirEntry> {
    WalkDir::new(dir)
        .min_depth(1)
        .follow_links(false)
        .sort_by(|a, b| {
            name_order(
                &a.file_name().to_string_lossy(),
                &b.file_name().to_string_lossy(),
            )
        })
        .into_iter()
        .filter_map(Result::ok)
}

#[derive(Debug, Clone)]
pub(crate) struct WalkEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_file: bool,
}

pub(crate) fn walk(dir: &Path) -> Vec<WalkEntry> {
    walker(dir)
        .map(|entry| WalkEntry {
            is_dir: entry.file_type().is_dir(),
            is_file: entry.file_type().is_file(),
            path: entry.into_path(),
        })
        .collect()
}

pub(crate) fn walk_files(dir: &Path) -> Vec<PathBuf> {
    walker(dir)
        .filter(|entry| entry.file_type().is_file())
        .map(DirEntry::into_path)
        .collect()
}

pub(crate) fn any_walked_file(dir: &Path, mut matches: impl FnMut(&Path) -> bool) -> bool {
    walker(dir).any(|entry| entry.file_type().is_file() && matches(entry.path()))
}

pub(crate) fn find_walked_file<T>(
    dir: &Path,
    mut pick: impl FnMut(&Path) -> Option<T>,
) -> Option<T> {
    walker(dir)
        .filter(|entry| entry.file_type().is_file())
        .find_map(|entry| pick(entry.path()))
}

pub(crate) fn lower_path_ends_with(path: &Path, suffix: &str) -> bool {
    path.to_string_lossy().to_lowercase().ends_with(suffix)
}

pub(crate) fn mtime(path: &Path) -> Option<Timestamp> {
    let modified = std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()?;
    Timestamp::try_from(modified).ok()
}

pub(crate) fn exists(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

pub(crate) fn remove_dir_quiet(path: &Path) {
    if path.is_dir() {
        let _ = std::fs::remove_dir_all(path);
    }
}

pub(crate) fn move_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    if std::fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    copy_dir(src, dest)?;
    let _ = std::fs::remove_dir_all(src);
    Ok(())
}

pub(crate) fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in walk(src) {
        let Ok(relative) = entry.path.strip_prefix(src) else {
            continue;
        };
        let target = dest.join(relative);
        if entry.is_dir {
            std::fs::create_dir_all(&target)?;
        } else if entry.is_file {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&entry.path, &target)?;
        }
    }
    Ok(())
}

pub(crate) fn unique_child_name(parent: &Path, name: &str) -> String {
    let mut candidate = name.to_owned();
    let mut counter = 2u32;
    while exists(&parent.join(&candidate)) {
        candidate = format!("{name} ({counter})");
        counter += 1;
    }
    candidate
}

pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut parts: Vec<Component<'_>> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match parts.last() {
                Some(Component::Normal(_)) => {
                    parts.pop();
                }
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => parts.push(component),
            },
            other => parts.push(other),
        }
    }
    if parts.is_empty() {
        return PathBuf::from(".");
    }
    parts.iter().map(|part| part.as_os_str()).collect()
}

fn root_length(parts: &[Component<'_>]) -> usize {
    parts
        .iter()
        .take_while(|part| matches!(part, Component::Prefix(_) | Component::RootDir))
        .count()
}

pub(crate) fn relative(path: &Path, from: &Path) -> Option<PathBuf> {
    let target = normalize(path);
    let base = normalize(from);
    let target_parts: Vec<Component<'_>> = target
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();
    let base_parts: Vec<Component<'_>> = base
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();
    let target_root = root_length(&target_parts);
    let base_root = root_length(&base_parts);
    if target_parts[..target_root] != base_parts[..base_root] {
        return None;
    }
    let common = target_parts
        .iter()
        .zip(base_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();
    if base_parts[common..].contains(&Component::ParentDir) {
        return None;
    }
    let mut out = PathBuf::new();
    for _ in common..base_parts.len() {
        out.push("..");
    }
    for part in &target_parts[common..] {
        out.push(part.as_os_str());
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    Some(out)
}

pub(crate) fn same_path(a: &Path, b: &Path) -> bool {
    normalize(a) == normalize(b)
}

pub(crate) fn is_within(parent: &Path, child: &Path) -> bool {
    let parent = normalize(parent);
    let child = normalize(child);
    child != parent && child.starts_with(&parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_names_case_insensitively_like_ntfs() {
        let mut names = vec!["b", "_x", "A", "a", "B"];
        names.sort_by(|a, b| name_order(a, b));
        assert_eq!(names, vec!["A", "a", "B", "b", "_x"]);
    }

    #[test]
    fn normalizes_lexically() {
        assert_eq!(
            normalize(Path::new("a/./b/../c")),
            PathBuf::from("a").join("c")
        );
        assert_eq!(normalize(Path::new("a/..")), PathBuf::from("."));
        assert_eq!(normalize(Path::new("../a")), PathBuf::from("..").join("a"));
    }

    #[test]
    fn relative_paths() {
        let root = Path::new("root");
        assert_eq!(
            relative(&root.join("a").join("b"), root),
            Some(PathBuf::from("a").join("b"))
        );
        assert_eq!(relative(root, root), Some(PathBuf::from(".")));
        assert_eq!(
            relative(Path::new("other"), root),
            Some(PathBuf::from("..").join("other"))
        );
        assert!(is_within(root, &root.join("a")));
        assert!(!is_within(root, root));
        assert!(same_path(&root.join("a").join(".."), root));
    }

    #[test]
    fn unique_names_count_from_two() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(unique_child_name(dir.path(), "x"), "x");
        std::fs::create_dir(dir.path().join("x")).unwrap();
        std::fs::write(dir.path().join("x (2)"), b"").unwrap();
        assert_eq!(unique_child_name(dir.path(), "x"), "x (3)");
    }
}
