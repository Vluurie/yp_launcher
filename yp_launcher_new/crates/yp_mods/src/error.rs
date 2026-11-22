#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileRejection {
    InvalidName,
    NameCollision,
    UnknownTarget,
    TargetMissing,
    ActiveMissing,
    CannotDeleteLast,
    CannotDeleteActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileStep {
    ParkSidecars,
    ParkMods,
    CreateMods,
    ActivateMods,
    ActivateSidecars,
    TextureState,
    WriteProfiles,
    RenameMods,
    RenameSidecars,
}

#[derive(Debug, thiserror::Error)]
pub enum ModsError {
    #[error(transparent)]
    Core(#[from] yp_core::Error),
    #[error("profile request rejected: {0:?}")]
    Profile(ProfileRejection),
    #[error("profile step {step:?} failed: {source}")]
    ProfileStep {
        step: ProfileStep,
        #[source]
        source: yp_core::Error,
    },
}

impl From<ProfileRejection> for ModsError {
    fn from(rejection: ProfileRejection) -> Self {
        ModsError::Profile(rejection)
    }
}

impl ModsError {
    pub fn profile_rejection(&self) -> Option<ProfileRejection> {
        match self {
            ModsError::Profile(rejection) => Some(*rejection),
            _ => None,
        }
    }

    pub fn profile_step(&self) -> Option<ProfileStep> {
        match self {
            ModsError::ProfileStep { step, .. } => Some(*step),
            _ => None,
        }
    }
}

pub type Result<T, E = ModsError> = std::result::Result<T, E>;
