use std::path::Path;

use gpui::{IntoElement, Pixels, SharedString, prelude::*};
use yp_ui::prelude::*;

#[cfg(feature = "tools")]
pub(crate) use tool_helpers::*;

pub(crate) fn path_text(path: &Path) -> SharedString {
    SharedString::from(path.display().to_string())
}

pub(crate) fn clamped(max_width: Pixels, content: impl IntoElement) -> gpui::Div {
    v_flex().w_full().child(div().max_w(max_width).child(content))
}

#[cfg(feature = "tools")]
mod tool_helpers {
    use std::path::{Path, PathBuf};

    use gpui::{
        App, DispatchPhase, IntoElement, MouseMoveEvent, MouseUpEvent, PathPromptOptions,
        SharedString, Task, Window, canvas, prelude::*,
    };
    use yp_core::GamePaths;
    use yp_state::prelude::*;
    use yp_ui::prelude::*;

    pub(crate) fn pointer_capture(
        active: bool,
        on_move: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
        on_up: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
    ) -> impl IntoElement {
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                if !active {
                    return;
                }
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture {
                        on_move(event, window, cx);
                    }
                });
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture {
                        on_up(event, window, cx);
                    }
                });
            },
        )
        .absolute()
        .inset_0()
    }

    pub(crate) fn pick_file(cx: &App) -> Task<Option<PathBuf>> {
        if yp_ui::ControllerMode::is_active(cx) {
            return Task::ready(None);
        }
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        cx.background_spawn(async move {
            match picked.await {
                Ok(Ok(Some(paths))) => paths.into_iter().next(),
                _ => None,
            }
        })
    }

    pub(crate) fn current_game(cx: &App) -> Option<GamePaths> {
        cx.stores().session.read(cx).game()
    }

    pub(crate) fn reveal_file(path: &Path, cx: &App) {
        if yp_ui::ControllerMode::is_active(cx) {
            return;
        }
        cx.reveal_path(path);
    }

    pub(crate) fn open_folder(path: &Path, cx: &App) {
        if yp_ui::ControllerMode::is_active(cx) {
            return;
        }
        cx.open_with_system(path);
    }

    pub(crate) fn status_line(
        icon: IconName,
        color: Color,
        text: impl Into<SharedString>,
        window: &Window,
        cx: &App,
    ) -> gpui::Div {
        let sizes = window.sizes(cx);
        h_flex()
            .w_full()
            .items_start()
            .gap(sizes.spacing_md)
            .child(
                div()
                    .pt(sizes.spacing_xs)
                    .child(Icon::new(icon).size(IconSize::Small).color(color)),
            )
            .child(
                v_flex().flex_1().min_w_0().child(
                    Label::new(text)
                        .size(LabelSize::XSmall)
                        .color(color)
                        .line_height(1.4),
                ),
            )
    }

    pub(crate) fn error_line(text: impl Into<SharedString>) -> Label {
        Label::new(text)
            .size(LabelSize::XSmall)
            .color(Color::Error)
            .line_height(1.4)
    }
}
