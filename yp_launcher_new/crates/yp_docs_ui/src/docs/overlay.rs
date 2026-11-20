#[cfg(feature = "tools")]
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Context, Entity, FocusHandle, Focusable, FontWeight,
    IntoElement, Pixels, Render, ScrollHandle, SharedString, Subscription, Task, TextRun, Window,
    div, prelude::*, px,
};
use yp_docs::{DocHeading, pretty_name};
use yp_i18n::Strings;
use yp_state::prelude::*;
use yp_state::{OverlayId, OverlayRequest};
use yp_ui::prelude::*;
use yp_ui::{
    ActiveTheme, ChipTone, InputEvent, ListItemSpacing, Toast, ToggleOption, Tooltip, WindowExt,
    easing,
};

use super::home::{BrowseAll, OpenFile, render_home};
use super::model::{
    CATEGORY_NATIVE, CATEGORY_SCRIPTING, DocsConfig, DocsEvent, DocsFailure, DocsModel,
    LinkTarget, NavEntry, NavGroup, OpenPage, nav_entries,
};
#[cfg(feature = "tools")]
use super::tool_panel::ToolPanel;
#[cfg(feature = "tools")]
use crate::builders::ToolKind;
use crate::dialogs::markdown_editor::{MarkdownEditorOptions, edit_markdown};
#[cfg(feature = "tools")]
use crate::markdown::ToolMarker;
use crate::markdown::{AnchorBounds, Figure, MarkdownContext, render_document};
use crate::util::path_text;

const CONTENT_MAX_WIDTH: f32 = 900.;
const OUTLINE_MIN_HEADINGS: usize = 3;
const ANCHOR_RETRIES: u8 = 4;
const ANCHOR_MARGIN: f32 = 0.05;
const SCROLL_STEPS: u16 = 14;
const SCROLL_FRAME: Duration = Duration::from_millis(16);
const PAGE_FADE: Duration = Duration::from_millis(260);
const SWITCH_SIZES: [ButtonSize; 3] = [ButtonSize::Medium, ButtonSize::Default, ButtonSize::Compact];
const SWITCH_BORDER: f32 = 2.;
const NAV_LEADING_ROWS: usize = 2;

pub fn nav_row_of(entries: &[NavEntry], path: &str) -> Option<usize> {
    entries
        .iter()
        .position(|entry| matches!(entry, NavEntry::Page(page) if page.path == path))
        .map(|index| index + NAV_LEADING_ROWS)
}

