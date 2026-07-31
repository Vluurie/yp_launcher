use crate::archive::{ArchiveEntry, DatArchive};
use crate::gpu;
use crate::model::{Scene, TextureFiles};
use crate::render;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone)]
pub struct ArchiveProbe {
    pub candidates: Vec<ModelCandidate>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ModelCandidate {
    pub id: String,
    pub name: String,
    pub archive: String,
    pub entry_index: u32,
    pub has_textures: bool,
}

#[derive(Clone, Debug)]
pub struct ModelSession {
    pub id: u64,
    pub vertex_count: u64,
    pub triangle_count: u64,
    pub lod_names: Vec<String>,
    pub meshes: Vec<ModelMesh>,
}

#[derive(Clone, Debug)]
pub struct ModelMesh {
    pub id: u32,
    pub name: String,
    pub lod_name: String,
    pub lod_index: u32,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub struct RenderFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
static SESSIONS: OnceLock<Mutex<HashMap<u64, Arc<Mutex<SessionState>>>>> = OnceLock::new();

struct SessionState {
    scene: Scene,
    gpu_active: bool,
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn probe_archive_pair(
    dat_path: Option<String>,
    dtt_path: Option<String>,
) -> Result<ArchiveProbe, String> {
    if dat_path.is_none() && dtt_path.is_none() {
        return Err("No DAT or DTT path was provided".to_string());
    }

    let mut archives = Vec::new();
    let mut warnings = Vec::new();

    if let Some(path) = dat_path {
        match DatArchive::open(PathBuf::from(&path)) {
            Ok(archive) => archives.push(("dat".to_string(), archive)),
            Err(error) => warnings.push(format!("{}: {}", path, error)),
        }
    }

    if let Some(path) = dtt_path {
        match DatArchive::open(PathBuf::from(&path)) {
            Ok(archive) => archives.push(("dtt".to_string(), archive)),
            Err(error) => warnings.push(format!("{}: {}", path, error)),
        }
    }

    if archives.is_empty() {
        return Err(warnings.join("\n"));
    }

    let all_entry_names = archives
        .iter()
        .flat_map(|(_, archive)| archive.entries().iter())
        .map(|entry| entry.name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let mut candidates = Vec::new();

    for (archive_kind, archive) in &mut archives {
        let entries = archive.entries().to_vec();
        for entry in entries {
            if !entry.name.to_ascii_lowercase().ends_with(".wmb") {
                continue;
            }

            match archive.read_entry_prefix(&entry, 4) {
                Ok(magic) if magic == b"WMB3" => {
                    let stem = entry_stem(&entry);
                    candidates.push(ModelCandidate {
                        id: format!("{}:{}", archive_kind, entry.index),
                        name: entry.name.clone(),
                        archive: archive_kind.clone(),
                        entry_index: entry.index,
                        has_textures: has_texture_pair(&all_entry_names, &stem),
                    });
                }
                Ok(_) => {}
                Err(error) => warnings.push(format!("{}: {}", entry.name, error)),
            }
        }
    }

    Ok(ArchiveProbe {
        candidates,
        warnings,
    })
}

pub fn open_model(
    dat_path: Option<String>,
    dtt_path: Option<String>,
    candidate_id: String,
) -> Result<ModelSession, String> {
    let (archive_kind, entry_index) = candidate_id
        .split_once(':')
        .ok_or_else(|| "Model candidate identifier is invalid".to_string())?;
    let entry_index = entry_index
        .parse::<u32>()
        .map_err(|_| "Model candidate entry index is invalid".to_string())?;
    let mut dat_archive = dat_path
        .map(PathBuf::from)
        .map(DatArchive::open)
        .transpose()?;
    let mut dtt_archive = dtt_path
        .map(PathBuf::from)
        .map(DatArchive::open)
        .transpose()?;
    let archive = match archive_kind {
        "dat" => dat_archive.as_mut(),
        "dtt" => dtt_archive.as_mut(),
        _ => return Err("Model candidate archive is invalid".to_string()),
    }
    .ok_or_else(|| "The model archive path is missing".to_string())?;
    let entry = archive
        .entries()
        .iter()
        .find(|entry| entry.index == entry_index)
        .cloned()
        .ok_or_else(|| "The model entry no longer exists".to_string())?;
    let bytes = archive.read_entry(&entry)?;
    let stem = entry_stem(&entry);
    let texture_files = load_texture_files(&mut dat_archive, &mut dtt_archive, &stem)?;
    let is_player = ["pl000", "pl010", "pl020"]
        .iter()
        .any(|prefix| stem.starts_with(prefix));
    let scene = Scene::parse(&bytes, texture_files, is_player)?;
    let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    let vertex_count = scene.vertex_count() as u64;
    let triangle_count = scene.triangle_count() as u64;
    let lod_names = scene.lod_names.clone();
    let meshes = scene
        .meshes
        .iter()
        .map(|mesh| ModelMesh {
            id: mesh.id,
            name: mesh.name.clone(),
            lod_name: mesh.lod_name.clone(),
            lod_index: mesh.lod_index,
            visible: mesh.enabled,
        })
        .collect();
    let gpu_active = gpu::upload(session_id, &scene).is_ok();
    sessions()
        .lock()
        .map_err(|_| "Model session storage is unavailable".to_string())?
        .insert(
            session_id,
            Arc::new(Mutex::new(SessionState { scene, gpu_active })),
        );
    Ok(ModelSession {
        id: session_id,
        vertex_count,
        triangle_count,
        lod_names,
        meshes,
    })
}

pub fn render_model(
    session_id: u64,
    width: u32,
    height: u32,
    yaw: f32,
    pitch: f32,
    distance: f32,
    pan_x: f32,
    pan_y: f32,
) -> Result<RenderFrame, String> {
    let scene = sessions()
        .lock()
        .map_err(|_| "Model session storage is unavailable".to_string())?
        .get(&session_id)
        .cloned()
        .ok_or_else(|| "Model session is closed".to_string())?;
    let mut state = scene
        .lock()
        .map_err(|_| "Model scene is unavailable".to_string())?;
    if state.gpu_active {
        match gpu::render(
            session_id, width, height, yaw, pitch, distance, pan_x, pan_y,
        ) {
            Ok(frame) => {
                return Ok(RenderFrame {
                    width: frame.width,
                    height: frame.height,
                    rgba: frame.rgba,
                });
            }
            Err(_) => state.gpu_active = false,
        }
    }
    let frame = render::render(
        &state.scene,
        width,
        height,
        yaw,
        pitch,
        distance,
        pan_x,
        pan_y,
    )?;
    Ok(RenderFrame {
        width: frame.width,
        height: frame.height,
        rgba: frame.rgba,
    })
}

pub fn set_mesh_visibility(session_id: u64, mesh_id: u32, visible: bool) -> Result<(), String> {
    let scene = sessions()
        .lock()
        .map_err(|_| "Model session storage is unavailable".to_string())?
        .get(&session_id)
        .cloned()
        .ok_or_else(|| "Model session is closed".to_string())?;
    let mut state = scene
        .lock()
        .map_err(|_| "Model scene is unavailable".to_string())?;
    state.scene.set_mesh_visibility(mesh_id, visible)?;
    if state.gpu_active
        && gpu::set_mesh_visibility(session_id, mesh_id, visible).is_err()
    {
        state.gpu_active = false;
    }
    Ok(())
}

pub fn set_model_lod(session_id: u64, lod_index: u32) -> Result<(), String> {
    let scene = sessions()
        .lock()
        .map_err(|_| "Model session storage is unavailable".to_string())?
        .get(&session_id)
        .cloned()
        .ok_or_else(|| "Model session is closed".to_string())?;
    let mut state = scene
        .lock()
        .map_err(|_| "Model scene is unavailable".to_string())?;
    state.scene.set_lod(lod_index)?;
    if state.gpu_active && gpu::set_lod(session_id, lod_index).is_err() {
        state.gpu_active = false;
    }
    Ok(())
}

#[flutter_rust_bridge::frb(sync)]
pub fn close_model(session_id: u64) {
    if let Ok(mut sessions) = sessions().lock() {
        sessions.remove(&session_id);
    }
    gpu::remove(session_id);
}

fn sessions() -> &'static Mutex<HashMap<u64, Arc<Mutex<SessionState>>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn load_texture_files(
    dat_archive: &mut Option<DatArchive>,
    dtt_archive: &mut Option<DatArchive>,
    stem: &str,
) -> Result<Option<TextureFiles>, String> {
    for candidate in [stem.to_string(), format!("{stem}scr")] {
        if let Some(wtb) = read_named(dtt_archive, &format!("{candidate}.wtb"))?
            .or(read_named(dat_archive, &format!("{candidate}.wtb"))?)
        {
            return Ok(Some(TextureFiles {
                index: wtb.clone(),
                payload: wtb,
            }));
        }
        let Some(index) = read_named(dat_archive, &format!("{candidate}.wta"))? else {
            continue;
        };
        let Some(payload) = read_named(dtt_archive, &format!("{candidate}.wtp"))? else {
            continue;
        };
        return Ok(Some(TextureFiles { index, payload }));
    }
    Ok(None)
}

fn read_named(archive: &mut Option<DatArchive>, name: &str) -> Result<Option<Vec<u8>>, String> {
    let Some(archive) = archive.as_mut() else {
        return Ok(None);
    };
    let entry = archive
        .entries()
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(name))
        .cloned();
    entry.map(|entry| archive.read_entry(&entry)).transpose()
}

fn entry_stem(entry: &ArchiveEntry) -> String {
    entry
        .name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(&entry.name)
        .to_ascii_lowercase()
}

fn has_texture_pair(entries: &HashSet<String>, stem: &str) -> bool {
    let wta = format!("{stem}.wta");
    let wtp = format!("{stem}.wtp");
    let wtb = format!("{stem}.wtb");
    let has_wta = entries.contains(&wta);
    let has_wtp = entries.contains(&wtp);
    let has_wtb = entries.contains(&wtb);
    has_wtb || has_wta && has_wtp
}
