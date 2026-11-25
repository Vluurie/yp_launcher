use crate::{INK, LIGHT, PANEL, Rect, Surface};
use gpui::{
    App, ElementId, IntoElement, MouseButton, MouseMoveEvent, MouseUpEvent, Pixels, RenderOnce,
    UniformListScrollHandle, Window, canvas, div, point, prelude::*, px,
};
use std::rc::Rc;

type Move = Rc<dyn Fn(f32, &mut Window, &mut App)>;
#[derive(Clone)]
enum Source {
    Rows {
        visible: f32,
        position: f32,
        change: Move,
    },
    List {
        handle: UniformListScrollHandle,
        row_height: f32,
    },
}
impl Source {
    fn rows(&self, scale: f32, height: f32, count: usize) -> (f32, f32) {
        match self {
            Self::Rows {
                visible, position, ..
            } => (*visible, *position),
            Self::List { handle, row_height } => {
                let base = &handle.0.borrow().base_handle;
                let row = px(row_height * scale);
                let measured = base.bounds().size.height;
                if measured <= px(0.) || row <= px(0.) {
                    return ((height / row_height).min(count as f32), 0.);
                }
                (measured / row, -base.offset().y / row)
            }
        }
    }
    fn seek(&self, index: f32, scale: f32, w: &mut Window, cx: &mut App) {
        match self {
            Self::Rows { change, .. } => change(index, w, cx),
            Self::List { handle, row_height } => {
                let base = &handle.0.borrow().base_handle;
                base.set_offset(point(base.offset().x, -px(index * row_height * scale)));
                w.refresh();
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Press {
    Up,
    Down,
    Thumb,
}
struct State {
    pressed: Option<Press>,
    dragging: Option<(Pixels, f32)>,
}
impl State {
    fn press(&mut self, target: Option<Press>) {
        self.pressed = target;
    }
}

#[derive(IntoElement)]
pub struct MenuScrollbar {
    id: ElementId,
    source: Source,
    count: usize,
    height: f32,
    scale: f32,
    compact: bool,
}
impl MenuScrollbar {
    pub fn new(
        id: impl Into<ElementId>,
        visible: usize,
        total: usize,
        first: f32,
        on_scroll: impl Fn(f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            source: Source::Rows {
                visible: visible as f32,
                position: first,
                change: Rc::new(on_scroll),
            },
            count: total,
            compact: false,
            height: 503.,
            scale: 1.,
        }
    }
    pub fn for_list(
        id: impl Into<ElementId>,
        handle: UniformListScrollHandle,
        count: usize,
        row_height: f32,
    ) -> Self {
        Self {
            id: id.into(),
            source: Source::List {
                handle,
                row_height: row_height.max(1.),
            },
            count,
            compact: false,
            height: 503.,
            scale: 1.,
        }
    }
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.);
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.);
        self
    }
}
impl RenderOnce for MenuScrollbar {
    fn render(self, win: &mut Window, cx: &mut App) -> impl IntoElement {
        let scale = self.scale;
        let height = self.height;
        let (visible, pos) = self.source.rows(scale, height, self.count);
        let max = (self.count as f32 - visible).max(0.);
        let pos = pos.clamp(0., max);
        let root = div()
            .id(self.id.clone())
            .relative()
            .w(px(40. * scale))
            .h(px(height * scale));
        let (ends, center, button) = if self.compact {
            (50., 25., 25.)
        } else {
            (92., 55., 46.)
        };
        if self.count == 0 || scale <= 0. || height <= ends || (max <= 0.001 && !self.compact) {
            return root;
        }
        let track = height - ends;
        let thumb = (track * visible / self.count as f32).clamp(20_f32.min(track), track);
        let step = if max > 0. { (track - thumb) / max } else { 0. };
        let offset = step * pos;
        let can = [pos > 0.001, pos + 0.001 < max];
        let state = win.use_keyed_state(self.id, cx, |_, _| State {
            pressed: None,
            dragging: None,
        });
        let pressed = state.read(cx).pressed;
        let mut rail = Surface::new(40., height);
        let top = height - center - track;
        rail.fill(Rect::new(19., top, 2., track), PANEL, 255);
        if max > 0. {
            rail.fill(
                Rect::new(16., top + offset, 8., thumb),
                if pressed == Some(Press::Thumb) {
                    LIGHT
                } else {
                    INK
                },
                255,
            );
        }
        for (kind, y, up, enabled) in [
            (Press::Up, button / 2., true, can[0]),
            (Press::Down, height - button / 2., false, can[1]),
        ] {
            let tint = if pressed == Some(kind) {
                LIGHT
            } else if enabled {
                INK
            } else {
                PANEL
            };
            for i in 0..5 {
                let d = i as f32;
                let yy = if up { y - 5. + d * 2. } else { y + 5. - d * 2. };
                rail.fill(Rect::new(19. - d * 1.6, yy, 2. + d * 3.2, 2.), tint, 255);
            }
        }
        let mut root = root.child(rail);
        for (kind, start_y, size, enabled) in [
            (Press::Up, 0., button, can[0]),
            (Press::Thumb, height - center - track + offset, thumb, true),
            (Press::Down, height - button, button, can[1]),
        ] {
            let local = state.clone();
            let source = self.source.clone();
            let target = div()
                .id(match kind {
                    Press::Up => "up",
                    Press::Thumb => "thumb",
                    Press::Down => "down",
                })
                .absolute()
                .left(px(7.5 * scale))
                .top(px(start_y * scale))
                .w(px(25. * scale))
                .h(px(size * scale))
                .when(enabled, |el| {
                    el.cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |e, w, cx| {
                            local.update(cx, |s, cx| {
                                s.press(Some(kind));
                                if kind == Press::Thumb {
                                    s.dragging = Some((e.position.y, pos));
                                }
                                cx.notify();
                            });
                            if kind != Press::Thumb {
                                source.seek(
                                    (pos.round() + if kind == Press::Up { -1. } else { 1. })
                                        .clamp(0., max),
                                    scale,
                                    w,
                                    cx,
                                );
                            }
                            cx.stop_propagation();
                        })
                });
            root = root.child(target);
        }
        let motion = state.clone();
        let source = self.source;
        root.child(
            canvas(
                |_, _, _| (),
                move |_, _, w, _| {
                    w.on_mouse_event(move |ev: &MouseMoveEvent, phase, w, cx| {
                        if !phase.capture() || step <= 0. {
                            return;
                        }
                        if let Some((at, index)) = motion.read(cx).dragging {
                            let dest =
                                (index + (ev.position.y - at) / px(step * scale)).clamp(0., max);
                            source.seek(dest, scale, w, cx);
                            cx.stop_propagation();
                        }
                    });
                    w.on_mouse_event(move |ev: &MouseUpEvent, phase, _, cx| {
                        if phase.capture()
                            && ev.button == MouseButton::Left
                            && state.read(cx).pressed.is_some()
                        {
                            state.update(cx, |s, cx| {
                                s.dragging = None;
                                s.press(None);
                                cx.notify();
                            });
                            cx.stop_propagation();
                        }
                    });
                },
            )
            .absolute()
            .inset_0(),
        )
    }
}
