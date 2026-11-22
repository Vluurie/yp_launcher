use gpui::{
    AnyElement, App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ListAlignment, ListOffset, ListState, Render, SharedString, Subscription, Window,
};
use yp_state::logs::{LOG_LEVELS, LogEntry, LogLevel, LogTab};
use yp_state::prelude::*;
use yp_state::{LogEvent, LogStore};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, Chip, ChipSize, ChipTone, EmptyState, IconSize, Indicator, Input, InputEvent,
    InputState, LabelSize, LoadingState, SelectableText, Sheet, Sizes, ToggleButtonGroup,
    ToggleOption, Tooltip, VirtualList,
};

use crate::folders;
use crate::metrics;

const MODULE_SEPARATOR: &str = "::";

pub fn short_module(module: &str) -> &str {
    module.rsplit(MODULE_SEPARATOR).next().unwrap_or(module)
}

pub fn level_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Error => Color::Error,
        LogLevel::Warn => Color::Warning,
        LogLevel::Info => Color::Success,
        LogLevel::Debug => Color::LogDebug,
        LogLevel::Trace => Color::Muted,
    }
}

pub fn level_tone(level: LogLevel) -> ChipTone {
    match level {
        LogLevel::Error => ChipTone::Error,
        LogLevel::Warn => ChipTone::Warning,
        LogLevel::Info => ChipTone::Success,
        LogLevel::Debug => ChipTone::Info,
        LogLevel::Trace => ChipTone::Muted,
    }
}

