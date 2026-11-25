use crate::static_noise::HackingStatic;
use crate::{GridNoise, Lens, Rect, Surface};
use gpui::{
    AnyElement, App, Bounds, Element, ElementId, Entity, Global, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, Pixels, Window,
};
use std::panic::Location;

struct Capture {
    surface: Surface,
    bounds: Rect,
}
#[derive(Default)]
struct Captures(Vec<Capture>);
impl Global for Captures {}

pub(crate) fn collect(child: &Surface, src: Rect, dst: Rect, clip: Rect, cx: &mut App) -> bool {
    if !cx.has_global::<Captures>() {
        return false;
    }
    let Some(current) = cx.global_mut::<Captures>().0.last_mut() else {
        return false;
    };
    let parent = Rect::new(0., 0., current.surface.width, current.surface.height);
    current.surface.append(
        child,
        src,
        dst.mapped(current.bounds, parent),
        clip.mapped(current.bounds, parent),
    );
    true
}

pub struct EffectLayer {
    child: AnyElement,
    lens: Option<Lens>,
    noise: Option<GridNoise>,
    size: [f32; 2],
    static_id: Option<(ElementId, u32)>,
    static_state: Option<Entity<HackingStatic>>,
}
impl EffectLayer {
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            lens: None,
            noise: None,
            size: [1600., 900.],
            static_id: None,
            static_state: None,
        }
    }
    pub fn lens(mut self, lens: Lens) -> Self {
        self.lens = Some(lens);
        self
    }
    pub fn grid_noise(mut self, noise: GridNoise) -> Self {
        self.noise = Some(noise);
        self
    }
    pub fn coordinates(mut self, width: f32, height: f32) -> Self {
        self.size = [width, height];
        self
    }
    pub fn hacking_static(mut self, id: impl Into<ElementId>, seed: u32) -> Self {
        self.static_id = Some((id.into(), seed));
        self
    }
}
impl Element for EffectLayer {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        w: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        if let Some((id, seed)) = self.static_id.clone() {
            self.static_state = Some(w.use_keyed_state(id, cx, |_, _| HackingStatic::new(seed)));
            w.request_animation_frame();
        }
        (self.child.request_layout(w, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        w: &mut Window,
        cx: &mut App,
    ) {
        self.child.prepaint(w, cx);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        w: &mut Window,
        cx: &mut App,
    ) {
        if (self.lens.is_none() && self.noise.is_none() && self.static_state.is_none())
            || bounds.is_empty()
        {
            self.child.paint(w, cx);
            return;
        }
        if !cx.has_global::<Captures>() {
            cx.set_global(Captures::default());
        }
        let mut surface = Surface::new(self.size[0], self.size[1]);
        if let Some(state) = &self.static_state {
            surface.static_blocks = Some(state.update(cx, |state, _| state.advance()));
        }
        if let Some(lens) = self.lens {
            surface.lens(lens);
        }
        if let Some(noise) = self.noise {
            surface.grid_noise(noise);
        }
        cx.global_mut::<Captures>().0.push(Capture {
            surface,
            bounds: Rect::from_bounds(bounds),
        });
        self.child.paint(w, cx);
        if let Some(capture) = cx.global_mut::<Captures>().0.pop() {
            capture.surface.paint(bounds, w, cx);
        }
    }
}
