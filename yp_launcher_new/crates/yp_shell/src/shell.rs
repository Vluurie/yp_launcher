use std::collections::HashMap;
use std::time::Duration;

use gpui::{
    AnyElement, AnyView, AnyWindowHandle, App, Context, Entity, Global, IntoElement, Render,
    Subscription, Task, WeakEntity, Window,
};
use yp_core::Os;
use yp_core::runtime::RuntimeStatus;
use yp_i18n::Msg;
use yp_state::prelude::*;
use yp_state::{
    CloseGuard, NavigationBlock, NavigationEvent, Notice, NotificationsEvent, OverlayId,
    OverlayRequest, PlayState, Registry, SessionEvent,
};
use yp_ui::prelude::*;
use yp_ui::{
    ConfirmOptions, ConfirmTone, EmptyState, IconSize, LabelSize, NierBackground, Severity,
    Spinner, Toast, ToastEvent, ToastId, WindowExt, confirm,
};

use crate::ShellOptions;
use crate::launch_failure::LaunchFailureDialog;
use crate::log_panel::LogPanelView;
use crate::metrics;
use crate::notify;
use crate::onboarding::{OnboardingEvent, OnboardingWizard, Step};
use crate::sidebar;
use crate::text;
use crate::window as window_state;

pub const MINIMIZE_DELAY: Duration = Duration::from_secs(3);
const TEXTURES_BUSY_TOAST: &str = "textures_busy";
const OVERLAY_UNAVAILABLE_TOAST: &str = "overlay_unavailable";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Splash,
    Onboarding,
    Main,
}

#[derive(Clone)]
pub struct ShellHandle {
    pub window: AnyWindowHandle,
    pub view: WeakEntity<ShellView>,
}

impl Global for ShellHandle {}

