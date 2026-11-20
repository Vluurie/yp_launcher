use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{DocsError, Result};
use crate::links;
use crate::markdown::split_lines;
use crate::search::{self, DocSearchHit};

pub const ASSET_ROOT: &str = "assets/docs/";
pub const IMAGES_DIR: &str = "images";
pub const MARKDOWN_EXTENSION: &str = ".md";
const RESOURCES_SEGMENT: &str = "resources";
const AUTHORING_SEGMENT: &str = "authoring";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocPage {
    pub path: String,
    pub category: String,
    pub title: String,
    pub file_name: String,
}

fn has_segment(path: &str, segment: &str) -> bool {
    let segments: Vec<&str> = path.split('/').collect();
    segments.len() > 2 && segments[1..segments.len() - 1].contains(&segment)
}

impl DocPage {
    pub fn is_resource(&self) -> bool {
        has_segment(&self.path, RESOURCES_SEGMENT)
    }

    pub fn is_authoring(&self) -> bool {
        has_segment(&self.path, AUTHORING_SEGMENT)
    }
}

pub fn pretty_name(file_name: &str) -> String {
    let mut name = file_name
        .strip_suffix(MARKDOWN_EXTENSION)
        .unwrap_or(file_name);
    if let Some((prefix, rest)) = name.split_once('_')
        && !prefix.is_empty()
        && prefix.parse::<i64>().is_ok()
    {
        name = rest;
    }
    let spaced = name.replace('_', " ");
    let trimmed = spaced.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => file_name.to_owned(),
    }
}

pub fn title_for(markdown: &str, file_name: &str) -> String {
    for line in split_lines(markdown) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.trim().to_owned();
        }
        break;
    }
    pretty_name(file_name)
}

fn path_rank(path: &str) -> u8 {
    if has_segment(path, AUTHORING_SEGMENT) {
        2
    } else if has_segment(path, RESOURCES_SEGMENT) {
        1
    } else {
        0
    }
}

pub fn compare_paths(a: &str, b: &str) -> Ordering {
    path_rank(a).cmp(&path_rank(b)).then_with(|| a.cmp(b))
}

pub fn asset_path(relative_slash: &str) -> String {
    format!("{ASSET_ROOT}{relative_slash}")
}

pub fn relative_of(asset_path: &str) -> Option<&str> {
    asset_path.strip_prefix(ASSET_ROOT)
}

#[derive(Debug)]
pub struct DocsLibrary {
    docs_dir: PathBuf,
    editable: bool,
    pages: Option<Vec<DocPage>>,
    contents: HashMap<String, String>,
}

impl DocsLibrary {
    pub fn new(docs_dir: impl Into<PathBuf>) -> Self {
        Self {
            docs_dir: docs_dir.into(),
            editable: false,
            pages: None,
            contents: HashMap::new(),
        }
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn docs_dir(&self) -> &Path {
        &self.docs_dir
    }

    pub fn can_edit(&self) -> bool {
        self.editable && self.docs_dir.is_dir()
    }

    pub fn file_for(&self, asset_path: &str) -> Option<PathBuf> {
        let relative = relative_of(asset_path)?;
        let mut file = self.docs_dir.clone();
        for part in relative.split('/').filter(|part| !part.is_empty()) {
            if part == ".." || part == "." {
                return None;
            }
            file.push(part);
        }
        Some(file)
    }

    pub fn source_file_for(&self, asset_path: &str) -> Option<PathBuf> {
        if !self.can_edit() {
            return None;
        }
        self.file_for(asset_path).filter(|file| file.is_file())
    }

    pub fn save_source(&mut self, asset_path: &str, content: &str) -> Result<bool> {
        let Some(file) = self.source_file_for(asset_path) else {
            return Ok(false);
        };
        yp_core::fs::write_text_atomic(&file, content)?;
        self.contents
            .insert(asset_path.to_owned(), content.to_owned());
        self.pages = None;
        Ok(true)
    }

    pub fn invalidate(&mut self) {
        self.contents.clear();
        self.pages = None;
    }

    pub fn cached_pages(&self) -> Option<&[DocPage]> {
        self.pages.as_deref()
    }

    fn markdown_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = walkdir::WalkDir::new(&self.docs_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter_map(|entry| yp_core::fs::rel_slash(&self.docs_dir, entry.path()))
            .filter(|relative| relative.ends_with(MARKDOWN_EXTENSION))
            .filter(|relative| relative.split('/').next() != Some(IMAGES_DIR))
            .map(|relative| asset_path(&relative))
            .collect();
        paths.sort_by(|a, b| compare_paths(a, b));
        paths
    }

    pub fn load_index(&mut self) -> Result<&[DocPage]> {
        if self.pages.is_none() {
            let mut pages = Vec::new();
            for path in self.markdown_paths() {
                let Some(relative) = relative_of(&path) else {
                    continue;
                };
                let Some((category, _)) = relative.split_once('/') else {
                    continue;
                };
                let file_name = relative.rsplit('/').next().unwrap_or(relative).to_owned();
                let category = category.to_owned();
                let markdown = self.content(&path)?;
                pages.push(DocPage {
                    title: title_for(&markdown, &file_name),
                    path,
                    category,
                    file_name,
                });
            }
            self.pages = Some(pages);
        }
        Ok(self.pages.as_deref().unwrap_or_default())
    }

    pub fn content(&mut self, asset_path: &str) -> Result<String> {
        if let Some(cached) = self.contents.get(asset_path) {
            return Ok(cached.clone());
        }
        let file = self
            .file_for(asset_path)
            .ok_or_else(|| DocsError::NotADocPath {
                path: asset_path.to_owned(),
            })?;
        let text = std::fs::read_to_string(&file).map_err(|source| yp_core::Error::Io {
            op: yp_core::IoOp::Read,
            path: file.clone(),
            source,
        })?;
        self.contents.insert(asset_path.to_owned(), text.clone());
        Ok(text)
    }

    pub fn resolve_link(&self, from_path: &str, href: &str) -> Option<String> {
        links::resolve_link(self.pages.as_deref(), from_path, href)
    }

    pub fn search(&mut self, query: &str) -> Result<Vec<DocSearchHit>> {
        let pages = self.load_index()?.to_vec();
        let mut bodies = Vec::with_capacity(pages.len());
        for page in &pages {
            bodies.push(self.content(&page.path)?);
        }
        Ok(search::search_pages(&pages, &bodies, query))
    }
}
