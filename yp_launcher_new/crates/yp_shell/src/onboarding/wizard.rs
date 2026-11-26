use std::path::PathBuf;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Context, Div, EventEmitter, IntoElement,
    PathPromptOptions, Render, ScrollHandle, SharedString, Task, Window,
};
use yp_core::Os;
use yp_state::prelude::*;
use yp_state::{ModInstallReport, ModInstallRequest, Registry};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, Callout, Card, Checkbox, Chip, ChipTone, DropProgress, DropZone,
    Headline, HeadlineSize, IconSize, LabelSize, LanguageSelector, NierBackground,
    ScrollableElement, SelectableText, Severity, Sizes, Spinner, StepDots, ToggleState, easing,
};

use super::flow::{
    MigrationFindings, OnboardingFlow, STEP_COUNT, Step, detect_migration, migration_rows,
    summary_rows,
};
use crate::directory::{self, ManualRejection, OnboardingSearchPhase};
use crate::metrics;
use crate::text;

const FIRST_PLAYTHROUGH_ID: &str = "plugins.yorha_protocol.firstPlaythrough";
const SLIDE_FRACTION: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingEvent {
    Completed { play: bool },
}

pub struct OnboardingWizard {
    flow: OnboardingFlow,
    searching: Option<OnboardingSearchPhase>,
    installing: bool,
    install_error: Option<SharedString>,
    installed: Vec<SharedString>,
    transition: usize,
    scroll: ScrollHandle,
    search_task: Option<Task<()>>,
    detect_task: Option<Task<()>>,
    install_task: Option<Task<()>>,
    finish_task: Option<Task<()>>,
}

impl EventEmitter<OnboardingEvent> for OnboardingWizard {}

