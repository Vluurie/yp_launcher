use super::{
    LayerKind, SpriteKey, UidDocument, UidPlayer, UidResources,
    geometry::{Transform, anchored},
    graphics::GraphicPainter,
    raster::{Ink, Mask, Painter},
};
use crate::{GridNoise, Lens, Rect, Surface};
use gpui::{App, ElementId, IntoElement, RenderOnce, Window, canvas, prelude::*};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc, time::Instant};

pub type SharedPlayer = Rc<RefCell<UidPlayer>>;

#[derive(Default, Clone)]
struct Bindings {
    labels: HashMap<u32, (u16, String)>,
    sprites: HashMap<u32, SpriteKey>,
    visible: HashMap<u32, bool>,
    width: f32,
    offsets: HashMap<u32, [f32; 2]>,
    sizes: HashMap<u32, [f32; 2]>,
    fractions: HashMap<u32, [f32; 2]>,
}

#[derive(IntoElement)]
pub struct UidElement {
    id: ElementId,
    document: Arc<UidDocument>,
    resources: Rc<UidResources>,
    player: Option<SharedPlayer>,
    timeline: Option<f32>,
    since: Option<Instant>,
    clip: Option<u32>,
    viewport: [f32; 2],
    origin: Option<[f32; 2]>,
    bindings: Bindings,
}

impl UidElement {
    pub fn new(id: impl Into<ElementId>, document: Arc<UidDocument>) -> Self {
        thread_local! {static RES:Rc<UidResources>=Rc::new(UidResources::new());}
        Self {
            id: id.into(),
            document,
            resources: RES.with(Clone::clone),
            player: None,
            timeline: None,
            since: None,
            clip: None,
            viewport: [1600., 900.],
            origin: None,
            bindings: Bindings::default(),
        }
    }
    pub fn resources(mut self, resources: Rc<UidResources>) -> Self {
        self.resources = resources;
        self
    }
    pub fn player(mut self, player: SharedPlayer) -> Self {
        self.document = player.borrow().document.clone();
        self.player = Some(player);
        self
    }
    pub fn animation(mut self, id: u32) -> Self {
        self.clip = Some(id);
        self
    }
    pub fn at_time(mut self, seconds: f32) -> Self {
        self.timeline = Some(seconds.max(0.));
        self
    }
    pub fn viewport(mut self, width: f32, height: f32) -> Self {
        self.viewport = [width.max(1.), height.max(1.)];
        self
    }
    pub fn origin(mut self, x: f32, y: f32) -> Self {
        self.origin = Some([x, y]);
        self
    }
    pub fn text(mut self, part: u32, font: u16, text: impl Into<String>) -> Self {
        self.bindings.labels.insert(part, (font, text.into()));
        self
    }
    pub fn sprite(mut self, part: u32, key: SpriteKey) -> Self {
        self.bindings.sprites.insert(part, key);
        self
    }
    pub fn visible(mut self, part: u32, visible: bool) -> Self {
        self.bindings.visible.insert(part, visible);
        self
    }
    pub fn offset(mut self, part: u32, x: f32, y: f32) -> Self {
        self.bindings.offsets.insert(part, [x, y]);
        self
    }
    pub fn image_size(mut self, part: u32, w: f32, h: f32) -> Self {
        self.bindings.sizes.insert(part, [w.max(0.), h.max(0.)]);
        self
    }
    pub fn image_fraction(mut self, part: u32, x: f32, y: f32) -> Self {
        self.bindings
            .fractions
            .insert(part, [x.clamp(0., 1.), y.clamp(0., 1.)]);
        self
    }
    pub fn extra_width(mut self, width: f32) -> Self {
        self.bindings.width = width;
        self
    }
    pub fn since(mut self, start: Instant) -> Self {
        self.since = Some(start);
        self
    }
}

#[derive(Clone)]
struct Node {
    transform: Transform,
    color: [f32; 4],
    gradient: [f32; 4],
    visible: bool,
    masks: Vec<Mask>,
}
impl Node {
    fn root(transform: Transform) -> Self {
        Self {
            transform,
            color: [1.; 4],
            gradient: [1.; 4],
            visible: true,
            masks: Vec::new(),
        }
    }
}

struct Scene<'a, 'b> {
    painter: Painter<'a>,
    resources: &'b UidResources,
    seconds: f32,
    nested: &'b mut HashMap<Vec<usize>, UidPlayer>,
}

