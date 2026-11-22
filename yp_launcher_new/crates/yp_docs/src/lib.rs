pub mod anchor;
pub mod error;
pub mod index;
pub mod links;
pub mod markdown;
pub mod marker;
pub mod outline;
pub mod search;

pub use anchor::{anchor_for, anchor_in};
pub use error::{DocsError, Result};
pub use index::{ASSET_ROOT, DocPage, DocsLibrary, pretty_name, title_for};
pub use links::resolve_link;
pub use markdown::image_sources;
pub use marker::DocsMarker;
pub use outline::{DocHeading, outline};
pub use search::{DocSearchHit, Excerpt, search_pages};
