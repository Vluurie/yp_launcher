pub mod model;
mod motion;
mod transition;
mod unit_data;

use std::collections::HashMap;

use gpui::{Global, Pixels, Point, ScrollHandle, Subscription, canvas};
use yp_state::inspector::ArchivePaths;
use yp_state::prelude::*;
use yp_state::{ModelSession, OverlayId};
use yp_ui::prelude::*;
use yp_ui::{
    ButtonSize, Cancel, Divider, FrameViewport, Icon, IconButton, IconName, IconSize, Label,
    LabelSize, ScrollableElement, Select, SelectItem, SettingSwitch, Tooltip,
};

use model::{
    InspectorModel, ViewMode, is_vanilla, mesh_groups, missing_mesh_names, preset_label,
    state_label,
};

pub const KEY_CONTEXT: &str = "YpInspector";
const PREVIEW_ASPECT: f32 = 16. / 9.;

#[derive(Default)]
struct PreviewSelection {
    outfit: Option<i64>,
    preset: Option<String>,
    candidate: Option<yp_inspector::CandidateId>,
    single: bool,
    preview: Option<gpui::WeakEntity<InspectorModel>>,
}

#[derive(Default)]
pub struct PendingInspectorOutfits(HashMap<ArchivePaths, PreviewSelection>);

impl Global for PendingInspectorOutfits {}

pub fn remember_outfit(archives: &ArchivePaths, outfit: Option<i64>, cx: &mut App) {
    let pending = &mut cx.default_global::<PendingInspectorOutfits>().0;
    if let Some(outfit) = outfit {
        pending.entry(archives.clone()).or_default().outfit = Some(outfit);
    } else {
        pending.remove(archives);
    }
}

pub fn take_outfit(archives: &ArchivePaths, cx: &mut App) -> Option<i64> {
    cx.default_global::<PendingInspectorOutfits>().0.remove(archives)?.outfit
}

