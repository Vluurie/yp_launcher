use crate::reports::Problem;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ConfigError {
    #[error("NAMS cannot be started: {0}")]
    Unavailable(String),
    #[error("NAMS did not answer in time")]
    TimedOut,
    #[error("NAMS gave no readable answer (exit code {exit_code:?}): {detail}")]
    Output { exit_code: Option<i32>, detail: String },
    #[error("{message}")]
    Nams {
        code: String,
        message: String,
        exit_code: Option<i32>,
        problems: Vec<Problem>,
    },
    #[error(transparent)]
    Io(#[from] IoFailure),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("{0}")]
pub struct IoFailure(pub String);

impl From<yp_core::Error> for ConfigError {
    fn from(error: yp_core::Error) -> Self {
        ConfigError::Io(IoFailure(error.to_string()))
    }
}

impl ConfigError {
    pub fn problems(&self) -> &[Problem] {
        match self {
            ConfigError::Nams { problems, .. } => problems,
            _ => &[],
        }
    }

    pub fn rejected_ids(&self) -> Vec<String> {
        self.problems()
            .iter()
            .filter(|problem| problem.is_error())
            .filter_map(|problem| problem.id.clone())
            .collect()
    }
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;
