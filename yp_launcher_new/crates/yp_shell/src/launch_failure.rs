use gpui::{
    App, Context, DismissEvent, EventEmitter, FocusHandle, Focusable, IntoElement, Render,
    ScrollHandle, Task, Window,
};
use yp_host::launch::failure::LaunchFailure;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, CopyButton, DescriptionList, IconSize, KeyValueRow, LabelSize, Modal,
    ModalFooter, ModalHeader, ModalView, ModalWidth, SelectableText, TintColor, WindowExt,
};

use crate::folders;
use crate::metrics;
use crate::parts;
use crate::text;

pub struct LaunchFailureDialog {
    failure: LaunchFailure,
    command: Option<String>,
    focus_handle: FocusHandle,
    scroll: ScrollHandle,
    _command_task: Option<Task<()>>,
}

impl LaunchFailureDialog {
    pub fn new(failure: LaunchFailure, cx: &mut Context<Self>) -> Self {
        let command_task = failure.reported_by_nams().then(|| {
            let preview = cx.stores().session.read(cx).launch_command_preview(cx);
            cx.spawn(async move |this, cx| {
                let command = preview.await;
                this.update(cx, |this, cx| {
                    this.command = command;
                    cx.notify();
                })
                .ok();
            })
        });
        Self {
            failure,
            command: None,
            focus_handle: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            _command_task: command_task,
        }
    }

    pub fn failure(&self) -> &LaunchFailure {
        &self.failure
    }

    pub fn open(failure: LaunchFailure, window: &mut Window, cx: &mut App) {
        window.open_modal(cx, |_, cx| Self::new(failure, cx));
    }
}

impl EventEmitter<DismissEvent> for LaunchFailureDialog {}

impl Focusable for LaunchFailureDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for LaunchFailureDialog {}

