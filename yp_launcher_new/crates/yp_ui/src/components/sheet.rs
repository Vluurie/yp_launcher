use std::rc::Rc;

use gpui::{
    Animation, AnimationExt, AnyElement, App, CursorStyle, ElementId, IntoElement, MouseButton,
    ParentElement, Pixels, RenderOnce, Window, div, prelude::*, px,
};

use super::root::WindowExt;
use super::util::pointer_capture;
use crate::styled_ext::{StyledExt, v_flex};
use crate::theme::{ActiveTheme, ResponsiveExt, Sizes, easing};
use crate::traits::{ActionHandler, ValueHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetSide {
    Left,
    #[default]
    Right,
}

#[derive(Default)]
struct SheetState {
    width: Option<Pixels>,
    dragging: Option<(Pixels, Pixels)>,
}

pub fn clamp_sheet_width(width: Pixels, available: Pixels) -> Pixels {
    let max = (available - Sizes::TOAST_WINDOW_MARGIN).max(Sizes::SHEET_MIN_WIDTH.min(available));
    width.max(Sizes::SHEET_MIN_WIDTH.min(max)).min(max)
}

pub struct Sheet {
    id: ElementId,
    side: SheetSide,
    width: Pixels,
    resizable: bool,
    scrim: bool,
    animate: bool,
    on_dismiss: Option<ActionHandler>,
    on_resize: Option<ValueHandler<Pixels>>,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(Sheet);

impl Sheet {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            side: SheetSide::default(),
            width: Sizes::SHEET_DEFAULT_WIDTH,
            resizable: false,
            scrim: true,
            animate: true,
            on_dismiss: None,
            on_resize: None,
            children: Vec::new(),
        }
    }

    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn scrim(mut self, scrim: bool) -> Self {
        self.scrim = scrim;
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    pub fn on_resize(mut self, handler: impl Fn(&Pixels, &mut Window, &mut App) + 'static) -> Self {
        self.on_resize = Some(Rc::new(handler));
        self
    }
}

impl ParentElement for Sheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| SheetState::default());
        let (stored_width, dragging) = {
            let state = state.read(cx);
            (state.width, state.dragging)
        };
        let available = sizes.window_width;
        let width = clamp_sheet_width(stored_width.unwrap_or(self.width), available);
        let side = self.side;
        let accent = theme.colors.accent_primary;
        let handle_hover = accent.opacity(theme.alpha.line);
        let closing = window
            .yp_root()
            .is_some_and(|root| root.read(cx).is_sheet_closing());
        let on_dismiss = self.on_dismiss.filter(|_| !closing);
        let on_resize = self.on_resize;
        let move_state = state.clone();
        let up_state = state.clone();
        let handle_state = state.clone();

        let handle = self.resizable.then(|| {
            div()
                .id("yp-sheet-resize")
                .absolute()
                .top_0()
                .bottom_0()
                .w(Sizes::RESIZE_HANDLE_WIDTH)
                .map(|this| match side {
                    SheetSide::Right => this.left_0(),
                    SheetSide::Left => this.right_0(),
                })
                .cursor(CursorStyle::ResizeLeftRight)
                .when(dragging.is_some(), |this| {
                    this.bg(accent.opacity(theme.alpha.soft))
                })
                .hover(move |style| style.bg(handle_hover))
                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    cx.stop_propagation();
                    handle_state.update(cx, |state, cx| {
                        state.dragging = Some((event.position.x, width));
                        cx.notify();
                    });
                })
        });
        let panel = v_flex()
            .id("yp-sheet-panel")
            .absolute()
            .top_0()
            .bottom_0()
            .w(width)
            .map(|this| match side {
                SheetSide::Right => this.right_0().border_l_1(),
                SheetSide::Left => this.left_0().border_r_1(),
            })
            .occlude()
            .bg(theme.colors.background_card)
            .border_color(theme.colors.border_light)
            .elevation_3(cx)
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .children(self.children)
            .children(handle);
        let panel = if closing {
            let distance = f32::from(width);
            panel
                .with_animation(
                    "yp-sheet-slide-out",
                    Animation::new(theme.motion.cross),
                    move |this, delta| {
                        let offset = px(distance * easing::nier_smooth(delta));
                        match side {
                            SheetSide::Right => this.right(-offset),
                            SheetSide::Left => this.left(-offset),
                        }
                    },
                )
                .into_any_element()
        } else if self.animate && dragging.is_none() && stored_width.is_none() {
            let distance = f32::from(width);
            panel
                .with_animation(
                    "yp-sheet-slide",
                    Animation::new(theme.motion.cross),
                    move |this, delta| {
                        let offset = px(distance * (1. - easing::nier_smooth(delta)));
                        match side {
                            SheetSide::Right => this.right(-offset),
                            SheetSide::Left => this.left(-offset),
                        }
                    },
                )
                .into_any_element()
        } else {
            panel.into_any_element()
        };
        div()
            .id(self.id)
            .absolute()
            .inset_0()
            .overflow_hidden()
            .child(pointer_capture(
                dragging.is_some(),
                move |event, window, cx| {
                    let Some((start_pointer, start_width)) = move_state.read(cx).dragging else {
                        return;
                    };
                    let delta = event.position.x - start_pointer;
                    let wanted = match side {
                        SheetSide::Right => start_width - delta,
                        SheetSide::Left => start_width + delta,
                    };
                    let next = clamp_sheet_width(wanted, available);
                    move_state.update(cx, |state, cx| {
                        if state.width != Some(next) {
                            state.width = Some(next);
                            cx.notify();
                        }
                    });
                    if let Some(handler) = &on_resize {
                        handler(&next, window, cx);
                    }
                },
                move |_, _, cx| {
                    up_state.update(cx, |state, cx| {
                        state.dragging = None;
                        cx.notify();
                    });
                },
            ))
            .when(self.scrim && closing, |this| {
                let scrim = theme.colors.scrim;
                let strength = theme.alpha.overlay;
                this.child(div().absolute().inset_0().with_animation(
                    "yp-sheet-scrim-out",
                    Animation::new(theme.motion.cross),
                    move |this, delta| this.bg(scrim.opacity(strength * (1. - delta))),
                ))
            })
            .when(self.scrim && !closing, |this| {
                this.child(
                    div()
                        .id("yp-sheet-scrim")
                        .absolute()
                        .inset_0()
                        .occlude()
                        .bg(theme.colors.scrim.opacity(theme.alpha.overlay))
                        .when_some(on_dismiss, |this, handler| {
                            this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                cx.stop_propagation();
                                handler(window, cx);
                            })
                        }),
                )
            })
            .child(panel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_width_stays_between_the_minimum_and_the_window() {
        assert_eq!(
            clamp_sheet_width(px(100.), px(1300.)),
            Sizes::SHEET_MIN_WIDTH
        );
        assert_eq!(clamp_sheet_width(px(5000.), px(1300.)), px(1220.));
        assert_eq!(clamp_sheet_width(px(600.), px(1300.)), px(600.));
        let narrow = clamp_sheet_width(px(600.), px(300.));
        assert!(narrow <= px(300.));
        assert!(narrow > px(0.));
    }
}
