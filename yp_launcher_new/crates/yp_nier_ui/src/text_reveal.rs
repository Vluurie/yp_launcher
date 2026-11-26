mod list_row;

use crate::motion::next_random;
use gpui::{App, ElementId, Global, IntoElement, SharedString, Window};
pub(crate) use list_row::ListRowText;
use list_row::Playback;
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;

const UPDATES_PER_LETTER: u64 = 7;
const TYPING_RATE: f64 = 0.76;
const PAGE_LETTER_MS: usize = 28;
pub(crate) const PAGE_TEXT_LIMIT: Duration = Duration::from_millis(380);

#[derive(Clone, Copy)]
pub(crate) enum Cadence {
    Title,
    Row,
}

impl Cadence {
    fn visible(self, frame: u32) -> usize {
        let updates = match self {
            Self::Title => 2,
            Self::Row => 3,
        };
        (u64::from(frame) * updates / UPDATES_PER_LETTER) as usize
    }
}

struct Sequence(u32);
impl Global for Sequence {}

struct Reveal {
    text: SharedString,
    offsets: Vec<usize>,
    line_len: usize,
    started: Instant,
    seed: u32,
    last_tick: Option<u32>,
    shown: SharedString,
}

impl Reveal {
    fn new(text: SharedString, seed: u32) -> Self {
        let offsets: Vec<_> = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(text.len()))
            .collect();
        let line_len = text
            .split(['\r', '\n'])
            .next()
            .unwrap_or_default()
            .graphemes(true)
            .count();
        Self {
            text,
            offsets,
            line_len,
            started: Instant::now(),
            seed,
            last_tick: None,
            shown: SharedString::default(),
        }
    }

    fn at(&mut self, tick: u32, cadence: Cadence) -> (SharedString, bool) {
        let count = self.offsets.len() - 1;
        let visible = cadence.visible(tick);
        if visible > count || count == 0 {
            return (self.text.clone(), false);
        }
        if self.last_tick != Some(tick) {
            self.last_tick = Some(tick);
            if visible > 0 {
                let unit = (next_random(&mut self.seed) >> 8) as f32 / 16_777_215.;
                let i = ((1. - unit) * self.line_len.saturating_sub(1) as f32) as usize;
                let glyph = |i: usize| &self.text[self.offsets[i]..self.offsets[i + 1]];
                let random = glyph(i);
                let edge = if random.trim().is_empty() {
                    glyph(0)
                } else {
                    random
                };
                let mut out = String::from(&self.text[..self.offsets[visible - 1]]);
                out.push_str(edge);
                self.shown = out.into();
            }
        }
        (self.shown.clone(), true)
    }
}

pub fn page_text_scope(child: impl IntoElement, start: Instant) -> impl IntoElement {
    ListRowText::new(child.into_any_element(), Some(start))
}

pub(crate) fn page_start(cx: &App) -> Option<Instant> {
    match Playback::current(cx) {
        Playback::Opening(start) => Some(start),
        _ => None,
    }
}

pub struct PageTextFrame<'a> {
    pub prefix_len: usize,
    pub glyph: &'a str,
}

fn frame_at(text: &str, elapsed: Duration) -> Option<PageTextFrame<'_>> {
    if elapsed >= PAGE_TEXT_LIMIT {
        return None;
    }
    let age = elapsed.as_millis() as usize;
    let count = text.graphemes(true).count();
    let duration = count
        .saturating_mul(PAGE_LETTER_MS)
        .min(PAGE_TEXT_LIMIT.as_millis() as usize);
    if count < 2 || age >= duration {
        return None;
    }
    let visible = ((age as u128 * count as u128) / duration as u128) as usize;
    let (prefix_len, _) = text.grapheme_indices(true).nth(visible)?;
    let glyph = text
        .graphemes(true)
        .nth((age / PAGE_LETTER_MS + visible) % count)?;
    Some(PageTextFrame { prefix_len, glyph })
}

pub fn page_text_frame<'a>(
    text: &'a str,
    window: &mut Window,
    cx: &App,
) -> Option<PageTextFrame<'a>> {
    let start = page_start(cx)?;
    if crate::ReducedMotion::enabled(cx) {
        return None;
    }
    let frame = frame_at(text, start.elapsed())?;
    window.request_animation_frame();
    Some(frame)
}

pub fn page_text(text: &str, window: &mut Window, cx: &App) -> Option<SharedString> {
    let start = page_start(cx)?;
    if crate::ReducedMotion::enabled(cx) {
        return None;
    }
    reveal_since(text, start.elapsed(), window)
}

