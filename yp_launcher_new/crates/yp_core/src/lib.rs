pub mod archive;
pub mod consts;
pub mod error;
pub mod fs;
pub mod hash;
pub mod host;
pub mod ini;
pub mod logging;
pub mod paths;
pub mod platform;
pub mod process;
pub mod runtime;
pub mod settings;
pub mod task;
pub mod text;
pub mod toml_io;
pub mod watch;
pub mod window_geometry;
pub mod wine_paths;

#[cfg(any(test, feature = "test-support"))]
pub mod testing;

pub use archive::{ArchiveEntry, ArchiveFormat, Extractor};
pub use error::{ArchiveFailure, Error, IoOp, IoResultExt, ProcessFailure, Result};
pub use host::{HostEnv, Os};
pub use paths::{AppPaths, GamePaths};
pub use platform::{
    LaunchUnavailable, NamsCommandRequest, NamsInvocation, NierInstallation, PlatformAdapter, SelectionRejection,
};
pub use process::{ChildProcess, ProcessOutput, ProcessRunner, ProcessSpec, StreamKind, SystemRunner};
pub use settings::{OpenReport, SettingKey, SettingsStore};
pub use task::{CancelToken, NoProgress, Progress, ProgressSink};
