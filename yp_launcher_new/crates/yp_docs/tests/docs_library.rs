use std::path::Path;

use yp_docs::{DocsLibrary, Excerpt, search_pages};

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn sample_docs() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(
        root,
        "native/02_mod_folder_layout.md",
        "# Mod folder layout\n\nEvery mod lives in nams/mods.\n",
    );
    write(
        root,
        "native/01_first_mod.md",
        "\n\n# Your first mod\n\nStart here.\n",
    );
    write(
        root,
        "native/resources/AreaAction.md",
        "No title line.\n\nAreaAction reference.\n",
    );
    write(root, "native/authoring/10_style.md", "# Style guide\n");
    write(
        root,
        "scripting/03_getting_started.md",
        "Intro without heading.\n",
    );
    write(root, "images/README.md", "# Images\n");
    write(root, "images/a.png", "png");
    write(root, "README.md", "# Root readme\n");
    dir
}

#[test]
fn the_index_lists_category_pages_with_guides_first() {
    let dir = sample_docs();
    let mut library = DocsLibrary::new(dir.path());
    let pages = library.load_index().unwrap().to_vec();
    let paths: Vec<&str> = pages.iter().map(|page| page.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "assets/docs/native/01_first_mod.md",
            "assets/docs/native/02_mod_folder_layout.md",
            "assets/docs/scripting/03_getting_started.md",
            "assets/docs/native/resources/AreaAction.md",
            "assets/docs/native/authoring/10_style.md",
        ]
    );
    let titles: Vec<&str> = pages.iter().map(|page| page.title.as_str()).collect();
    assert_eq!(
        titles,
        vec![
            "Your first mod",
            "Mod folder layout",
            "Getting started",
            "AreaAction",
            "Style guide"
        ]
    );
    assert_eq!(pages[0].category, "native");
    assert_eq!(pages[2].category, "scripting");
    assert_eq!(pages[0].file_name, "01_first_mod.md");
    assert!(pages[3].is_resource());
    assert!(!pages[3].is_authoring());
    assert!(pages[4].is_authoring());
    assert!(!pages[0].is_resource());
}

#[test]
fn links_resolve_against_the_loaded_index() {
    let dir = sample_docs();
    let mut library = DocsLibrary::new(dir.path());
    let from = "assets/docs/scripting/03_getting_started.md";
    assert_eq!(
        library
            .resolve_link(from, "../native/01_first_mod.md#start")
            .as_deref(),
        Some("assets/docs/native/01_first_mod.md")
    );
    library.load_index().unwrap();
    assert_eq!(
        library.resolve_link(from, "01_first_mod.md").as_deref(),
        Some("assets/docs/native/01_first_mod.md")
    );
    assert_eq!(library.resolve_link(from, "nope.md"), None);
}

#[test]
fn search_puts_title_hits_before_body_hits() {
    let dir = sample_docs();
    let mut library = DocsLibrary::new(dir.path());
    let hits = library.search("  MOD ").unwrap();
    let paths: Vec<&str> = hits.iter().map(|hit| hit.page.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "assets/docs/native/01_first_mod.md",
            "assets/docs/native/02_mod_folder_layout.md"
        ]
    );
    assert!(hits.iter().all(|hit| hit.excerpt.is_none()));

    let hits = library.search("reference").unwrap();
    assert_eq!(hits.len(), 1);
    let excerpt = hits[0].excerpt.clone().unwrap();
    assert_eq!(excerpt.text, "No title line. AreaAction reference.");
    assert!(!excerpt.clipped_start && !excerpt.clipped_end);
    assert!(library.search("   ").unwrap().is_empty());
}

