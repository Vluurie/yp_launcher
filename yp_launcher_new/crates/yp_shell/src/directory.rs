use std::path::PathBuf;

use futures::channel::oneshot;
use gpui::{
    App, AsyncWindowContext, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, PathPromptOptions, Render, ScrollHandle, Subscription, Task, Window,
};
use yp_core::NierInstallation;
use yp_state::SessionError;
use yp_state::prelude::*;
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, ButtonStyle, IconSize, Input, InputSize, InputState, LabelSize, ListItem,
    ListItemSpacing, Modal, ModalFooter, ModalHeader, ModalView, Sizes, Spinner,
};

use crate::ShellOptions;
use crate::text;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallChoice {
    Selected(PathBuf),
    DeepSearch,
    Manual,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChoiceAction {
    DeepSearch,
    Manual,
    Cancel,
    Accept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallDialogKind {
    FastResults,
    ConfirmSingle,
    DeepResults,
    Onboarding,
}

pub struct InstallChoiceDialog {
    kind: InstallDialogKind,
    installs: Vec<NierInstallation>,
    sender: Option<oneshot::Sender<InstallChoice>>,
    focus_handle: FocusHandle,
    scroll: ScrollHandle,
}

impl InstallChoiceDialog {
    pub fn new(
        kind: InstallDialogKind,
        installs: Vec<NierInstallation>,
        sender: oneshot::Sender<InstallChoice>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            kind,
            installs,
            sender: Some(sender),
            focus_handle: cx.focus_handle(),
            scroll: ScrollHandle::new(),
        }
    }

    fn finish(&mut self, choice: InstallChoice, cx: &mut Context<Self>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(choice);
        }
        cx.emit(DismissEvent);
    }

    fn act(&mut self, action: ChoiceAction, cx: &mut Context<Self>) {
        let choice = match action {
            ChoiceAction::DeepSearch => InstallChoice::DeepSearch,
            ChoiceAction::Manual => InstallChoice::Manual,
            ChoiceAction::Cancel => InstallChoice::Cancelled,
            ChoiceAction::Accept => match self.installs.first() {
                Some(install) => InstallChoice::Selected(install.path.clone()),
                None => InstallChoice::Cancelled,
            },
        };
        self.finish(choice, cx);
    }

    fn install_row(
        &self,
        index: usize,
        install: &NierInstallation,
        clickable: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let tone = if install.has_data {
            Color::Success
        } else {
            Color::Warning
        };
        let icon = if install.has_data {
            IconName::Folder
        } else {
            IconName::FolderOpen
        };
        let path = install.path.clone();
        let row = ListItem::new(("shell-install-row", index))
            .spacing(ListItemSpacing::Sparse)
            .rounded(true)
            .start_slot(Icon::new(icon).size(IconSize::Medium).color(tone))
            .child(
                Label::new(install.path.to_string_lossy().into_owned())
                    .size(LabelSize::Small)
                    .truncate(),
            );
        if clickable {
            row.end_slot(
                Icon::new(IconName::ArrowForward)
                    .size(IconSize::Small)
                    .color(Color::Accent),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.finish(InstallChoice::Selected(path.clone()), cx);
            }))
        } else {
            row
        }
    }

    fn group_header(
        icon: IconName,
        color: Color,
        label: &'static str,
        gap: Pixels,
    ) -> impl IntoElement {
        h_flex()
            .gap(gap)
            .child(Icon::new(icon).size(IconSize::Small).color(color))
            .child(Label::new(label).size(LabelSize::Small).color(color).bold())
    }
}

impl EventEmitter<DismissEvent> for InstallChoiceDialog {}

impl Focusable for InstallChoiceDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for InstallChoiceDialog {
    fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.sender.take();
        true
    }
}

