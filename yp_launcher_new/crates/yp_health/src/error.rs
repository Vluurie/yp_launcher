#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error(transparent)]
    Core(#[from] yp_core::Error),
}

impl HealthError {
    pub fn is_cancelled(&self) -> bool {
        match self {
            HealthError::Core(inner) => inner.is_cancelled(),
        }
    }
}

pub type Result<T, E = HealthError> = std::result::Result<T, E>;