#[test]
fn excerpts_are_clipped_around_the_match() {
    let body = format!("{}needle{}", "a ".repeat(50), "b\n".repeat(60));
    let pages = vec![yp_docs::DocPage {
        path: "assets/docs/x/y.md".into(),
        category: "x".into(),
        title: "Other".into(),
        file_name: "y.md".into(),
    }];
    let hits = search_pages(&pages, &[body], "NEEDLE");
    let excerpt = hits[0].excerpt.clone().unwrap();
    assert!(excerpt.clipped_start);
    assert!(excerpt.clipped_end);
    assert!(excerpt.text.contains("needle"));
    assert!(!excerpt.text.contains('\n'));
    let shown = Excerpt {
        text: "x".into(),
        clipped_start: true,
        clipped_end: false,
    };
    assert_eq!(shown.to_string(), "...x");
}

#[test]
fn search_finds_matches_after_characters_that_change_length_when_lowercased() {
    let pages = vec![yp_docs::DocPage {
        path: "assets/docs/x/y.md".into(),
        category: "x".into(),
        title: "Other".into(),
        file_name: "y.md".into(),
    }];
    let hits = search_pages(&pages, &["İİİ Zielwort".to_owned()], "zielwort");
    assert_eq!(hits[0].excerpt.clone().unwrap().text, "İİİ Zielwort");
}

#[test]
fn editing_writes_the_source_and_refreshes_the_index() {
    let dir = sample_docs();
    let mut library = DocsLibrary::new(dir.path()).editable(true);
    assert!(library.can_edit());
    library.load_index().unwrap();
    let path = "assets/docs/native/01_first_mod.md";
    assert!(library.save_source(path, "# Renamed\n").unwrap());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("native").join("01_first_mod.md")).unwrap(),
        "# Renamed\n"
    );
    assert!(library.cached_pages().is_none());
    assert_eq!(library.content(path).unwrap(), "# Renamed\n");
    let titles: Vec<String> = library
        .load_index()
        .unwrap()
        .iter()
        .map(|page| page.title.clone())
        .collect();
    assert!(titles.contains(&"Renamed".to_owned()));
    assert!(
        !library
            .save_source("assets/docs/native/missing.md", "x")
            .unwrap()
    );
}

#[test]
fn a_read_only_library_never_writes() {
    let dir = sample_docs();
    let mut library = DocsLibrary::new(dir.path());
    assert!(!library.can_edit());
    assert!(
        !library
            .save_source("assets/docs/native/01_first_mod.md", "changed")
            .unwrap()
    );
    assert_eq!(
        library.source_file_for("assets/docs/native/01_first_mod.md"),
        None
    );
}

#[test]
fn content_is_cached_until_invalidated() {
    let dir = sample_docs();
    let mut library = DocsLibrary::new(dir.path());
    let path = "assets/docs/native/01_first_mod.md";
    let first = library.content(path).unwrap();
    std::fs::write(dir.path().join("native").join("01_first_mod.md"), "changed").unwrap();
    assert_eq!(library.content(path).unwrap(), first);
    library.invalidate();
    assert_eq!(library.content(path).unwrap(), "changed");
}

#[test]
fn paths_outside_the_docs_root_are_rejected() {
    let dir = sample_docs();
    let mut library = DocsLibrary::new(dir.path());
    assert!(library.content("assets/other/x.md").is_err());
    assert_eq!(library.file_for("assets/docs/../secret.md"), None);
    assert!(library.content("assets/docs/native/none.md").is_err());
}

#[test]
fn the_shipped_docs_index_loads() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/docs");
    if !docs.is_dir() {
        return;
    }
    let mut library = DocsLibrary::new(docs);
    let pages = library.load_index().unwrap().to_vec();
    assert!(pages.len() > 50);
    for page in &pages {
        let markdown = library.content(&page.path).unwrap();
        let anchors: Vec<String> = yp_docs::outline(&markdown)
            .into_iter()
            .map(|heading| heading.anchor)
            .collect();
        let unique: std::collections::BTreeSet<&String> = anchors.iter().collect();
        assert_eq!(unique.len(), anchors.len(), "{}", page.path);
    }
}
