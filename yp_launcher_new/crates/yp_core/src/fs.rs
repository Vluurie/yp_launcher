use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{Error, IoOp, IoResultExt, Result};
use crate::host::HostEnv;
use crate::paths::GamePaths;

const UNWRAP_IGNORED_SUFFIXES: [&str; 4] = [".txt", ".md", ".pdf", ".url"];
const UNWRAP_IGNORED_PREFIXES: [&str; 2] = ["readme", "__macosx"];

pub fn read_text_opt(path: &Path) -> Result<Option<String>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).into_owned())),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).io(IoOp::Read, path),
    }
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).io(IoOp::Create, dir)?;
    let mut temp = tempfile::NamedTempFile::new_in(dir).io(IoOp::Create, dir)?;
    temp.write_all(bytes).io(IoOp::Write, temp.path())?;
    temp.as_file().sync_all().io(IoOp::Write, temp.path())?;
    let mut temp = temp.into_temp_path();
    let mut attempt = 0u64;
    loop {
        match temp.persist(path) {
            Ok(()) => return Ok(()),
            Err(err) if attempt < 5 && cfg!(windows) && err.error.kind() == std::io::ErrorKind::PermissionDenied => {
                attempt += 1;
                temp = err.path;
                std::thread::sleep(std::time::Duration::from_millis(20 * attempt));
            }
            Err(err) => return Err(err.error).io(IoOp::Rename, path),
        }
    }
}

pub fn write_text_atomic(path: &Path, text: &str) -> Result<()> {
    write_atomic(path, text.as_bytes())
}

pub fn remove_dir_quiet(path: &Path) -> bool {
    std::fs::remove_dir_all(path).is_ok()
}

pub fn remove_file_quiet(path: &Path) -> bool {
    std::fs::remove_file(path).is_ok()
}

pub fn is_dir_writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".write_test");
    let ok = std::fs::write(&probe, b"ok").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

pub fn rel_lower_slash(root: &Path, path: &Path) -> Option<String> {
    rel_slash(root, path).map(|relative| relative.to_lowercase())
}

pub fn rel_slash(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    Some(parts.join("/"))
}

pub fn size_label(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= MIB {
        format!("{:.1} MB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{} KB", (bytes as f64 / KIB as f64).round() as u64)
    } else {
        format!("{bytes} B")
    }
}

pub fn file_size_label(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok().filter(|meta| meta.is_file())?;
    Some(size_label(meta.len()))
}

pub fn scoped_temp_dir(game: Option<&GamePaths>, host: &HostEnv, prefix: &str) -> Result<tempfile::TempDir> {
    if let Some(game) = game.filter(|game| !game.root().as_os_str().is_empty()) {
        let root = game.scoped_temp_root();
        if std::fs::create_dir_all(&root).is_ok()
            && let Ok(dir) = tempfile::Builder::new().prefix(prefix).tempdir_in(&root)
        {
            return Ok(dir);
        }
    }
    let root = host.temp_dir();
    std::fs::create_dir_all(root).io(IoOp::Create, root)?;
    tempfile::Builder::new().prefix(prefix).tempdir_in(root).io(IoOp::Create, root)
}

pub fn temp_sweep_roots(game: Option<&GamePaths>, host: &HostEnv) -> Vec<PathBuf> {
    let mut roots = vec![host.temp_dir().to_path_buf()];
    if let Some(game) = game.filter(|game| !game.root().as_os_str().is_empty()) {
        roots.push(game.scoped_temp_root());
    }
    roots
}

pub fn sweep_stale_temp(roots: &[PathBuf], prefixes: &[&str]) {
    for root in roots {
        for entry in sorted_entries(root) {
            let is_dir = std::fs::symlink_metadata(&entry).map(|meta| meta.is_dir()).unwrap_or(false);
            let name = file_name_lossy(&entry);
            if is_dir && prefixes.iter().any(|prefix| name.starts_with(prefix)) {
                remove_dir_quiet(&entry);
            }
        }
    }
}

pub fn files_differ(a: &Path, b: &Path) -> Result<bool> {
    let (Ok(meta_a), Ok(meta_b)) = (std::fs::metadata(a), std::fs::metadata(b)) else {
        return Ok(true);
    };
    if !meta_a.is_file() || !meta_b.is_file() {
        return Ok(true);
    }
    if meta_a.len() != meta_b.len() {
        return Ok(true);
    }
    Ok(crate::hash::sha256_file(a)? != crate::hash::sha256_file(b)?)
}

pub fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    merge_dir(src, dest)
}

pub fn merge_dir(src: &Path, dest: &Path) -> Result<()> {
    merge_dir_with(src, dest, true)
}

pub fn merge_dir_keep_existing(src: &Path, dest: &Path) -> Result<()> {
    merge_dir_with(src, dest, false)
}

