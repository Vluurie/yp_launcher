use std::path::{Path, PathBuf};

use gpui::{PathPromptOptions, ScrollHandle, Subscription, Task};
use yp_builders::CheatTableOutcome;
use yp_config::{Kind, Setting};
use yp_state::ConfigStore;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_host::launch::nams_dll;
use yp_ui::{
    Button, ButtonStyle, Icon, IconButton, IconName, IconSize, Label, LabelSize, TintColor, Tooltip,
};

use crate::extras::CONTENT_GRID_PREFIX;
use crate::fields::Marker;
use crate::page::{banner, card, column, muted_text, row_divider, tab_page};
use crate::schema_page::{self, CardPlan, RowContext, TabScope};

const SCOPE: &str = "nams";
const FILE: &str = "nams.toml";
const CONTENT_CATEGORY: &str = "content";
pub const TAB: TabScope = TabScope::roots(&["native", "content", "thirdparty"]);

pub fn is_grid_setting(setting: &Setting) -> bool {
    setting.kind == Kind::Bool && setting.id.starts_with(CONTENT_GRID_PREFIX)
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

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub struct NamsTab {
    config: Entity<ConfigStore>,
    scroll: ScrollHandle,
    cheat_table: Option<CheatTableNotice>,
    convert_task: Option<Task<()>>,
    dll_rejected: bool,
    dll_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl NamsTab {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        let config = stores.config.clone();
        let subscriptions = vec![
            cx.observe(&config, |_, _, cx| cx.notify()),
            cx.observe(&stores.launch_options, |_, _, cx| cx.notify()),
            cx.observe(&stores.session, |_, _, cx| cx.notify()),
        ];
        Self {
            config,
            scroll: ScrollHandle::new(),
            cheat_table: None,
            convert_task: None,
            dll_rejected: false,
            dll_task: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn dll_rejected(&self) -> bool {
        self.dll_rejected
    }

    pub fn use_nams_dll(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.dll_rejected = !nams_dll::is_nams_dll(&path);
        if !self.dll_rejected {
            let options = cx.stores().launch_options.clone();
            options.update(cx, |options, cx| options.set_nams_dll(Some(path), cx));
        }
        cx.notify();
    }

    pub fn stop_using_nams_dll(&mut self, cx: &mut Context<Self>) {
        self.dll_rejected = false;
        let options = cx.stores().launch_options.clone();
        options.update(cx, |options, cx| options.set_nams_dll(None, cx));
        cx.notify();
    }

    fn pick_nams_dll(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        self.dll_task = Some(cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |this, cx| this.use_nams_dll(path, cx)).ok();
        }));
    }

    fn nams_dll_card(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let stores = cx.stores();
        let options = stores.launch_options.read(cx);
        let dll = options.nams_dll().map(Path::to_path_buf);
        let changed: Vec<SharedString> = dll
            .iter()
            .map(|dll| SharedString::from(file_name(dll)))
            .collect();
        let pick_label = if dll.is_some() {
            strings.nams_dll_change()
        } else {
            strings.nams_dll_pick()
        };
        let pick = Button::new("nams-dll-pick", pick_label)
            .controller_support(false)
            .style(if dll.is_some() {
                ButtonStyle::Outlined
            } else {
                ButtonStyle::Tinted(TintColor::Accent)
            })
            .on_click(cx.listener(|this, _, _, cx| this.pick_nams_dll(cx)));
        let chosen = dll.map(|dll| {
            h_flex()
                .w_full()
                .items_center()
                .gap(sizes.spacing_md)
                .pb(sizes.padding_sm)
                .child(
                    Icon::new(IconName::ExtensionOutlined)
                        .size(IconSize::Small)
                        .color(Color::Accent),
                )
                .child(
                    div().flex_1().min_w_0().child(
                        Label::new(dll.to_string_lossy().into_owned())
                            .size(LabelSize::Small)
                            .truncate(),
                    ),
                )
                .child(
                    IconButton::new("nams-dll-remove", IconName::Close)
                        .icon_size(IconSize::Small)
                        .icon_color(Color::Secondary)
                        .tooltip(Tooltip::text(strings.nams_dll_remove()))
                        .on_click(cx.listener(|this, _, _, cx| this.stop_using_nams_dll(cx))),
                )
        });
        card("nams-dll", strings.nams_dll_title(), cx)
            .changed(changed)
            .child(muted_text(strings.nams_dll_description(), false, window, cx))
            .children(chosen)
            .child(h_flex().pb(sizes.padding_sm).child(pick))
            .when(self.dll_rejected, |this| {
                this.child(
                    Label::new(strings.nams_dll_not_a_dll())
                        .size(LabelSize::XSmall)
                        .color(Color::Warning),
                )
            })
            .into_any_element()
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

    pub fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = schema_page::reset_text(&TAB, self.config.read(cx), cx.t());
        schema_page::confirm_reset(TAB, &text, &self.config, window, cx);
    }

    fn cheat_table_card(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let notice = self.cheat_table.clone();
        card("nams-cheat-table", strings.card_cheat_engine(), cx)
            .child(muted_text(strings.cheat_table_convert_desc(), false, window, cx))
            .child(
                h_flex().child(
                    Button::new("nams-cheat-table-convert", strings.cheat_table_convert_button())
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

    fn content_card(&self, plan: CardPlan, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let sizes = window.sizes(cx);
        let available = sizes.window_width
            - sizes.sidebar_width
            - sizes.content_padding * 2.
            - sizes.card_padding_h * 2.;
        let columns = grid_columns(available);
        let (grid_settings, settings): (Vec<Setting>, Vec<Setting>) =
            plan.settings.iter().cloned().partition(is_grid_setting);
        let mut context = RowContext::default();
        let mut rows: Vec<AnyElement> = settings
            .iter()
            .flat_map(|setting| {
                context.field(SCOPE, setting, &self.config, Marker::RestartOnly, window, cx)
            })
            .collect();
        if !grid_settings.is_empty() {
            if !rows.is_empty() {
                rows.push(row_divider(window, cx).into_any_element());
            }
            let mut grid = v_flex().w_full().pb(sizes.padding_md);
            for chunk in grid_settings.chunks(columns) {
                let mut line = h_flex().w_full().items_start().gap(sizes.spacing_lg);
                for setting in chunk {
                    line = line.child(
                        div()
                            .flex_basis(relative(0.))
                            .flex_grow_1()
                            .flex_shrink_1()
                            .min_w_0()
                            .child(schema_page::field(
                                SCOPE,
                                setting,
                                &self.config,
                                Marker::RestartOnly,
                                cx,
                            )),
                    );
                }
                for _ in chunk.len()..columns {
                    line = line.child(div().flex_basis(relative(0.)).flex_grow_1().flex_shrink_1());
                }
                grid = grid.child(line);
            }
            rows.push(grid.into_any_element());
        }
        let advanced = schema_page::generic_rows(
            SCOPE,
            &plan.advanced,
            &self.config,
            Marker::RestartOnly,
            window,
            cx,
        );
        schema_page::finish_card(plan, rows, advanced, window, cx).into_any_element()
    }
}

impl Render for NamsTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let store = self.config.read(cx);
        let loading = schema_page::is_loading(store);
        let reset = TAB.has_changes(store).then(|| {
            let text = schema_page::reset_text(&TAB, store, strings);
            schema_page::reset_button("nams-reset", TAB, text, &self.config, cx)
        });
        let header = schema_page::header(
            strings.header_nams(),
            strings.tooltip_edits_nams_toml(),
            Some(FILE),
            reset,
            &self.config,
            cx,
        );
        let content = if loading {
            div().into_any_element()
        } else {
            let mut sections = schema_page::problems(SCOPE, &TAB, &self.config, cx);
            sections.push(
                banner(IconName::InfoOutline, strings.nams_description(), window, cx)
                    .into_any_element(),
            );
            let plans = schema_page::plans(SCOPE, &TAB, self.config.read(cx));
            let mut cards: Vec<AnyElement> = Vec::new();
            let mut wide: Vec<AnyElement> = Vec::new();
            for plan in plans {
                if plan.category.id == CONTENT_CATEGORY {
                    wide.push(self.content_card(plan, window, cx));
                } else {
                    cards.push(schema_page::generic_card(
                        SCOPE,
                        plan,
                        &self.config,
                        Marker::RestartOnly,
                        window,
                        cx,
                    ));
                }
            }
            cards.insert(0, self.nams_dll_card(window, cx));
            cards.push(self.cheat_table_card(window, cx));
            sections.push(schema_page::spread(cards, "nams-columns", window, cx));
            sections.extend(wide);
            column(sections, window, cx).into_any_element()
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
    fn content_features_go_into_the_grid_and_switches_stay_rows() {
        let schema: yp_config::Schema =
            serde_json::from_value(yp_config::testing::sample_schema()).unwrap();
        let grid: Vec<&str> = schema
            .settings_in(CONTENT_CATEGORY)
            .filter(|setting| is_grid_setting(setting))
            .map(|setting| setting.id.as_str())
            .collect();
        assert_eq!(grid, vec!["nams.content.custom_items"]);
        let master = schema.setting("nams.experimental_default_outfits").unwrap();
        assert!(!is_grid_setting(master));
    }
}
