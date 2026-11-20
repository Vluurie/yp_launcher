use std::ops::Range;

use gpui::{FontStyle, FontWeight, HighlightStyle, StyledText};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use yp_ui::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpanStyle {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub range: Range<usize>,
    pub style: SpanStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    Heading,
    Bullet,
    Numbered(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub text: String,
    pub spans: Vec<Span>,
}

impl Block {
    fn new(kind: BlockKind) -> Self {
        Self {
            kind,
            text: String::new(),
            spans: Vec::new(),
        }
    }

    fn push(&mut self, text: &str, style: SpanStyle) {
        if text.is_empty() {
            return;
        }
        let start = self.text.len();
        self.text.push_str(text);
        if style != SpanStyle::default() {
            self.spans.push(Span {
                range: start..self.text.len(),
                style,
            });
        }
    }
}

struct Builder {
    blocks: Vec<Block>,
    current: Option<Block>,
    lists: Vec<Option<u64>>,
    pending_item: Option<BlockKind>,
    style: SpanStyle,
}

impl Builder {
    fn open(&mut self, kind: BlockKind) {
        self.close();
        self.current = Some(Block::new(kind));
    }

    fn close(&mut self) {
        if let Some(block) = self.current.take()
            && !block.text.trim().is_empty()
        {
            self.blocks.push(block);
        }
    }

    fn text(&mut self, text: &str, style: SpanStyle) {
        if self.current.is_none() {
            let kind = self.pending_item.take().unwrap_or(BlockKind::Paragraph);
            self.current = Some(Block::new(kind));
        }
        if let Some(block) = &mut self.current {
            block.push(text, style);
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                let kind = self.pending_item.take().unwrap_or(BlockKind::Paragraph);
                self.open(kind);
            }
            Tag::Heading { .. } => self.open(BlockKind::Heading),
            Tag::List(first) => {
                self.close();
                self.lists.push(first);
            }
            Tag::Item => {
                self.close();
                let kind = match self.lists.last_mut() {
                    Some(Some(number)) => {
                        let kind = BlockKind::Numbered(*number);
                        *number += 1;
                        kind
                    }
                    _ => BlockKind::Bullet,
                };
                self.pending_item = Some(kind);
            }
            Tag::Strong => self.style.bold = true,
            Tag::Emphasis => self.style.italic = true,
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::Heading(_) => self.close(),
            TagEnd::Item => {
                self.close();
                self.pending_item = None;
            }
            TagEnd::List(_) => {
                self.close();
                self.lists.pop();
            }
            TagEnd::Strong => self.style.bold = false,
            TagEnd::Emphasis => self.style.italic = false,
            _ => {}
        }
    }
}

pub fn parse(markdown: &str) -> Vec<Block> {
    let mut builder = Builder {
        blocks: Vec::new(),
        current: None,
        lists: Vec::new(),
        pending_item: None,
        style: SpanStyle::default(),
    };
    for event in Parser::new_ext(markdown, Options::empty()) {
        match event {
            Event::Start(tag) => builder.start(tag),
            Event::End(tag) => builder.end(tag),
            Event::Text(text) => {
                let style = builder.style;
                builder.text(&text, style);
            }
            Event::Code(code) => {
                let style = SpanStyle {
                    code: true,
                    ..builder.style
                };
                builder.text(&code, style);
            }
            Event::SoftBreak | Event::HardBreak => {
                let style = builder.style;
                builder.text("\n", style);
            }
            _ => {}
        }
    }
    builder.close();
    builder.blocks
}

pub struct MarkdownText {
    blocks: Vec<Block>,
    reading: bool,
}

yp_ui::impl_into_element!(MarkdownText);

impl MarkdownText {
    pub fn new(markdown: &str) -> Self {
        Self::from_blocks(parse(markdown))
    }

    pub fn from_blocks(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            reading: false,
        }
    }

    pub fn reading(mut self, reading: bool) -> Self {
        self.reading = reading;
        self
    }
}

