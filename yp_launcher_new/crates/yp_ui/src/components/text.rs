use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId, HighlightStyle,
    InspectorElementId, LayoutId, Pixels, SharedString, Size, Style, TextOverflow,
    TextRun, TextStyle, TruncateFrom, WhiteSpace, Window, WrappedLine, px, size,
};

const TOLERANCE: Pixels = px(1.);
const CLAMP_PASSES: usize = 4;

fn last_row_width(line: &WrappedLine) -> Pixels {
    let layout = &line.unwrapped_layout;
    let start = line
        .wrap_boundaries()
        .last()
        .and_then(|boundary| {
            layout
                .runs
                .get(boundary.run_ix)?
                .glyphs
                .get(boundary.glyph_ix)
        })
        .map(|glyph| glyph.position.x)
        .unwrap_or_default();
    layout.width - start
}

#[derive(Clone, Copy)]
struct Measured {
    wrap_width: Pixels,
    size: Size<Pixels>,
}

#[derive(Default)]
struct Measurements {
    natural: Option<Size<Pixels>>,
    wrapped: Vec<Measured>,
}

#[derive(Clone)]
struct Shaper {
    text: SharedString,
    highlights: Rc<[(Range<usize>, HighlightStyle)]>,
    style: TextStyle,
    font_size: Pixels,
    line_height: Pixels,
}

impl Shaper {
    fn wraps(&self) -> bool {
        self.style.white_space == WhiteSpace::Normal
    }

    fn suffix(&self) -> Option<(SharedString, TruncateFrom)> {
        self.style
            .text_overflow
            .as_ref()
            .map(|overflow| match overflow {
                TextOverflow::Truncate(affix) => (affix.clone(), TruncateFrom::End),
                TextOverflow::TruncateStart(affix) => (affix.clone(), TruncateFrom::Start),
                TextOverflow::TruncateMiddle(affix) => (affix.clone(), TruncateFrom::Middle),
            })
    }

    fn single_line(&self) -> bool {
        !self.text.contains('\n')
    }

    fn runs(&self, decoration: &TextStyle) -> Vec<TextRun> {
        let mut style = self.style.clone();
        style.color = decoration.color;
        style.background_color = decoration.background_color;
        style.underline = decoration.underline;
        style.strikethrough = decoration.strikethrough;
        let text = self.text.as_ref();
        let mut runs = Vec::new();
        let mut cursor = 0;
        for (range, highlight) in self.highlights.iter() {
            let valid = range.start >= cursor
                && range.end > range.start
                && range.end <= text.len()
                && text.is_char_boundary(range.start)
                && text.is_char_boundary(range.end);
            if !valid {
                continue;
            }
            if range.start > cursor {
                runs.push(style.to_run(range.start - cursor));
            }
            runs.push(style.clone().highlight(*highlight).to_run(range.len()));
            cursor = range.end;
        }
        if cursor < text.len() || runs.is_empty() {
            runs.push(style.to_run(text.len() - cursor));
        }
        runs
    }

    fn shape(
        &self,
        text: SharedString,
        runs: &[TextRun],
        wrap_width: Option<Pixels>,
        clamp: Option<usize>,
        window: &Window,
    ) -> Vec<WrappedLine> {
        window
            .text_system()
            .shape_text(text, self.font_size, runs, wrap_width, clamp)
            .map(|lines| lines.into_iter().collect())
            .unwrap_or_default()
    }

    fn lines(
        &self,
        decoration: &TextStyle,
        limit: Option<Pixels>,
        window: &Window,
        cx: &App,
    ) -> Vec<WrappedLine> {
        let wraps = self.wraps();
        let clamp = self.style.line_clamp.filter(|_| wraps);
        let wrap_width = limit.filter(|_| wraps);
        let budget = match (limit, clamp) {
            (Some(limit), Some(lines)) => Some(limit * lines as f32),
            (Some(limit), None) if !wraps => Some(limit),
            _ => None,
        };
        let suffix = self.suffix().filter(|_| self.single_line());
        let (Some((suffix, from)), Some(mut budget)) = (suffix, budget) else {
            let runs = self.runs(decoration);
            return self.shape(self.text.clone(), &runs, wrap_width, clamp, window);
        };
        let mut lines = Vec::new();
        for _ in 0..CLAMP_PASSES {
            let runs = self.runs(decoration);
            let (text, runs) = cx
                .text_system()
                .line_wrapper(self.style.font(), self.font_size)
                .truncate_line(self.text.clone(), budget, &suffix, &runs, from);
            lines = self.shape(text, &runs, wrap_width, clamp, window);
            let overflow = match (wrap_width, lines.last()) {
                (Some(width), Some(line)) => last_row_width(line) - width,
                _ => px(0.),
            };
            if overflow <= TOLERANCE {
                break;
            }
            budget -= overflow + self.font_size;
        }
        lines
    }

