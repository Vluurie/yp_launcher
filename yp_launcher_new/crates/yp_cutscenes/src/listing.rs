use std::path::{Path, PathBuf};

use yp_core::{IoOp, IoResultExt, Result};
use yp_formats::usm;

pub(crate) fn children(dir: &Path) -> Result<Vec<PathBuf>> {
    let reader = match std::fs::read_dir(dir) {
        Ok(reader) => reader,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).io(IoOp::List, dir),
    };
    let mut paths = Vec::new();
    for entry in reader {
        paths.push(entry.io(IoOp::List, dir)?.path());
    }
    paths.sort();
    Ok(paths)
}

pub(crate) fn child_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(children(dir)?
        .into_iter()
        .filter(|path| path.is_dir())
        .collect())
}

pub(crate) fn usm_files(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(children(dir)?
        .into_iter()
        .filter(|path| usm::is_usm_path(path) && path.is_file())
        .collect())
}

pub(crate) fn has_usm_file(dir: &Path) -> bool {
    usm_files(dir).is_ok_and(|files| !files.is_empty())
}

pub(crate) fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub(crate) fn is_hidden_name(name: &str) -> bool {
    name.starts_with('.') || name.starts_with('_')
}

pub(crate) fn visible_child_dirs(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(child_dirs(dir)?
        .into_iter()
        .filter(|path| !is_hidden_name(&file_name(path)))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listings_are_sorted_and_missing_dirs_are_empty() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["b.usm", "A.USM", "c.txt", "d.usm.bak"] {
            std::fs::write(dir.path().join(name), b"x").unwrap();
        }
        std::fs::create_dir(dir.path().join("e.usm")).unwrap();
        std::fs::create_dir(dir.path().join(".hidden")).unwrap();
        std::fs::create_dir(dir.path().join("_off")).unwrap();
        std::fs::create_dir(dir.path().join("mod")).unwrap();
        let names: Vec<String> = usm_files(dir.path())
            .unwrap()
            .iter()
            .map(|path| file_name(path))
            .collect();
        assert_eq!(names, ["A.USM", "b.usm"]);
        let dirs: Vec<String> = visible_child_dirs(dir.path())
            .unwrap()
            .iter()
            .map(|path| file_name(path))
            .collect();
        assert_eq!(dirs, ["e.usm", "mod"]);
        assert!(children(&dir.path().join("missing")).unwrap().is_empty());
        assert!(has_usm_file(dir.path()));
        assert!(!has_usm_file(&dir.path().join("mod")));
    }
}
