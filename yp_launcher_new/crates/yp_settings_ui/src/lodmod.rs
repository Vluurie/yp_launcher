use gpui::{ScrollHandle, Subscription};
use yp_config::Setting;
use yp_state::{ConfigStore, prelude::*};
use yp_ui::prelude::*;

use crate::extras;
use crate::fields::Marker;
use crate::page::{column, tab_page};
use crate::schema_page::{self, CardPlan, ResetText, TabScope};

const SCOPE: &str = "lodmod";
const FILE: &str = "lodmod.toml";
const MASTER: &str = "lodmod.enabled";
pub const TAB: TabScope = TabScope::roots(&["graphics"]);

#[derive(Clone, PartialEq)]
enum Destination {
    Category(String),
    Setting(String),
}

pub struct LodModTab {
    config: Entity<ConfigStore>,
    search: Entity<InputState>,
    selected: Option<Destination>,
    index_scroll: ScrollHandle,
    detail_scroll: ScrollHandle,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

fn matches_query(setting: &Setting, category: &str, words: &[&str]) -> bool {
    let text = format!(
        "{} {} {} {} {} {}",
        setting.label,
        setting.id,
        setting.key,
        setting.description,
        category,
        setting.group.as_deref().unwrap_or_default()
    )
    .to_lowercase();
    words.iter().all(|word| text.contains(word))
}

impl LodModTab {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx.stores().config.clone();
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder(cx.t().lod_search_settings()));
        let subscriptions = vec![
            cx.observe(&config, |_, _, cx| cx.notify()),
            cx.observe_global::<I18n>(|this, cx| {
                let hint = cx.t().lod_search_settings();
                this.search
                    .update(cx, |input, cx| input.set_placeholder(hint, cx));
            }),
            cx.subscribe(&search, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.index_scroll.set_offset(Default::default());
                    cx.notify();
                }
            }),
        ];
        Self {
            config,
            search,
            selected: None,
            index_scroll: ScrollHandle::new(),
            detail_scroll: ScrollHandle::new(),
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        if self.selected.is_some() {
            &self.detail_scroll
        } else {
            &self.index_scroll
        }
    }

    fn navigate(
        &mut self,
        target: Option<Destination>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected = target;
        self.detail_scroll.set_offset(Default::default());
        if self.selected.is_some() {
            self.focus.focus(window, cx);
        } else {
            self.search.focus_handle(cx).focus(window, cx);
        }
        cx.notify();
    }

    fn reset_text(cx: &App) -> ResetText {
        let strings = cx.t();
        ResetText {
            title: SharedString::new_static(strings.lod_mod_reset_confirm_title()),
            body: SharedString::new_static(strings.lod_mod_reset_confirm_body()),
        }
    }

    pub fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = Self::reset_text(cx);
        schema_page::confirm_reset(TAB, &text, &self.config, window, cx);
    }

    fn entry(
        &self,
        destination: Destination,
        title: &str,
        detail: String,
        changed: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = cx.theme().colors;
        let detail = if changed == 0 {
            detail
        } else {
            format!("{detail} · {}", cx.t().card_changed_count(changed as i64))
        };
        let key = match &destination {
            Destination::Category(id) => format!("lod-category-{id}"),
            Destination::Setting(id) => format!("lod-result-{id}"),
        };
        h_flex()
            .id(SharedString::from(key.clone()))
            .debug_selector(move || key.clone())
            .role(gpui::Role::Link)
            .aria_label(format!("{title}, {detail}"))
            .keyboard_activate(cx)
            .cursor_pointer()
            .w_full()
            .min_w_0()
            .min_h(px(48.))
            .gap(px(8.))
            .px(px(12.))
            .py(px(7.))
            .bg(colors.background_card)
            .border_b_1()
            .border_color(colors.border_medium)
            .hover(move |s| {
                s.bg(colors.surface_medium)
                    .border_color(colors.accent_primary)
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(Label::new(title.to_owned()).semibold())
                    .child(
                        Label::new(detail)
                            .size(LabelSize::XSmall)
                            .color(Color::Secondary),
                    ),
            )
            .child(
                Icon::new(IconName::ChevronRight)
                    .size(IconSize::Small)
                    .color(Color::Secondary),
            )
            .on_click(
                cx.listener(move |this, _, w, cx| this.navigate(Some(destination.clone()), w, cx)),
            )
            .into_any_element()
    }

    fn overview(
        &self,
        plans: &[CardPlan],
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let strings = cx.t();
        let search = self.search.read(cx).value().trim().to_lowercase();
        let words: Vec<_> = search.split_whitespace().collect();
        let mut entries = Vec::new();
        for plan in plans {
            let settings = plan.settings.iter().chain(&plan.advanced);
            if words.is_empty() {
                let visible = settings.filter(|s| s.id != MASTER);
                let count = visible.clone().count();
                let changed = visible
                    .filter(|s| self.config.read(cx).is_changed(&s.id))
                    .count();
                if count > 0 {
                    entries.push(self.entry(
                        Destination::Category(plan.category.id.clone()),
                        &plan.category.title,
                        strings.lod_settings_count(count as i64),
                        changed,
                        cx,
                    ));
                }
            } else {
                for setting in settings.filter(|s| matches_query(s, &plan.category.title, &words)) {
                    let changed = usize::from(self.config.read(cx).is_changed(&setting.id));
                    entries.push(self.entry(
                        Destination::Setting(setting.id.clone()),
                        &setting.label,
                        plan.category.title.clone(),
                        changed,
                        cx,
                    ));
                }
            }
        }
        let mut content = Vec::new();
        if words.is_empty() {
            if let Some(master) = self.config.read(cx).setting(MASTER) {
                content.push(
                    schema_page::field(SCOPE, master, &self.config, Marker::Always, cx)
                        .description(None)
                        .into_any_element(),
                );
            }
        }
        if entries.is_empty() && !words.is_empty() {
            content.push(
                EmptyState::new(strings.lod_no_settings())
                    .compact()
                    .into_any_element(),
            );
        } else if !entries.is_empty() {
            content.push(
                ResponsiveGrid::new("lod-category-grid", window.sizes(cx).font_md * 17., 3)
                    .children(entries)
                    .into_any_element(),
            );
        }
        content
    }

    fn category(&self, plan: &CardPlan, window: &Window, cx: &App) -> Vec<AnyElement> {
        let mut content: Vec<_> = extras::intro(&plan.category.id)
            .map(|intro| extras::render_intro(intro, window, cx))
            .into_iter()
            .collect();
        let mut rows = schema_page::RowContext::default();
        for setting in plan.settings.iter().filter(|s| s.id != MASTER) {
            content.extend(rows.field(SCOPE, setting, &self.config, Marker::Always, window, cx));
        }
        let advanced = schema_page::generic_rows(
            SCOPE,
            &plan.advanced,
            &self.config,
            Marker::Always,
            window,
            cx,
        );
        if !advanced.is_empty() {
            content.push(
                CollapsibleCard::new(plan.advanced_id.clone(), cx.t().card_experimental())
                    .children(advanced)
                    .into_any_element(),
            );
        }
        content
    }
}