impl Scene<'_, '_> {
    fn document(
        &mut self,
        doc: &UidDocument,
        values: &[[f32; 16]],
        root: Node,
        bindings: &Bindings,
        path: &mut Vec<usize>,
    ) {
        let mut nodes: Vec<Node> = Vec::with_capacity(doc.layers.len());
        for (i, (layer, &channels)) in doc.layers.iter().zip(values).enumerate() {
            let parent = layer.parent.and_then(|p| nodes.get(p)).unwrap_or(&root);
            let visible = parent.visible
                && bindings
                    .visible
                    .get(&layer.id)
                    .copied()
                    .unwrap_or(layer.visible);
            let mut local = channels;
            if let Some([x, y]) = bindings.offsets.get(&layer.id) {
                local[0] += x;
                local[1] += y;
            }
            let transform = parent.transform.child(local, layer.ignore_parent);
            let color = std::array::from_fn(|i| {
                channels[8 + i]
                    * if layer.ignore_parent[3] {
                        1.
                    } else {
                        parent.color[i]
                    }
            });
            let gradient = std::array::from_fn(|i| {
                channels[12 + i]
                    * if layer.ignore_parent[3] {
                        1.
                    } else {
                        parent.gradient[i]
                    }
            });
            let mut node = Node {
                transform,
                color,
                gradient,
                visible,
                masks: parent.masks.clone(),
            };
            if visible && color[3] > 0. {
                let ink = Ink {
                    color,
                    gradient,
                    mode: layer.gradient,
                };
                let mut graphic = GraphicPainter {
                    painter: &mut self.painter,
                    resources: self.resources,
                    transform,
                    masks: &node.masks,
                    ink: &ink,
                };
                let extension = layer.extension.as_ref();
                let mut handled = false;
                if let Some(ext) = extension {
                    let u = |i: usize| ext.words.get(i).copied().unwrap_or_default();
                    let f = |i: usize| f32::from_bits(u(i));
                    match ext.id {
                        1020 => {
                            let v = |i: usize| u(i) as i32 as f32;
                            let dist = if u(0) == 0 {
                                [v(1), v(2), v(3)]
                            } else {
                                std::array::from_fn(|i| v(i + 1) + (gradient[i] * 255. - 100.))
                            };
                            graphic.painter.surface.lens(Lens {
                                distortion: dist,
                                stretch: f(6),
                                dark_edges: u(4) != 0,
                                shift: [[v(7), v(8)], [v(9), v(10)], [v(11), v(12)]],
                                monochrome: f(13),
                            });
                            handled = true;
                        }
                        1000 => {
                            graphic.painter.surface.blur_strength =
                                if u(1) != 0 { color[3] * f(6) } else { f(6) };
                            handled = true;
                        }
                        1030 => {
                            let blur_strength = graphic.painter.surface.blur_strength;
                            graphic.painter.surface.grid_noise(GridNoise {
                                cell: [u(0) as f32, u(1) as f32],
                                levels: u(2).clamp(1, 255) as u8,
                                monochrome: f(6),
                                blur_strength,
                            });
                            handled = true;
                        }
                        1003 => {
                            let name = if u(0) == 0 {
                                "core_pt_00"
                            } else {
                                "core_pt_01"
                            };
                            let start = u(1) as f32;
                            let end = u(2) as f32;
                            let t = ((self.seconds / doc.frame_duration - start)
                                / (end - start).max(1.))
                            .clamp(0., 1.);
                            let t = if u(3) == 0 { t } else { 1. - t };
                            let rect = Rect::new(
                                0.,
                                0.,
                                graphic.painter.surface.width,
                                graphic.painter.surface.height,
                            );
                            if t >= 1. {
                                graphic.painter.surface.fill(
                                    rect,
                                    [color[0], color[1], color[2]]
                                        .map(|c| (c * 255.).round() as u8),
                                    (color[3] * 255.) as u8,
                                );
                            } else if t > 0.
                                && let Some(mask) = self
                                    .resources
                                    .sprite_key(name)
                                    .and_then(|k| self.resources.sprite(k))
                            {
                                let tex = super::raster::reveal(
                                    &mask,
                                    self.resources.id(),
                                    u(0),
                                    t,
                                    color,
                                    graphic.painter.cx,
                                );
                                graphic.painter.surface.raster(rect, tex, None);
                            }
                            handled = true;
                        }
                        _ => {}
                    }
                }
                if !handled {
                    match &layer.kind {
                        LayerKind::Locator { name, part }
                            if !name.is_empty() && path.len() < 24 =>
                        {
                            if let Some(child) = self.resources.document(name, *part).cloned() {
                                path.push(i);
                                let player = self
                                    .nested
                                    .entry(path.clone())
                                    .or_insert_with(|| UidPlayer::new(child.clone(), i as u32 + 1));
                                let values = player.sample(self.seconds, None);
                                self.document(
                                    &child,
                                    &values,
                                    node.clone(),
                                    &Bindings::default(),
                                    path,
                                );
                                path.pop();
                            }
                        }
                        LayerKind::Image(img) => {
                            let key = bindings
                                .sprites
                                .get(&layer.id)
                                .copied()
                                .unwrap_or(img.graphic.key);
                            graphic.image(
                                img,
                                key,
                                bindings.sizes.get(&layer.id).copied(),
                                bindings.fractions.get(&layer.id).copied(),
                            );
                        }
                        LayerKind::NineGrid(grid) => {
                            let key = bindings
                                .sprites
                                .get(&layer.id)
                                .copied()
                                .unwrap_or(grid.graphic.key);
                            graphic.nine_grid(grid, key, bindings.width);
                        }
                        LayerKind::Message(msg) => {
                            let override_text = bindings.labels.get(&layer.id);
                            let original = || {
                                msg.events.iter().find_map(|&id| {
                                    self.resources.message(&msg.bank, id, msg.paragraph)
                                })
                            };
                            if let Some((font, text)) = override_text
                                .map(|(f, s)| (*f, s.as_str()))
                                .or_else(original)
                            {
                                let anchor = [msg.anchor >> 4, msg.anchor & 15];
                                let tint = |c: [f32; 4]| std::array::from_fn(|i| c[i] * color[i]);
                                if msg.shadow_offset != [0, 0] && msg.shadow_color[3] > 0. {
                                    graphic.label(
                                        text,
                                        font,
                                        anchor,
                                        msg.letter_spacing,
                                        msg.shadow_offset.map(f32::from),
                                        tint(msg.shadow_color),
                                    );
                                }
                                graphic.label(
                                    text,
                                    font,
                                    anchor,
                                    msg.letter_spacing,
                                    [0.; 2],
                                    tint(msg.main_color),
                                );
                            }
                        }
                        LayerKind::String { font, anchor, .. } => {
                            if let Some((override_font, text)) = bindings.labels.get(&layer.id) {
                                graphic.text(
                                    text,
                                    if *override_font == 0 {
                                        *font as u16
                                    } else {
                                        *override_font
                                    },
                                    *anchor,
                                );
                            }
                        }
                        LayerKind::Text { anchor, scale, .. } => {
                            if let Some((font, text)) = bindings.labels.get(&layer.id) {
                                for (axis, factor) in graphic.transform.axes.iter_mut().zip(scale) {
                                    for v in axis {
                                        *v *= factor;
                                    }
                                }
                                graphic.text(text, *font, *anchor);
                            }
                        }
                        LayerKind::Mask { mode, anchor, size } if mode & 255 != 0 => {
                            node.masks.push(Mask {
                                transform,
                                rect: anchored(*size, *anchor),
                                inverse: false,
                            });
                        }
                        _ => {}
                    }
                }
            }
            nodes.push(node);
        }
    }
}