pub fn first_fitting<T: Copy>(candidates: &[(T, Pixels)], available: Pixels) -> Option<T> {
    candidates
        .iter()
        .find(|(_, width)| *width <= available)
        .map(|(candidate, _)| *candidate)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchFit {
    pub size: ButtonSize,
    pub equal_widths: bool,
}

pub fn switch_candidates(size: ButtonSize, widths: &[Pixels]) -> [(SwitchFit, Pixels); 2] {
    let widest = widths.iter().fold(px(0.), |widest, width| widest.max(*width));
    let total = widths.iter().fold(px(0.), |total, width| total + *width);
    [
        (
            SwitchFit {
                size,
                equal_widths: true,
            },
            widest * widths.len() as f32,
        ),
        (
            SwitchFit {
                size,
                equal_widths: false,
            },
            total,
        ),
    ]
}

fn switch_fit(
    labels: &[SharedString],
    available: Pixels,
    window: &Window,
    cx: &App,
) -> Option<SwitchFit> {
    let sizes = window.sizes(cx);
    let mut font = window.text_style().font();
    font.weight = FontWeight::BOLD;
    let color = cx.theme().colors.text_primary;
    let candidates: Vec<(SwitchFit, Pixels)> = SWITCH_SIZES
        .into_iter()
        .flat_map(|size| {
            let (padding, _) = size.padding(&sizes);
            let font_size = size.label_size().pixels(&sizes);
            let widths: Vec<Pixels> = labels
                .iter()
                .map(|label| {
                    let run = TextRun {
                        len: label.len(),
                        font: font.clone(),
                        color,
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    let text = window
                        .text_system()
                        .shape_line(label.clone(), font_size, &[run], None);
                    text.width + padding * 2. + px(SWITCH_BORDER)
                })
                .collect();
            switch_candidates(size, &widths)
        })
        .collect();
    first_fitting(&candidates, available)
}

pub fn category_label(category: &str, strings: &Strings) -> SharedString {
    match category {
        CATEGORY_NATIVE => SharedString::new_static(strings.docs_category_native()),
        CATEGORY_SCRIPTING => SharedString::new_static(strings.docs_category_scripting()),
        other => SharedString::from(pretty_name(other)),
    }
}

pub fn shows_outline(outline: &[DocHeading]) -> bool {
    outline.len() >= OUTLINE_MIN_HEADINGS
}

pub fn anchor_scroll_target(
    current: Pixels,
    heading_top: Pixels,
    viewport_top: Pixels,
    viewport_height: Pixels,
    max_offset: Pixels,
) -> Pixels {
    let wanted = current - (heading_top - viewport_top) + viewport_height * ANCHOR_MARGIN;
    wanted.max(-max_offset).min(px(0.))
}

pub struct DocsOverlay {
    model: Entity<DocsModel>,
    search: Entity<InputState>,
    focus_handle: FocusHandle,
    nav_scroll: ScrollHandle,
    content_scroll: ScrollHandle,
    home_scroll: ScrollHandle,
    outline_scroll: ScrollHandle,
    anchors: AnchorBounds,
    pending_anchor: Option<(String, u8)>,
    #[cfg(feature = "tools")]
    tools: HashMap<String, Entity<ToolPanel>>,
    scroll_task: Option<Task<()>>,
    edit_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl DocsOverlay {
    pub fn new(
        initial_page: Option<String>,
        single_page: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = DocsConfig::current(cx);
        let model = cx.new(|cx| DocsModel::new(config, initial_page, single_page, cx));
        let placeholder = cx.t().docs_search_hint();
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
        let subscriptions = vec![
            cx.observe(&model, |_, _, cx| cx.notify()),
            cx.subscribe_in(&model, window, Self::on_docs_event),
            cx.subscribe(&search, |this, search, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let query = search.read(cx).value().to_owned();
                    this.model
                        .update(cx, |model, cx| model.set_query(&query, cx));
                }
            }),
        ];
        Self {
            model,
            search,
            focus_handle: cx.focus_handle(),
            nav_scroll: ScrollHandle::new(),
            content_scroll: ScrollHandle::new(),
            home_scroll: ScrollHandle::new(),
            outline_scroll: ScrollHandle::new(),
            anchors: AnchorBounds::default(),
            pending_anchor: None,
            #[cfg(feature = "tools")]
            tools: HashMap::new(),
            scroll_task: None,
            edit_task: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn from_request(request: &OverlayRequest, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let page = match request {
            OverlayRequest::Docs { page } => page.clone(),
            OverlayRequest::Inspector { .. } => None,
        };
        Self::new(page, !cfg!(feature = "library"), window, cx)
    }

    pub fn model(&self) -> &Entity<DocsModel> {
        &self.model
    }

    pub fn search(&self) -> &Entity<InputState> {
        &self.search
    }

    pub fn content_scroll(&self) -> &ScrollHandle {
        &self.content_scroll
    }

    pub fn panel_heights(&self) -> [Pixels; 4] {
        [
            self.nav_scroll.bounds().size.height,
            self.content_scroll.bounds().size.height,
            self.outline_scroll.bounds().size.height,
            self.home_scroll.bounds().size.height,
        ]
    }

    pub fn anchor_bounds(&self, anchor: &str) -> Option<gpui::Bounds<Pixels>> {
        self.anchors.borrow().get(anchor).copied()
    }

    pub fn set_query(&mut self, query: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = query.to_owned();
        self.search
            .update(cx, |search, cx| search.set_value(value, window, cx));
        self.model
            .update(cx, |model, cx| model.set_query(query, cx));
    }

    #[cfg(feature = "tools")]
    pub fn tool(&self, kind: ToolKind) -> Option<&Entity<ToolPanel>> {
        self.tools.get(kind.marker_id())
    }

    fn on_docs_event(
        &mut self,
        _: &Entity<DocsModel>,
        event: &DocsEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            DocsEvent::PageOpened { anchor } => {
                self.scroll_task = None;
                self.anchors.borrow_mut().clear();
                #[cfg(feature = "tools")]
                self.tools.clear();
                self.content_scroll.set_offset(gpui::point(px(0.), px(0.)));
                self.pending_anchor = anchor.clone().map(|anchor| (anchor, ANCHOR_RETRIES));
                self.reveal_current_in_nav(cx);
            }
            DocsEvent::PageReloaded => self.anchors.borrow_mut().clear(),
            DocsEvent::WentHome => {
                self.scroll_task = None;
                #[cfg(feature = "tools")]
                self.tools.clear();
                self.anchors.borrow_mut().clear();
                self.pending_anchor = None;
            }
            DocsEvent::Failed(failure) => {
                let strings = cx.t();
                let text = match failure {
                    DocsFailure::Page(detail) => strings.docs_page_failed(detail),
                    DocsFailure::Save(detail) => strings.docs_save_failed(detail),
                    DocsFailure::BrokenLink(target) => strings.docs_link_broken(target),
                };
                window.push_toast(Toast::warning(text).id("docs-failure"), cx);
            }
        }
        cx.notify();
    }

    fn reveal_current_in_nav(&mut self, cx: &mut Context<Self>) {
        let model = self.model.read(cx);
        if model.is_searching() {
            return;
        }
        let Some(current) = model.current_path() else {
            return;
        };
        let entries = nav_entries(model.pages(), model.active_category());
        if let Some(row) = nav_row_of(&entries, current) {
            self.nav_scroll.scroll_to_item(row);
        }
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        let navigation = cx.stores().navigation.clone();
        navigation.update(cx, |navigation, cx| {
            navigation.close_overlay(OverlayId::Docs, cx);
        });
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        let (has_history, single_page) = {
            let model = self.model.read(cx);
            (model.has_history(), model.is_single_page())
        };
        if has_history {
            self.model.update(cx, |model, cx| {
                model.go_back(cx);
            });
        } else if single_page {
            self.close(cx);
        } else {
            self.model.update(cx, |model, cx| model.go_home(cx));
        }
    }

    pub fn follow_link(&mut self, href: &str, cx: &mut Context<Self>) {
        let target = self.model.read(cx).link_target(href);
        match &target {
            LinkTarget::External(url) => yp_ui::open_link(url, cx),
            LinkTarget::Anchor(anchor) => {
                self.scroll_to_anchor(anchor, true, cx);
            }
            LinkTarget::Page { .. } => self.model.update(cx, |model, cx| model.follow(&target, cx)),
            LinkTarget::Unknown => {
                self.model
                    .update(cx, |model, cx| model.report_broken_link(href, cx));
            }
        }
    }

    pub fn scroll_to_anchor(&mut self, anchor: &str, animate: bool, cx: &mut Context<Self>) -> bool {
        let Some(heading) = self.anchors.borrow().get(anchor).copied() else {
            return false;
        };
        let viewport = self.content_scroll.bounds();
        let offset = self.content_scroll.offset();
        let target = anchor_scroll_target(
            offset.y,
            heading.top(),
            viewport.top(),
            viewport.size.height,
            self.content_scroll.max_offset().y,
        );
        if !animate {
            self.scroll_task = None;
            self.content_scroll.set_offset(gpui::point(offset.x, target));
            cx.notify();
            return true;
        }
        let from = offset.y;
        self.scroll_task = Some(cx.spawn(async move |this, cx| {
            for step in 1..=SCROLL_STEPS {
                cx.background_executor().timer(SCROLL_FRAME).await;
                let progress = easing::nier_fill(f32::from(step) / f32::from(SCROLL_STEPS));
                let updated = this.update(cx, |this, cx| {
                    let mut offset = this.content_scroll.offset();
                    offset.y = from + (target - from) * progress;
                    this.content_scroll.set_offset(offset);
                    cx.notify();
                });
                if updated.is_err() {
                    return;
                }
            }
        }));
        true
    }

    fn resolve_pending_anchor(&mut self, cx: &mut Context<Self>) {
        let Some((anchor, retries)) = self.pending_anchor.take() else {
            return;
        };
        if self.scroll_to_anchor(&anchor, false, cx) || retries == 0 {
            return;
        }
        self.pending_anchor = Some((anchor, retries - 1));
        cx.notify();
    }

    fn edit_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(open) = self.model.read(cx).current().cloned() else {
            return;
        };
        let options = MarkdownEditorOptions {
            file_name: SharedString::from(open.page.file_name.clone()),
            file_path: open
                .source_file
                .as_deref()
                .map(path_text)
                .unwrap_or_else(|| SharedString::from(open.page.path.clone())),
            initial: open.markdown.clone(),
            save_label: SharedString::new_static(cx.t().docs_save_file()),
        };
        let editing = edit_markdown(options, window, cx);
        let model = self.model.clone();
        self.edit_task = Some(cx.spawn(async move |_, cx| {
            if let Some(edited) = editing.await {
                model
                    .update(cx, |model, cx| model.save_current(edited, cx));
            }
        }));
    }

    #[cfg(feature = "tools")]
    fn ensure_tools(&mut self, open: &OpenPage, window: &mut Window, cx: &mut Context<Self>) {
        for marker in open.document.tools() {
            let Some(kind) = ToolKind::from_marker(&marker.id) else {
                continue;
            };
            if !self.tools.contains_key(&marker.id) {
                let panel = cx.new(|cx| ToolPanel::new(kind, window, cx));
                self.tools.insert(marker.id.clone(), panel);
            }
        }
    }

    fn top_bar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let colors = cx.theme().colors;
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let single_page = self.model.read(cx).is_single_page();
        h_flex()
            .w_full()
            .flex_none()
            .gap(sizes.spacing_lg)
            .px(sizes.card_padding_h)
            .py(sizes.spacing_md)
            .bg(colors.surface_medium)
            .border_b_1()
            .border_color(colors.border_light)
            .child(
                Link::new("docs-back-to-launcher", strings.docs_back_to_launcher())
                    .icon(IconName::ArrowBack)
                    .size(LabelSize::Small)
                    .color(Color::Secondary)
                    .hover_color(Color::Accent)
                    .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
            )
            .child(div().flex_1())
            .when(!single_page, |this| {
                this.child(
                    Icon::new(IconName::MenuBookOutlined)
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(
                    Label::new(strings.tab_docs())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .bold(),
                )
            })
    }

    fn nav_panel(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let colors = cx.theme().colors;
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let (searching, category) = {
            let model = self.model.read(cx);
            (model.is_searching(), model.active_category().to_owned())
        };
        let rows: Vec<AnyElement> = if searching {
            self.search_results(window, cx)
        } else {
            self.category_tree(window, cx)
        };
        let entity = self.model.clone();
        let labels = [
            SharedString::new_static(strings.docs_category_native()),
            SharedString::new_static(strings.docs_category_scripting()),
        ];
        let available = sizes.docs_nav_width - sizes.padding_md * 2. - px(1.);
        let fitting = switch_fit(&labels, available, window, cx);
        let [native, scripting] = labels;
        let options = match fitting {
            Some(_) => [ToggleOption::new(native), ToggleOption::new(scripting)],
            None => [
                ToggleOption::new(SharedString::default())
                    .icon(IconName::Inventory2Outlined)
                    .tooltip(native),
                ToggleOption::new(SharedString::default())
                    .icon(IconName::MovieFilterOutlined)
                    .tooltip(scripting),
            ],
        };
        let switch = options
            .into_iter()
            .fold(ToggleButtonGroup::new("docs-category"), |group, option| {
                group.option_with(option)
            })
            .selected(match category.as_str() {
                CATEGORY_NATIVE => Some(0),
                CATEGORY_SCRIPTING => Some(1),
                _ => None,
            })
            .size(fitting.map_or(ButtonSize::Default, |fit| fit.size))
            .map(|group| {
                if fitting.is_none_or(|fit| fit.equal_widths) {
                    group.full_width()
                } else {
                    group
                }
            })
            .on_select(move |index, _, cx| {
                let category = if *index == 0 {
                    CATEGORY_NATIVE
                } else {
                    CATEGORY_SCRIPTING
                };
                entity.update(cx, |model, cx| model.set_category(category, cx));
            });
        v_flex()
            .flex_none()
            .w(sizes.docs_nav_width)
            .bg(colors.surface_medium)
            .border_r_1()
            .border_color(colors.border_light)
            .child(
                v_flex()
                    .w_full()
                    .flex_none()
                    .gap(sizes.padding_md)
                    .p(sizes.padding_md)
                    .child(
                        Input::new(&self.search)
                            .size(InputSize::Small)
                            .cleanable(true)
                            .prefix(
                                Icon::new(IconName::Search)
                                    .size(IconSize::Small)
                                    .color(Color::Muted),
                            ),
                    )
                    .when(!searching, |this| this.child(h_flex().w_full().child(switch))),
            )
            .child(
                fill_parent(
                    v_flex()
                        .id("docs-nav-list")
                        .size_full()
                        .pb(sizes.padding_lg)
                        .children(rows)
                        .overflow_y_scrollbar(&self.nav_scroll),
                ),
            )
    }

    fn search_results(&self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let model = self.model.read(cx);
        let query = model.query().trim().to_owned();
        let hits = model.hits();
        if hits.is_empty() {
            return vec![
                v_flex()
                    .px(sizes.card_padding_h)
                    .py(sizes.spacing_md)
                    .child(
                        Label::new(strings.docs_no_results())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .italic(),
                    )
                    .into_any_element(),
            ];
        }
        let current = model.current_path().map(str::to_owned);
        let mut rows = vec![
            ListHeader::new(
                "docs-result-count",
                SharedString::from(strings.docs_result_count(hits.len() as i64)),
            )
            .into_any_element(),
        ];
        for (index, hit) in hits.iter().enumerate() {
            let page = hit.page.clone();
            let entity = self.model.clone();
            rows.push(
                ListItem::new(SharedString::from(format!("docs-hit-{index}")))
                    .selected(current.as_deref() == Some(hit.page.path.as_str()))
                    .on_click(move |_, _, cx| {
                        let page = page.clone();
                        entity.update(cx, |model, cx| model.open(page, None, true, cx));
                    })
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .child(
                                HighlightedLabel::matching(
                                    SharedString::from(hit.page.title.clone()),
                                    &query,
                                )
                                .size(LabelSize::Small)
                                .color(Color::Secondary)
                                .truncate(),
                            )
                            .children(hit.excerpt.as_ref().map(|excerpt| {
                                v_flex().max_h(sizes.font_xs * 2.9).overflow_hidden().child(
                                    HighlightedLabel::matching(
                                        SharedString::from(excerpt.to_string()),
                                        &query,
                                    )
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                                )
                            })),
                    )
                    .into_any_element(),
            );
        }
        rows
    }

    fn category_tree(&self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let model = self.model.read(cx);
        let current = model.current_path().map(str::to_owned);
        let home_entity = self.model.clone();
        let mut rows = vec![
            ListItem::new("docs-nav-home")
                .selected(model.shows_home())
                .on_click(move |_, _, cx| home_entity.update(cx, |model, cx| model.go_home(cx)))
                .child(
                    Label::new(strings.docs_home())
                        .size(LabelSize::Small)
                        .color(if model.shows_home() {
                            Color::Accent
                        } else {
                            Color::Secondary
                        })
                        .truncate(),
                )
                .into_any_element(),
            div().h(sizes.spacing_md).flex_none().into_any_element(),
        ];
        for (index, entry) in nav_entries(model.pages(), model.active_category())
            .into_iter()
            .enumerate()
        {
            match entry {
                NavEntry::Header(group) => rows.push(
                    ListHeader::new(
                        SharedString::from(format!("docs-nav-header-{index}")),
                        match group {
                            NavGroup::Authoring => strings.docs_category_authoring(),
                            NavGroup::Resources => strings.docs_category_resources(),
                        },
                    )
                    .into_any_element(),
                ),
                NavEntry::Page(page) => {
                    let active = current.as_deref() == Some(page.path.as_str());
                    let entity = self.model.clone();
                    let title = SharedString::from(page.title.clone());
                    rows.push(
                        ListItem::new(SharedString::from(format!("docs-nav-{index}")))
                            .selected(active)
                            .tooltip(Tooltip::text(title.clone()))
                            .on_click(move |_, _, cx| {
                                let page = page.clone();
                                entity.update(cx, |model, cx| model.open(page, None, true, cx));
                            })
                            .child(
                                Label::new(title)
                                    .size(LabelSize::Small)
                                    .color(if active {
                                        Color::Accent
                                    } else {
                                        Color::Secondary
                                    })
                                    .truncate(),
                            )
                            .into_any_element(),
                    );
                }
            }
        }
        rows
    }

    fn content_header(
        &self,
        open: &OpenPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let can_edit = self.model.read(cx).can_edit();
        let reload_entity = self.model.clone();
        PageHeader::new(SharedString::from(open.page.title.clone()))
            .start_slot(
                IconButton::new("docs-page-back", IconName::ArrowBack)
                    .icon_size(IconSize::Medium)
                    .icon_color(Color::Secondary)
                    .tooltip(Tooltip::text(strings.button_back()))
                    .on_click(cx.listener(|this, _, _, cx| this.back(cx))),
            )
            .actions(
                h_flex()
                    .flex_none()
                    .gap(sizes.spacing_md)
                    .when(can_edit, |this| {
                        this.child(
                            IconButton::new("docs-page-reload", IconName::Refresh)
                                .icon_size(IconSize::Small)
                                .icon_color(Color::Muted)
                                .tooltip(Tooltip::text(strings.docs_reload()))
                                .on_click(move |_, _, cx| {
                                    reload_entity
                                        .update(cx, |model, cx| model.reload_current(cx));
                                }),
                        )
                        .child(
                            IconButton::new("docs-page-edit", IconName::EditOutlined)
                                .icon_size(IconSize::Small)
                                .icon_color(Color::Accent)
                                .tooltip(Tooltip::text(strings.docs_edit()))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.edit_current(window, cx);
                                })),
                        )
                    })
                    .child(
                        Chip::new(category_label(&open.page.category, strings))
                            .tone(ChipTone::Accent),
                    ),
            )
    }

    fn content_panel(
        &mut self,
        open: &OpenPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let sizes = window.sizes(cx);
        let single_page = self.model.read(cx).is_single_page();
        let generation = self.model.read(cx).generation();
        #[cfg(feature = "tools")]
        self.ensure_tools(open, window, cx);
        let figures = open.figures.clone();
        let entity = cx.entity();
        let context = MarkdownContext::new("docs-page")
            .anchors(self.anchors.clone())
            .on_link(move |href, _, cx| {
                let href = href.to_owned();
                entity.update(cx, |this, cx| this.follow_link(&href, cx));
            })
            .figures(move |source| -> Option<Figure> { figures.get(source).cloned() });
        #[cfg(feature = "tools")]
        let context = {
            let tools: Rc<HashMap<String, Entity<ToolPanel>>> = Rc::new(self.tools.clone());
            context.tools(move |marker: &ToolMarker, _, _| {
                tools
                    .get(&marker.id)
                    .map(|panel| panel.clone().into_any_element())
            })
        };
        let blocks = render_document(&open.document, &context, window, cx);
        let header = (!single_page).then(|| self.content_header(open, window, cx));
        v_flex()
            .flex_1()
            .min_w_0()
            .children(header)
            .child(
                fill_parent(
                    div()
                        .id("docs-content")
                        .size_full()
                        .p(sizes.content_padding)
                        .child(
                            v_flex()
                                .max_w(px(CONTENT_MAX_WIDTH))
                                .mx_auto()
                                .gap(sizes.spacing_lg)
                                .pb(sizes.padding_xl)
                                .children(blocks)
                                .with_animation(
                                    ("docs-page-fade", generation),
                                    Animation::new(PAGE_FADE).with_easing(easing::nier_fade),
                                    |this, delta| this.opacity(delta),
                                ),
                        )
                        .overflow_y_scrollbar(&self.content_scroll),
                ),
            )
    }

    fn outline_panel(
        &self,
        outline: &[DocHeading],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let colors = cx.theme().colors;
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let rows = outline.iter().enumerate().map(|(index, heading)| {
            let anchor = heading.anchor.clone();
            ListItem::new(SharedString::from(format!("docs-outline-{index}")))
                .spacing(ListItemSpacing::ExtraDense)
                .indent_level(usize::from(heading.level >= 3))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.scroll_to_anchor(&anchor, true, cx);
                }))
                .child(
                    v_flex().flex_1().min_w_0().child(
                        Label::new(SharedString::from(heading.text.clone()))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .line_height(1.35),
                    ),
                )
        });
        v_flex()
            .flex_none()
            .w(sizes.docs_outline_width)
            .bg(colors.surface_medium)
            .border_l_1()
            .border_color(colors.border_light)
            .child(fill_parent(
                v_flex()
                    .id("docs-outline")
                    .size_full()
                    .py(sizes.padding_md)
                    .child(ListHeader::new(
                        "docs-outline-header",
                        strings.docs_on_this_page(),
                    ))
                    .children(rows)
                    .overflow_y_scrollbar(&self.outline_scroll),
            ))
    }

    fn empty_state(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let strings = cx.t();
        let detail = self.model.read(cx).load_error().map(str::to_owned);
        let state = EmptyState::new(strings.docs_unavailable()).icon(IconName::MenuBookOutlined);
        match detail {
            Some(detail) => state.description(SharedString::from(detail)),
            None => state,
        }
    }
}