impl Render for LaunchFailureDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let failure = &self.failure;
        let accent = if failure.is_panic() {
            Color::Error
        } else {
            Color::Warning
        };
        let title = text::failure_title(failure, strings);
        let detail = text::failure_detail(failure, strings);
        let hints = text::failure_hints(failure, strings);
        let report = failure.report().cloned();

        let mut header = ModalHeader::new()
            .icon(
                Icon::new(if failure.is_panic() {
                    IconName::BugReportOutlined
                } else {
                    IconName::ErrorOutline
                })
                .size(IconSize::Large)
                .color(accent),
            )
            .headline(title.to_uppercase());
        if let Some(subtitle) = text::failure_subtitle(failure) {
            header = header.description(subtitle);
        }

        let mut modal = Modal::new("shell-launch-failure", Some(self.scroll.clone()))
            .width(ModalWidth::Custom(metrics::FAILURE_DIALOG_WIDTH))
            .header(header)
            .section(
                parts::section_card(
                    IconName::InfoOutline,
                    strings.fail_section_what_happened(),
                    accent,
                    None,
                    window,
                    cx,
                )
                .child(
                    Label::new(text::failure_explanation(failure, strings))
                        .size(LabelSize::Small)
                        .color(Color::Secondary),
                ),
            );

        if let Some(report) = report.as_ref()
            && !report.headline.is_empty()
            && report.headline.as_str() != title.as_ref()
        {
            modal = modal.section(
                parts::section_card(
                    IconName::ReportOutlined,
                    strings.fail_section_reported_by_nams(),
                    accent,
                    None,
                    window,
                    cx,
                )
                .child(
                    SelectableText::new("shell-failure-headline", report.headline.clone())
                        .mono()
                        .size(LabelSize::XSmall)
                        .color(Color::Default),
                ),
            );
        }

        if !hints.is_empty() {
            let mut list = v_flex().w_full().gap(sizes.spacing_md);
            for hint in hints {
                list = list.child(
                    h_flex()
                        .w_full()
                        .items_start()
                        .gap(sizes.spacing_md)
                        .child(
                            Icon::new(IconName::ChevronRight)
                                .size(IconSize::Small)
                                .color(Color::Accent),
                        )
                        .child(
                            div().flex_1().min_w_0().child(
                                Label::new(hint)
                                    .size(LabelSize::Small)
                                    .color(Color::Secondary),
                            ),
                        ),
                );
            }
            modal = modal.section(
                parts::section_card(
                    IconName::LightbulbOutline,
                    strings.fail_section_try_this(),
                    Color::Accent,
                    None,
                    window,
                    cx,
                )
                .child(list),
            );
        }

        if let Some(report) = report.as_ref()
            && (report.cause.is_some() || report.os_error.is_some())
        {
            let mut list = DescriptionList::new().label_width(metrics::DETAIL_LABEL_WIDTH);
            let mut row = |label: &'static str, value: &Option<String>| {
                if let Some(value) = value {
                    let item = KeyValueRow::new(label, value.clone())
                        .mono()
                        .value_color(Color::Secondary);
                    list = std::mem::take(&mut list).item(item);
                }
            };
            row(strings.fail_detail_os(), &report.os_error);
            row(strings.fail_detail_cause(), &report.cause);
            row(strings.fail_detail_suggested(), &report.fix);
            modal = modal.section(
                parts::section_card(
                    IconName::Terminal,
                    strings.fail_section_diagnostic_detail(),
                    Color::Muted,
                    None,
                    window,
                    cx,
                )
                .child(list.label_width(metrics::DETAIL_LABEL_WIDTH)),
            );
        }

        if let Some(command) = self.command.clone() {
            modal = modal.section(
                parts::section_card(
                    IconName::Code,
                    strings.fail_section_launch_manually(),
                    Color::Accent,
                    None,
                    window,
                    cx,
                )
                .child(
                    v_flex()
                        .w_full()
                        .gap(sizes.spacing_md)
                        .child(
                            Label::new(strings.fail_manual_command_hint())
                                .size(LabelSize::XSmall)
                                .color(Color::Muted)
                                .italic(),
                        )
                        .child(parts::code_block(
                            "shell-failure-command",
                            command.clone(),
                            window,
                            cx,
                        ))
                        .child(
                            h_flex().child(
                                CopyButton::new("shell-failure-copy-command", command)
                                    .label(strings.copy_command_button()),
                            ),
                        ),
                ),
            );
        }

        modal = modal.section(
            parts::section_card(
                IconName::ArticleOutlined,
                strings.fail_section_raw_output(),
                Color::Muted,
                None,
                window,
                cx,
            )
            .child(parts::code_block(
                "shell-failure-raw",
                detail.clone(),
                window,
                cx,
            )),
        );

        let log_dir = failure
            .captured_log
            .as_ref()
            .and_then(|log| log.parent().map(|dir| dir.to_path_buf()));
        let footer = ModalFooter::new()
            .start_slot(
                h_flex()
                    .gap(sizes.spacing_md)
                    .child(
                        CopyButton::new("shell-failure-copy-report", detail)
                            .label(strings.fail_action_copy_report())
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .size(ButtonSize::Default),
                    )
                    .when_some(log_dir, |this, dir| {
                        this.child(
                            Button::new(
                                "shell-failure-open-log",
                                strings.fail_action_open_log_file(),
                            )
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .size(ButtonSize::Default)
                            .start_icon(IconName::FolderOpen)
                            .controller_support(false)
                            .on_click(move |_, _, cx| folders::open_dir(dir.clone(), cx)),
                        )
                    }),
            )
            .end_slot(
                Button::new("shell-failure-close", strings.diagnostics_close())
                    .style(ButtonStyle::Subtle)
                    .color(Color::Muted)
                    .on_click(|_, window, cx| {
                        window.close_modal(cx);
                    }),
            );

        div()
            .track_focus(&self.focus_handle)
            .child(modal.footer(footer))
    }
}
