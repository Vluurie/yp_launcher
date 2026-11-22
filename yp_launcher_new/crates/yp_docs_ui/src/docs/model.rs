use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use gpui::{App, AppContext as _, Context, EventEmitter, Global, Task};
use yp_docs::links::{is_external, normalize_relative};
use yp_docs::{DocHeading, DocPage, DocSearchHit, DocsLibrary, anchor_in, outline, resolve_link};
use yp_state::prelude::*;

use crate::markdown::{Block, Document, Figure, parse_document};

pub const CATEGORY_NATIVE: &str = "native";
pub const CATEGORY_SCRIPTING: &str = "scripting";
pub const FIRST_MOD_PAGE: &str = "01_first_mod.md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsConfig {
    pub docs_dir: PathBuf,
    pub editable: bool,
}

impl Global for DocsConfig {}

impl DocsConfig {
    pub fn detect(cx: &App) -> Self {
        let paths = &cx.env().paths;
        let beside_launcher = paths.docs_dir();
        let docs_dir = if beside_launcher.is_dir() {
            beside_launcher
        } else {
            let unpacked = paths.data_dir.join(super::embedded::UNPACKED_DIR_NAME);
            match super::embedded::unpack_embedded(&unpacked) {
                Ok(Some(dir)) => dir,
                Ok(None) => beside_launcher,
                Err(error) => {
                    log::warn!("could not unpack the embedded docs to {}: {error}", unpacked.display());
                    beside_launcher
                }
            }
        };
        Self {
            docs_dir,
            editable: cfg!(debug_assertions),
        }
    }

    pub fn current(cx: &App) -> Self {
        cx.try_global::<DocsConfig>()
            .cloned()
            .unwrap_or_else(|| Self::detect(cx))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    External(String),
    Anchor(String),
    Page {
        page: DocPage,
        anchor: Option<String>,
    },
    Unknown,
}

pub fn link_target(pages: &[DocPage], from_path: Option<&str>, href: &str) -> LinkTarget {
    if is_external(href) {
        return LinkTarget::External(href.to_owned());
    }
    if let Some(anchor) = href.strip_prefix('#') {
        return if anchor.is_empty() {
            LinkTarget::Unknown
        } else {
            LinkTarget::Anchor(anchor.to_owned())
        };
    }
    let Some(from_path) = from_path else {
        return LinkTarget::Unknown;
    };
    let Some(resolved) = resolve_link(Some(pages), from_path, href) else {
        return LinkTarget::Unknown;
    };
    match pages.iter().find(|page| page.path == resolved) {
        Some(page) => LinkTarget::Page {
            page: page.clone(),
            anchor: anchor_in(href).map(str::to_owned),
        },
        None => LinkTarget::Unknown,
    }
}

pub fn find_by_file_name<'a>(pages: &'a [DocPage], file_name: &str) -> Option<&'a DocPage> {
    let suffix = format!("/{file_name}");
    pages
        .iter()
        .find(|page| page.path.ends_with(&suffix) && !page.is_resource())
        .or_else(|| pages.iter().find(|page| page.path.ends_with(&suffix)))
        .or_else(|| pages.iter().find(|page| page.file_name == file_name))
}

