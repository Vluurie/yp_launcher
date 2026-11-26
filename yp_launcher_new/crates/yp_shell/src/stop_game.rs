use gpui::{DismissEvent, EventEmitter, Subscription};
use yp_state::{PlayState, SessionEvent, StateExt, Stop};
use yp_ui::prelude::*;
use yp_ui::{ModalView, WindowExt};

use crate::ShellHandle;

pub(super) fn register(cx: &mut App) {
    cx.on_action(|_: &Stop, cx| {
        let Some(shell) = cx.try_global::<ShellHandle>().cloned() else {
            return;
        };
        cx.defer(move |cx| {
            shell
                .window
                .update(cx, |_, window, cx| {
                    if !window.has_modal(cx)
                        && cx.stores().session.read(cx).play_state() == PlayState::Running
                    {
                        window.open_modal(cx, |_, cx| StopGameDialog::new(cx));
                    }
                })
                .ok();
        });
    });
}

struct StopGameDialog {
    focus: FocusHandle,
    active: bool,
    _session: Subscription,
}

impl StopGameDialog {
    fn new(cx: &mut Context<Self>) -> Self {
        let session = cx.stores().session.clone();
        let subscription = cx.subscribe(&session, |this: &mut Self, _, event, cx| {
            if matches!(event, SessionEvent::DirectoryChanged { .. })
                || matches!(event, SessionEvent::PlayStateChanged { state, .. } if *state != PlayState::Running)
            {
                this.answer(false, cx);
            }
        });
        Self {
            focus: cx.focus_handle(),
            active: true,
            _session: subscription,
        }
    }

    fn answer(&mut self, confirmed: bool, cx: &mut Context<Self>) {
        if !self.active {
            return;
        }
        self.active = false;
        cx.emit(DismissEvent);
        if confirmed {
            let session = cx.stores().session.clone();
            session.update(cx, |session, cx| session.stop(cx));
        }
    }
}

impl EventEmitter<DismissEvent> for StopGameDialog {}

impl Focusable for StopGameDialog {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl ModalView for StopGameDialog {
    fn on_before_dismiss(&mut self, _: &mut Window, _: &mut Context<Self>) -> bool {
        self.active = false;
        true
    }
}

impl Render for StopGameDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let view = cx.weak_entity();
        let viewport = window.viewport_size();
        let scale = yp_ui::theme::MenuTextSize::factor(cx)
            .min((f32::from(viewport.width) - 24.).max(1.) / 674.)
            .min((f32::from(viewport.height) - 64.).max(1.) / 320.);
        yp_ui::nier::ConfirmDialog::new(
            strings.stop_game_confirmation(),
            move |confirmed, _, cx| {
                view.update(cx, |dialog, cx| dialog.answer(confirmed, cx))
                    .ok();
            },
        )
        .id("stop-game-confirmation")
        .title(strings.stop_button())
        .labels(strings.button_yes(), strings.button_no())
        .in_modal(self.focus.clone())
        .scale(scale)
    }
}
