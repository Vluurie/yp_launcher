use std::collections::BTreeSet;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value as Json;

pub const SEVERITY_ERROR: &str = "error";

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Problem {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
}

impl Problem {
    pub fn is_error(&self) -> bool {
        self.severity == SEVERITY_ERROR
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct FileState {
    #[serde(default)]
    pub exists: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Values {
    #[serde(default)]
    pub config_version: u32,
    #[serde(default)]
    pub nams_dir: String,
    #[serde(default)]
    pub files: IndexMap<String, FileState>,
    #[serde(default)]
    pub not_in_file: BTreeSet<String>,
    #[serde(default)]
    pub values: IndexMap<String, Json>,
}

impl Values {
    pub fn get(&self, id: &str) -> Option<&Json> {
        self.values.get(id)
    }

    pub fn file_errors(&self) -> impl Iterator<Item = (&str, &str)> {
        self.files
            .iter()
            .filter_map(|(name, state)| state.error.as_deref().map(|error| (name.as_str(), error)))
    }

    pub fn file_error(&self, file: &str) -> Option<&str> {
        self.files.get(file).and_then(|state| state.error.as_deref())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Change {
    pub id: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub old: Json,
    #[serde(default)]
    pub new: Json,
    #[serde(default)]
    pub restart_required: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SetReport {
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub changed: Vec<Change>,
    #[serde(default)]
    pub unchanged: Vec<String>,
    #[serde(default)]
    pub files_written: Vec<String>,
}

impl SetReport {
    pub fn needs_restart(&self) -> bool {
        self.changed.iter().any(|change| change.restart_required)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ValidateReport {
    #[serde(default)]
    pub valid: bool,
    #[serde(default)]
    pub errors: usize,
    #[serde(default)]
    pub warnings: usize,
    #[serde(default)]
    pub problems: Vec<Problem>,
    #[serde(default)]
    pub plugin_errors: Vec<Json>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct MigrationStep {
    #[serde(default)]
    pub step: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub old: Option<Json>,
    #[serde(default)]
    pub value: Option<Json>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Backup {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub backup: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SkippedFile {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct MigrateReport {
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub up_to_date: bool,
    #[serde(default)]
    pub to_version: u32,
    #[serde(default)]
    pub changes: Vec<MigrationStep>,
    #[serde(default)]
    pub backups: Vec<Backup>,
    #[serde(default)]
    pub files_written: Vec<String>,
    #[serde(default)]
    pub skipped: Vec<SkippedFile>,
}
