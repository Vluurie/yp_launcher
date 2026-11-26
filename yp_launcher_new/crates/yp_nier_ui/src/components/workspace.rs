use super::MenuBackdrop;
use crate::{Rect, Surface, menu_screen_overlay, motion::ease_out};
use gpui::{
    AnyElement, App, Context, ElementId, IntoElement, ParentElement, Render, RenderOnce, Window,
    canvas, div, prelude::*, px, relative, rgb,
};
use std::time::Instant;

struct Paper(Instant);

impl Render for Paper {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        MenuBackdrop::new("rings").since(self.0)
    }
}

pub fn menu_rule() -> impl IntoElement {
    border_band(1., false)
}

fn border_band(progress: f32, upper: bool) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            let width = f32::from(bounds.size.width);
            let height = f32::from(bounds.size.height);
            let mut band = Surface::new(width, height);
            let visible = width * progress;
            let left = if upper { width - visible } else { 0. };
            let sprite_top = if upper { -23.5 } else { -22. };
            band.clip(Rect::new(left, 0., visible, height), |band| {
                let center = width / 2.;
                for i in 0..(center / 800.).ceil() as usize {
                    let offset = i as f32 * 800.;
                    for (sprite, x) in [
                        ("divider-reverse", center - offset - 800.),
                        ("divider", center + offset),
                    ] {
                        band.sprite(sprite, Rect::new(x, sprite_top, 800., 48.), crate::INK);
                    }
                }
            });
            band.paint(bounds, window, cx);
        },
    )
    .w_full()
    .h(px(14.))
    .flex_none()
}

#[derive(IntoElement)]
pub struct MenuWorkspace {
    id: ElementId,
    child: AnyElement,
    footer: Option<AnyElement>,
    screen_overlay: bool,
}

impl MenuWorkspace {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
            footer: None,
            screen_overlay: true,
        }
    }

    pub fn screen_overlay(mut self, enabled: bool) -> Self {
        self.screen_overlay = enabled;
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

impl RenderOnce for MenuWorkspace {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let paper = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| {
                cx.new(|_| Paper(Instant::now()))
            })
            .read(cx)
            .clone();
        let t = paper.read(cx).0.elapsed().as_secs_f32() * 60.;
        let reduced = crate::ReducedMotion::enabled(cx);
        let border = if reduced {
            1.
        } else {
            ease_out((t - 20.) / 20.)
        };
        let opacity = if reduced {
            1.
        } else {
            ease_out((t - 21.) / 18.)
        };
        if opacity < 1. || border < 1. {
            window.request_animation_frame();
        }
        div()
            .id(self.id)
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(rgb(0xcac6ad))
            .child(div().absolute().inset_0().child(paper))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .flex_col()
                    .child(border_band(border, true))
                    .child(
                        div().relative().flex_1().min_h_0().w_full().child(
                            div()
                                .absolute()
                                .inset_0()
                                .opacity(opacity)
                                .child(self.child),
                        ),
                    )
                    .child(border_band(border, false))
                    .when_some(self.footer, |el, footer| {
                        el.child(div().w_full().flex_none().opacity(opacity).child(footer))
                    }),
            )
            .when(self.screen_overlay, |el| el.child(menu_screen_overlay()))
    }
}

#[derive(IntoElement)]
pub struct PageReveal {
    child: AnyElement,
    start: Instant,
}

impl PageReveal {
    pub fn new(child: impl IntoElement, start: Instant) -> Self {
        Self {
            child: child.into_any_element(),
            start,
        }
    }
}

impl RenderOnce for PageReveal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduced = crate::ReducedMotion::enabled(cx);
        let elapsed = self.start.elapsed();
        let progress = if reduced {
            1.
        } else {
            elapsed.as_secs_f32() / 0.20
        };
        if progress < 1. {
            window.request_animation_frame();
        }
        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .child(crate::text_reveal::ListRowText::new(
                        self.child,
                        (!reduced && elapsed < crate::text_reveal::PAGE_TEXT_LIMIT)
                            .then_some(self.start),
                    )),
            )
            .when(progress < 1., |el| {
                el.child(
                    div()
                        .absolute()
                        .inset_0()
                        .top(relative(ease_out(progress)))
                        .bg(rgb(0xcac6ad)),
                )
            })
    }
}
