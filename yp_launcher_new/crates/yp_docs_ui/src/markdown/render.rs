use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Div, ElementId, Font, FontStyle, FontWeight, Hsla, ObjectFit, Pixels,
    ScrollHandle, SharedString, Stateful, StrikethroughStyle, TextRun, UnderlineStyle, Window,
    canvas, div, img, prelude::*, px, relative,
};
use yp_ui::prelude::*;
use yp_ui::{ActiveTheme, Lightbox, Sizes, Theme};

use super::model::{Block, CellAlign, Document, List, RichText, Table, ToolMarker};
use super::rich_text::{LinkHandler, LinkRange, RichBlock};
use crate::syntax::{Language, Token};

const MAX_FIGURE_WIDTH: f32 = 720.;
const LIST_MARKER_WIDTH: f32 = 22.;
const BULLET: &str = "\u{2022}";

pub type AnchorBounds = Rc<RefCell<HashMap<String, Bounds<Pixels>>>>;
pub type ToolRenderer = Rc<dyn Fn(&ToolMarker, &mut Window, &mut App) -> Option<AnyElement>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

pub type FigureResolver = Rc<dyn Fn(&str) -> Option<Figure>>;

#[derive(Clone)]
pub struct MarkdownContext {
    pub scope: SharedString,
    pub compact: bool,
    pub on_link: LinkHandler,
    pub figure: FigureResolver,
    pub tool: ToolRenderer,
    pub anchors: AnchorBounds,
}

impl MarkdownContext {
    pub fn new(scope: impl Into<SharedString>) -> Self {
        Self {
            scope: scope.into(),
            compact: false,
            on_link: Rc::new(|_, _, _| {}),
            figure: Rc::new(|_| None),
            tool: Rc::new(|_, _, _| None),
            anchors: AnchorBounds::default(),
        }
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn on_link(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_link = Rc::new(handler);
        self
    }

    pub fn figures(mut self, resolver: impl Fn(&str) -> Option<Figure> + 'static) -> Self {
        self.figure = Rc::new(resolver);
        self
    }

    pub fn tools(
        mut self,
        renderer: impl Fn(&ToolMarker, &mut Window, &mut App) -> Option<AnyElement> + 'static,
    ) -> Self {
        self.tool = Rc::new(renderer);
        self
    }

    pub fn anchors(mut self, anchors: AnchorBounds) -> Self {
        self.anchors = anchors;
        self
    }
}

#[derive(Clone)]
pub(crate) struct Look {
    pub theme: Arc<Theme>,
    pub sizes: Sizes,
    pub body: Pixels,
    pub code: Pixels,
    pub compact: bool,
    pub ui: Font,
    pub mono: Font,
}

impl Look {
    pub fn new(compact: bool, window: &Window, cx: &App) -> Self {
        let theme = cx.theme();
        let sizes = window.sizes(cx);
        Self {
            body: if compact {
                sizes.font_xs
            } else {
                sizes.font_sm
            },
            code: sizes.font_xs,
            compact,
            ui: theme.fonts.ui_font(),
            mono: theme.fonts.mono_font(),
            theme: theme.clone(),
            sizes,
        }
    }

    fn heading_scale(&self, level: u8) -> f32 {
        match (self.compact, level) {
            (false, 1) => 1.9,
            (false, 2) => 1.5,
            (false, 3) => 1.22,
            (false, 4) => 1.08,
            (true, 1) => 1.3,
            (true, 2) => 1.15,
            _ => 1.,
        }
    }

    fn heading_line_height(level: u8) -> f32 {
        match level {
            1 => 1.25,
            2 => 1.3,
            3 => 1.35,
            _ => 1.4,
        }
    }

