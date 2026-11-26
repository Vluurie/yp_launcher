use super::{MenuDropdown, State, commit};
use crate::{
    INK, LIGHT, MenuScrollbar, PANEL, PulldownOption, Rect, Surface, components::modal_backdrop,
};
use gpui::{
    App, Entity, IntoElement, MouseButton, RenderOnce, Window, anchored, deferred, div, point,
    prelude::*, px,
};
use std::time::Duration;

#[derive(IntoElement)]
pub(super) struct Popup {
    pub dropdown: MenuDropdown,
    pub state: Entity<State>,
    pub large: bool,
}
impl RenderOnce for Popup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let drop = self.dropdown;
        let state = self.state;
        let s = state.read(cx);
        let count = drop.choices.len();
        let shown = count.min(s.shown);
        let pos = s.scroll_position();
        let bounds = s.bounds;
        let closing = s.closing.is_some();
        let clock = s.closing.unwrap_or(s.opened).elapsed().as_secs_f32();
        let selection = s.cursor.unwrap_or(drop.selected);
        let scale = drop.scale;
        let large = self.large;
        let (width, paper, height, row_h, viewport_top, first_center, left) = if large {
            (380., 328., 506., 53., 10., 44., 33.)
        } else {
            (
                289.,
                236.,
                113. + 45. * (shown as f32 - 2.),
                45.,
                0.,
                34.,
                0.,
            )
        };
        let t = crate::motion::ease_out(clock * 6.);
        let alpha = if closing { 1. - t } else { t };
        if clock < 10. / 60.
            || s.scroll_from
                .is_some_and(|(_, t)| t.elapsed() < Duration::from_secs_f32(19. / 60.))
        {
            window.request_animation_frame();
        }
        let screen = window.viewport_size();
        let outside = state.clone();
        let backdrop = div()
            .id("pulldown-backdrop")
            .occlude()
            .w(screen.width)
            .h(screen.height)
            .child(modal_backdrop(alpha))
            .on_mouse_down(MouseButton::Left, move |_, w, cx| {
                outside.update(cx, |s, cx| {
                    s.close(w, cx);
                    cx.notify();
                });
                cx.stop_propagation();
            });
        let mut bg = Surface::new(width, height + 3.);
        bg.fill(Rect::new(3., 3., paper, height), INK, (45. * alpha) as u8);
        bg.fill(
            Rect::new(0., 0., paper, height),
            LIGHT,
            (255. * alpha) as u8,
        );
        for y in [4., height - 6.] {
            bg.fill(
                Rect::new(7., y, paper - 14., 1.5),
                PANEL,
                (255. * alpha) as u8,
            );
        }
        let first = pos.floor() as usize;
        let rows = drop
            .choices
            .iter()
            .enumerate()
            .skip(first)
            .take(shown + usize::from(large))
            .map(|(idx, item)| {
                let hover = state.clone();
                let click = state.clone();
                let changes = drop.changed.clone();
                let choices = drop.choices.clone();
                let button = PulldownOption::new(("choice", idx), item.label.clone())
                    .scrolling(large)
                    .when(large, |option| option.width(paper - left))
                    .scale(scale)
                    .selected(selection == idx)
                    .enabled(item.enabled)
                    .on_hover(move |over, _, cx| {
                        if reselect_on_motion(over, &hover, idx, cx) {
                            hover.update(cx, |s, cx| {
                                s.cursor = Some(idx);
                                cx.notify();
                            });
                        }
                    })
                    .on_click(move |_, w, cx| {
                        if click.read(cx).cursor.is_some() {
                            commit(idx, &click, &choices, &changes, w, cx);
                        }
                        cx.stop_propagation();
                    });
                div()
                    .absolute()
                    .left(px(left * scale))
                    .top(px((first_center - viewport_top - row_h * 0.5
                        + (idx as f32 - pos) * row_h)
                        * scale))
                    .child(button)
            });
        let wheel = state.clone();
        let mut panel = div()
            .id("choices")
            .occlude()
            .relative()
            .w(px(width * scale))
            .h(px((height + 3.) * scale))
            .on_scroll(move |ev, _, cx| {
                let y = ev.delta.pixel_delta(px(row_h * scale)).y;
                if let Some(i) = wheel.read(cx).cursor
                    && y != px(0.)
                {
                    let next = if y > px(0.) {
                        i.saturating_sub(1)
                    } else {
                        (i + 1).min(count - 1)
                    };
                    wheel.update(cx, |s, cx| {
                        s.move_to(next, count);
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
            })
            .child(div().absolute().inset_0().child(bg))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(viewport_top * scale))
                    .h(px((if large { 487. } else { height }) * scale))
                    .overflow_hidden()
                    .opacity(alpha)
                    .children(rows),
            );
        if large && shown > 0 {
            let bar = state.clone();
            let scroll =
                MenuScrollbar::new("choice-scroll", shown, count, pos, move |value, _, cx| {
                    bar.update(cx, |s, cx| {
                        let value = value.clamp(0., count.saturating_sub(shown) as f32);
                        let slot = s
                            .cursor
                            .unwrap_or(s.first)
                            .saturating_sub(s.first)
                            .min(shown - 1);
                        s.first = value.floor() as usize;
                        s.position = value;
                        s.scroll_from = None;
                        s.cursor = Some(s.first + slot);
                        cx.notify();
                    });
                })
                .compact()
                .height(height)
                .scale(scale);
            panel = panel.child(
                div()
                    .absolute()
                    .left(px(-7. * scale))
                    .top_0()
                    .opacity(alpha)
                    .child(scroll),
            );
        }
        let dy = if large {
            12. - 66. * drop.slot.min(7) as f32
        } else {
            12.
        };
        let [dx, dy] = drop.anchor.unwrap_or([730., dy]);
        div()
            .child(
                deferred(anchored().position(point(px(0.), px(0.))).child(backdrop))
                    .with_priority(1),
            )
            .child(
                deferred(
                    anchored()
                        .position(bounds.origin + point(px(dx * scale), px(dy * scale)))
                        .snap_to_window_with_margin(px(3.))
                        .child(panel),
                )
                .with_priority(2),
            )
    }
}

fn reselect_on_motion(over: bool, state: &Entity<State>, i: usize, cx: &App) -> bool {
    let s = state.read(cx);
    over && s.cursor.is_some_and(|current| current != i)
        && s.scroll_from
            .is_none_or(|(_, t)| t.elapsed() >= Duration::from_secs_f32(19. / 60.))
}
