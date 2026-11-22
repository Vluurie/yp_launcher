use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use yp_formats::usm;

use crate::listing::has_usm_file;

const MOVIE_DIR: &str = "movie";

pub fn find_movie_dir(root: &Path) -> Option<PathBuf> {
    if !root.is_dir() {
        return None;
    }
    if has_usm_file(root) {
        return Some(root.to_path_buf());
    }
    let direct = root.join(MOVIE_DIR);
    if direct.is_dir() && has_usm_file(&direct) {
        return Some(direct);
    }
    WalkDir::new(root)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .filter(|entry| entry.file_name().to_string_lossy().to_lowercase() == MOVIE_DIR)
        .map(|entry| entry.into_path())
        .find(|dir| has_usm_file(dir))
}

pub fn usm_files_below(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file() && usm::is_usm_path(entry.path()))
        .map(|entry| entry.into_path())
        .collect()
}

pub fn dir_contains_usm(dir: &Path) -> bool {
    WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .any(|entry| entry.file_type().is_file() && usm::is_usm_path(entry.path()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"usm").unwrap();
    }

    #[test]
    fn prefers_the_root_then_a_direct_movie_folder() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("movie").join("ev0010.usm"));
        assert_eq!(find_movie_dir(dir.path()), Some(dir.path().join("movie")));
        touch(&dir.path().join("ev0030.USM"));
        assert_eq!(find_movie_dir(dir.path()), Some(dir.path().to_path_buf()));
    }

    #[test]
    fn finds_nested_movie_folders_case_insensitively_in_sorted_order() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("b").join("Movie").join("ev0010.usm"));
        touch(
            &dir.path()
                .join("a")
                .join("data")
                .join("MOVIE")
                .join("ev0020.usm"),
        );
        std::fs::create_dir_all(dir.path().join("a").join("movie")).unwrap();
        touch(&dir.path().join("a").join("movie").join("readme.txt"));
        assert_eq!(
            find_movie_dir(dir.path()),
            Some(dir.path().join("a").join("data").join("MOVIE"))
        );
    }

    #[test]
    fn nothing_found() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("videos").join("ev0010.usm"));
        assert_eq!(find_movie_dir(dir.path()), None);
        assert_eq!(find_movie_dir(&dir.path().join("missing")), None);
        assert_eq!(
            find_movie_dir(&dir.path().join("videos").join("ev0010.usm")),
            None
        );
        assert!(dir_contains_usm(dir.path()));
        assert!(!dir_contains_usm(&dir.path().join("missing")));
    }

    #[test]
    fn lists_usm_files_recursively_in_order() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("b.usm"));
        touch(&dir.path().join("sub").join("a.usm"));
        touch(&dir.path().join("a.usm"));
        touch(&dir.path().join("c.txt"));
        let names: Vec<PathBuf> = usm_files_below(dir.path())
            .iter()
            .map(|path| path.strip_prefix(dir.path()).unwrap().to_path_buf())
            .collect();
        assert_eq!(
            names,
            [
                PathBuf::from("a.usm"),
                PathBuf::from("b.usm"),
                Path::new("sub").join("a.usm")
            ]
        );
    }
}
