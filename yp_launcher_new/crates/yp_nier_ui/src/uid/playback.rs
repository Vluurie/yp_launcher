use super::{RandomChannel, UidDocument, UidError};
use crate::animation::{Clock, RandomUnit};
use std::sync::Arc;
pub struct UidPlayer {
    pub document: Arc<UidDocument>,
    clocks: Vec<Clock>,
    units: Vec<Vec<(usize, RandomUnit)>>,
    rng: u32,
    playing: Vec<(u32, Clock)>,
}

impl UidPlayer {
    pub fn new(document: Arc<UidDocument>, mut seed: u32) -> Self {
        let units = document
            .layers
            .iter()
            .map(|l| {
                l.random
                    .iter()
                    .map(|r| {
                        let RandomChannel {
                            channel,
                            period,
                            amplitude,
                            interpolation,
                        } = *r;
                        (
                            usize::from(channel),
                            RandomUnit::new(period, amplitude, interpolation, &mut seed),
                        )
                    })
                    .collect()
            })
            .collect();
        let clocks = vec![
            Clock {
                start: 0.,
                end: 0.,
                since: 0.,
                looping: false
            };
            document.layers.len()
        ];
        Self {
            document,
            clocks,
            units,
            rng: seed,
            playing: Vec::new(),
        }
    }

    pub fn play(&mut self, id: u32, now: f32) -> Result<(), UidError> {
        self.start(id, now, false)
    }
    pub fn finish(&mut self, id: u32, now: f32) -> Result<(), UidError> {
        self.start(id, now, true)
    }
    fn start(&mut self, id: u32, now: f32, end: bool) -> Result<(), UidError> {
        let clip = self
            .document
            .clips
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| UidError(format!("no UID animation {id}")))?;
        let clock = Clock {
            start: if end { clip.end } else { clip.start },
            end: clip.end,
            since: now,
            looping: clip.looping && !end,
        };
        for &layer in &clip.members {
            if let Some(c) = self.clocks.get_mut(layer) {
                *c = clock;
            }
        }
        self.playing.retain(|(i, _)| *i != id);
        self.playing.push((id, clock));
        Ok(())
    }
    pub fn is_playing(&self, now: f32) -> bool {
        self.units.iter().any(|u| !u.is_empty())
            || self
                .playing
                .iter()
                .any(|(_, c)| c.looping || c.at(now) < c.end)
    }
    pub fn clip_done(&self, id: u32, now: f32) -> bool {
        self.playing
            .iter()
            .find(|(n, _)| *n == id)
            .is_none_or(|(_, c)| !c.looping && c.at(now) >= c.end)
    }
    pub fn sample(&mut self, now: f32, timeline: Option<f32>) -> Vec<[f32; 16]> {
        self.document
            .layers
            .iter()
            .enumerate()
            .map(|(index, l)| {
                let mut values = l.values;
                let time = timeline.unwrap_or_else(|| self.clocks[index].at(now));
                for t in &l.tracks {
                    if let Some(v) = t.sample(time) {
                        values[usize::from(t.channel)] = v;
                    }
                }
                for (ch, unit) in &mut self.units[index] {
                    let sum = values[*ch] + unit.sample(now, &mut self.rng);
                    values[*ch] = match *ch {
                        3..=5 => {
                            (sum + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                                - std::f32::consts::PI
                        }
                        8..=15 => sum.clamp(0., 1.),
                        _ => sum,
                    };
                }
                values
            })
            .collect()
    }
}
