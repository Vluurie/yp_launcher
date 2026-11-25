use super::{Changed, State};
use gpui::{
    App, Bounds, Entity, IntoElement, Pixels, Point, RenderOnce, Window, canvas, div, prelude::*,
    px,
};
use std::{cell::Cell, rc::Rc};

#[derive(IntoElement)]
pub(super) struct SegmentInput {
    pub state: Entity<State>,
    pub first_center: f32,
    pub scale: f32,
    pub steps: usize,
    pub minimum: usize,
    pub changed: Changed,
}

#[derive(Clone, Copy)]
struct Track {
    bounds: Bounds<Pixels>,
    scale: f32,
    steps: usize,
    minimum: usize,
}
impl Track {
    fn point_at(&self, p: Point<Pixels>, state: &mut State, device_scale: f32) -> bool {
        let Some(old) = state.draft else { return false };
        let new = if self.bounds.contains(&p) {
            state.last_hit_x = Some(p.x);
            let x = (p.x - self.bounds.left()) / px(self.scale);
            let value = 1 + ((x - 12.) / 8.).ceil().max(0.) as usize;
            if value < self.minimum {
                return false;
            }
            value.min(self.steps)
        } else {
            let last = state.last_hit_x.take();
            if self.minimum == 0
                && old == 1
                && last.is_some_and(|last| (last - p.x) * device_scale >= px(1.))
            {
                0
            } else {
                return false;
            }
        };
        state.draft = Some(new);
        old != new
    }
}
impl RenderOnce for SegmentInput {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let track = Rc::new(Cell::new(Track {
            bounds: Default::default(),
            scale: self.scale,
            steps: self.steps,
            minimum: self.minimum,
        }));
        let measure = track.clone();
        let hover = self.state.clone();
        let motion = track.clone();
        let leave = self.state.clone();
        let edge = track.clone();
        div()
            .id("bar-input")
            .absolute()
            .top(px(20. * self.scale))
            .left(px((self.first_center - 6.) * self.scale))
            .w(px((12. + (self.steps - 1) as f32 * 8.) * self.scale))
            .h(px(24. * self.scale))
            .cursor_pointer()
            .on_mouse_move(move |ev, w, cx| {
                hover.update(cx, |s, cx| {
                    if motion.get().point_at(ev.position, s, w.scale_factor()) {
                        cx.notify();
                    }
                });
            })
            .on_hover(move |over, w, cx| {
                if !w.last_input_was_keyboard() && (*over || leave.read(cx).last_hit_x.is_some()) {
                    leave.update(cx, |s, cx| {
                        if edge.get().point_at(w.mouse_position(), s, w.scale_factor()) {
                            cx.notify();
                        }
                    });
                }
            })
            .on_click(move |ev, w, cx| {
                let value = self.state.update(cx, |s, cx| {
                    if let Some(p) = ev.mouse_position() {
                        track.get().point_at(p, s, w.scale_factor());
                    }
                    let draft = s.draft;
                    s.exit(w, cx);
                    cx.notify();
                    draft
                });
                if let Some(value) = value {
                    (self.changed)(value, w, cx);
                }
                cx.stop_propagation();
            })
            .child(
                canvas(
                    move |bounds, _, _| {
                        measure.set(Track {
                            bounds,
                            ..measure.get()
                        })
                    },
                    |_, _, _, _| (),
                )
                .size_full(),
            )
    }
}
