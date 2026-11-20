use gpui::{
    AnyElement, App, Context, Anchor, Entity, IntoElement, Render, SharedString, Task, Window,
};
use yp_config::fields::nams;
use yp_core::consts::{
    CLI_DOCS_URL, DISCORD_URL, GITHUB_URL, GUIDE_URL, NAMS_GITLAB_URL, NAO_LAUNCHER_URL,
};
use yp_core::Os;
use yp_i18n::{Msg, Strings};
use yp_state::prelude::*;
use yp_state::{GameError, LaunchStatus, NoticeKind, NoticeSeverity, PlayState, TabId};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, Callout, Checkbox, IconSize, LabelSize, LanguageSelector, Link,
    NierBackground, Popover, Severity, ThemeToggle, TintColor, ToggleState, Tooltip,
};

use crate::detection_strip::DetectionStrip;
use crate::directory::DirectorySelector;
use crate::folders;
use crate::metrics;
use crate::notify;
use crate::play_button::PlayButton;
use crate::text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusLine {
    Error(SharedString),
    Status(LaunchStatus),
    Unsupported(Os),
    Ready,
    SelectGame,
    Empty,
}

impl StatusLine {
    pub fn resolve(
        error: Option<SharedString>,
        status: Option<LaunchStatus>,
        can_launch: bool,
        ui_os: Os,
        directory_selected: bool,
        play_state: PlayState,
    ) -> Self {
        if let Some(error) = error {
            return StatusLine::Error(error);
        }
        if let Some(status) = status {
            return StatusLine::Status(status);
        }
        if !can_launch {
            return StatusLine::Unsupported(ui_os);
        }
        if directory_selected && play_state == PlayState::Idle {
            return StatusLine::Ready;
        }
        if !directory_selected {
            return StatusLine::SelectGame;
        }
        StatusLine::Empty
    }

    pub fn is_multi_line(&self) -> bool {
        matches!(self, StatusLine::Unsupported(_))
    }

    pub fn color(&self) -> Color {
        match self {
            StatusLine::Error(_) => Color::Error,
            StatusLine::Status(_) => Color::Secondary,
            StatusLine::Unsupported(_) => Color::Warning,
            StatusLine::Ready => Color::Success,
            StatusLine::SelectGame | StatusLine::Empty => Color::Muted,
        }
    }

    pub fn text(&self, strings: &Strings) -> SharedString {
        match self {
            StatusLine::Error(error) => error.clone(),
            StatusLine::Status(status) => {
                SharedString::new_static(text::launch_status_text(*status, strings))
            }
            StatusLine::Unsupported(os) => {
                SharedString::new_static(text::platform_unsupported_text(*os, strings))
            }
            StatusLine::Ready => SharedString::new_static(strings.status_ready()),
            StatusLine::SelectGame => SharedString::new_static(strings.status_select_game()),
            StatusLine::Empty => SharedString::default(),
        }
    }
}

pub fn launch_warnings(values: &toml::Table, strings: &Strings) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    if nams::DISABLE_PLUGIN_LOADING.value_in(values) {
        warnings.push(strings.warning_plugin_loading_disabled());
    }
    if nams::DISABLE_RESHADE_LOADING.value_in(values) {
        warnings.push(strings.warning_re_shade_disabled());
    }
    if nams::DISABLE_TEXTURE_INJECTION.value_in(values) {
        warnings.push(strings.warning_texture_injection_disabled());
    }
    warnings
}

pub struct LauncherTab {
    directory: Entity<DirectorySelector>,
    shortcut_task: Option<Task<()>>,
}