impl Render for LodModTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let store = self.config.read(cx);
        let loading = schema_page::is_loading(store);
        let reset = TAB.has_changes(store).then(|| {
            schema_page::reset_button("lodmod-reset", TAB, Self::reset_text(cx), &self.config, cx)
        });
        let header = schema_page::header(
            strings.header_lod_mod(),
            strings.tooltip_edits_lodmod_toml(),
            Some(FILE),
            reset,
            &self.config,
            cx,
        );
        let plans = schema_page::plans(SCOPE, &TAB, store);
        let detail = self.selected.as_ref().and_then(|selected| match selected {
            Destination::Category(id) => {
                let plan = plans.iter().find(|p| p.category.id == *id)?;
                Some((plan.category.title.clone(), self.category(plan, window, cx)))
            }
            Destination::Setting(id) => plans.iter().find_map(|plan| {
                let setting = plan
                    .settings
                    .iter()
                    .chain(&plan.advanced)
                    .find(|s| s.id == *id)?;
                let mut content: Vec<_> = extras::intro(&plan.category.id)
                    .map(|intro| extras::render_intro(intro, window, cx))
                    .into_iter()
                    .collect();
                if setting.advanced {
                    content.push(
                        Label::new(strings.card_experimental())
                            .size(LabelSize::Small)
                            .color(Color::Warning)
                            .into_any_element(),
                    );
                }
                content.extend(schema_page::RowContext::default().field(
                    SCOPE,
                    setting,
                    &self.config,
                    Marker::Always,
                    window,
                    cx,
                ));
                Some((plan.category.title.clone(), content))
            }),
        });
        if detail.is_none() && !loading {
            self.selected = None;
        }
        let in_detail = detail.is_some();
        let mut content = schema_page::problems(SCOPE, &TAB, &self.config, cx);
        let bar = if let Some((title, rows)) = detail {
            content.push(
                v_flex()
                    .w_full()
                    .min_w_0()
                    .children(rows)
                    .into_any_element(),
            );
            h_flex()
                .w_full()
                .min_w_0()
                .flex_wrap()
                .gap(px(12.))
                .child(
                    Button::new(
                        "lod-back",
                        if self.search.read(cx).is_empty() {
                            strings.lod_categories()
                        } else {
                            strings.button_back()
                        },
                    )
                    .style(ButtonStyle::Link)
                    .start_icon(IconName::ArrowBack)
                    .on_click(cx.listener(|v, _, w, cx| v.navigate(None, w, cx))),
                )
                .child(
                    div()
                        .min_w_0()
                        .max_w_full()
                        .id("lod-section-title")
                        .debug_selector(|| "lod-section-title".to_owned())
                        .role(gpui::Role::Heading)
                        .aria_level(2)
                        .aria_label(title.clone())
                        .child(Label::new(title).size(LabelSize::Large).semibold()),
                )
                .into_any_element()
        } else {
            content.extend(self.overview(&plans, window, cx));
            div()
                .w_full()
                .max_w(px(440.))
                .debug_selector(|| "lod-search".to_owned())
                .child(
                    Input::new(&self.search)
                        .prefix(Icon::new(IconName::Search).size(IconSize::Small))
                        .cleanable(true),
                )
                .into_any_element()
        };
        let menu = cx.theme().id == ThemeId::NierNew;
        let pad = window.sizes(cx).content_padding;
        let width = if in_detail { px(800.) } else { px(1120.) };
        let header = v_flex().w_full().flex_none().child(header).child(
            v_flex()
                .w_full()
                .items_center()
                .when(!menu, |el| el.px(pad))
                .pt(px(4.))
                .pb(px(8.))
                .child(div().w_full().max_w(width).child(bar)),
        );
        let body = v_flex().w_full().items_center().child(
            div()
                .w_full()
                .min_w_0()
                .max_w(width)
                .debug_selector(|| "lod-page-content".to_owned())
                .child(column(content, window, cx)),
        );
        tab_page(
            "lodmod-scroll",
            header,
            self.scroll_handle(),
            loading,
            body,
            window,
            cx,
        )
        .track_focus(&self.focus)
        .on_key_down(cx.listener(|v, e: &gpui::KeyDownEvent, w, cx| {
            if e.keystroke.key == "escape" && v.selected.is_some() {
                v.navigate(None, w, cx);
                cx.stop_propagation();
            }
        }))
    }
}
