use std::collections::HashMap;
use std::path::{Path, PathBuf};

use indexmap::IndexSet;

use crate::fsx;
use crate::model::DataArchivePair;
use crate::text::utf16_cmp;
use crate::toml_doc::read_values;

pub const STATE_SELF_DESTRUCTED: &str = "self_destructed";
pub const STATE_EYEMASK: &str = "eyemask";
pub const STATE_COMBAT: &str = "combat";
pub const STATE_PROLOGUE: &str = "prologue";
pub const STATE_HOLY: &str = "holy";
pub const STATE_NO_RIGHT: &str = "no_right";
pub const STATE_TOWER: &str = "tower";
pub const STATE_WIG: &str = "wig";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Character {
    TwoB,
    NineS,
    A2,
}

impl Character {
    pub fn for_stem(stem: &str) -> Option<Character> {
        let normalized = stem.to_lowercase();
        if normalized.starts_with("pl000") {
            Some(Character::TwoB)
        } else if normalized.starts_with("pl020") {
            Some(Character::NineS)
        } else if normalized.starts_with("pl010") {
            Some(Character::A2)
        } else {
            None
        }
    }

    pub fn config_name(self) -> &'static str {
        match self {
            Character::TwoB => "2B",
            Character::NineS => "9S",
            Character::A2 => "A2",
        }
    }

    fn wax_list_key(self) -> &'static str {
        match self {
            Character::TwoB => "pl_2b_outfits",
            Character::NineS => "pl_9s_outfits",
            Character::A2 => "pl_a2_outfits",
        }
    }

    fn override_model(self) -> &'static str {
        match self {
            Character::TwoB => "pl2b",
            Character::NineS => "pl9s",
            Character::A2 => "pla2",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InspectorMeshState {
    pub id: String,
    pub active_show_meshes: Vec<String>,
    pub active_hide_meshes: Vec<String>,
    pub inactive_show_meshes: Vec<String>,
    pub inactive_hide_meshes: Vec<String>,
    pub replaces_base: bool,
}

impl InspectorMeshState {
    pub fn has_meshes(&self) -> bool {
        !self.active_show_meshes.is_empty()
            || !self.active_hide_meshes.is_empty()
            || !self.inactive_show_meshes.is_empty()
            || !self.inactive_hide_meshes.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InspectorOutfitPreset {
    pub id: String,
    pub outfit_id: i64,
    pub name: String,
    pub custom_assembly: bool,
    pub base_meshes: Vec<String>,
    pub states: Vec<InspectorMeshState>,
    pub force_show_meshes: Vec<String>,
    pub force_hide_meshes: Vec<String>,
}

fn remove_all(set: &mut IndexSet<String>, items: &[String]) {
    for item in items {
        set.shift_remove(item);
    }
}

impl InspectorOutfitPreset {
    pub fn visible_mesh_names(&self, state_values: &HashMap<String, bool>) -> IndexSet<String> {
        let mut visible: IndexSet<String> = self.base_meshes.iter().cloned().collect();
        for state in &self.states {
            if state_values.get(&state.id).copied().unwrap_or(false) {
                if state.replaces_base {
                    remove_all(&mut visible, &self.base_meshes);
                }
                remove_all(&mut visible, &state.active_hide_meshes);
                visible.extend(state.active_show_meshes.iter().cloned());
            } else {
                remove_all(&mut visible, &state.inactive_hide_meshes);
                visible.extend(state.inactive_show_meshes.iter().cloned());
            }
        }
        remove_all(&mut visible, &self.force_hide_meshes);
        visible.extend(self.force_show_meshes.iter().cloned());
        visible
    }

    pub fn configured_mesh_names(&self) -> IndexSet<String> {
        let mut out: IndexSet<String> = self.base_meshes.iter().cloned().collect();
        for state in &self.states {
            out.extend(state.active_show_meshes.iter().cloned());
        }
        for state in &self.states {
            out.extend(state.active_hide_meshes.iter().cloned());
        }
        for state in &self.states {
            out.extend(state.inactive_show_meshes.iter().cloned());
        }
        for state in &self.states {
            out.extend(state.inactive_hide_meshes.iter().cloned());
        }
        out.extend(self.force_show_meshes.iter().cloned());
        out.extend(self.force_hide_meshes.iter().cloned());
        out
    }
}

trait PresetFields {
    fn string_list(&self, key: &str) -> Vec<String>;
    fn int(&self, key: &str) -> Option<i64>;
    fn string(&self, key: &str) -> Option<&str>;
}

impl PresetFields for toml::Table {
    fn string_list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .and_then(toml::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .filter(|item| !item.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(toml::Value::as_integer)
    }

    fn string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(toml::Value::as_str)
    }
}

impl PresetFields for serde_json::Map<String, serde_json::Value> {
    fn string_list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .filter(|item| !item.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(serde_json::Value::as_i64)
    }

    fn string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(serde_json::Value::as_str)
    }
}

const MESH_LIST_FIELDS: [&str; 15] = [
    "meshes",
    "self_destruct_show_meshes",
    "self_destruct_hide_meshes",
    "combat_meshes",
    "non_combat_meshes",
    "eyemask_meshes",
    "non_eyemask_meshes",
    "wig_meshes",
    "hairspray_meshes",
    "broken_meshes",
    "non_broken_meshes",
    "prologue_meshes",
    "holy_meshes",
    "no_right_meshes",
    "tower_meshes",
];

fn has_any_mesh_list(values: &dyn PresetFields) -> bool {
    MESH_LIST_FIELDS
        .iter()
        .any(|field| !values.string_list(field).is_empty())
}

fn concat(parts: &[Vec<String>]) -> Vec<String> {
    parts.iter().flatten().cloned().collect()
}

fn read_states(values: &dyn PresetFields) -> Vec<InspectorMeshState> {
    let combat = values.string_list("combat_meshes");
    let non_combat = values.string_list("non_combat_meshes");
    let non_eyemask = values.string_list("non_eyemask_meshes");
    let serious_eyelash: Vec<String> = non_eyemask
        .iter()
        .chain(combat.iter())
        .chain(non_combat.iter())
        .find(|name| name.ends_with("Eyelash") && !name.ends_with("Eyelash_serious"))
        .map(|first| vec![format!("{first}_serious")])
        .unwrap_or_default();
    let replacing = |id: &str, key: &str| InspectorMeshState {
        id: id.to_owned(),
        active_show_meshes: values.string_list(key),
        replaces_base: true,
        ..InspectorMeshState::default()
    };
    [
        replacing(STATE_PROLOGUE, "prologue_meshes"),
        replacing(STATE_HOLY, "holy_meshes"),
        replacing(STATE_NO_RIGHT, "no_right_meshes"),
        replacing(STATE_TOWER, "tower_meshes"),
        InspectorMeshState {
            id: STATE_SELF_DESTRUCTED.to_owned(),
            active_show_meshes: concat(&[
                values.string_list("broken_meshes"),
                values.string_list("self_destruct_show_meshes"),
            ]),
            active_hide_meshes: values.string_list("self_destruct_hide_meshes"),
            inactive_show_meshes: values.string_list("non_broken_meshes"),
            ..InspectorMeshState::default()
        },
        InspectorMeshState {
            id: STATE_EYEMASK.to_owned(),
            active_show_meshes: values.string_list("eyemask_meshes"),
            inactive_show_meshes: non_eyemask.clone(),
            ..InspectorMeshState::default()
        },
        InspectorMeshState {
            id: STATE_COMBAT.to_owned(),
            active_show_meshes: concat(&[combat.clone(), serious_eyelash.clone()]),
            active_hide_meshes: non_combat.clone(),
            inactive_show_meshes: non_combat.clone(),
            inactive_hide_meshes: concat(&[combat, serious_eyelash]),
            ..InspectorMeshState::default()
        },
        InspectorMeshState {
            id: STATE_WIG.to_owned(),
            active_hide_meshes: values.string_list("wig_meshes"),
            ..InspectorMeshState::default()
        },
    ]
    .into_iter()
    .filter(InspectorMeshState::has_meshes)
    .collect()
}

struct MeshOverride {
    mesh: String,
    show: bool,
    outfit_id: Option<i64>,
}

fn read_overrides(values: &toml::Table, character: Character) -> Vec<MeshOverride> {
    let Some(entries) = values.get("mesh_override").and_then(toml::Value::as_array) else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(toml::Value::as_table)
        .filter_map(|entry| {
            let model = entry.string("model")?;
            let mesh = entry.string("mesh").filter(|mesh| !mesh.is_empty())?;
            let force = entry.string("force")?;
            if model != character.override_model() || (force != "show" && force != "hide") {
                return None;
            }
            Some(MeshOverride {
                mesh: mesh.to_owned(),
                show: force == "show",
                outfit_id: entry.int("outfit_id"),
            })
        })
        .collect()
}

fn path_sort(files: &mut [PathBuf]) {
    files.sort_by(|a, b| utf16_cmp(&a.to_string_lossy(), &b.to_string_lossy()));
}

fn load_native(root: &Path, character: Character) -> Vec<InspectorOutfitPreset> {
    let entities = root.join("entities");
    if !entities.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = fsx::walk_files(&entities)
        .into_iter()
        .filter(|file| fsx::lower_path_ends_with(file, ".toml"))
        .collect();
    path_sort(&mut files);
    let parsed: Vec<(PathBuf, toml::Table)> = files
        .into_iter()
        .map(|file| {
            let values = yp_core::fs::read_text_opt(&file)
                .ok()
                .flatten()
                .map(|text| read_values(&text))
                .unwrap_or_default();
            (file, values)
        })
        .collect();
    let overrides: Vec<MeshOverride> = parsed
        .iter()
        .flat_map(|(_, values)| read_overrides(values, character))
        .collect();

    let mut presets = Vec::new();
    for (file, values) in &parsed {
        if !fsx::file_name(file).to_lowercase().starts_with("outfit_") {
            continue;
        }
        if values.string("character") != Some(character.config_name()) {
            continue;
        }
        let Some(outfit_id) = values.int("outfit_id") else {
            continue;
        };
        let matching: Vec<&MeshOverride> = overrides
            .iter()
            .filter(|item| item.outfit_id.is_none_or(|id| id == outfit_id))
            .collect();
        presets.push(InspectorOutfitPreset {
            id: format!("{}|{outfit_id}", file.to_string_lossy()),
            outfit_id,
            name: values.string("name").unwrap_or_default().to_owned(),
            custom_assembly: has_any_mesh_list(values),
            base_meshes: values.string_list("meshes"),
            states: read_states(values),
            force_show_meshes: matching
                .iter()
                .filter(|item| item.show)
                .map(|item| item.mesh.clone())
                .collect(),
            force_hide_meshes: matching
                .iter()
                .filter(|item| !item.show)
                .map(|item| item.mesh.clone())
                .collect(),
        });
    }
    presets
}

fn load_wax(root: &Path, character: Character) -> Vec<InspectorOutfitPreset> {
    let list_key = character.wax_list_key();
    let mut files: Vec<PathBuf> = fsx::walk_files(root)
        .into_iter()
        .filter(|file| fsx::lower_path_ends_with(file, ".json"))
        .filter(|file| {
            file.components()
                .any(|part| part.as_os_str().to_string_lossy().to_lowercase() == "outfits")
        })
        .collect();
    path_sort(&mut files);

    let mut presets = Vec::new();
    for file in files {
        let Some(serde_json::Value::Object(decoded)) = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        else {
            continue;
        };
        let Some(outfits) = decoded.get(list_key).and_then(serde_json::Value::as_array) else {
            continue;
        };
        for (entry_index, raw) in outfits.iter().enumerate() {
            let Some(values) = raw.as_object() else {
                continue;
            };
            let Some(outfit_id) = values.int("outfit_id") else {
                continue;
            };
            presets.push(InspectorOutfitPreset {
                id: format!("{}|{outfit_id}|{entry_index}", file.to_string_lossy()),
                outfit_id,
                name: values.string("outfit_name").unwrap_or_default().to_owned(),
                custom_assembly: has_any_mesh_list(values),
                base_meshes: values.string_list("meshes"),
                states: read_states(values),
                ..InspectorOutfitPreset::default()
            });
        }
    }
    presets
}

pub fn load(archive: &DataArchivePair) -> Vec<InspectorOutfitPreset> {
    let Some(character) = Character::for_stem(&archive.stem) else {
        return Vec::new();
    };
    load_character(&archive.config_root, character)
}

pub fn load_character(root: &Path, character: Character) -> Vec<InspectorOutfitPreset> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut presets = load_native(root, character);
    presets.extend(load_wax(root, character));
    presets.sort_by(|a, b| {
        a.outfit_id
            .cmp(&b.outfit_id)
            .then_with(|| utf16_cmp(&a.name, &b.name))
    });
    presets
}

struct VanillaVariant {
    id: &'static str,
    outfit_id: i64,
    meshes: &'static [&'static str],
}

