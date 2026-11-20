pub mod collect;
pub mod config;
pub mod findings;
pub mod full_report;
pub mod model;
pub mod redact;
pub mod summary;

mod scan;
mod text;

pub use collect::Collector;
pub use config::{LEGACY_KEYS, LegacyKey, LegacyMatch, legacy_note};
pub use findings::{build_findings, pack_disabled_by};
pub use full_report::{format_full_report, report_file_name, write_full_report};
pub use model::{
    CompatLayerReport, ConfigDelta, ConfigReport, DISCORD_LIMIT, DiagnosticFinding,
    DiagnosticsReport, DlcSource, FilesystemRole, FindingLevel, GameIdentity, LogsReport,
    NamsHealth, PackSwitch, REPORT_FORMAT, ReportHost, VANILLA_DROP_CAP, VanillaBucket,
    VanillaDirFinding,
};
pub use redact::{Redactor, redact};
pub use scan::{VANILLA_DATA_DIRS, VANILLA_DATA_ROOT_FILES, VANILLA_GAME_ROOT_ENTRIES};
pub use summary::format_summary;
pub use text::{cap_list, file_line, hex_label, human_size, time_label};