fn merge_dir_with(src: &Path, dest: &Path, overwrite: bool) -> Result<()> {
    if !src.is_dir() {
        return Ok(());
    }
    let mut first_error = None;
    merge_into(src, dest, overwrite, &mut HashSet::new(), &mut first_error);
    first_error.map_or(Ok(()), Err)
}

fn merge_into(src: &Path, dest: &Path, overwrite: bool, visited: &mut HashSet<PathBuf>, error: &mut Option<Error>) {
    let Ok(resolved) = std::fs::canonicalize(src) else {
        return;
    };
    if !visited.insert(resolved) {
        return;
    }
    if let Err(err) = std::fs::create_dir_all(dest).io(IoOp::Create, dest) {
        error.get_or_insert(err);
        return;
    }
    let entries = match list_sorted(src) {
        Ok(entries) => entries,
        Err(err) => {
            error.get_or_insert(err);
            return;
        }
    };
    for entry in entries {
        let Some(name) = entry.file_name() else {
            continue;
        };
        let target = dest.join(name);
        let Ok(meta) = std::fs::metadata(&entry) else {
            continue;
        };
        if meta.is_dir() {
            merge_into(&entry, &target, overwrite, visited, error);
        } else if meta.is_file() {
            if !overwrite && target.is_file() {
                continue;
            }
            if let Err(err) = std::fs::copy(&entry, &target).io(IoOp::Copy, &target) {
                error.get_or_insert(err);
            }
        }
    }
}

pub fn move_dir(src: &Path, dest: &Path) -> Result<()> {
    if !src.is_dir() {
        return Ok(());
    }
    if let Some(parent) = dest.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).io(IoOp::Create, parent)?;
    }
    if std::fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    merge_dir(src, dest)?;
    remove_dir_quiet(src);
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryType {
    Missing,
    File,
    Dir,
    Other,
}

fn entry_type(path: &Path) -> EntryType {
    match std::fs::symlink_metadata(path) {
        Err(_) => EntryType::Missing,
        Ok(meta) if meta.is_dir() => EntryType::Dir,
        Ok(meta) if meta.is_file() => EntryType::File,
        Ok(_) => EntryType::Other,
    }
}

pub fn move_merging(src: &Path, dest: &Path) -> Result<Vec<PathBuf>> {
    let mut left = Vec::new();
    move_merging_into(src, dest, &mut left);
    Ok(left)
}

fn move_merging_into(src: &Path, dest: &Path, left: &mut Vec<PathBuf>) {
    let source_type = entry_type(src);
    let dest_type = entry_type(dest);
    match source_type {
        EntryType::Dir => move_dir_merging(src, dest, dest_type, left),
        EntryType::File => move_file_merging(src, dest, dest_type, left),
        EntryType::Missing => {}
        EntryType::Other => left.push(src.to_path_buf()),
    }
}

fn move_dir_merging(src: &Path, dest: &Path, dest_type: EntryType, left: &mut Vec<PathBuf>) {
    match dest_type {
        EntryType::Missing => {
            if let Some(parent) = dest.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if std::fs::rename(src, dest).is_ok() {
                return;
            }
        }
        EntryType::Dir => {}
        EntryType::File | EntryType::Other => {
            left.push(src.to_path_buf());
            return;
        }
    }
    if std::fs::create_dir_all(dest).is_err() {
        left.push(src.to_path_buf());
        return;
    }
    let Ok(children) = list_sorted(src) else {
        left.push(src.to_path_buf());
        return;
    };
    for child in children {
        if let Some(name) = child.file_name() {
            move_merging_into(&child, &dest.join(name), left);
        }
    }
    let _ = std::fs::remove_dir(src);
}

fn move_file_merging(src: &Path, dest: &Path, dest_type: EntryType, left: &mut Vec<PathBuf>) {
    if dest_type != EntryType::Missing {
        if dest_type == EntryType::File && !files_differ(src, dest).unwrap_or(true) {
            let _ = std::fs::remove_file(src);
        } else {
            left.push(src.to_path_buf());
        }
        return;
    }
    if let Some(parent) = dest.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::rename(src, dest).is_ok() {
        return;
    }
    let copied = std::fs::copy(src, dest)
        .ok()
        .zip(std::fs::metadata(src).ok())
        .is_some_and(|(written, meta)| written == meta.len());
    if !copied {
        let _ = std::fs::remove_file(dest);
        left.push(src.to_path_buf());
        return;
    }
    let _ = std::fs::remove_file(src);
}

pub fn copy_file_into(file: &Path, dir: &Path) -> Result<PathBuf> {
    let name = file
        .file_name()
        .ok_or_else(|| Error::invalid("file name", file.display().to_string()))?;
    copy_file_into_as(file, dir, &name.to_string_lossy())
}

