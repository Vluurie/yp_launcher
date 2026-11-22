#[derive(Debug, thiserror::Error)]
pub enum DocsError {
    #[error(transparent)]
    Core(#[from] yp_core::Error),
    #[error("{path} is not inside the docs folder")]
    NotADocPath { path: String },
}

pub type Result<T, E = DocsError> = std::result::Result<T, E>;