const fn vanilla(
    id: &'static str,
    outfit_id: i64,
    meshes: &'static [&'static str],
) -> VanillaVariant {
    VanillaVariant {
        id,
        outfit_id,
        meshes,
    }
}

const VANILLA_2B: [VanillaVariant; 6] = [
    vanilla(
        "vanilla:normal",
        -1,
        &[
            "Body",
            "Eyelash",
            "Eyemask",
            "facial_normal",
            "facial_serious",
            "Feather",
            "Hair",
            "Skirt",
        ],
    ),
    vanilla(
        "vanilla:damaged",
        -2,
        &[
            "Broken",
            "Body",
            "Eyelash",
            "Eyemask",
            "facial_normal",
            "facial_serious",
            "Feather",
            "Hair",
        ],
    ),
    vanilla(
        "vanilla:base",
        -3,
        &[
            "Body",
            "Eyelash",
            "Eyemask",
            "facial_normal",
            "facial_serious",
            "Hair",
        ],
    ),
    vanilla("vanilla:armor", -4, &["Armor_Body", "Armor_Head"]),
    vanilla(
        "vanilla:dlc",
        -5,
        &[
            "DLC_Body",
            "DLC_Skirt",
            "Eyelash",
            "facial_normal",
            "facial_serious",
            "Hair",
        ],
    ),
    vanilla(
        "vanilla:dlc_damaged",
        -6,
        &[
            "DLC_Body",
            "DLC_Broken",
            "Eyelash",
            "facial_normal",
            "facial_serious",
            "Hair",
        ],
    ),
];

