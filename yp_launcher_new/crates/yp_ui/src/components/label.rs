use std::ops::Range;

use gpui::{
    App, FontWeight, HighlightStyle, IntoElement, Pixels, RenderOnce, SharedString, Window,
    prelude::*, relative,
};

use super::text::Text;
use crate::styled_ext::{StyledExt, v_flex};
use crate::theme::{ActiveTheme, Color, ResponsiveExt, Sizes};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LabelSize {
    XSmall,
    Small,
    #[default]
    Default,
    Large,
    XLarge,
}

impl LabelSize {
    pub fn pixels(self, sizes: &Sizes) -> Pixels {
        match self {
            LabelSize::XSmall => sizes.font_xs,
            LabelSize::Small => sizes.font_sm,
            LabelSize::Default => sizes.font_md,
            LabelSize::Large => sizes.font_lg,
            LabelSize::XLarge => sizes.font_xl,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tracking {
    #[default]
    Normal,
    Wide,
    Caps,
}

pub const LINE_HEIGHT_TIGHT: f32 = 1.2;
pub const LINE_HEIGHT_BODY: f32 = 1.35;

#[derive(Clone)]
pub struct Label {
    text: SharedString,
    size: LabelSize,
    color: Color,
    alpha: Option<f32>,
    weight: Option<FontWeight>,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    uppercase: bool,
    tracking: Tracking,
    mono: bool,
    truncate: bool,
    line_clamp: Option<usize>,
    line_height: f32,
    centered: bool,
    group_hover: Option<(SharedString, Color)>,
}

crate::impl_into_element!(Label);

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            size: LabelSize::default(),
            color: Color::Default,
            alpha: None,
            weight: None,
            italic: false,
            underline: false,
            strikethrough: false,
            uppercase: false,
            tracking: Tracking::Normal,
            mono: false,
            truncate: false,
            line_clamp: None,
            line_height: LINE_HEIGHT_BODY,
            centered: false,
            group_hover: None,
        }
    }

    pub fn group_hover_color(mut self, group: impl Into<SharedString>, color: Color) -> Self {
        self.group_hover = Some((group.into(), color));
        self
    }

    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = Some(alpha);
        self
    }

    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn bold(self) -> Self {
        self.weight(FontWeight::BOLD)
    }

    pub fn semibold(self) -> Self {
        self.weight(FontWeight::SEMIBOLD)
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    pub fn uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }

    pub fn tracking(mut self, tracking: Tracking) -> Self {
        self.tracking = tracking;
        self
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    pub fn line_clamp(mut self, lines: usize) -> Self {
        self.line_clamp = Some(lines);
        self
    }

    pub fn line_height(mut self, factor: f32) -> Self {
        self.line_height = factor;
        self
    }

    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
}

impl RenderOnce for Label {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let color = match self.alpha {
            Some(alpha) => self.color.color(cx).opacity(alpha),
            None => self.color.color(cx),
        };
        let text = if self.uppercase {
            SharedString::from(self.text.to_uppercase())
        } else {
            self.text
        };
        let underline = self.underline;
        let hover = self
            .group_hover
            .map(|(group, color)| (group, color.color(cx)));
        v_flex()
            .text_size(self.size.pixels(&sizes))
            .line_height(relative(self.line_height))
            .text_color(color)
            .when_some(self.weight, |this, weight| this.font_weight(weight))
            .when(self.italic, |this| this.italic())
            .when(underline, |this| {
                this.underline().text_decoration_color(color)
            })
            .when(self.strikethrough, |this| this.line_through())
            .when(self.mono, |this| this.font_mono(cx))
            .when(self.truncate, |this| this.min_w_0().truncate())
            .when(self.centered, |this| this.text_center())
            .when_some(self.line_clamp, |this, lines| {
                this.min_w_0().line_clamp(lines).text_ellipsis()
            })
            .when_some(hover, |this, (group, color)| {
                this.group_hover(group, move |style| {
                    let style = style.text_color(color);
                    if underline {
                        style.text_decoration_color(color)
                    } else {
                        style
                    }
                })
            })
            .child(Text::new(text))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HeadlineSize {
    #[default]
    Page,
    Hero,
}

#[derive(Clone)]
pub struct Headline {
    text: SharedString,
    size: HeadlineSize,
    color: Color,
    uppercase: bool,
    centered: bool,
}

crate::impl_into_element!(Headline);

impl Headline {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            size: HeadlineSize::default(),
            color: Color::Default,
            uppercase: false,
            centered: false,
        }
    }

    pub fn size(mut self, size: HeadlineSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }

    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }
}

