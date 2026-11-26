use std::path::PathBuf;
use std::rc::Rc;

use serde_json::Value as Json;
use yp_config::schema::{PLUGIN_CATEGORY_PREFIX, is_in_category};
use yp_config::{Category, ResetTarget, Schema, Setting};
use yp_state::ConfigStore;
use yp_ui::prelude::*;
use yp_ui::{
    Button, ButtonStyle, CollapsibleCard, ConfigErrorBanner, ConfirmOptions, ConfirmTone,
    PageHeader, TintColor, TwoColumn, confirm,
};

use crate::errors::config_failure_text;
use crate::extras;
use crate::fields::{Control, FieldRow, Marker, ValueSink, format_number, number_value};
use crate::page::{card, column, failure_banner, locked, reveal};
use crate::rules::RuleColumn;

pub const HIDDEN_CATEGORIES: [&str; 1] = ["mods"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabScope {
    pub roots: &'static [&'static str],
    pub plugins: bool,
    pub managed: &'static [&'static str],
}

impl TabScope {
    pub const fn roots(roots: &'static [&'static str]) -> Self {
        Self {
            roots,
            plugins: false,
            managed: &[],
        }
    }

    pub const fn plugins() -> Self {
        Self {
            roots: &[],
            plugins: true,
            managed: &[],
        }
    }

    pub const fn managed(mut self, managed: &'static [&'static str]) -> Self {
        self.managed = managed;
        self
    }

    pub fn contains(&self, category: &str) -> bool {
        if HIDDEN_CATEGORIES.contains(&category) {
            return false;
        }
        if self.plugins && category.starts_with(PLUGIN_CATEGORY_PREFIX) {
            return true;
        }
        self.roots.iter().any(|root| is_in_category(category, root))
    }

    fn is_root(&self, category: &str) -> bool {
        self.roots.contains(&category)
    }

    pub fn categories(&self, schema: &Schema) -> Vec<Category> {
        let mut categories: Vec<Category> = schema
            .categories
            .iter()
            .filter(|category| self.contains(&category.id))
            .cloned()
            .collect();
        for setting in &schema.settings {
            if self.contains(&setting.category)
                && !categories.iter().any(|category| category.id == setting.category)
            {
                categories.push(Category {
                    id: setting.category.clone(),
                    title: schema.category_title(&setting.category),
                    count: 0,
                });
            }
        }
        categories.sort_by_key(|category| !self.is_root(&category.id));
        categories
    }

    pub fn settings<'a>(&'a self, schema: &'a Schema) -> impl Iterator<Item = &'a Setting> + 'a {
        schema
            .settings
            .iter()
            .filter(move |setting| self.contains(&setting.category))
            .filter(move |setting| !self.managed.contains(&setting.id.as_str()))
    }

    pub fn ids(&self, store: &ConfigStore) -> Vec<String> {
        self.settings(store.schema())
            .map(|setting| setting.id.clone())
            .collect()
    }

    pub fn files(&self, store: &ConfigStore) -> Vec<String> {
        let mut files: Vec<String> = Vec::new();
        for setting in self.settings(store.schema()) {
            if !setting.file.is_empty() && !files.contains(&setting.file) {
                files.push(setting.file.clone());
            }
        }
        files
    }

    pub fn has_changes(&self, store: &ConfigStore) -> bool {
        self.settings(store.schema())
            .any(|setting| store.is_changed(&setting.id))
    }

    pub fn category_titles(&self, store: &ConfigStore) -> Vec<String> {
        self.categories(store.schema())
            .into_iter()
            .map(|category| category.title)
            .collect()
    }
}

pub fn categories_under(schema: &Schema, roots: &'static [&'static str]) -> Vec<Category> {
    TabScope::roots(roots).categories(schema)
}

pub fn settings_of<'a>(schema: &'a Schema, category: &'a str, hidden: &[&str]) -> Vec<&'a Setting> {
    schema
        .settings_in(category)
        .filter(|setting| !hidden.contains(&setting.id.as_str()))
        .collect()
}