impl Focusable for DocsOverlay {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DocsOverlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;
        let sizes = window.sizes(cx);
        if self.pending_anchor.is_some() {
            cx.defer_in(window, |this, _, cx| this.resolve_pending_anchor(cx));
        }
        let (loading, single_page, has_pages, shows_home, current) = {
            let model = self.model.read(cx);
            (
                model.is_loading(),
                model.is_single_page(),
                !model.pages().is_empty(),
                model.shows_home(),
                model.current().cloned(),
            )
        };
        let body = if loading {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(Spinner::new().size(IconSize::Large))
                .into_any_element()
        } else if single_page {
            match &current {
                Some(open) => {
                    let outline =
                        shows_outline(&open.outline).then(|| open.outline.clone());
                    div()
                        .flex()
                        .flex_row()
                        .size_full()
                        .child(self.content_panel(open, window, cx))
                        .children(
                            outline.map(|outline| self.outline_panel(&outline, window, cx)),
                        )
                        .into_any_element()
                }
                None => self.empty_state(cx).into_any_element(),
            }
        } else if !has_pages {
            self.empty_state(cx).into_any_element()
        } else {
            let nav = self.nav_panel(window, cx).into_any_element();
            let main = match &current {
                Some(open) if !shows_home => {
                    self.content_panel(open, window, cx).into_any_element()
                }
                _ => {
                    let pages = self.model.read(cx).pages().to_vec();
                    let width = sizes.window_width - sizes.docs_nav_width - sizes.content_padding * 2.;
                    let model = self.model.clone();
                    let open_file: OpenFile = Rc::new(move |file_name, _, cx| {
                        model.update(cx, |model, cx| model.open_file_name(file_name, cx));
                    });
                    let search = self.search.clone();
                    let browse_all: BrowseAll =
                        Rc::new(move |window, cx| window.focus(&search.focus_handle(cx), cx));
                    fill_parent(render_home(
                            &pages,
                            width,
                            &self.home_scroll,
                            open_file,
                            browse_all,
                            window,
                            cx,
                        ))
                        .into_any_element()
                }
            };
            let outline = current
                .as_ref()
                .filter(|open| !shows_home && shows_outline(&open.outline))
                .map(|open| open.outline.clone());
            div()
                .flex()
                .flex_row()
                .size_full()
                .child(nav)
                .child(main)
                .children(outline.map(|outline| self.outline_panel(&outline, window, cx)))
                .into_any_element()
        };
        v_flex()
            .id("docs-overlay")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(colors.background_primary)
            .child(self.top_bar(window, cx))
            .child(fill_parent(body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heading(text: &str) -> DocHeading {
        DocHeading {
            level: 2,
            text: text.to_owned(),
            anchor: yp_docs::anchor_for(text),
        }
    }

    #[gpui::test]
    fn the_third_party_page_opens_alone_unless_the_library_is_built(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            yp_ui::init_for_tests(cx);
            cx.set_global(DocsConfig {
                docs_dir: crate::markdown::tests::docs_dir(),
                editable: false,
            });
        });
        let request = OverlayRequest::Docs {
            page: Some(yp_core::consts::THIRD_PARTY_DOC_PATH.to_owned()),
        };
        let (overlay, cx) =
            cx.add_window_view(|window, cx| DocsOverlay::from_request(&request, window, cx));
        cx.run_until_parked();
        overlay.read_with(cx, |overlay, cx| {
            let model = overlay.model().read(cx);
            assert_eq!(model.is_single_page(), !cfg!(feature = "library"));
            assert_eq!(model.current().expect("page").page.file_name, "35_third_party.md");
        });
    }

