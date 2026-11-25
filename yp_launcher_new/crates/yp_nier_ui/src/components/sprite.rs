use super::element;
use crate::{PANEL, Rect, Surface, Theme, Tint, WHITE};
use gpui::{App, IntoElement, RenderOnce, SharedString, Window};

#[derive(IntoElement)]
pub struct Sprite {
    name: SharedString,
    size: [f32; 2],
    tint: Tint,
}
impl Sprite {
    pub fn new(name: impl Into<SharedString>) -> Self {
        let name = name.into();
        let size = Theme::bundled()
            .sprite_size(&name)
            .unwrap_or_default()
            .map(|x| x as f32 * 0.5);
        Self {
            name,
            size,
            tint: WHITE,
        }
    }
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.size = [w, h];
        self
    }
    pub fn tint(mut self, tint: Tint) -> Self {
        self.tint = tint;
        self
    }
}
impl RenderOnce for Sprite {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let [w, h] = self.size;
        let mut s = Surface::new(w, h);
        s.sprite(&self.name, Rect::new(0., 0., w, h), self.tint);
        element(s)
    }
}

#[derive(IntoElement)]
pub struct NineSlice {
    pub sprite: SharedString,
    pub size: [f32; 2],
    pub source_border: [f32; 2],
    pub border: [f32; 2],
    pub tint: Tint,
}
impl NineSlice {
    pub fn panel(w: f32, h: f32) -> Self {
        Self {
            sprite: "square".into(),
            size: [w, h],
            source_border: [32., 32.],
            border: [16., 16.],
            tint: PANEL,
        }
    }
}
impl RenderOnce for NineSlice {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let [w, h] = self.size;
        let mut s = Surface::new(w, h);
        s.nine_slice(
            &self.sprite,
            Rect::new(0., 0., w, h),
            self.source_border,
            self.border,
            self.tint,
        );
        element(s)
    }
}
