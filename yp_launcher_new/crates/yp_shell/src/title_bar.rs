use gpui::{Context, IntoElement, Render, Window, div};
use yp_state::StateExt;
use yp_ui::prelude::*;
use yp_ui::window_frame_is_ours;

use crate::ShellOptions;

pub struct ShellTitleBar;

impl ShellTitleBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ShellTitleBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let system_draws_it = cx.env().platform.uses_native_title_bar();
        if system_draws_it && !window_frame_is_ours(window) {
            return div().into_any_element();
        }
        let strings = cx.t();
        let sandbox = cx
            .try_global::<ShellOptions>()
            .and_then(|options| options.sandbox.clone());
        TitleBar::new(strings.shell_title_short())
            .icon(IconName::SportsEsports)
            .when_some(sandbox, |bar, dir| {
                bar.child(
                    Chip::new(strings.shell_sandbox_badge())
                        .id("shell-sandbox-badge")
                        .tone(ChipTone::Warning)
                        .size(ChipSize::Small)
                        .tooltip_text(strings.shell_sandbox_tooltip(&dir.to_string_lossy())),
                )
            })
            .into_any_element()
    }
}
