use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use yp_formats::wmb3::{self, Model, TextureFiles};
use yp_formats::{ArchiveEntry, DatArchive, FormatError};

use crate::cpu::{self, Camera, RenderFrame, RenderStyle};
use crate::error::{InspectorError, Result};
use crate::preset::{self, MeshSelection};

const PLAYER_STEMS: [&str; 3] = ["pl000", "pl010", "pl020"];
const PLAYER_HIDDEN_PARTS: [&str; 4] = ["armor", "serious", "broken", "dlc"];
const SCREEN_TEXTURE_SUFFIX: &str = "scr";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArchiveSide {
    Dat,
    Dtt,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ArchivePaths {
    pub dat: Option<PathBuf>,
    pub dtt: Option<PathBuf>,
}

impl ArchivePaths {
    pub fn new(dat: Option<PathBuf>, dtt: Option<PathBuf>) -> Self {
        Self { dat, dtt }
    }

    pub fn path(&self, side: ArchiveSide) -> Option<&Path> {
        match side {
            ArchiveSide::Dat => self.dat.as_deref(),
            ArchiveSide::Dtt => self.dtt.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CandidateId {
    pub side: ArchiveSide,
    pub entry_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelCandidate {
    pub id: CandidateId,
    pub name: String,
    pub has_textures: bool,
}

#[derive(Debug, Clone)]
pub enum ProbeWarning {
    ArchiveUnreadable {
        side: ArchiveSide,
        path: PathBuf,
        error: Arc<FormatError>,
    },
    EntryUnreadable {
        side: ArchiveSide,
        entry: String,
        error: Arc<FormatError>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct ArchiveProbe {
    pub candidates: Vec<ModelCandidate>,
    pub warnings: Vec<ProbeWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelMesh {
    pub id: u32,
    pub name: String,
    pub lod_name: String,
    pub lod_index: u32,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSummary {
    pub vertex_count: u64,
    pub triangle_count: u64,
    pub lod_names: Vec<String>,
    pub meshes: Vec<ModelMesh>,
}

type OpenArchive = DatArchive<BufReader<File>>;

fn open_side(
    paths: &ArchivePaths,
    side: ArchiveSide,
) -> Option<(PathBuf, std::result::Result<OpenArchive, FormatError>)> {
    let path = paths.path(side)?;
    Some((path.to_path_buf(), DatArchive::open(path)))
}

pub fn probe(paths: &ArchivePaths) -> Result<ArchiveProbe> {
    if paths.dat.is_none() && paths.dtt.is_none() {
        return Err(InspectorError::NoArchives);
    }
    let mut archives = Vec::new();
    let mut warnings = Vec::new();
    for side in [ArchiveSide::Dat, ArchiveSide::Dtt] {
        match open_side(paths, side) {
            Some((_, Ok(archive))) => archives.push((side, archive)),
            Some((path, Err(error))) => warnings.push(ProbeWarning::ArchiveUnreadable {
                side,
                path,
                error: Arc::new(error),
            }),
            None => {}
        }
    }
    if archives.is_empty() {
        return Err(InspectorError::NoReadableArchive { warnings });
    }

    let all_names: HashSet<String> = archives
        .iter()
        .flat_map(|(_, archive)| archive.entries().iter())
        .map(|entry| entry.name.to_ascii_lowercase())
        .collect();
    let mut candidates = Vec::new();
    for (side, archive) in &mut archives {
        let entries = archive.entries().to_vec();
        for entry in entries.iter().filter(|entry| entry.has_extension("wmb")) {
            match archive.read_entry_prefix(entry, 8) {
                Ok(header) if wmb3::is_game_layout(&header) => candidates.push(ModelCandidate {
                    id: CandidateId {
                        side: *side,
                        entry_index: entry.index,
                    },
                    name: entry.name.clone(),
                    has_textures: has_texture_pair(&all_names, &entry_stem(entry)),
                }),
                Ok(_) => {}
                Err(error) => warnings.push(ProbeWarning::EntryUnreadable {
                    side: *side,
                    entry: entry.name.clone(),
                    error: Arc::new(error),
                }),
            }
        }
    }
    Ok(ArchiveProbe {
        candidates,
        warnings,
    })
}

fn entry_stem(entry: &ArchiveEntry) -> String {
    entry.stem().to_ascii_lowercase()
}

fn has_texture_pair(names: &HashSet<String>, stem: &str) -> bool {
    let has = |extension: &str| names.contains(&format!("{stem}.{extension}"));
    has("wtb") || (has("wta") && has("wtp"))
}

fn read_named<R: Read + Seek>(
    archive: &mut Option<DatArchive<R>>,
    name: &str,
) -> Result<Option<Vec<u8>>> {
    match archive.as_mut() {
        Some(archive) => Ok(archive.read_named(name)?),
        None => Ok(None),
    }
}

fn load_texture_files<R: Read + Seek>(
    dat: &mut Option<DatArchive<R>>,
    dtt: &mut Option<DatArchive<R>>,
    stem: &str,
) -> Result<Option<TextureFiles>> {
    for candidate in [stem.to_string(), format!("{stem}{SCREEN_TEXTURE_SUFFIX}")] {
        let wtb_name = format!("{candidate}.wtb");
        let in_dtt = read_named(dtt, &wtb_name)?;
        let in_dat = read_named(dat, &wtb_name)?;
        if let Some(wtb) = in_dtt.or(in_dat) {
            return Ok(Some(TextureFiles::combined(wtb)));
        }
        let Some(index) = read_named(dat, &format!("{candidate}.wta"))? else {
            continue;
        };
        let Some(payload) = read_named(dtt, &format!("{candidate}.wtp"))? else {
            continue;
        };
        return Ok(Some(TextureFiles { index, payload }));
    }
    Ok(None)
}

pub fn is_player_stem(stem: &str) -> bool {
    PLAYER_STEMS.iter().any(|prefix| stem.starts_with(prefix))
}

pub fn hidden_on_player_by_default(mesh_name: &str) -> bool {
    let lower = mesh_name.to_ascii_lowercase();
    PLAYER_HIDDEN_PARTS.iter().any(|part| lower.contains(part))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    model: Model,
    visible: Vec<bool>,
    default_visible: BTreeSet<u32>,
    selected_lod: u32,
}

impl Session {
    pub fn open(paths: &ArchivePaths, candidate: CandidateId) -> Result<Self> {
        let mut dat = paths.dat.as_deref().map(DatArchive::open).transpose()?;
        let mut dtt = paths.dtt.as_deref().map(DatArchive::open).transpose()?;
        let archive = match candidate.side {
            ArchiveSide::Dat => dat.as_mut(),
            ArchiveSide::Dtt => dtt.as_mut(),
        }
        .ok_or(InspectorError::MissingArchive(candidate.side))?;
        let entry =
            archive
                .entry(candidate.entry_index)
                .cloned()
                .ok_or(InspectorError::EntryMissing {
                    side: candidate.side,
                    index: candidate.entry_index,
                })?;
        let bytes = archive.read_entry(&entry)?;
        let stem = entry_stem(&entry);
        let textures = load_texture_files(&mut dat, &mut dtt, &stem)?;
        let model = Model::parse(&bytes, textures.as_ref())?;
        Ok(Self::from_model(model, is_player_stem(&stem)))
    }

    pub fn from_model(model: Model, is_player: bool) -> Self {
        let visible: Vec<bool> = model
            .meshes
            .iter()
            .map(|mesh| !is_player || !hidden_on_player_by_default(&mesh.name))
            .collect();
        let default_visible = model
            .meshes
            .iter()
            .zip(&visible)
            .filter(|(_, shown)| **shown)
            .map(|(mesh, _)| mesh.id)
            .collect();
        Self {
            model,
            visible,
            default_visible,
            selected_lod: 0,
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn summary(&self) -> ModelSummary {
        ModelSummary {
            vertex_count: self.model.vertex_count() as u64,
            triangle_count: self.model.triangle_count() as u64,
            lod_names: self.model.lod_names.clone(),
            meshes: self.meshes(),
        }
    }

    pub fn meshes(&self) -> Vec<ModelMesh> {
        self.model
            .meshes
            .iter()
            .zip(&self.visible)
            .map(|(mesh, visible)| ModelMesh {
                id: mesh.id,
                name: mesh.name.clone(),
                lod_name: mesh.lod_name.clone(),
                lod_index: mesh.lod_index,
                visible: *visible,
            })
            .collect()
    }

    pub fn selected_lod(&self) -> u32 {
        self.selected_lod
    }

    pub fn default_visible_meshes(&self) -> &BTreeSet<u32> {
        &self.default_visible
    }

    pub fn visible_meshes(&self) -> BTreeSet<u32> {
        self.model
            .meshes
            .iter()
            .zip(&self.visible)
            .filter(|(_, shown)| **shown)
            .map(|(mesh, _)| mesh.id)
            .collect()
    }

    pub fn is_mesh_visible(&self, mesh_id: u32) -> Option<bool> {
        let position = self.position_of(mesh_id)?;
        Some(self.visible[position])
    }

    fn position_of(&self, mesh_id: u32) -> Option<usize> {
        self.model.meshes.iter().position(|mesh| mesh.id == mesh_id)
    }

    pub fn set_mesh_visibility(&mut self, mesh_id: u32, visible: bool) -> Result<()> {
        let position = self
            .position_of(mesh_id)
            .ok_or(InspectorError::MeshMissing(mesh_id))?;
        self.visible[position] = visible;
        Ok(())
    }

    pub fn set_lod(&mut self, lod_index: u32) -> Result<()> {
        if lod_index as usize >= self.model.lod_names.len() {
            return Err(InspectorError::LodMissing(lod_index));
        }
        self.selected_lod = lod_index;
        Ok(())
    }

    pub fn show_only(&mut self, target: &BTreeSet<u32>) -> Vec<u32> {
        let mut changed = Vec::new();
        for (mesh, visible) in self.model.meshes.iter().zip(self.visible.iter_mut()) {
            let wanted = target.contains(&mesh.id);
            if *visible != wanted {
                *visible = wanted;
                changed.push(mesh.id);
            }
        }
        changed
    }

    pub fn apply_preset(&mut self, selection: &MeshSelection) -> BTreeSet<u32> {
        let target = preset::target_meshes(&self.meshes(), selection, &self.default_visible);
        self.show_only(&target);
        target
    }

    pub fn render(
        &self,
        camera: &Camera,
        width: u32,
        height: u32,
        style: &RenderStyle,
    ) -> Result<RenderFrame> {
        cpu::render(
            &self.model,
            &self.visible,
            self.selected_lod,
            camera,
            width,
            height,
            style,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_detection_and_default_hidden_parts() {
        assert!(is_player_stem("pl0000"));
        assert!(is_player_stem("pl0100"));
        assert!(is_player_stem("pl020f"));
        assert!(!is_player_stem("pl0300"));
        assert!(!is_player_stem("wp0003"));
        assert!(hidden_on_player_by_default("Armor_Top"));
        assert!(hidden_on_player_by_default("body_SERIOUS"));
        assert!(hidden_on_player_by_default("BrokenSkirt"));
        assert!(hidden_on_player_by_default("DLC_Hair"));
        assert!(!hidden_on_player_by_default("Body"));
    }

    #[test]
    fn texture_pairs_need_wtb_or_both_halves() {
        let names: HashSet<String> = ["a.wta", "a.wtp", "b.wta", "c.wtb"]
            .iter()
            .map(|name| name.to_string())
            .collect();
        assert!(has_texture_pair(&names, "a"));
        assert!(!has_texture_pair(&names, "b"));
        assert!(has_texture_pair(&names, "c"));
        assert!(!has_texture_pair(&names, "d"));
    }
}