pub fn requested_page<'a>(pages: &'a [DocPage], requested: &str) -> Option<&'a DocPage> {
    let normalized = requested.replace('\\', "/");
    let trimmed = normalized.trim_start_matches("./").trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let file_name = trimmed.rsplit('/').next().unwrap_or(trimmed);
    pages
        .iter()
        .find(|page| page.path == trimmed)
        .or_else(|| find_by_file_name(pages, trimmed))
        .or_else(|| find_by_file_name(pages, file_name))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavGroup {
    Authoring,
    Resources,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavEntry {
    Header(NavGroup),
    Page(DocPage),
}

pub fn nav_entries(pages: &[DocPage], category: &str) -> Vec<NavEntry> {
    let mut entries = Vec::new();
    let mut resources_started = false;
    let mut authoring_started = false;
    for page in pages.iter().filter(|page| page.category == category) {
        if page.is_authoring() && !authoring_started {
            authoring_started = true;
            entries.push(NavEntry::Header(NavGroup::Authoring));
        } else if page.is_resource() && !resources_started {
            resources_started = true;
            entries.push(NavEntry::Header(NavGroup::Resources));
        }
        entries.push(NavEntry::Page(page.clone()));
    }
    entries
}

fn image_sources(blocks: &[Block], sources: &mut Vec<String>) {
    for block in blocks {
        match block {
            Block::Image { source, .. } => sources.push(source.clone()),
            Block::Quote(children) => image_sources(children, sources),
            Block::List(list) => {
                for item in &list.items {
                    image_sources(&item.blocks, sources);
                }
            }
            _ => {}
        }
    }
}

fn figures_of(library: &DocsLibrary, page_path: &str, document: &Document) -> HashMap<String, Figure> {
    let mut sources = Vec::new();
    image_sources(&document.blocks, &mut sources);
    let mut figures = HashMap::new();
    for source in sources {
        let asset = normalize_relative(page_path, &source);
        let Some(path) = library.file_for(&asset) else {
            continue;
        };
        let Ok((width, height)) = image::image_dimensions(&path) else {
            continue;
        };
        figures.insert(
            source,
            Figure {
                path,
                width,
                height,
            },
        );
    }
    figures
}

#[derive(Debug, Clone)]
pub struct OpenPage {
    pub page: DocPage,
    pub markdown: String,
    pub document: Arc<Document>,
    pub outline: Vec<DocHeading>,
    pub figures: Arc<HashMap<String, Figure>>,
    pub source_file: Option<PathBuf>,
}

fn lock(library: &Mutex<DocsLibrary>) -> MutexGuard<'_, DocsLibrary> {
    library
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn load_page(library: &Mutex<DocsLibrary>, page: &DocPage) -> Result<OpenPage, String> {
    let mut library = lock(library);
    let markdown = library
        .content(&page.path)
        .map_err(|error| error.to_string())?;
    let document = parse_document(&markdown);
    let figures = figures_of(&library, &page.path, &document);
    Ok(OpenPage {
        page: page.clone(),
        outline: outline(&markdown),
        source_file: library.source_file_for(&page.path),
        document: Arc::new(document),
        figures: Arc::new(figures),
        markdown,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocsEvent {
    PageOpened { anchor: Option<String> },
    PageReloaded,
    WentHome,
    Failed(DocsFailure),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocsFailure {
    Page(String),
    Save(String),
    BrokenLink(String),
}

pub struct DocsModel {
    library: Arc<Mutex<DocsLibrary>>,
    editable: bool,
    single_page: bool,
    pages: Vec<DocPage>,
    loading: bool,
    load_error: Option<String>,
    show_home: bool,
    active_category: String,
    current: Option<OpenPage>,
    history: Vec<String>,
    query: String,
    hits: Vec<DocSearchHit>,
    generation: u64,
    load_task: Option<Task<()>>,
    open_task: Option<Task<()>>,
    search_task: Option<Task<()>>,
}

impl EventEmitter<DocsEvent> for DocsModel {}

impl DocsModel {
    pub fn new(
        config: DocsConfig,
        initial_page: Option<String>,
        single_page: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let library = DocsLibrary::new(config.docs_dir).editable(config.editable);
        let editable = library.can_edit();
        let mut this = Self {
            library: Arc::new(Mutex::new(library)),
            editable,
            single_page,
            pages: Vec::new(),
            loading: true,
            load_error: None,
            show_home: true,
            active_category: CATEGORY_NATIVE.to_owned(),
            current: None,
            history: Vec::new(),
            query: String::new(),
            hits: Vec::new(),
            generation: 0,
            load_task: None,
            open_task: None,
            search_task: None,
        };
        this.load(initial_page, cx);
        this
    }

    pub fn pages(&self) -> &[DocPage] {
        &self.pages
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn shows_home(&self) -> bool {
        self.show_home
    }

    pub fn is_single_page(&self) -> bool {
        self.single_page
    }

    pub fn can_edit(&self) -> bool {
        self.editable
    }

    pub fn active_category(&self) -> &str {
        &self.active_category
    }

    pub fn current(&self) -> Option<&OpenPage> {
        self.current.as_ref()
    }

    pub fn current_path(&self) -> Option<&str> {
        self.current.as_ref().map(|open| open.page.path.as_str())
    }

    pub fn has_history(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn is_searching(&self) -> bool {
        !self.query.trim().is_empty()
    }

    pub fn hits(&self) -> &[DocSearchHit] {
        &self.hits
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    fn load(&mut self, initial_page: Option<String>, cx: &mut Context<Self>) {
        let library = self.library.clone();
        let job = cx.background_spawn(async move {
            lock(&library)
                .load_index()
                .map(<[DocPage]>::to_vec)
                .map_err(|error| error.to_string())
        });
        self.load_task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| {
                this.loading = false;
                match result {
                    Ok(pages) => {
                        this.pages = pages;
                        this.load_error = None;
                    }
                    Err(error) => {
                        this.pages = Vec::new();
                        this.load_error = Some(error);
                    }
                }
                let target = initial_page
                    .and_then(|path| requested_page(&this.pages, &path).cloned());
                if let Some(page) = target {
                    this.open(page, None, false, cx);
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn open(
        &mut self,
        page: DocPage,
        anchor: Option<String>,
        push_history: bool,
        cx: &mut Context<Self>,
    ) {
        let library = self.library.clone();
        let target = page.clone();
        let job = cx.background_spawn(async move { load_page(&library, &target) });
        self.open_task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| match result {
                Ok(open) => {
                    if push_history && let Some(previous) = &this.current {
                        this.history.push(previous.page.path.clone());
                    }
                    this.show_home = false;
                    this.active_category = open.page.category.clone();
                    this.current = Some(open);
                    this.generation += 1;
                    cx.emit(DocsEvent::PageOpened { anchor });
                    cx.notify();
                }
                Err(error) => cx.emit(DocsEvent::Failed(DocsFailure::Page(error))),
            })
            .ok();
        }));
    }

    pub fn open_file_name(&mut self, file_name: &str, cx: &mut Context<Self>) {
        if let Some(page) = find_by_file_name(&self.pages, file_name).cloned() {
            self.open(page, None, true, cx);
        }
    }

    pub fn go_home(&mut self, cx: &mut Context<Self>) {
        self.open_task = None;
        self.show_home = true;
        self.current = None;
        self.history.clear();
        self.generation += 1;
        cx.emit(DocsEvent::WentHome);
        cx.notify();
    }

    pub fn go_back(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        let page = self
            .pages
            .iter()
            .find(|page| page.path == previous)
            .or_else(|| self.pages.first())
            .cloned();
        match page {
            Some(page) => {
                self.open(page, None, false, cx);
                true
            }
            None => false,
        }
    }

    pub fn link_target(&self, href: &str) -> LinkTarget {
        link_target(&self.pages, self.current_path(), href)
    }

    pub fn follow(&mut self, target: &LinkTarget, cx: &mut Context<Self>) {
        if let LinkTarget::Page { page, anchor } = target {
            self.open(page.clone(), anchor.clone(), true, cx);
        }
    }

    pub fn report_broken_link(&mut self, href: &str, cx: &mut Context<Self>) {
        cx.emit(DocsEvent::Failed(DocsFailure::BrokenLink(href.to_owned())));
    }

    pub fn set_category(&mut self, category: &str, cx: &mut Context<Self>) {
        if self.active_category != category {
            self.active_category = category.to_owned();
            cx.notify();
        }
    }

    pub fn set_query(&mut self, query: &str, cx: &mut Context<Self>) {
        if self.query == query {
            return;
        }
        self.query = query.to_owned();
        cx.notify();
        if query.trim().is_empty() {
            self.hits.clear();
            self.search_task = None;
            return;
        }
        let library = self.library.clone();
        let needle = query.to_owned();
        let job = cx.background_spawn(async move {
            lock(&library).search(&needle).unwrap_or_default()
        });
        self.search_task = Some(cx.spawn(async move |this, cx| {
            let hits = job.await;
            this.update(cx, |this, cx| {
                this.hits = hits;
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn reload_current(&mut self, cx: &mut Context<Self>) {
        let Some(page) = self.current.as_ref().map(|open| open.page.clone()) else {
            return;
        };
        let library = self.library.clone();
        let job = cx.background_spawn(async move {
            lock(&library).invalidate();
            let open = load_page(&library, &page);
            let pages = lock(&library).load_index().map(<[DocPage]>::to_vec).ok();
            (open, pages)
        });
        self.open_task = Some(cx.spawn(async move |this, cx| {
            let (open, pages) = job.await;
            this.update(cx, |this, cx| this.finish_reload(open, pages, cx))
                .ok();
        }));
    }

    pub fn save_current(&mut self, content: String, cx: &mut Context<Self>) {
        let Some(page) = self.current.as_ref().map(|open| open.page.clone()) else {
            return;
        };
        let library = self.library.clone();
        let job = cx.background_spawn(async move {
            let saved = lock(&library)
                .save_source(&page.path, &content)
                .map_err(|error| error.to_string());
            let open = load_page(&library, &page);
            let pages = lock(&library).load_index().map(<[DocPage]>::to_vec).ok();
            (saved, open, pages)
        });
        self.open_task = Some(cx.spawn(async move |this, cx| {
            let (saved, open, pages) = job.await;
            this.update(cx, |this, cx| {
                if let Err(error) = saved {
                    cx.emit(DocsEvent::Failed(DocsFailure::Save(error)));
                }
                this.finish_reload(open, pages, cx);
            })
            .ok();
        }));
    }

    fn finish_reload(
        &mut self,
        open: Result<OpenPage, String>,
        pages: Option<Vec<DocPage>>,
        cx: &mut Context<Self>,
    ) {
        if let Some(pages) = pages {
            self.pages = pages;
        }
        match open {
            Ok(mut open) => {
                if let Some(refreshed) = self.pages.iter().find(|page| page.path == open.page.path)
                {
                    open.page = refreshed.clone();
                }
                self.current = Some(open);
                cx.emit(DocsEvent::PageReloaded);
            }
            Err(error) => cx.emit(DocsEvent::Failed(DocsFailure::Page(error))),
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::tests::docs_dir;

    fn page(path: &str, title: &str) -> DocPage {
        let relative = path.strip_prefix("assets/docs/").expect("asset path");
        DocPage {
            path: path.to_owned(),
            category: relative.split('/').next().expect("category").to_owned(),
            title: title.to_owned(),
            file_name: relative.rsplit('/').next().expect("file").to_owned(),
        }
    }

    fn sample_pages() -> Vec<DocPage> {
        vec![
            page("assets/docs/native/01_first_mod.md", "Your first mod"),
            page("assets/docs/native/03_item.md", "Items"),
            page("assets/docs/native/README.md", "Modding guide"),
            page("assets/docs/native/resources/vanilla_item_ids.md", "Item ids"),
            page("assets/docs/native/resources/README.md", "Tables"),
            page("assets/docs/scripting/README.md", "Scripting"),
            page("assets/docs/scripting/01_hap.md", "HAP"),
            page("assets/docs/scripting/authoring/tools.md", "Tools"),
        ]
    }

    #[test]
    fn links_are_classified_as_external_anchor_page_or_unknown() {
        let pages = sample_pages();
        let from = Some("assets/docs/native/01_first_mod.md");
        assert_eq!(
            link_target(&pages, from, "https://example.com/a#b"),
            LinkTarget::External("https://example.com/a#b".into())
        );
        assert_eq!(
            link_target(&pages, from, "#the-file"),
            LinkTarget::Anchor("the-file".into())
        );
        assert_eq!(link_target(&pages, from, "#"), LinkTarget::Unknown);
        assert_eq!(
            link_target(&pages, from, "03_item.md#fields"),
            LinkTarget::Page {
                page: pages[1].clone(),
                anchor: Some("fields".into()),
            }
        );
        assert_eq!(
            link_target(&pages, from, "resources/vanilla_item_ids.md"),
            LinkTarget::Page {
                page: pages[3].clone(),
                anchor: None,
            }
        );
        assert_eq!(
            link_target(&pages, from, "../scripting/01_hap.md"),
            LinkTarget::Page {
                page: pages[6].clone(),
                anchor: None,
            }
        );
        assert_eq!(link_target(&pages, from, "missing.md"), LinkTarget::Unknown);
        assert_eq!(link_target(&pages, None, "03_item.md"), LinkTarget::Unknown);
    }

    #[test]
    fn links_fall_back_to_the_file_name_when_the_folder_is_wrong() {
        let pages = sample_pages();
        assert_eq!(
            link_target(
                &pages,
                Some("assets/docs/scripting/01_hap.md"),
                "03_item.md"
            ),
            LinkTarget::Page {
                page: pages[1].clone(),
                anchor: None,
            }
        );
    }

    #[test]
    fn guides_win_over_reference_tables_when_a_file_name_repeats() {
        let pages = sample_pages();
        assert_eq!(
            find_by_file_name(&pages, "README.md").map(|page| page.path.as_str()),
            Some("assets/docs/native/README.md")
        );
        assert_eq!(
            find_by_file_name(&pages, "scripting/README.md").map(|page| page.path.as_str()),
            Some("assets/docs/scripting/README.md")
        );
        assert_eq!(
            find_by_file_name(&pages, "vanilla_item_ids.md").map(|page| page.title.as_str()),
            Some("Item ids")
        );
        assert_eq!(find_by_file_name(&pages, "nope.md"), None);
    }

    #[test]
    fn a_requested_page_is_found_by_asset_path_docs_path_or_file_name() {
        let pages = sample_pages();
        let path = |requested: &str| requested_page(&pages, requested).map(|page| page.path.as_str());
        assert_eq!(path("assets/docs/native/03_item.md"), Some("assets/docs/native/03_item.md"));
        assert_eq!(path("docs/native/03_item.md"), Some("assets/docs/native/03_item.md"));
        assert_eq!(path("docs\\native\\03_item.md"), Some("assets/docs/native/03_item.md"));
        assert_eq!(path("/native/03_item.md"), Some("assets/docs/native/03_item.md"));
        assert_eq!(path("03_item.md"), Some("assets/docs/native/03_item.md"));
        assert_eq!(path("docs/elsewhere/03_item.md"), Some("assets/docs/native/03_item.md"));
        assert_eq!(path("docs/scripting/README.md"), Some("assets/docs/scripting/README.md"));
        assert_eq!(path("docs/native/README.md"), Some("assets/docs/native/README.md"));
        assert_eq!(path("docs/native/missing.md"), None);
        assert_eq!(path(""), None);
        assert_eq!(path("/"), None);
    }

    #[test]
    fn the_nav_lists_one_category_with_group_headers() {
        let pages = sample_pages();
        let native = nav_entries(&pages, CATEGORY_NATIVE);
        assert_eq!(native.len(), 6);
        assert_eq!(native[3], NavEntry::Header(NavGroup::Resources));
        assert!(matches!(&native[4], NavEntry::Page(page) if page.title == "Item ids"));
        let scripting = nav_entries(&pages, CATEGORY_SCRIPTING);
        assert_eq!(scripting[2], NavEntry::Header(NavGroup::Authoring));
        assert_eq!(scripting.len(), 4);
        assert!(nav_entries(&pages, "other").is_empty());
    }

    #[test]
    fn every_internal_link_in_the_bundled_docs_resolves() {
        let mut library = DocsLibrary::new(docs_dir());
        let pages = library.load_index().expect("index").to_vec();
        assert!(pages.len() > 50);
        let mut checked = 0;
        let mut broken = Vec::new();
        for page in &pages {
            let markdown = library.content(&page.path).expect("content");
            let document = parse_document(&markdown);
            let anchors: Vec<String> = document.anchors().into_iter().map(str::to_owned).collect();
            let mut hrefs = Vec::new();
            collect_links(&document.blocks, &mut hrefs);
            for href in hrefs {
                checked += 1;
                match link_target(&pages, Some(&page.path), &href) {
                    LinkTarget::External(_) | LinkTarget::Page { .. } => {}
                    LinkTarget::Anchor(anchor) => {
                        if !anchors.contains(&anchor) {
                            broken.push(format!("{} -> #{anchor}", page.path));
                        }
                    }
                    LinkTarget::Unknown => broken.push(format!("{} -> {href}", page.path)),
                }
            }
        }
        assert!(checked > 100, "only {checked} links found");
        assert!(broken.is_empty(), "broken links:\n{}", broken.join("\n"));
    }

    fn collect_links(blocks: &[Block], hrefs: &mut Vec<String>) {
        for block in blocks {
            match block {
                Block::Paragraph(text) | Block::Heading { text, .. } => {
                    hrefs.extend(text.links.iter().cloned());
                }
                Block::Table(table) => {
                    for cell in table.head.iter().chain(table.rows.iter().flatten()) {
                        hrefs.extend(cell.links.iter().cloned());
                    }
                }
                Block::Quote(children) => collect_links(children, hrefs),
                Block::List(list) => {
                    for item in &list.items {
                        collect_links(&item.blocks, hrefs);
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn a_page_loads_with_its_outline_figures_and_tools() {
        let library = Mutex::new(DocsLibrary::new(docs_dir()));
        let pages = lock(&library).load_index().expect("index").to_vec();
        let first = find_by_file_name(&pages, FIRST_MOD_PAGE).expect("first mod page");
        let open = load_page(&library, first).expect("page");
        assert_eq!(open.page.path, first.path);
        assert_eq!(
            open.document
                .tools()
                .iter()
                .map(|tool| tool.id.as_str())
                .collect::<Vec<_>>(),
            vec!["first-mod"]
        );
        assert_eq!(open.outline.len(), open.document.anchors().len());
        assert!(open.source_file.is_none());

        let with_images = pages
            .iter()
            .filter_map(|page| load_page(&library, page).ok())
            .find(|open| !open.figures.is_empty())
            .expect("a page with a figure");
        for figure in with_images.figures.values() {
            assert!(figure.path.is_file());
            assert!(figure.width > 0 && figure.height > 0);
        }
    }

    #[test]
    fn editing_is_only_offered_when_the_library_allows_it() {
        let dir = tempfile::tempdir().expect("temp dir");
        let native = dir.path().join("native");
        std::fs::create_dir_all(&native).expect("mkdir");
        std::fs::write(native.join("a.md"), "# A\n\ntext\n").expect("write");

        let locked = Mutex::new(DocsLibrary::new(dir.path()));
        let page = lock(&locked).load_index().expect("index")[0].clone();
        assert!(load_page(&locked, &page).expect("page").source_file.is_none());
        assert!(!lock(&locked).save_source(&page.path, "# B\n").expect("save"));

        let open = Mutex::new(DocsLibrary::new(dir.path()).editable(true));
        lock(&open).load_index().expect("index");
        assert!(load_page(&open, &page).expect("page").source_file.is_some());
        assert!(lock(&open).save_source(&page.path, "# B\n\nnew\n").expect("save"));
        let reloaded = load_page(&open, &page).expect("page");
        assert_eq!(reloaded.markdown, "# B\n\nnew\n");
        assert_eq!(lock(&open).load_index().expect("index")[0].title, "B");
    }
}