impl OnboardingWizard {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let selected = cx.stores().session.read(cx).selected_dir();
        Self {
            flow: OnboardingFlow::new(selected),
            searching: None,
            installing: false,
            install_error: None,
            installed: Vec::new(),
            transition: 0,
            scroll: ScrollHandle::new(),
            search_task: None,
            detect_task: None,
            install_task: None,
            finish_task: None,
        }
    }

    pub fn flow(&self) -> &OnboardingFlow {
        &self.flow
    }

    pub fn step(&self) -> Step {
        self.flow.step()
    }

    fn entered_step(&mut self, cx: &mut Context<Self>) {
        self.transition += 1;
        self.scroll = ScrollHandle::new();
        if self.flow.step() == Step::FirstPlaythrough {
            let config = cx.stores().config.read(cx);
            let saved = if config.values().not_in_file.contains(FIRST_PLAYTHROUGH_ID) {
                true
            } else {
                config.bool(FIRST_PLAYTHROUGH_ID)
            };
            self.flow.set_first_playthrough(saved);
        }
        if self.flow.needs_detection() {
            self.run_detection(cx);
        }
        cx.notify();
    }

    pub fn advance(&mut self, cx: &mut Context<Self>) {
        if self.flow.advance() {
            self.entered_step(cx);
        }
    }

    pub fn go_back(&mut self, cx: &mut Context<Self>) {
        if self.flow.go_back() {
            self.entered_step(cx);
        }
    }

    pub fn show_step(&mut self, step: Step, cx: &mut Context<Self>) {
        self.flow.jump_to(step);
        self.entered_step(cx);
    }

    pub fn answer_had_mods(&mut self, had_mods: bool, cx: &mut Context<Self>) {
        self.flow.answer_had_mods(had_mods);
        self.entered_step(cx);
    }

    pub fn answer_wants_mod_install(&mut self, wants: bool, cx: &mut Context<Self>) {
        self.flow.answer_wants_mod_install(wants);
        self.entered_step(cx);
    }

    fn run_detection(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.flow.selected_path().map(|path| path.to_path_buf()) else {
            return;
        };
        let game = cx.env().game(&path);
        let detect = cx.background_spawn(async move { detect_migration(&game) });
        self.detect_task = Some(cx.spawn(async move |this, cx| {
            let findings = detect.await;
            this.update(cx, |this, cx| {
                this.flow.set_findings(&path, findings);
                cx.notify();
            })
            .ok();
        }));
    }

    fn path_selected(&mut self, path: Option<PathBuf>, cx: &mut Context<Self>) {
        self.searching = None;
        if let Some(path) = path {
            self.flow.set_selected_path(path);
        }
        cx.notify();
    }

    fn run_auto_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.searching.is_some() {
            return;
        }
        self.searching = Some(OnboardingSearchPhase::Fast);
        cx.notify();
        self.search_task = Some(cx.spawn_in(window, async move |this, cx| {
            let weak = this.clone();
            let found = directory::onboarding_search(
                move |phase, cx| {
                    weak.update(cx, |this, cx| {
                        this.searching = Some(phase);
                        cx.notify();
                    })
                    .ok();
                },
                cx,
            )
            .await;
            this.update(cx, |this, cx| this.path_selected(found, cx))
                .ok();
        }));
    }

    fn run_manual_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_task = Some(cx.spawn_in(window, async move |this, cx| {
            let found = directory::select_manually(ManualRejection::ShowAsToast, cx).await;
            this.update(cx, |this, cx| this.path_selected(found, cx))
                .ok();
        }));
    }

    pub fn choose_first_playthrough(&mut self, first: bool, cx: &mut Context<Self>) {
        self.flow.set_first_playthrough(first);
        cx.notify();
    }

    pub fn save_first_playthrough(&mut self, cx: &mut Context<Self>) {
        let value = self.flow.first_playthrough();
        let config = cx.stores().config.clone();
        config.update(cx, |config, cx| {
            config.set_now(FIRST_PLAYTHROUGH_ID, serde_json::Value::Bool(value), cx);
        });
        self.advance(cx);
    }

    pub fn installed(&self) -> &[SharedString] {
        &self.installed
    }

    pub(crate) fn install(
        &mut self,
        sources: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.installing || sources.is_empty() {
            return;
        }
        let strings = cx.t();
        if self.flow.selected_path().is_none() {
            self.install_error = Some(strings.onboarding_mod_install_no_game_dir().into());
            cx.notify();
            return;
        }
        let request = ModInstallRequest::guided(sources).with_texture_packs();
        let Some(running) = Registry::install_mods(request, window, cx) else {
            self.install_error = Some(strings.shell_tab_unavailable_body().into());
            cx.notify();
            return;
        };
        self.installing = true;
        self.install_error = None;
        cx.notify();
        self.install_task = Some(cx.spawn(async move |this, cx| {
            let report = running.await;
            this.update(cx, |this, cx| this.install_finished(report, cx))
                .ok();
        }));
    }

    fn install_finished(&mut self, report: ModInstallReport, cx: &mut Context<Self>) {
        let strings = cx.t();
        self.installing = false;
        self.installed
            .extend(report.installed.into_iter().map(SharedString::from));
        self.installed.extend(
            report
                .texture_packs
                .iter()
                .map(|name| SharedString::from(strings.onboarding_texture_pack_chip(name))),
        );
        self.install_error = report.failed.first().map(|(source, _)| {
            let name = source
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| source.to_string_lossy().into_owned());
            strings.onboarding_mod_install_failed(&name).into()
        });
        cx.notify();
    }

    fn browse(&mut self, folders: bool, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        let prompt = if folders {
            strings.onboarding_mod_install_folder_picker_title()
        } else {
            strings.onboarding_mod_install_picker_title()
        };
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: !folders,
            directories: folders,
            multiple: !folders,
            prompt: Some(prompt.into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            this.update_in(cx, |this, window, cx| this.install(paths, window, cx))
                .ok();
        })
        .detach();
    }

    pub fn finish(&mut self, play: bool, cx: &mut Context<Self>) {
        let wants_shortcut = self.flow.create_shortcut()
            && cx.env().platform.supports_desktop_shortcut()
            && self.flow.selected_path().is_some();
        let shortcut =
            wants_shortcut.then(|| cx.stores().session.read(cx).create_desktop_shortcut(cx));
        self.finish_task = Some(cx.spawn(async move |this, cx| {
            if let Some(shortcut) = shortcut {
                shortcut.await;
            }
            this.update(cx, |_, cx| {
                cx.prefs().set_onboarding_complete(true);
                cx.emit(OnboardingEvent::Completed { play });
            })
            .ok();
        }));
    }

    fn column(window: &Window, cx: &App) -> Div {
        let sizes = window.sizes(cx);
        v_flex().w_full().items_center().gap(sizes.padding_lg)
    }

    fn title(text: &'static str) -> impl IntoElement {
        Label::new(text)
            .size(LabelSize::XLarge)
            .color(Color::Accent)
            .bold()
            .centered()
    }

    fn primary(id: &'static str, label: &'static str) -> Button {
        Button::new(id, label)
            .style(ButtonStyle::Filled)
            .size(ButtonSize::Large)
    }

    fn back_button(&self, enabled: bool, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        Button::new("shell-onboarding-back", cx.t().button_back())
            .style(ButtonStyle::Transparent)
            .size(ButtonSize::Medium)
            .start_icon(IconName::ArrowBack)
            .disabled(!enabled)
            .on_click(cx.listener(|this, _, _, cx| this.go_back(cx)))
    }

    fn actions(window: &Window, cx: &App) -> Div {
        let sizes = window.sizes(cx);
        h_flex()
            .flex_wrap()
            .justify_center()
            .gap(sizes.padding_md)
            .pt(sizes.padding_sm)
    }

    fn welcome(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let strings = cx.t();
        Self::column(window, cx)
            .child(super::menu::heading(
                strings.onboarding_welcome_title(),
                window,
                cx,
            ))
            .child(
                Label::new(strings.onboarding_welcome_subtitle())
                    .size(LabelSize::Large)
                    .color(Color::Secondary)
                    .line_height(1.5)
                    .centered(),
            )
            .child(
                Self::actions(window, cx).child(if cx.theme().id == ThemeId::NierNew {
                    yp_ui::nier::DialogButton::new(
                        "shell-onboarding-start",
                        strings.button_get_started(),
                    )
                    .width(175.)
                    .scale(0.85)
                    .on_click(cx.listener(|this, _, _, cx| this.advance(cx)))
                    .into_any_element()
                } else {
                    Self::primary("shell-onboarding-start", strings.button_get_started())
                        .on_click(cx.listener(|this, _, _, cx| this.advance(cx)))
                        .into_any_element()
                }),
            )
    }

    fn select_game(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let column =
            Self::column(window, cx).child(Self::title(strings.onboarding_select_title()));
        let manual = |cx: &mut Context<Self>| {
            Button::new("shell-onboarding-manual", strings.button_select_manually())
                .controller_support(false)
                .style(ButtonStyle::Outlined)
                .size(ButtonSize::Medium)
                .on_click(cx.listener(|this, _, window, cx| this.run_manual_select(window, cx)))
        };
        if let Some(path) = self.flow.selected_path() {
            return column
                .child(
                    h_flex()
                        .w_full()
                        .gap(sizes.spacing_lg)
                        .px(sizes.padding_lg)
                        .py(sizes.padding_md)
                        .rounded(sizes.border_radius)
                        .bg(theme.colors.input_background)
                        .border_1()
                        .border_color(theme.colors.success)
                        .child(
                            Icon::new(IconName::CheckCircle)
                                .size(IconSize::Large)
                                .color(Color::Success),
                        )
                        .child(
                            div().flex_1().min_w_0().overflow_hidden().child(
                                SelectableText::new(
                                    "shell-onboarding-path",
                                    path.to_string_lossy().into_owned(),
                                )
                                .size(LabelSize::Small)
                                .color(Color::Default),
                            ),
                        ),
                )
                .child(
                    Self::actions(window, cx)
                        .child(self.back_button(true, cx))
                        .child(manual(cx))
                        .child(
                            Self::primary("shell-onboarding-continue", strings.button_continue())
                                .on_click(cx.listener(|this, _, _, cx| this.advance(cx))),
                        ),
                );
        }
        let column = match self.searching {
            Some(phase) => column.child(
                h_flex()
                    .gap(sizes.spacing_lg)
                    .child(Spinner::new().size(IconSize::Medium))
                    .child(
                        Label::new(match phase {
                            OnboardingSearchPhase::Fast => strings.onboarding_searching_nier(),
                            OnboardingSearchPhase::Deep => strings.onboarding_searching_drives(),
                        })
                        .size(LabelSize::Default)
                        .color(Color::Muted)
                        .italic(),
                    ),
            ),
            None => column
                .child(
                    Self::primary("shell-onboarding-auto", strings.button_auto_detect())
                        .start_icon(IconName::Search)
                        .full_width()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.run_auto_search(window, cx)),
                        ),
                )
                .child(manual(cx).full_width()),
        };
        column.child(Self::actions(window, cx).child(self.back_button(true, cx)))
    }

    fn first_playthrough(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let strings = cx.t();
        let first = self.flow.first_playthrough();
        Self::column(window, cx)
            .child(Self::title(strings.onboarding_first_playthrough_title()))
            .child(
                Label::new(strings.onboarding_first_playthrough_hint())
                    .size(LabelSize::Small)
                    .color(Color::Muted)
                    .centered(),
            )
            .child(super::menu::choice(
                "shell-onboarding-first-yes",
                strings.onboarding_first_yes(),
                first,
                window,
                cx,
                cx.listener(|this, _, _, cx| {
                    this.choose_first_playthrough(true, cx);
                }),
            ))
            .child(super::menu::choice(
                "shell-onboarding-first-no",
                strings.onboarding_first_no(),
                !first,
                window,
                cx,
                cx.listener(|this, _, _, cx| {
                    this.choose_first_playthrough(false, cx);
                }),
            ))
            .child(
                Self::actions(window, cx)
                    .child(self.back_button(true, cx))
                    .child(
                        Self::primary("shell-onboarding-continue", strings.button_continue())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.save_first_playthrough(cx);
                            })),
                    ),
            )
    }

    fn migration(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        if self.flow.had_mods_before() != Some(true) {
            return Self::column(window, cx)
                .child(Self::title(strings.onboarding_migration_ask_title()))
                .child(
                    Label::new(strings.onboarding_migration_ask_body())
                        .size(LabelSize::Small)
                        .color(Color::Secondary)
                        .centered(),
                )
                .child(super::menu::choice(
                    "shell-onboarding-migration-yes",
                    strings.onboarding_migration_yes(),
                    false,
                    window,
                    cx,
                    cx.listener(|this, _, _, cx| this.answer_had_mods(true, cx)),
                ))
                .child(super::menu::choice(
                    "shell-onboarding-migration-no",
                    strings.onboarding_migration_no(),
                    false,
                    window,
                    cx,
                    cx.listener(|this, _, _, cx| this.answer_had_mods(false, cx)),
                ))
                .child(Self::actions(window, cx).child(self.back_button(true, cx)));
        }
        let panel = v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .p(sizes.padding_lg)
            .rounded(sizes.border_radius)
            .bg(theme.colors.surface_medium);
        let panel = match self.flow.findings() {
            None => panel.items_center().child(Spinner::new()),
            Some(findings) => {
                let rows = migration_rows(findings, strings);
                if rows.is_empty() {
                    panel.child(
                        Label::new(strings.onboarding_migration_nothing_found())
                            .size(LabelSize::Small)
                            .color(Color::Secondary)
                            .centered(),
                    )
                } else {
                    panel.children(rows.into_iter().map(|row| {
                        h_flex()
                            .w_full()
                            .items_start()
                            .gap(sizes.spacing_lg)
                            .child(
                                Icon::new(if row.ok {
                                    IconName::CheckCircleOutline
                                } else {
                                    IconName::WarningAmberRounded
                                })
                                .size(IconSize::Medium)
                                .color(if row.ok {
                                    Color::Success
                                } else {
                                    Color::Warning
                                }),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(
                                        h_flex()
                                            .gap(sizes.spacing_md)
                                            .child(
                                                Icon::new(row.icon)
                                                    .size(IconSize::Small)
                                                    .color(Color::Muted),
                                            )
                                            .child(
                                                Label::new(row.label)
                                                    .size(LabelSize::Small)
                                                    .semibold(),
                                            ),
                                    )
                                    .child(
                                        Label::new(row.action)
                                            .size(LabelSize::XSmall)
                                            .color(Color::Secondary),
                                    ),
                            )
                    }))
                }
            }
        };
        Self::column(window, cx)
            .child(Self::title(strings.onboarding_migration_results_title()))
            .child(panel)
            .child(
                Self::actions(window, cx)
                    .child(self.back_button(true, cx))
                    .child(
                        Self::primary("shell-onboarding-continue", strings.button_continue())
                            .on_click(cx.listener(|this, _, _, cx| this.advance(cx))),
                    ),
            )
    }

    fn mods(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        if self.flow.wants_mod_install() != Some(true) {
            return Self::column(window, cx)
                .child(Self::title(strings.onboarding_mod_install_ask_title()))
                .child(
                    Label::new(strings.onboarding_mod_install_ask_body())
                        .size(LabelSize::Small)
                        .color(Color::Secondary)
                        .centered(),
                )
                .child(super::menu::choice(
                    "shell-onboarding-mods-yes",
                    strings.onboarding_mod_install_yes(),
                    false,
                    window,
                    cx,
                    cx.listener(|this, _, _, cx| {
                        this.answer_wants_mod_install(true, cx);
                    }),
                ))
                .child(super::menu::choice(
                    "shell-onboarding-mods-skip",
                    strings.onboarding_mod_install_skip(),
                    false,
                    window,
                    cx,
                    cx.listener(|this, _, _, cx| {
                        this.answer_wants_mod_install(false, cx);
                    }),
                ))
                .child(Self::actions(window, cx).child(self.back_button(true, cx)));
        }
        let busy = self.installing;
        let entity = cx.entity();
        let drop_target = entity.clone();
        let files_target = entity.clone();
        let folder_target = entity;
        let mut zone = DropZone::new("shell-onboarding-drop")
            .title(strings.drop_mod_here())
            .hint(strings.drop_mod_here_hint())
            .icon(IconName::DownloadForOfflineOutlined)
            .on_drop_paths(move |paths, window, cx| {
                let paths = paths.to_vec();
                drop_target.update(cx, |this, cx| this.install(paths, window, cx));
            })
            .on_browse_files(move |window, cx| {
                files_target.update(cx, |this, cx| this.browse(false, window, cx));
            })
            .on_browse_folder(move |window, cx| {
                folder_target.update(cx, |this, cx| this.browse(true, window, cx));
            });
        if busy {
            zone = zone.busy(DropProgress::new(strings.onboarding_mod_install_busy()));
        }
        let finish_label = if self.installed.is_empty() {
            strings.onboarding_mod_install_skip_button()
        } else {
            strings.onboarding_mod_install_done_button()
        };
        Self::column(window, cx)
            .child(
                h_flex()
                    .gap(sizes.spacing_md)
                    .child(
                        Icon::new(IconName::DownloadForOfflineOutlined)
                            .size(IconSize::Large)
                            .color(Color::Accent),
                    )
                    .child(
                        Label::new(strings.onboarding_mod_install_title())
                            .size(LabelSize::Large)
                            .color(Color::Accent)
                            .bold(),
                    ),
            )
            .child(
                v_flex()
                    .items_center()
                    .child(
                        Label::new(strings.onboarding_mod_install_body())
                            .size(LabelSize::Small)
                            .color(Color::Secondary)
                            .centered(),
                    )
                    .child(
                        Label::new(strings.onboarding_mod_install_sub_body())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .italic()
                            .centered(),
                    ),
            )
            .child(zone)
            .when_some(self.install_error.clone(), |this, error| {
                this.child(
                    Callout::new()
                        .id("shell-onboarding-install-error")
                        .severity(Severity::Error)
                        .title(error)
                        .title_size(LabelSize::XSmall),
                )
            })
            .when(!self.installed.is_empty(), |this| {
                this.child(
                    h_flex()
                        .w_full()
                        .flex_wrap()
                        .justify_center()
                        .gap(sizes.spacing_md)
                        .children(self.installed.iter().cloned().map(|name| {
                            Chip::new(name)
                                .icon(IconName::CheckCircle)
                                .tone(ChipTone::Success)
                                .truncate()
                        })),
                )
            })
            .child(
                Self::actions(window, cx)
                    .child(self.back_button(!busy, cx))
                    .child(
                        Self::primary("shell-onboarding-mods-done", finish_label)
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.advance(cx))),
                    ),
            )
    }

    fn ready(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let fallback = MigrationFindings::default();
        let rows = summary_rows(
            self.flow.selected_path(),
            self.flow.first_playthrough(),
            self.flow.findings().unwrap_or(&fallback),
            strings,
        );
        let shortcut = cx.env().platform.supports_desktop_shortcut();
        let create_shortcut = self.flow.create_shortcut();
        Self::column(window, cx)
            .child(
                Headline::new(strings.onboarding_ready_title())
                    .size(HeadlineSize::Hero)
                    .color(Color::Accent)
                    .centered(),
            )
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_md)
                    .p(sizes.padding_lg)
                    .rounded(sizes.border_radius)
                    .bg(theme.colors.surface_medium)
                    .children(rows.into_iter().map(|row| {
                        h_flex()
                            .w_full()
                            .gap(sizes.spacing_md)
                            .child(
                                Icon::new(row.icon)
                                    .size(IconSize::Small)
                                    .color(Color::Accent),
                            )
                            .child(
                                Label::new(strings.shell_summary_label(row.label))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Label::new(row.value).size(LabelSize::Small).truncate()),
                            )
                    })),
            )
            .when(shortcut, |this| {
                this.child(
                    Checkbox::new("shell-onboarding-shortcut", create_shortcut)
                        .label(strings.onboarding_create_shortcut())
                        .label_size(LabelSize::Small)
                        .on_click(cx.listener(|this, state: &ToggleState, _, cx| {
                            this.flow.set_create_shortcut(state.selected());
                            cx.notify();
                        })),
                )
            })
            .child(
                Button::new("shell-onboarding-play", strings.play_button())
                    .style(ButtonStyle::Filled)
                    .size(ButtonSize::Hero)
                    .on_click(cx.listener(|this, _, _, cx| this.finish(true, cx))),
            )
            .child(
                Button::new(
                    "shell-onboarding-skip-play",
                    strings.button_go_to_launcher(),
                )
                .style(ButtonStyle::Link)
                .size(ButtonSize::Medium)
                .color(Color::Muted)
                .on_click(cx.listener(|this, _, _, cx| this.finish(false, cx))),
            )
    }

    fn unsupported(&self, window: &Window, cx: &App) -> AnyElement {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let os: Os = cx.env().host.ui_os();
        NierBackground::new()
            .id("shell-onboarding-background")
            .child(
                v_flex()
                    .absolute()
                    .inset_0()
                    .items_center()
                    .justify_center()
                    .bg(theme.colors.error.opacity(theme.alpha.subtle))
                    .child(
                        v_flex()
                            .max_w(metrics::UNSUPPORTED_MAX_WIDTH)
                            .items_center()
                            .gap(sizes.padding_lg)
                            .p(sizes.padding_xl)
                            .child(
                                Icon::new(IconName::ErrorOutline)
                                    .size(IconSize::Custom(metrics::UNSUPPORTED_ICON_SIZE))
                                    .color(Color::Error),
                            )
                            .child(
                                Headline::new(strings.platform_unsupported_title())
                                    .color(Color::Error)
                                    .centered(),
                            )
                            .child(
                                Label::new(text::platform_unsupported_text(os, strings))
                                    .size(LabelSize::Small)
                                    .color(Color::Secondary)
                                    .line_height(1.5)
                                    .centered(),
                            ),
                    ),
            )
            .into_any_element()
    }
}

