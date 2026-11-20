use std::path::PathBuf;
use std::rc::Rc;

use gpui::{PathPromptOptions, ScrollHandle, Subscription, Task};
use yp_builders::CheatTableOutcome;
use yp_config::ConfigFile;
use yp_config::fields::{content, nams};
use yp_state::ConfigStore;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    Button, ButtonStyle, ConfigErrorBanner, Icon, IconName, IconSize, Label, LabelSize, PageHeader,
    TintColor, TwoColumn,
};

use crate::errors::{config_failure_text, toml_error_text};
use crate::fields::{FieldRef, FieldRow, ValueSink, changed_in};
use crate::page::{
    banner, card, column, failure_banner, muted_text, reveal, row_divider, subheading, tab_page,
};

const SCOPE: &str = "nams";

pub const GENERAL_FIELDS: [FieldRef; 6] = [
    &nams::VALIDATE_MODEL_DATA,
    &nams::VALIDATE_SCRIPTS,
    &nams::LOADING_STALL_HINTS,
    &nams::FIX_WIND_TIMER_BUG,
    &nams::DISABLE_DEBUG_HOTKEYS,
    &nams::INGAME_SETTINGS,
];

pub const LOADING_FIELDS: [FieldRef; 9] = [
    &nams::DISABLE_PLUGIN_LOADING,
    &nams::DISABLE_MAIN_OVERLAY,
    &nams::DISABLE_CONTENT_FEATURES,
    &nams::DISABLE_RESHADE_LOADING,
    &nams::DISABLE_3DMIGOTO_LOADING,
    &nams::DISABLE_GRAPHIC_MODS_LOADING,
    &nams::DISABLE_TEXTURE_INJECTION,
    &nams::DISABLE_SPLASH_SCREEN,
    &nams::SKIP_STARTUP_LOGOS,
];

pub const CONTENT_TAIL_FIELDS: [FieldRef; 2] = [
    &nams::OUTFIT_SWAP_VISUAL_EFFECTS,
    &nams::EXPERIMENTAL_DEFAULT_OUTFITS,
];

pub fn content_fields() -> Vec<FieldRef> {
    let mut fields: Vec<FieldRef> = content::fields().map(|field| field as FieldRef).collect();
    fields.extend(CONTENT_TAIL_FIELDS);
    fields
}

