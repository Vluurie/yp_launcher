use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use gpui::AppContext as _;
use gpui::{Context, Entity, EventEmitter, RenderImage, Task};
use yp_inspector::cpu::RenderFrame;
use yp_inspector::{Session, probe};

pub use yp_inspector::{
    ArchivePaths, ArchiveProbe, ArchiveSide, Camera, CandidateId, InspectorError, MeshSelection,
    ModelCandidate, ModelMesh, ModelSummary, ProbeWarning, RenderStyle,
};

use crate::cell::Guarded;

pub const DEFAULT_VIEWPORT: (u32, u32) = (640, 480);

#[derive(Debug, Clone)]
pub enum InspectorEvent {
    Probed(ArchivePaths),
}

#[derive(Default)]
pub struct InspectorService {
    probes: HashMap<ArchivePaths, ArchiveProbe>,
}

impl EventEmitter<InspectorEvent> for InspectorService {}

impl InspectorService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self::default()
    }

    pub fn cached(&self, archives: &ArchivePaths) -> Option<&ArchiveProbe> {
        self.probes.get(archives)
    }

    pub fn probe(
        &mut self,
        archives: ArchivePaths,
        cx: &mut Context<Self>,
    ) -> Task<Result<ArchiveProbe, InspectorError>> {
        if let Some(found) = self.probes.get(&archives) {
            return Task::ready(Ok(found.clone()));
        }
        let paths = archives.clone();
        let job = cx.background_spawn(async move { probe(&paths) });
        cx.spawn(async move |this, cx| {
            let result = job.await;
            if let Ok(found) = &result {
                this.update(cx, |this, cx| {
                    this.probes.insert(archives.clone(), found.clone());
                    cx.emit(InspectorEvent::Probed(archives));
                    cx.notify();
                })
                .ok();
            }
            result
        })
    }

    pub fn forget(&mut self, archives: &ArchivePaths, cx: &mut Context<Self>) {
        if self.probes.remove(archives).is_some() {
            cx.notify();
        }
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.probes.clear();
        cx.notify();
    }

    pub fn open_session(
        &mut self,
        archives: ArchivePaths,
        candidate: CandidateId,
        cx: &mut Context<Self>,
    ) -> Entity<ModelSession> {
        cx.new(|cx| ModelSession::open(archives, candidate, cx))
    }
}

#[derive(Debug, Clone)]
pub enum ModelStatus {
    Loading,
    Ready,
    Failed(InspectorError),
}

#[derive(Debug, Clone)]
pub enum ModelEvent {
    Ready,
    Failed(InspectorError),
    FrameReady,
    RenderFailed(InspectorError),
}

pub struct ModelSession {
    cell: Guarded<Option<Session>>,
    status: ModelStatus,
    summary: Option<ModelSummary>,
    meshes: Vec<ModelMesh>,
    selected_lod: u32,
    camera: Camera,
    style: RenderStyle,
    viewport: (u32, u32),
    frame: Option<Arc<RenderImage>>,
    retired: Option<Arc<RenderImage>>,
    rendering: bool,
    render_stale: bool,
    open_task: Option<Task<()>>,
    render_task: Option<Task<()>>,
}

impl EventEmitter<ModelEvent> for ModelSession {}

struct Opened {
    summary: ModelSummary,
    selected_lod: u32,
}

fn describe(session: &Session) -> Opened {
    Opened {
        summary: session.summary(),
        selected_lod: session.selected_lod(),
    }
}

fn to_image(frame: RenderFrame) -> Option<Arc<RenderImage>> {
    let (width, height) = (frame.width, frame.height);
    let pixels = image::RgbaImage::from_raw(width, height, frame.into_bgra())?;
    Some(Arc::new(RenderImage::new(vec![image::Frame::new(pixels)])))
}

impl ModelSession {
    fn empty(camera: Camera, cx: &mut Context<Self>) -> Self {
        cx.on_release(|this, cx| {
            for image in [this.frame.take(), this.retired.take()]
                .into_iter()
                .flatten()
            {
                cx.drop_image(image, None);
            }
        })
        .detach();
        Self {
            cell: Guarded::new(None, cx.background_executor()),
            status: ModelStatus::Loading,
            summary: None,
            meshes: Vec::new(),
            selected_lod: 0,
            camera,
            style: RenderStyle::default(),
            viewport: DEFAULT_VIEWPORT,
            frame: None,
            retired: None,
            rendering: false,
            render_stale: false,
            open_task: None,
            render_task: None,
        }
    }

    pub fn open(archives: ArchivePaths, candidate: CandidateId, cx: &mut Context<Self>) -> Self {
        let mut this = Self::empty(Camera::default(), cx);
        let job = this.cell.spawn(move |slot| {
            let session = Session::open(&archives, candidate)?;
            let opened = describe(&session);
            *slot = Some(session);
            Ok::<_, InspectorError>(opened)
        });
        this.open_task = Some(cx.spawn(async move |this, cx| {
            let Ok(result) = job.await else {
                return;
            };
            this.update(cx, |this, cx| this.finish_open(result, cx))
                .ok();
        }));
        this
    }

    pub fn from_session(session: Session, cx: &mut Context<Self>) -> Self {
        let mut this = Self::empty(Camera::default(), cx);
        let opened = describe(&session);
        this.cell.edit(move |slot| *slot = Some(session), |_| ());
        this.finish_open(Ok(opened), cx);
        this
    }

