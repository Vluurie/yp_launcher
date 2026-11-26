use super::{Slider, SliderRange};
use crate::components::util::{KeyboardFocus, bounds_tracker, pointer_capture};
use crate::theme::ResponsiveExt;
use crate::traits::ValueHandler;
use gpui::{
    AnyElement, App, Bounds, DefiniteLength, FocusHandle, IntoElement, ParentElement, Pixels,
    Window, div, prelude::*, px, relative,
};
use std::{rc::Rc, time::Instant};
use yp_nier_ui::MenuSliderTrack;

struct Editor {
    focus: FocusHandle,
    draft: Option<f32>,
    track: Bounds<Pixels>,
    changed: Instant,
    hovered: bool,
    dragging: bool,
    press_x: Option<Pixels>,
}

impl Editor {
    fn edit(&mut self, value: Option<f32>) {
        self.dragging = false;
        self.press_x = None;
        self.draft = value;
        self.changed = Instant::now();
    }
}

fn commit(
    value: f32,
    change: &Option<ValueHandler<f32>>,
    done: &Option<ValueHandler<f32>>,
    w: &mut Window,
    cx: &mut App,
) {
    if let Some(callback) = change {
        callback(&value, w, cx);
    }
    if let Some(callback) = done {
        callback(&value, w, cx);
    }
}

fn keyboard(key: &str, range: &SliderRange, value: f32) -> Option<f32> {
    match key {
        "left" | "down" => Some(range.adjacent(value, false)),
        "right" | "up" => Some(range.adjacent(value, true)),
        "home" => Some(range.snap(range.min)),
        "end" => Some(range.snap(range.max)),
        _ => None,
    }
}

