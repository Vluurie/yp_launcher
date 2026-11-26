use super::{GameText, LoadingIcon, MenuRow};
use crate::{INK, LIGHT, PANEL, Rect, Surface, Theme};
use gpui::{
    App, ElementId, ExternalPaths, IntoElement, RenderOnce, SharedString, Window, canvas, div,
    prelude::*, px,
};
use std::{cell::Cell, path::PathBuf, rc::Rc};

pub fn file_drop_highlight(
    accent: gpui::Hsla,
    foreground: gpui::Hsla,
    label: impl Into<SharedString>,
    accepts: impl Fn(&[PathBuf]) -> bool + 'static,
) -> impl IntoElement {
    drop_highlight(accent, foreground, label.into(), true, accepts)
}

pub fn workspace_drop_highlight(
    accent: gpui::Hsla,
    foreground: gpui::Hsla,
    label: impl Into<SharedString>,
) -> impl IntoElement {
    drop_highlight(accent, foreground, label.into(), false, |_| true)
}

fn drop_highlight(
    accent: gpui::Hsla,
    foreground: gpui::Hsla,
    label: SharedString,
    outlined: bool,
    accepts: impl Fn(&[PathBuf]) -> bool + 'static,
) -> impl IntoElement {
    let color = gpui::Rgba::from(accent);
    let tint = [color.r, color.g, color.b].map(|channel| (channel * 255.).round() as u8);
    let hint_bounds = Rc::new(Cell::new(None::<gpui::Bounds<gpui::Pixels>>));
    let measured_hint = hint_bounds.clone();
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .invisible()
        .when(outlined, |target| target.border_4().border_color(accent))
        .drag_over::<ExternalPaths>(move |style, paths, _, _| {
            if paths.paths().is_empty() || !accepts(paths.paths()) {
                return style;
            }
            style.visible().bg(accent.opacity(0.19))
        })
        .child(
            canvas(|_, _, _| (), move |bounds, _, window, cx| {
                if crate::ReducedMotion::enabled(cx) || bounds.is_empty() {
                    return;
                }
                let state = window.use_keyed_state("drop-hacking-static", cx, |_, _| {
                    crate::static_noise::HackingStatic::new(1)
                });
                let width = f32::from(bounds.size.width);
                let height = f32::from(bounds.size.height);
                let mut surface = Surface::new(width, height);
                if outlined {
                    for x in [width * 0.05, width * 0.95 - 2.] {
                        surface.fill(Rect::new(x, 4., 2., (height - 8.).max(0.)), tint, 170);
                    }
                } else if let Some(hint) = hint_bounds.get() {
                    let origin = hint.origin - bounds.origin;
                    let plate = Rect::new(
                        origin.x.into(), origin.y.into(),
                        hint.size.width.into(), hint.size.height.into(),
                    );
                    surface.fill(plate, tint, 255);
                }
                surface.static_blocks = Some(state.update(cx, |effect, _| effect.advance()));
                surface.paint(bounds, window, cx);
                window.request_animation_frame();
            })
            .absolute()
            .inset_0()
            .size_full()
        )
        .when(outlined, |target| {
            target.child(div()
                .absolute()
                .inset_0()
                .border_1()
                .border_color(foreground))
        })
        .child(
            div()
                .relative()
                .max_w_full()
                .min_w_0()
                .overflow_hidden()
                .px(px(12.))
                .py(px(4.))
                .bg(accent)
                .text_color(foreground)
                .text_center()
                .when(!outlined, |hint| hint.child(
                    canvas(move |bounds, _, _| measured_hint.set(Some(bounds)), |_, _, _, _| {})
                        .absolute().inset_0()
                ))
                .child(label),
        )
}