    #[test]
    fn the_outline_needs_three_headings() {
        let two = vec![heading("One"), heading("Two")];
        assert!(!shows_outline(&two));
        let three = vec![heading("One"), heading("Two"), heading("Three")];
        assert!(shows_outline(&three));
    }

    #[test]
    fn a_heading_scrolls_just_below_the_top_and_stays_in_range() {
        let target = anchor_scroll_target(px(0.), px(900.), px(100.), px(600.), px(2000.));
        assert_eq!(target, px(-770.));
        let from_scrolled = anchor_scroll_target(px(-500.), px(400.), px(100.), px(600.), px(2000.));
        assert_eq!(from_scrolled, px(-770.));
        let near_end = anchor_scroll_target(px(0.), px(5000.), px(100.), px(600.), px(2000.));
        assert_eq!(near_end, px(-2000.));
        let at_top = anchor_scroll_target(px(0.), px(110.), px(100.), px(600.), px(2000.));
        assert_eq!(at_top, px(0.));
    }

    #[test]
    fn the_largest_switch_that_fits_wins_and_none_means_icons() {
        let candidates = [("medium", px(280.)), ("default", px(240.)), ("compact", px(190.))];
        assert_eq!(first_fitting(&candidates, px(300.)), Some("medium"));
        assert_eq!(first_fitting(&candidates, px(250.)), Some("default"));
        assert_eq!(first_fitting(&candidates, px(190.)), Some("compact"));
        assert_eq!(first_fitting(&candidates, px(120.)), None);
    }

