use std::path::{Path, PathBuf};

pub use yp_core::fs::EntryKind;
use yp_core::{IoOp, IoResultExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    pub kind: EntryKind,
}

impl Entry {
    pub fn name(&self) -> String {
        file_name(&self.path)
    }

    pub fn is_dir(&self) -> bool {
        self.kind == EntryKind::Dir
    }

    pub fn is_file(&self) -> bool {
        self.kind == EntryKind::File
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExistingFiles {
    Overwrite,
    Keep,
}

pub fn file_name(path: &Path) -> String {
    yp_core::fs::file_name_lossy(path)
}

pub fn file_name_lower(path: &Path) -> String {
    file_name(path).to_lowercase()
}

pub fn file_stem(path: &Path) -> String {
    path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default()
}

pub fn file_size(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().filter(|meta| meta.is_file()).map(|meta| meta.len())
}

pub fn list_dir(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut entries: Vec<Entry> = read
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let kind = match entry.file_type() {
                Ok(kind) if kind.is_dir() => EntryKind::Dir,
                Ok(kind) if kind.is_file() => EntryKind::File,
                _ => EntryKind::Other,
            };
            Entry {
                path: entry.path(),
                kind,
            }
        })
        .collect();
    entries.sort_by_cached_key(|entry| {
        let name = entry.name();
        (name.to_lowercase(), name)
    });
    entries
}

pub fn walk_entities(root: &Path, visit: &mut dyn FnMut(&Entry) -> bool) {
    if !root.is_dir() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in list_dir(&current) {
            if !visit(&entry) {
                return;
            }
            if entry.is_dir() {
                stack.push(entry.path);
            }
        }
    }
}

fn walk_files(root: &Path, visit: &mut dyn FnMut(&str, &Path) -> bool) {
    walk_entities(root, &mut |entry| {
        if !entry.is_file() {
            return true;
        }
        let rel = yp_core::fs::rel_lower_slash(root, &entry.path).unwrap_or_default();
        visit(&rel, &entry.path)
    });
}

pub fn files_where(root: &Path, mut keep: impl FnMut(&str, &Path) -> bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_files(root, &mut |rel, full| {
        if keep(rel, full) {
            out.push(full.to_path_buf());
        }
        true
    });
    out
}

pub fn first_file_where(root: &Path, mut keep: impl FnMut(&str, &Path) -> bool) -> Option<PathBuf> {
    let mut hit = None;
    walk_files(root, &mut |rel, full| {
        if keep(rel, full) {
            hit = Some(full.to_path_buf());
            false
        } else {
            true
        }
    });
    hit
}

pub fn any_file(root: &Path, matches: impl FnMut(&str, &Path) -> bool) -> bool {
    first_file_where(root, matches).is_some()
}

pub fn file_count(root: &Path) -> usize {
    let mut count = 0;
    walk_entities(root, &mut |entry| {
        if entry.is_file() {
            count += 1;
        }
        true
    });
    count
}

pub fn merge_tree(src: &Path, dest: &Path, existing: ExistingFiles) {
    let _ = match existing {
        ExistingFiles::Overwrite => yp_core::fs::merge_dir(src, dest),
        ExistingFiles::Keep => yp_core::fs::merge_dir_keep_existing(src, dest),
    };
}

pub fn copy_file_as(src: &Path, dir: &Path, name: &str) -> yp_core::Result<()> {
    if !src.is_file() {
        return Ok(());
    }
    std::fs::create_dir_all(dir).io(IoOp::Create, dir)?;
    let _ = yp_core::fs::copy_file_into_as(src, dir, name);
    Ok(())
}

pub fn copy_file_into(src: &Path, dir: &Path) -> yp_core::Result<()> {
    copy_file_as(src, dir, &file_name(src))
}

pub fn remove_dir_if_empty(dir: &Path) {
    let _ = std::fs::remove_dir(dir);
}

pub fn files_differ(a: &Path, b: &Path) -> bool {
    if !a.is_file() || !b.is_file() {
        return true;
    }
    yp_core::fs::files_differ(a, b).unwrap_or(true)
}

pub fn move_merging(src: &Path, dest: &Path) -> Vec<PathBuf> {
    yp_core::fs::move_merging(src, dest).unwrap_or_else(|_| vec![src.to_path_buf()])
}

pub fn unwrap_single_child(root: &Path) -> PathBuf {
    yp_core::fs::unwrap_single_child(root).unwrap_or_else(|_| root.to_path_buf())
}

pub fn normalize_lexically(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let popped = matches!(out.components().next_back(), Some(Component::Normal(_))) && out.pop();
                if !popped && !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn comparable(path: &Path) -> String {
    let text = normalize_lexically(path).to_string_lossy().replace('\\', "/");
    if cfg!(windows) { text.to_lowercase() } else { text }
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    comparable(a) == comparable(b)
}

pub fn is_within(parent: &Path, child: &Path) -> bool {
    let parent = comparable(parent);
    let child = comparable(child);
    let prefix = if parent.ends_with('/') { parent } else { format!("{parent}/") };
    child.len() > prefix.len() && child.starts_with(&prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn walk_visits_root_entries_before_descending() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("b").join("Shaders").join("x.fx"), "x");
        write(&dir.path().join("a").join("deep").join("Shaders").join("y.fx"), "y");
        let mut seen = Vec::new();
        walk_entities(dir.path(), &mut |entry| {
            seen.push(yp_core::fs::rel_lower_slash(dir.path(), &entry.path).unwrap());
            true
        });
        assert_eq!(seen[..2], ["a".to_owned(), "b".to_owned()]);
        assert_eq!(seen[2], "b/shaders");
    }

    #[test]
    fn files_where_reports_lowercase_slash_paths() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("Sub").join("File.INI"), "x");
        let mut rels = Vec::new();
        let found = files_where(dir.path(), |rel, _| {
            rels.push(rel.to_owned());
            rel.ends_with(".ini")
        });
        assert_eq!(rels, ["sub/file.ini"]);
        assert_eq!(found.len(), 1);
        assert!(files_where(&dir.path().join("missing"), |_, _| true).is_empty());
    }

    #[test]
    fn merge_keeps_or_overwrites_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        let dest = dir.path().join("dest");
        write(&src.join("a.txt"), "new");
        write(&src.join("nested").join("b.txt"), "new");
        write(&dest.join("a.txt"), "old");
        merge_tree(&src, &dest, ExistingFiles::Keep);
        assert_eq!(std::fs::read_to_string(dest.join("a.txt")).unwrap(), "old");
        assert_eq!(std::fs::read_to_string(dest.join("nested").join("b.txt")).unwrap(), "new");
        merge_tree(&src, &dest, ExistingFiles::Overwrite);
        assert_eq!(std::fs::read_to_string(dest.join("a.txt")).unwrap(), "new");
    }

    #[test]
    fn lexical_paths() {
        assert_eq!(normalize_lexically(Path::new("a/./b/../c")), PathBuf::from("a").join("c"));
        assert!(is_within(Path::new("a/b"), Path::new("a/b/c")));
        assert!(!is_within(Path::new("a/b"), Path::new("a/b")));
        assert!(!is_within(Path::new("a/b"), Path::new("a/bc")));
        assert!(same_path(Path::new("a/b/../c"), Path::new("a/c")));
    }
}