pub fn level_chip_label(level: LogLevel, count: usize) -> SharedString {
    if count > 0 {
        format!("{} {count}", level.as_str()).into()
    } else {
        SharedString::new_static(level.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListSync {
    Keep,
    Append { from: usize, added: usize },
    Reset { count: usize },
}

pub fn list_sync(shown: usize, available: usize, appended: bool) -> ListSync {
    if shown == available {
        ListSync::Keep
    } else if appended && available > shown {
        ListSync::Append {
            from: shown,
            added: available - shown,
        }
    } else {
        ListSync::Reset { count: available }
    }
}

pub struct LogPanelView {
    logs: Entity<LogStore>,
    focus_handle: FocusHandle,
    search: Entity<InputState>,
    list: ListState,
    wrap: bool,
    follow: bool,
    appended: bool,
    _subscriptions: Vec<Subscription>,
}

impl LogPanelView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let logs = cx.stores().logs.clone();
        let placeholder = cx.t().log_search_placeholder();
        let query = logs.read(cx).query().to_owned();
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder)
                .default_value(query)
        });
        let count = logs.read(cx).filtered().len();
        let list = ListState::new(count, ListAlignment::Top, metrics::LOG_ROW_ESTIMATE * 8.);
        let weak = cx.weak_entity();
        list.set_scroll_handler(move |event, _, cx| {
            let at_end = event.visible_range.end >= event.count;
            weak.update(cx, |this, _| this.follow = at_end).ok();
        });
        let subscriptions = vec![
            cx.subscribe(&search, |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let query = input.read(cx).value().to_owned();
                    this.logs
                        .update(cx, |logs, cx| logs.set_search_query(query, cx));
                }
            }),
            cx.subscribe(&logs, |this, _, event: &LogEvent, _| match event {
                LogEvent::Appended { .. } => this.appended = true,
                LogEvent::Reset => this.appended = false,
                LogEvent::StreamingChanged(_) => {}
            }),
            cx.observe(&logs, |this, _, cx| {
                this.sync_list(cx);
                cx.notify();
            }),
        ];
        let mut this = Self {
            logs,
            focus_handle: cx.focus_handle(),
            search,
            list,
            wrap: true,
            follow: true,
            appended: false,
            _subscriptions: subscriptions,
        };
        this.scroll_to_end();
        this
    }

    fn scroll_to_end(&mut self) {
        self.list.scroll_to(ListOffset {
            item_ix: self.list.item_count(),
            offset_in_item: px(0.),
        });
        self.follow = true;
    }

    fn sync_list(&mut self, cx: &mut Context<Self>) {
        let available = self.logs.read(cx).filtered().len();
        match list_sync(self.list.item_count(), available, self.appended) {
            ListSync::Keep => {}
            ListSync::Append { from, added } => {
                let follow = self.follow;
                self.list.splice(from..from, added);
                if follow {
                    self.scroll_to_end();
                }
            }
            ListSync::Reset { count } => {
                self.list.reset(count);
                self.scroll_to_end();
            }
        }
        self.appended = false;
    }

    fn toggle_wrap(&mut self, cx: &mut Context<Self>) {
        self.wrap = !self.wrap;
        let count = self.list.item_count();
        let follow = self.follow;
        self.list.reset(count);
        if follow {
            self.scroll_to_end();
        }
        cx.notify();
    }

    fn header(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let logs = self.logs.read(cx);
        let total = logs.active_entries().len();
        let streaming = logs.is_streaming();
        let logs_dir = logs.logs_dir().to_path_buf();
        let action = |id: &'static str, icon: IconName, tooltip: &'static str| {
            IconButton::new(id, icon)
                .icon_size(IconSize::Medium)
                .icon_color(Color::Secondary)
                .size(ButtonSize::Compact)
                .tooltip(Tooltip::text(tooltip))
        };
        h_flex()
            .w_full()
            .flex_none()
            .gap(sizes.spacing_lg)
            .px(sizes.card_padding_h)
            .py(sizes.card_padding_v)
            .bg(theme.colors.log_header_background)
            .border_b_1()
            .border_color(theme.colors.border_light)
            .child(Label::new(strings.log_viewer_title()).size(LabelSize::XLarge).bold())
            .child(
                Label::new(format!("{total} {}", strings.entries_suffix()))
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            )
            .when(streaming, |this| {
                this.child(
                    h_flex()
                        .flex_none()
                        .child(Indicator::dot().tone(ChipTone::Success).pulse(true))
                        .child(
                            Chip::new(strings.log_live_badge())
                                .tone(ChipTone::Success)
                                .size(ChipSize::Small),
                        ),
                )
            })
            .child(div().flex_1())
            .child(
                action(
                    "shell-log-wrap",
                    if self.wrap {
                        IconName::WrapText
                    } else {
                        IconName::ShortText
                    },
                    if self.wrap {
                        strings.log_wrap_off()
                    } else {
                        strings.log_wrap_on()
                    },
                )
                .on_click(cx.listener(|this, _, _, cx| this.toggle_wrap(cx))),
            )
            .child(
                action(
                    "shell-log-bottom",
                    IconName::VerticalAlignBottom,
                    strings.scroll_to_bottom(),
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.scroll_to_end();
                    cx.notify();
                })),
            )
            .child(
                action(
                    "shell-log-folder",
                    IconName::FolderOpen,
                    strings.open_logs_folder(),
                )
                .controller_support(false)
                .on_click(move |_, _, cx| folders::open_dir(logs_dir.clone(), cx)),
            )
            .child(
                action("shell-log-close", IconName::Close, strings.tooltip_close())
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(CloseLogs), cx)),
            )
    }

    fn tab_bar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let logs = self.logs.read(cx);
        let selected = match logs.active_tab() {
            LogTab::Modloader => 0,
            LogTab::Yorha => 1,
        };
        let store = self.logs.clone();
        h_flex()
            .w_full()
            .flex_none()
            .px(sizes.card_padding_h)
            .py(sizes.spacing_md)
            .bg(theme.colors.log_header_background)
            .border_b_1()
            .border_color(theme.colors.border_light)
            .child(
                ToggleButtonGroup::new("shell-log-tabs")
                    .option_with(
                        ToggleOption::new(strings.tab_modloader_label())
                            .count(logs.entries(LogTab::Modloader).len()),
                    )
                    .option_with(
                        ToggleOption::new(strings.tab_yorha_label())
                            .count(logs.entries(LogTab::Yorha).len()),
                    )
                    .selected(selected)
                    .size(ButtonSize::Medium)
                    .on_select(move |index, _, cx| {
                        let tab = if *index == 0 {
                            LogTab::Modloader
                        } else {
                            LogTab::Yorha
                        };
                        store.update(cx, |logs, cx| logs.set_active_tab(tab, cx));
                    }),
            )
    }

    fn filter_bar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let logs = self.logs.read(cx);
        let counts = logs.counts();
        let searching = !logs.query().is_empty();
        let ratio = format!("{}/{}", logs.filtered().len(), logs.active_entries().len());
        let mut chips = h_flex().w_full().flex_wrap().gap(sizes.spacing_md);
        for level in LOG_LEVELS {
            let shown = logs.is_level_shown(level);
            let count = counts.get(&level).copied().unwrap_or(0);
            let store = self.logs.clone();
            chips = chips.child(
                Chip::new(level_chip_label(level, count))
                    .id(("shell-log-level", level as usize))
                    .tone(if shown {
                        level_tone(level)
                    } else {
                        ChipTone::Muted
                    })
                    .outlined(!shown)
                    .selected(shown)
                    .on_click(move |_, _, cx| {
                        store.update(cx, |logs, cx| logs.toggle_level(level, cx));
                    }),
            );
        }
        v_flex()
            .w_full()
            .flex_none()
            .gap(sizes.spacing_md)
            .px(sizes.card_padding_h)
            .py(sizes.spacing_md)
            .bg(theme.colors.log_header_background)
            .border_b_1()
            .border_color(theme.colors.border_light)
            .child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .child(
                        v_flex().flex_1().min_w_0().child(
                            Input::new(&self.search)
                                .prefix(
                                    Icon::new(IconName::Search)
                                        .size(IconSize::Small)
                                        .color(Color::Muted),
                                )
                                .cleanable(true),
                        ),
                    )
                    .when(searching, |this| {
                        this.child(
                            Label::new(ratio)
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .mono(),
                        )
                    }),
            )
            .child(chips)
    }

    fn body(&self, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let logs = self.logs.read(cx);
        if logs.is_loading() && logs.active_entries().is_empty() {
            return LoadingState::new().into_any_element();
        }
        if logs.filtered().is_empty() {
            let message = if logs.query().is_empty() {
                strings.no_log_entries()
            } else {
                strings.no_log_matches()
            };
            return EmptyState::new(message).into_any_element();
        }
        let wrap = self.wrap;
        VirtualList::list("shell-log-list", &self.list, move |index, window, cx| {
            let entry = cx.stores().logs.read(cx).filtered().get(index).cloned();
            match entry {
                Some(entry) => render_entry(index, &entry, wrap, window, cx).into_any_element(),
                None => div().into_any_element(),
            }
        })
        .into_any_element()
    }
}

