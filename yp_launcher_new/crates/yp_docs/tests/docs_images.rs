use std::path::{Path, PathBuf};

use yp_docs::{DocsMarker, image_sources};

const MAX_BYTES: u64 = 100 * 1024;
const ALLOWED_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

fn docs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/docs")
}

fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect();
    files.sort();
    files
}

fn extension_of(path: &Path) -> String {
    path.extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

#[test]
fn every_docs_image_stays_under_100_kb() {
    let root = docs_dir().join("images");
    if !root.is_dir() {
        return;
    }
    let too_big: Vec<String> = files_under(&root)
        .into_iter()
        .filter(|file| ALLOWED_EXTENSIONS.contains(&extension_of(file).as_str()))
        .filter_map(|file| {
            let bytes = file.metadata().ok()?.len();
            (bytes > MAX_BYTES).then(|| format!("{} is {} KB", file.display(), bytes / 1024))
        })
        .collect();
    assert!(
        too_big.is_empty(),
        "docs images must be 100 KB or less:\n{}",
        too_big.join("\n")
    );
}

#[test]
fn docs_images_use_a_web_friendly_format() {
    let root = docs_dir().join("images");
    if !root.is_dir() {
        return;
    }
    let wrong_type: Vec<String> = files_under(&root)
        .into_iter()
        .filter(|file| {
            let extension = extension_of(file);
            !extension.is_empty()
                && extension != "md"
                && !ALLOWED_EXTENSIONS.contains(&extension.as_str())
        })
        .map(|file| file.display().to_string())
        .collect();
    assert!(
        wrong_type.is_empty(),
        "docs images must be .jpg, .png or .webp:\n{}",
        wrong_type.join("\n")
    );
}

#[test]
fn every_image_referenced_by_a_doc_page_exists_and_no_page_uses_yp_figure() {
    let docs = docs_dir();
    if !docs.is_dir() {
        return;
    }
    let mut missing = Vec::new();
    let mut figure_markers = Vec::new();
    for file in files_under(&docs)
        .into_iter()
        .filter(|file| extension_of(file) == "md")
    {
        let text = std::fs::read_to_string(&file).unwrap();
        if DocsMarker::all_in(&text)
            .iter()
            .any(|marker| marker.id == "figure")
        {
            figure_markers.push(file.display().to_string());
        }
        for source in image_sources(&text) {
            if source.starts_with("http") {
                continue;
            }
            let target = file.parent().unwrap().join(&source);
            if !target.exists() {
                missing.push(format!("{} -> {source}", file.display()));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "broken image references:\n{}",
        missing.join("\n")
    );
    assert!(
        figure_markers.is_empty(),
        "yp:figure is retired; use a plain markdown image instead:\n{}",
        figure_markers.join("\n")
    );
}

#[test]
fn image_sources_skip_code_blocks() {
    let markdown =
        "![a](images/a.png)\n\n```\n![b](images/b.png)\n```\n\nText ![c](../c.jpg \"title\")\n";
    assert_eq!(image_sources(markdown), vec!["images/a.png", "../c.jpg"]);
}