    fn size_of(&self, lines: &[WrappedLine]) -> Size<Pixels> {
        let mut total: Size<Pixels> = Size::default();
        for line in lines {
            let line_size = line.size(self.line_height);
            total.height += line_size.height;
            total.width = total.width.max(line_size.width).ceil();
        }
        total
    }

    fn natural(&self, cache: &mut Measurements, window: &Window, cx: &App) -> Size<Pixels> {
        if let Some(natural) = cache.natural {
            return natural;
        }
        let natural = self.size_of(&self.lines(&self.style, None, window, cx));
        cache.natural = Some(natural);
        natural
    }

    fn shrinks(&self) -> bool {
        !self.wraps() && self.suffix().is_some()
    }

    fn measure(
        &self,
        cache: &mut Measurements,
        known_width: Option<Pixels>,
        available: AvailableSpace,
        window: &Window,
        cx: &App,
    ) -> Size<Pixels> {
        let natural = self.natural(cache, window, cx);
        let limit = known_width.or(match available {
            AvailableSpace::Definite(width) => Some(width),
            AvailableSpace::MinContent | AvailableSpace::MaxContent => None,
        });
        let Some(limit) = limit else {
            return natural;
        };
        if natural.width <= limit {
            return natural;
        }
        if !self.wraps() {
            return match self.shrinks() {
                true => size(limit, natural.height),
                false => natural,
            };
        }
        if let Some(entry) = cache.wrapped.iter().find(|entry| entry.wrap_width == limit) {
            return entry.size;
        }
        let wrapped = self.size_of(&self.lines(&self.style, Some(limit), window, cx));
        cache.wrapped.push(Measured {
            wrap_width: limit,
            size: wrapped,
        });
        wrapped
    }

    fn paint_limit(&self, cache: &Measurements, bounds: Size<Pixels>) -> Option<Pixels> {
        let natural = cache.natural.unwrap_or_default();
        if natural.width <= bounds.width + TOLERANCE {
            return None;
        }
        if !self.wraps() {
            return self.shrinks().then_some(bounds.width);
        }
        let measured = cache
            .wrapped
            .iter()
            .rev()
            .find(|entry| {
                (entry.size.height - bounds.height).abs() <= TOLERANCE
                    && (entry.wrap_width - bounds.width).abs() <= TOLERANCE
            })
            .map(|entry| entry.wrap_width);
        Some(measured.unwrap_or(bounds.width + TOLERANCE))
    }
}

pub struct TextFrame {
    shaper: Shaper,
    cache: Rc<RefCell<Measurements>>,
}

pub struct Text {
    text: SharedString,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
}

impl Text {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            highlights: Vec::new(),
        }
    }

    pub fn highlights(
        mut self,
        highlights: impl IntoIterator<Item = (Range<usize>, HighlightStyle)>,
    ) -> Self {
        self.highlights = highlights.into_iter().collect();
        self
    }
}


impl Element for Text {
    type RequestLayoutState = TextFrame;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = style
            .line_height
            .to_pixels(font_size.into(), window.rem_size());
        let shaper = Shaper {
            text: self.text.clone(),
            highlights: std::mem::take(&mut self.highlights).into(),
            style,
            font_size,
            line_height,
        };
        let cache = Rc::new(RefCell::new(Measurements::default()));
        let layout_id = window.request_measured_layout(Style::default(), {
            let shaper = shaper.clone();
            let cache = cache.clone();
            move |known, available, window, cx| {
                shaper.measure(
                    &mut cache.borrow_mut(),
                    known.width,
                    available.width,
                    window,
                    cx,
                )
            }
        });
        (layout_id, TextFrame { shaper, cache })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _frame: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        frame: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let shaper = &frame.shaper;
        let decoration = window.text_style();
        let limit = {
            let mut cache = frame.cache.borrow_mut();
            shaper.natural(&mut cache, window, cx);
            shaper.paint_limit(&cache, bounds.size)
        };
        let lines = shaper.lines(&decoration, limit, window, cx);
        #[cfg(test)]
        probe::record(&shaper.text, &lines, bounds, decoration.color);
        let align = shaper.style.text_align;
        let mut origin = bounds.origin;
        for line in &lines {
            if let Err(error) =
                line.paint_background(origin, shaper.line_height, align, Some(bounds), window, cx)
            {
                log::debug!("text background not painted: {error}");
            }
            if let Err(error) =
                line.paint(origin, shaper.line_height, align, Some(bounds), window, cx)
            {
                log::debug!("text not painted: {error}");
            }
            origin.y += line.size(shaper.line_height).height;
        }
    }
}

