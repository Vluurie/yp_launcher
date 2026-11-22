use std::collections::HashMap;

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use yp_docs::markdown::markdown_options;
use yp_docs::{DocsMarker, anchor_for};

use super::model::{
    Block, CellAlign, Document, InlineStyle, List, ListItem, RichText, Table, ToolMarker,
};
use crate::syntax::{Language, tokenize};

const ANCHOR_LEVELS: std::ops::RangeInclusive<u8> = 2..=3;
const COMMENT_OPEN: &str = "<!--";
const COMMENT_CLOSE: &str = "-->";
const REMOTE_PREFIX: &str = "http";

#[derive(Default)]
struct AnchorCounter {
    seen: HashMap<String, usize>,
}

impl AnchorCounter {
    fn assign(&mut self, text: &str) -> Option<String> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let base = anchor_for(text);
        let count = self
            .seen
            .entry(base.clone())
            .and_modify(|count| *count += 1)
            .or_insert(0);
        Some(if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        })
    }
}

pub fn parse_document(markdown: &str) -> Document {
    let mut anchors = AnchorCounter::default();
    let mut blocks = Vec::new();
    let mut cursor = 0;
    for marker in DocsMarker::all_in(markdown) {
        if marker.start < cursor
            || !markdown.is_char_boundary(marker.start)
            || !markdown.is_char_boundary(marker.end)
        {
            continue;
        }
        parse_segment(&markdown[cursor..marker.start], &mut anchors, &mut blocks);
        blocks.push(Block::Tool(ToolMarker {
            id: marker.id,
            args: marker.args,
        }));
        cursor = marker.end;
    }
    parse_segment(&markdown[cursor..], &mut anchors, &mut blocks);
    Document { blocks }
}