impl Render for InstallChoiceDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let sizes = window.sizes(cx);
        let kind = self.kind;
        let (title, body) = match kind {
            InstallDialogKind::FastResults => {
                (strings.nier_found(), Some(strings.select_installation()))
            }
            InstallDialogKind::ConfirmSingle => {
                (strings.nier_found(), Some(strings.confirm_installation()))
            }
            InstallDialogKind::DeepResults => (strings.installations_found_title(), None),
            InstallDialogKind::Onboarding => (strings.onboarding_select_installation(), None),
        };
        let clickable = kind != InstallDialogKind::ConfirmSingle;
        let mut list = v_flex().w_full().gap(sizes.spacing_sm);
        if kind == InstallDialogKind::DeepResults {
            let (with_data, without_data): (Vec<_>, Vec<_>) = self
                .installs
                .iter()
                .cloned()
                .enumerate()
                .partition(|(_, install)| install.has_data);
            if !with_data.is_empty() {
                list = list.child(Self::group_header(
                    IconName::CheckCircle,
                    Color::Success,
                    strings.valid_installations(),
                    sizes.spacing_md,
                ));
                for (index, install) in &with_data {
                    list = list.child(self.install_row(*index, install, true, cx));
                }
            }
            if !without_data.is_empty() {
                list = list.child(Self::group_header(
                    IconName::WarningRounded,
                    Color::Warning,
                    strings.without_data_folder(),
                    sizes.spacing_md,
                ));
                for (index, install) in &without_data {
                    list = list.child(self.install_row(*index, install, true, cx));
                }
            }
        } else {
            let installs = self.installs.clone();
            for (index, install) in installs.iter().enumerate() {
                list = list.child(self.install_row(index, install, clickable, cx));
            }
        }
        let action_button = |id: &'static str,
                             label: &'static str,
                             action: ChoiceAction,
                             color: Color,
                             cx: &mut Context<Self>| {
            Button::new(id, label)
                .controller_support(action != ChoiceAction::Manual)
                .style(ButtonStyle::Subtle)
                .size(ButtonSize::Default)
                .color(color)
                .on_click(cx.listener(move |this, _, _, cx| this.act(action, cx)))
        };
        let footer = match kind {
            InstallDialogKind::FastResults => ModalFooter::new().end_slot(
                h_flex()
                    .gap(sizes.spacing_sm)
                    .child(action_button(
                        "shell-install-deep",
                        strings.search_all_drives(),
                        ChoiceAction::DeepSearch,
                        Color::Accent,
                        cx,
                    ))
                    .child(action_button(
                        "shell-install-manual",
                        strings.select_manually(),
                        ChoiceAction::Manual,
                        Color::Secondary,
                        cx,
                    )),
            ),
            InstallDialogKind::ConfirmSingle => ModalFooter::new().end_slot(
                h_flex()
                    .gap(sizes.spacing_sm)
                    .child(action_button(
                        "shell-install-cancel",
                        strings.cancel_button(),
                        ChoiceAction::Cancel,
                        Color::Muted,
                        cx,
                    ))
                    .child(action_button(
                        "shell-install-manual",
                        strings.no_select_manually(),
                        ChoiceAction::Manual,
                        Color::Muted,
                        cx,
                    ))
                    .child(action_button(
                        "shell-install-accept",
                        strings.yes_button(),
                        ChoiceAction::Accept,
                        Color::Accent,
                        cx,
                    )),
            ),
            InstallDialogKind::DeepResults => ModalFooter::new().end_slot(
                h_flex()
                    .gap(sizes.spacing_sm)
                    .child(action_button(
                        "shell-install-cancel",
                        strings.cancel_button(),
                        ChoiceAction::Cancel,
                        Color::Muted,
                        cx,
                    ))
                    .child(action_button(
                        "shell-install-manual",
                        strings.select_manually(),
                        ChoiceAction::Manual,
                        Color::Secondary,
                        cx,
                    )),
            ),
            InstallDialogKind::Onboarding => ModalFooter::new().end_slot(action_button(
                "shell-install-cancel",
                strings.button_cancel(),
                ChoiceAction::Cancel,
                Color::Muted,
                cx,
            )),
        };
        let mut header = ModalHeader::new()
            .headline(title)
            .headline_color(Color::Accent);
        if let Some(body) = body {
            header = header.description(body);
        }
        div().track_focus(&self.focus_handle).child(
            Modal::new("shell-install-dialog", Some(self.scroll.clone()))
                .header(header)
                .section(list)
                .when(kind == InstallDialogKind::FastResults, |modal| {
                    modal.section(
                        v_flex()
                            .gap(sizes.spacing_sm)
                            .child(Divider::horizontal())
                            .child(
                                Label::new(strings.not_your_game())
                                    .size(LabelSize::XSmall)
                                    .color(Color::Muted),
                            ),
                    )
                })
                .footer(footer),
        )
    }
}

