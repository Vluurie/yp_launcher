pub mod embedded;
pub mod home;
pub mod model;
pub mod overlay;
#[cfg(feature = "tools")]
pub mod tool_panel;

pub use model::{
    DocsConfig, DocsEvent, DocsFailure, DocsModel, LinkTarget, NavEntry, NavGroup, OpenPage,
    find_by_file_name, link_target, nav_entries, requested_page,
};
pub use overlay::DocsOverlay;
#[cfg(feature = "tools")]
pub use tool_panel::ToolPanel;
