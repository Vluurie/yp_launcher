use std::path::{Path, PathBuf};

use gpui::{
    App, Context, Entity, EventEmitter, IntoElement, Pixels, Render, SharedString, Size,
    Subscription, Task, Window, div, prelude::*, px,
};
use yp_builders::outfit::existing_sibling;
use yp_state::inspector::{ArchivePaths, Camera, InspectorError, ModelMesh, ProbeWarning};
use yp_state::prelude::*;
use yp_state::{ModelEvent, ModelSession};
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, Sizes};

const MAX_RENDER_WIDTH: f32 = 960.;
const MAX_RENDER_SCALE: f32 = 2.;
const MIN_RENDER_SIDE: f32 = 16.;

pub fn archives_for(path: &Path) -> ArchivePaths {
    let is_dat = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("dat"));
    if is_dat {
        ArchivePaths::new(Some(path.to_path_buf()), existing_sibling(path, ".dtt"))
    } else {
        ArchivePaths::new(existing_sibling(path, ".dat"), Some(path.to_path_buf()))
    }
}

pub fn render_size(logical: Size<Pixels>) -> (u32, u32) {
    let width = f32::from(logical.width).max(MIN_RENDER_SIDE);
    let height = f32::from(logical.height).max(MIN_RENDER_SIDE);
    let scale = MAX_RENDER_SCALE.min(MAX_RENDER_WIDTH / width);
    (
        (width * scale).round().max(MIN_RENDER_SIDE) as u32,
        (height * scale).round().max(MIN_RENDER_SIDE) as u32,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelProblem {
    NoModel,
    Unreadable(String),
}

pub fn probe_problem(warnings: &[ProbeWarning]) -> ModelProblem {
    if warnings.is_empty() {
        return ModelProblem::NoModel;
    }
    let lines: Vec<String> = warnings
        .iter()
        .map(|warning| match warning {
            ProbeWarning::ArchiveUnreadable { path, error, .. } => {
                format!("{}: {error}", path.display())
            }
            ProbeWarning::EntryUnreadable { entry, error, .. } => format!("{entry}: {error}"),
        })
        .collect();
    ModelProblem::Unreadable(lines.join("\n"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelViewEvent {
    Ready,
    Failed,
}

pub struct ModelView {
    source: PathBuf,
    session: Option<Entity<ModelSession>>,
    opening: bool,
    problem: Option<ModelProblem>,
    failure_text: Option<SharedString>,
    measured: Option<(u32, u32)>,
    open_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ModelViewEvent> for ModelView {}

impl ModelView {
    pub fn new(source: PathBuf, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            source,
            session: None,
            opening: true,
            problem: None,
            failure_text: None,
            measured: None,
            open_task: None,
            _subscriptions: Vec::new(),
        };
        this.open(cx);
        this
    }

    pub fn failure_text(mut self, text: impl Into<SharedString>) -> Self {
        self.failure_text = Some(text.into());
        self
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn session(&self) -> Option<&Entity<ModelSession>> {
        self.session.as_ref()
    }

    pub fn is_opening(&self) -> bool {
        self.opening
    }

    pub fn problem(&self) -> Option<&ModelProblem> {
        self.problem.as_ref()
    }

    pub fn meshes(&self, cx: &App) -> Vec<ModelMesh> {
        self.session
            .as_ref()
            .map(|session| session.read(cx).meshes().to_vec())
            .unwrap_or_default()
    }

    pub fn measured(&self) -> Option<(u32, u32)> {
        self.measured
    }

    pub fn set_measured(&mut self, width: u32, height: u32, cx: &mut Context<Self>) {
        self.measured = Some((width, height));
        self.apply_measured(cx);
    }

    fn apply_measured(&self, cx: &mut Context<Self>) {
        let (Some((width, height)), Some(session)) = (self.measured, self.session.clone()) else {
            return;
        };
        if session.read(cx).viewport() != (width, height) {
            session.update(cx, |session, cx| session.set_viewport_size(width, height, cx));
        }
    }

    fn fail(&mut self, problem: ModelProblem, cx: &mut Context<Self>) {
        self.opening = false;
        self.problem = Some(problem);
        cx.emit(ModelViewEvent::Failed);
        cx.notify();
    }

    fn open(&mut self, cx: &mut Context<Self>) {
        let source = self.source.clone();
        let inspector = cx.stores().inspector.clone();
        let resolve = cx.background_spawn(async move { archives_for(&source) });
        self.open_task = Some(cx.spawn(async move |this, cx| {
            let archives = resolve.await;
            let probing = inspector.update(cx, |inspector, cx| {
                inspector.probe(archives.clone(), cx)
            });
            let probe = probing.await;
            this.update(cx, |this, cx| match probe {
                Ok(probe) => match probe.candidates.first() {
                    Some(candidate) => {
                        let session = inspector.update(cx, |inspector, cx| {
                            inspector.open_session(archives, candidate.id, cx)
                        });
                        session.update(cx, |session, cx| session.set_camera(Camera::PREVIEW, cx));
                        this._subscriptions = vec![cx.subscribe(&session, Self::on_model_event)];
                        this.session = Some(session);
                        this.apply_measured(cx);
                        cx.notify();
                    }
                    None => this.fail(probe_problem(&probe.warnings), cx),
                },
                Err(InspectorError::NoReadableArchive { warnings }) => {
                    this.fail(probe_problem(&warnings), cx);
                }
                Err(error) => this.fail(ModelProblem::Unreadable(error.to_string()), cx),
            })
            .ok();
        }));
    }

    fn on_model_event(
        &mut self,
        _: Entity<ModelSession>,
        event: &ModelEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ModelEvent::Ready => {
                self.opening = false;
                self.problem = None;
                cx.emit(ModelViewEvent::Ready);
            }
            ModelEvent::Failed(error) | ModelEvent::RenderFailed(error) => {
                self.opening = false;
                self.problem = Some(ModelProblem::Unreadable(error.to_string()));
                cx.emit(ModelViewEvent::Failed);
            }
            ModelEvent::FrameReady => {}
        }
        cx.notify();
    }

    fn problem_text(&self, cx: &App) -> Option<SharedString> {
        let problem = self.problem.as_ref()?;
        if let Some(text) = &self.failure_text {
            return Some(text.clone());
        }
        let strings = cx.t();
        Some(match problem {
            ModelProblem::NoModel => {
                SharedString::new_static(strings.outfit_builder_no_model_in_archive())
            }
            ModelProblem::Unreadable(detail) => {
                SharedString::from(strings.outfit_builder_preview_failed(detail))
            }
        })
    }
}

impl Render for ModelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let colors = cx.theme().colors;
        let session = self.session.clone();
        let frame = session
            .as_ref()
            .and_then(|session| session.read(cx).latest_frame());
        let loading = self.opening
            || session
                .as_ref()
                .is_some_and(|session| session.read(cx).is_loading());
        let problem = self.problem_text(cx).filter(|_| frame.is_none());
        let orbit_session = session.clone();
        let zoom_session = session.clone();
        let entity = cx.entity();
        v_flex()
            .relative()
            .size_full()
            .overflow_hidden()
            .rounded(sizes.border_radius)
            .border_1()
            .border_color(colors.border_light)
            .child(
                FrameViewport::new(("docs-model-viewport", cx.entity_id()))
                    .frame(frame)
                    .loading(loading)
                    .on_resize(move |size, _, cx| {
                        let (width, height) = render_size(*size);
                        entity.update(cx, |this, cx| this.set_measured(width, height, cx));
                    })
                    .when_some(orbit_session, |this, session| {
                        this.on_orbit(move |delta, _, cx| {
                            session.update(cx, |session, cx| {
                                session.orbit(f32::from(delta.x), f32::from(delta.y), cx);
                            });
                        })
                    })
                    .when_some(zoom_session, |this, session| {
                        this.on_zoom(move |delta, _, cx| {
                            session.update(cx, |session, cx| session.zoom(*delta, cx));
                        })
                    })
                    .when_some(problem, |this, text| {
                        this.overlay(
                            div()
                                .absolute()
                                .inset_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .p(sizes.padding_sm)
                                .child(
                                    Label::new(text)
                                        .size(LabelSize::XSmall)
                                        .color(Color::OnDark)
                                        .line_height(1.4)
                                        .centered(),
                                ),
                        )
                    }),
            )
            .min_h(px(f32::from(Sizes::PREVIEW_ROW_HEIGHT)))
    }
}
