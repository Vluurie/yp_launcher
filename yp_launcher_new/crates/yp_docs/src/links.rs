use crate::index::DocPage;

const EXTERNAL_SCHEMES: [&str; 2] = ["http://", "https://"];

pub fn is_external(href: &str) -> bool {
    EXTERNAL_SCHEMES
        .iter()
        .any(|scheme| href.starts_with(scheme))
}

pub fn normalize_relative(from_path: &str, target: &str) -> String {
    let mut base: Vec<&str> = from_path.split('/').collect();
    base.pop();
    let mut stack: Vec<&str> = Vec::new();
    for segment in base.into_iter().chain(target.split('/')) {
        match segment {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            other => stack.push(other),
        }
    }
    stack.join("/")
}

pub fn resolve_link(pages: Option<&[DocPage]>, from_path: &str, href: &str) -> Option<String> {
    if is_external(href) {
        return None;
    }
    let target = href.split('#').next().unwrap_or_default();
    if target.is_empty() {
        return None;
    }
    let resolved = normalize_relative(from_path, target);
    let Some(pages) = pages else {
        return Some(resolved);
    };
    if let Some(page) = pages.iter().find(|page| page.path == resolved) {
        return Some(page.path.clone());
    }
    let file_name = target.rsplit('/').next().unwrap_or(target);
    pages
        .iter()
        .find(|page| page.file_name == file_name)
        .map(|page| page.path.clone())
}
