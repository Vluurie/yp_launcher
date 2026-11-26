use super::super::control::{layout::Transition, mix};
use crate::{INK, LIGHT, PANEL, PAPER, Rect, ReducedMotion, Surface, Tint};
use gpui::{
    App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, Window, canvas,
    prelude::*,
};

pub(crate) fn stem(art: &mut Surface, x: f32, center: f32, height: f32, ink: Tint) {
    let width = 3.;
    art.fill(
        Rect::new(x - width / 2., center - height / 2., width, height),
        ink,
        255,
    );
}

pub(crate) fn default_mark(art: &mut Surface, x: f32, center: f32, editing: bool, ink: Tint) {
    let (width, height, top, bottom) = if editing {
        (6., 3., -18., 15.)
    } else {
        (4., 2., -19., 17.)
    };
    for offset in [top, bottom] {
        art.fill(
            Rect::new(x - width / 2., center + offset, width, height),
            ink,
            255,
        );
    }
}

pub(super) struct SegmentMotion {
    filled: Transition,
    active: Transition,
}

impl SegmentMotion {
    pub(super) fn new(filled: bool, active: bool) -> Self {
        Self {
            filled: Transition::new(filled),
            active: Transition::new(active),
        }
    }

    pub(super) fn sample(&mut self, filled: bool, active: bool, editing: bool) -> (f32, bool) {
        let (fill, a) = self.filled.sample(filled, 5.);
        let (accent, b) = self.active.sample(active, 4.);
        let base = if editing { 4. } else { 3. };
        (base + (16. - base) * fill + 6. * accent, a || b)
    }
}

const STEPS: usize = 20;
const FIRST: f32 = 12.;
const SPACING: f32 = 8.;
const SPAN: f32 = (STEPS - 1) as f32 * SPACING;
const WIDTH: f32 = FIRST * 2. + SPAN;

fn mark_position(default: f32) -> f32 {
    if default == 0. || default == 1. {
        return default;
    }
    let last = (STEPS - 1) as f32;
    (default * last).round().clamp(1., last - 1.) / last
}

fn map_at_mark(value: f32, from: f32, to: f32) -> f32 {
    if from == to {
        value
    } else if value <= from {
        value / from * to
    } else {
        to + (value - from) / (1. - from) * (1. - to)
    }
}

fn pointer_value(position: f32, default: Option<f32>) -> f32 {
    let position = position.clamp(0., 1.);
    let Some(default) = default.filter(|v| v.is_finite()).map(|v| v.clamp(0., 1.)) else {
        return position;
    };
    let mark = mark_position(default);
    if (position - mark).abs() * SPAN <= 2. {
        default
    } else {
        map_at_mark(position, mark, default)
    }
}

#[derive(IntoElement)]
pub struct MenuSliderTrack {
    id: ElementId,
    label: Option<SharedString>,
    extent: f32,
    value: f32,
    default: Option<f32>,
    focused: bool,
    editing: bool,
    enabled: bool,
}

impl MenuSliderTrack {
    pub fn new(id: impl Into<ElementId>, fraction: f32) -> Self {
        Self {
            id: id.into(),
            label: None,
            extent: 0.,
            value: fraction.clamp(0., 1.),
            default: None,
            focused: false,
            editing: false,
            enabled: true,
        }
    }

    pub fn setting_row(mut self, label: SharedString, extent: f32) -> Self {
        self.label = Some(label);
        self.extent = extent.clamp(0., 1.);
        self
    }

    pub fn label_fits(bounds: Bounds<Pixels>, label: &str) -> bool {
        let (w, _) = row_size(bounds);
        crate::Theme::bundled().measure(label, 36, 0.8)[0] <= (w - 325.).max(0.)
    }

    pub fn row_fraction_at(
        bounds: Bounds<Pixels>,
        x: Pixels,
        extent: f32,
        default: Option<f32>,
    ) -> Option<f32> {
        if bounds.size.width <= gpui::px(0.) {
            return None;
        }
        let (width, _) = row_size(bounds);
        let logical = (x - bounds.left()) / bounds.size.width * width;
        let first = row_first(width, extent);
        ((first - 6. ..=first + SPAN + 6.).contains(&logical))
            .then(|| pointer_value((logical - first) / SPAN, default))
    }

    pub fn default_mark(mut self, fraction: f32) -> Self {
        self.default = fraction.is_finite().then(|| fraction.clamp(0., 1.));
        self
    }

    pub fn focused(mut self, value: bool) -> Self {
        self.focused = value;
        self
    }
    pub fn editing(mut self, value: bool) -> Self {
        self.editing = value;
        self
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }

    pub fn fraction_at(bounds: Bounds<Pixels>, x: Pixels, default: Option<f32>) -> f32 {
        if bounds.size.width <= gpui::px(0.) {
            return 0.;
        }
        let position =
            (((x - bounds.left()) / bounds.size.width * WIDTH - FIRST) / SPAN).clamp(0., 1.);
        default
            .filter(|v| (0.0..=1.0).contains(v) && (position - v).abs() * SPAN <= 2.)
            .unwrap_or(position)
    }
}

