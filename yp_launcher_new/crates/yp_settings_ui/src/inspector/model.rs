use std::collections::{BTreeSet, HashMap};

use gpui::{EventEmitter, Subscription, Task};
use yp_i18n::Strings;
use yp_mods::inspector_presets::{
    self, InspectorOutfitPreset, STATE_COMBAT, STATE_EYEMASK, STATE_HOLY, STATE_NO_RIGHT,
    STATE_PROLOGUE, STATE_SELF_DESTRUCTED, STATE_TOWER, STATE_WIG,
};
use yp_mods::{DataArchivePair, InstalledMod};
use yp_state::inspector::{
    ArchivePaths, ArchiveProbe, Camera, InspectorError, MeshSelection, ModelCandidate, ModelMesh,
};
use yp_state::prelude::*;
use yp_state::{InspectorService, ModelEvent, ModelSession};
use yp_ui::prelude::*;

use crate::errors::{inspector_error_text, probe_warning_text};

pub const VANILLA_PREFIX: &str = "vanilla:";
pub const STORY_STATES: [&str; 4] = [STATE_PROLOGUE, STATE_HOLY, STATE_NO_RIGHT, STATE_TOWER];
const MIN_RENDER_SIDE: f32 = 16.;
const PAGE_MAX_SIDE: f32 = 1920.;
const PAGE_MAX_PIXELS: f32 = 2_073_600.;
const PREVIEW_MAX_SCALE: f32 = 2.;
const PREVIEW_TARGET_WIDTH: f32 = 960.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewMode {
    Page,
    Preview,
}

impl ViewMode {
    pub fn camera(self) -> Camera {
        match self {
            ViewMode::Page => Camera::INSPECTOR,
            ViewMode::Preview => Camera::PREVIEW,
        }
    }

    pub fn render_size(self, width: f32, height: f32) -> (u32, u32) {
        let width = width.max(MIN_RENDER_SIDE);
        let height = height.max(MIN_RENDER_SIDE);
        let scale = match self {
            ViewMode::Page => (PAGE_MAX_SIDE / width)
                .min(PAGE_MAX_SIDE / height)
                .min((PAGE_MAX_PIXELS / (width * height)).sqrt())
                .min(1.),
            ViewMode::Preview => (PREVIEW_TARGET_WIDTH / width).min(PREVIEW_MAX_SCALE),
        };
        let side = |value: f32| (value * scale).round().max(MIN_RENDER_SIDE) as u32;
        (side(width), side(height))
    }
}

pub fn archive_pair(archives: &ArchivePaths, mods: &[InstalledMod]) -> DataArchivePair {
    let known = mods
        .iter()
        .filter_map(|installed| installed.data.as_ref())
        .flat_map(|data| data.archives.iter())
        .find(|pair| pair.dat_path == archives.dat && pair.dtt_path == archives.dtt);
    if let Some(pair) = known {
        return pair.clone();
    }
    let source = archives.dat.as_ref().or(archives.dtt.as_ref());
    let stem = source
        .and_then(|path| path.file_stem())
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let label = source
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let config_root = source
        .and_then(|path| path.parent())
        .map(|parent| parent.to_path_buf())
        .unwrap_or_default();
    DataArchivePair {
        stem,
        label,
        config_root,
        dat_path: archives.dat.clone(),
        dtt_path: archives.dtt.clone(),
    }
}

pub fn initial_state_values(preset: &InspectorOutfitPreset) -> HashMap<String, bool> {
    preset
        .states
        .iter()
        .map(|state| (state.id.clone(), state.id == STATE_EYEMASK))
        .collect()
}

pub fn with_state(
    values: &HashMap<String, bool>,
    state_id: &str,
    enabled: bool,
) -> HashMap<String, bool> {
    let mut next = values.clone();
    if enabled && STORY_STATES.contains(&state_id) {
        for story in STORY_STATES {
            if let Some(value) = next.get_mut(story) {
                *value = false;
            }
        }
    }
    next.insert(state_id.to_owned(), enabled);
    next
}