impl RenderOnce for MarkdownText {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let (body_size, heading_size, gap) = if self.reading {
            (sizes.font_md, sizes.font_lg, sizes.padding_md)
        } else {
            (sizes.font_sm, sizes.font_md, sizes.spacing_lg)
        };
        let accent = theme.colors.accent_primary;
        let strong = theme.colors.text_primary;
        let code_background = accent.opacity(theme.alpha.faint);
        let highlight = move |style: SpanStyle| HighlightStyle {
            color: if style.code {
                Some(accent)
            } else if style.bold {
                Some(strong)
            } else {
                None
            },
            font_weight: style.bold.then_some(FontWeight::BOLD),
            font_style: style.italic.then_some(FontStyle::Italic),
            background_color: style.code.then_some(code_background),
            ..HighlightStyle::default()
        };
        v_flex()
            .w_full()
            .gap(gap)
            .text_size(body_size)
            .line_height(relative(1.45))
            .text_color(theme.colors.text_secondary)
            .children(self.blocks.into_iter().map(|block| {
                let highlights: Vec<(Range<usize>, HighlightStyle)> = block
                    .spans
                    .iter()
                    .map(|span| (span.range.clone(), highlight(span.style)))
                    .collect();
                let text = StyledText::new(block.text).with_highlights(highlights);
                match block.kind {
                    BlockKind::Paragraph => div().w_full().child(text).into_any_element(),
                    BlockKind::Heading => div()
                        .w_full()
                        .pt(sizes.spacing_sm)
                        .text_size(heading_size)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.colors.text_primary)
                        .child(text)
                        .into_any_element(),
                    BlockKind::Bullet => list_row("\u{2022}".into(), text, accent, &sizes),
                    BlockKind::Numbered(number) => {
                        list_row(format!("{number}.").into(), text, accent, &sizes)
                    }
                }
            }))
    }
}

fn list_row(
    marker: SharedString,
    text: StyledText,
    marker_color: gpui::Hsla,
    sizes: &Sizes,
) -> AnyElement {
    h_flex()
        .w_full()
        .items_start()
        .gap(sizes.spacing_md)
        .pl(sizes.padding_sm)
        .child(
            div()
                .flex_none()
                .min_w(sizes.icon_sm)
                .text_color(marker_color)
                .child(marker),
        )
        .child(div().flex_1().min_w_0().child(text))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn styled(block: &Block) -> Vec<(&str, SpanStyle)> {
        block
            .spans
            .iter()
            .map(|span| (&block.text[span.range.clone()], span.style))
            .collect()
    }

    #[test]
    fn paragraphs_keep_inline_styles_as_ranges() {
        let blocks = parse("**NAMS** is *not* a proxy for `dxgi.dll`.\n\nSecond.");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].kind, BlockKind::Paragraph);
        assert_eq!(blocks[0].text, "NAMS is not a proxy for dxgi.dll.");
        assert_eq!(
            styled(&blocks[0]),
            vec![
                (
                    "NAMS",
                    SpanStyle {
                        bold: true,
                        ..SpanStyle::default()
                    }
                ),
                (
                    "not",
                    SpanStyle {
                        italic: true,
                        ..SpanStyle::default()
                    }
                ),
                (
                    "dxgi.dll",
                    SpanStyle {
                        code: true,
                        ..SpanStyle::default()
                    }
                ),
            ]
        );
        assert_eq!(blocks[1].text, "Second.");
    }

    #[test]
    fn lists_and_headings_become_their_own_blocks() {
        let blocks = parse("### Why\n\n- **One** thing\n- Two\n\n1. First\n2. Second\n\nEnd");
        let kinds: Vec<BlockKind> = blocks.iter().map(|block| block.kind).collect();
        assert_eq!(
            kinds,
            vec![
                BlockKind::Heading,
                BlockKind::Bullet,
                BlockKind::Bullet,
                BlockKind::Numbered(1),
                BlockKind::Numbered(2),
                BlockKind::Paragraph,
            ]
        );
        assert_eq!(blocks[1].text, "One thing");
        assert_eq!(blocks[4].text, "Second");
    }

    #[test]
    fn soft_breaks_stay_line_breaks() {
        let blocks = parse("line one\nline two");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "line one\nline two");
    }

    #[test]
    fn code_inside_bold_keeps_both_styles() {
        let blocks = parse("**replacing files in `data/`**");
        assert_eq!(
            styled(&blocks[0]),
            vec![
                (
                    "replacing files in ",
                    SpanStyle {
                        bold: true,
                        ..SpanStyle::default()
                    }
                ),
                (
                    "data/",
                    SpanStyle {
                        bold: true,
                        code: true,
                        ..SpanStyle::default()
                    }
                ),
            ]
        );
    }
}