async fn ask_install(
    kind: InstallDialogKind,
    installs: Vec<NierInstallation>,
    cx: &mut AsyncWindowContext,
) -> InstallChoice {
    let (sender, receiver) = oneshot::channel();
    let opened = cx.update(|window, cx| {
        window
            .open_modal(cx, |_, cx| {
                InstallChoiceDialog::new(kind, installs, sender, cx)
            })
            .is_some()
    });
    if !matches!(opened, Ok(true)) {
        return InstallChoice::Cancelled;
    }
    receiver.await.unwrap_or(InstallChoice::Cancelled)
}

async fn pick_exe(cx: &mut AsyncWindowContext) -> Result<Option<PathBuf>, String> {
    if cx
        .update(|_, cx| yp_ui::ControllerMode::is_active(cx))
        .map_err(|error| error.to_string())?
    {
        return Ok(None);
    }
    let picked = cx
        .update(|_, cx| {
            cx.prompt_for_paths(PathPromptOptions {
                files: true,
                directories: false,
                multiple: false,
                prompt: Some(cx.t().file_picker_title().into()),
            })
        })
        .map_err(|error| error.to_string())?;
    let paths = picked
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    Ok(paths.and_then(|paths| paths.into_iter().next()))
}

async fn apply_directory(
    dir: PathBuf,
    rejection: ManualRejection,
    cx: &mut AsyncWindowContext,
) -> Option<PathBuf> {
    let text = dir.to_string_lossy().into_owned();
    let saved = cx
        .update(|_, cx| {
            cx.prefs()
                .set_with_result(&yp_state::prefs::NIER_DIRECTORY, &text)
        })
        .ok()?;
    let result = saved.await.unwrap_or_else(|error| Err(error.to_string()));
    if let Err(error) = result {
        log::warn!(
            "could not save selected game directory {}: {error}",
            dir.display()
        );
        let message = cx
            .update(|_, cx| cx.t().err_file_write_failed(&error))
            .ok()?;
        report_selection_error(SessionError::Message(message), rejection, cx);
        return None;
    }
    let selected = dir.clone();
    cx.update(|_, cx| {
        let session = cx.stores().session.clone();
        session.update(cx, |session, cx| session.set_directory(dir, cx));
    })
    .ok()?;
    Some(selected)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualRejection {
    ShowAsError,
    ShowAsToast,
}

fn report_selection_error(
    error: SessionError,
    rejection: ManualRejection,
    cx: &mut AsyncWindowContext,
) {
    cx.update(|window, cx| match rejection {
        ManualRejection::ShowAsError => {
            let session = cx.stores().session.clone();
            session.update(cx, |session, cx| session.set_error(error, cx));
        }
        ManualRejection::ShowAsToast => {
            let message = text::session_error_text(&error, cx.t());
            let id = if matches!(error, SessionError::Rejected(_)) {
                "game_outside_prefix"
            } else {
                "game_selection_failed"
            };
            window.push_toast(yp_ui::Toast::warning(message).id(id), cx);
        }
    })
    .ok();
}

pub async fn select_manually(
    rejection: ManualRejection,
    cx: &mut AsyncWindowContext,
) -> Option<PathBuf> {
    let exe = match pick_exe(cx).await {
        Ok(Some(exe)) => exe,
        Ok(None) => return None,
        Err(error) => {
            log::warn!("game file picker failed: {error}");
            let message = cx
                .update(|_, cx| format!("{}: {error}", cx.t().error_select_failed()))
                .ok()?;
            report_selection_error(SessionError::Message(message), rejection, cx);
            return None;
        }
    };
    log::info!("game file picker selected {}", exe.display());
    let check = cx
        .update(|_, cx| {
            cx.stores()
                .session
                .read(cx)
                .check_selection(exe.clone(), cx)
        })
        .ok()?
        .await;
    if let Some(error) = text::selection_error(&check) {
        log::warn!("game selection rejected for {}: {check:?}", exe.display());
        report_selection_error(error, rejection, cx);
        return None;
    }
    let dir = exe.parent()?.to_path_buf();
    log::info!("selected game directory {}", dir.display());
    apply_directory(dir, rejection, cx).await
}

async fn deep_search(cx: &mut AsyncWindowContext) -> Option<PathBuf> {
    let found = cx
        .update(|_, cx| cx.stores().session.read(cx).discover_deep(cx))
        .ok()?
        .await;
    match found.len() {
        0 => {
            cx.update(|window, cx| {
                let message = cx.t().fail_title_install_not_found();
                window.push_toast(yp_ui::Toast::warning(message).id("game_search_empty"), cx);
            }).ok();
            None
        }
        1 => match ask_install(InstallDialogKind::ConfirmSingle, found, cx).await {
            InstallChoice::Selected(dir) => {
                apply_directory(dir, ManualRejection::ShowAsError, cx).await
            }
            InstallChoice::Manual => select_manually(ManualRejection::ShowAsError, cx).await,
            InstallChoice::DeepSearch | InstallChoice::Cancelled => None,
        },
        _ => match ask_install(InstallDialogKind::DeepResults, found, cx).await {
            InstallChoice::Selected(dir) => {
                apply_directory(dir, ManualRejection::ShowAsError, cx).await
            }
            InstallChoice::Manual => select_manually(ManualRejection::ShowAsError, cx).await,
            InstallChoice::Cancelled | InstallChoice::DeepSearch => None,
        },
    }
}

async fn auto_search_steps(cx: &mut AsyncWindowContext) -> Option<PathBuf> {
    let found = cx
        .update(|_, cx| cx.stores().session.read(cx).discover_fast(cx))
        .ok()?
        .await;
    if found.is_empty() {
        return deep_search(cx).await;
    }
    match ask_install(InstallDialogKind::FastResults, found, cx).await {
        InstallChoice::Selected(dir) => {
            apply_directory(dir, ManualRejection::ShowAsError, cx).await
        }
        InstallChoice::DeepSearch => deep_search(cx).await,
        InstallChoice::Manual => select_manually(ManualRejection::ShowAsError, cx).await,
        InstallChoice::Cancelled => None,
    }
}

fn set_searching(searching: bool, cx: &mut AsyncWindowContext) {
    cx.update(|_, cx| {
        let session = cx.stores().session.clone();
        session.update(cx, |session, cx| session.set_auto_searching(searching, cx));
    })
    .ok();
}

pub async fn auto_search(cx: &mut AsyncWindowContext) -> Option<PathBuf> {
    set_searching(true, cx);
    let selected = auto_search_steps(cx).await;
    set_searching(false, cx);
    selected
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingSearchPhase {
    Fast,
    Deep,
}

pub async fn onboarding_search(
    mut on_phase: impl FnMut(OnboardingSearchPhase, &mut AsyncWindowContext),
    cx: &mut AsyncWindowContext,
) -> Option<PathBuf> {
    on_phase(OnboardingSearchPhase::Fast, cx);
    let mut found = cx
        .update(|_, cx| cx.stores().session.read(cx).discover_fast(cx))
        .ok()?
        .await;
    if found.is_empty() {
        on_phase(OnboardingSearchPhase::Deep, cx);
        found = cx
            .update(|_, cx| cx.stores().session.read(cx).discover_deep(cx))
            .ok()?
            .await;
    }
    let dir = match found.len() {
        0 => return None,
        1 => found.remove(0).path,
        _ => match ask_install(InstallDialogKind::Onboarding, found, cx).await {
            InstallChoice::Selected(dir) => dir,
            _ => return None,
        },
    };
    apply_directory(dir, ManualRejection::ShowAsToast, cx).await
}

pub struct DirectorySelector {
    path: Entity<InputState>,
    shown: String,
    search: Option<Task<()>>,
    auto_search_started: bool,
    _session: Subscription,
}

impl DirectorySelector {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shown = cx.stores().session.read(cx).selected_directory().to_owned();
        let value = shown.clone();
        let path = cx.new(|cx| {
            InputState::new(window, cx)
                .read_only(true)
                .default_value(value)
        });
        let session = cx.stores().session.clone();
        let subscription = cx.observe_in(&session, window, |this, session, window, cx| {
            let directory = session.read(cx).selected_directory().to_owned();
            this.sync_path(directory, window, cx);
        });
        Self {
            path,
            shown,
            search: None,
            auto_search_started: false,
            _session: subscription,
        }
    }

    fn sync_path(&mut self, directory: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.shown == directory {
            return;
        }
        self.shown = directory.clone();
        self.path
            .update(cx, |path, cx| path.set_value(directory, window, cx));
        cx.notify();
    }

    fn is_searching(&self, cx: &App) -> bool {
        cx.stores().session.read(cx).auto_searching()
    }

    pub fn run_auto_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_searching(cx) {
            return;
        }
        self.search = Some(cx.spawn_in(window, async move |_, cx| {
            auto_search(cx).await;
        }));
    }

    pub fn run_manual_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_searching(cx) {
            return;
        }
        self.search = Some(cx.spawn_in(window, async move |_, cx| {
            select_manually(ManualRejection::ShowAsError, cx).await;
        }));
    }

    fn maybe_auto_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.auto_search_started {
            return;
        }
        self.auto_search_started = true;
        let allowed = cx
            .try_global::<ShellOptions>()
            .is_none_or(|options| options.auto_search);
        let session = cx.stores().session.read(cx);
        if !allowed || session.is_directory_selected() || session.is_preparing() {
            return;
        }
        cx.defer_in(window, |this, window, cx| this.run_auto_search(window, cx));
    }
}