pub(super) fn render(slider: Slider, w: &mut Window, cx: &mut App) -> AnyElement {
    let edit = w.use_keyed_state(slider.id.clone(), cx, |_, cx| Editor {
        focus: cx.focus_handle().tab_stop(true),
        draft: None,
        hovered: false,
        dragging: false,
        press_x: None,
        track: Bounds::default(),
        changed: Instant::now() - std::time::Duration::from_secs(1),
    });
    let (focus, draft, hovered, t) = edit.update(cx, |v, _| {
        if v.draft.is_some() && (slider.disabled || !v.focus.is_focused(w)) {
            v.edit(None);
        }
        (
            v.focus.clone(),
            v.draft,
            v.hovered,
            v.changed.elapsed().as_secs_f32(),
        )
    });
    let open = draft.is_some();
    let progress = if yp_nier_ui::ReducedMotion::enabled(cx) {
        1.
    } else {
        (t * 60. / 5.).clamp(0., 1.)
    };
    if progress < 1. {
        w.request_animation_frame();
    }
    let extension = if open { progress } else { 1. - progress };
    let range = Rc::new(slider.range);
    let value = slider.value;
    let shown = draft.unwrap_or(value);
    let fraction = range.fraction(shown);
    let (label, value_label) = slider.menu_row.unzip();
    let labelled = label.is_some();
    let default = slider
        .default_mark
        .filter(|v| v.is_finite())
        .map(|v| range.fraction(v));
    let fraction_at = move |bounds, x| MenuSliderTrack::fraction_at(bounds, x, default);
    let paint = MenuSliderTrack::new("nier-long-track", fraction)
        .editing(open)
        .focused(hovered || focus.is_focused(w))
        .enabled(!slider.disabled)
        .when_some(default, MenuSliderTrack::default_mark);
    let bounds = edit.clone();
    let hover = edit.clone();
    let click_range = range.clone();
    let enter = edit.clone();
    let exit = edit.clone();
    let pointer = edit.clone();
    let pointer_range = range.clone();
    let number = value_label.as_ref().and_then(|label| {
        label.parse::<f64>().ok()?;
        let decimals = label.split_once('.').map_or(0, |(_, digits)| digits.len());
        Some((
            slider.id.clone(),
            decimals,
            slider.on_change.clone(),
            slider.on_commit.clone(),
            range.clone(),
        ))
    });
    let drag_move = edit.clone();
    let drag_up = edit.clone();
    let drag_range = range.clone();
    let release_range = range.clone();
    let release_change = slider.on_change.clone();
    let release_done = slider.on_commit.clone();
    let button = div()
        .id(slider.id)
        .role(gpui::Role::Slider)
        .when_some(slider.accessible_label, |el, label| el.aria_label(label))
        .aria_orientation(gpui::Orientation::Horizontal)
        .aria_numeric_value(shown.into())
        .aria_min_numeric_value(range.min.min(range.max).into())
        .aria_max_numeric_value(range.min.max(range.max).into())
        .when(range.step > 0., |el| {
            el.aria_numeric_value_step(range.step.into())
        })
        .when(!slider.disabled, |el| {
            let state = edit.clone();
            let change = slider.on_change.clone();
            let done = slider.on_commit.clone();
            super::accessible_range(el, range.clone(), shown, move |next, w, cx| {
                state.update(cx, |v, cx| {
                    v.edit(None);
                    cx.notify();
                });
                commit(next, &change, &done, w, cx);
            })
        })
        .relative()
        .min_w_0()
        .max_w_full()
        .h(px(if labelled { 44. } else { 38. }))
        .map(|el| el.w(slider.width.unwrap_or(DefiniteLength::Fraction(1.))))
        .when(!slider.disabled, |el| {
            el.track_focus(&focus)
                .keyboard_focus(cx)
                .on_key_down(move |event, w, cx| {
                    let current = enter.read(cx).draft;
                    let key = event.keystroke.key.as_str();
                    match key {
                        "escape" if current.is_some() => enter.update(cx, |v, cx| {
                            v.edit(None);
                            cx.notify();
                        }),
                        "enter" | "space" => {
                            if let Some(v) = current {
                                commit(v, &slider.on_change, &slider.on_commit, w, cx);
                            }
                            enter.update(cx, |v, cx| {
                                v.edit(current.is_none().then_some(value));
                                cx.notify();
                            });
                        }
                        _ => {
                            let Some(next) = keyboard(key, &range, current.unwrap_or(value)) else {
                                return;
                            };
                            if current.is_some() {
                                enter.update(cx, |v, cx| {
                                    v.draft = Some(next);
                                    cx.notify();
                                });
                            } else {
                                commit(next, &slider.on_change, &slider.on_commit, w, cx);
                            }
                        }
                    }
                    cx.stop_propagation();
                    w.prevent_default();
                })
        });
    let control = button
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .max_w(px(if labelled { 363. } else { 252. }))
                .h_full()
                .child(
                    div()
                        .id("scale")
                        .relative()
                        .min_w_0()
                        .h_full()
                        .w(relative(if labelled {
                            1.
                        } else {
                            0.80 + extension * 0.20
                        }))
                        .when(labelled, |el| {
                            el.debug_selector(|| "nier-setting-slider-face".into())
                        })
                        .when(slider.disabled, |el| el.cursor_not_allowed())
                        .when(!slider.disabled, |el| {
                            el.cursor_pointer()
                                .on_hover(move |over, _, cx| {
                                    hover.update(cx, |s, cx| {
                                        if s.hovered != *over {
                                            s.hovered = *over;
                                            cx.notify();
                                        }
                                    })
                                })
                                .on_mouse_down(gpui::MouseButton::Left, move |event, w, cx| {
                                    let current = edit.read(cx).draft;
                                    edit.read(cx).focus.clone().focus(w, cx);
                                    let select_on_release = labelled || current.is_some();
                                    let next = if select_on_release {
                                        click_range.value_at(fraction_at(edit.read(cx).track, event.position.x))
                                    } else {
                                        value
                                    };
                                    edit.update(cx, |v, cx| {
                                        v.edit(Some(next));
                                        v.press_x = Some(event.position.x);
                                        v.dragging = select_on_release;
                                        cx.notify();
                                    });
                                    cx.stop_propagation();
                                })
                                .on_mouse_move(move |event, _, cx| {
                                    let v = pointer.read(cx);
                                    let Some(previous) = v.draft else { return };
                                    if labelled {
                                        return;
                                    }
                                    let f = fraction_at(v.track, event.position.x);
                                    let next = pointer_range.value_at(f);
                                    if next != previous {
                                        pointer.update(cx, |v, cx| {
                                            v.draft = Some(next);
                                            cx.notify();
                                        });
                                    }
                                })
                                .on_mouse_down_out(move |_, _, cx| {
                                    if exit.read(cx).draft.is_some() {
                                        exit.update(cx, |v, cx| {
                                            v.edit(None);
                                            cx.notify();
                                        });
                                    }
                                })
                        })
                        .child(bounds_tracker(move |b, _, cx| {
                            bounds.update(cx, |v, cx| {
                                if labelled && v.track.size != b.size {
                                    cx.notify();
                                }
                                v.track = b;
                            })
                        }))
                        .child(pointer_capture(
                            !slider.disabled,
                            move |ev, _, cx| {
                                let state = drag_move.read(cx);
                                if state.press_x.is_some_and(|start| {
                                    state.dragging || (ev.position.x - start).abs() >= px(3.)
                                }) {
                                    let value = drag_range
                                        .value_at(fraction_at(state.track, ev.position.x));
                                    if !state.dragging || state.draft != Some(value) {
                                        drag_move.update(cx, |s, cx| {
                                            s.dragging = true;
                                            s.draft = Some(value);
                                            cx.notify();
                                        });
                                    }
                                }
                            },
                            move |ev, w, cx| {
                                if ev.button != gpui::MouseButton::Left {
                                    return;
                                }
                                let value = drag_up.update(cx, |s, cx| {
                                    let start = s.press_x.take()?;
                                    if !s.dragging && (ev.position.x - start).abs() < px(3.) {
                                        return None;
                                    }
                                    let value = release_range.value_at(fraction_at(s.track, ev.position.x));
                                    s.edit(None);
                                    cx.notify();
                                    Some(value)
                                });
                                if let Some(value) = value {
                                    commit(value, &release_change, &release_done, w, cx);
                                }
                            },
                        ))
                        .child(paint),
                ),
        )
        .when((open || labelled) && number.is_none(), |el| {
            let value_label = value_label
                .filter(|_| !open || shown == value)
                .unwrap_or_else(|| {
                    let number = format!("{shown:.4}");
                    number
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_owned()
                        .into()
                });
            el.child(
                div()
                    .min_w_0()
                    .max_w(relative(0.36))
                    .when(labelled, |value| value.w(px(72.)))
                    .truncate()
                    .text_size(px(14.))
                    .child(value_label),
            )
        });
    let control = if let Some((id, decimals, changed, done, range)) = number {
        let font_size = w.sizes(cx).font_md;
        let style = w.text_style();
        let precision = decimals.clamp(1, 12);
        let field_width = [range.min, range.max, shown]
            .into_iter()
            .filter(|v| v.is_finite())
            .map(|v| {
                let text = format!("{v:.precision$}");
                let run = style.to_run(text.len());
                w.text_system()
                    .shape_line(text.into(), font_size, &[run], None)
                    .width
                    + px(30.)
            })
            .fold(px(62.), |a, b| a.max(b));
        let readout = crate::components::NumberField::new(
            (id, "number"),
            if shown == 0. { 0. } else { shown as f64 },
        )
        .kind(crate::components::NumberKind::Float { decimals })
        .min(range.min.min(range.max) as f64)
        .max(range.min.max(range.max) as f64)
        .step(range.step as f64)
        .disabled(slider.disabled)
        .width(relative(1.))
        .on_commit(move |v, w, cx| commit(range.snap(*v as f32), &changed, &done, w, cx));
        div()
            .w_full()
            .min_w_0()
            .flex()
            .items_center()
            .gap(px(3.))
            .child(div().flex_1().min_w_0().child(control))
            .child(
                div()
                    .debug_selector(|| "nier-setting-slider-number".into())
                    .w(field_width)
                    .flex_none()
                    .max_w_full()
                    .min_w_0()
                    .child(readout),
            )
            .into_any_element()
    } else {
        control.into_any_element()
    };
    if let Some(label) = label {
        div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .child(crate::components::text::Text::new(label))
            .child(control)
            .into_any_element()
    } else {
        control.into_any_element()
    }
}
