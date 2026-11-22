#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] yp_core::Error),
    #[error(transparent)]
    Format(#[from] yp_formats::FormatError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
