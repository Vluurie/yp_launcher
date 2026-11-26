use gpui::{App, Context};
use std::time::{Duration, Instant};

pub(super) struct Playback {
    pub name: Option<String>,
    pub playing: bool,
    since: Instant,
    offset: Duration,
}

impl Playback {
    pub fn new(cx: &App) -> Self {
        Self {
            name: None,
            playing: !yp_ui::nier::ReducedMotion::enabled(cx),
            since: Instant::now(),
            offset: Duration::ZERO,
        }
    }

    pub fn seconds(&self) -> f32 {
        (self.offset
            + if self.playing {
                self.since.elapsed()
            } else {
                Duration::ZERO
            })
        .as_secs_f32()
    }

    pub fn select(&mut self, name: String, cx: &mut Context<Self>) {
        if self.name.as_ref() != Some(&name) {
            self.name = Some(name);
            self.offset = Duration::ZERO;
            self.since = Instant::now();
            cx.notify();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        if self.playing {
            self.offset += self.since.elapsed();
        }
        self.since = Instant::now();
        self.playing = !self.playing;
        cx.notify();
    }
}