fn viewport(
    id: &'static str,
    model: &Entity<InspectorModel>,
    session: Option<&Entity<ModelSession>>,
    allow_pan: bool,
    cx: &App,
) -> FrameViewport {
    let state = model.read(cx);
    let frame = session.and_then(|session| session.read(cx).latest_frame());
    let failed = state.error_text(cx).is_some();
    let loading = !failed && (state.is_loading(cx) || frame.is_none());
    let measure_model = model.downgrade();
    let scene_bounds = state.scene_bounds.clone();
    let animated = session.map(Entity::downgrade);
    let embedded = state.mode() == ViewMode::Preview;
    let orbit_model = model.clone();
    let zoom_model = model.clone();
    let view = FrameViewport::new(id)
        .frame(frame)
        .loading(loading)
        .on_orbit(move |delta: &Point<Pixels>, _, cx| {
            orbit_model.update(cx, |model, cx| {
                model.orbit(f32::from(delta.x), f32::from(delta.y), cx)
            });
        })
        .on_zoom(move |delta, _, cx| {
            zoom_model.update(cx, |model, cx| model.zoom(*delta, cx));
        })
        .overlay(
            canvas(
                move |bounds, window, cx| {
                    scene_bounds.set(Some(bounds));
                    let width = f32::from(bounds.size.width);
                    let height = f32::from(bounds.size.height);
                    let covered = embedded && cx.stores().navigation.read(cx).top_overlay().is_some();
                    if !covered && window.is_window_active() && bounds.intersects(&window.content_mask().bounds)
                        && let Some(session) = animated.as_ref().and_then(|weak| weak.upgrade())
                        && session.read(cx).motion_playing(cx)
                    {
                        window.request_animation_frame();
                        if session.read(cx).tick_due(cx) {
                            window.defer(cx, move |_, cx| session.update(cx, |state, cx| {
                                if state.tick_due(cx) { state.request_render(cx); }
                            }));
                        }
                    }
                    let model = measure_model.clone();
                    let unchanged = model
                        .read_with(cx, |model, _| model.viewport() == Some((width, height)))
                        .unwrap_or(true);
                    if unchanged {
                        return;
                    }
                    window.defer(cx, move |_, cx| {
                        model
                            .update(cx, |model, cx| model.set_viewport(width, height, cx))
                            .ok();
                    });
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        );
    if !allow_pan {
        return view;
    }
    let pan_model = model.clone();
    view.on_pan(move |delta: &Point<Pixels>, _, cx| {
        pan_model.update(cx, |model, cx| {
            model.pan(f32::from(delta.x), f32::from(delta.y), cx)
        });
    })
}

fn error_layer(text: String, small: bool, window: &Window, cx: &App) -> gpui::Div {
    let sizes = window.sizes(cx);
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .p(sizes.padding_lg)
        .child(
            div().max_w(sizes.dialog_width).child(
                Label::new(text)
                    .size(if small {
                        LabelSize::XSmall
                    } else {
                        LabelSize::Default
                    })
                    .color(Color::Error)
                    .centered(),
            ),
        )
}

fn candidate_select(
    id: &'static str,
    model: &Entity<InspectorModel>,
    cx: &App,
) -> Option<Select<usize>> {
    let state = model.read(cx);
    let candidates = state.candidates();
    if candidates.len() <= 1 {
        return None;
    }
    let current = state.candidate().map(|candidate| candidate.id);
    let selected = candidates
        .iter()
        .position(|candidate| Some(candidate.id) == current);
    let label = state.title();
    let options: Vec<_> = candidates.to_vec();
    let select_model = model.clone();
    Some(
        Select::new(id, label)
            .items(
                options
                    .iter()
                    .enumerate()
                    .map(|(index, candidate)| SelectItem::new(candidate.name.clone(), index)),
            )
            .selected_index(selected)
            .on_select(move |index: &usize, _, cx| {
                if let Some(candidate) = options.get(*index).cloned() {
                    select_model.update(cx, |model, cx| model.select_candidate(candidate, cx));
                }
            }),
    )
}

fn preset_select(
    id: &'static str,
    model: &Entity<InspectorModel>,
    cx: &App,
) -> Option<Select<usize>> {
    let strings = cx.t();
    let state = model.read(cx);
    let selected = state.selected_index()?;
    let current = state.selected()?;
    let select_model = model.clone();
    Some(
        Select::new(id, preset_label(current, strings))
            .items(
                state
                    .presets()
                    .iter()
                    .enumerate()
                    .map(|(index, preset)| SelectItem::new(preset_label(preset, strings), index)),
            )
            .selected_index(selected)
            .highlight(true)
            .on_select(move |index: &usize, _, cx| {
                select_model.update(cx, |model, cx| model.select_preset(*index, cx));
            }),
    )
}

pub struct InspectorPage {
    model: Entity<InspectorModel>,
    focus_handle: FocusHandle,
    mesh_scroll: ScrollHandle,
    state_scroll: ScrollHandle,
    focus_requested: bool,
    unit_view: Option<bool>,
    single_preview: bool,
    preview: Option<gpui::WeakEntity<InspectorModel>>,
    transition: Option<transition::ViewportTransition>,
    controls_open: bool,
    returning: bool,
    origin: Option<SharedString>,
    unit_scroll: gpui::UniformListScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl InspectorPage {
    pub fn new(archives: ArchivePaths, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let selection = cx
            .default_global::<PendingInspectorOutfits>()
            .0
            .remove(&archives)
            .unwrap_or_default();
        let stores = cx.stores();
        let origin = archives.dat.as_ref().or(archives.dtt.as_ref()).and_then(|path| {
            let installed = stores
                .mods
                .read(cx)
                .mods()
                .iter()
                .find(|entry| path.starts_with(&entry.root_path))?;
            let name = stores
                .mod_names
                .read(cx)
                .custom_name_of(&installed.id)
                .unwrap_or(&installed.display_name);
            Some(SharedString::from(name.to_owned()))
        });
        let model = cx.new(|cx| {
            let mut model = InspectorModel::new(archives, ViewMode::Page, selection.outfit, cx);
            model.initial_candidate = selection.candidate;
            model.initial_preset = selection.preset;
            model.initial_camera = selection.preview.as_ref().and_then(|source| source.upgrade()).and_then(|source| {
                source.read(cx).session().map(|session| *session.read(cx).camera())
            });
            model
        });
        let subscriptions = vec![cx.observe(&model, |_, _, cx| cx.notify())];
        Self {
            model,
            focus_handle: cx.focus_handle(),
            mesh_scroll: ScrollHandle::new(),
            state_scroll: ScrollHandle::new(),
            focus_requested: false,
            unit_view: None,
            single_preview: selection.single,
            preview: selection.preview,
            transition: None,
            controls_open: false,
            returning: false,
            origin,
            unit_scroll: Default::default(),
            _subscriptions: subscriptions,
        }
    }

    pub fn model(&self) -> &Entity<InspectorModel> {
        &self.model
    }

    fn sync_preview_outfit(&self, cx: &mut Context<Self>) {
        let Some(preview) = self.preview.as_ref().and_then(|p| p.upgrade()) else { return };
        let state = self.model.read(cx);
        let Some(candidate) = state.candidate().cloned() else { return };
        let preset = state.selected().cloned();
        preview.update(cx, |p, cx| p.restore_selection(candidate, preset.as_ref(), cx));
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_outfit(cx);
        let navigation = cx.stores().navigation.clone();
        navigation.update(cx, |navigation, cx| {
            navigation.close_overlay(OverlayId::Inspector, cx)
        });
    }

    fn toolbar(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = self.model.read(cx);
        let in_drawer = self.controls_open && theme.id == ThemeId::NierNew;
        let title = state.title();
        let session = state.session().cloned();
        let summary = session
            .as_ref()
            .and_then(|session| session.read(cx).summary().cloned());
        let selected_lod = session
            .as_ref()
            .map(|session| session.read(cx).selected_lod())
            .unwrap_or_default();
        let candidates = candidate_select("inspector-candidate", &self.model, cx);
        let warnings = state.warning_lines(cx);
        let lod_model = self.model.clone();
        let reset_model = self.model.clone();
        let lod_select = summary
            .as_ref()
            .filter(|summary| summary.lod_names.len() > 1)
            .map(|summary| {
                let label = |name: &str| SharedString::from(strings.model_view_lod_option(name));
                let current = summary
                    .lod_names
                    .get(selected_lod as usize)
                    .map(|name| label(name))
                    .unwrap_or_default();
                Select::new("inspector-lod", current)
                    .items(
                        summary
                            .lod_names
                            .iter()
                            .enumerate()
                            .map(|(index, name)| SelectItem::new(label(name), index)),
                    )
                    .selected_index(selected_lod as usize)
                    .on_select(move |index: &usize, _, cx| {
                        let lod = u32::try_from(*index).unwrap_or_default();
                        lod_model.update(cx, |model, cx| model.set_lod(lod, cx));
                    })
            });
        let stats = summary.as_ref().map(|summary| {
            strings.model_view_stats(
                i64::try_from(summary.vertex_count).unwrap_or(i64::MAX),
                i64::try_from(summary.triangle_count).unwrap_or(i64::MAX),
            )
        });
        h_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .px(sizes.padding_sm)
            .py(sizes.padding_xs)
            .rounded(sizes.border_radius)
            .border_1()
            .border_color(theme.colors.border_medium)
            .bg(theme.colors.background_card.opacity(theme.alpha.tooltip))
            .flex_wrap().min_w_0()
            .when(!in_drawer, |bar| bar.child(
                IconButton::new("inspector-close", IconName::Close)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Secondary)
                    .bordered(true)
                    .tooltip(Tooltip::text(strings.three_d_inspector_close()))
                    .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
            ))
            .child(div().flex_1().min_w_0().child(match candidates {
                Some(select) => h_flex().child(select).into_any_element(),
                None => Label::new(title).semibold().truncate().into_any_element(),
            }))
            .when(!in_drawer && !state.candidates().is_empty(), |bar| {
                bar.child(
                    IconButton::new("inspector-unit-data", IconName::OpenInFull)
                        .icon_size(IconSize::Small)
                        .icon_color(Color::Accent)
                        .bordered(true)
                        .tooltip(Tooltip::text(strings.three_d_inspector_expand()))
                        .on_click(cx.listener(|this, _, w, cx| {
                            this.set_unit_view(Some(false), w, cx);
                        })),
                )
            })
            .children(lod_select)
            .when(!warnings.is_empty(), |this| {
                this.child(
                    div()
                        .id("inspector-warnings")
                        .flex_none()
                        .child(
                            Icon::new(IconName::WarningAmber)
                                .size(IconSize::Small)
                                .color(Color::Warning),
                        )
                        .tooltip(Tooltip::lines(
                            strings.model_view_warnings_title(),
                            warnings,
                        )),
                )
            })
            .when_some(stats.filter(|_| !in_drawer), |this, stats| {
                this.child(
                    Label::new(stats)
                        .size(LabelSize::XSmall)
                        .color(Color::Secondary),
                )
            })
            .child(
                IconButton::new("inspector-reset-camera", IconName::CenterFocusStrong)
                    .icon_size(IconSize::Small)
                    .icon_color(Color::Accent)
                    .bordered(true)
                    .accessible_label(strings.three_d_inspector_reset_camera())
                    .tooltip(Tooltip::text(strings.three_d_inspector_reset_camera()))
                    .on_click(move |_, _, cx| {
                        reset_model.update(cx, |model, cx| model.reset_camera(cx));
                    }),
            )
            .into_any_element()
    }

    fn mesh_panel(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = self.model.read(cx);
        let session = state.session()?.clone();
        let session_state = session.read(cx);
        if !session_state.is_ready() {
            return None;
        }
        let meshes = session_state.meshes().to_vec();
        let groups = mesh_groups(&meshes, session_state.selected_lod());
        let preset = state.selected().cloned();
        let state_values = state.state_values().clone();
        let missing = preset
            .as_ref()
            .map(|preset| missing_mesh_names(preset, &meshes))
            .unwrap_or_default();
        let preset_picker = preset_select("inspector-preset", &self.model, cx);
        let mut panel = v_flex()
            .size_full()
            .py(sizes.padding_sm)
            .rounded(sizes.border_radius)
            .border_1()
            .border_color(theme.colors.border_medium)
            .bg(theme.colors.background_card.opacity(theme.alpha.tooltip))
            .children(motion::controls("inspector-motion", &self.model, cx))
            .child(Divider::horizontal());
        if let (Some(preset), Some(picker)) = (preset.as_ref(), preset_picker) {
            let heading = if is_vanilla(preset) {
                strings.three_d_inspector_variant()
            } else {
                strings.three_d_inspector_outfit()
            };
            let mut section = v_flex()
                .w_full()
                .flex_none()
                .gap(sizes.spacing_md)
                .px(sizes.padding_md)
                .py(sizes.spacing_md)
                .child(Label::new(heading).size(LabelSize::Small).bold())
                .child(picker.full_width().menu_matches_trigger_width(true));
            if !preset.states.is_empty() {
                let rows = preset.states.iter().map(|state| {
                    let state_model = self.model.clone();
                    let state_id = state.id.clone();
                    SettingSwitch::new(
                        SharedString::from(format!("inspector-state-{}", state.id)),
                        state_label(&state.id, strings),
                        state_values.get(&state.id).copied().unwrap_or(false),
                    )
                    .on_change(move |enabled, _, cx| {
                        let enabled = *enabled;
                        state_model.update(cx, |model, cx| model.set_state(&state_id, enabled, cx));
                    })
                });
                section = section
                    .child(Label::new(strings.three_d_inspector_states()).size(LabelSize::Small).bold())
                    .child(
                        div()
                            .id("inspector-states")
                            .w_full()
                            .max_h(sizes.dialog_width * 0.5)
                            .children(rows)
                            .overflow_y_scrollbar(&self.state_scroll),
                    );
            }
            if !missing.is_empty() {
                let count = i64::try_from(missing.len()).unwrap_or(i64::MAX);
                let lines: Vec<SharedString> = missing.into_iter().map(SharedString::from).collect();
                section = section.child(
                    div()
                        .id("inspector-missing-meshes")
                        .child(
                            Label::new(strings.three_d_inspector_missing_meshes(count))
                                .size(LabelSize::XSmall)
                                .color(Color::Warning),
                        )
                        .tooltip(Tooltip::lines(
                            strings.three_d_inspector_missing_meshes(count),
                            lines,
                        )),
                );
            }
            panel = panel.child(section).child(Divider::horizontal());
        }
        let rows = groups.into_iter().enumerate().map(|(index, group)| {
            let mesh_model = self.model.clone();
            let ids = group.ids.clone();
            SettingSwitch::new(("inspector-mesh", index), group.name.clone(), group.visible)
                .on_change(move |visible, _, cx| {
                    let visible = *visible;
                    mesh_model.update(cx, |model, cx| {
                        model.set_group_visibility(&ids, visible, cx)
                    });
                })
        });
        Some(
            panel
                .child(
                    div()
                        .flex_none()
                        .px(sizes.padding_md)
                        .py(sizes.spacing_md)
                        .child(Label::new(strings.three_d_inspector_meshes()).size(LabelSize::Small).bold()),
                )
                .child(Divider::horizontal())
                .child(
                    div()
                        .id("inspector-meshes")
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .px(sizes.padding_xs)
                        .children(rows)
                        .overflow_y_scrollbar(&self.mesh_scroll),
                )
                .into_any_element(),
        )
    }
}

impl Focusable for InspectorPage {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for InspectorPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        if !self.focus_requested {
            self.focus_requested = true;
            window.focus(&self.focus_handle, cx);
            if theme.id == ThemeId::NierNew {
                self.set_unit_view(Some(self.single_preview), window, cx);
                if self.single_preview { self.start_transition(true, cx); }
            }
        }
        if let Some(expanded) = self.unit_view {
            let content = self.unit_data(expanded, window, cx);
            return self.transition_view(content, window, cx);
        }
        let state = self.model.read(cx);
        let session = state.session().cloned();
        let error = state.error_text(cx);
        let has_frame = session
            .as_ref()
            .is_some_and(|session| session.read(cx).latest_frame().is_some());
        let view = viewport("inspector-viewport", &self.model, session.as_ref(), true, cx);
        let toolbar = self.toolbar(window, cx);
        let panel = self.mesh_panel(window, cx);
        let inset = sizes.padding_lg;
        let panel_width = sizes.dialog_width * 0.6;
        let toolbar_space = Sizes::TITLE_BAR_HEIGHT + inset * 2.;
        div()
            .id("inspector-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &Cancel, _, cx| this.close(cx)))
            .relative()
            .size_full()
            .occlude()
            .bg(theme.colors.background_primary)
            .child(div().absolute().inset_0().child(view))
            .when_some(error.filter(|_| !has_frame), |this, error| {
                this.child(error_layer(error, false, window, cx))
            })
            .child(
                div()
                    .absolute()
                    .top(inset)
                    .left(inset)
                    .right(inset)
                    .child(toolbar),
            )
            .when_some(panel, |this, panel| {
                this.child(
                    div()
                        .absolute()
                        .top(toolbar_space)
                        .right(inset)
                        .bottom(toolbar_space)
                        .w(panel_width)
                        .child(panel),
                )
            })
            .child(
                div()
                    .absolute()
                    .left(inset)
                    .bottom(inset)
                    .px(sizes.padding_sm)
                    .py(sizes.padding_xs)
                    .rounded(sizes.border_radius)
                    .border_1()
                    .border_color(theme.colors.border_light)
                    .bg(theme.colors.background_card.opacity(theme.alpha.panel))
                    .child(
                        Label::new(strings.three_d_inspector_controls())
                            .size(LabelSize::XSmall)
                            .color(Color::Secondary),
                    ),
            )
            .into_any_element()
    }
}

pub struct ModelPreview {
    model: Entity<InspectorModel>,
    _subscriptions: Vec<Subscription>,
}

impl ModelPreview {
    pub fn new(archives: ArchivePaths, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let model = cx.new(|cx| InspectorModel::new(archives, ViewMode::Preview, None, cx));
        let subscriptions = vec![cx.observe(&model, |_, _, cx| cx.notify())];
        Self {
            model,
            _subscriptions: subscriptions,
        }
    }

    pub fn model(&self) -> &Entity<InspectorModel> {
        &self.model
    }

    pub(super) fn expand(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let state = self.model.read(cx);
        let archives = state.archives().clone();
        let selection = PreviewSelection {
            outfit: state.selected().map(|preset| preset.outfit_id),
            preset: state.selected().map(|preset| preset.id.clone()),
            candidate: state.candidate().map(|candidate| candidate.id),
            single: state.candidates().len() == 1,
            preview: Some(self.model.downgrade()),
        };
        cx.default_global::<PendingInspectorOutfits>()
            .0
            .insert(archives.clone(), selection);
        window.dispatch_action(
            Box::new(OpenThreeDInspector {
                data: archives.dat.clone(),
                dtt: archives.dtt.clone(),
            }),
            cx,
        );
    }
}

impl ModelPreview {
    pub fn has_model(&self, cx: &App) -> bool {
        !self.model.read(cx).candidates().is_empty()
    }
}

impl Render for ModelPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.has_model(cx) {
            return gpui::Empty.into_any_element();
        }
        let strings = cx.t();
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = self.model.read(cx);
        let session = state.session().cloned();
        let error = state.error_text(cx);
        let label = SharedString::from(state.pair().label.clone());
        let has_frame = session
            .as_ref()
            .is_some_and(|session| session.read(cx).latest_frame().is_some());
        let show_presets = state.presets().len() > 1;
        let candidates = candidate_select("model-preview-candidate", &self.model, cx);
        let presets = show_presets
            .then(|| preset_select("model-preview-preset", &self.model, cx))
            .flatten();
        let view = viewport("model-preview-viewport", &self.model, session.as_ref(), false, cx);
        let mut frame = div().relative().w_full().overflow_hidden();
        frame.style().aspect_ratio = Some(PREVIEW_ASPECT);
        v_flex()
            .w_full()
            .mb(sizes.spacing_md)
            .rounded(sizes.border_radius)
            .border_1()
            .border_color(theme.colors.border_light)
            .bg(theme.colors.background_card)
            .overflow_hidden()
            .child(
                h_flex()
                    .w_full()
                    .gap(sizes.spacing_sm)
                    .px(sizes.padding_sm)
                    .py(sizes.padding_xs)
                    .bg(theme.colors.surface_medium).min_w_0().flex_wrap()
                    .child(div().flex_1().min_w_0().child(match candidates {
                        Some(select) => h_flex().child(select).into_any_element(),
                        None => Label::new(label)
                            .size(LabelSize::Small)
                            .semibold()
                            .truncate()
                            .into_any_element(),
                    }))
                    .children(presets)
                    .child(if theme.id == ThemeId::NierNew {
                        Button::new("model-preview-expand",strings.three_d_inspector_open())
                            .style(ButtonStyle::Outlined).start_icon(IconName::OpenInFull)
                            .on_click(cx.listener(|this,_,w,cx|this.expand(w,cx))).into_any_element()
                    } else {
                        IconButton::new("model-preview-expand", IconName::OpenInFull)
                            .icon_size(IconSize::Small).icon_color(Color::Accent).size(ButtonSize::Compact)
                            .tooltip(Tooltip::text(strings.three_d_inspector_expand()))
                            .on_click(cx.listener(|this,_,w,cx|this.expand(w,cx))).into_any_element()
                    }),
            )
            .child(
                frame
                    .child(div().absolute().inset_0().child(view))
                    .when_some(error.filter(|_| !has_frame), |this, error| {
                        this.child(error_layer(error, true, window, cx))
                    }),
            )
            .children(motion::controls("preview-motion", &self.model, cx))
            .into_any_element()
    }
}