    fn finish_open(&mut self, result: Result<Opened, InspectorError>, cx: &mut Context<Self>) {
        match result {
            Ok(opened) => {
                self.meshes = opened.summary.meshes.clone();
                self.selected_lod = opened.selected_lod;
                self.summary = Some(opened.summary);
                self.status = ModelStatus::Ready;
                cx.emit(ModelEvent::Ready);
                self.request_render(cx);
            }
            Err(error) => {
                self.status = ModelStatus::Failed(error.clone());
                cx.emit(ModelEvent::Failed(error));
            }
        }
        cx.notify();
    }

    pub fn status(&self) -> &ModelStatus {
        &self.status
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.status, ModelStatus::Ready)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status, ModelStatus::Loading)
    }

    pub fn error(&self) -> Option<&InspectorError> {
        match &self.status {
            ModelStatus::Failed(error) => Some(error),
            _ => None,
        }
    }

    pub fn summary(&self) -> Option<&ModelSummary> {
        self.summary.as_ref()
    }

    pub fn meshes(&self) -> &[ModelMesh] {
        &self.meshes
    }

    pub fn lods(&self) -> &[String] {
        self.summary
            .as_ref()
            .map(|summary| summary.lod_names.as_slice())
            .unwrap_or_default()
    }

    pub fn selected_lod(&self) -> u32 {
        self.selected_lod
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn viewport(&self) -> (u32, u32) {
        self.viewport
    }

    pub fn latest_frame(&self) -> Option<Arc<RenderImage>> {
        self.frame.clone()
    }

    pub fn is_rendering(&self) -> bool {
        self.rendering
    }

    pub fn orbit(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        self.camera.orbit(dx, dy);
        self.request_render(cx);
    }

    pub fn pan(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        self.camera.pan(dx, dy);
        self.request_render(cx);
    }

    pub fn zoom(&mut self, scroll_dy: f32, cx: &mut Context<Self>) {
        self.camera.zoom(scroll_dy);
        self.request_render(cx);
    }

    pub fn set_camera(&mut self, camera: Camera, cx: &mut Context<Self>) {
        self.camera = camera;
        self.request_render(cx);
    }

    pub fn set_style(&mut self, style: RenderStyle, cx: &mut Context<Self>) {
        if self.style != style {
            self.style = style;
            self.request_render(cx);
        }
    }

    pub fn set_viewport_size(&mut self, width: u32, height: u32, cx: &mut Context<Self>) {
        if self.viewport != (width, height) {
            self.viewport = (width, height);
            self.request_render(cx);
        }
    }

    pub fn set_mesh_visibility(&mut self, mesh_id: u32, visible: bool, cx: &mut Context<Self>) {
        let Some(mesh) = self.meshes.iter_mut().find(|mesh| mesh.id == mesh_id) else {
            return;
        };
        mesh.visible = visible;
        self.cell.edit(
            move |slot| {
                if let Some(session) = slot {
                    let _ = session.set_mesh_visibility(mesh_id, visible);
                }
            },
            |_| (),
        );
        cx.notify();
        self.request_render(cx);
    }

    pub fn set_lod(&mut self, lod_index: u32, cx: &mut Context<Self>) {
        if lod_index as usize >= self.lods().len() {
            return;
        }
        self.selected_lod = lod_index;
        self.cell.edit(
            move |slot| {
                if let Some(session) = slot {
                    let _ = session.set_lod(lod_index);
                }
            },
            |_| (),
        );
        cx.notify();
        self.request_render(cx);
    }

    pub fn show_only(&mut self, visible: BTreeSet<u32>, cx: &mut Context<Self>) {
        for mesh in &mut self.meshes {
            mesh.visible = visible.contains(&mesh.id);
        }
        self.cell.edit(
            move |slot| {
                if let Some(session) = slot {
                    session.show_only(&visible);
                }
            },
            |_| (),
        );
        cx.notify();
        self.request_render(cx);
    }

    pub fn apply_preset(&mut self, selection: MeshSelection, cx: &mut Context<Self>) {
        let job = self.cell.spawn(move |slot| {
            slot.as_mut().map(|session| {
                session.apply_preset(&selection);
                session.meshes()
            })
        });
        cx.spawn(async move |this, cx| {
            let Ok(Some(meshes)) = job.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.meshes = meshes;
                cx.notify();
                this.request_render(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn request_render(&mut self, cx: &mut Context<Self>) {
        if !self.is_ready() {
            return;
        }
        if self.rendering {
            self.render_stale = true;
            return;
        }
        self.rendering = true;
        self.render_stale = false;
        let camera = self.camera;
        let style = self.style;
        let (width, height) = self.viewport;
        let job = self.cell.spawn(move |slot| {
            let session = slot.as_ref()?;
            Some(session.render(&camera, width, height, &style).map(to_image))
        });
        cx.notify();
        self.render_task = Some(cx.spawn(async move |this, cx| {
            let outcome = job.await;
            this.update(cx, |this, cx| {
                this.rendering = false;
                match outcome {
                    Ok(Some(Ok(Some(image)))) => this.present(image, cx),
                    Ok(Some(Err(error))) => cx.emit(ModelEvent::RenderFailed(error)),
                    _ => {}
                }
                if this.render_stale {
                    this.request_render(cx);
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn present(&mut self, image: Arc<RenderImage>, cx: &mut Context<Self>) {
        if let Some(retired) = self.retired.take() {
            cx.drop_image(retired, None);
        }
        self.retired = self.frame.replace(image);
        cx.emit(ModelEvent::FrameReady);
    }
}