fn render_entry(
    index: usize,
    entry: &LogEntry,
    wrap: bool,
    window: &Window,
    cx: &App,
) -> impl IntoElement + use<> {
    let theme = cx.theme().clone();
    let sizes = window.sizes(cx);
    let color = level_color(entry.level);
    let highlighted = matches!(entry.level, LogLevel::Warn | LogLevel::Error);
    let tint = color.color(cx);
    let background = if highlighted {
        tint.opacity(theme.alpha.faint)
    } else if index % 2 == 1 {
        theme.colors.surface_light.opacity(theme.alpha.overlay)
    } else {
        theme.colors.transparent
    };
    v_flex()
        .w_full()
        .gap(sizes.spacing_xs)
        .px(sizes.padding_md)
        .py(sizes.spacing_md)
        .bg(background)
        .border_l(Sizes::ACCENT_BAR_WIDTH)
        .border_color(if highlighted {
            tint
        } else {
            theme.colors.transparent
        })
        .child(
            h_flex()
                .w_full()
                .gap(sizes.spacing_lg)
                .child(
                    Label::new(entry.timestamp.clone())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .mono(),
                )
                .child(
                    Label::new(entry.level.as_str())
                        .size(LabelSize::XSmall)
                        .color(color)
                        .bold()
                        .mono(),
                )
                .child(
                    Label::new(short_module(&entry.module).to_owned())
                        .size(LabelSize::XSmall)
                        .color(Color::AccentSecondary)
                        .mono()
                        .truncate(),
                ),
        )
        .map(|this| {
            if wrap {
                this.child(
                    SelectableText::new(("shell-log-message", index), entry.message.clone())
                        .mono()
                        .size(LabelSize::XSmall)
                        .color(Color::LogText),
                )
            } else {
                this.child(
                    Label::new(entry.message.clone())
                        .size(LabelSize::XSmall)
                        .color(Color::LogText)
                        .mono()
                        .truncate(),
                )
            }
        })
}

impl EventEmitter<DismissEvent> for LogPanelView {}

impl Focusable for LogPanelView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LogPanelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        Sheet::new("shell-log-sheet")
            .width(sizes.window_width / 2.)
            .resizable(true)
            .on_dismiss(|window, cx| window.dispatch_action(Box::new(CloseLogs), cx))
            .child(
                v_flex()
                    .size_full()
                    .bg(theme.colors.log_background)
                    .child(self.header(window, cx))
                    .child(self.tab_bar(window, cx))
                    .child(self.filter_bar(window, cx))
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .child(v_flex().absolute().inset_0().child(self.body(cx))),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_names_are_cut_to_the_last_segment() {
        assert_eq!(short_module("nier_rs::content::quests"), "quests");
        assert_eq!(short_module("modloader"), "modloader");
        assert_eq!(short_module(""), "");
    }

    #[test]
    fn level_chips_show_counts_only_when_there_are_entries() {
        assert_eq!(level_chip_label(LogLevel::Error, 0).as_ref(), "ERROR");
        assert_eq!(level_chip_label(LogLevel::Warn, 12).as_ref(), "WARN 12");
    }

    #[test]
    fn list_follows_appends_and_resets_on_filter_changes() {
        assert_eq!(list_sync(10, 10, true), ListSync::Keep);
        assert_eq!(
            list_sync(10, 14, true),
            ListSync::Append { from: 10, added: 4 }
        );
        assert_eq!(list_sync(10, 14, false), ListSync::Reset { count: 14 });
        assert_eq!(list_sync(10, 3, true), ListSync::Reset { count: 3 });
    }
}