const VANILLA_9S: [VanillaVariant; 11] = [
    vanilla(
        "vanilla:normal",
        -1,
        &[
            "Body",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "Leg",
            "Pants",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
            "mesh_pl0200",
        ],
    ),
    vanilla(
        "vanilla:self_destruct",
        -2,
        &[
            "Body",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "Leg",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
            "mesh_pl0200",
        ],
    ),
    vanilla(
        "vanilla:damaged_left",
        -3,
        &[
            "Body",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "Leg",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
            "mesh_es0200",
        ],
    ),
    vanilla(
        "vanilla:damaged_right",
        -4,
        &[
            "Body",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "Leg",
            "Pants",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
            "mesh_es0206",
        ],
    ),
    vanilla(
        "vanilla:damaged_holes",
        -5,
        &[
            "Body",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "Leg",
            "Pants",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
            "mesh_es0201",
        ],
    ),
    vanilla(
        "vanilla:damaged_2b_hand",
        -6,
        &[
            "Body",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "Leg",
            "Pants",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
            "mesh_es0202",
        ],
    ),
    vanilla(
        "vanilla:dlc",
        -7,
        &[
            "DLC_Body",
            "DLC_Leg",
            "DLC_Pants",
            "DLC_mesh_pl0200",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
        ],
    ),
    vanilla(
        "vanilla:dlc_damaged_left",
        -8,
        &[
            "DLC_Body",
            "DLC_Leg",
            "DLC_mesh_es0200",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
        ],
    ),
    vanilla(
        "vanilla:dlc_damaged_right",
        -9,
        &[
            "DLC_Body",
            "DLC_Leg",
            "DLC_Pants",
            "DLC_mesh_es0206",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
        ],
    ),
    vanilla(
        "vanilla:dlc_damaged_holes",
        -10,
        &[
            "DLC_Body",
            "DLC_Leg",
            "DLC_Pants",
            "DLC_mesh_es0201",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
        ],
    ),
    vanilla(
        "vanilla:dlc_damaged_2b_hand",
        -11,
        &[
            "DLC_Body",
            "DLC_Leg",
            "DLC_Pants",
            "DLC_mesh_es0202",
            "Eyelash",
            "Eyelash_serious",
            "Eyemask",
            "facial_normal",
            "facial_normal-2",
            "facial_serious",
            "facial_serious-2",
        ],
    ),
];

