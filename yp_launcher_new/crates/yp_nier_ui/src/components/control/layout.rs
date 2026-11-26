use super::{Control, Kind, mix};
use crate::{INK, LIGHT, PANEL, PAPER, Rect, Surface, Theme};
use std::time::Instant;

pub(crate) struct Transition {
    from: f32,
    to: f32,
    changed: Instant,
}
impl Transition {
    pub fn new(target: bool) -> Self {
        let t = u8::from(target) as f32;
        Self {
            from: t,
            to: t,
            changed: Instant::now(),
        }
    }
    pub fn sample(&mut self, target: bool, frames: f32) -> (f32, bool) {
        self.approach(target, frames, 2)
    }
    fn approach(&mut self, target: bool, frames: f32, power: i32) -> (f32, bool) {
        let to = u8::from(target) as f32;
        let time = (self.changed.elapsed().as_secs_f32() * 60. / frames).min(1.);
        let now = self.from + (self.to - self.from) * (1. - (1. - time).powi(power));
        if self.to != to {
            self.from = now;
            self.to = to;
            self.changed = Instant::now();
        }
        (now, (now - self.to).abs() > 0.001)
    }
}
pub(super) struct ControlState {
    pub hovered: bool,
    focus: Transition,
    extent: Transition,
}
impl ControlState {
    pub fn new() -> Self {
        Self {
            hovered: false,
            focus: Transition::new(false),
            extent: Transition::new(false),
        }
    }
    pub fn update(&mut self, control: &Control, kind: Kind, reduced: bool) -> (f32, f32, bool) {
        let focused = (control.enabled || kind.tracks_unavailable_cursor())
            && (control.selected.unwrap_or(self.hovered) || (kind == Kind::Tab && self.hovered));
        let expanded = control.expanded || (kind == Kind::Setting && control.editing);
        if reduced {
            self.focus = Transition::new(focused);
            self.extent = Transition::new(expanded);
            return (self.focus.to, self.extent.to, false);
        }
        let frames = match (focused, kind) {
            (true, Kind::Row | Kind::Setting | Kind::Item) => 26.,
            (true, Kind::Tab) => 20.,
            _ => 10.,
        };
        let (focus, a) = self.focus.approach(focused, frames, 4);
        let (extent, b) = self.extent.sample(expanded, 10.);
        (focus, extent, a || b)
    }
}
impl Kind {
    pub fn tracks_unavailable_cursor(self) -> bool {
        matches!(self, Self::Setting | Self::Choice | Self::ScrollChoice)
    }
}

pub(super) fn art(
    control: &Control,
    kind: Kind,
    w: f32,
    h: f32,
    focus: f32,
    extent: f32,
    label: &str,
) -> Surface {
    let mut art = Surface::new(w, h);
    let box_width = control.width
        + if kind == Kind::Setting && control.slider.is_some() {
            36. * extent
        } else {
            0.
        };
    let (x, y, body_height) = match kind {
        Kind::Choice => (0., 2.5, 40.),
        Kind::ScrollChoice => (0., 6.5, 40.),
        Kind::Tab => (50., 12., 42. + 20. * extent),
        Kind::Row => (50., 12., 40.),
        Kind::Item => (60., 5., 40.),
        _ => (60., 12., 40.),
    };
    let box_rect = Rect::new(x, y, box_width, body_height);
    let idle = if control.enabled {
        PANEL
    } else {
        mix(PANEL, PAPER, 0.55)
    };
    let idle_ink = if control.enabled {
        INK
    } else {
        mix(INK, PANEL, 0.52)
    };
    let focus_ink = if control.enabled { LIGHT } else { PANEL };
    let shadow = if kind == Kind::Row || kind == Kind::Tab {
        (55. * focus) as u8
    } else {
        22
    };
    art.fill(
        Rect::new(x + 3., y + 3., box_width, body_height),
        INK,
        shadow,
    );
    art.fill(box_rect, idle, 255);
    let center = y + 20.;
    let place = if matches!(kind, Kind::Choice | Kind::ScrollChoice) {
        8.
    } else {
        10.
    };
    let paint_content = |art: &mut Surface, content| {
        let icon = control.icon.as_deref().unwrap_or("square");
        let button = matches!(kind, Kind::Choice | Kind::ScrollChoice);
        let label_x = if button { x + place } else { x + 35. };
        if !button {
            let side = if kind == Kind::Tab { 25. } else { 20. };
            art.sprite(
                icon,
                Rect::new(x + 20. - side / 2., center - side / 2., side, side),
                content,
            );
        }
        let right = x + box_width - 9.;
        let mut label_end = right;
        if let Some(value) = &control.value {
            let v = Theme::bundled().measure(value, 36, 0.8)[0].min(box_width * 0.52);
            label_end = right - v - 13.;
            art.clip(Rect::new(right - v, y, v + 2., 40.), |s| {
                s.text(value, 36, 0.8, right - v, center, content)
            });
        }
        if let Some(q) = control.quantity {
            let qty = q.to_string();
            let tw = Theme::bundled().measure(&qty, 6, 0.8)[0];
            label_end = right - tw - 10.;
            art.text(&qty, 6, 0.8, right - tw, center, content);
        }
        if let Some((n, max, mark, editing)) = control.slider {
            let first = right - 9. - max as f32 * 8.;
            label_end = first - 10.;
            if !editing {
                for i in 0..max {
                    let x = first + 8. * i as f32;
                    let height = if i < n { 16. } else { 3. };
                    crate::components::slider::track::stem(art, x + 2., center, height, content);
                    if mark.is_some_and(|v| v.saturating_sub(1) == i) {
                        crate::components::slider::track::default_mark(art, x + 2., center, false, content);
                    }
                }
            }
        }
        art.clip(
            Rect::new(label_x, y, (label_end - label_x).max(0.), 40.),
            |s| s.text(label, 36, 0.8, label_x, center, content),
        );
    };
    if focus < 1. {
        paint_content(&mut art, idle_ink);
    }
    if focus > 0. {
        let swept = Rect::new(x, y, box_width * focus, body_height);
        art.fill(swept, INK, 255);
        art.clip(swept, |s| paint_content(s, focus_ink));
        if matches!(kind, Kind::Row | Kind::Setting | Kind::Item | Kind::Tab) {
            let length = box_width * focus;
            for yy in [y - 4., y + body_height + 3.] {
                art.fill(
                    Rect::new(x + box_width - length, yy, length, 1.5),
                    INK,
                    (210. * focus) as u8,
                );
            }
        }
        let arrow = if matches!(kind, Kind::Choice | Kind::ScrollChoice) {
            x + box_width + 4.
        } else {
            x - 43.
        };
        art.sprite(
            "pointer",
            Rect::new(arrow - 7. * (1. - focus), center - 9., 37., 18.),
            mix(PAPER, INK, focus),
        );
    }
    art
}
