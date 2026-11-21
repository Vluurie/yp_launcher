#![recursion_limit = "512"]

pub mod cli;
pub mod error;
pub mod legacy;
pub mod naiom_keys;
pub mod reports;
pub mod schema;
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

pub use cli::{CliOutput, NamsCli, ResetTarget};
pub use error::{ConfigError, Result};
pub use reports::{Change, FileState, MigrateReport, Problem, SetReport, SkippedFile, ValidateReport, Values};
pub use schema::{Category, Choice, ItemType, Kind, Preset, Schema, Setting, TableField, values_equal};