const VANILLA_A2: [VanillaVariant; 3] = [
    vanilla(
        "vanilla:normal",
        -1,
        &["Body", "Cloth", "Hair", "facial_normal", "facial_serious"],
    ),
    vanilla(
        "vanilla:berserk",
        -2,
        &["Body", "Hair", "facial_normal", "facial_serious"],
    ),
    vanilla(
        "vanilla:dlc",
        -3,
        &[
            "DLC_Body",
            "DLC_Cloth",
            "Hair",
            "facial_normal",
            "facial_serious",
        ],
    ),
];

fn vanilla_variants(character: Character) -> &'static [VanillaVariant] {
    match character {
        Character::TwoB => &VANILLA_2B,
        Character::NineS => &VANILLA_9S,
        Character::A2 => &VANILLA_A2,
    }
}

pub fn infer_vanilla_variants<I, S>(archive_stem: &str, mesh_names: I) -> Vec<InspectorOutfitPreset>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let available: IndexSet<String> = mesh_names
        .into_iter()
        .map(|name| name.as_ref().to_owned())
        .collect();
    let Some(character) = Character::for_stem(archive_stem) else {
        return Vec::new();
    };
    let mut variants = Vec::new();
    let mut seen: IndexSet<Vec<String>> = IndexSet::new();
    for definition in vanilla_variants(character) {
        let meshes: IndexSet<&str> = definition
            .meshes
            .iter()
            .copied()
            .filter(|mesh| available.contains(*mesh))
            .collect();
        if meshes.is_empty() {
            continue;
        }
        let mut sorted: Vec<String> = meshes.iter().map(|mesh| (*mesh).to_owned()).collect();
        sorted.sort_by(|a, b| utf16_cmp(a, b));
        if !seen.insert(sorted.clone()) {
            continue;
        }
        variants.push(InspectorOutfitPreset {
            id: definition.id.to_owned(),
            outfit_id: definition.outfit_id,
            name: String::new(),
            custom_assembly: true,
            base_meshes: sorted,
            ..InspectorOutfitPreset::default()
        });
    }
    variants
}