pub struct CardPlan {
    pub id: ElementId,
    pub advanced_id: ElementId,
    pub category: Category,
    pub settings: Vec<Setting>,
    pub advanced: Vec<Setting>,
    pub changed: Vec<SharedString>,
}

pub fn plans(scope: &str, tab: &TabScope, store: &ConfigStore) -> Vec<CardPlan> {
    let schema = store.schema().clone();
    tab.categories(&schema)
        .into_iter()
        .filter_map(|category| {
            let visible: Vec<&Setting> = schema
                .settings_in(&category.id)
                .filter(|setting| !tab.managed.contains(&setting.id.as_str()))
                .filter(|setting| extras::is_shown(setting, store))
                .collect();
            if visible.is_empty() {
                return None;
            }
            let changed = changed_labels(&visible, store);
            let (advanced, settings): (Vec<&Setting>, Vec<&Setting>) =
                visible.into_iter().partition(|setting| setting.advanced);
            Some(CardPlan {
                id: card_id(scope, &category.id),
                advanced_id: card_id(scope, &format!("{}-advanced", category.id)),
                settings: settings.into_iter().cloned().collect(),
                advanced: advanced.into_iter().cloned().collect(),
                category,
                changed,
            })
        })
        .collect()
}

pub fn sink(config: &Entity<ConfigStore>, id: &str) -> ValueSink {
    let config = config.clone();
    let id = id.to_owned();
    Rc::new(move |value: Json, _, cx| {
        config.update(cx, |store, cx| {
            let overrides = extras::overrides_to_clear(&id, store);
            store.set(&id, value, cx);
            for by in overrides {
                store.set(by, Json::Null, cx);
            }
        });
    })
}

fn labelled_columns(columns: Vec<RuleColumn>, strings: &'static yp_i18n::Strings) -> Vec<RuleColumn> {
    columns
        .into_iter()
        .map(|mut column| {
            if let Some(label) = extras::column_label(&column.key) {
                column.label = SharedString::new_static(label(strings));
            }
            column
        })
        .collect()
}

pub fn field(
    scope: &'static str,
    setting: &Setting,
    config: &Entity<ConfigStore>,
    marker: Marker,
    cx: &App,
) -> FieldRow {
    let strings = cx.t();
    let store = config.read(cx);
    let value = extras::shown_value(&setting.id, store);
    let zero = value.as_f64().is_some_and(|number| number == 0.);
    let mut row = FieldRow::new(scope, setting, value)
        .marker(marker)
        .disabled(extras::lock(setting, store).is_some())
        .sink(sink(config, &setting.id));
    if zero && let Some(label) = extras::zero_label(&setting.id) {
        row = row.value_label(SharedString::new_static(label(strings)));
    }
    if let Some((add, empty)) = extras::list_labels(&setting.id) {
        row = row.add_label(add(strings)).empty_label(empty(strings));
    }
    if let Some(options) = extras::options(&setting.id) {
        row = row.control_override(Control::Select {
            options: options
                .iter()
                .map(|option| {
                    let label = SharedString::from(format_number(setting.kind, *option));
                    (number_value(setting.kind, *option), label)
                })
                .collect(),
        });
    }
    match row.control() {
        Control::Rules { columns } => {
            row = row.control_override(Control::Rules {
                columns: labelled_columns(columns, strings),
            });
        }
        Control::Select { options } => {
            if let Some(label_for) = extras::option_label(&setting.id) {
                let options = options
                    .into_iter()
                    .map(|(value, label)| {
                        let localized = label_for(strings, &value);
                        (value, localized.map(SharedString::from).unwrap_or(label))
                    })
                    .collect();
                row = row.control_override(Control::Select { options });
            }
        }
        Control::Numbers { length, .. } => {
            if let Some(item) = extras::item_label(&setting.id) {
                let labels = (1..=length)
                    .map(|number| {
                        let cascade = item(strings, i64::try_from(number).unwrap_or(i64::MAX));
                        SharedString::from(strings.lodmod_ui_cascade_label(&setting.label, &cascade))
                    })
                    .collect();
                row = row.item_labels(labels);
            }
        }
        _ => {}
    }
    row
}

