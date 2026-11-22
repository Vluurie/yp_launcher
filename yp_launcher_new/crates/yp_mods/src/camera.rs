use std::path::{Path, PathBuf};

use yp_core::{Error, IoOp, IoResultExt, toml_io};

use crate::fsx;
use crate::toml_doc::read_values;

pub const TABLE: &str = "player_camera_settings";
const ENTITIES_DIR: &str = "entities";
const CONFIG_SUFFIX: &str = ".toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraField {
    SideOffset,
    HeightOffset,
    DistanceOffset,
    FovOffset,
    PitchUpLimit,
    PitchDownLimit,
}

impl CameraField {
    pub const ALL: [CameraField; 6] = [
        CameraField::SideOffset,
        CameraField::HeightOffset,
        CameraField::DistanceOffset,
        CameraField::FovOffset,
        CameraField::PitchUpLimit,
        CameraField::PitchDownLimit,
    ];

    pub fn key(self) -> &'static str {
        match self {
            CameraField::SideOffset => "side_offset",
            CameraField::HeightOffset => "height_offset",
            CameraField::DistanceOffset => "distance_offset",
            CameraField::FovOffset => "fov_offset",
            CameraField::PitchUpLimit => "pitch_up_limit",
            CameraField::PitchDownLimit => "pitch_down_limit",
        }
    }

    pub fn default_value(self) -> f64 {
        match self {
            CameraField::PitchUpLimit | CameraField::PitchDownLimit => 72.0,
            _ => 0.0,
        }
    }

    pub fn min(self) -> f64 {
        match self {
            CameraField::SideOffset => -3.0,
            CameraField::HeightOffset => -2.0,
            CameraField::DistanceOffset => -5.0,
            CameraField::FovOffset => -25.0,
            CameraField::PitchUpLimit | CameraField::PitchDownLimit => 1.0,
        }
    }

    pub fn max(self) -> f64 {
        match self {
            CameraField::SideOffset => 3.0,
            CameraField::HeightOffset => 2.0,
            CameraField::DistanceOffset => 10.0,
            CameraField::FovOffset => 60.0,
            CameraField::PitchUpLimit | CameraField::PitchDownLimit => 89.0,
        }
    }

    pub fn step(self) -> f64 {
        match self {
            CameraField::SideOffset | CameraField::HeightOffset | CameraField::DistanceOffset => {
                0.05
            }
            CameraField::FovOffset | CameraField::PitchUpLimit | CameraField::PitchDownLimit => 1.0,
        }
    }

    pub fn decimals(self) -> u32 {
        match self {
            CameraField::SideOffset | CameraField::HeightOffset | CameraField::DistanceOffset => 2,
            CameraField::FovOffset | CameraField::PitchUpLimit | CameraField::PitchDownLimit => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSettings {
    pub side_offset: f64,
    pub height_offset: f64,
    pub distance_offset: f64,
    pub fov_offset: f64,
    pub pitch_up_limit: f64,
    pub pitch_down_limit: f64,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            side_offset: CameraField::SideOffset.default_value(),
            height_offset: CameraField::HeightOffset.default_value(),
            distance_offset: CameraField::DistanceOffset.default_value(),
            fov_offset: CameraField::FovOffset.default_value(),
            pitch_up_limit: CameraField::PitchUpLimit.default_value(),
            pitch_down_limit: CameraField::PitchDownLimit.default_value(),
        }
    }
}

impl CameraSettings {
    pub fn get(&self, field: CameraField) -> f64 {
        match field {
            CameraField::SideOffset => self.side_offset,
            CameraField::HeightOffset => self.height_offset,
            CameraField::DistanceOffset => self.distance_offset,
            CameraField::FovOffset => self.fov_offset,
            CameraField::PitchUpLimit => self.pitch_up_limit,
            CameraField::PitchDownLimit => self.pitch_down_limit,
        }
    }

    pub fn set(&mut self, field: CameraField, value: f64) {
        match field {
            CameraField::SideOffset => self.side_offset = value,
            CameraField::HeightOffset => self.height_offset = value,
            CameraField::DistanceOffset => self.distance_offset = value,
            CameraField::FovOffset => self.fov_offset = value,
            CameraField::PitchUpLimit => self.pitch_up_limit = value,
            CameraField::PitchDownLimit => self.pitch_down_limit = value,
        }
    }

    fn from_table(table: &toml::Table) -> Self {
        let mut settings = Self::default();
        for field in CameraField::ALL {
            let number = match table.get(field.key()) {
                Some(toml::Value::Integer(value)) => Some(*value as f64),
                Some(toml::Value::Float(value)) => Some(*value),
                _ => None,
            };
            if let Some(number) = number {
                settings.set(field, number);
            }
        }
        settings
    }

    fn to_updates(self) -> toml::Table {
        let mut table = toml::Table::new();
        for field in CameraField::ALL {
            table.insert(field.key().to_owned(), toml::Value::Float(self.get(field)));
        }
        let mut updates = toml::Table::new();
        updates.insert(TABLE.to_owned(), toml::Value::Table(table));
        updates
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFile {
    pub path: PathBuf,
    pub settings: CameraSettings,
}

fn is_entity_config(file: &Path) -> bool {
    fsx::lower_path_ends_with(file, CONFIG_SUFFIX)
        && file
            .components()
            .any(|part| part.as_os_str() == ENTITIES_DIR)
}

fn read_settings(file: &Path) -> Option<CameraSettings> {
    let raw = std::fs::read_to_string(file).ok()?;
    let values = read_values(&raw);
    values
        .get(TABLE)?
        .as_table()
        .map(CameraSettings::from_table)
}

pub fn find(mod_root: &Path) -> Option<CameraFile> {
    if !mod_root.is_dir() {
        return None;
    }
    fsx::find_walked_file(mod_root, |file| {
        if !is_entity_config(file) {
            return None;
        }
        read_settings(file).map(|settings| CameraFile {
            path: file.to_path_buf(),
            settings,
        })
    })
}

pub fn save(file: &CameraFile, settings: &CameraSettings) -> yp_core::Result<()> {
    let raw = std::fs::read_to_string(&file.path).io(IoOp::Read, &file.path)?;
    let updated = toml_io::update_toml(&raw, &settings.to_updates()).map_err(|err| match err {
        Error::TomlParse {
            path: None,
            message,
        } => Error::TomlParse {
            path: Some(file.path.clone()),
            message,
        },
        Error::TomlRoundTrip { path: None } => Error::TomlRoundTrip {
            path: Some(file.path.clone()),
        },
        other => other,
    })?;
    if updated == raw {
        return Ok(());
    }
    yp_core::fs::write_text_atomic(&file.path, &updated)
}