    fn heading_gap(&self, level: u8) -> Pixels {
        match (self.compact, level) {
            (true, _) => self.sizes.spacing_sm,
            (false, 1 | 2) => self.sizes.padding_lg,
            (false, _) => self.sizes.padding_md,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TextTone {
    pub color: Hsla,
    pub weight: FontWeight,
}

pub(crate) fn rich_runs(rich: &RichText, tone: TextTone, look: &Look) -> (Vec<TextRun>, Vec<LinkRange>) {
    let colors = &look.theme.colors;
    let alpha = &look.theme.alpha;
    let mut runs = Vec::with_capacity(rich.spans.len());
    let mut links: Vec<LinkRange> = Vec::new();
    let mut covered = 0;
    for span in &rich.spans {
        if span.range.start != covered || span.range.end > rich.text.len() {
            return (Vec::new(), Vec::new());
        }
        covered = span.range.end;
        let href = span.link.and_then(|index| rich.links.get(index));
        let mut font = if span.style.code {
            look.mono.clone()
        } else {
            look.ui.clone()
        };
        if !span.style.code {
            font.weight = if span.style.bold {
                FontWeight::BOLD
            } else {
                tone.weight
            };
        } else if span.style.bold {
            font.weight = FontWeight::BOLD;
        }
        if span.style.italic {
            font.style = FontStyle::Italic;
        }
        let color = if href.is_some() || span.style.code {
            colors.accent_primary
        } else if span.style.bold {
            colors.text_primary
        } else {
            tone.color
        };
        runs.push(TextRun {
            len: span.range.len(),
            font,
            color,
            background_color: span
                .style
                .code
                .then(|| colors.accent_primary.opacity(alpha.faint)),
            underline: href.map(|_| UnderlineStyle {
                thickness: px(1.),
                color: Some(colors.accent_primary.opacity(alpha.overlay)),
                wavy: false,
            }),
            strikethrough: span.style.strikethrough.then(|| StrikethroughStyle {
                thickness: px(1.),
                color: Some(color),
            }),
        });
        if let Some(href) = href {
            match links.last_mut() {
                Some(last)
                    if last.range.end == span.range.start && last.href.as_ref() == href.as_str() =>
                {
                    last.range.end = span.range.end;
                }
                _ => links.push(LinkRange {
                    range: span.range.clone(),
                    href: SharedString::from(href.clone()),
                }),
            }
        }
    }
    if covered != rich.text.len() {
        return (Vec::new(), Vec::new());
    }
    (runs, links)
}

pub(crate) fn code_runs(
    code: &str,
    tokens: &[Token],
    language: Option<Language>,
    look: &Look,
) -> Vec<TextRun> {
    let colors = &look.theme.colors;
    let run = |len: usize, color: Hsla| TextRun {
        len,
        font: look.mono.clone(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    if code.is_empty() {
        return Vec::new();
    }
    if language.is_none() || tokens.is_empty() {
        return vec![run(code.len(), colors.accent_secondary)];
    }
    let covered: usize = tokens.iter().map(|token| token.range.len()).sum();
    if covered != code.len() {
        return vec![run(code.len(), colors.text_secondary)];
    }
    tokens
        .iter()
        .map(|token| run(token.range.len(), token.kind.color(colors)))
        .collect()
}

struct Renderer<'a> {
    context: &'a MarkdownContext,
    look: Look,
    next_id: usize,
}

impl Renderer<'_> {
    fn id(&mut self, kind: &'static str) -> gpui::ElementId {
        self.next_id += 1;
        gpui::ElementId::Name(SharedString::from(format!(
            "{}-{kind}-{}",
            self.context.scope, self.next_id
        )))
    }

    fn rich(&mut self, rich: &RichText, tone: TextTone) -> RichBlock {
        let (runs, links) = rich_runs(rich, tone, &self.look);
        let id = self.id("text");
        RichBlock::new(id, SharedString::from(rich.text.clone()), runs)
            .links(links, self.context.on_link.clone())
    }

    fn body_tone(&self) -> TextTone {
        TextTone {
            color: self.look.theme.colors.text_secondary,
            weight: FontWeight::MEDIUM,
        }
    }

    fn blocks(&mut self, blocks: &[Block], window: &mut Window, cx: &mut App) -> Vec<AnyElement> {
        blocks
            .iter()
            .filter_map(|block| self.block(block, window, cx))
            .collect()
    }

    fn block(&mut self, block: &Block, window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        match block {
            Block::Heading {
                level,
                text,
                anchor,
            } => Some(self.heading(*level, text, anchor.as_deref())),
            Block::Paragraph(text) => Some(self.paragraph(text)),
            Block::Code {
                language,
                code,
                tokens,
            } => Some(self.code_block(code, tokens, *language, window, cx)),
            Block::List(list) => Some(self.list(list, window, cx)),
            Block::Quote(children) => Some(self.quote(children, window, cx)),
            Block::Table(table) => Some(self.table(table)),
            Block::Image { source, caption } => self.figure(source, caption.as_deref()),
            Block::Rule => Some(
                div()
                    .w_full()
                    .h(Sizes::BORDER_WIDTH)
                    .bg(self.look.theme.colors.border_light)
                    .into_any_element(),
            ),
            Block::Tool(marker) => (self.context.tool)(marker, window, cx),
        }
    }

    fn heading(&mut self, level: u8, text: &RichText, anchor: Option<&str>) -> AnyElement {
        let colors = self.look.theme.colors;
        let tone = TextTone {
            color: if level >= 6 {
                colors.text_secondary
            } else {
                colors.text_primary
            },
            weight: FontWeight::BOLD,
        };
        let size = self.look.body * self.look.heading_scale(level);
        let tracker = anchor.map(|anchor| {
            let anchor = anchor.to_owned();
            let bounds = self.context.anchors.clone();
            canvas(
                move |area, _, _| {
                    bounds.borrow_mut().insert(anchor, area);
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0()
        });
        v_flex()
            .relative()
            .w_full()
            .pt(self.look.heading_gap(level))
            .text_size(size)
            .line_height(relative(Look::heading_line_height(level)))
            .text_color(tone.color)
            .child(self.rich(text, tone))
            .children(tracker)
            .into_any_element()
    }

    fn paragraph(&mut self, text: &RichText) -> AnyElement {
        let tone = self.body_tone();
        v_flex()
            .w_full()
            .text_size(self.look.body)
            .line_height(relative(1.5))
            .text_color(tone.color)
            .child(self.rich(text, tone))
            .into_any_element()
    }

    fn code_block(
        &mut self,
        code: &str,
        tokens: &[Token],
        language: Option<Language>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let colors = self.look.theme.colors;
        let sizes = self.look.sizes;
        let runs = code_runs(code, tokens, language, &self.look);
        let text = SharedString::from(code.to_owned());
        let scroll_id = self.id("code-scroll");
        let group = SharedString::from(format!("{}-code-group-{}", self.context.scope, self.next_id));
        let handle = window
            .use_keyed_state(scroll_id.clone(), cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let text_id = self.id("code");
        let copy_id = self.id("copy");
        let scroller = code_scroller(
            scroll_id,
            &handle,
            sizes.padding_md,
            self.look.code,
            &self.look,
            RichBlock::new(text_id, text.clone(), runs),
        );
        v_flex()
            .group(group.clone())
            .relative()
            .w_full()
            .rounded(Sizes::ROW_RADIUS)
            .bg(colors.surface_light)
            .border_1()
            .border_color(colors.border_light)
            .child(scroller)
            .child(
                div()
                    .absolute()
                    .top(px(4.))
                    .right(px(4.))
                    .visible_on_hover(group)
                    .child(CopyButton::new(copy_id, text).bordered(true)),
            )
            .into_any_element()
    }

    fn list(&mut self, list: &List, window: &mut Window, cx: &mut App) -> AnyElement {
        let colors = self.look.theme.colors;
        let sizes = self.look.sizes;
        let body = self.look.body;
        let mut number = list.start;
        let mut rows = Vec::with_capacity(list.items.len());
        for item in &list.items {
            let marker = match (item.checked, number) {
                (Some(checked), _) => Icon::new(if checked {
                    IconName::CheckBox
                } else {
                    IconName::CheckBoxOutlineBlank
                })
                .size(IconSize::Small)
                .color(if checked { Color::Accent } else { Color::Muted })
                .into_any_element(),
                (None, Some(current)) => SharedString::from(format!("{current}."))
                    .into_any_element(),
                (None, None) => SharedString::new_static(BULLET).into_any_element(),
            };
            if let Some(current) = &mut number {
                *current += 1;
            }
            let children = self.blocks(&item.blocks, window, cx);
            rows.push(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .w_full()
                    .gap(sizes.spacing_sm)
                    .child(
                        div()
                            .flex_none()
                            .min_w(px(LIST_MARKER_WIDTH))
                            .flex()
                            .justify_end()
                            .pr(sizes.spacing_sm)
                            .text_size(body)
                            .line_height(relative(1.5))
                            .text_color(colors.accent_primary)
                            .child(marker),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(sizes.spacing_sm)
                            .children(children),
                    ),
            );
        }
        v_flex()
            .w_full()
            .gap(sizes.spacing_sm)
            .children(rows)
            .into_any_element()
    }

    fn quote(&mut self, children: &[Block], window: &mut Window, cx: &mut App) -> AnyElement {
        let colors = self.look.theme.colors;
        let sizes = self.look.sizes;
        let children = self.blocks(children, window, cx);
        v_flex()
            .w_full()
            .gap(sizes.spacing_md)
            .px(sizes.padding_md)
            .py(sizes.padding_sm)
            .rounded(Sizes::CONTROL_RADIUS)
            .bg(colors.surface_light)
            .border_l_3()
            .border_color(colors.accent_primary)
            .children(children)
            .into_any_element()
    }

    fn table(&mut self, table: &Table) -> AnyElement {
        let colors = self.look.theme.colors;
        let sizes = self.look.sizes;
        let columns = table.columns();
        let weights = table.column_weights();
        let body = self.look.body;
        let head_tone = TextTone {
            color: colors.text_primary,
            weight: FontWeight::BOLD,
        };
        let body_tone = self.body_tone();
        let empty = RichText::default();
        let mut rows = Vec::with_capacity(table.rows.len() + 1);
        let sources = (!table.head.is_empty())
            .then_some((&table.head, head_tone, true))
            .into_iter()
            .chain(table.rows.iter().map(|row| (row, body_tone, false)));
        for (cells, tone, is_head) in sources {
            let mut row = div().flex().flex_row().w_full();
            for column in 0..columns {
                let cell = cells.get(column).unwrap_or(&empty);
                let content = (!cell.is_blank()).then(|| self.rich(cell, tone));
                let mut sized = v_flex().min_w_0();
                let style = sized.style();
                style.flex_grow = weights.get(column).copied();
                style.flex_shrink = Some(1.);
                style.flex_basis = Some(relative(0.).into());
                row = row.child(
                    sized
                        .px(sizes.padding_md)
                        .py(sizes.padding_sm)
                        .border_r_1()
                        .border_b_1()
                        .border_color(colors.border_light)
                        .text_size(body)
                        .line_height(relative(1.4))
                        .text_color(tone.color)
                        .when(is_head, |this| this.bg(colors.surface_medium))
                        .map(|this| match table.alignment(column) {
                            CellAlign::Start => this,
                            CellAlign::Center => this.text_center(),
                            CellAlign::End => this.text_right(),
                        })
                        .children(content),
                );
            }
            rows.push(row);
        }
        v_flex()
            .w_full()
            .border_l_1()
            .border_t_1()
            .border_color(colors.border_light)
            .children(rows)
            .into_any_element()
    }

    fn figure(&mut self, source: &str, caption: Option<&str>) -> Option<AnyElement> {
        let figure = (self.context.figure)(source)?;
        if figure.width == 0 || figure.height == 0 {
            return None;
        }
        let colors = self.look.theme.colors;
        let alpha = self.look.theme.alpha;
        let sizes = self.look.sizes;
        let hover = colors.accent_primary.opacity(alpha.vignette);
        let path = figure.path.clone();
        let frame = h_flex()
            .id(self.id("figure"))
            .w_full()
            .max_w(px((figure.width as f32).min(MAX_FIGURE_WIDTH)))
            .aspect_ratio(figure.width as f32 / figure.height as f32)
            .overflow_hidden()
            .rounded(Sizes::ROW_RADIUS)
            .border_1()
            .border_color(colors.border_light)
            .cursor_pointer()
            .hover(move |style| style.border_color(hover))
            .on_click(move |_, window, cx| Lightbox::open(path.clone(), window, cx))
            .child(
                img(figure.path)
                    .size_full()
                    .object_fit(ObjectFit::Cover),
            );
        Some(
            v_flex()
                .w_full()
                .gap(sizes.spacing_sm)
                .child(frame)
                .children(caption.map(|caption| {
                    Label::new(SharedString::from(caption.to_owned()))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                }))
                .into_any_element(),
        )
    }
}

pub(crate) fn code_scroller(
    id: impl Into<ElementId>,
    handle: &ScrollHandle,
    padding: Pixels,
    text_size: Pixels,
    look: &Look,
    content: impl IntoElement,
) -> Stateful<Div> {
    let mut scroller = div()
        .id(id)
        .flex()
        .w_full()
        .whitespace_nowrap()
        .text_size(text_size)
        .line_height(relative(1.4))
        .text_color(look.theme.colors.text_secondary)
        .font(look.mono.clone())
        .child(div().flex_none().p(padding).child(content))
        .overflow_x_scrollbar(handle);
    scroller.style().restrict_scroll_to_axis = Some(true);
    scroller
}

pub fn render_document(
    document: &Document,
    context: &MarkdownContext,
    window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement> {
    let mut renderer = Renderer {
        context,
        look: Look::new(context.compact, window, cx),
        next_id: 0,
    };
    renderer.blocks(&document.blocks, window, cx)
}