#[derive(Default)]
pub struct RowContext {
    last_lock: Option<&'static str>,
}

impl RowContext {
    pub fn wrap(
        &mut self,
        setting: &Setting,
        element: AnyElement,
        config: &Entity<ConfigStore>,
        window: &Window,
        cx: &App,
    ) -> Vec<AnyElement> {
        let store = config.read(cx);
        let reason = extras::lock(setting, store).map(|reason| reason(cx.t()));
        let repeated = reason.is_some() && reason == self.last_lock;
        self.last_lock = reason;
        let element = match reason {
            Some(reason) if !repeated => locked(false, reason, element, window, cx),
            _ => element,
        };
        let mut rows = vec![element];
        rows.extend(extras::after(setting, store, window, cx));
        rows
    }

    pub fn field(
        &mut self,
        scope: &'static str,
        setting: &Setting,
        config: &Entity<ConfigStore>,
        marker: Marker,
        window: &Window,
        cx: &App,
    ) -> Vec<AnyElement> {
        let element = field(scope, setting, config, marker, cx).into_any_element();
        self.wrap(setting, element, config, window, cx)
    }
}

pub fn generic_rows(
    scope: &'static str,
    settings: &[Setting],
    config: &Entity<ConfigStore>,
    marker: Marker,
    window: &Window,
    cx: &App,
) -> Vec<AnyElement> {
    let mut context = RowContext::default();
    settings
        .iter()
        .flat_map(|setting| context.field(scope, setting, config, marker, window, cx))
        .collect()
}

pub fn finish_card(
    plan: CardPlan,
    rows: Vec<AnyElement>,
    advanced: Vec<AnyElement>,
    window: &Window,
    cx: &App,
) -> CollapsibleCard {
    let intro = extras::intro(&plan.category.id).map(|intro| extras::render_intro(intro, window, cx));
    let advanced_card = (!advanced.is_empty()).then(|| {
        div().w_full().pt(window.sizes(cx).spacing_md).child(
            CollapsibleCard::new(plan.advanced_id.clone(), cx.t().card_experimental())
                .default_open(false)
                .children(advanced),
        )
    });
    card(plan.id, plan.category.title, cx)
        .changed(plan.changed)
        .children(intro)
        .children(rows)
        .children(advanced_card)
}

pub fn generic_card(
    scope: &'static str,
    plan: CardPlan,
    config: &Entity<ConfigStore>,
    marker: Marker,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let rows = generic_rows(scope, &plan.settings, config, marker, window, cx);
    let advanced = generic_rows(scope, &plan.advanced, config, marker, window, cx);
    finish_card(plan, rows, advanced, window, cx).into_any_element()
}

pub fn changed_labels(settings: &[&Setting], store: &ConfigStore) -> Vec<SharedString> {
    settings
        .iter()
        .filter(|setting| store.is_changed(&setting.id))
        .map(|setting| SharedString::from(setting.label.clone()))
        .collect()
}

pub fn card_id(scope: &str, category: &str) -> ElementId {
    ElementId::Name(SharedString::from(format!("{scope}-{category}")))
}

pub fn spread(cards: Vec<AnyElement>, id: &str, window: &Window, cx: &App) -> AnyElement {
    let mut left: Vec<AnyElement> = Vec::new();
    let mut right: Vec<AnyElement> = Vec::new();
    for (index, card) in cards.into_iter().enumerate() {
        if index % 2 == 0 {
            left.push(card);
        } else {
            right.push(card);
        }
    }
    TwoColumn::new(column(left, window, cx), column(right, window, cx))
        .id(ElementId::Name(SharedString::from(id.to_owned())))
        .breakpoint((window.sizes(cx).font_md * 64.).max(px(820.)))
        .into_any_element()
}

