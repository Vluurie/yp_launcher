use std::collections::BTreeMap;
use std::ops::Range;

use crate::syntax::{Language, Token};

const MIN_COLUMN_WEIGHT: usize = 10;
const WRAPPING_COLUMN_WEIGHT: usize = 36;
const MAX_COLUMN_WEIGHT: usize = 48;
const WORD_ALLOWANCE: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSpan {
    pub range: Range<usize>,
    pub style: InlineStyle,
    pub link: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RichText {
    pub text: String,
    pub spans: Vec<StyleSpan>,
    pub links: Vec<String>,
}

impl RichText {
    pub fn plain(text: impl Into<String>) -> Self {
        let mut rich = RichText::default();
        rich.push(&text.into(), InlineStyle::default(), None);
        rich
    }

    pub fn push(&mut self, text: &str, style: InlineStyle, link: Option<usize>) {
        if text.is_empty() {
            return;
        }
        let start = self.text.len();
        self.text.push_str(text);
        let end = self.text.len();
        match self.spans.last_mut() {
            Some(last) if last.style == style && last.link == link && last.range.end == start => {
                last.range.end = end;
            }
            _ => self.spans.push(StyleSpan {
                range: start..end,
                style,
                link,
            }),
        }
    }

    pub fn add_link(&mut self, href: impl Into<String>) -> usize {
        self.links.push(href.into());
        self.links.len() - 1
    }

    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }

    pub fn link_at(&self, index: usize) -> Option<usize> {
        self.spans
            .iter()
            .find(|span| span.range.contains(&index))
            .and_then(|span| span.link)
    }

    pub fn trim_end(&mut self) {
        let trimmed = self.text.trim_end().len();
        self.text.truncate(trimmed);
        self.spans.retain_mut(|span| {
            span.range.end = span.range.end.min(trimmed);
            span.range.start < span.range.end
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CellAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Table {
    pub alignments: Vec<CellAlign>,
    pub head: Vec<RichText>,
    pub rows: Vec<Vec<RichText>>,
}

impl Table {
    pub fn columns(&self) -> usize {
        self.rows
            .iter()
            .map(Vec::len)
            .chain([self.head.len(), self.alignments.len()])
            .max()
            .unwrap_or(0)
    }

    pub fn alignment(&self, column: usize) -> CellAlign {
        self.alignments.get(column).copied().unwrap_or_default()
    }

    pub fn column_weights(&self) -> Vec<f32> {
        (0..self.columns())
            .map(|column| {
                let mut longest_word = 0;
                let mut longest_cell = 0;
                for cell in std::iter::once(&self.head)
                    .chain(self.rows.iter())
                    .filter_map(|row| row.get(column))
                {
                    longest_cell = longest_cell.max(cell.text.chars().count());
                    longest_word = longest_word.max(
                        cell.text
                            .split_whitespace()
                            .map(|word| word.chars().count())
                            .max()
                            .unwrap_or(0),
                    );
                }
                (longest_word + WORD_ALLOWANCE)
                    .max(longest_cell.min(WRAPPING_COLUMN_WEIGHT))
                    .clamp(MIN_COLUMN_WEIGHT, MAX_COLUMN_WEIGHT) as f32
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct List {
    pub start: Option<u64>,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolMarker {
    pub id: String,
    pub args: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: u8,
        text: RichText,
        anchor: Option<String>,
    },
    Paragraph(RichText),
    Code {
        language: Option<Language>,
        code: String,
        tokens: Vec<Token>,
    },
    List(List),
    Quote(Vec<Block>),
    Table(Table),
    Image {
        source: String,
        caption: Option<String>,
    },
    Rule,
    Tool(ToolMarker),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub blocks: Vec<Block>,
}

impl Document {
    pub fn anchors(&self) -> Vec<&str> {
        let mut anchors = Vec::new();
        collect_anchors(&self.blocks, &mut anchors);
        anchors
    }

    pub fn tools(&self) -> Vec<&ToolMarker> {
        self.blocks
            .iter()
            .filter_map(|block| match block {
                Block::Tool(marker) => Some(marker),
                _ => None,
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

fn collect_anchors<'a>(blocks: &'a [Block], anchors: &mut Vec<&'a str>) {
    for block in blocks {
        match block {
            Block::Heading {
                anchor: Some(anchor),
                ..
            } => anchors.push(anchor),
            Block::Quote(children) => collect_anchors(children, anchors),
            Block::List(list) => {
                for item in &list.items {
                    collect_anchors(&item.blocks, anchors);
                }
            }
            _ => {}
        }
    }
}
