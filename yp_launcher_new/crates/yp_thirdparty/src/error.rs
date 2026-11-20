#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] yp_core::Error),
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        match self {
            Error::Core(inner) => inner.is_not_found(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