pub fn mesh_selection(
    preset: &InspectorOutfitPreset,
    state_values: &HashMap<String, bool>,
) -> MeshSelection {
    MeshSelection {
        custom_assembly: preset.custom_assembly,
        visible_names: preset.visible_mesh_names(state_values).into_iter().collect(),
        force_hide: preset.force_hide_meshes.iter().cloned().collect(),
        force_show: preset.force_show_meshes.iter().cloned().collect(),
    }
}

pub fn is_vanilla(preset: &InspectorOutfitPreset) -> bool {
    preset.id.starts_with(VANILLA_PREFIX)
}

pub fn preset_index_for(presets: &[InspectorOutfitPreset], outfit_id: Option<i64>) -> Option<usize> {
    if presets.is_empty() {
        return None;
    }
    let wanted = outfit_id.and_then(|id| presets.iter().position(|preset| preset.outfit_id == id));
    Some(wanted.unwrap_or(0))
}

pub fn preset_label(preset: &InspectorOutfitPreset, strings: &'static Strings) -> SharedString {
    let inferred = match preset.id.as_str() {
        "vanilla:normal" => Some(strings.three_d_inspector_variant_normal()),
        "vanilla:damaged" => Some(strings.three_d_inspector_variant_damaged()),
        "vanilla:base" => Some(strings.three_d_inspector_variant_base()),
        "vanilla:self_destruct" => Some(strings.three_d_inspector_variant_self_destruct()),
        "vanilla:damaged_left" => Some(strings.three_d_inspector_variant_damaged_left()),
        "vanilla:damaged_right" => Some(strings.three_d_inspector_variant_damaged_right()),
        "vanilla:damaged_holes" => Some(strings.three_d_inspector_variant_damaged_holes()),
        "vanilla:damaged_2b_hand" => Some(strings.three_d_inspector_variant_damaged2b_hand()),
        "vanilla:armor" => Some(strings.three_d_inspector_variant_armor()),
        "vanilla:dlc" => Some(strings.three_d_inspector_variant_dlc()),
        "vanilla:dlc_damaged" => Some(strings.three_d_inspector_variant_dlc_damaged()),
        "vanilla:dlc_damaged_left" => Some(strings.three_d_inspector_variant_dlc_damaged_left()),
        "vanilla:dlc_damaged_right" => {
            Some(strings.three_d_inspector_variant_dlc_damaged_right())
        }
        "vanilla:dlc_damaged_holes" => {
            Some(strings.three_d_inspector_variant_dlc_damaged_holes())
        }
        "vanilla:dlc_damaged_2b_hand" => {
            Some(strings.three_d_inspector_variant_dlc_damaged2b_hand())
        }
        "vanilla:berserk" => Some(strings.three_d_inspector_variant_berserk()),
        _ => None,
    };
    if let Some(label) = inferred {
        return SharedString::new_static(label);
    }
    if preset.name.is_empty() {
        SharedString::from(strings.three_d_inspector_outfit_number(preset.outfit_id))
    } else {
        SharedString::from(preset.name.clone())
    }
}

