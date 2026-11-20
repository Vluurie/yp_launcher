use std::path::PathBuf;

use yp_core::toml_io::TomlError;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Core(#[from] yp_core::Error),
    #[error("{} is not valid TOML: {error}", path.display())]
    Toml { path: PathBuf, error: TomlError },
    #[error("{} does not hold a JSON object", path.display())]
    SettingsNotObject { path: PathBuf },
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;