type Action = Rc<dyn Fn(&mut Window, &mut App)>;
type DropFiles = Rc<dyn Fn(&[PathBuf], &mut Window, &mut App)>;
#[derive(IntoElement)]
pub struct MenuDropZone {
    id: ElementId,
    label: SharedString,
    scale: f32,
    files: Option<Action>,
    folder: Option<Action>,
    enabled: bool,
    drop: DropFiles,
}
impl MenuDropZone {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        drop: impl Fn(&[PathBuf], &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            scale: 1.,
            files: None,
            folder: None,
            enabled: true,
            drop: Rc::new(drop),
        }
    }
    pub fn scale(mut self, s: f32) -> Self {
        self.scale = s.max(0.);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn on_browse_files(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.files = Some(Rc::new(f));
        self
    }
    pub fn on_browse_folder(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.folder = Some(Rc::new(f));
        self
    }
}
impl RenderOnce for MenuDropZone {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let s = self.scale;
        let enabled = self.enabled;
        let measured = window.use_keyed_state(self.id.clone(), cx, |_, _| None::<gpui::Pixels>);
        let available = *measured.read(cx);
        let row_width = Theme::bundled().measure("Browse folder", 36, 0.8)[0]
            .max(Theme::bundled().measure("Browse files", 36, 0.8)[0])
            .ceil()
            + 57.;
        let button_scale = available.map_or(s, |width| s.min(f32::from(width) / (row_width + 72.)));
        let tracker = canvas(
            move |bounds, w, cx| {
                let width = bounds.size.width;
                if measured.read(cx) != &Some(width) {
                    let measured = measured.clone();
                    w.defer(cx, move |_, cx| {
                        measured.update(cx, |previous, cx| {
                            if previous.replace(width) != Some(width) {
                                cx.notify();
                            }
                        })
                    });
                }
            },
            |_, _, _, _| (),
        )
        .absolute()
        .inset_0();
        let drop_label = self.label.clone();
        let buttons = [
            ("files", "Browse files", self.files),
            ("folder", "Browse folder", self.folder),
        ]
        .into_iter()
        .filter_map(|(id, label, f)| {
            f.map(|f| {
                div()
                    .debug_selector(move || format!("browse-{id}"))
                    .flex_none()
                    .child(
                        MenuRow::new(id, label)
                            .width(row_width)
                            .enabled(enabled)
                            .scale(button_scale)
                            .on_click(move |_, w, cx| f(w, cx)),
                    )
            })
        });
        div()
            .id(self.id)
            .relative()
            .w_full()
            .min_w_0()
            .overflow_hidden()
            .flex()
            .flex_col()
            .p(px(12. * s))
            .border_1()
            .border_color(gpui::rgba(if enabled { 0x777369ff } else { 0x77736980 }))
            .when(enabled, |el| {
                el.on_drop(move |paths: &ExternalPaths, w, cx| {
                    (self.drop)(paths.paths(), w, cx);
                    cx.stop_propagation();
                })
            })
            .child(
                div().when(!enabled, |el| el.opacity(0.5)).child(
                    GameText::new(self.label)
                        .scale(0.8 * s)
                        .when_some(available, |label, width| label.wrap(f32::from(width))),
                ),
            )
            .child(
                div()
                    .debug_selector(|| "browse-actions".to_owned())
                    .relative()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_wrap()
                    .gap_x(px(8. * s))
                    .overflow_hidden()
                    .child(tracker)
                    .children(buttons),
            )
            .when(enabled, |zone| {
                zone.child(file_drop_highlight(
                    gpui::rgb(0x4b4940).into(),
                    gpui::rgb(0xd9d4bb).into(),
                    drop_label,
                    |_| true,
                ))
            })
    }
}
#[derive(IntoElement)]
pub struct MenuProgress {
    id: ElementId,
    label: SharedString,
    value: Option<f32>,
    scale: f32,
}
impl MenuProgress {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: None,
            scale: 1.,
        }
    }
    pub fn fraction(mut self, value: f32) -> Self {
        self.value = Some(value.clamp(0., 1.));
        self
    }
    pub fn scale(mut self, s: f32) -> Self {
        self.scale = s.max(0.);
        self
    }
}
impl RenderOnce for MenuProgress {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let scale = self.scale;
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12. * scale))
                    .child(LoadingIcon::new("activity").scale(scale))
                    .child(GameText::new(self.label).scale(0.8 * scale)),
            )
            .when_some(self.value, |el, frac| {
                el.child(
                    canvas(
                        |_, _, _| (),
                        move |b, _, w, cx| {
                            let mut s = Surface::new(400., 10.);
                            s.fill(Rect::new(0., 0., 400., 10.), PANEL, 255);
                            s.fill(Rect::new(0., 0., 400. * frac, 10.), INK, 255);
                            if frac > 0. && frac < 1. {
                                s.fill(Rect::new(400. * frac - 1., 0., 1., 10.), LIGHT, 255);
                            }
                            s.paint(b, w, cx)
                        },
                    )
                    .w_full()
                    .h(px(10. * scale)),
                )
            })
    }
}
