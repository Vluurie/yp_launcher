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
    let header = fit_name(title, 0.8, 906.);
    paper.text(&header, 36, 0.8, 35., 20., INK);
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
                .role(gpui::Role::Label)
                .aria_label(description.replace('\n', ", "))
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

fn fit_name(label: &str, scale: f32, available: f32) -> std::borrow::Cow<'_, str> {
    let theme = Theme::bundled();
    let line = label.lines().next().unwrap_or_default();
    if line == label && theme.measure(line, 36, scale)[0] <= available {
        return line.into();
    }
    let room = available - theme.measure("…", 36, scale)[0] - 4.;
    let end = theme
        .caret_positions(line, 36, scale)
        .into_iter()
        .take_while(|(_, x)| *x <= room)
        .last()
        .map_or(0, |(index, _)| index);
    format!("{}…", &line[..end]).into()
}

pub(super) fn title(label: &str, x: f32, alpha: f32, s: f32) -> impl IntoElement {
    let fitted = fit_name(label, 1., 1420.);
    let label = fitted.as_ref();
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
    playback: bool,
    equipment: bool,
    prompts: &[gpui::SharedString; 4],
    emit: Action,
) -> impl IntoElement {
    let mut actions = vec![("rotate", prompts[0].as_ref(), None)];
    if playback {
        actions.push((
            "confirm",
            prompts[1].as_ref(),
            Some(ViewerAction::Animation),
        ));
    }
    if equipment {
        actions.push(("equipment", prompts[2].as_ref(), Some(ViewerAction::Equipment)));
    }
    actions.push(("back", prompts[3].as_ref(), Some(ViewerAction::Back)));
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
            regions.push((
                action,
                label.to_owned(),
                Rect::new(right, 55., end - right, 56.),
            ));
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
                .map(move |(i, (action, label, r))| {
                    let emit = emit.clone();
                    let activate = emit.clone();
                    place(
                        r,
                        s,
                        div()
                            .id(("viewer-prompt", i))
                            .role(gpui::Role::Button)
                            .aria_label(label)
                            .tab_index(0)
                            .focus(|s| s.border_2().border_color(gpui::rgb(0x48463d)))
                            .size_full()
                            .occlude()
                            .cursor_pointer()
                            .on_key_down(move |event: &gpui::KeyDownEvent, w, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    activate(action, w, cx);
                                    cx.stop_propagation();
                                }
                            })
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