pub fn overlay_key(overlay: OverlayId) -> &'static str {
    match overlay {
        OverlayId::Docs => "shell-overlay-docs",
        OverlayId::Inspector => "shell-overlay-inspector",
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosePrompt {
    pub options: ConfirmOptions,
}

pub fn close_prompt(guard: CloseGuard, cx: &App) -> Option<ClosePrompt> {
    let strings = cx.t();
    let options = match guard {
        CloseGuard::Clear => return None,
        CloseGuard::Busy { .. } => {
            ConfirmOptions::new(strings.busy_close_title(), strings.busy_close_body())
                .confirm_label(strings.busy_close_force())
                .tone(ConfirmTone::Warning)
        }
        CloseGuard::UnsavedChanges => ConfirmOptions::new(
            strings.unsaved_changes_title(),
            strings.unsaved_changes_message(),
        )
        .confirm_label(strings.discard())
        .tone(ConfirmTone::Danger),
    };
    Some(ClosePrompt {
        options: options.cancel_label(strings.stay()),
    })
}

pub struct ShellView {
    stage: Stage,
    tabs: HashMap<TabId, AnyView>,
    onboarding: Option<Entity<OnboardingWizard>>,
    log_panel: Option<WeakEntity<LogPanelView>>,
    force_close: bool,
    toasts_bound: bool,
    startup_task: Option<Task<()>>,
    save_task: Option<Task<()>>,
    minimize_task: Option<Task<()>>,
    discard_task: Option<Task<()>>,
    close_task: Option<Task<()>>,
    log_release: Option<Subscription>,
    onboarding_subscription: Option<Subscription>,
    toast_subscription: Option<Subscription>,
    _subscriptions: Vec<Subscription>,
}

impl ShellView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let stores = cx.stores().clone();
        cx.set_global(ShellHandle {
            window: window.window_handle(),
            view: cx.weak_entity(),
        });
        let subscriptions = vec![
            cx.subscribe_in(&stores.navigation, window, Self::on_navigation),
            cx.subscribe_in(&stores.session, window, Self::on_session),
            cx.subscribe_in(&stores.notifications, window, Self::on_notification),
            cx.observe_window_bounds(window, |this, window, cx| {
                this.schedule_window_save(window, cx);
            }),
            cx.observe_window_activation(window, |_, window, cx| {
                if window.is_window_active() {
                    let session = cx.stores().session.clone();
                    session.update(cx, |session, cx| session.sync_game_running(cx));
                }
                cx.notify();
            }),
        ];
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            weak.update(cx, |this, cx| this.should_close(window, cx))
                .unwrap_or(true)
        });
        Self {
            stage: Stage::Splash,
            tabs: HashMap::new(),
            onboarding: None,
            log_panel: None,
            force_close: false,
            toasts_bound: false,
            startup_task: None,
            save_task: None,
            minimize_task: None,
            discard_task: None,
            close_task: None,
            log_release: None,
            onboarding_subscription: None,
            toast_subscription: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn onboarding(&self) -> Option<&Entity<OnboardingWizard>> {
        self.onboarding.as_ref()
    }

    pub fn is_log_panel_open(&self) -> bool {
        self.log_panel
            .as_ref()
            .is_some_and(|panel| panel.upgrade().is_some())
    }

    pub fn cached_tabs(&self) -> Vec<TabId> {
        let mut tabs: Vec<TabId> = self.tabs.keys().copied().collect();
        tabs.sort();
        tabs
    }

    pub fn attach_startup(
        &mut self,
        startup: Task<yp_core::Result<RuntimeStatus>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.startup_task = Some(cx.spawn_in(window, async move |this, cx| {
            let status = startup.await;
            this.update_in(cx, |this, window, cx| {
                this.finish_startup(status, window, cx);
            })
            .ok();
        }));
    }

    pub fn finish_startup(
        &mut self,
        status: yp_core::Result<RuntimeStatus>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Err(err) = status {
            notify::error(
                "runtime_setup",
                Msg::ShellRuntimeSetupFailed {
                    reason: err.to_string(),
                },
                cx,
            );
        }
        if self.stage != Stage::Splash {
            return;
        }
        if cx.prefs().onboarding_complete() {
            self.show_main(cx);
        } else {
            self.show_onboarding(window, cx);
        }
        let notifications = cx.stores().notifications.clone();
        notifications.update(cx, |store, cx| store.check_platform_support(cx));
    }

    pub fn show_main(&mut self, cx: &mut Context<Self>) {
        self.stage = Stage::Main;
        self.onboarding = None;
        self.onboarding_subscription = None;
        cx.notify();
    }

    pub fn show_onboarding(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let wizard = cx.new(|cx| OnboardingWizard::new(window, cx));
        self.onboarding_subscription = Some(cx.subscribe_in(
            &wizard,
            window,
            |this, _, event: &OnboardingEvent, window, cx| match event {
                OnboardingEvent::Completed { play } => {
                    this.show_main(cx);
                    if *play {
                        window.dispatch_action(Box::new(Play), cx);
                    }
                }
            },
        ));
        self.onboarding = Some(wizard);
        self.stage = Stage::Onboarding;
        cx.notify();
    }

    pub fn show_onboarding_step(&mut self, step: Step, window: &mut Window, cx: &mut Context<Self>) {
        if self.onboarding.is_none() {
            self.show_onboarding(window, cx);
        }
        if let Some(wizard) = self.onboarding.clone() {
            wizard.update(cx, |wizard, cx| wizard.show_step(step, cx));
        }
    }

    fn schedule_window_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let persist = cx
            .try_global::<ShellOptions>()
            .is_none_or(|options| options.persist_window);
        if !persist {
            return;
        }
        self.save_task = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(window_state::SAVE_DELAY)
                .await;
            this.update_in(cx, |_, window, cx| window_state::save(window, cx))
                .ok();
        }));
    }

    fn save_window_now(&mut self, window: &Window, cx: &App) {
        let persist = cx
            .try_global::<ShellOptions>()
            .is_none_or(|options| options.persist_window);
        self.save_task = None;
        if persist {
            window_state::save(window, cx);
        }
    }

    pub fn should_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.force_close {
            return true;
        }
        let guard = cx.stores().navigation.read(cx).close_guard(cx);
        match close_prompt(guard, cx) {
            None => {
                self.save_window_now(window, cx);
                true
            }
            Some(prompt) => {
                self.ask_to_close(prompt, window, cx);
                false
            }
        }
    }

    pub fn is_asking_to_close(&self) -> bool {
        self.close_task.is_some()
    }

    fn ask_to_close(&mut self, prompt: ClosePrompt, window: &mut Window, cx: &mut Context<Self>) {
        if self.close_task.is_some() {
            return;
        }
        let answer = confirm(prompt.options, window, cx);
        self.close_task = Some(cx.spawn_in(window, async move |this, cx| {
            let close = answer.await;
            this.update_in(cx, |this, window, cx| {
                this.close_task = None;
                if close {
                    this.force_close = true;
                    this.save_window_now(window, cx);
                    window.remove_window();
                }
            })
            .ok();
        }));
    }

    fn ask_to_discard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let strings = cx.t();
        let answer = confirm(
            ConfirmOptions::new(
                strings.unsaved_changes_title(),
                strings.unsaved_changes_message(),
            )
            .confirm_label(strings.discard())
            .cancel_label(strings.stay())
            .tone(ConfirmTone::Danger),
            window,
            cx,
        );
        self.discard_task = Some(cx.spawn(async move |_, cx| {
            let discard = answer.await;
            cx.update(|cx| {
                let navigation = cx.stores().navigation.clone();
                navigation.update(cx, |navigation, cx| {
                    if discard {
                        navigation.confirm_discard(cx);
                    } else {
                        navigation.cancel_discard(cx);
                    }
                });
            });
        }));
    }

    fn open_overlay(&mut self, request: &OverlayRequest, window: &mut Window, cx: &mut Context<Self>) {
        match Registry::build_overlay(request, window, cx) {
            Some(view) => window.show_overlay(overlay_key(request.id()), view, cx),
            None => {
                let strings = cx.t();
                window.push_toast(
                    Toast::warning(strings.shell_overlay_unavailable())
                        .id(OVERLAY_UNAVAILABLE_TOAST),
                    cx,
                );
                let navigation = cx.stores().navigation.clone();
                let overlay = request.id();
                cx.defer(move |cx| {
                    navigation.update(cx, |navigation, cx| navigation.close_overlay(overlay, cx));
                });
            }
        }
    }

    fn open_log_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_log_panel_open() {
            return;
        }
        let logs = cx.stores().logs.clone();
        logs.update(cx, |logs, cx| logs.load(cx));
        let Some(panel) = window.open_sheet(cx, LogPanelView::new) else {
            return;
        };
        self.log_release = Some(cx.observe_release(&panel, |this, _, cx| {
            this.log_panel = None;
            let stores = cx.stores().clone();
            stores
                .logs
                .update(cx, |logs, cx| logs.stop_streaming(cx));
            stores
                .navigation
                .update(cx, |navigation, cx| navigation.close_logs(cx));
        }));
        self.log_panel = Some(panel.downgrade());
    }

    fn close_log_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_log_panel_open() {
            window.close_sheet(cx);
            self.log_panel = None;
        }
        let logs = cx.stores().logs.clone();
        logs.update(cx, |logs, cx| logs.stop_streaming(cx));
    }

    fn on_navigation(
        &mut self,
        _: &Entity<yp_state::Navigation>,
        event: &NavigationEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NavigationEvent::TabChanged { .. } | NavigationEvent::SelectionRequested(_) => {
                cx.notify();
            }
            NavigationEvent::Blocked(NavigationBlock::TexturesBusy) => {
                window.push_toast(
                    Toast::new(cx.t().textures_busy_message())
                        .id(TEXTURES_BUSY_TOAST)
                        .severity(Severity::Accent),
                    cx,
                );
            }
            NavigationEvent::DiscardRequested(_) => self.ask_to_discard(window, cx),
            NavigationEvent::OverlayOpened(request) => self.open_overlay(request, window, cx),
            NavigationEvent::OverlayClosed(overlay) => {
                window.hide_overlay(overlay_key(*overlay), cx);
            }
            NavigationEvent::LogsToggled(true) => self.open_log_panel(window, cx),
            NavigationEvent::LogsToggled(false) => self.close_log_panel(window, cx),
        }
    }

    fn on_session(
        &mut self,
        _: &Entity<yp_state::GameSession>,
        event: &SessionEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            SessionEvent::Started(_) => self.minimize_after_launch(window, cx),
            SessionEvent::Stopped(record) => {
                self.minimize_task = None;
                if record.is_some() && cx.env().host.os() == Os::Windows {
                    window.activate_window();
                }
            }
            SessionEvent::LaunchFailed(failure) => {
                self.minimize_task = None;
                LaunchFailureDialog::open(failure.clone(), window, cx);
            }
            SessionEvent::DirectoryChanged { .. } => cx.notify(),
            SessionEvent::PlayStateChanged { .. }
            | SessionEvent::LaunchRequested { .. }
            | SessionEvent::StopFailed
            | SessionEvent::CacheCleared(_) => {}
        }
    }

    fn minimize_after_launch(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let wanted = cx.stores().launch_options.read(cx).minimize_on_launch()
            && cx.env().host.os() == Os::Windows;
        if !wanted {
            return;
        }
        self.minimize_task = Some(cx.spawn_in(window, async move |_, cx| {
            cx.background_executor().timer(MINIMIZE_DELAY).await;
            cx.update(|window, cx| {
                if cx.stores().session.read(cx).play_state() == PlayState::Running {
                    window.minimize_window();
                }
            })
            .ok();
        }));
    }

    fn toast_for(notice: &Notice, cx: &App) -> Toast {
        Toast::new(cx.tr(&notice.msg))
            .id(notice.id.clone())
            .severity(notice.severity)
            .icon(text::notice_icon(notice.kind, notice.severity))
    }

    fn on_notification(
        &mut self,
        _: &Entity<yp_state::Notifications>,
        event: &NotificationsEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NotificationsEvent::Pushed(notice) => {
                window.push_toast(Self::toast_for(notice, cx), cx);
            }
            NotificationsEvent::Dismissed(id) => {
                window.dismiss_toast(&ToastId::from(id.clone()), cx);
            }
            NotificationsEvent::DetectionsFinished => {}
        }
    }

    fn bind_toasts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.toasts_bound {
            return;
        }
        let Some(stack) = window.toast_stack(cx) else {
            return;
        };
        self.toasts_bound = true;
        self.toast_subscription = Some(cx.subscribe(&stack, |_, _, event: &ToastEvent, cx| {
            let ToastEvent::Dismissed(id) = event;
            let notifications = cx.stores().notifications.clone();
            let id = id.0.clone();
            notifications.update(cx, |store, cx| store.dismiss(id.as_ref(), cx));
        }));
        let pending: Vec<Notice> = cx.stores().notifications.read(cx).items().to_vec();
        cx.defer_in(window, move |_, window, cx| {
            for notice in &pending {
                window.push_toast(Self::toast_for(notice, cx), cx);
            }
        });
    }

    fn splash(&self, window: &Window, cx: &App) -> AnyElement {
        let sizes = window.sizes(cx);
        NierBackground::new()
            .id("shell-splash-background")
            .child(
                v_flex()
                    .absolute()
                    .inset_0()
                    .items_center()
                    .justify_center()
                    .gap(sizes.spacing_lg)
                    .child(
                        Spinner::new().size(IconSize::Custom(metrics::SPLASH_SPINNER_SIZE)),
                    )
                    .child(
                        Label::new(cx.t().app_title())
                            .size(LabelSize::Large)
                            .color(Color::Secondary)
                            .bold(),
                    ),
            )
            .into_any_element()
    }

    fn tab_view(&mut self, tab: TabId, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if !self.tabs.contains_key(&tab)
            && let Some(view) = Registry::build_tab(tab, window, cx)
        {
            self.tabs.insert(tab, view);
        }
        match self.tabs.get(&tab) {
            Some(view) => view.clone().into_any_element(),
            None => {
                let strings = cx.t();
                EmptyState::new(strings.shell_tab_unavailable_title())
                    .icon(sidebar::tab_icon(tab))
                    .description(strings.shell_tab_unavailable_body())
                    .into_any_element()
            }
        }
    }

    fn main(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let stores = cx.stores();
        let has_directory = stores.session.read(cx).is_directory_selected();
        let active = stores.navigation.read(cx).active();
        let content = self.tab_view(active, window, cx);
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(theme.colors.background_primary)
            .when(has_directory, |this| {
                this.child(sidebar::render(active, window, cx))
            })
            .child(
                div()
                    .id(("shell-tab-host", active.index()))
                    .relative()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_hidden()
                    .child(v_flex().absolute().inset_0().child(content)),
            )
            .into_any_element()
    }
}

impl Render for ShellView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.bind_toasts(window, cx);
        let body = match self.stage {
            Stage::Splash => self.splash(window, cx),
            Stage::Onboarding => match self.onboarding.clone() {
                Some(wizard) => wizard.into_any_element(),
                None => self.splash(window, cx),
            },
            Stage::Main => self.main(window, cx),
        };
        v_flex().size_full().child(body)
    }
}