#[cfg(test)]
pub(crate) mod probe {
    use std::cell::RefCell;

    use gpui::{Bounds, Hsla, Pixels, SharedString, WrappedLine};

    #[derive(Clone, Debug)]
    pub(crate) struct Painted {
        pub shown: String,
        pub lines: usize,
        pub bounds: Bounds<Pixels>,
        pub color: Hsla,
    }

    thread_local! {
        static PAINTED: RefCell<Vec<(SharedString, Painted)>> = const { RefCell::new(Vec::new()) };
    }

    pub(super) fn record(
        source: &SharedString,
        lines: &[WrappedLine],
        bounds: Bounds<Pixels>,
        color: Hsla,
    ) {
        let shown = lines
            .iter()
            .map(|line| line.text.as_ref())
            .collect::<Vec<_>>()
            .join(
                "
",
            );
        let wrapped: usize = lines
            .iter()
            .map(|line| line.wrap_boundaries().len() + 1)
            .sum();
        let painted = Painted {
            shown,
            lines: wrapped,
            bounds,
            color,
        };
        PAINTED.with(|all| {
            let mut all = all.borrow_mut();
            all.retain(|(text, _)| text != source);
            all.push((source.clone(), painted));
        });
    }

    pub(crate) fn find(source: &str) -> Option<Painted> {
        PAINTED.with(|all| {
            all.borrow()
                .iter()
                .find(|(text, _)| text.as_ref() == source)
                .map(|(_, painted)| painted.clone())
        })
    }

    pub(crate) fn painted(source: &str) -> Painted {
        find(source).unwrap_or_else(|| panic!("no text painted for {source:?}"))
    }

    pub(crate) fn reset() {
        PAINTED.with(|all| all.borrow_mut().clear());
    }
}

#[cfg(test)]
mod tests {
    use gpui::{TestAppContext, div, prelude::*};

    use super::*;
    use crate::components::test_support::open_root;

    const LONG: &str = "The android stood in the ruins and counted the stars, twice, to be sure.";

    fn tagged(name: &'static str) -> gpui::Div {
        div().debug_selector(move || name.to_owned())
    }

    #[gpui::test]
    fn truncating_text_shrinks_inside_a_row_and_keeps_one_line(cx: &mut TestAppContext) {
        let (_root, cx) = open_root(cx, |_, _| {
            div()
                .w(px(200.))
                .flex()
                .flex_row()
                .child(tagged("label").truncate().child(Text::new(LONG)))
                .child(tagged("tail").flex_none().w(px(40.)).h(px(10.)))
                .into_any_element()
        });
        cx.run_until_parked();
        let label = cx.debug_bounds("label").expect("label");
        let tail = cx.debug_bounds("tail").expect("tail");
        assert_eq!(label.size.width, px(160.));
        assert_eq!(tail.origin.x, px(160.));
        assert!(label.size.height < px(30.));
    }

    #[gpui::test]
    fn wrapping_text_grows_in_height_when_the_width_is_limited(cx: &mut TestAppContext) {
        let (_root, cx) = open_root(cx, |_, _| {
            div()
                .flex()
                .flex_col()
                .items_start()
                .child(tagged("wide").child(Text::new(LONG)))
                .child(tagged("narrow").w(px(120.)).child(Text::new(LONG)))
                .into_any_element()
        });
        cx.run_until_parked();
        let wide = cx.debug_bounds("wide").expect("wide");
        let narrow = cx.debug_bounds("narrow").expect("narrow");
        assert!(wide.size.width > px(200.));
        assert_eq!(narrow.size.width, px(120.));
        assert!(narrow.size.height >= wide.size.height * 3.);
    }

    #[gpui::test]
    fn highlight_ranges_outside_the_text_are_ignored(cx: &mut TestAppContext) {
        let (_root, cx) = open_root(cx, |_, _| {
            let highlight = HighlightStyle::default();
            div()
                .child(Text::new("h\u{00e9}llo").highlights([
                    (0..1, highlight),
                    (1..2, highlight),
                    (3..40, highlight),
                    (4..6, highlight),
                ]))
                .child(Text::new(""))
                .into_any_element()
        });
        cx.run_until_parked();
    }
}
