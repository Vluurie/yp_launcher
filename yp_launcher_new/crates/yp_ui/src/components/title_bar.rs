use std::rc::Rc;

use gpui::{
    AnyElement, App, Decorations, FontWeight, IntoElement, MouseButton, ParentElement, RenderOnce,
    SharedString, Window, WindowControlArea, div, prelude::*, px, relative, svg,
};

use super::icon::{Icon, IconName, IconSize};
use super::label::LINE_HEIGHT_TIGHT;
use super::text::Text;
use super::tooltip::Tooltip;
use crate::i18n::ActiveStrings;
use crate::styled_ext::{h_flex, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};
use crate::traits::ActionHandler;

pub fn window_frame_is_ours(window: &Window) -> bool {
    matches!(window.window_decorations(), Decorations::Client { .. })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowControl {
    Minimize,
    Maximize,
    Close,
}

pub struct WindowControls {
    on_close: Option<ActionHandler>,
}

crate::impl_into_element!(WindowControls);

impl Default for WindowControls {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowControls {
    pub fn new() -> Self {
        Self { on_close: None }
    }

    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

fn caption_button(
    control: WindowControl,
    on_close: Option<ActionHandler>,
    window: &Window,
    cx: &App,
) -> impl IntoElement {
    let theme = cx.theme().clone();
    let strings = cx.t();
    let maximized = window.is_maximized();
    let (id, icon, icon_size, tooltip, area) = match control {
        WindowControl::Minimize => (
            "yp-caption-minimize",
            IconName::CaptionMinimize,
            Sizes::CAPTION_GLYPH_SIZE,
            strings.tooltip_minimize(),
            WindowControlArea::Min,
        ),
        WindowControl::Maximize => (
            "yp-caption-maximize",
            if maximized {
                IconName::CaptionRestore
            } else {
                IconName::CaptionMaximize
            },
            Sizes::CAPTION_GLYPH_SIZE,
            if maximized {
                strings.tooltip_restore()
            } else {
                strings.tooltip_maximize()
            },
            WindowControlArea::Max,
        ),
        WindowControl::Close => (
            "yp-caption-close",
            IconName::CaptionClose,
            Sizes::CAPTION_GLYPH_SIZE,
            strings.tooltip_close(),
            WindowControlArea::Close,
        ),
    };
    let is_close = control == WindowControl::Close;
    let hover_background = if is_close {
        theme.colors.error
    } else {
        theme.colors.border_light
    };
    let hover_icon = if is_close {
        theme.colors.text_on_dark
    } else {
        theme.colors.text_primary
    };
    let group = SharedString::new_static(id);
    let custom_close = on_close.filter(|_| is_close);
    let os_handles_click = cfg!(target_os = "windows") && custom_close.is_none();
    div()
        .id(id)
        .group(group.clone())
        .occlude()
        .flex_none()
        .w(Sizes::CAPTION_BUTTON_WIDTH)
        .h(Sizes::TITLE_BAR_HEIGHT)
        .flex()
        .items_center()
        .justify_center()
        .hover(move |style| style.bg(hover_background))
        .child(
            svg()
                .path(icon.path())
                .size(icon_size)
                .flex_none()
                .text_color(theme.colors.text_primary)
                .group_hover(group, move |style| style.text_color(hover_icon)),
        )
        .tooltip(Tooltip::text(tooltip))
        .when(os_handles_click, |this| this.window_control_area(area))
        .when(!os_handles_click, |this| {
            this.on_click(move |_, window, cx| {
                cx.stop_propagation();
                match control {
                    WindowControl::Minimize => window.minimize_window(),
                    WindowControl::Maximize => window.zoom_window(),
                    WindowControl::Close => match &custom_close {
                        Some(handler) => handler(window, cx),
                        None => window.remove_window(),
                    },
                }
            })
        })
}

impl RenderOnce for WindowControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .flex_none()
            .child(caption_button(WindowControl::Minimize, None, window, cx))
            .child(caption_button(WindowControl::Maximize, None, window, cx))
            .child(caption_button(
                WindowControl::Close,
                self.on_close,
                window,
                cx,
            ))
    }
}

pub struct TitleBar {
    icon: Option<IconName>,
    title: SharedString,
    show_controls: bool,
    on_close: Option<ActionHandler>,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(TitleBar);

impl TitleBar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            title: title.into(),
            show_controls: true,
            on_close: None,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for TitleBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let controls = self.show_controls.then(|| WindowControls {
            on_close: self.on_close,
        });
        let moves_itself = window_frame_is_ours(window);
        let pressed = window.use_keyed_state("yp-title-bar-pressed", cx, |_, _| false);
        let (press, release, drag) = (pressed.clone(), pressed.clone(), pressed);
        h_flex()
            .id("yp-title-bar")
            .w_full()
            .h(Sizes::TITLE_BAR_HEIGHT)
            .flex_none()
            .bg(theme.colors.surface_medium)
            .border_b_1()
            .border_color(theme.colors.border_light)
            .window_control_area(WindowControlArea::Drag)
            .when(!cfg!(target_os = "windows"), |this| {
                this.on_click(|event, window, _| {
                    if event.click_count() == 2 {
                        window.zoom_window();
                    }
                })
            })
            .when(moves_itself, |this| {
                this.on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    press.update(cx, |pressed, _| *pressed = true);
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    release.update(cx, |pressed, _| *pressed = false);
                })
                .on_mouse_move(move |_, window, cx| {
                    if *drag.read(cx) {
                        drag.update(cx, |pressed, _| *pressed = false);
                        window.start_window_move();
                    }
                })
                .on_mouse_down(MouseButton::Right, |event, window, _| {
                    window.show_window_menu(event.position);
                })
            })
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .pl(px(12.))
                    .gap(px(8.))
                    .when_some(self.icon, |this, icon| {
                        this.child(Icon::new(icon).size(IconSize::Medium).color(Color::Accent))
                    })
                    .child(
                        v_flex()
                            .min_w_0()
                            .truncate()
                            .text_size(sizes.font_lg)
                            .line_height(relative(LINE_HEIGHT_TIGHT))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.colors.text_primary)
                            .child(Text::new(self.title)),
                    )
                    .children(
                        self.children
                            .into_iter()
                            .map(|child| div().occlude().flex_none().child(child)),
                    ),
            )
            .children(controls)
    }
}