fn parse_segment(markdown: &str, anchors: &mut AnchorCounter, blocks: &mut Vec<Block>) {
    if markdown.trim().is_empty() {
        return;
    }
    let mut builder = Builder::new(anchors);
    for event in Parser::new_ext(markdown, markdown_options()) {
        builder.event(event);
    }
    blocks.extend(builder.finish());
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InlineTarget {
    Paragraph,
    Heading(u8),
    Cell,
}

struct Inline {
    target: InlineTarget,
    rich: RichText,
}

enum Container {
    Root(Vec<Block>),
    Quote(Vec<Block>),
    List(List),
    Item(ListItem),
}

#[derive(Default)]
struct TableBuilder {
    table: Table,
    row: Vec<RichText>,
}

struct ImageBuilder {
    source: String,
    alt: String,
}

struct CodeBuilder {
    language: Option<Language>,
    code: String,
}

struct Builder<'a> {
    anchors: &'a mut AnchorCounter,
    containers: Vec<Container>,
    inline: Option<Inline>,
    bold: usize,
    italic: usize,
    strikethrough: usize,
    links: Vec<usize>,
    code: Option<CodeBuilder>,
    table: Option<TableBuilder>,
    image: Option<ImageBuilder>,
    html: Option<String>,
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn cell_align(alignment: &Alignment) -> CellAlign {
    match alignment {
        Alignment::None | Alignment::Left => CellAlign::Start,
        Alignment::Center => CellAlign::Center,
        Alignment::Right => CellAlign::End,
    }
}

fn strip_comments(html: &str) -> String {
    let mut kept = String::new();
    let mut rest = html;
    while let Some(open) = rest.find(COMMENT_OPEN) {
        kept.push_str(&rest[..open]);
        let after = &rest[open + COMMENT_OPEN.len()..];
        match after.find(COMMENT_CLOSE) {
            Some(close) => rest = &after[close + COMMENT_CLOSE.len()..],
            None => {
                rest = "";
                break;
            }
        }
    }
    kept.push_str(rest);
    kept
}

fn is_line_break_tag(html: &str) -> bool {
    let tag: String = html
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    matches!(tag.as_str(), "<br>" | "<br/>")
}

impl<'a> Builder<'a> {
    fn new(anchors: &'a mut AnchorCounter) -> Self {
        Self {
            anchors,
            containers: vec![Container::Root(Vec::new())],
            inline: None,
            bold: 0,
            italic: 0,
            strikethrough: 0,
            links: Vec::new(),
            code: None,
            table: None,
            image: None,
            html: None,
        }
    }

    fn style(&self) -> InlineStyle {
        InlineStyle {
            bold: self.bold > 0,
            italic: self.italic > 0,
            strikethrough: self.strikethrough > 0,
            code: false,
        }
    }

    fn push_block(&mut self, block: Block) {
        match self.containers.last_mut() {
            Some(Container::Root(blocks) | Container::Quote(blocks)) => blocks.push(block),
            Some(Container::Item(item)) => item.blocks.push(block),
            Some(Container::List(list)) => list.items.push(ListItem {
                checked: None,
                blocks: vec![block],
            }),
            None => self.containers.push(Container::Root(vec![block])),
        }
    }

    fn open_inline(&mut self, target: InlineTarget) {
        self.close_paragraph();
        self.inline = Some(Inline {
            target,
            rich: RichText::default(),
        });
    }

    fn inline_mut(&mut self) -> &mut RichText {
        &mut self
            .inline
            .get_or_insert_with(|| Inline {
                target: InlineTarget::Paragraph,
                rich: RichText::default(),
            })
            .rich
    }

    fn push_text(&mut self, text: &str, style: InlineStyle) {
        if let Some(image) = &mut self.image {
            image.alt.push_str(text);
            return;
        }
        let link = self.links.last().copied();
        self.inline_mut().push(text, style, link);
    }

    fn close_paragraph(&mut self) {
        if self
            .inline
            .as_ref()
            .is_some_and(|inline| inline.target == InlineTarget::Paragraph)
        {
            self.close_inline();
        }
    }

    fn close_inline(&mut self) -> Option<RichText> {
        let Inline { target, mut rich } = self.inline.take()?;
        self.links.clear();
        rich.trim_end();
        match target {
            InlineTarget::Paragraph => {
                if !rich.is_blank() {
                    self.push_block(Block::Paragraph(rich));
                }
                None
            }
            InlineTarget::Heading(level) => {
                let anchor = if ANCHOR_LEVELS.contains(&level) {
                    self.anchors.assign(&rich.text)
                } else {
                    None
                };
                if !rich.is_blank() {
                    self.push_block(Block::Heading {
                        level,
                        text: rich,
                        anchor,
                    });
                }
                None
            }
            InlineTarget::Cell => Some(rich),
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => self.open_inline(InlineTarget::Paragraph),
            Tag::Heading { level, .. } => {
                self.open_inline(InlineTarget::Heading(heading_level(level)));
            }
            Tag::BlockQuote(_) => {
                self.close_paragraph();
                self.containers.push(Container::Quote(Vec::new()));
            }
            Tag::CodeBlock(kind) => {
                self.close_paragraph();
                let language = match kind {
                    CodeBlockKind::Fenced(info) => Language::from_fence(&info),
                    CodeBlockKind::Indented => None,
                };
                self.code = Some(CodeBuilder {
                    language,
                    code: String::new(),
                });
            }
            Tag::HtmlBlock => {
                self.close_paragraph();
                self.html = Some(String::new());
            }
            Tag::List(start) => {
                self.close_paragraph();
                self.containers.push(Container::List(List {
                    start,
                    items: Vec::new(),
                }));
            }
            Tag::Item => {
                self.close_paragraph();
                self.containers.push(Container::Item(ListItem::default()));
            }
            Tag::Table(alignments) => {
                self.close_paragraph();
                self.table = Some(TableBuilder {
                    table: Table {
                        alignments: alignments.iter().map(cell_align).collect(),
                        ..Table::default()
                    },
                    row: Vec::new(),
                });
            }
            Tag::TableHead | Tag::TableRow => {
                if let Some(table) = &mut self.table {
                    table.row.clear();
                }
            }
            Tag::TableCell => self.open_inline(InlineTarget::Cell),
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Strikethrough => self.strikethrough += 1,
            Tag::Link { dest_url, .. } => {
                let index = self.inline_mut().add_link(dest_url.to_string());
                self.links.push(index);
            }
            Tag::Image { dest_url, .. } => {
                self.image = Some(ImageBuilder {
                    source: dest_url.to_string(),
                    alt: String::new(),
                });
            }
            Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
            | Tag::MetadataBlock(_)
            | Tag::Superscript
            | Tag::Subscript => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::Heading(_) => {
                self.close_inline();
            }
            TagEnd::BlockQuote(_) => {
                self.close_paragraph();
                if let Some(Container::Quote(blocks)) = self.pop_container() {
                    self.push_block(Block::Quote(blocks));
                }
            }
            TagEnd::CodeBlock => {
                if let Some(mut code) = self.code.take() {
                    if code.code.ends_with('\n') {
                        code.code.pop();
                    }
                    if code.code.ends_with('\r') {
                        code.code.pop();
                    }
                    let tokens = code
                        .language
                        .map(|language| tokenize(language, &code.code))
                        .unwrap_or_default();
                    self.push_block(Block::Code {
                        language: code.language,
                        code: code.code,
                        tokens,
                    });
                }
            }
            TagEnd::HtmlBlock => {
                if let Some(html) = self.html.take() {
                    self.html_block(&html);
                }
            }
            TagEnd::Item => {
                self.close_paragraph();
                if let Some(Container::Item(item)) = self.pop_container()
                    && let Some(Container::List(list)) = self.containers.last_mut()
                {
                    list.items.push(item);
                }
            }
            TagEnd::List(_) => {
                self.close_paragraph();
                if let Some(Container::List(list)) = self.pop_container() {
                    self.push_block(Block::List(list));
                }
            }
            TagEnd::TableHead => {
                if let Some(table) = &mut self.table {
                    table.table.head = std::mem::take(&mut table.row);
                }
            }
            TagEnd::TableRow => {
                if let Some(table) = &mut self.table {
                    let row = std::mem::take(&mut table.row);
                    table.table.rows.push(row);
                }
            }
            TagEnd::TableCell => {
                let cell = self.close_inline().unwrap_or_default();
                if let Some(table) = &mut self.table {
                    table.row.push(cell);
                }
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    self.push_block(Block::Table(table.table));
                }
            }
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Strikethrough => self.strikethrough = self.strikethrough.saturating_sub(1),
            TagEnd::Link => {
                self.links.pop();
            }
            TagEnd::Image => self.finish_image(),
            TagEnd::FootnoteDefinition
            | TagEnd::DefinitionList
            | TagEnd::DefinitionListTitle
            | TagEnd::DefinitionListDefinition
            | TagEnd::MetadataBlock(_)
            | TagEnd::Superscript
            | TagEnd::Subscript => {}
        }
    }

    fn pop_container(&mut self) -> Option<Container> {
        if self.containers.len() > 1 {
            self.containers.pop()
        } else {
            None
        }
    }

    fn finish_image(&mut self) {
        let Some(image) = self.image.take() else {
            return;
        };
        let in_flow = self
            .inline
            .as_ref()
            .is_none_or(|inline| inline.target == InlineTarget::Paragraph);
        if !in_flow {
            let style = self.style();
            self.push_text(&image.alt, style);
            return;
        }
        self.close_paragraph();
        self.links.clear();
        if image.source.starts_with(REMOTE_PREFIX) {
            return;
        }
        let caption = Some(image.alt.trim().to_owned()).filter(|alt| !alt.is_empty());
        self.push_block(Block::Image {
            source: image.source,
            caption,
        });
    }

    fn html_block(&mut self, html: &str) {
        let visible = strip_comments(html);
        let visible = visible.trim();
        if visible.is_empty() {
            return;
        }
        let mut rich = RichText::default();
        rich.push(
            visible,
            InlineStyle {
                code: true,
                ..InlineStyle::default()
            },
            None,
        );
        self.push_block(Block::Paragraph(rich));
    }

    fn inline_html(&mut self, html: &str) {
        if is_line_break_tag(html) {
            let style = self.style();
            self.push_text("\n", style);
            return;
        }
        let visible = strip_comments(html);
        if !visible.is_empty() {
            let style = self.style();
            self.push_text(&visible, style);
        }
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => {
                if let Some(code) = &mut self.code {
                    code.code.push_str(&text);
                } else {
                    let style = self.style();
                    self.push_text(&text, style);
                }
            }
            Event::Code(text) => {
                let style = InlineStyle {
                    code: true,
                    ..self.style()
                };
                self.push_text(&text, style);
            }
            Event::InlineMath(text) | Event::DisplayMath(text) | Event::FootnoteReference(text) => {
                let style = self.style();
                self.push_text(&text, style);
            }
            Event::Html(html) => match &mut self.html {
                Some(buffer) => buffer.push_str(&html),
                None => self.html_block(&html),
            },
            Event::InlineHtml(html) => self.inline_html(&html),
            Event::SoftBreak => {
                let style = self.style();
                self.push_text(" ", style);
            }
            Event::HardBreak => {
                let style = self.style();
                self.push_text("\n", style);
            }
            Event::Rule => {
                self.close_paragraph();
                self.push_block(Block::Rule);
            }
            Event::TaskListMarker(checked) => {
                if let Some(Container::Item(item)) = self.containers.last_mut() {
                    item.checked = Some(checked);
                }
            }
        }
    }

    fn finish(mut self) -> Vec<Block> {
        self.close_inline();
        while self.containers.len() > 1 {
            match self.containers.pop() {
                Some(Container::Quote(blocks)) => self.push_block(Block::Quote(blocks)),
                Some(Container::List(list)) => self.push_block(Block::List(list)),
                Some(Container::Item(item)) => {
                    if let Some(Container::List(list)) = self.containers.last_mut() {
                        list.items.push(item);
                    }
                }
                Some(Container::Root(_)) | None => {}
            }
        }
        match self.containers.pop() {
            Some(Container::Root(blocks)) => blocks,
            _ => Vec::new(),
        }
    }
}