struct Runtime {
    clock: Instant,
    player: SharedPlayer,
    clip: Option<u32>,
    nested: HashMap<Vec<usize>, UidPlayer>,
}
impl RenderOnce for UidElement {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id, cx, |_, _| Runtime {
            clock: Instant::now(),
            clip: None,
            player: Rc::new(RefCell::new(UidPlayer::new(self.document.clone(), 1))),
            nested: HashMap::new(),
        });
        let seconds = self
            .since
            .unwrap_or(state.read(cx).clock)
            .elapsed()
            .as_secs_f32();
        let (player, active) = state.update(cx, |rt, _| {
            let player = self.player.clone().unwrap_or_else(|| {
                if !Arc::ptr_eq(&self.document, &rt.player.borrow().document) {
                    rt.player = Rc::new(RefCell::new(UidPlayer::new(self.document.clone(), 1)));
                    rt.clip = None;
                    rt.nested.clear();
                }
                rt.player.clone()
            });
            if self.clip != rt.clip {
                if let Some(id) = self.clip {
                    let _ = player.borrow_mut().play(id, seconds);
                }
                rt.clip = self.clip;
            }
            let moving = player.borrow().is_playing(seconds)
                || !rt.nested.is_empty()
                || self.document.layers.iter().any(|l| l.extension.is_some());
            (player, moving)
        });
        if self.timeline.is_none() && active {
            window.request_animation_frame();
        }
        let [width, height] = self.viewport;
        let origin = self.origin.unwrap_or([
            width * 0.5 * ((self.document.alignment >> 4) & 3) as f32,
            height * 0.5 * (self.document.alignment & 3) as f32,
        ]);
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let now = self.timeline.unwrap_or(seconds);
                let values = player.borrow_mut().sample(now, self.timeline);
                let density = (bounds.size.width / gpui::px(width))
                    .max(bounds.size.height / gpui::px(height))
                    * window.scale_factor();
                let mut surface = Surface::new(width, height);
                state.update(cx, |rt, cx| {
                    let mut scene = Scene {
                        painter: Painter {
                            surface: &mut surface,
                            cx,
                            density: density.max(0.01),
                        },
                        resources: self.resources.as_ref(),
                        seconds: now,
                        nested: &mut rt.nested,
                    };
                    scene.document(
                        &self.document,
                        &values,
                        Node::root(Transform::at(origin[0], origin[1])),
                        &self.bindings,
                        &mut Vec::new(),
                    );
                });
                surface.paint(bounds, window, cx);
            },
        )
        .size_full()
    }
}
