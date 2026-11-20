use std::collections::BTreeSet;

use gpui::{
    App, Context, IntoElement, Render, SharedString, Subscription, Task, Window, div, prelude::*,
};
use yp_builders::first_mod;
use yp_core::{GamePaths, Os};
use yp_host::nams_cli::VerifyStatus;
use yp_i18n::Strings;
use yp_state::SessionEvent;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, TintColor};

use super::widgets::tint_button;
use crate::dialogs::toml_editor::{TomlEditorOptions, edit_toml};
use crate::errors::builder_error_text;
use crate::util::{current_game, error_line, open_folder, status_line};

pub const STEP_FOLDER: usize = 0;
pub const STEP_MANIFEST: usize = 1;
pub const STEP_ITEM: usize = 2;
pub const STEP_VERIFY: usize = 3;
pub const STEP_COUNT: usize = 4;

pub fn existing_steps(game: &GamePaths) -> BTreeSet<usize> {
    let mut done = BTreeSet::new();
    if first_mod::has_folder(game) {
        done.insert(STEP_FOLDER);
    }
    if first_mod::has_manifest(game) {
        done.insert(STEP_MANIFEST);
    }
    if first_mod::has_item(game) {
        done.insert(STEP_ITEM);
    }
    done
}

pub fn verify_text(status: VerifyStatus, os: Os, strings: &Strings) -> &'static str {
    match status {
        VerifyStatus::Ok => strings.verify_install_ok(),
        VerifyStatus::Failed => strings.verify_install_failed(),
        VerifyStatus::NoRuntime => strings.verify_no_runtime(),
        VerifyStatus::SteamNotRunning => strings.verify_steam_not_running(),
        VerifyStatus::TimedOut if os == Os::Linux => strings.verify_timed_out_linux(),
        VerifyStatus::TimedOut => strings.verify_timed_out(),
        VerifyStatus::CannotParse | VerifyStatus::Error => strings.verify_install_error(),
    }
}

pub struct FirstModRunner {
    done: BTreeSet<usize>,
    busy: bool,
    error: Option<SharedString>,
    verify: Option<VerifyStatus>,
    task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl FirstModRunner {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let session = cx.stores().session.clone();
        let subscriptions = vec![cx.subscribe_in(
            &session,
            window,
            |this, _, event: &SessionEvent, _, cx| {
                if matches!(event, SessionEvent::DirectoryChanged { .. }) {
                    this.verify = None;
                    this.refresh(cx);
                }
            },
        )];
        let mut this = Self {
            done: BTreeSet::new(),
            busy: false,
            error: None,
            verify: None,
            task: None,
            _subscriptions: subscriptions,
        };
        this.refresh(cx);
        this
    }