pub fn grid_columns(width: Pixels) -> usize {
    let width = f32::from(width);
    if width >= 1100. {
        4
    } else if width >= 820. {
        3
    } else if width >= 520. {
        2
    } else {
        1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheatTableNotice {
    Converted(String),
    NothingToFix,
    Failed,
}

impl CheatTableNotice {
    pub fn from_result(result: Result<CheatTableOutcome, yp_builders::CheatTableError>) -> Self {
        match result {
            Ok(CheatTableOutcome::Converted { output, .. }) => CheatTableNotice::Converted(
                output
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            Ok(CheatTableOutcome::NothingToReplace) => CheatTableNotice::NothingToFix,
            Err(_) => CheatTableNotice::Failed,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, CheatTableNotice::Converted(_))
    }
}

pub struct NamsTab {
    config: Entity<ConfigStore>,
    scroll: ScrollHandle,
    cheat_table: Option<CheatTableNotice>,
    convert_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl NamsTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = cx.stores().config.clone();
        let subscriptions = vec![cx.observe(&config, |_, _, cx| cx.notify())];
        Self {
            config,
            scroll: ScrollHandle::new(),
            cheat_table: None,
            convert_task: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn scroll_handle(&self) -> &ScrollHandle {
        &self.scroll
    }

    pub fn cheat_table_notice(&self) -> Option<&CheatTableNotice> {
        self.cheat_table.as_ref()
    }

    pub fn convert_cheat_table(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let job = cx.background_spawn(async move { yp_builders::cheat_table::convert(&path) });
        self.convert_task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| {
                this.cheat_table = Some(CheatTableNotice::from_result(result));
                cx.notify();
            })
            .ok();
        }));
    }

    fn pick_cheat_table(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        self.convert_task = Some(cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |this, cx| this.convert_cheat_table(path, cx))
                .ok();
        }));
    }

    fn sink(&self, field: FieldRef) -> ValueSink {
        let config = self.config.clone();
        let meta = *field.meta();
        Rc::new(move |value, _, cx| {
            config.update(cx, |store, cx| {
                store.update_nams(meta.key, value, meta.section, cx)
            });
        })
    }

    fn row(&self, field: FieldRef, values: &toml::Table) -> FieldRow {
        FieldRow::toml(SCOPE, field, values).sink(self.sink(field))
    }

    fn general_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let notice = self.cheat_table.clone();
        card("nams-general", strings.card_general(), cx)
            .changed(changed_in(&GENERAL_FIELDS, values, strings))
            .child(self.row(&nams::VALIDATE_MODEL_DATA, values))
            .child(self.row(&nams::VALIDATE_SCRIPTS, values))
            .child(self.row(&nams::LOADING_STALL_HINTS, values))
            .child(self.row(&nams::FIX_WIND_TIMER_BUG, values))
            .child(self.row(&nams::DISABLE_DEBUG_HOTKEYS, values))
            .child(self.row(&nams::INGAME_SETTINGS, values))
            .child(row_divider(window, cx))
            .child(muted_text(strings.cheat_table_convert_desc(), false, window, cx))
            .child(
                h_flex().child(
                    Button::new("nams-cheat-table", strings.cheat_table_convert_button())
                        .controller_support(false)
                        .style(ButtonStyle::Tinted(TintColor::Accent))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.pick_cheat_table(window, cx)
                        })),
                ),
            )
            .when_some(notice, |this, notice| {
                let success = notice.is_success();
                let text = match &notice {
                    CheatTableNotice::Converted(file) => {
                        SharedString::from(strings.cheat_table_convert_success(file))
                    }
                    CheatTableNotice::NothingToFix => {
                        SharedString::new_static(strings.cheat_table_convert_none())
                    }
                    CheatTableNotice::Failed => {
                        SharedString::new_static(strings.cheat_table_convert_error())
                    }
                };
                let color = if success {
                    Color::Success
                } else {
                    Color::Warning
                };
                this.child(
                    h_flex()
                        .w_full()
                        .items_start()
                        .gap(sizes.spacing_md)
                        .pt(sizes.padding_sm)
                        .child(
                            Icon::new(if success {
                                IconName::CheckCircleOutline
                            } else {
                                IconName::InfoOutline
                            })
                            .size(IconSize::Small)
                            .color(color),
                        )
                        .child(
                            div().flex_1().min_w_0().child(
                                Label::new(text)
                                    .size(LabelSize::XSmall)
                                    .color(color)
                                    .line_height(1.35),
                            ),
                        ),
                )
            })
            .into_any_element()
    }

    fn loading_card(&self, values: &toml::Table, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        card("nams-loading", strings.card_loading(), cx)
            .changed(changed_in(&LOADING_FIELDS, values, strings))
            .children(LOADING_FIELDS.iter().map(|field| self.row(*field, values)))
            .into_any_element()
    }

    fn content_card(
        &self,
        values: &toml::Table,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let available = sizes.window_width
            - sizes.sidebar_width
            - sizes.content_padding * 2.
            - sizes.card_padding_h * 2.;
        let columns = grid_columns(available);
        let mut card = card("nams-content", strings.card_content_features(), cx)
            .changed(changed_in(&content_fields(), values, strings))
            .child(muted_text(
                strings.content_features_description(),
                true,
                window,
                cx,
            ));
        for group in &content::GROUPS {
            card = card.child(subheading(group.title.text(strings), window, cx));
            let mut grid = v_flex().w_full().pb(sizes.padding_md);
            for chunk in group.fields.chunks(columns) {
                let mut line = h_flex().w_full().items_start().gap(sizes.spacing_lg);
                for field in chunk {
                    let field: FieldRef = *field;
                    line = line.child(
                        div()
                            .flex_basis(relative(0.))
                            .flex_grow_1()
                            .flex_shrink_1()
                            .min_w_0()
                            .child(self.row(field, values)),
                    );
                }
                for _ in chunk.len()..columns {
                    line = line.child(div().flex_basis(relative(0.)).flex_grow_1().flex_shrink_1());
                }
                grid = grid.child(line);
            }
            card = card.child(grid);
        }
        card.child(row_divider(window, cx))
            .children(
                CONTENT_TAIL_FIELDS
                    .iter()
                    .map(|field| self.row(*field, values)),
            )
            .into_any_element()
    }
}

