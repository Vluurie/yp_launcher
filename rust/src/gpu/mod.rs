//! The renderer implementation was inspired by RaiderB's rusty_platinum_utils.

use crate::model::{Scene, Texture};
use std::collections::HashMap;
use std::sync::mpsc::{self, Sender, SyncSender};
use std::sync::{Arc, OnceLock};
use three_d::*;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

const MAX_PIXELS: usize = 2_073_600;

pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

struct SceneData {
    meshes: Vec<MeshData>,
    textures: HashMap<u32, Texture>,
    center: [f32; 3],
    radius: f32,
    selected_lod: u32,
}

struct MeshData {
    id: u32,
    lod_index: u32,
    enabled: bool,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
    albedo_texture_id: Option<u32>,
    normal_texture_id: Option<u32>,
}

impl From<&Scene> for SceneData {
    fn from(scene: &Scene) -> Self {
        Self {
            meshes: scene
                .meshes
                .iter()
                .map(|mesh| MeshData {
                    id: mesh.id,
                    lod_index: mesh.lod_index,
                    enabled: mesh.enabled,
                    positions: mesh.positions.clone(),
                    normals: mesh.normals.clone(),
                    tangents: mesh.tangents.clone(),
                    uvs: mesh.uvs.clone(),
                    indices: mesh.indices.clone(),
                    albedo_texture_id: mesh.albedo_texture_id,
                    normal_texture_id: mesh.normal_texture_id,
                })
                .collect(),
            textures: scene.textures.clone(),
            center: scene.center,
            radius: scene.radius,
            selected_lod: scene.selected_lod,
        }
    }
}

enum Command {
    Upload {
        session_id: u64,
        scene: SceneData,
        response: SyncSender<Result<(), String>>,
    },
    Render {
        session_id: u64,
        width: u32,
        height: u32,
        yaw: f32,
        pitch: f32,
        distance: f32,
        pan_x: f32,
        pan_y: f32,
        response: SyncSender<Result<Frame, String>>,
    },
    SetMeshVisibility {
        session_id: u64,
        mesh_id: u32,
        visible: bool,
        response: SyncSender<Result<(), String>>,
    },
    SetLod {
        session_id: u64,
        lod_index: u32,
        response: SyncSender<Result<(), String>>,
    },
    Remove {
        session_id: u64,
    },
}

struct GpuRenderer {
    sender: Sender<Command>,
}

struct GpuContext {
    context: WindowedContext,
    _window: Window,
    _event_loop: EventLoop<()>,
}

impl std::ops::Deref for GpuContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

struct ModelInfo {
    id: u32,
    lod_index: u32,
    enabled: bool,
    model: Gm<Mesh, DeferredPhysicalMaterial>,
}

struct GpuScene {
    models: Vec<ModelInfo>,
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    center: Vec3,
    radius: f32,
    selected_lod: u32,
}

impl GpuScene {
    fn new(context: &Context, data: SceneData) -> Result<Self, String> {
        let mut texture_handles = HashMap::new();
        let mut models = Vec::with_capacity(data.meshes.len());
        for mesh in data.meshes {
            let cpu_mesh = CpuMesh {
                positions: Positions::F32(
                    mesh.positions
                        .into_iter()
                        .map(|value| vec3(value[0], value[1], value[2]))
                        .collect(),
                ),
                indices: Indices::U32(mesh.indices),
                normals: Some(
                    mesh.normals
                        .into_iter()
                        .map(|value| vec3(value[0], value[1], value[2]))
                        .collect(),
                ),
                tangents: Some(
                    mesh.tangents
                        .into_iter()
                        .map(|value| vec4(value[0], value[1], value[2], value[3]))
                        .collect(),
                ),
                uvs: Some(
                    mesh.uvs
                        .into_iter()
                        .map(|value| vec2(value[0], value[1]))
                        .collect(),
                ),
                ..Default::default()
            };
            let material = DeferredPhysicalMaterial {
                albedo_texture: texture_ref(
                    context,
                    mesh.albedo_texture_id,
                    &data.textures,
                    &mut texture_handles,
                ),
                normal_texture: texture_ref(
                    context,
                    mesh.normal_texture_id,
                    &data.textures,
                    &mut texture_handles,
                ),
                alpha_cutout: Some(0.5),
                ..Default::default()
            };
            models.push(ModelInfo {
                id: mesh.id,
                lod_index: mesh.lod_index,
                enabled: mesh.enabled,
                model: Gm::new(Mesh::new(context, &cpu_mesh), material),
            });
        }
        Ok(Self {
            models,
            ambient_light: AmbientLight::new(context, 0.3, Srgba::WHITE),
            directional_light: DirectionalLight::new(
                context,
                3.0,
                Srgba::WHITE,
                vec3(-1.0, -1.0, -1.0),
            ),
            center: vec3(data.center[0], data.center[1], data.center[2]),
            radius: data.radius.max(f32::EPSILON),
            selected_lod: data.selected_lod,
        })
    }