pub fn file_path(store: &ConfigStore, file: &str) -> Option<PathBuf> {
    let path = store
        .values()
        .files
        .get(file)
        .map(|state| state.path.clone())
        .filter(|path| !path.is_empty())
        .or_else(|| {
            store
                .schema()
                .files
                .iter()
                .find(|info| info.file == file)
                .map(|info| info.path.clone())
        })
        .filter(|path| !path.is_empty())?;
    let path = PathBuf::from(path);
    if path.is_absolute() {
        return Some(path);
    }
    store.game().map(|game| game.root().join(path))
}

pub fn problems(scope: &str, tab: &TabScope, config: &Entity<ConfigStore>, cx: &App) -> Vec<AnyElement> {
    let strings = cx.t();
    let store = config.read(cx);
    let mut banners: Vec<AnyElement> = tab
        .files(store)
        .into_iter()
        .filter_map(|file| {
            let error = store.file_error(&file)?.to_owned();
            Some(
                ConfigErrorBanner::new(file.clone(), error)
                    .id(ElementId::Name(SharedString::from(format!("{scope}-file-error-{file}"))))
                    .into_any_element(),
            )
        })
        .collect();
    if let Some(failure) = store.failure() {
        banners.push(
            failure_banner(
                ElementId::Name(SharedString::from(format!("{scope}-failure"))),
                config_failure_text(failure, strings),
            )
            .into_any_element(),
        );
    }
    banners
}

pub struct ResetText {
    pub title: SharedString,
    pub body: SharedString,
}

pub fn reset_text(tab: &TabScope, store: &ConfigStore, strings: &'static yp_i18n::Strings) -> ResetText {
    ResetText {
        title: SharedString::new_static(strings.lod_mod_reset_button()),
        body: SharedString::from(tab.category_titles(store).join(", ")),
    }
}

pub fn confirm_reset(
    tab: TabScope,
    text: &ResetText,
    config: &Entity<ConfigStore>,
    window: &mut Window,
    cx: &mut App,
) {
    let strings = cx.t();
    let answer = confirm(
        ConfirmOptions::new(text.title.clone(), text.body.clone())
            .confirm_label(strings.lod_mod_reset_confirm_action())
            .cancel_label(strings.cancel_button())
            .tone(ConfirmTone::Danger),
        window,
        cx,
    );
    let config = config.clone();
    cx.spawn(async move |cx| {
        if answer.await {
            config.update(cx, |store, cx| reset_tab(&tab, store, cx));
        }
    })
    .detach();
}

pub fn reset_tab(tab: &TabScope, store: &mut ConfigStore, cx: &mut Context<ConfigStore>) {
    let ids = tab.ids(store);
    store.reset(&ResetTarget::Ids(ids), cx);
}

pub fn reset_button(
    id: impl Into<ElementId>,
    tab: TabScope,
    text: ResetText,
    config: &Entity<ConfigStore>,
    cx: &App,
) -> Button {
    let config = config.clone();
    Button::new(id, cx.t().lod_mod_reset_button())
        .style(ButtonStyle::Tinted(TintColor::Warning))
        .on_click(move |_, window, cx| confirm_reset(tab, &text, &config, window, cx))
}

pub fn header(
    title: &'static str,
    tooltip: &'static str,
    file: Option<&str>,
    reset: Option<Button>,
    config: &Entity<ConfigStore>,
    cx: &App,
) -> PageHeader {
    let store = config.read(cx);
    let path = file.and_then(|file| file_path(store, file));
    let header = PageHeader::new(title);
    let header = match path {
        Some(path) => header.info(tooltip, move |_, cx| reveal(path.clone(), true, cx)),
        None => header.info_tooltip(tooltip),
    };
    match reset {
        Some(reset) => header.actions(reset),
        None => header,
    }
}

pub fn is_loading(store: &ConfigStore) -> bool {
    store.is_loading() && !store.is_loaded()
}
