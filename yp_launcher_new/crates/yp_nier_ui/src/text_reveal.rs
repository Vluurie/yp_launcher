mod list_row;

use crate::motion::next_random;
use gpui::{App, ElementId, Global, SharedString, Window};
pub(crate) use list_row::ListRowText;
use list_row::Playback;
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;

const UPDATES_PER_LETTER: u64 = 7;

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

pub(crate) fn display(
    text: &SharedString,
    key: impl Into<ElementId>,
    delay: u32,
    cadence: Cadence,
    w: &mut Window,
    cx: &mut App,
) -> SharedString {
    let playback = Playback::current(cx);
    if matches!(playback, Playback::Complete) {
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
        let tick = (started.elapsed().as_secs_f64() * 60.) as u32;
        state.at(tick.saturating_sub(delay), cadence)
    });
    if active {
        w.request_animation_frame();
    }
    shown
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