    fn render(
        &mut self,
        context: &Context,
        width: u32,
        height: u32,
        yaw: f32,
        pitch: f32,
        distance: f32,
        pan_x: f32,
        pan_y: f32,
    ) -> Result<Frame, String> {
        if width < 16 || height < 16 {
            return Err("Render size is too small".to_string());
        }
        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or_else(|| "Render size overflowed".to_string())?;
        if pixel_count > MAX_PIXELS {
            return Err("Render size is too large".to_string());
        }
        let distance = distance.clamp(0.9, 12.0);
        let pitch = pitch.clamp(-1.45, 1.45);
        let camera_distance = self.radius * distance;
        let direction = vec3(
            yaw.sin() * pitch.cos(),
            -pitch.sin(),
            yaw.cos() * pitch.cos(),
        );
        let mut camera = Camera::new_perspective(
            Viewport::new_at_origo(width, height),
            self.center + direction * camera_distance,
            self.center,
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            (camera_distance / 4000.0).max(0.0001),
            (camera_distance * 4.0).max(1.0),
        );
        let scale = 2.0 * self.radius / width.min(height) as f32;
        let offset =
            camera.right_direction() * (-pan_x * scale) + camera.up_orthogonal() * (pan_y * scale);
        camera.translate(offset);
        let visible_models = self
            .models
            .iter()
            .filter(|model| model.enabled && model.lod_index == self.selected_lod)
            .map(|model| &model.model)
            .collect::<Vec<_>>();
        let texture = Texture2D::new_empty::<[u8; 4]>(
            context,
            width,
            height,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let depth_texture = DepthTexture2D::new::<f32>(
            context,
            width,
            height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        self.directional_light
            .generate_shadow_map(2048, &visible_models)
            .map_err(|error| error.to_string())?;
        let render_target = RenderTarget::new(
            texture.as_color_target(None),
            depth_texture.as_depth_target(),
        );
        let lights: [&dyn Light; 2] = [&self.ambient_light, &self.directional_light];
        let rgba = render_target
            .clear(ClearState::color_and_depth(
                25.0 / 255.0,
                27.0 / 255.0,
                26.0 / 255.0,
                1.0,
                1.0,
            ))
            .render(&camera, &visible_models, &lights)
            .read_color::<[u8; 4]>()
            .into_flattened();
        context.error_check().map_err(|error| error.to_string())?;
        Ok(Frame {
            width,
            height,
            rgba,
        })
    }
}

fn texture_ref(
    context: &Context,
    texture_id: Option<u32>,
    textures: &HashMap<u32, Texture>,
    handles: &mut HashMap<u32, Arc<Texture2D>>,
) -> Option<Texture2DRef> {
    let texture_id = texture_id?;
    if !handles.contains_key(&texture_id) {
        let source = textures.get(&texture_id)?;
        let mut cpu_texture = CpuTexture {
            name: texture_id.to_string(),
            data: TextureData::RgbaU8(
                source
                    .rgba
                    .chunks_exact(4)
                    .map(|value| [value[0], value[1], value[2], value[3]])
                    .collect(),
            ),
            width: source.width,
            height: source.height,
            min_filter: Interpolation::Linear,
            mag_filter: Interpolation::Linear,
            mipmap: Some(Mipmap {
                filter: Interpolation::Linear,
                max_levels: 4,
                max_ratio: 4,
            }),
            wrap_s: Wrapping::Repeat,
            wrap_t: Wrapping::Repeat,
        };
        cpu_texture.data.to_linear_srgb();
        handles.insert(texture_id, Arc::new(Texture2D::new(context, &cpu_texture)));
    }
    handles.get(&texture_id).map(|texture| Texture2DRef {
        texture: texture.clone(),
        transformation: Mat3::identity(),
    })
}

fn renderer() -> Option<&'static GpuRenderer> {
    static RENDERER: OnceLock<Option<GpuRenderer>> = OnceLock::new();
    RENDERER
        .get_or_init(|| {
            let (sender, receiver) = mpsc::channel();
            let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
            if std::thread::Builder::new()
                .name("yp-3d-inspector-gpu".to_string())
                .spawn(move || {
                    let context = match create_context() {
                        Ok(context) => context,
                        Err(error) => {
                            let _ = ready_sender.send(Err(error.to_string()));
                            return;
                        }
                    };
                    if ready_sender.send(Ok(())).is_err() {
                        return;
                    }
                    let mut scenes = HashMap::new();
                    while let Ok(command) = receiver.recv() {
                        handle_command(&context, &mut scenes, command);
                    }
                })
                .is_err()
            {
                return None;
            }
            match ready_receiver.recv() {
                Ok(Ok(())) => Some(GpuRenderer { sender }),
                _ => None,
            }
        })
        .as_ref()
}

#[cfg(target_os = "windows")]
fn build_event_loop() -> EventLoop<()> {
    use winit::platform::windows::EventLoopBuilderExtWindows;
    winit::event_loop::EventLoopBuilder::new()
        .with_any_thread(true)
        .build()
}

#[cfg(not(target_os = "windows"))]
fn build_event_loop() -> EventLoop<()> {
    EventLoop::new()
}

fn create_context() -> Result<GpuContext, String> {
    match std::panic::catch_unwind(|| {
        let event_loop = build_event_loop();
        let window = WindowBuilder::new()
            .with_visible(false)
            .build(&event_loop)
            .map_err(|error| error.to_string())?;
        let context = WindowedContext::from_winit_window(
            &window,
            SurfaceSettings {
                vsync: false,
                ..Default::default()
            },
        )
        .map_err(|error| error.to_string())?;
        Ok::<_, String>(GpuContext {
            context,
            _window: window,
            _event_loop: event_loop,
        })
    }) {
        Ok(result) => result,
        Err(_) => Err("GPU window context creation failed".to_string()),
    }
}

fn handle_command(
    context: &Context,
    scenes: &mut HashMap<u64, GpuScene>,
    command: Command,
) {
    match command {
        Command::Upload {
            session_id,
            scene,
            response,
        } => {
            let result =
                GpuScene::new(context, scene).map(|scene| scenes.insert(session_id, scene));
            let _ = response.send(result.map(|_| ()));
        }
        Command::Render {
            session_id,
            width,
            height,
            yaw,
            pitch,
            distance,
            pan_x,
            pan_y,
            response,
        } => {
            let result = scenes
                .get_mut(&session_id)
                .ok_or_else(|| "GPU model session is closed".to_string())
                .and_then(|scene| {
                    scene.render(context, width, height, yaw, pitch, distance, pan_x, pan_y)
                });
            let _ = response.send(result);
        }
        Command::SetMeshVisibility {
            session_id,
            mesh_id,
            visible,
            response,
        } => {
            let result = scenes
                .get_mut(&session_id)
                .ok_or_else(|| "GPU model session is closed".to_string())
                .and_then(|scene| {
                    scene
                        .models
                        .iter_mut()
                        .find(|model| model.id == mesh_id)
                        .ok_or_else(|| "GPU model mesh no longer exists".to_string())
                        .map(|model| model.enabled = visible)
                });
            let _ = response.send(result);
        }
        Command::SetLod {
            session_id,
            lod_index,
            response,
        } => {
            let result = scenes
                .get_mut(&session_id)
                .ok_or_else(|| "GPU model session is closed".to_string())
                .map(|scene| scene.selected_lod = lod_index);
            let _ = response.send(result);
        }
        Command::Remove { session_id } => {
            scenes.remove(&session_id);
        }
    }
}

pub fn upload(session_id: u64, scene: &Scene) -> Result<(), String> {
    request(|response| Command::Upload {
        session_id,
        scene: SceneData::from(scene),
        response,
    })
}

pub fn render(
    session_id: u64,
    width: u32,
    height: u32,
    yaw: f32,
    pitch: f32,
    distance: f32,
    pan_x: f32,
    pan_y: f32,
) -> Result<Frame, String> {
    request(|response| Command::Render {
        session_id,
        width,
        height,
        yaw,
        pitch,
        distance,
        pan_x,
        pan_y,
        response,
    })
}

pub fn set_mesh_visibility(
    session_id: u64,
    mesh_id: u32,
    visible: bool,
) -> Result<(), String> {
    request(|response| Command::SetMeshVisibility {
        session_id,
        mesh_id,
        visible,
        response,
    })
}

pub fn set_lod(session_id: u64, lod_index: u32) -> Result<(), String> {
    request(|response| Command::SetLod {
        session_id,
        lod_index,
        response,
    })
}

pub fn remove(session_id: u64) {
    if let Some(renderer) = renderer() {
        let _ = renderer.sender.send(Command::Remove { session_id });
    }
}

fn request<T>(
    make_command: impl FnOnce(SyncSender<Result<T, String>>) -> Command,
) -> Result<T, String> {
    let renderer = renderer().ok_or_else(|| "GPU renderer is unavailable".to_string())?;
    let (response_sender, response_receiver) = mpsc::sync_channel(1);
    renderer
        .sender
        .send(make_command(response_sender))
        .map_err(|_| "GPU renderer stopped".to_string())?;
    response_receiver
        .recv()
        .map_err(|_| "GPU renderer stopped".to_string())?
}