    pub fn done(&self) -> &BTreeSet<usize> {
        &self.done
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    pub fn error(&self) -> Option<&SharedString> {
        self.error.as_ref()
    }

    pub fn verify_status(&self) -> Option<VerifyStatus> {
        self.verify
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            self.done.clear();
            cx.notify();
            return;
        };
        let job = cx.background_spawn(async move { existing_steps(&game) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let done = job.await;
            this.update(cx, |this, cx| {
                this.done = done;
                cx.notify();
            })
            .ok();
        }));
    }

    fn run(
        &mut self,
        step: usize,
        action: impl FnOnce(&GamePaths) -> yp_builders::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(game) = current_game(cx) else {
            return;
        };
        self.busy = true;
        self.error = None;
        cx.notify();
        let job = cx.background_spawn(async move { action(&game) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(()) => {
                        this.done.insert(step);
                    }
                    Err(error) => {
                        let detail = builder_error_text(&error, cx.t());
                        this.error = Some(SharedString::from(cx.t().first_mod_failed(&detail)));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn create_folder(&mut self, cx: &mut Context<Self>) {
        self.run(STEP_FOLDER, first_mod::create_folder, cx);
    }

    pub fn write_manifest(&mut self, content: String, cx: &mut Context<Self>) {
        self.run(
            STEP_MANIFEST,
            move |game| first_mod::write_manifest(game, Some(&content)),
            cx,
        );
    }

    pub fn write_item(&mut self, content: String, cx: &mut Context<Self>) {
        self.run(
            STEP_ITEM,
            move |game| first_mod::write_item(game, Some(&content)),
            cx,
        );
    }

    fn edit(&mut self, step: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            return;
        };
        let strings = cx.t();
        let (title, file_name, explanation, reset) = if step == STEP_MANIFEST {
            (
                strings.first_mod_step2(),
                SharedString::new_static(first_mod::MANIFEST_FILE),
                strings.first_mod_explain_manifest(),
                first_mod::MANIFEST,
            )
        } else {
            (
                strings.first_mod_step3(),
                SharedString::from(format!("entities/{}", first_mod::ITEM_FILE)),
                strings.first_mod_explain_item(),
                first_mod::ITEM,
            )
        };
        let reading = cx.background_spawn(async move {
            if step == STEP_MANIFEST {
                first_mod::read_manifest(&game)
            } else {
                first_mod::read_item(&game)
            }
        });
        self.task = Some(cx.spawn_in(window, async move |this, cx| {
            let initial = reading.await.unwrap_or_else(|_| reset.to_owned());
            let options = TomlEditorOptions {
                title: SharedString::new_static(title),
                file_name,
                explanation: SharedString::new_static(explanation),
                initial,
                reset: reset.to_owned(),
            };
            let Ok(editing) = cx.update(|window, cx| edit_toml(options, window, cx)) else {
                return;
            };
            let Some(edited) = editing.await else {
                return;
            };
            this.update(cx, |this, cx| {
                if step == STEP_MANIFEST {
                    this.write_manifest(edited, cx);
                } else {
                    this.write_item(edited, cx);
                }
            })
            .ok();
        }));
    }

    pub fn verify(&mut self, cx: &mut Context<Self>) {
        self.busy = true;
        self.error = None;
        self.verify = None;
        cx.notify();
        let checking = cx.stores().session.read(cx).verify_install(cx);
        self.task = Some(cx.spawn(async move |this, cx| {
            let outcome = checking.await;
            this.update(cx, |this, cx| {
                this.busy = false;
                let status = outcome.map_or(VerifyStatus::Error, |outcome| outcome.status);
                this.apply_verify(status);
                cx.notify();
            })
            .ok();
        }));
    }

    pub fn apply_verify(&mut self, status: VerifyStatus) {
        self.verify = Some(status);
        if status == VerifyStatus::Ok {
            self.done.insert(STEP_VERIFY);
        }
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        let Some(game) = current_game(cx) else {
            return;
        };
        let job = cx.background_spawn(async move { first_mod::remove_all(&game) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        this.done.clear();
                        this.verify = None;
                        this.error = None;
                    }
                    Err(error) => {
                        let detail = builder_error_text(&error, cx.t());
                        this.error = Some(SharedString::from(cx.t().first_mod_failed(&detail)));
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn open_mod_folder(&mut self, cx: &mut Context<Self>) {
        if let Some(game) = current_game(cx) {
            open_folder(&first_mod::mod_root(&game), cx);
        }
    }

    fn step(
        &self,
        index: usize,
        title: &'static str,
        detail: &'static str,
        action: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let strings = cx.t();
        let done = self.done.contains(&index);
        let badge = sizes.icon_lg;
        let success = theme.colors.success;
        let label = if done {
            strings.first_mod_redo()
        } else {
            action
        };
        h_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(
                div()
                    .flex_none()
                    .size(badge)
                    .rounded(badge)
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_1()
                    .border_color(if done {
                        success
                    } else {
                        theme.colors.border_light
                    })
                    .bg(if done {
                        success.opacity(theme.alpha.hover)
                    } else {
                        theme.colors.surface_light
                    })
                    .child(if done {
                        Icon::new(IconName::Check)
                            .size(IconSize::XSmall)
                            .color(Color::Success)
                            .into_any_element()
                    } else {
                        Label::new(SharedString::from((index + 1).to_string()))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .bold()
                            .into_any_element()
                    }),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        Label::new(title)
                            .size(LabelSize::Small)
                            .weight(if done {
                                gpui::FontWeight::MEDIUM
                            } else {
                                gpui::FontWeight::BOLD
                            }),
                    )
                    .child(
                        Label::new(detail)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .mono(),
                    ),
            )
            .child(
                tint_button(
                    SharedString::from(format!("first-mod-step-{index}")),
                    label,
                    if done {
                        TintColor::Muted
                    } else {
                        TintColor::Accent
                    },
                )
                .loading(self.busy)
                .disabled(self.busy)
                .on_click(cx.listener(move |this, _, window, cx| match index {
                    STEP_FOLDER => this.create_folder(cx),
                    STEP_MANIFEST | STEP_ITEM => this.edit(index, window, cx),
                    _ => this.verify(cx),
                })),
            )
    }
}

impl Render for FirstModRunner {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let strings = cx.t();
        if current_game(cx).is_none() {
            return v_flex().w_full().child(
                Label::new(strings.first_mod_no_game_dir())
                    .size(LabelSize::XSmall)
                    .color(Color::Warning),
            );
        }
        let os = cx.env().host.ui_os();
        let verify = self.verify.map(|status| {
            let ok = status == VerifyStatus::Ok;
            status_line(
                if ok {
                    IconName::CheckCircleOutline
                } else {
                    IconName::ErrorOutline
                },
                if ok { Color::Success } else { Color::Warning },
                verify_text(status, os, strings),
                window,
                cx,
            )
        });
        let finished = (self.done.len() == STEP_COUNT).then(|| {
            status_line(
                IconName::CelebrationOutlined,
                Color::Success,
                strings.first_mod_finished(),
                window,
                cx,
            )
        });
        let actions = (!self.done.is_empty()).then(|| {
            h_flex()
                .gap(sizes.spacing_lg)
                .pt(sizes.spacing_sm)
                .child(
                    tint_button(
                        "first-mod-open-folder",
                        strings.first_mod_open_folder(),
                        TintColor::Accent,
                    )
                    .controller_support(false)
                    .on_click(cx.listener(|this, _, _, cx| this.open_mod_folder(cx))),
                )
                .child(
                    tint_button("first-mod-reset", strings.first_mod_reset(), TintColor::Muted)
                        .on_click(cx.listener(|this, _, _, cx| this.reset(cx))),
                )
        });
        v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(self.step(
                STEP_FOLDER,
                strings.first_mod_step1(),
                strings.first_mod_step1_detail(),
                strings.first_mod_do(),
                window,
                cx,
            ))
            .child(self.step(
                STEP_MANIFEST,
                strings.first_mod_step2(),
                strings.first_mod_step2_detail(),
                strings.first_mod_edit(),
                window,
                cx,
            ))
            .child(self.step(
                STEP_ITEM,
                strings.first_mod_step3(),
                strings.first_mod_step3_detail(),
                strings.first_mod_edit(),
                window,
                cx,
            ))
            .child(self.step(
                STEP_VERIFY,
                strings.first_mod_step4(),
                strings.first_mod_step4_detail(),
                strings.first_mod_verify(),
                window,
                cx,
            ))
            .children(verify)
            .children(self.error.clone().map(error_line))
            .children(finished)
            .children(actions)
    }
}

pub(crate) fn build(window: &mut Window, cx: &mut App) -> gpui::AnyView {
    cx.new(|cx| FirstModRunner::new(window, cx)).into()
}