impl Render for OnboardingWizard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !cx.env().can_launch_game() {
            return self.unsupported(window, cx);
        }
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let step = self.flow.step();
        let content = match step {
            Step::Welcome => self.welcome(window, cx),
            Step::SelectGame => self.select_game(window, cx),
            Step::FirstPlaythrough => self.first_playthrough(window, cx),
            Step::Migration => self.migration(window, cx),
            Step::Mods => self.mods(window, cx),
            Step::Ready => self.ready(window, cx),
        };
        let duration = theme.motion.page;
        let travel = f32::from(metrics::ONBOARDING_CARD_MAX_WIDTH) * SLIDE_FRACTION;
        let card = v_flex()
            .relative()
            .w_full()
            .max_w(metrics::ONBOARDING_CARD_MAX_WIDTH)
            .child(
                Card::new().translucent(true).child(
                    v_flex()
                        .w_full()
                        .px(sizes.padding_xl)
                        .py(sizes.padding_lg)
                        .child(content),
                ),
            )
            .with_animation(
                ("shell-onboarding-step", self.transition),
                Animation::new(duration),
                move |this, delta| {
                    this.opacity(easing::nier_fill(delta))
                        .left(px(travel * (1. - easing::nier_fill(delta))))
                },
            );
        let stage = div()
            .id("shell-onboarding-scroll")
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .child(
                v_flex()
                    .w_full()
                    .min_h_full()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .px(sizes.padding_xl)
                    .pt(Sizes::TITLE_BAR_HEIGHT + sizes.padding_lg)
                    .pb(sizes.padding_lg)
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(metrics::ONBOARDING_MAX_WIDTH)
                            .items_center()
                            .child(card),
                    ),
            )
            .overflow_y_scrollbar(&self.scroll);
        NierBackground::new()
            .id("shell-onboarding-background")
            .animate(window.is_window_active())
            .child(
                v_flex()
                    .absolute()
                    .inset_0()
                    .items_center()
                    .child(stage)
                    .child(
                        div()
                            .flex_none()
                            .pt(sizes.padding_md)
                            .pb(Sizes::NIER_BORDER_INSET
                                + Sizes::NIER_BORDER_TILE_HEIGHT
                                + sizes.padding_md)
                            .child(StepDots::new(STEP_COUNT, step.index())),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(sizes.padding_md)
                    .left(sizes.info_bar_padding_h)
                    .px(sizes.chip_padding_h)
                    .py(sizes.chip_padding_v)
                    .rounded(sizes.border_radius)
                    .bg(theme.colors.background_card)
                    .border_1()
                    .border_color(theme.colors.border_light)
                    .child(
                        LanguageSelector::new()
                            .id("shell-onboarding-language")
                            .anchor(gpui::Anchor::TopLeft)
                            .attach(gpui::Anchor::BottomLeft),
                    ),
            )
            .into_any_element()
    }
}
