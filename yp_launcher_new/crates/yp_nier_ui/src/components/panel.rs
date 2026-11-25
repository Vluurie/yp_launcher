use crate::{GameText, INK, LIGHT, Rect, Surface};
use gpui::{
    AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Window, div, prelude::*,
    px,
};

#[derive(IntoElement)]
pub struct MenuPanel {
    title: SharedString,
    scale: f32,
    children: Vec<AnyElement>,
}
impl MenuPanel {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            scale: 1.,
            children: Vec::new(),
        }
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}
impl ParentElement for MenuPanel {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(children);
    }
}
impl RenderOnce for MenuPanel {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let scale = self.scale;
        let paper = gpui::canvas(
            |_, _, _| (),
            move |b, _, w, cx| {
                let width = b.size.width / px(scale);
                let height = b.size.height / px(scale);
                let mut s = Surface::new(width, height);
                s.nine_slice(
                    "panel",
                    Rect::new(-5., -5., width + 10., height + 10.),
                    [20., 20.],
                    [10., 10.],
                    LIGHT,
                );
                s.fill(Rect::new(0., 0., width, 48.), INK, 255);
                s.sprite("panel", Rect::new(12., 10., 28., 28.), LIGHT);
                s.paint(b, w, cx);
            },
        )
        .absolute()
        .inset_0()
        .size_full();
        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .child(paper)
            .child(
                div()
                    .absolute()
                    .left(px(44. * scale))
                    .top(px(6. * scale))
                    .child(
                        GameText::new(self.title)
                            .font_id(1)
                            .scale(0.8 * scale)
                            .tint(LIGHT),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .top(px(52. * scale))
                    .p(px(14. * scale))
                    .flex()
                    .flex_col()
                    .children(self.children),
            )
    }
}
