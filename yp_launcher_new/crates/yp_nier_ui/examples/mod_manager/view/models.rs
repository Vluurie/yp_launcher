use super::{ModManager, catalog};
use gpui::{
    App, Bounds, Entity, IntoElement, PathBuilder, Pixels, RenderOnce, Window, canvas, point,
    prelude::*, px, rgb,
};
use std::time::Instant;
use yp_nier_ui::{Focus, ModelViewer, ViewerAction};

pub(super) struct ModelState {
    pub expanded: bool,
    yaw: f32,
    pitch: f32,
    finish: bool,
    motion: usize,
    started: Instant,
}

impl Default for ModelState {
    fn default() -> Self {
        Self {
            expanded: false,
            yaw: -0.55,
            pitch: 0.62,
            finish: false,
            motion: 0,
            started: Instant::now(),
        }
    }
}

impl ModManager {
    pub(super) fn model_view(&self, owner: Entity<Self>, scale: f32) -> ModelViewer {
        let i = self.menu.row_index();
        let (name, description) = catalog::MODELS[i];
        let state = &self.model;
        let scene = Chip {
            memory: i == 1,
            yaw: state.yaw,
            pitch: state.pitch,
            finish: state.finish,
            motion: state.motion,
            started: state.started,
        };
        ModelViewer::new("unit-model", name, scene, move |action, _, cx| {
            owner.update(cx, |v, cx| v.viewer_action(action, cx));
        })
        .description(description)
        .expanded(state.expanded)
        .motions(state.motion, 2)
        .equipment(true)
        .scale(scale)
    }

    pub(super) fn viewer_action(&mut self, action: ViewerAction, cx: &mut gpui::Context<Self>) {
        match action {
            ViewerAction::Open => {
                if self.menu.focus() == Focus::Tabs {
                    self.menu.select(self.menu.row_index());
                }
                self.model.expanded = true;
            }
            ViewerAction::Back => self.model.expanded = false,
            ViewerAction::Rotate { dx, dy } => {
                self.model.yaw += dx * 0.012;
                self.model.pitch = (self.model.pitch + dy * 0.012).clamp(-1.45, 1.45);
            }
            ViewerAction::Animation => {
                self.model.motion = self.model.motion % 2 + 1;
                self.model.started = Instant::now();
            }
            ViewerAction::Equipment => self.model.finish = !self.model.finish,
            ViewerAction::Previous | ViewerAction::Next => {
                let count = catalog::MODELS.len();
                let delta = if action == ViewerAction::Next {
                    1
                } else {
                    count - 1
                };
                self.menu.select((self.menu.row_index() + delta) % count);
                self.model.motion = 0;
            }
        }
        cx.notify();
    }

    pub fn viewer_state(&self) -> (bool, usize, bool, usize, f32) {
        (
            self.model.expanded,
            self.model.motion,
            self.model.finish,
            self.menu.row_index(),
            self.model.yaw,
        )
    }
}

#[derive(IntoElement)]
struct Chip {
    memory: bool,
    yaw: f32,
    pitch: f32,
    finish: bool,
    motion: usize,
    started: Instant,
}

struct Face {
    depth: f32,
    points: [gpui::Point<Pixels>; 4],
    color: u32,
}

struct Projection {
    bounds: Bounds<Pixels>,
    yaw: f32,
    pitch: f32,
    bob: f32,
}

impl Projection {
    fn project(&self, [x, y, z]: [f32; 3]) -> (gpui::Point<Pixels>, f32) {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let (x, z) = (cy * x - sy * z, sy * x + cy * z);
        let (y, z) = (cp * y + sp * z, -sp * y + cp * z);
        let scale = f32::from(self.bounds.size.width.min(self.bounds.size.height)) * 0.29
            / (1. - 0.065 * z);
        (
            point(
                self.bounds.center().x + px(x * scale),
                self.bounds.center().y + px((y + self.bob) * scale),
            ),
            z,
        )
    }

    fn block(&self, center: [f32; 3], half: [f32; 3], tint: [u8; 3], faces: &mut Vec<Face>) {
        let points: [_; 8] = std::array::from_fn(|i| {
            self.project(std::array::from_fn(|axis| {
                center[axis]
                    + if i & (1 << axis) == 0 {
                        -half[axis]
                    } else {
                        half[axis]
                    }
            }))
        });
        for (ids, shade) in [
            ([0, 1, 3, 2], 0.70),
            ([4, 6, 7, 5], 0.87),
            ([0, 2, 6, 4], 0.62),
            ([1, 5, 7, 3], 0.78),
            ([0, 4, 5, 1], 1.),
            ([2, 3, 7, 6], 0.53),
        ] {
            let vertices = ids.map(|i| points[i]);
            let [r, g, b] = tint.map(|v| (f32::from(v) * shade) as u8);
            faces.push(Face {
                depth: vertices.iter().map(|p| p.1).sum::<f32>() / 4.,
                points: vertices.map(|v| v.0),
                color: u32::from_be_bytes([0, r, g, b]),
            });
        }
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        if self.motion != 0 {
            window.request_animation_frame();
        }
        canvas(
            |_, _, _| (),
            move |bounds, _, w, _| {
                let t = self.started.elapsed().as_secs_f32();
                let projection = Projection {
                    bounds,
                    yaw: self.yaw + if self.motion == 1 { t * 0.7 } else { 0. },
                    pitch: self.pitch,
                    bob: if self.motion == 2 {
                        (t * 2.5).sin() * 0.16
                    } else {
                        0.
                    },
                };
                let mut faces = Vec::new();
                let shell = if self.finish {
                    [139, 114, 150]
                } else {
                    [111, 136, 129]
                };
                let width = if self.memory { 1.28 } else { 0.82 };
                let depth = if self.memory { 0.48 } else { 0.78 };
                projection.block([0.; 3], [width, 0.065, depth], shell, &mut faces);
                let chips = if self.memory { 3 } else { 1 };
                for i in 0..chips {
                    let x = (i as f32 - (chips - 1) as f32 * 0.5) * 0.72;
                    projection.block(
                        [x, -0.16, 0.],
                        [if self.memory { 0.29 } else { 0.53 }, 0.13, depth * 0.64],
                        [72, 70, 63],
                        &mut faces,
                    );
                    projection.block(
                        [x, -0.295, 0.],
                        [if self.memory { 0.18 } else { 0.39 }, 0.02, depth * 0.43],
                        [196, 190, 165],
                        &mut faces,
                    );
                }
                for i in 0..12 {
                    let x = width * (2. * (i as f32 + 0.5) / 12. - 1.);
                    for sign in [-1., 1.] {
                        projection.block(
                            [x, 0.01, sign * depth],
                            [0.034, 0.045, 0.12],
                            [193, 164, 103],
                            &mut faces,
                        );
                    }
                }
                faces.sort_by(|a, b| a.depth.total_cmp(&b.depth));
                for face in faces {
                    let mut path = PathBuilder::fill();
                    path.move_to(face.points[0]);
                    for p in &face.points[1..] {
                        path.line_to(*p);
                    }
                    path.close();
                    if let Ok(path) = path.build() {
                        w.paint_path(path, rgb(face.color));
                    }
                }
            },
        )
        .size_full()
    }
}