impl RenderOnce for Headline {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let size = match self.size {
            HeadlineSize::Page => sizes.font_xl,
            HeadlineSize::Hero => sizes.font_xxl,
        };
        let text = if self.uppercase {
            SharedString::from(self.text.to_uppercase())
        } else {
            self.text
        };
        v_flex()
            .text_size(size)
            .line_height(relative(LINE_HEIGHT_TIGHT))
            .font_weight(FontWeight::BOLD)
            .text_color(self.color.color(cx))
            .when(self.centered, |this| this.text_center())
            .child(Text::new(text))
    }
}

#[derive(Clone)]
pub struct HighlightedLabel {
    text: SharedString,
    ranges: Vec<Range<usize>>,
    size: LabelSize,
    color: Color,
    highlight_color: Color,
    weight: Option<FontWeight>,
    mono: bool,
    truncate: bool,
}

crate::impl_into_element!(HighlightedLabel);

impl HighlightedLabel {
    pub fn new(text: impl Into<SharedString>, ranges: Vec<Range<usize>>) -> Self {
        Self {
            text: text.into(),
            ranges,
            size: LabelSize::default(),
            color: Color::Default,
            highlight_color: Color::Accent,
            weight: None,
            mono: false,
            truncate: false,
        }
    }

    pub fn matching(text: impl Into<SharedString>, query: &str) -> Self {
        let text = text.into();
        let ranges = match_ranges(&text, query);
        Self::new(text, ranges)
    }

    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn highlight_color(mut self, color: Color) -> Self {
        self.highlight_color = color;
        self
    }

    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }
}

pub fn match_ranges(text: &str, query: &str) -> Vec<Range<usize>> {
    if query.is_empty() {
        return Vec::new();
    }
    let needle: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
    let haystack: Vec<(usize, char)> = text.char_indices().collect();
    let mut ranges = Vec::new();
    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        let matches = needle.iter().enumerate().all(|(offset, expected)| {
            let mut lowered = haystack[start + offset].1.to_lowercase();
            lowered.next() == Some(*expected) && lowered.next().is_none()
        });
        if matches {
            let from = haystack[start].0;
            let to = haystack
                .get(start + needle.len())
                .map(|(index, _)| *index)
                .unwrap_or(text.len());
            ranges.push(from..to);
            start += needle.len();
        } else {
            start += 1;
        }
    }
    ranges
}

fn valid_ranges(text: &str, ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    let mut cursor = 0;
    let mut valid = Vec::new();
    let mut sorted = ranges;
    sorted.sort_by_key(|range| range.start);
    for range in sorted {
        if range.start < cursor
            || range.end <= range.start
            || range.end > text.len()
            || !text.is_char_boundary(range.start)
            || !text.is_char_boundary(range.end)
        {
            continue;
        }
        cursor = range.end;
        valid.push(range);
    }
    valid
}

impl RenderOnce for HighlightedLabel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sizes = window.sizes(cx);
        let theme = cx.theme().clone();
        let highlight = self.highlight_color.color(cx);
        let style = HighlightStyle {
            color: Some(highlight),
            font_weight: Some(FontWeight::BOLD),
            background_color: Some(highlight.opacity(theme.alpha.hover)),
            ..Default::default()
        };
        let highlights: Vec<(Range<usize>, HighlightStyle)> = valid_ranges(&self.text, self.ranges)
            .into_iter()
            .map(|range| (range, style))
            .collect();
        v_flex()
            .text_size(self.size.pixels(&sizes))
            .line_height(relative(LINE_HEIGHT_BODY))
            .text_color(self.color.color(cx))
            .when_some(self.weight, |this, weight| this.font_weight(weight))
            .when(self.mono, |this| this.font_mono(cx))
            .when(self.truncate, |this| this.min_w_0().truncate())
            .child(Text::new(self.text).highlights(highlights))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_ranges_finds_every_case_insensitive_hit() {
        assert_eq!(match_ranges("Texture texture", "TEX"), vec![0..3, 8..11]);
        assert_eq!(match_ranges("abc", ""), Vec::<Range<usize>>::new());
        assert_eq!(match_ranges("abc", "abcd"), Vec::<Range<usize>>::new());
    }

    #[test]
    fn match_ranges_respects_multibyte_boundaries() {
        let text = "ä 简体 ä";
        let ranges = match_ranges(text, "Ä");
        assert_eq!(ranges.len(), 2);
        for range in &ranges {
            assert_eq!(&text[range.clone()], "ä");
        }
    }

    #[test]
    fn invalid_ranges_are_dropped() {
        let text = "héllo";
        let ranges = valid_ranges(text, vec![3..5, 0..1, 1..2, 4..40, 2..2]);
        assert_eq!(ranges, vec![0..1, 3..5]);
    }
}