impl Render for DirectorySelector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.maybe_auto_search(window, cx);
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let session = cx.stores().session.read(cx);
        let selected = session.is_directory_selected();
        let searching = session.auto_searching();
        if theme.id == ThemeId::NierNew {
            let path = if searching {
                strings.searching_for_nier().to_owned()
            } else if selected {
                session.selected_directory().to_owned()
            } else {
                strings.no_file_selected().to_owned()
            };
            return v_flex()
                .id("shell-directory-selector")
                .w_full()
                .min_w_0()
                .gap(px(6.))
                .child(SelectableText::new("installed-game-path", path).size(LabelSize::Small))
                .child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .flex_wrap()
                        .gap(px(12.))
                        .child(
                            Button::new(
                                "shell-directory-select",
                                if selected {
                                    strings.menu_change_installation()
                                } else {
                                    strings.select_manually()
                                },
                            )
                            .style(ButtonStyle::Link)
                            .start_icon(IconName::FolderOpen)
                            .controller_support(false)
                            .disabled(searching)
                            .on_click(cx.listener(|v, _, w, cx| v.run_manual_select(w, cx))),
                        )
                        .child(
                            Button::new("shell-directory-auto", strings.auto_button())
                                .style(ButtonStyle::Link)
                                .start_icon(IconName::Search)
                                .disabled(searching)
                                .on_click(cx.listener(|v, _, w, cx| v.run_auto_search(w, cx))),
                        ),
                )
                .into_any_element();
        }
        let hover_border = theme.colors.border_medium;
        let field = h_flex()
            .flex_1()
            .min_w_0()
            .gap(sizes.spacing_md)
            .when(!selected || searching, |this| {
                this.px(sizes.padding_sm).py(sizes.spacing_md)
            })
            .rounded(Sizes::CONTROL_RADIUS)
            .bg(theme.colors.input_background)
            .border_1()
            .border_color(if selected {
                theme.colors.success
            } else {
                theme.colors.border_medium
            })
            .map(|this| {
                if searching {
                    this.child(Spinner::new().size(IconSize::Small)).child(
                        Label::new(strings.searching_for_nier())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .italic()
                            .truncate(),
                    )
                } else if selected {
                    this.child(
                        v_flex().flex_1().min_w_0().child(
                            Input::new(&self.path)
                                .size(InputSize::Small)
                                .bordered(false),
                        ),
                    )
                } else {
                    this.child(
                        Label::new(strings.no_file_selected())
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                            .italic(),
                    )
                }
            });
        h_flex()
            .id("shell-directory-selector")
            .w_full()
            .gap(sizes.spacing_md)
            .px(sizes.padding_sm)
            .py(sizes.spacing_sm)
            .rounded(sizes.border_radius)
            .bg(theme.colors.background_card)
            .border_1()
            .border_color(theme.colors.border_light)
            .hover(move |style| style.border_color(hover_border))
            .child(field)
            .child(
                Button::new("shell-directory-auto", strings.auto_button())
                    .style(ButtonStyle::Outlined)
                    .size(ButtonSize::Compact)
                    .color(Color::Accent)
                    .disabled(searching)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.run_auto_search(window, cx);
                    })),
            )
            .child(
                Button::new("shell-directory-select", strings.select_button())
                    .controller_support(false)
                    .style(ButtonStyle::Filled)
                    .size(ButtonSize::Compact)
                    .disabled(searching)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.run_manual_select(window, cx);
                    })),
            )
            .into_any_element()
    }
}