impl Render for NamsTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let store = self.config.read(cx);
        let loading = store.is_loading();
        let dirty = store.is_dirty();
        let values = store.nams().clone();
        let error = store
            .error(ConfigFile::Nams)
            .map(|error| toml_error_text(error, strings));
        let failure = store
            .failure()
            .map(|failure| config_failure_text(failure, strings));
        let path = store.path(ConfigFile::Nams);
        let save_store = self.config.clone();
        let discard_store = self.config.clone();
        let header = PageHeader::new(strings.header_nams())
            .dirty(dirty)
            .save_discard(
                move |_, cx| save_store.update(cx, |store, cx| store.save(cx)),
                move |_, cx| discard_store.update(cx, |store, cx| store.discard(cx)),
            );
        let header = match path {
            Some(path) => header.info(strings.tooltip_edits_nams_toml(), move |_, cx| {
                reveal(path.clone(), true, cx)
            }),
            None => header.info_tooltip(strings.tooltip_edits_nams_toml()),
        };
        let content = if loading {
            div().into_any_element()
        } else {
            let general = self.general_card(&values, window, cx);
            let loading_card = self.loading_card(&values, cx);
            let content_card = self.content_card(&values, window, cx);
            let mut sections: Vec<AnyElement> = Vec::new();
            if let Some(error) = error {
                sections.push(
                    ConfigErrorBanner::new(ConfigFile::Nams.file_name(), error).into_any_element(),
                );
            }
            if let Some(failure) = failure {
                sections.push(failure_banner("nams-failure", failure).into_any_element());
            }
            sections.push(
                banner(IconName::InfoOutline, strings.nams_description(), window, cx)
                    .into_any_element(),
            );
            sections.push(
                TwoColumn::new(general, loading_card)
                    .id("nams-columns")
                    .into_any_element(),
            );
            sections.push(content_card);
            column(sections, window, cx)
            .into_any_element()
        };
        tab_page("nams-scroll", header, &self.scroll, loading, content, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_grid_columns_follow_the_available_width() {
        assert_eq!(grid_columns(px(1200.)), 4);
        assert_eq!(grid_columns(px(1100.)), 4);
        assert_eq!(grid_columns(px(900.)), 3);
        assert_eq!(grid_columns(px(600.)), 2);
        assert_eq!(grid_columns(px(300.)), 1);
    }

    #[test]
    fn cheat_table_results_map_to_the_three_messages() {
        let converted = CheatTableNotice::from_result(Ok(CheatTableOutcome::Converted {
            replacements: 3,
            output: PathBuf::from("C:/tables/nier-NAMS.CT"),
            encoding: yp_builders::TextEncoding::Utf8,
        }));
        assert_eq!(converted, CheatTableNotice::Converted("nier-NAMS.CT".into()));
        assert!(converted.is_success());
        assert_eq!(
            CheatTableNotice::from_result(Ok(CheatTableOutcome::NothingToReplace)),
            CheatTableNotice::NothingToFix
        );
        assert_eq!(
            CheatTableNotice::from_result(Err(yp_builders::CheatTableError::NotFound)),
            CheatTableNotice::Failed
        );
    }

    #[test]
    fn every_content_toggle_is_on_the_page_once() {
        let fields = content_fields();
        let mut keys: Vec<&str> = fields.iter().map(|field| field.meta().key).collect();
        let total = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), total);
        assert_eq!(total, content::fields().count() + 2);
    }
}