impl LauncherTab {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            directory: cx.new(|cx| DirectorySelector::new(window, cx)),
            shortcut_task: None,
        }
    }

    fn create_shortcut(&mut self, cx: &mut Context<Self>) {
        let created = cx.stores().session.read(cx).create_desktop_shortcut(cx);
        self.shortcut_task = Some(cx.spawn(async move |_, cx| {
            let created = created.await;
            cx.update(|cx| {
                let (msg, severity) = if created {
                    (Msg::NotifyShortcutCreated, NoticeSeverity::Success)
                } else {
                    (Msg::NotifyShortcutFailed, NoticeSeverity::Error)
                };
                notify::push("shortcut", msg, NoticeKind::Shortcut, severity, cx);
            });
        }));
    }

    fn link_bar(&self, window: &Window, cx: &App) -> impl IntoElement + use<> {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let link = |id: &'static str, icon: IconName, tooltip: &'static str, url: &'static str| {
            IconButton::new(id, icon)
                .icon_size(IconSize::Medium)
                .icon_color(Color::Accent)
                .size(ButtonSize::Compact)
                .tooltip(Tooltip::text(tooltip))
                .on_click(move |_, _, cx| folders::open_link(url, cx))
        };
        h_flex()
            .flex_none()
            .gap(sizes.spacing_sm)
            .px(sizes.chip_padding_h)
            .py(sizes.chip_padding_v)
            .rounded(sizes.border_radius)
            .bg(theme.colors.background_card)
            .border_1()
            .border_color(theme.colors.border_light)
            .child(link(
                "shell-link-source",
                IconName::Code,
                strings.tooltip_launcher_source(),
                GITHUB_URL,
            ))
            .child(link(
                "shell-link-nams",
                IconName::MergeType,
                strings.tooltip_nams_source(),
                NAMS_GITLAB_URL,
            ))
            .child(link(
                "shell-link-guide",
                IconName::MenuBook,
                strings.tooltip_guide(),
                GUIDE_URL,
            ))
            .child(link(
                "shell-link-discord",
                IconName::Forum,
                strings.tooltip_discord(),
                DISCORD_URL,
            ))
            .child(ThemeToggle::new().id("shell-theme-toggle"))
            .child(LanguageSelector::new().id("shell-language"))
    }

    fn status_line(&self, window: &Window, cx: &App) -> impl IntoElement + use<> {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let session = cx.stores().session.read(cx);
        let error = session
            .error()
            .map(|error: &GameError| text::game_error_text(error, strings));
        let line = StatusLine::resolve(
            error,
            session.status(),
            session.can_launch_game(),
            cx.env().host.ui_os(),
            session.is_directory_selected(),
            session.play_state(),
        );
        let multi_line = line.is_multi_line();
        div()
            .min_h(sizes.font_lg * 1.4)
            .when(multi_line, |this| this.max_w(metrics::STATUS_MAX_WIDTH))
            .child(
                Label::new(line.text(strings))
                    .size(if multi_line {
                        LabelSize::Default
                    } else {
                        LabelSize::Large
                    })
                    .color(line.color())
                    .line_height(1.4)
                    .centered(),
            )
    }

    fn launch_options(ui_os: Os, window: &Window, cx: &App) -> Option<AnyElement> {
        if ui_os == Os::MacOs {
            return None;
        }
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let gap = sizes.spacing_md;
        let popover = Popover::new("shell-launch-options")
            .anchor(Anchor::BottomLeft)
            .attach(Anchor::TopLeft)
            .trigger_with(move |open, _, _| {
                Button::new("shell-launch-options-trigger", strings.launch_options_title())
                    .style(ButtonStyle::Outlined)
                    .size(ButtonSize::Default)
                    .color(Color::Accent)
                    .start_icon(IconName::Tune)
                    .end_icon(if open {
                        IconName::KeyboardArrowDown
                    } else {
                        IconName::KeyboardArrowUp
                    })
                    .toggle_state(open)
                    .into_any_element()
            })
            .content(move |_, cx| {
                let strings = cx.t();
                let options = cx.stores().launch_options.read(cx);
                let minimize = options.minimize_on_launch();
                let dedicated_gpu = options.prefer_dedicated_gpu();
                let vanilla = options.vanilla();
                v_flex()
                    .whitespace_nowrap()
                    .gap(gap)
                    .when(ui_os == Os::Windows, |this| {
                        this.child(
                            Checkbox::new("shell-option-minimize", minimize)
                                .label(strings.launch_option_minimize_on_launch())
                                .label_size(LabelSize::XSmall)
                                .on_click(|state: &ToggleState, _, cx| {
                                    let enabled = state.selected();
                                    let options = cx.stores().launch_options.clone();
                                    options.update(cx, |options, cx| {
                                        options.set_minimize_on_launch(enabled, cx);
                                    });
                                }),
                        )
                    })
                    .child(
                        div()
                            .id("shell-option-gpu-row")
                            .tooltip(Tooltip::text(
                                strings.launch_option_prefer_dedicated_gpu_tooltip(),
                            ))
                            .child(
                                Checkbox::new("shell-option-gpu", dedicated_gpu)
                                    .label(strings.launch_option_prefer_dedicated_gpu())
                                    .label_size(LabelSize::XSmall)
                                    .on_click(|state: &ToggleState, _, cx| {
                                        let enabled = state.selected();
                                        let options = cx.stores().launch_options.clone();
                                        options.update(cx, |options, cx| {
                                            options.set_prefer_dedicated_gpu(enabled, cx);
                                        });
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .id("shell-option-vanilla-row")
                            .gap(gap)
                            .tooltip(Tooltip::text(strings.launch_option_vanilla_tooltip()))
                            .child(
                                Checkbox::new("shell-option-vanilla", vanilla)
                                    .label(strings.launch_option_vanilla())
                                    .label_size(LabelSize::XSmall)
                                    .on_click(|state: &ToggleState, _, cx| {
                                        let enabled = state.selected();
                                        let options = cx.stores().launch_options.clone();
                                        options.update(cx, |options, cx| {
                                            options.set_vanilla(enabled, cx);
                                        });
                                    }),
                            )
                            .when(vanilla, |this| {
                                this.child(
                                    Icon::new(IconName::WarningAmber)
                                        .size(IconSize::Small)
                                        .color(Color::Warning),
                                )
                            }),
                    )
                    .into_any_element()
            });
        Some(popover.into_any_element())
    }

    fn info_bar(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        if !cx.stores().session.read(cx).is_directory_selected() {
            return None;
        }
        let ui_os = cx.env().host.ui_os();
        let shortcut = cx.env().platform.supports_desktop_shortcut();
        let bar = h_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .px(sizes.card_padding_h)
            .py(sizes.card_padding_v)
            .rounded(sizes.border_radius)
            .bg(theme.colors.background_card)
            .border_1()
            .border_color(theme.colors.border_light)
            .children(Self::launch_options(ui_os, window, cx))
            .child(div().flex_1())
            .child(
                Button::new("shell-open-logs", strings.info_bar_logs())
                    .style(ButtonStyle::Tinted(TintColor::Accent))
                    .size(ButtonSize::Medium)
                    .tooltip(Tooltip::text(strings.tooltip_open_logs()))
                    .on_click(|_, window, cx| window.dispatch_action(Box::new(OpenLogs), cx)),
            )
            .when(shortcut, |this| {
                this.child(
                    Button::new("shell-create-shortcut", strings.info_bar_shortcut())
                        .style(ButtonStyle::Tinted(TintColor::Accent))
                        .size(ButtonSize::Medium)
                        .tooltip(Tooltip::text(strings.tooltip_create_shortcut()))
                        .on_click(cx.listener(|this, _, _, cx| this.create_shortcut(cx))),
                )
            });
        Some(bar.into_any_element())
    }

    fn help_text(&self, window: &Window, cx: &App) -> impl IntoElement + use<> {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let plain = |text: &'static str| {
            Label::new(text)
                .size(LabelSize::XSmall)
                .color(Color::Muted)
                .into_any_element()
        };
        let link = |id: &'static str, label: &'static str, url: &'static str| {
            Link::new(id, label)
                .on_click(move |_, _, cx| folders::open_link(url, cx))
                .size(LabelSize::XSmall)
                .color(Color::Accent)
                .hover_color(Color::Default)
                .underline()
                .into_any_element()
        };
        h_flex()
            .w_full()
            .flex_wrap()
            .justify_center()
            .px(sizes.chip_padding_h)
            .py(sizes.chip_padding_v)
            .rounded(sizes.border_radius)
            .bg(theme.colors.background_card.opacity(theme.alpha.glass))
            .child(plain(strings.help_prefix()))
            .child(link(
                "shell-help-nao",
                strings.help_nao_launcher(),
                NAO_LAUNCHER_URL,
            ))
            .child(plain(strings.help_or()))
            .child(link(
                "shell-help-cli",
                strings.help_command_line(),
                CLI_DOCS_URL,
            ))
            .child(plain(strings.help_join_discord()))
            .child(link("shell-help-discord", strings.help_discord(), DISCORD_URL))
            .child(plain(strings.help_suffix()))
    }

    fn warnings(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let stores = cx.stores().clone();
        if !stores.launch_options.read(cx).show_launch_warnings() {
            return None;
        }
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let warnings = launch_warnings(stores.config.read(cx).nams(), strings);
        if warnings.is_empty() {
            let options = stores.launch_options.clone();
            cx.defer(move |cx| {
                options.update(cx, |options, cx| {
                    options.set_show_launch_warnings(false, cx);
                });
            });
            return None;
        }
        Some(
            v_flex()
                .gap(sizes.spacing_md)
                .children(warnings.into_iter().enumerate().map(|(index, warning)| {
                    Callout::new()
                        .id(("shell-launch-warning", index))
                        .severity(Severity::Warning)
                        .icon(IconName::WarningAmber)
                        .title(warning)
                        .title_size(LabelSize::XSmall)
                }))
                .into_any_element(),
        )
    }
}

