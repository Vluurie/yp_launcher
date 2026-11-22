mod model;
mod parse;
mod render;
pub(crate) mod rich_text;

#[cfg(test)]
pub(crate) mod tests;

pub use model::{
    Block, CellAlign, Document, InlineStyle, List, ListItem, RichText, StyleSpan, Table,
    ToolMarker,
};
pub use parse::parse_document;
pub use render::{
    AnchorBounds, Figure, FigureResolver, MarkdownContext, ToolRenderer, render_document,
};
pub use rich_text::{LinkHandler, LinkRange, RichBlock, link_index_at, restyle_range, runs_cover};

#[cfg(feature = "tools")]
pub(crate) use render::{Look, code_runs, code_scroller};
