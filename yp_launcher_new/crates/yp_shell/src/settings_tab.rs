use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, IntoElement, Render, ScrollHandle,
    Subscription, Task, Window,
};
use yp_core::Os;
use yp_host::game_probe::ExeVariant;
use yp_host::controller::CONTROLLER_NAVIGATION;
use yp_host::launch::steam;
use yp_host::nams_cli::verify::{VerifyCheck, VerifyOutcome, VerifyStatus};
use yp_i18n::Msg;
use yp_state::prelude::*;
use yp_state::{DetectionEvent, SessionEvent};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, Card, Chip, ChipSize, ChipTone, ConfirmOptions, ConfirmTone, IconSize,
    Input, InputEvent, InputState, LabelSize, PageHeader, ScrollableElement, Spinner, TintColor,
    SettingRow, Switch, ToggleState, TwoColumn, confirm,
};

use crate::folders;
use crate::notify;
use crate::parts;
use crate::text;

const LIST_SEPARATOR: &str = ", ";

pub struct LauncherSettingsTab {
    scroll: ScrollHandle,
    verifying: bool,
    verify_outcome: Option<VerifyOutcome>,
    missing_files: Vec<&'static str>,
    wrapper: Entity<InputState>,
    verify_task: Option<Task<()>>,
    health_task: Option<Task<()>>,
    command_task: Option<Task<()>>,
    cache_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl LauncherSettingsTab {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let strings = cx.t();
        let stores = cx.stores().clone();
        let current_wrapper = stores.launch_options.read(cx).launch_wrapper().to_owned();
        let wrapper = cx.new(|cx| {
            InputState::new(window, cx)
                .mono()
                .placeholder(strings.launch_wrapper_hint())
                .default_value(current_wrapper)
        });
        let subscriptions = vec![
            cx.subscribe(&wrapper, |_, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    let value = input.read(cx).value().to_owned();
                    let options = cx.stores().launch_options.clone();
                    options.update(cx, |options, cx| options.set_launch_wrapper(value, cx));
                }
            }),
            cx.subscribe(&stores.detection, |this, _, event: &DetectionEvent, cx| {
                if matches!(event, DetectionEvent::Updated { .. }) {
                    this.load_health(cx);
                }
            }),
            cx.subscribe(&stores.session, |this, _, event: &SessionEvent, cx| {
                if matches!(event, SessionEvent::CacheCleared(_)) {
                    this.load_health(cx);
                }
            }),
        ];
        let mut this = Self {
            scroll: ScrollHandle::new(),
            verifying: false,
            verify_outcome: None,
            missing_files: Vec::new(),
            wrapper,
            verify_task: None,
            health_task: None,
            command_task: None,
            cache_task: None,
            _subscriptions: subscriptions,
        };
        this.load_health(cx);
        this
    }

    pub fn verify_outcome(&self) -> Option<&VerifyOutcome> {
        self.verify_outcome.as_ref()
    }

    pub fn is_verifying(&self) -> bool {
        self.verifying
    }

    pub fn missing_files(&self) -> &[&'static str] {
        &self.missing_files
    }

    fn load_health(&mut self, cx: &mut Context<Self>) {
        let env = cx.env().clone();
        let probe = cx.background_spawn(async move { env.missing_runtime_files() });
        self.health_task = Some(cx.spawn(async move |this, cx| {
            let missing = probe.await;
            this.update(cx, |this, cx| {
                if this.missing_files != missing {
                    this.missing_files = missing;
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    pub fn run_verify(&mut self, cx: &mut Context<Self>) {
        if self.verifying {
            return;
        }
        self.verifying = true;
        self.verify_outcome = None;
        cx.notify();
        let verify = cx.stores().session.read(cx).verify_install(cx);
        self.verify_task = Some(cx.spawn(async move |this, cx| {
            let outcome = verify
                .await
                .unwrap_or_else(|| VerifyOutcome::status(VerifyStatus::Error));
            this.update(cx, |this, cx| {
                this.verifying = false;
                this.verify_outcome = Some(outcome);
                cx.notify();
            })
            .ok();
        }));
    }

    fn copy_launch_command(&mut self, cx: &mut Context<Self>) {
        let preview = cx.stores().session.read(cx).launch_command_preview(cx);
        self.command_task = Some(cx.spawn(async move |_, cx| {
            let command = preview.await;
            cx.update(|cx| match command {
                Some(command) => {
                    cx.write_to_clipboard(ClipboardItem::new_string(command));
                    notify::success("launch_cmd", Msg::NotificationCommandCopied, cx);
                }
                None => notify::error("launch_cmd", Msg::NotificationCommandNotReady, cx),
            });
        }));
    }

    fn clear_cache(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        let answer = confirm(
            ConfirmOptions::new(
                strings.trouble_clear_cache_title(),
                strings.trouble_clear_cache_confirm(),
            )
            .confirm_label(strings.trouble_clear_cache_button())
            .cancel_label(strings.diagnostics_close())
            .tone(ConfirmTone::Warning),
            window,
            cx,
        );
        self.cache_task = Some(cx.spawn(async move |_, cx| {
            if !answer.await {
                return;
            }
            let clearing = cx.update(|cx| {
                let session = cx.stores().session.clone();
                session.update(cx, |session, cx| session.clear_cache(cx))
            });
            let result = clearing.await;
            cx.update(|cx| {
                notify::success(
                    "cache_cleared",
                    Msg::TroubleClearCacheDone {
                        count: result.logs_deleted as i64,
                    },
                    cx,
                );
            });
        }));
    }

    fn description(text: &'static str) -> impl IntoElement {
        Label::new(text).size(LabelSize::XSmall).color(Color::Muted)
    }

    fn alert(title: &'static str, window: &Window, cx: &App) -> gpui::Div {
        let sizes = window.sizes(cx);
        v_flex().w_full().gap(sizes.spacing_lg).child(
            Label::new(title)
                .size(LabelSize::Small)
                .color(Color::Error)
                .bold(),
        )
    }

    fn action_button(
        id: &'static str,
        label: &'static str,
        tint: TintColor,
    ) -> Button {
        Button::new(id, label)
            .style(ButtonStyle::Tinted(tint))
            .size(ButtonSize::Medium)
    }

    fn check_row(check: &VerifyCheck, window: &Window, cx: &App) -> impl IntoElement + use<> {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let (icon, color) = if check.ok() {
            (IconName::CheckCircleOutline, Color::Success)
        } else {
            (IconName::ErrorOutline, Color::Error)
        };
        h_flex()
            .w_full()
            .items_start()
            .gap(sizes.spacing_md)
            .child(Icon::new(icon).size(IconSize::Small).color(color))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        Label::new(text::verify_check_label(&check.name, strings))
                            .size(LabelSize::Small)
                            .color(Color::Secondary),
                    )
                    .when_some(
                        check.detail.clone().filter(|detail| !detail.is_empty()),
                        |this, detail| {
                            this.child(
                                Label::new(detail)
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            )
                        },
                    ),
            )
    }

    fn verify_card(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let hint = cx.env().platform.verify_timeout_hint();
        let mut body = v_flex()
            .w_full()
            .gap(sizes.spacing_lg)
            .child(Self::description(strings.verify_install_desc()));
        body = if self.verifying {
            body.child(
                h_flex()
                    .gap(sizes.spacing_md)
                    .child(Spinner::new().size(IconSize::Small))
                    .child(
                        Label::new(strings.verify_install_running())
                            .size(LabelSize::Small)
                            .color(Color::Muted),
                    ),
            )
        } else {
            body.child(
                h_flex().child(
                    Self::action_button(
                        "shell-verify-run",
                        strings.verify_install_button(),
                        TintColor::Accent,
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.run_verify(cx))),
                ),
            )
        };
        if let Some(outcome) = self.verify_outcome.as_ref().filter(|_| !self.verifying) {
            let ok = outcome.status == VerifyStatus::Ok;
            body = body.child(parts::status_line(
                ok,
                text::verify_summary(outcome, hint, strings),
                window,
                cx,
            ));
            if outcome.status == VerifyStatus::Failed
                && let Some(result) = &outcome.result
            {
                let mut checks = v_flex().w_full().gap(sizes.spacing_sm);
                for check in &result.checks {
                    checks = checks.child(Self::check_row(check, window, cx));
                }
                body = body.child(checks);
            }
        }
        Card::new()
            .title(strings.verify_install_title())
            .child(body)
            .into_any_element()
    }

    fn diagnostics_card(&self, window: &Window, cx: &App) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        Card::new()
            .title(strings.diagnostics_title())
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(Self::description(strings.diagnostics_subtitle()))
                    .child(
                        h_flex().child(
                            Self::action_button(
                                "shell-diagnostics-open",
                                strings.diagnostics_button(),
                                TintColor::Accent,
                            )
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(OpenDiagnostics), cx);
                            }),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn copy_command_card(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        Card::new()
            .title(strings.copy_command_title())
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(Self::description(strings.copy_command_desc()))
                    .child(
                        h_flex().child(
                            Self::action_button(
                                "shell-copy-command",
                                strings.copy_command_button(),
                                TintColor::Accent,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.copy_launch_command(cx);
                            })),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn folders_card(&self, window: &Window, cx: &App) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let game = cx.stores().session.read(cx).game();
        let paths = cx.env().paths.clone();
        let folder = |id: &'static str, label: &'static str, dir: std::path::PathBuf| {
            Self::action_button(id, label, TintColor::Primary)
                .controller_support(false)
                .on_click(move |_, _, cx| folders::open_dir(dir.clone(), cx))
        };
        let mut buttons = h_flex().w_full().flex_wrap().gap(sizes.spacing_md);
        if let Some(game) = &game {
            buttons = buttons
                .child(folder(
                    "shell-folder-game",
                    strings.trouble_open_game(),
                    game.root().to_path_buf(),
                ))
                .child(folder(
                    "shell-folder-nams",
                    strings.trouble_open_nams_config(),
                    game.nams_dir(),
                ));
        }
        buttons = buttons
            .child(folder(
                "shell-folder-logs",
                strings.trouble_open_logs(),
                paths.logs_dir.clone(),
            ))
            .child(folder(
                "shell-folder-launcher",
                strings.trouble_open_bins(),
                paths.runtime_dir.clone(),
            ));
        Card::new()
            .title(strings.trouble_folders_title())
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(Self::description(strings.trouble_folders_desc()))
                    .child(buttons),
            )
            .into_any_element()
    }

    fn controller_card(&self, cx: &App) -> AnyElement {
        let strings = cx.t();
        let enabled = cx.prefs().get(&CONTROLLER_NAVIGATION);
        Card::new()
            .child(
                SettingRow::new("shell-controller-navigation", strings.controller_navigation_title())
                    .description(strings.controller_navigation_description())
                    .control(
                        Switch::new("shell-controller-enabled", enabled)
                            .on_click(|state: &ToggleState, _, cx| {
                                cx.prefs().set(&CONTROLLER_NAVIGATION, &state.selected());
                                cx.refresh_windows();
                            }),
                    ),
            )
            .into_any_element()
    }

    fn clear_cache_card(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        Card::new()
            .title(strings.trouble_clear_cache_title())
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(Self::description(strings.trouble_clear_cache_desc()))
                    .child(
                        h_flex().child(
                            Self::action_button(
                                "shell-clear-cache",
                                strings.trouble_clear_cache_button(),
                                TintColor::Warning,
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_cache(window, cx);
                            })),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn legacy_exe_card(&self, window: &Window, cx: &App) -> AnyElement {
        let strings = cx.t();
        Card::new()
            .accent(Color::Error)
            .child(
                Self::alert(strings.trouble_wrong_exe_title(), window, cx)
                    .child(parts::status_line(
                        false,
                        strings.trouble_wrong_exe_summary(),
                        window,
                        cx,
                    ))
                    .child(
                        Label::new(strings.detection_exe_legacy_win7_tooltip())
                            .size(LabelSize::XSmall)
                            .color(Color::Secondary),
                    ),
            )
            .into_any_element()
    }

    fn missing_files_card(&self, window: &Window, cx: &App) -> AnyElement {
        let strings = cx.t();
        let launcher_dir = cx.env().paths.runtime_dir.clone();
        Card::new()
            .accent(Color::Error)
            .child(
                Self::alert(strings.trouble_missing_files_title(), window, cx)
                    .child(parts::status_line(
                        false,
                        strings
                            .trouble_missing_files_summary(&self.missing_files.join(LIST_SEPARATOR)),
                        window,
                        cx,
                    ))
                    .child(
                        Label::new(strings.trouble_missing_files_desc())
                            .size(LabelSize::XSmall)
                            .color(Color::Secondary),
                    )
                    .child(
                        h_flex().child(
                            Self::action_button(
                                "shell-missing-open-launcher",
                                strings.trouble_open_bins(),
                                TintColor::Accent,
                            )
                            .controller_support(false)
                            .on_click(move |_, _, cx| {
                                folders::open_dir(launcher_dir.clone(), cx);
                            }),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn steam_card(&self, window: &Window, cx: &App) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let host = &cx.env().host;
        let option = steam::launch_option(host);
        let copied = option.clone();
        Card::new()
            .title(strings.steam_overlay_title())
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(Self::description(strings.steam_overlay_desc()))
                    .child(
                        Label::new(option)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .mono(),
                    )
                    .child(
                        h_flex()
                            .gap(sizes.spacing_md)
                            .child(
                                Self::action_button(
                                    "shell-copy-steam-option",
                                    strings.steam_overlay_copy(),
                                    TintColor::Accent,
                                )
                                .on_click(move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                                    notify::success(
                                        "steam_option",
                                        Msg::NotificationSteamOptionCopied,
                                        cx,
                                    );
                                }),
                            )
                            .when(steam::started_by_steam(host), |this| {
                                this.child(
                                    Chip::new(strings.steam_overlay_active())
                                        .tone(ChipTone::Success)
                                        .size(ChipSize::Small),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    fn wrapper_card(&self, window: &Window, cx: &App) -> AnyElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        Card::new()
            .title(strings.launch_wrapper_title())
            .child(
                v_flex()
                    .w_full()
                    .gap(sizes.spacing_lg)
                    .child(Self::description(strings.launch_wrapper_desc()))
                    .child(Input::new(&self.wrapper))
                    .child(
                        Label::new(strings.launch_wrapper_example())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .mono(),
                    ),
            )
            .into_any_element()
    }
}

impl Render for LauncherSettingsTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let legacy_exe = cx
            .stores()
            .detection
            .read(cx)
            .snapshot()
            .is_some_and(|snapshot| snapshot.exe_variant == ExeVariant::LegacyWindows7);
        let linux = cx.env().host.ui_os() == Os::Linux;
        let gap = sizes.padding_lg;

        let left = v_flex()
            .w_full()
            .gap(gap)
            .child(self.verify_card(window, cx))
            .child(self.copy_command_card(window, cx))
            .child(self.clear_cache_card(window, cx));
        let right = v_flex()
            .w_full()
            .gap(gap)
            .child(self.diagnostics_card(window, cx))
            .child(self.controller_card(cx))
            .child(self.folders_card(window, cx));

        let content = div()
            .id("shell-settings-scroll")
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .gap(gap)
            .p(sizes.content_padding)
            .when(legacy_exe, |this| {
                this.child(self.legacy_exe_card(window, cx))
            })
            .when(!self.missing_files.is_empty(), |this| {
                this.child(self.missing_files_card(window, cx))
            })
            .child(TwoColumn::new(left, right).id("shell-settings-columns"))
            .when(linux, |this| {
                this.child(self.steam_card(window, cx))
                    .child(self.wrapper_card(window, cx))
            })
            .overflow_y_scrollbar(&self.scroll);

        v_flex()
            .size_full()
            .when(theme.id != ThemeId::NierNew, |el| {
                el.bg(theme.colors.background_primary)
            })
            .child(PageHeader::new(strings.tab_launcher_settings()))
            .child(content)
    }
}
