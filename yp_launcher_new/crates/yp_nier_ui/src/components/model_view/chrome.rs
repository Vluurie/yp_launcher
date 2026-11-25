use super::ViewerAction;
use crate::{GameText, INK, LIGHT, PANEL, Rect, Surface, Theme};
use gpui::{App, IntoElement, Window, div, prelude::*, px};
use std::rc::Rc;

pub(super) fn place(rect: Rect, s: f32, child: impl IntoElement) -> impl IntoElement {
    div()
        .absolute()
        .left(px(rect.x * s))
        .top(px(rect.y * s))
        .w(px(rect.w * s))
        .h(px(rect.h * s))
        .child(child)
}

pub(super) fn details(title: &str, description: &str, s: f32) -> impl IntoElement {
    let mut paper = Surface::new(956., 480.);
    paper.fill(Rect::new(0., 0., 956., 40.), PANEL, 255);
    paper.fill(Rect::new(0., 40., 956., 440.), LIGHT, 255);
    paper.sprite("panel", Rect::new(6., 6., 28., 28.), INK);
    paper.text(title, 36, 0.8, 35., 20., INK);
    for y in [45., 473.] {
        paper.fill(Rect::new(10., y, 914., 2.), PANEL, 255);
    }
    let lines = Theme::bundled().wrapped(description, 36, 0.75, 475.);
    div()
        .absolute()
        .left(px(595. * s))
        .top(px(230. * s))
        .w(px(956. * s))
        .h(px(480. * s))
        .child(div().absolute().inset_0().child(paper))
        .child(
            div()
                .id("unit-description")
                .absolute()
                .left(px(449. * s))
                .top(px(53. * s))
                .w(px(478. * s))
                .h(px(410. * s))
                .overflow_y_scroll()
                .children(lines.split('\n').map(move |line| {
                    div()
                        .h(px(35. * s))
                        .child(GameText::new(line.to_owned()).reveal(false).scale(0.75 * s))
                })),
        )
}

pub(super) fn title(label: &str, x: f32, alpha: f32, s: f32) -> impl IntoElement {
    let text_width = Theme::bundled().measure(label, 36, 1.)[0];
    let width = 102. + text_width;
    let mut title = Surface::new(width, 82.);
    title.fill(Rect::new(10., 15., width - 24., 49.), INK, 248);
    title.fill(Rect::new(16., 21., 3., 37.), PANEL, 255);
    title.text(label, 36, 1., 32., 40., LIGHT);
    div()
        .absolute()
        .left(px((x - 10.) * s))
        .top(px(90. * s))
        .w(px(width * s))
        .h(px(82. * s))
        .opacity(alpha)
        .child(title)
}

pub(super) fn footer(
    offset: f32,
    alpha: f32,
    s: f32,
    motions: Option<(usize, usize)>,
    equipment: bool,
    emit: Action,
) -> impl IntoElement {
    let mut actions = vec![("rotate", "Rotate", None)];
    if motions.is_some() {
        actions.push(("confirm", "Animation", Some(ViewerAction::Animation)));
    }
    if equipment {
        actions.push(("tab", "Change Equipment", Some(ViewerAction::Equipment)));
    }
    actions.push(("back", "Back", Some(ViewerAction::Back)));
    let mut art = Surface::new(1600., 150.);
    let mut right = 1519.;
    let mut regions = Vec::new();
    let mut ink = Surface::new(1600., 150.);
    for (key, label, action) in actions.into_iter().rev() {
        let end = right;
        let tw = Theme::bundled().measure(label, 36, 0.75)[0];
        right -= tw;
        ink.text(label, 36, 0.75, right, 84., INK);
        let [w, h] = Theme::bundled().sprite_size(key).expect("viewer key");
        let (w, h) = (w as f32 * 0.9, h as f32 * 0.9);
        right -= w + 5.;
        ink.sprite(key, Rect::new(right, 84. - h / 2., w, h), INK);
        if let Some(action) = action {
            regions.push((action, Rect::new(right, 55., end - right, 56.)));
        }
        right -= 31.;
    }
    let left = right - 12.;
    art.fill(Rect::new(left, 50., 1548. - left, 67.), LIGHT, 255);
    art.fill(Rect::new(left + 6., 50., 2., 67.), INK, 255);
    if let Some((current, total)) = motions {
        let index = if current == 0 {
            "-".to_owned()
        } else {
            current.to_string()
        };
        let caption = format!("Motion Number : {index}/{total}");
        let w = Theme::bundled().measure(&caption, 36, 0.8)[0];
        art.text(&caption, 36, 0.8, 1548. - w, 29., LIGHT);
    }
    div()
        .absolute()
        .left(px(offset * s))
        .top(px(726. * s))
        .w(px(1600. * s))
        .h(px(150. * s))
        .opacity(alpha)
        .child(div().absolute().inset_0().child(art))
        .child(div().absolute().inset_0().child(ink))
        .children(
            regions
                .into_iter()
                .enumerate()
                .map(move |(i, (action, r))| {
                    let emit = emit.clone();
                    place(
                        r,
                        s,
                        div()
                            .id(("viewer-prompt", i))
                            .size_full()
                            .occlude()
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation()
                            })
                            .on_click(move |_, w, cx| {
                                emit(action, w, cx);
                                cx.stop_propagation();
                            }),
                    )
                }),
        )
}

pub(super) type Action = Rc<dyn Fn(ViewerAction, &mut Window, &mut App)>;
