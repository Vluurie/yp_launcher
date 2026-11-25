use super::{MenuBackdrop, borders};
use crate::draw::ScreenArt;
use crate::motion::{ease_out, row_alpha};
use crate::{EffectLayer, GridNoise, Lens, MenuTitle, Rect, Surface};
use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Size,
    Window, canvas, deferred, div, prelude::*, px, rgb,
};
use std::time::{Duration, Instant};

type Slot = Box<dyn FnOnce(f32, &mut App) -> AnyElement>;

#[derive(IntoElement)]
pub struct MenuFrame {
    id: ElementId,
    title: SharedString,
    opened_at: Option<Instant>,
    closing_at: Option<Instant>,
    subtitle: Option<SharedString>,
    header: Option<Slot>,
    list: Option<Slot>,
    detail: Option<Slot>,
    footer: Option<Slot>,
    overlay: Option<Slot>,
    foreground: Option<Slot>,
    lens: Option<Lens>,
    noise: Option<GridNoise>,
}
impl MenuFrame {
    pub const CLOSE_DURATION: Duration = Duration::from_millis(350);

    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            opened_at: None,
            closing_at: None,
            subtitle: None,
            header: None,
            list: None,
            detail: None,
            footer: None,
            overlay: None,
            foreground: None,
            lens: None,
            noise: None,
        }
    }
    pub fn subtitle(mut self, text: impl Into<SharedString>) -> Self {
        self.subtitle = Some(text.into());
        self
    }
    pub fn opened_at(mut self, start: Instant) -> Self {
        self.opened_at = Some(start);
        self
    }
    pub fn closing_at(mut self, start: Instant) -> Self {
        self.closing_at = Some(start);
        self
    }
    pub fn header<E: IntoElement>(
        mut self,
        slot: impl FnOnce(f32, &mut App) -> E + 'static,
    ) -> Self {
        self.header = Some(Box::new(move |s, cx| slot(s, cx).into_any_element()));
        self
    }
    pub fn list<E: IntoElement>(mut self, slot: impl FnOnce(f32, &mut App) -> E + 'static) -> Self {
        self.list = Some(Box::new(move |s, cx| slot(s, cx).into_any_element()));
        self
    }
    pub fn detail<E: IntoElement>(
        mut self,
        slot: impl FnOnce(f32, &mut App) -> E + 'static,
    ) -> Self {
        self.detail = Some(Box::new(move |s, cx| slot(s, cx).into_any_element()));
        self
    }
    pub fn footer<E: IntoElement>(
        mut self,
        slot: impl FnOnce(f32, &mut App) -> E + 'static,
    ) -> Self {
        self.footer = Some(Box::new(move |s, cx| slot(s, cx).into_any_element()));
        self
    }
    pub fn foreground<E: IntoElement>(
        mut self,
        slot: impl FnOnce(f32, &mut App) -> E + 'static,
    ) -> Self {
        self.foreground = Some(Box::new(move |s, cx| slot(s, cx).into_any_element()));
        self
    }
    pub fn overlay<E: IntoElement>(
        mut self,
        slot: impl FnOnce(f32, &mut App) -> E + 'static,
    ) -> Self {
        self.overlay = Some(Box::new(move |s, cx| slot(s, cx).into_any_element()));
        self
    }
    pub fn effects(mut self, lens: Option<Lens>, noise: Option<GridNoise>) -> Self {
        self.lens = lens;
        self.noise = noise;
        self
    }
}
fn place(rect: Rect, scale: f32, child: impl IntoElement) -> impl IntoElement {
    div()
        .absolute()
        .left(px(rect.x * scale))
        .top(px(rect.y * scale))
        .w(px(rect.w * scale))
        .h(px(rect.h * scale))
        .child(child)
}
impl RenderOnce for MenuFrame {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |window, _| {
            let now = Instant::now();
            (window.viewport_size(), now, self.title.clone(), now, false)
        });
        let (area, mounted, page_start) = state.update(cx, |state, _| {
            if !state.4 && state.0.width > px(2.) && state.0.height > px(2.) {
                state.1 = Instant::now();
                state.3 = state.1;
                state.4 = true;
            }
            if state.2 != self.title {
                state.2 = self.title.clone();
                state.3 = Instant::now();
            }
            (state.0, state.1, state.3)
        });
        let scale = (area.width / px(1600.)).min(area.height / px(900.));
        let start = self.opened_at.unwrap_or(mounted);
        let age = start.elapsed().as_secs_f32() * 60.;
        let page_age = if mounted == page_start {
            (age - 20.).max(0.)
        } else {
            page_start.elapsed().as_secs_f32() * 60.
        };
        let head_y = -32. * (1. - ease_out((age - 20.) / 20.));
        let mut rules = Surface::new(1600., 900.);
        if age >= 20. {
            borders(&mut rules, head_y, ease_out((age - 20.) / 20.));
        }
        let list_age = (age - 20.).max(0.);
        let mut face = div()
            .relative()
            .w(px(1600. * scale))
            .h(px(900. * scale))
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .child(MenuBackdrop::new("backdrop").since(start)),
            )
            .child(div().absolute().inset_0().child(rules));
        if age >= 20. {
            face = face.child(place(
                Rect::new(52., 112. + head_y, 1480., 90.),
                scale,
                div()
                    .opacity(row_alpha(page_age))
                    .flex()
                    .child(MenuTitle::new(self.title.clone()).scale(scale))
                    .when_some(self.subtitle, |row, caption| {
                        row.child(div().mt(px(24. * scale)).child(
                            crate::GameText::new(format!("- {caption}")).scale(0.85 * scale),
                        ))
                    }),
            ));
            for (index, (slot, r, opacity)) in [
                (self.header, Rect::new(48., 18. + head_y, 1510., 68.), 1.),
                (self.list, Rect::new(49., 224., 507., 512.), 1.),
                (
                    self.detail,
                    Rect::new(603., 230., 945., 480.),
                    row_alpha(page_age),
                ),
                (
                    self.footer,
                    Rect::new(
                        0.,
                        450. + 344. + 56. * (1. - ease_out(list_age / 20.)) - 50.,
                        1600.,
                        100.,
                    ),
                    row_alpha(list_age.min(page_age)),
                ),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(slot) = slot {
                    face = face.child(place(
                        r,
                        scale,
                        div()
                            .id(("frame-slot", index))
                            .size_full()
                            .opacity(opacity)
                            .child(
                                div()
                                    .id(if matches!(index, 1 | 2) {
                                        self.title.clone()
                                    } else {
                                        "persistent".into()
                                    })
                                    .size_full()
                                    .child(slot(scale, cx)),
                            ),
                    ));
                }
            }
        }
        if age >= 20.
            && let Some(fg) = self.foreground
        {
            face = face.child(div().absolute().inset_0().child(fg(scale, cx)));
        }
        if let Some(overlay) = self.overlay {
            face = face.child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .child(overlay(scale, cx)),
            );
        }
        let mut overlay = Surface::new(1600., 900.);
        if let Some(start) = self.closing_at {
            super::backdrop::closing(&mut overlay, start.elapsed().as_secs_f32() * 60.);
            window.request_animation_frame();
            face = face.child(div().absolute().inset_0().occlude());
        }
        overlay.art(ScreenArt::Grid, Rect::new(0., 0., 1600., 900.));
        overlay.art(ScreenArt::Vignette, Rect::new(0., 0., 1600., 900.));
        let finish = deferred(div().absolute().inset_0().child(overlay)).with_priority(3);
        let mut filtered = EffectLayer::new(face.child(finish));
        if let Some(lens) = self.lens {
            filtered = filtered.lens(lens);
        }
        if let Some(noise) = self.noise {
            filtered = filtered.grid_noise(noise);
        }
        let tracker = canvas(
            move |b: Bounds<Pixels>, window, cx| {
                let measured: Size<Pixels> = b.size;
                if measured != state.read(cx).0 {
                    let state = state.clone();
                    window.defer(cx, move |_, cx| {
                        state.update(cx, |state, cx| {
                            if state.0 != measured {
                                state.0 = measured;
                                cx.notify();
                            }
                        })
                    });
                }
            },
            |_, _, _, _| (),
        )
        .absolute()
        .inset_0()
        .size_full();
        div()
            .id(self.id)
            .size_full()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .bg(rgb(0xcac6ad))
            .child(tracker)
            .child(filtered)
    }
}
