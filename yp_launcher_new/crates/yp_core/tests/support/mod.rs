#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "test-support")]
use std::path::PathBuf;
use std::path::{Path};
use std::sync::Mutex;

use yp_core::archive::ArchiveEntry;
use yp_core::{CancelToken, Progress, ProgressSink};

#[cfg(feature = "test-support")]
pub fn fixture(name: &str) -> PathBuf {
    yp_core::testing::fixtures::path(&format!("yp_core/{name}"))
}

pub fn tree(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1).sort_by_file_name() {
        let entry = entry.unwrap();
        let relative = yp_core::fs::rel_slash(root, entry.path()).unwrap();
        if entry.file_type().is_dir() {
            found.push(format!("{relative}/"));
        } else {
            found.push(relative);
        }
    }
    found.sort();
    found
}

pub fn files(root: &Path) -> Vec<String> {
    tree(root).into_iter().filter(|path| !path.ends_with('/')).collect()
}

pub fn read(root: &Path, relative: &str) -> Vec<u8> {
    let path = relative.split('/').fold(root.to_path_buf(), |path, part| path.join(part));
    std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub fn text(root: &Path, relative: &str) -> String {
    String::from_utf8(read(root, relative)).unwrap()
}

fn key(path: &str) -> String {
    if cfg!(any(windows, target_os = "macos")) {
        path.to_lowercase()
    } else {
        path.to_owned()
    }
}

fn parents(path: &str) -> impl Iterator<Item = &str> {
    path.char_indices()
        .filter(|(_, character)| *character == '/')
        .map(move |(index, _)| &path[..index])
}

pub fn listing_mismatch(listing: &[ArchiveEntry], root: &Path) -> Option<String> {
    let mut listed_files = BTreeMap::new();
    let mut listed_dirs = BTreeSet::new();
    for entry in listing {
        listed_dirs.extend(parents(&entry.path).map(key));
        if entry.is_dir {
            listed_dirs.insert(key(&entry.path));
        } else {
            listed_files.insert(key(&entry.path), entry.size);
        }
    }
    let mut found_files = BTreeMap::new();
    let mut found_dirs = BTreeSet::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Some(format!("walk failed: {error}")),
        };
        let Some(relative) = yp_core::fs::rel_slash(root, entry.path()) else {
            return Some(format!("outside the root: {}", entry.path().display()));
        };
        if entry.file_type().is_dir() {
            found_dirs.insert(key(&relative));
        } else {
            let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(u64::MAX);
            found_files.insert(key(&relative), size);
        }
    }
    for (path, size) in &listed_files {
        match found_files.get(path) {
            None => return Some(format!("listed but not extracted: {path}")),
            Some(found) if found != size => {
                return Some(format!("{path}: listed {size} bytes, extracted {found} bytes"));
            }
            Some(_) => {}
        }
    }
    if let Some(extra) = found_files.keys().find(|path| !listed_files.contains_key(*path)) {
        return Some(format!("extracted but not listed: {extra}"));
    }
    if let Some(missing) = listed_dirs.difference(&found_dirs).next() {
        return Some(format!("listed folder not created: {missing}"));
    }
    if let Some(extra) = found_dirs.difference(&listed_dirs).next() {
        return Some(format!("created folder not listed: {extra}"));
    }
    None
}

#[derive(Default)]
pub struct Recorder {
    seen: Mutex<Vec<Progress>>,
    cancel_after: Option<(CancelToken, f32)>,
}

impl Recorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancelling(cancel: &CancelToken, once_past: f32) -> Self {
        Self {
            seen: Mutex::default(),
            cancel_after: Some((cancel.clone(), once_past)),
        }
    }

    pub fn seen(&self) -> Vec<Progress> {
        self.seen.lock().unwrap().clone()
    }

    pub fn fractions(&self) -> Vec<f32> {
        self.seen().iter().filter_map(|progress| progress.fraction).collect()
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.seen().into_iter().filter_map(|progress| progress.current).collect();
        names.dedup();
        names
    }
}

impl ProgressSink for Recorder {
    fn report(&self, progress: Progress) {
        if let Some((cancel, once_past)) = &self.cancel_after
            && progress.fraction.is_some_and(|fraction| fraction > *once_past)
        {
            cancel.cancel();
        }
        self.seen.lock().unwrap().push(progress);
    }
}