    #[test]
    fn a_switch_prefers_equal_buttons_and_falls_back_to_natural_widths() {
        let widths = [px(150.), px(90.)];
        let [equal, natural] = switch_candidates(ButtonSize::Medium, &widths);
        assert_eq!(equal.1, px(300.));
        assert!(equal.0.equal_widths);
        assert_eq!(natural.1, px(240.));
        assert!(!natural.0.equal_widths);
        let candidates: Vec<_> = [ButtonSize::Medium, ButtonSize::Compact]
            .into_iter()
            .zip([[px(150.), px(90.)], [px(110.), px(70.)]])
            .flat_map(|(size, widths)| switch_candidates(size, &widths))
            .collect();
        let fit = |available: f32| first_fitting(&candidates, px(available));
        assert_eq!(
            fit(320.),
            Some(SwitchFit {
                size: ButtonSize::Medium,
                equal_widths: true
            })
        );
        assert_eq!(
            fit(250.),
            Some(SwitchFit {
                size: ButtonSize::Medium,
                equal_widths: false
            })
        );
        assert_eq!(
            fit(230.),
            Some(SwitchFit {
                size: ButtonSize::Compact,
                equal_widths: true
            })
        );
        assert_eq!(
            fit(190.),
            Some(SwitchFit {
                size: ButtonSize::Compact,
                equal_widths: false
            })
        );
        assert_eq!(fit(120.), None);
    }

