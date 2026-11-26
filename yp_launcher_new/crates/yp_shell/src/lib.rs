mod controller;
mod detection_strip;
mod diagnostics;
mod directory;
mod folders;
mod launch_failure;
mod launcher_tab;
mod log_panel;
mod menu_navigation;
mod metrics;
mod notify;
pub mod onboarding;
mod parts;
mod play_button;
pub mod sandbox;
mod settings_tab;
mod shell;
mod sidebar;
mod stop_game;
#[cfg(test)]
mod tests;
pub mod text;
mod title_bar;
pub mod window;

use std::path::PathBuf;

use gpui::{App, AppContext as _, Global, Task, WindowHandle};
use yp_core::runtime::RuntimeStatus;
use yp_state::prelude::*;
use yp_ui::Root;

pub use detection_strip::{DetectionChip, DetectionKind, DetectionState, DetectionStrip};
pub use diagnostics::{DiagnosticsDialog, DiagnosticsLoading};
pub use directory::{DirectorySelector, InstallChoice, InstallChoiceDialog, InstallDialogKind};
pub use launch_failure::LaunchFailureDialog;
pub use launcher_tab::{LauncherTab, StatusLine};
pub use log_panel::LogPanelView;
pub use play_button::{PlayButton, PlayButtonModel};
pub use settings_tab::LauncherSettingsTab;
pub use shell::{ShellHandle, ShellView, Stage};
pub use title_bar::ShellTitleBar;

#[derive(Debug, Clone, PartialEq)]
pub struct ShellOptions {
    pub sandbox: Option<PathBuf>,
    pub auto_search: bool,
    pub persist_window: bool,
    pub focus_window: bool,
    pub window_size: Option<(f32, f32)>,
    pub docs: bool,
    pub tools: bool,
}

impl Default for ShellOptions {
    fn default() -> Self {
        Self {
            sandbox: None,
            auto_search: true,
            persist_window: true,
            focus_window: true,
            window_size: None,
            docs: true,
            tools: true,
        }
    }
}

impl ShellOptions {
    pub fn sandboxed(dir: impl Into<PathBuf>) -> Self {
        Self {
            sandbox: Some(dir.into()),
            auto_search: false,
            ..Self::default()
        }
    }

    pub fn is_sandboxed(cx: &App) -> bool {
        cx.try_global::<Self>()
            .is_some_and(|options| options.sandbox.is_some())
    }
}

impl Global for ShellOptions {}

pub fn init(cx: &mut App) {
    init_with(ShellOptions::default(), cx);
}

pub fn init_with(options: ShellOptions, cx: &mut App) {
    cx.set_global(options);
    yp_state::register_default_handlers(cx);
    stop_game::register(cx);
    cx.on_action(|_: &OpenDiagnostics, cx| {
        cx.defer(|cx| {
            let Some(handle) = cx.try_global::<ShellHandle>().cloned() else {
                return;
            };
            handle
                .window
                .update(cx, |_, window, cx| diagnostics::open(window, cx))
                .ok();
        });
    });
    let registry = cx.registry_mut();
    registry.tabs.register(TabId::Launcher, |window, cx| {
        cx.new(|cx| LauncherTab::new(window, cx)).into()
    });
    registry
        .tabs
        .register(TabId::LauncherSettings, |window, cx| {
            cx.new(|cx| LauncherSettingsTab::new(window, cx)).into()
        });
}

pub fn open_window(cx: &mut App) -> gpui::Result<WindowHandle<Root>> {
    let options = cx.try_global::<ShellOptions>().cloned().unwrap_or_default();
    let window_options = window::window_options(&options, cx);
    let handle = cx.open_window(window_options, |window, cx| {
        let shell = cx.new(|cx| ShellView::new(window, cx));
        cx.new(|cx| {
            let mut root = Root::new(shell, window, cx);
            let title_bar = cx.new(ShellTitleBar::new);
            root.set_title_bar(Some(title_bar.into()), cx);
            root
        })
    })?;
    controller::start(handle, cx);
    Ok(handle)
}

pub fn shell_view(cx: &App) -> Option<gpui::Entity<ShellView>> {
    cx.try_global::<ShellHandle>()?.view.upgrade()
}

pub fn attach_startup(startup: Task<yp_core::Result<RuntimeStatus>>, cx: &mut App) {
    let Some(handle) = cx.try_global::<ShellHandle>().cloned() else {
        startup.detach();
        return;
    };
    let view = handle.view.clone();
    handle
        .window
        .update(cx, |_, window, cx| {
            view.update(cx, |shell, cx| shell.attach_startup(startup, window, cx))
                .ok();
        })
        .ok();
}