pub fn copy_file_into_as(file: &Path, dir: &Path, name: &str) -> Result<PathBuf> {
    if !file.is_file() {
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound)).io(IoOp::Read, file);
    }
    std::fs::create_dir_all(dir).io(IoOp::Create, dir)?;
    let target = dir.join(name);
    std::fs::copy(file, &target).io(IoOp::Copy, &target)?;
    Ok(target)
}

pub fn unwrap_single_child(dir: &Path) -> Result<PathBuf> {
    if !dir.is_dir() {
        return Ok(dir.to_path_buf());
    }
    let entries = list_sorted(dir)?;
    let mut dirs = Vec::new();
    let mut files = 0usize;
    for entry in entries {
        let Ok(meta) = std::fs::symlink_metadata(&entry) else {
            continue;
        };
        if meta.is_dir() {
            dirs.push(entry);
        } else if meta.is_file() && !is_unwrap_noise(&file_name_lossy(&entry)) {
            files += 1;
        }
    }
    if dirs.len() == 1 && files == 0 {
        return Ok(dirs.remove(0));
    }
    Ok(dir.to_path_buf())
}

fn is_unwrap_noise(name: &str) -> bool {
    let lower = name.to_lowercase();
    UNWRAP_IGNORED_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix))
        || UNWRAP_IGNORED_PREFIXES.iter().any(|prefix| lower.starts_with(prefix))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Dir,
    Other,
}

pub fn walk_entities(root: &Path, visit: &mut dyn FnMut(&Path, EntryKind) -> bool) {
    if !root.is_dir() {
        return;
    }
    walk_dir(root, visit);
}

fn walk_dir(dir: &Path, visit: &mut dyn FnMut(&Path, EntryKind) -> bool) -> bool {
    let Ok(entries) = list_sorted(dir) else {
        return true;
    };
    let mut subdirs = Vec::new();
    for entry in entries {
        let kind = match std::fs::symlink_metadata(&entry) {
            Ok(meta) if meta.is_dir() => EntryKind::Dir,
            Ok(meta) if meta.is_file() => EntryKind::File,
            Ok(_) => EntryKind::Other,
            Err(_) => continue,
        };
        if !visit(&entry, kind) {
            return false;
        }
        if kind == EntryKind::Dir {
            subdirs.push(entry);
        }
    }
    subdirs.into_iter().all(|subdir| walk_dir(&subdir, visit))
}

pub fn files_where(root: &Path, keep: &mut dyn FnMut(&str, &Path) -> bool) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk_entities(root, &mut |path, kind| {
        if kind == EntryKind::File
            && let Some(relative) = rel_lower_slash(root, path)
            && keep(&relative, path)
        {
            out.push(path.to_path_buf());
        }
        true
    });
    Ok(out)
}

pub fn any_file(root: &Path, matches: &mut dyn FnMut(&str, &Path) -> bool) -> bool {
    let mut found = false;
    walk_entities(root, &mut |path, kind| {
        if kind == EntryKind::File
            && let Some(relative) = rel_lower_slash(root, path)
            && matches(&relative, path)
        {
            found = true;
            return false;
        }
        true
    });
    found
}

pub fn unique_child_name(dir: &Path, base: &str) -> String {
    let mut candidate = base.to_owned();
    let mut counter = 2u32;
    while std::fs::symlink_metadata(dir.join(&candidate)).is_ok() {
        candidate = format!("{base} ({counter})");
        counter += 1;
    }
    candidate
}

pub fn list_sorted(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .io(IoOp::List, dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort_by_cached_key(|path| {
        let name = file_name_lossy(path);
        (name.to_lowercase(), name)
    });
    Ok(entries)
}

fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    list_sorted(dir).unwrap_or_default()
}

pub fn file_name_lossy(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested").join("a.json");
        write_text_atomic(&file, "one").unwrap();
        write_text_atomic(&file, "two").unwrap();
        assert_eq!(read_text_opt(&file).unwrap().as_deref(), Some("two"));
        assert_eq!(read_text_opt(&dir.path().join("missing")).unwrap(), None);
        assert_eq!(std::fs::read_dir(file.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn labels_sizes_like_the_flutter_launcher() {
        assert_eq!(size_label(512), "512 B");
        assert_eq!(size_label(1536), "2 KB");
        assert_eq!(size_label(1024), "1 KB");
        assert_eq!(size_label(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(size_label(1024 * 1024 + 512 * 1024), "1.5 MB");
    }

    #[test]
    fn relative_lowercase_slash_paths() {
        let root = Path::new("Root");
        assert_eq!(
            rel_lower_slash(root, &root.join("Data").join("PL.dat")).as_deref(),
            Some("data/pl.dat")
        );
        assert_eq!(rel_slash(root, &root.join("Data").join("PL.dat")).as_deref(), Some("Data/PL.dat"));
        assert_eq!(rel_lower_slash(root, Path::new("Other")), None);
    }
}