fn row_size(bounds: Bounds<Pixels>) -> (f32, f32) {
    let scale = (bounds.size.height / gpui::px(64.))
        .min(bounds.size.width / gpui::px(300.))
        .max(0.01);
    (
        f32::from(bounds.size.width) / scale,
        f32::from(bounds.size.height) / scale,
    )
}

fn row_first(width: f32, extent: f32) -> f32 {
    width - 100. + 36. + 32.5 * extent - SPAN
}

impl RenderOnce for MenuSliderTrack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let selected = self.enabled && (self.focused || self.editing);
        let continuous = self.label.is_none();
        let mark = if continuous {
            self.default
        } else {
            self.default.map(mark_position)
        };
        let position = self
            .default
            .zip(mark)
            .map_or(self.value, |(default, mark)| {
                map_at_mark(self.value, default, mark)
            });
        let active = (position * (STEPS - 1) as f32).round() as usize;
        let targets: [(bool, bool); STEPS] = std::array::from_fn(|i| {
            let filled = if continuous {
                i as f32 * SPACING < position * SPAN
            } else {
                i <= active
            };
            (
                self.value > 0. && filled,
                !continuous && self.editing && i == active,
            )
        });
        let (focus, heights) = if ReducedMotion::enabled(cx) {
            (
                f32::from(u8::from(selected)),
                targets.map(|(filled, active)| {
                    if active {
                        22.
                    } else if filled {
                        16.
                    } else {
                        3.
                    }
                }),
            )
        } else {
            let state = window.use_keyed_state(self.id, cx, |_, _| {
                (
                    Transition::new(false),
                    targets.map(|(f, a)| SegmentMotion::new(f, a)),
                )
            });
            let (focus, heights, moving) = state.update(cx, |(sweep, segments), _| {
                let (focus, mut moving) = sweep.sample(selected, if selected { 10. } else { 5. });
                let heights: [f32; STEPS] = std::array::from_fn(|i| {
                    let (height, animated) =
                        segments[i].sample(targets[i].0, targets[i].1, self.editing);
                    moving |= animated;
                    height
                });
                (focus, heights, moving)
            });
            if moving {
                window.request_animation_frame();
            }
            (focus, heights)
        };
        let ink = if self.enabled {
            INK
        } else {
            mix(INK, PANEL, 0.52)
        };
        let editing = self.editing;
        let paint = move |s: &mut Surface, first: f32, color| {
            for (i, height) in heights.iter().enumerate() {
                let offset = i as f32 * SPACING;
                let distance = offset - position * SPAN;
                if continuous && distance.abs() < 5. {
                    continue;
                }
                let height = if continuous && distance > 0. {
                    3.
                } else {
                    *height
                };
                stem(s, first + offset, 32., height, color);
            }
            if continuous {
                let x = first + position * SPAN;
                s.fill(Rect::new(x - 1.5, 21., 3., 22.), color, 255);
            }
            if let Some(mark) = mark {
                default_mark(s, first + mark * SPAN, 32., editing || continuous, color);
            }
        };
        if let Some(label) = self.label {
            return canvas(
                |_, _, _| (),
                move |bounds, _, w, cx| {
                    let (width, height) = row_size(bounds);
                    let body_width = width - 100.;
                    let mut art = super::super::control::slider_face(
                        width,
                        height,
                        body_width,
                        &label,
                        self.enabled,
                        focus,
                        self.extent,
                    );
                    let first = row_first(width, self.extent);
                    paint(&mut art, first, ink);
                    if focus > 0. {
                        art.clip(
                            Rect::new(60., 12., (body_width + 36. * self.extent) * focus, 40.),
                            |s| paint(s, first, LIGHT),
                        );
                    }
                    art.paint(bounds, w, cx);
                },
            )
            .size_full()
            .into_any_element();
        }
        let mut art = Surface::new(WIDTH, 64.);
        let body = Rect::new(1., 12., WIDTH - 4., 40.);
        let paper = if self.enabled {
            PANEL
        } else {
            mix(PANEL, PAPER, 0.55)
        };
        art.fill(Rect::new(3., 15., body.w, 40.), INK, 22);
        art.fill(body, paper, 255);
        paint(&mut art, FIRST, ink);
        if focus > 0. {
            let swept = Rect::new(body.x, body.y, body.w * focus, body.h);
            art.fill(swept, INK, 255);
            art.clip(swept, |s| paint(s, FIRST, LIGHT));
            for y in [8., 55.] {
                art.fill(
                    Rect::new(body.x + body.w - swept.w, y, swept.w, 1.5),
                    INK,
                    (210. * focus) as u8,
                );
            }
        }
        art.into_any_element()
    }
}
