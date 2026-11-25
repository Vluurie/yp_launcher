use super::{InspectorPage, preset_label, viewport};
use gpui::{KeyDownEvent, ScrollStrategy};
use yp_nier_ui::{
    ButtonPrompt, GameText, HelpBar, LoadingIcon, MenuFrame, MenuItemRow, MenuList, ModelViewer,
    ViewerAction,
};
use yp_ui::prelude::*;

impl InspectorPage {
    pub(super) fn set_unit_view(
        &mut self,
        view: Option<bool>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.unit_view.is_some() != view.is_some() {
            let style = if view.is_some() {
                yp_inspector::RenderStyle {
                    background: [0; 4],
                    ..Default::default()
                }
            } else {
                Default::default()
            };
            self.model
                .update(cx, |model, cx| model.set_style(style, cx));
        }
        self.unit_view = view;
        if view != Some(true) {
            self.focus_handle.focus(w, cx);
        }
        cx.notify();
    }

    fn change_unit(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(candidate) = self.model.read(cx).candidates().get(index).cloned() {
            self.model
                .update(cx, |m, cx| m.select_candidate(candidate, cx));
            self.unit_scroll
                .scroll_to_item(index, ScrollStrategy::Nearest);
        }
    }

    fn unit_action(&mut self, action: ViewerAction, w: &mut Window, cx: &mut Context<Self>) {
        match action {
            ViewerAction::Open => self.set_unit_view(Some(true), w, cx),
            ViewerAction::Back => {
                self.set_unit_view(self.unit_view.filter(|&open| open).map(|_| false), w, cx)
            }
            ViewerAction::Rotate { dx, dy } => self.model.update(cx, |m, cx| m.orbit(dx, dy, cx)),
            ViewerAction::Equipment => self.model.update(cx, |m, cx| {
                if let Some(i) = m.selected_index()
                    && !m.presets().is_empty()
                {
                    m.select_preset((i + 1) % m.presets().len(), cx);
                }
            }),
            ViewerAction::Previous | ViewerAction::Next => {
                let model = self.model.read(cx);
                let candidates = model.candidates();
                if let Some(i) = candidates
                    .iter()
                    .position(|c| model.candidate().is_some_and(|v| v.id == c.id))
                {
                    let n = candidates.len();
                    self.change_unit(
                        (i + if action == ViewerAction::Next {
                            1
                        } else {
                            n - 1
                        }) % n,
                        cx,
                    );
                }
            }
            ViewerAction::Animation => {}
        }
    }

    fn unit_scene(&self, scale: f32, expanded: bool, cx: &mut Context<Self>) -> ModelViewer {
        let model = self.model.read(cx);
        let name = model.title();
        let strings = cx.t();
        let session = model.session();
        let frame = session.and_then(|s| s.read(cx).latest_frame());
        let mut description = vec![model.pair().label.clone()];
        if let Some(summary) = session.and_then(|s| s.read(cx).summary()) {
            description.push(strings.model_view_stats(
                i64::try_from(summary.vertex_count).unwrap_or(i64::MAX),
                i64::try_from(summary.triangle_count).unwrap_or(i64::MAX),
            ));
        }
        if let Some(preset) = model.selected() {
            description.push(preset_label(preset, strings).to_string());
        }
        description.extend(model.warning_lines(cx).iter().map(ToString::to_string));
        let equip = model.presets().len() > 1;
        let error = model.error_text(cx);
        let scene = div()
            .relative()
            .size_full()
            .child(
                viewport("unit-model-frame", &self.model, session, false, cx)
                    .transparent()
                    .loading(false),
            )
            .when(frame.is_none() && error.is_none(), |v| {
                v.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(LoadingIcon::new("unit-loading").scale(scale)),
                )
            })
            .when_some(error, |v, error| {
                v.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            GameText::new(error)
                                .tint(yp_nier_ui::LIGHT)
                                .scale(0.7 * scale),
                        ),
                )
            });
        let owner = cx.entity();
        ModelViewer::new("archive-model", name, scene, move |action, w, cx| {
            owner.update(cx, |v, cx| v.unit_action(action, w, cx))
        })
        .description(description.join("\n"))
        .equipment(equip)
        .expanded(expanded)
        .scale(scale)
    }

    pub(super) fn unit_data(&self, expanded: bool, cx: &mut Context<Self>) -> AnyElement {
        let list = cx.entity();
        let scene = cx.entity();
        let open = cx.entity();
        let back = cx.entity();
        let count = self.model.read(cx).candidates().len();
        let scroll = self.unit_scroll.clone();
        let frame = MenuFrame::new("archive-unit-data", "INTEL")
            .subtitle("Unit Data")
            .list(move |scale, _| {
                MenuList::new("archive-units", count, move |i, _, cx| {
                    let model = list.read(cx).model.read(cx);
                    let Some(c) = model.candidates().get(i) else {
                        return gpui::Empty.into_any_element();
                    };
                    let select = list.clone();
                    let open = list.clone();
                    MenuItemRow::new(("candidate", i), c.name.clone())
                        .icon("panel")
                        .scale(scale)
                        .selected(model.candidate().is_some_and(|v| v.id == c.id))
                        .on_hover(move |over, _, cx| {
                            if over {
                                select.update(cx, |v, cx| {
                                    if v.unit_view == Some(false) {
                                        v.change_unit(i, cx);
                                    }
                                });
                            }
                        })
                        .on_click(move |_, w, cx| {
                            open.update(cx, |v, cx| {
                                v.change_unit(i, cx);
                                v.unit_action(ViewerAction::Open, w, cx);
                            })
                        })
                        .into_any_element()
                })
                .track_scroll(scroll)
                .item_layout(true)
                .scale(scale)
            })
            .foreground(move |scale, cx| {
                scene.update(cx, |v, cx| v.unit_scene(scale, expanded, cx))
            })
            .footer(move |_, _| {
                HelpBar::new("Select unit to view.").prompts(vec![
                    ButtonPrompt::new("up", "Select").with_key("down"),
                    ButtonPrompt::new("confirm", "Model Viewer").on_click(move |w, cx| {
                        open.update(cx, |v, cx| v.unit_action(ViewerAction::Open, w, cx))
                    }),
                    ButtonPrompt::new("back", "Back").on_click(move |w, cx| {
                        back.update(cx, |v, cx| v.unit_action(ViewerAction::Back, w, cx))
                    }),
                ])
            });
        div()
            .id("unit-data-inspector")
            .size_full()
            .occlude()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|v, event: &KeyDownEvent, w, cx| {
                let action = match event.keystroke.key.as_str() {
                    "escape" => ViewerAction::Back,
                    "enter" if v.unit_view == Some(false) => ViewerAction::Open,
                    "up" | "w" => ViewerAction::Previous,
                    "down" | "s" => ViewerAction::Next,
                    _ => return,
                };
                v.unit_action(action, w, cx);
                cx.stop_propagation();
            }))
            .child(frame)
            .into_any_element()
    }
}
