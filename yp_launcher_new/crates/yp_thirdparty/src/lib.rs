pub mod classify;
pub mod detect;
pub mod dll_id;
pub mod error;
pub mod flags;
pub mod game_mods;
pub mod graphic_mods;
pub mod migoto;
pub mod migration;
pub mod model;
pub mod paths;
pub mod reshade;
pub mod runtime;
pub mod staging;
pub mod state;
pub mod tree;

mod toml_ext;

pub use classify::{classify, source_name_from_path};
pub use detect::{auto_disable_reshade_loading, detect_reshade, ReShadeStatus};
pub use dll_id::GraphicsDll;
pub use error::{Error, Result};
pub use flags::LoadingFlag;
pub use game_mods::GameModsRuntime;
pub use graphic_mods::GraphicModsRuntime;
pub use migoto::MigotoRuntime;
pub use migration::{migrate_legacy, MigrationResult};
pub use model::{
    Classification, DllSummary, DropRoute, GameModEntry, GameModSettings, GameModsInfo, InstallFailure, InstallOutcome,
    InstallReport, MigotoConfig, MigotoHunting, MigotoInfo, MigotoMarking, ReShadeConfig, ReShadeInfo, RuntimePresence,
    RuntimeStatus, ThirdPartyKind, ThirdPartyRuntime, ThirdPartyUpdateInfo,
};
pub use reshade::ReShadeRuntime;
pub use runtime::{status_all, GraphicsRuntime, Runtimes, ThirdPartyStatuses};
pub use staging::{stage_source, SourceKind, StageFailure, StagedSource};
pub use state::{ThirdPartyNotice, ThirdPartyState, WatchDepth};
