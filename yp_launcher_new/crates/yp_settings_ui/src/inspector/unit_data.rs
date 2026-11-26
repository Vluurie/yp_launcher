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
            let style = super::model_style(view.is_some());
            self.model
                .update(cx, |model, cx| model.set_style(style, cx));
        }
        self.unit_view = view;
        if view != Some(true) || Self::responsive_unit(w, cx) {
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
        if self.returning {
            return;
        }
        match action {
            ViewerAction::Open => self.set_unit_view(Some(true), w, cx),
            ViewerAction::Back if self.controls_open => {
                self.toggle_controls(cx);
                self.focus_handle.focus(w, cx);
            }
            ViewerAction::Back
                if cx.theme().id == ThemeId::NierNew
                    && (self.single_preview || self.unit_view == Some(false)) =>
            {
                if !self.start_transition(false, cx) {
                    self.close(cx);
                }
            }
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
            ViewerAction::Animation => {
                if let Some(session) = self.model.read(cx).session().cloned() {
                    session.update(cx, |s, cx| s.toggle_motion(cx));
                }
            }
        }
    }

    fn responsive_unit(window: &Window, cx: &App) -> bool {
        cx.theme().id == ThemeId::NierNew
            && (window.viewport_size().width < px(720.)
                || yp_ui::theme::MenuTextSize::percent(cx) > 100)
    }

    fn model_caption(&self, cx: &App) -> SharedString {
        let model = self.model.read(cx).title();
        self.origin.as_ref().map_or_else(
            || model.clone(),
            |origin| format!("{origin} · {model}").into(),
        )
    }

    fn unit_description(&self, cx: &App) -> String {
        let model = self.model.read(cx);
        let strings = cx.t();
        let session = model.session();
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
        description.join("\n")
    }

    fn unit_frame(&self, cx: &App) -> Option<std::sync::Arc<gpui::RenderImage>> {
        let model = self.model.read(cx);
        model.session().and_then(|s| s.read(cx).latest_frame()).or_else(|| {
            let preview = self.preview.as_ref()?.upgrade()?;
            let source = preview.read(cx);
            let selected = model.candidate().map(|c| c.id).or(model.initial_candidate)?;
            if source.candidate()?.id != selected {
                return None;
            }
            source.session()?.read(cx).latest_frame()
        })
    }

    fn unit_scene(&self, scale: f32, expanded: bool, resize: bool, cx: &mut Context<Self>) -> ModelViewer {
        let model = self.model.read(cx);
        let name = self.model_caption(cx);
        let strings = cx.t();
        let session = model.session();
        let frame = self.unit_frame(cx);
        let equip = model.presets().len() > 1;
        let playback = session.map(|s| s.read(cx));
        let motion =
            playback.and_then(|p| p.motion_index(cx).map(|i| (i + 1, p.motion_names().len())));
        let toggle = playback.map_or_else(|| strings.motion_play(), |p| super::motion::playback_label(p, cx));
        let error = model.error_text(cx);
        let scene = div()
            .relative()
            .size_full()
            .child(
                viewport("unit-model-frame", &self.model, session, true, resize, cx)
                    .frame(frame.clone())
                    .fit(if expanded { gpui::ObjectFit::Contain } else { gpui::ObjectFit::Cover })
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
                                .scale(0.7 * scale)
                                .wrap(390. * scale),
                        ),
                )
            });
        let owner = cx.entity();
        ModelViewer::new("archive-model", name, scene, move |action, w, cx| {
            owner.update(cx, |v, cx| v.unit_action(action, w, cx))
        })
        .prompts(
            strings.menu_rotate(),
            toggle,
            strings.menu_equipment(),
            strings.button_back(),
        )
        .description(self.unit_description(cx))
        .equipment(equip)
        .playback(playback.is_some_and(|p| p.can_play(cx)))
        .when_some(motion, |viewer, (i, n)| viewer.motions(i, n))
        .expanded(expanded)
        .animate_viewport(!(self.single_preview && self.preview.is_some()))
        .scale(scale)
    }

    pub(super) fn unit_data(
        &mut self,
        expanded: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (drawer_progress, drawer_moving) = self.controls_progress(window, cx);
        let strings = cx.t();
        let controls = || {
            let open = self.controls_open;
            let colors = cx.theme().colors.clone();
            h_flex()
                .id("unit-controls")
                .debug_selector(|| "unit-controls".into())
                .role(gpui::Role::Button)
                .aria_label(strings.menu_model_controls())
                .aria_expanded(open)
                .tab_index(0)
                .min_w_0()
                .gap(px(6.))
                .px(px(10.))
                .py(px(5.))
                .border_1()
                .border_color(colors.border_medium)
                .bg(colors.background_card)
                .hover(move |s| s.bg(colors.surface_medium))
                .cursor_pointer()
                .keyboard_activate(cx)
                .on_disclosure_toggle(
                    open,
                    cx.listener(|v, _, _, cx| {
                        v.toggle_controls(cx);
                        cx.stop_propagation();
                    }),
                )
                .child(Icon::new(IconName::Tune).size(IconSize::Small))
                .child(
                    Label::new(strings.menu_model_controls())
                        .size(LabelSize::Small)
                        .semibold(),
                )
                .child(
                    Icon::new(if open {
                        IconName::KeyboardArrowUp
                    } else {
                        IconName::KeyboardArrowDown
                    })
                    .size(IconSize::Small),
                )
        };
        if Self::responsive_unit(window, cx) {
            let model = self.model.read(cx);
            let frame = self.unit_frame(cx);
            let viewer = viewport("compact-unit-model", &self.model, model.session(), true, true, cx)
                .loading(frame.is_none() && model.error_text(cx).is_none())
                .frame(frame)
                .transparent();
            let pick = (!expanded)
                .then(|| super::candidate_select("compact-unit-picker", &self.model, cx))
                .flatten();
            let name = self.model_caption(cx);
            let description = self.unit_description(cx);
            let panel = v_flex()
                .id("unit-data-inspector")
                .role(gpui::Role::Group)
                .aria_label(name.clone())
                .aria_description(description.clone())
                .size_full()
                .min_w_0()
                .min_h_0()
                .p(px(10.))
                .gap(px(5.))
                .bg(cx.theme().colors.background_primary)
                .track_focus(&self.focus_handle)
                .on_key_down(cx.listener(|v, k: &KeyDownEvent, w, cx| {
                    if v.controls_open && k.keystroke.key != "escape" {
                        return;
                    }
                    let action = match k.keystroke.key.as_str() {
                        "escape" => ViewerAction::Back,
                        "left" => ViewerAction::Rotate { dx: -12., dy: 0. },
                        "right" => ViewerAction::Rotate { dx: 12., dy: 0. },
                        "up" => ViewerAction::Rotate { dx: 0., dy: -12. },
                        "down" => ViewerAction::Rotate { dx: 0., dy: 12. },
                        _ => return,
                    };
                    v.unit_action(action, w, cx);
                    cx.stop_propagation();
                }))
                .child(
                    h_flex()
                        .w_full()
                        .min_w_0()
                        .justify_between()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(
                            Button::new("compact-unit-back", strings.button_back())
                                .style(ButtonStyle::Link)
                                .start_icon(IconName::ArrowBack)
                                .on_click(cx.listener(|v, _, w, cx| {
                                    v.unit_action(ViewerAction::Back, w, cx)
                                })),
                        )
                        .when(!self.returning, |el| el.child(controls())),
                );
            let body = v_flex()
                .absolute()
                .inset_0()
                .gap(px(5.))
                .child(
                    div()
                        .id("compact-unit-caption")
                        .w_full()
                        .min_w_0()
                        .tooltip(Tooltip::text(name.clone()))
                        .child(Label::new(name).semibold().line_clamp(2)),
                )
                .children(pick)
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .bg(ModelViewer::background())
                        .child(div().absolute().inset_0().child(viewer)),
                )
                .when(!self.returning, |el| {
                    el.children(super::motion::controls(
                        "compact-model-motion",
                        &self.model,
                        cx,
                    ))
                })
                .when(!expanded, |el| {
                    el.child(
                        div()
                            .flex()
                            .items_center()
                            .flex_wrap()
                            .w_full()
                            .min_w_0()
                            .gap(px(6.))
                            .child(
                                div().flex_1().min_w_0().child(
                                    Label::new(description.replace('\n', " · "))
                                        .size(LabelSize::XSmall),
                                ),
                            )
                            .child(
                                Button::new(
                                    "compact-unit-expand",
                                    strings.three_d_inspector_expand(),
                                )
                                .style(ButtonStyle::Filled)
                                .on_click(cx.listener(
                                    |v, _, w, cx| v.unit_action(ViewerAction::Open, w, cx),
                                )),
                            ),
                    )
                });
            let details = (drawer_progress > 0.).then(|| {
                v_flex()
                    .id("unit-controls-panel")
                    .debug_selector(|| "unit-controls-panel".into())
                    .role(gpui::Role::Group)
                    .aria_label(strings.menu_model_controls())
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .w_full()
                    .left(relative(1. - drawer_progress))
                    .occlude()
                    .bg(cx.theme().colors.background_primary)
                    .child(self.toolbar(window, cx))
                    .child(div().flex_1().min_h_0().children(self.mesh_panel(window, cx)))
            });
            let panel = panel.child(
                div().relative().w_full().flex_1().min_h_0().overflow_hidden()
                    .when(drawer_progress < 1., |el| el.child(body))
                    .children(details)
            );
            if self.preview.is_some() || yp_nier_ui::ReducedMotion::enabled(cx) {
                return panel.into_any_element();
            }
            return gpui::AnimationExt::with_animation(
                panel,
                ("responsive-model-transition", usize::from(expanded)),
                gpui::Animation::new(std::time::Duration::from_millis(230)),
                |panel, progress| {
                    panel
                        .pt(px(10. + 12. * (1. - progress)))
                        .opacity(0.6 + 0.4 * progress)
                },
            )
            .into_any_element();
        }
        let list = cx.entity();
        let scene = cx.entity();
        let open = cx.entity();
        let back = cx.entity();
        let count = self.model.read(cx).candidates().len();
        let scroll = self.unit_scroll.clone();
        let returning = self.returning;
        let frame = MenuFrame::new(
            "archive-unit-data",
            strings.menu_model_archive().to_uppercase(),
        )
        .screen_overlay(window.yp_root().is_none())
        .subtitle(
            self.origin
                .clone()
                .unwrap_or_else(|| strings.menu_models().into()),
        )
        .list(move |scale, _| {
            if expanded || returning {
                return gpui::Empty.into_any_element();
            }
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
            .into_any_element()
        })
        .foreground(move |scale, cx| scene.update(cx, |v, cx| v.unit_scene(scale, expanded, !drawer_moving, cx)))
        .footer(move |_, _| {
            if expanded || returning {
                return gpui::Empty.into_any_element();
            }
            HelpBar::new(strings.menu_choose_model())
                .prompts(vec![
                    ButtonPrompt::new("up", strings.menu_select()).with_key("down"),
                    ButtonPrompt::new("confirm", strings.three_d_inspector_expand()).on_click(
                        move |w, cx| {
                            open.update(cx, |v, cx| v.unit_action(ViewerAction::Open, w, cx))
                        },
                    ),
                    ButtonPrompt::new("back", strings.button_back()).on_click(move |w, cx| {
                        back.update(cx, |v, cx| v.unit_action(ViewerAction::Back, w, cx))
                    }),
                ])
                .into_any_element()
        });
        let frame = if self.preview.is_some() {
            frame.opened_at(std::time::Instant::now() - std::time::Duration::from_secs(2))
        } else {
            frame
        };
        let appearance = cx.theme().clone();
        let controls_bar = (appearance.id == ThemeId::NierNew).then(|| {
            h_flex()
                .w_full()
                .flex_none()
                .justify_end()
                .px(px(12.))
                .py(px(5.))
                .border_b_1()
                .border_color(appearance.colors.border_medium)
                .bg(appearance.colors.background_card)
                .child(controls())
        });
        let drawer = (drawer_progress > 0.).then(|| {
            let width = px(320.).min(window.viewport_size().width * 0.45);
            let panel = v_flex()
                .id("unit-controls-panel")
                .debug_selector(|| "unit-controls-panel".into())
                .role(gpui::Role::Group)
                .aria_label(strings.menu_model_controls())
                .w(width)
                .h_full()
                .flex_none()
                .min_w_0()
                .bg(appearance.colors.background_card)
                .child(self.toolbar(window, cx))
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .children(self.mesh_panel(window, cx)),
                );
            div().flex_none().h_full().w(width * drawer_progress).overflow_hidden().child(panel)
        });
        v_flex()
            .id("unit-data-inspector")
            .role(gpui::Role::Group)
            .aria_label(self.model_caption(cx))
            .bg(appearance.colors.background_primary)
            .size_full()
            .occlude()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|v, event: &KeyDownEvent, w, cx| {
                if v.controls_open && event.keystroke.key != "escape" {
                    return;
                }
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
            .children(controls_bar)
            .child(
                h_flex()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .child(div().relative().flex_1().min_w_0().h_full().child(frame))
                    .children(drawer),
            )
            .into_any_element()
    }
}