    #[test]
    fn the_nav_row_of_a_page_counts_the_home_row_the_gap_and_group_headers() {
        let page = |path: &str| yp_docs::DocPage {
            path: path.to_owned(),
            category: CATEGORY_NATIVE.to_owned(),
            title: path.to_owned(),
            file_name: path.rsplit('/').next().unwrap_or(path).to_owned(),
        };
        let entries = vec![
            NavEntry::Page(page("assets/docs/native/01_a.md")),
            NavEntry::Header(NavGroup::Resources),
            NavEntry::Page(page("assets/docs/native/resources/items.md")),
        ];
        assert_eq!(nav_row_of(&entries, "assets/docs/native/01_a.md"), Some(2));
        assert_eq!(nav_row_of(&entries, "assets/docs/native/resources/items.md"), Some(4));
        assert_eq!(nav_row_of(&entries, "assets/docs/native/none.md"), None);
    }

    #[test]
    fn unknown_categories_get_a_readable_name() {
        let strings = yp_i18n::Strings::for_locale(yp_i18n::Locale::En);
        assert_eq!(category_label(CATEGORY_NATIVE, strings).as_ref(), strings.docs_category_native());
        assert_eq!(
            category_label(CATEGORY_SCRIPTING, strings).as_ref(),
            strings.docs_category_scripting()
        );
        assert_eq!(category_label("deep_dives", strings).as_ref(), pretty_name("deep_dives"));
    }
}
