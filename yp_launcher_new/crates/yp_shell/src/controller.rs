use gpui::{
    point, App, AppContext, KeyUpEvent, Keystroke, PlatformInput, ScrollDelta, ScrollEvent, Window, WindowHandle,
};
use yp_host::controller::{ControllerAction, ControllerInput, Direction, CONTROLLER_NAVIGATION, POLL_INTERVAL};
use yp_state::prelude::*;
use yp_state::SidebarSection;
use yp_ui::{ControllerMode, Root, WindowExt};

use crate::sidebar::{self, OptionalTabs};
use crate::{shell_view, ShellHandle, Stage};

pub fn start(handle: WindowHandle<Root>, cx: &mut App) {
    let input = match ControllerInput::start() {
        Ok(input) => input,
        Err(error) => {
            log::warn!("could not start controller input: {error}");
            return;
        }
    };
    cx.spawn(async move |cx| loop {
        cx.background_executor().timer(POLL_INTERVAL).await;
        match dispatch_pending(handle, input.drain(), cx) {
            Ok(enabled) => input.set_enabled(enabled),
            Err(_) => break,
        }
    })
    .detach();
}

pub(crate) fn dispatch_pending(
    handle: WindowHandle<Root>,
    actions: impl IntoIterator<Item = ControllerAction>,
    cx: &mut impl AppContext,
) -> gpui::Result<bool> {
    cx.update_window(handle.into(), |_, window, cx| {
        let enabled = window.is_window_active() && cx.prefs().get(&CONTROLLER_NAVIGATION);
        let actions: Vec<_> = actions.into_iter().collect();
        if enabled && !actions.is_empty() {
            ControllerMode::set_active(true, cx);
        }
        for action in actions {
            if enabled {
                dispatch(action, window, cx);
            }
        }
        enabled
    })
}

fn press(key: &'static str, window: &mut Window, cx: &mut App) -> bool {
    let Ok(keystroke) = Keystroke::parse(key) else {
        return false;
    };
    ControllerMode::synthetic_input(cx, |cx| {
        let handled = window.dispatch_keystroke(keystroke.clone(), cx);
        window.dispatch_event(PlatformInput::KeyUp(KeyUpEvent { keystroke }), cx);
        handled
    })
}

pub(crate) fn dispatch(action: ControllerAction, window: &mut Window, cx: &mut App) {
    if !window.is_window_active() || !cx.prefs().get(&CONTROLLER_NAVIGATION) {
        return;
    }
    match action {
        ControllerAction::Move(direction) => {
            let key = match direction {
                Direction::Up => "up",
                Direction::Down => "down",
                Direction::Left => "left",
                Direction::Right => "right",
            };
            let menu = window
                .context_stack()
                .iter()
                .any(|context| context.contains("YpMenu") || context.contains("YpTour"));
            let vertical_navigation = matches!(direction, Direction::Up | Direction::Down) && !menu;
            if vertical_navigation || !press(key, window, cx) {
                let key = match direction {
                    Direction::Up | Direction::Left => "shift-tab",
                    Direction::Down | Direction::Right => "tab",
                };
                press(key, window, cx);
            }
        }
        ControllerAction::Confirm => {
            if window.focused(cx).is_none() {
                press("tab", window, cx);
            } else {
                press("enter", window, cx);
            }
        }
        ControllerAction::Back => {
            if !press("escape", window, cx) {
                back(window, cx);
            }
        }
        ControllerAction::PreviousTab => cycle_tab(false, window, cx),
        ControllerAction::NextTab => cycle_tab(true, window, cx),
        ControllerAction::ScrollUp | ControllerAction::ScrollDown => {
            let delta = if action == ControllerAction::ScrollUp {
                3.0
            } else {
                -3.0
            };
            let viewport = window.viewport_size();
            window.dispatch_event(
                PlatformInput::Scroll(ScrollEvent {
                    position: point(viewport.width / 2.0, viewport.height / 2.0),
                    delta: ScrollDelta::Lines(point(0.0, delta)),
                    ..ScrollEvent::default()
                }),
                cx,
            );
        }
    }
}

fn back(window: &mut Window, cx: &mut App) {
    if window.has_modal(cx) || window.has_sheet(cx) {
        return;
    }
    let Some(shell) = shell_view(cx) else {
        return;
    };
    if shell.read(cx).stage() == Stage::Onboarding {
        if let Some(wizard) = shell.read(cx).onboarding().cloned() {
            wizard.update(cx, |wizard, cx| wizard.go_back(cx));
        }
        return;
    }
    let navigation = cx.stores().navigation.clone();
    let overlay = navigation.read(cx).top_overlay().map(|overlay| overlay.id());
    if let Some(overlay) = overlay {
        navigation.update(cx, |navigation, cx| navigation.close_overlay(overlay, cx));
    } else if navigation.read(cx).active() != TabId::Launcher {
        window.dispatch_action(Box::new(OpenTab::new(TabId::Launcher)), cx);
    }
}

fn cycle_tab(forward: bool, window: &mut Window, cx: &mut App) {
    if window.has_modal(cx) || window.has_sheet(cx) || window.has_active_prompt() {
        return;
    }
    if window.context_stack().iter().any(|context| context.contains("YpMenu")) {
        return;
    }
    let Some(shell) = cx.try_global::<ShellHandle>().and_then(|handle| handle.view.upgrade()) else {
        return;
    };
    if shell.read(cx).stage() != Stage::Main {
        return;
    }
    let stores = cx.stores();
    if stores.navigation.read(cx).top_overlay().is_some() {
        return;
    }
    let has_game = stores.session.read(cx).is_directory_selected();
    let optional = OptionalTabs::read(cx);
    let tabs: Vec<_> = SidebarSection::ALL
        .into_iter()
        .flat_map(|section| sidebar::visible_tabs(section, optional))
        .filter(|tab| tab.overlay().is_none() && cx.registry().tabs.contains(*tab))
        .filter(|tab| has_game || !tab.needs_game_dir())
        .collect();
    if tabs.is_empty() {
        return;
    }
    let active = stores.navigation.read(cx).active();
    let index = tabs.iter().position(|tab| *tab == active).unwrap_or(0);
    let next = if forward {
        (index + 1) % tabs.len()
    } else {
        (index + tabs.len() - 1) % tabs.len()
    };
    window.dispatch_action(Box::new(OpenTab::new(tabs[next])), cx);
}