impl Render for LauncherTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let stores = cx.stores();
        let session = stores.session.read(cx);
        let idle = session.play_state() == PlayState::Idle;
        let can_launch = session.can_launch_game();
        let active = stores.navigation.read(cx).active() == TabId::Launcher;
        let animate = active && idle && window.is_window_active();
        let edge = sizes.info_bar_padding_h;

        let top = h_flex()
            .w_full()
            .items_start()
            .justify_between()
            .gap(sizes.spacing_lg)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .max_w(metrics::DIRECTORY_MAX_WIDTH)
                    .child(self.directory.clone()),
            )
            .child(self.link_bar(window, cx));

        let center = v_flex()
            .absolute()
            .inset_0()
            .items_center()
            .justify_center()
            .gap(sizes.spacing_md)
            .px(sizes.padding_xl)
            .child(PlayButton::hero())
            .child(self.status_line(window, cx));

        let strip = h_flex()
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .child(DetectionStrip::new());

        let warnings = self.warnings(window, cx).map(|warnings| {
            div()
                .absolute()
                .left_0()
                .bottom(sizes.spacing_md)
                .child(warnings)
        });

        let middle = div()
            .relative()
            .flex_1()
            .min_h_0()
            .w_full()
            .child(center)
            .child(strip)
            .children(warnings);

        let content = v_flex()
            .absolute()
            .inset_0()
            .gap(sizes.spacing_md)
            .px(edge)
            .pt(sizes.padding_md)
            .pb(sizes.spacing_md)
            .when(!can_launch, |this| {
                this.bg(theme.colors.error.opacity(theme.alpha.subtle))
            })
            .child(top)
            .child(middle)
            .children(self.info_bar(window, cx))
            .child(self.help_text(window, cx));

        NierBackground::new()
            .id("shell-launcher-background")
            .animate(animate)
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yp_i18n::Locale;

    fn strings() -> &'static Strings {
        Strings::for_locale(Locale::En)
    }

    #[test]
    fn errors_win_over_everything_else() {
        let line = StatusLine::resolve(
            Some("broken".into()),
            Some(LaunchStatus::Running),
            true,
            Os::Windows,
            true,
            PlayState::Idle,
        );
        assert_eq!(line, StatusLine::Error("broken".into()));
        assert_eq!(line.color(), Color::Error);
        assert_eq!(line.text(strings()).as_ref(), "broken");
    }

    #[test]
    fn status_shows_while_launching_and_running() {
        let line = StatusLine::resolve(
            None,
            Some(LaunchStatus::Launching),
            true,
            Os::Windows,
            true,
            PlayState::Loading,
        );
        assert_eq!(line.text(strings()).as_ref(), strings().status_launching());
        assert!(!line.is_multi_line());
    }

    #[test]
    fn unsupported_platform_explains_per_system() {
        let mac = StatusLine::resolve(None, None, false, Os::MacOs, true, PlayState::Idle);
        assert!(mac.is_multi_line());
        assert_eq!(
            mac.text(strings()).as_ref(),
            strings().platform_unsupported_macos()
        );
        let linux = StatusLine::resolve(None, None, false, Os::Linux, false, PlayState::Idle);
        assert_eq!(
            linux.text(strings()).as_ref(),
            strings().platform_unsupported_linux()
        );
    }

    #[test]
    fn ready_and_select_game_follow_the_directory() {
        let ready = StatusLine::resolve(None, None, true, Os::Windows, true, PlayState::Idle);
        assert_eq!(ready, StatusLine::Ready);
        assert_eq!(ready.color(), Color::Success);
        let pick = StatusLine::resolve(None, None, true, Os::Windows, false, PlayState::Idle);
        assert_eq!(pick, StatusLine::SelectGame);
        let busy = StatusLine::resolve(None, None, true, Os::Windows, true, PlayState::Running);
        assert_eq!(busy, StatusLine::Empty);
        assert!(busy.text(strings()).is_empty());
    }

    #[test]
    fn warnings_list_only_the_disabled_loaders() {
        let mut values = toml::Table::new();
        assert!(launch_warnings(&values, strings()).is_empty());
        values.insert(
            nams::DISABLE_RESHADE_LOADING.key().to_owned(),
            toml::Value::Boolean(true),
        );
        assert_eq!(
            launch_warnings(&values, strings()),
            vec![strings().warning_re_shade_disabled()]
        );
    }
}
