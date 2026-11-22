use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use indexmap::IndexMap;
use jiff::Timestamp;
use yp_core::{ArchiveFailure, IoOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModKind {
    Native,
    Data,
    Texture,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DataCategory {
    Player,
    Weapon,
    Enemy,
    Accessory,
    Item,
    WorldProp,
    ModelVariant,
    Map,
    Scripting,
    Localization,
    Effects,
    Ui,
    Misc,
    Cutscenes,
    Audio,
    Archive,
    Other,
}

impl DataCategory {
    pub const ALL: [DataCategory; 17] = [
        DataCategory::Player,
        DataCategory::Weapon,
        DataCategory::Enemy,
        DataCategory::Accessory,
        DataCategory::Item,
        DataCategory::WorldProp,
        DataCategory::ModelVariant,
        DataCategory::Map,
        DataCategory::Scripting,
        DataCategory::Localization,
        DataCategory::Effects,
        DataCategory::Ui,
        DataCategory::Misc,
        DataCategory::Cutscenes,
        DataCategory::Audio,
        DataCategory::Archive,
        DataCategory::Other,
    ];

    pub fn name(self) -> &'static str {
        match self {
            DataCategory::Player => "player",
            DataCategory::Weapon => "weapon",
            DataCategory::Enemy => "enemy",
            DataCategory::Accessory => "accessory",
            DataCategory::Item => "item",
            DataCategory::WorldProp => "worldProp",
            DataCategory::ModelVariant => "modelVariant",
            DataCategory::Map => "map",
            DataCategory::Scripting => "scripting",
            DataCategory::Localization => "localization",
            DataCategory::Effects => "effects",
            DataCategory::Ui => "ui",
            DataCategory::Misc => "misc",
            DataCategory::Cutscenes => "cutscenes",
            DataCategory::Audio => "audio",
            DataCategory::Archive => "archive",
            DataCategory::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModConflictKind {
    OverlappingDataFile,
    OutfitIdCollision,
    VanillaOutfitId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManifestInfo {
    pub id: Option<String>,
    pub display_name: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub nams_min_version: Option<String>,
    pub requires: Vec<String>,
    pub requires_plugins: Vec<String>,
    pub replaces: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeSummary {
    pub bundles_by_kind: IndexMap<String, usize>,
    pub total_entity_files: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirEntry {
    pub dir_name: String,
    pub category: DataCategory,
    pub file_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerModelEntry {
    pub file_name: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct DataArchivePair {
    pub stem: String,
    pub label: String,
    pub config_root: PathBuf,
    pub dat_path: Option<PathBuf>,
    pub dtt_path: Option<PathBuf>,
}

impl PartialEq for DataArchivePair {
    fn eq(&self, other: &Self) -> bool {
        self.stem == other.stem
            && self.config_root == other.config_root
            && self.dat_path == other.dat_path
            && self.dtt_path == other.dtt_path
    }
}

impl Eq for DataArchivePair {}

impl Hash for DataArchivePair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stem.hash(state);
        self.config_root.hash(state);
        self.dat_path.hash(state);
        self.dtt_path.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitChoice {
    pub outfit_id: i64,
    pub name: Option<String>,
    pub needs_item: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataSummary {
    pub entries: Vec<DataDirEntry>,
    pub players: Vec<PlayerModelEntry>,
    pub archives: Vec<DataArchivePair>,
    pub has_compat_config: bool,
    pub has_outfit_config: bool,
    pub outfits_by_stem: IndexMap<String, Vec<OutfitChoice>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModConflict {
    pub other_mod_id: String,
    pub kind: ModConflictKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledMod {
    pub id: String,
    pub display_name: String,
    pub root_path: PathBuf,
    pub kind: ModKind,
    pub manifest: Option<ManifestInfo>,
    pub native: Option<NativeSummary>,
    pub data: Option<DataSummary>,
    pub requires_missing: Vec<String>,
    pub conflicts: Vec<ModConflict>,
    pub installed_at: Timestamp,
    pub bundled_texture_packs: Vec<String>,
    pub bundled_cutscenes: Vec<String>,
}

impl InstalledMod {
    pub fn new(id: impl Into<String>, root_path: impl Into<PathBuf>, kind: ModKind) -> Self {
        let id = id.into();
        Self {
            display_name: id.clone(),
            id,
            root_path: root_path.into(),
            kind,
            manifest: None,
            native: None,
            data: None,
            requires_missing: Vec::new(),
            conflicts: Vec::new(),
            installed_at: Timestamp::UNIX_EPOCH,
            bundled_texture_packs: Vec::new(),
            bundled_cutscenes: Vec::new(),
        }
    }

    pub fn has_data_overlay(&self) -> bool {
        self.data
            .as_ref()
            .is_some_and(|data| !data.entries.is_empty())
    }

    pub fn has_warnings(&self) -> bool {
        !self.conflicts.is_empty() || !self.requires_missing.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModVariant {
    pub sub_path: PathBuf,
    pub label: String,
    pub kind: ModKind,
    pub texture_only: bool,
    pub category: Option<DataCategory>,
}

impl ModVariant {
    pub fn is_outfit(&self) -> bool {
        self.category == Some(DataCategory::Player)
    }

    pub fn is_weapon(&self) -> bool {
        self.category == Some(DataCategory::Weapon)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturePack {
    pub path: PathBuf,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariantInstallRequest {
    pub requested_name: String,
    pub variant_sub_path: PathBuf,
    pub texture_pack_sub_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropRejection {
    TextureOnly,
    ArchiveExtractFailed(ArchiveFailure),
    UnknownDrop,
    UnsupportedNasa,
    InvalidMixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedDrop {
    pub unwrapped_root: PathBuf,
    pub kind: ModKind,
    pub manifest: Option<ManifestInfo>,
    pub native: Option<NativeSummary>,
    pub data: Option<DataSummary>,
    pub suggested_id: String,
    pub rejection: Option<DropRejection>,
    pub variants: Vec<ModVariant>,
    pub texture_variants: Vec<TexturePack>,
}

impl DetectedDrop {
    pub fn rejected(
        unwrapped_root: PathBuf,
        suggested_id: String,
        rejection: DropRejection,
    ) -> Self {
        Self {
            unwrapped_root,
            kind: ModKind::Unknown,
            manifest: None,
            native: None,
            data: None,
            suggested_id,
            rejection: Some(rejection),
            variants: Vec::new(),
            texture_variants: Vec::new(),
        }
    }

    pub fn has_variants(&self) -> bool {
        !self.variants.is_empty()
    }

    pub fn has_texture_variants(&self) -> bool {
        self.texture_variants.len() > 1
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Installed {
    pub id: String,
    pub side_installed_texture_packs: Vec<String>,
    pub unpaired_warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallFailure {
    Rejected(DropRejection),
    VariantMissing,
    Exists {
        id: String,
    },
    NativeEmpty,
    DataEmpty,
    MoveFailed {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    Io {
        op: IoOp,
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    Cancelled,
}

impl From<DropRejection> for InstallFailure {
    fn from(rejection: DropRejection) -> Self {
        InstallFailure::Rejected(rejection)
    }
}

impl InstallFailure {
    pub(crate) fn io(op: IoOp, path: &std::path::Path, error: &std::io::Error) -> Self {
        InstallFailure::Io {
            op,
            path: path.to_path_buf(),
            kind: error.kind(),
        }
    }

    pub fn extraction(error: yp_core::Error) -> Self {
        match error {
            yp_core::Error::Cancelled => InstallFailure::Cancelled,
            yp_core::Error::Archive { failure, .. } => DropRejection::ArchiveExtractFailed(failure).into(),
            yp_core::Error::Io { op, path, source } => InstallFailure::Io {
                op,
                path,
                kind: source.kind(),
            },
            _ => DropRejection::ArchiveExtractFailed(ArchiveFailure::Corrupt).into(),
        }
    }
}

pub type InstallResult = std::result::Result<Installed, InstallFailure>;