pub fn state_label(state_id: &str, strings: &'static Strings) -> SharedString {
    let label = match state_id {
        STATE_SELF_DESTRUCTED => strings.three_d_inspector_state_self_destructed(),
        STATE_EYEMASK => strings.three_d_inspector_state_eyemask(),
        STATE_COMBAT => strings.three_d_inspector_state_combat(),
        STATE_PROLOGUE => strings.three_d_inspector_state_prologue(),
        STATE_HOLY => strings.three_d_inspector_state_holy(),
        STATE_NO_RIGHT => strings.three_d_inspector_state_no_right(),
        STATE_TOWER => strings.three_d_inspector_state_tower(),
        STATE_WIG => strings.three_d_inspector_state_wig(),
        other => return SharedString::from(other.to_owned()),
    };
    SharedString::new_static(label)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshGroup {
    pub name: String,
    pub ids: Vec<u32>,
    pub visible: bool,
}

pub fn mesh_groups(meshes: &[ModelMesh], lod: u32) -> Vec<MeshGroup> {
    let mut groups: Vec<MeshGroup> = Vec::new();
    for mesh in meshes.iter().filter(|mesh| mesh.lod_index == lod) {
        match groups.iter_mut().find(|group| group.name == mesh.name) {
            Some(group) => {
                group.ids.push(mesh.id);
                group.visible &= mesh.visible;
            }
            None => groups.push(MeshGroup {
                name: mesh.name.clone(),
                ids: vec![mesh.id],
                visible: mesh.visible,
            }),
        }
    }
    groups.sort_by_key(|group| group.name.to_lowercase());
    groups
}

pub fn missing_mesh_names(preset: &InspectorOutfitPreset, meshes: &[ModelMesh]) -> Vec<String> {
    let available: BTreeSet<&str> = meshes.iter().map(|mesh| mesh.name.as_str()).collect();
    let mut missing: Vec<String> = preset
        .configured_mesh_names()
        .into_iter()
        .filter(|name| !available.contains(name.as_str()))
        .collect();
    missing.sort();
    missing
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorModelEvent {
    Probed,
    CandidateOpened,
    PresetApplied,
}

pub struct InspectorModel {
    mode: ViewMode,
    style: yp_inspector::RenderStyle,
    archives: ArchivePaths,
    pair: DataArchivePair,
    service: Entity<InspectorService>,
    probing: bool,
    probe: Option<ArchiveProbe>,
    probe_error: Option<InspectorError>,
    render_error: Option<InspectorError>,
    candidate: Option<ModelCandidate>,
    session: Option<Entity<ModelSession>>,
    presets: Vec<InspectorOutfitPreset>,
    selected_preset: Option<usize>,
    state_values: HashMap<String, bool>,
    initial_outfit: Option<i64>,
    viewport: Option<(f32, f32)>,
    probe_task: Option<Task<()>>,
    session_subscription: Option<Subscription>,
}

impl EventEmitter<InspectorModelEvent> for InspectorModel {}

impl InspectorModel {
    pub fn new(
        archives: ArchivePaths,
        mode: ViewMode,
        initial_outfit: Option<i64>,
        cx: &mut Context<Self>,
    ) -> Self {
        let stores = cx.stores().clone();
        let pair = archive_pair(&archives, stores.mods.read(cx).mods());
        let mut this = Self {
            mode,
            style: Default::default(),
            archives,
            pair,
            service: stores.inspector.clone(),
            probing: true,
            probe: None,
            probe_error: None,
            render_error: None,
            candidate: None,
            session: None,
            presets: Vec::new(),
            selected_preset: None,
            state_values: HashMap::new(),
            initial_outfit,
            viewport: None,
            probe_task: None,
            session_subscription: None,
        };
        this.start(cx);
        this
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        let pair = self.pair.clone();
        let presets = cx.background_spawn(async move { inspector_presets::load(&pair) });
        let archives = self.archives.clone();
        let probe = self
            .service
            .update(cx, |service, cx| service.probe(archives, cx));
        self.probe_task = Some(cx.spawn(async move |this, cx| {
            let presets = presets.await;
            let probe = probe.await;
            this.update(cx, |this, cx| this.finish_probe(presets, probe, cx))
                .ok();
        }));
    }

    fn finish_probe(
        &mut self,
        presets: Vec<InspectorOutfitPreset>,
        probe: Result<ArchiveProbe, InspectorError>,
        cx: &mut Context<Self>,
    ) {
        self.probing = false;
        self.presets = presets;
        self.selected_preset = preset_index_for(&self.presets, self.initial_outfit);
        self.refresh_state_values();
        match probe {
            Ok(probe) => {
                let first = probe.candidates.first().cloned();
                self.probe = Some(probe);
                match first {
                    Some(candidate) => self.open_candidate(candidate, cx),
                    None => self.probe_error = Some(InspectorError::NoArchives),
                }
            }
            Err(error) => self.probe_error = Some(error),
        }
        cx.emit(InspectorModelEvent::Probed);
        cx.notify();
    }

    fn refresh_state_values(&mut self) {
        self.state_values = self
            .selected()
            .map(initial_state_values)
            .unwrap_or_default();
    }

    pub fn mode(&self) -> ViewMode {
        self.mode
    }

    pub fn archives(&self) -> &ArchivePaths {
        &self.archives
    }

    pub fn pair(&self) -> &DataArchivePair {
        &self.pair
    }

    pub fn is_probing(&self) -> bool {
        self.probing
    }

    pub fn candidates(&self) -> &[ModelCandidate] {
        self.probe
            .as_ref()
            .map(|probe| probe.candidates.as_slice())
            .unwrap_or_default()
    }

    pub fn candidate(&self) -> Option<&ModelCandidate> {
        self.candidate.as_ref()
    }

    pub fn session(&self) -> Option<&Entity<ModelSession>> {
        self.session.as_ref()
    }

    pub fn presets(&self) -> &[InspectorOutfitPreset] {
        &self.presets
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_preset
    }

    pub fn selected(&self) -> Option<&InspectorOutfitPreset> {
        self.selected_preset
            .and_then(|index| self.presets.get(index))
    }

    pub fn state_values(&self) -> &HashMap<String, bool> {
        &self.state_values
    }

    pub fn is_loading(&self, cx: &App) -> bool {
        self.probing
            || self
                .session
                .as_ref()
                .is_some_and(|session| session.read(cx).is_loading())
    }

    pub fn error_text(&self, cx: &App) -> Option<String> {
        let strings = cx.t();
        if let Some(error) = &self.probe_error {
            return Some(inspector_error_text(error, strings));
        }
        self.session
            .as_ref()
            .and_then(|session| session.read(cx).error().cloned())
            .or_else(|| self.render_error.clone())
            .map(|error| inspector_error_text(&error, strings))
    }

    pub fn warning_lines(&self, cx: &App) -> Vec<SharedString> {
        let strings = cx.t();
        self.probe
            .as_ref()
            .map(|probe| {
                probe
                    .warnings
                    .iter()
                    .map(|warning| SharedString::from(probe_warning_text(warning, strings)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn title(&self) -> SharedString {
        match &self.candidate {
            Some(candidate) => SharedString::from(candidate.name.clone()),
            None => SharedString::from(self.pair.label.clone()),
        }
    }

    pub fn set_style(&mut self, style: yp_inspector::RenderStyle, cx: &mut Context<Self>) {
        self.style = style;
        if let Some(session) = &self.session {
            session.update(cx, |session, cx| session.set_style(style, cx));
        }
    }

    pub fn open_candidate(&mut self, candidate: ModelCandidate, cx: &mut Context<Self>) {
        let archives = self.archives.clone();
        let id = candidate.id;
        let session = self
            .service
            .update(cx, |service, cx| service.open_session(archives, id, cx));
        let camera = self.mode.camera();
        session.update(cx, |session, cx| {
            session.set_style(self.style, cx);
            session.set_camera(camera, cx);
        });
        self.session_subscription =
            Some(cx.subscribe(&session, |this, _, event: &ModelEvent, cx| match event {
                ModelEvent::Ready => this.session_ready(cx),
                ModelEvent::FrameReady => {
                    this.render_error = None;
                    cx.notify();
                }
                ModelEvent::RenderFailed(error) => {
                    this.render_error = Some(error.clone());
                    cx.notify();
                }
                ModelEvent::Failed(_) => cx.notify(),
            }));
        self.render_error = None;
        self.candidate = Some(candidate);
        self.session = Some(session);
        self.apply_viewport(cx);
        cx.emit(InspectorModelEvent::CandidateOpened);
        cx.notify();
    }

    pub fn select_candidate(&mut self, candidate: ModelCandidate, cx: &mut Context<Self>) {
        if self.candidate.as_ref().map(|current| current.id) == Some(candidate.id) {
            return;
        }
        self.open_candidate(candidate, cx);
    }

    fn session_ready(&mut self, cx: &mut Context<Self>) {
        let Some(session) = self.session.clone() else {
            return;
        };
        self.presets.retain(|preset| !is_vanilla(preset));
        if self.presets.is_empty() {
            let names: Vec<String> = session
                .read(cx)
                .meshes()
                .iter()
                .map(|mesh| mesh.name.clone())
                .collect();
            self.presets = inspector_presets::infer_vanilla_variants(&self.pair.stem, names);
            self.selected_preset = preset_index_for(&self.presets, None);
            self.refresh_state_values();
        } else if self.selected().is_none() {
            self.selected_preset = preset_index_for(&self.presets, self.initial_outfit);
            self.refresh_state_values();
        }
        self.apply_preset(cx);
        cx.notify();
    }

    pub fn apply_preset(&mut self, cx: &mut Context<Self>) {
        let (Some(session), Some(preset)) = (self.session.clone(), self.selected()) else {
            return;
        };
        if !session.read(cx).is_ready() {
            return;
        }
        let selection = mesh_selection(preset, &self.state_values);
        session.update(cx, |session, cx| session.apply_preset(selection, cx));
        cx.emit(InspectorModelEvent::PresetApplied);
    }

    pub fn select_preset(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.presets.len() || self.selected_preset == Some(index) {
            return;
        }
        self.selected_preset = Some(index);
        self.refresh_state_values();
        self.apply_preset(cx);
        cx.notify();
    }

    pub fn set_state(&mut self, state_id: &str, enabled: bool, cx: &mut Context<Self>) {
        self.state_values = with_state(&self.state_values, state_id, enabled);
        self.apply_preset(cx);
        cx.notify();
    }

    pub fn set_group_visibility(&mut self, ids: &[u32], visible: bool, cx: &mut Context<Self>) {
        let Some(session) = self.session.clone() else {
            return;
        };
        session.update(cx, |session, cx| {
            for id in ids {
                session.set_mesh_visibility(*id, visible, cx);
            }
        });
        cx.notify();
    }

    pub fn set_lod(&mut self, lod: u32, cx: &mut Context<Self>) {
        if let Some(session) = self.session.clone() {
            session.update(cx, |session, cx| session.set_lod(lod, cx));
            cx.notify();
        }
    }

    pub fn reset_camera(&mut self, cx: &mut Context<Self>) {
        let camera = self.mode.camera();
        if let Some(session) = self.session.clone() {
            session.update(cx, |session, cx| session.set_camera(camera, cx));
        }
    }

    pub fn orbit(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        if let Some(session) = self.session.clone() {
            session.update(cx, |session, cx| session.orbit(dx, dy, cx));
        }
    }

    pub fn pan(&mut self, dx: f32, dy: f32, cx: &mut Context<Self>) {
        let (Some(session), Some((width, height))) = (self.session.clone(), self.viewport) else {
            return;
        };
        let (render_width, render_height) = self.mode.render_size(width, height);
        let scale_x = render_width as f32 / width.max(1.);
        let scale_y = render_height as f32 / height.max(1.);
        session.update(cx, |session, cx| {
            session.pan(dx * scale_x, dy * scale_y, cx)
        });
    }

    pub fn zoom(&mut self, delta: f32, cx: &mut Context<Self>) {
        if let Some(session) = self.session.clone() {
            session.update(cx, |session, cx| session.zoom(delta, cx));
        }
    }

    pub fn viewport(&self) -> Option<(f32, f32)> {
        self.viewport
    }

    pub fn set_viewport(&mut self, width: f32, height: f32, cx: &mut Context<Self>) {
        if self.viewport == Some((width, height)) {
            return;
        }
        self.viewport = Some((width, height));
        self.apply_viewport(cx);
        cx.notify();
    }

    fn apply_viewport(&mut self, cx: &mut Context<Self>) {
        let (Some(session), Some((width, height))) = (self.session.clone(), self.viewport) else {
            return;
        };
        let (render_width, render_height) = self.mode.render_size(width, height);
        session.update(cx, |session, cx| {
            session.set_viewport_size(render_width, render_height, cx)
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use yp_i18n::Locale;
    use yp_mods::inspector_presets::InspectorMeshState;

    use super::*;

    fn mesh(id: u32, name: &str, lod: u32, visible: bool) -> ModelMesh {
        ModelMesh {
            id,
            name: name.to_owned(),
            lod_name: format!("LOD{lod}"),
            lod_index: lod,
            visible,
        }
    }

    fn preset(id: &str, outfit_id: i64) -> InspectorOutfitPreset {
        InspectorOutfitPreset {
            id: id.to_owned(),
            outfit_id,
            ..InspectorOutfitPreset::default()
        }
    }

    fn state(id: &str, show: &[&str]) -> InspectorMeshState {
        InspectorMeshState {
            id: id.to_owned(),
            active_show_meshes: show.iter().map(|name| (*name).to_owned()).collect(),
            ..InspectorMeshState::default()
        }
    }

    #[test]
    fn render_size_is_capped_for_the_page_and_scaled_for_the_preview() {
        assert_eq!(ViewMode::Page.render_size(1284., 742.), (1284, 742));
        assert_eq!(ViewMode::Page.render_size(4., 4.), (16, 16));
        let (width, height) = ViewMode::Page.render_size(3840., 2160.);
        assert!(width <= 1920 && height <= 1920);
        assert!(u64::from(width) * u64::from(height) <= 2_073_600 + 2_000);
        assert_eq!(ViewMode::Preview.render_size(480., 270.), (960, 540));
        assert_eq!(ViewMode::Preview.render_size(1920., 1080.), (960, 540));
        assert_eq!(ViewMode::Preview.render_size(200., 100.), (400, 200));
    }

    #[test]
    fn eyemask_starts_on_and_story_states_exclude_each_other() {
        let mut outfit = preset("outfit:1", 1);
        outfit.states = vec![
            state(STATE_EYEMASK, &[]),
            state(STATE_PROLOGUE, &[]),
            state(STATE_TOWER, &[]),
            state(STATE_COMBAT, &[]),
        ];
        let values = initial_state_values(&outfit);
        assert_eq!(values.get(STATE_EYEMASK), Some(&true));
        assert_eq!(values.get(STATE_PROLOGUE), Some(&false));
        let values = with_state(&values, STATE_PROLOGUE, true);
        let values = with_state(&values, STATE_COMBAT, true);
        let values = with_state(&values, STATE_TOWER, true);
        assert_eq!(values.get(STATE_PROLOGUE), Some(&false));
        assert_eq!(values.get(STATE_TOWER), Some(&true));
        assert_eq!(values.get(STATE_COMBAT), Some(&true));
        assert_eq!(values.get(STATE_EYEMASK), Some(&true));
        let values = with_state(&values, STATE_TOWER, false);
        assert_eq!(values.get(STATE_TOWER), Some(&false));
    }

    #[test]
    fn mesh_selection_follows_the_preset_and_its_states() {
        let mut outfit = preset("outfit:3", 3);
        outfit.custom_assembly = true;
        outfit.base_meshes = vec!["Body".into(), "Hair".into()];
        outfit.states = vec![state(STATE_EYEMASK, &["Eyemask"])];
        outfit.force_hide_meshes = vec!["Hair".into()];
        outfit.force_show_meshes = vec!["Skirt".into()];
        let on = mesh_selection(&outfit, &initial_state_values(&outfit));
        assert!(on.custom_assembly);
        assert_eq!(
            on.visible_names.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["Body", "Eyemask", "Skirt"]
        );
        assert!(on.force_hide.contains("Hair"));
        assert!(on.force_show.contains("Skirt"));
        let off = mesh_selection(&outfit, &HashMap::new());
        assert!(!off.visible_names.contains("Eyemask"));
    }

    #[test]
    fn presets_are_picked_by_outfit_id_with_a_first_entry_fallback() {
        let presets = vec![preset("outfit:1", 1), preset("outfit:7", 7)];
        assert_eq!(preset_index_for(&presets, Some(7)), Some(1));
        assert_eq!(preset_index_for(&presets, Some(99)), Some(0));
        assert_eq!(preset_index_for(&presets, None), Some(0));
        assert_eq!(preset_index_for(&[], Some(1)), None);
        assert!(is_vanilla(&preset("vanilla:normal", 0)));
        assert!(!is_vanilla(&preset("outfit:1", 1)));
    }

    #[test]
    fn labels_come_from_the_translations_or_the_preset_itself() {
        let strings = Strings::for_locale(Locale::En);
        assert_eq!(
            preset_label(&preset("vanilla:dlc_damaged_2b_hand", 0), strings).as_ref(),
            strings.three_d_inspector_variant_dlc_damaged2b_hand()
        );
        let mut named = preset("outfit:4", 4);
        named.name = "Kimono".into();
        assert_eq!(preset_label(&named, strings).as_ref(), "Kimono");
        assert_eq!(
            preset_label(&preset("outfit:4", 4), strings).as_ref(),
            strings.three_d_inspector_outfit_number(4)
        );
        assert_eq!(
            state_label(STATE_WIG, strings).as_ref(),
            strings.three_d_inspector_state_wig()
        );
        assert_eq!(state_label("custom", strings).as_ref(), "custom");
    }

    #[test]
    fn meshes_are_grouped_by_name_for_the_selected_lod() {
        let meshes = vec![
            mesh(0, "body", 0, true),
            mesh(1, "Armor", 0, false),
            mesh(2, "body", 0, false),
            mesh(3, "body", 1, true),
            mesh(4, "Hair", 0, true),
        ];
        let groups = mesh_groups(&meshes, 0);
        assert_eq!(
            groups.iter().map(|group| group.name.as_str()).collect::<Vec<_>>(),
            vec!["Armor", "body", "Hair"]
        );
        assert_eq!(groups[1].ids, vec![0, 2]);
        assert!(!groups[1].visible);
        assert!(groups[2].visible);
        assert_eq!(mesh_groups(&meshes, 1).len(), 1);
    }

    #[test]
    fn missing_meshes_are_the_configured_names_the_model_lacks() {
        let mut outfit = preset("outfit:2", 2);
        outfit.base_meshes = vec!["Body".into(), "Zeta".into(), "Alpha".into()];
        let meshes = vec![mesh(0, "Body", 0, true)];
        assert_eq!(missing_mesh_names(&outfit, &meshes), vec!["Alpha", "Zeta"]);
    }

    #[test]
    fn archive_pair_falls_back_to_the_file_name() {
        let archives = ArchivePaths::new(
            Some(PathBuf::from("mods/outfit/data/pl/pl000d.dat")),
            Some(PathBuf::from("mods/outfit/data/pl/pl000d.dtt")),
        );
        let pair = archive_pair(&archives, &[]);
        assert_eq!(pair.stem, "pl000d");
        assert_eq!(pair.label, "pl000d.dat");
        assert_eq!(pair.dat_path, archives.dat);
        let mut installed = InstalledMod::new("outfit", "mods/outfit", yp_mods::ModKind::Data);
        let known = DataArchivePair {
            stem: "pl000d".into(),
            label: "pl000d (2B)".into(),
            config_root: PathBuf::from("mods/outfit"),
            dat_path: archives.dat.clone(),
            dtt_path: archives.dtt.clone(),
        };
        installed.data = Some(yp_mods::DataSummary {
            archives: vec![known.clone()],
            ..yp_mods::DataSummary::default()
        });
        assert_eq!(archive_pair(&archives, &[installed]).label, "pl000d (2B)");
    }
}