pub(super) fn reveal_since(
    text: &str,
    elapsed: Duration,
    window: &mut Window,
) -> Option<SharedString> {
    if elapsed >= PAGE_TEXT_LIMIT
        || (text.len() > 192 && text.graphemes(true).nth(192).is_some())
        || text.contains(['\n', '\r'])
    {
        return None;
    }
    let frame = frame_at(text, elapsed)?;
    let mut shown = String::from(&text[..frame.prefix_len]);
    shown.push_str(frame.glyph);
    window.request_animation_frame();
    Some(shown.into())
}

pub(crate) fn display(
    text: &SharedString,
    key: impl Into<ElementId>,
    delay: u32,
    cadence: Cadence,
    w: &mut Window,
    cx: &mut App,
) -> SharedString {
    let playback = Playback::current(cx);
    if crate::ReducedMotion::enabled(cx) || matches!(playback, Playback::Complete) {
        return text.clone();
    }
    let state = w.use_keyed_state(key, cx, |_, cx| {
        if !cx.has_global::<Sequence>() {
            cx.set_global(Sequence(1));
        }
        Reveal::new(
            text.clone(),
            next_random(&mut cx.global_mut::<Sequence>().0),
        )
    });
    let (shown, active) = state.update(cx, |state, _| {
        if state.text != *text {
            *state = Reveal::new(text.clone(), next_random(&mut state.seed));
        }
        let started = match playback {
            Playback::Opening(started) => started,
            _ => state.started,
        };
        let elapsed_frames = started.elapsed().as_secs_f64() * 60.;
        let typing_frames = (elapsed_frames - f64::from(delay)).max(0.) * TYPING_RATE;
        state.at(typing_frames as u32, cadence)
    });
    if active {
        w.request_animation_frame();
    }
    shown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn page_labels_keep_typing_after_the_wipe_and_finish_within_the_limit(
        cx: &mut gpui::TestAppContext,
    ) {
        struct Page;
        impl gpui::Render for Page {
            fn render(
                &mut self,
                window: &mut Window,
                _: &mut gpui::Context<Self>,
            ) -> impl gpui::IntoElement {
                let title = "MOD MANAGER";
                let midway = reveal_since(title, Duration::from_millis(200), window).unwrap();
                assert_eq!(midway.len(), 8);
                assert!(midway.starts_with("MOD MAN"));
                assert!(reveal_since(title, Duration::from_millis(308), window).is_none());

                let label = "Create a desktop shortcut to launch with YoRHa Protocol";
                assert!(reveal_since(label, Duration::from_millis(350), window).is_some());
                assert!(reveal_since(label, PAGE_TEXT_LIMIT, window).is_none());
                gpui::div()
            }
        }
        let (_, window) = cx.add_window_view(|_, _| Page);
        window.run_until_parked();
    }

    #[test]
    fn only_the_leading_glyph_changes_between_steps() {
        let mut r = Reveal::new("WEAPONS".into(), 42);
        assert!(r.at(3, Cadence::Title).0.is_empty());
        let frames: Vec<_> = (14..18).map(|n| r.at(n, Cadence::Title).0).collect();
        assert!(
            frames
                .iter()
                .all(|s| s.starts_with("WEA") && s.len() == 4 && "WEAPONS".contains(&s[3..]))
        );
        assert!(frames.windows(2).any(|f| f[0] != f[1]));
        assert_eq!(r.at(17, Cadence::Title).0, frames[3]);
        assert_eq!(r.at(28, Cadence::Title), ("WEAPONS".into(), false));
        assert_eq!(r.at(19, Cadence::Row), ("WEAPONS".into(), false));
    }

    #[test]
    fn partial_labels_keep_user_graphemes_intact() {
        let mut r = Reveal::new("Aé e\u{301}界".into(), 1);
        assert_eq!(r.at(10, Cadence::Row).0.graphemes(true).count(), 4);
        assert_eq!(r.at(14, Cadence::Row).0.as_ref(), "Aé e\u{301}界");
        assert!(!Reveal::new("".into(), 1).at(0, Cadence::Row).1);
    }

    #[test]
    fn a_long_multiline_paragraph_reveals_whole_graphemes_within_the_page_timing() {
        let text = "界e\u{301}\n".repeat(200);
        let frame = frame_at(&text, PAGE_TEXT_LIMIT / 2).unwrap();
        assert_eq!(text[..frame.prefix_len].graphemes(true).count(), 300);
        assert!(["界", "e\u{301}", "\n"].contains(&frame.glyph));
        assert!(frame_at(&text, PAGE_TEXT_LIMIT).is_none());
    }
}
