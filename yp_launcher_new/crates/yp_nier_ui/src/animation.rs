use crate::motion::next_random;
#[cfg(feature = "uid-debug")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "uid-debug")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: u32,
    pub start: f32,
    pub end: f32,
    pub looping: bool,
    pub members: Vec<usize>,
}

#[cfg(feature = "uid-debug")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub channel: u8,
    pub base: f32,
    pub step: f32,
    pub range: [f32; 2],
    pub wrap: bool,
    pub keys: Vec<(f32, u16)>,
}

#[cfg(feature = "uid-debug")]
impl Track {
    pub fn sample(&self, mut time: f32) -> Option<f32> {
        let &[a, b] = &self.range;
        if self.wrap && b > a && (time < a || time > b) {
            time = a + (time - a).rem_euclid(b - a);
        }
        let value = |raw: u16| f32::from(raw) * self.step + self.base;
        let &(first_time, first) = self.keys.first()?;
        if !time.is_finite() || time <= first_time {
            return Some(value(first));
        }
        let i = self.keys.partition_point(|&(t, _)| t < time);
        let &(end, y) = match self.keys.get(i) {
            Some(k) => k,
            None => return self.keys.last().map(|&(_, v)| value(v)),
        };
        let (start, x) = self.keys[i - 1];
        let (mut x, mut y) = (value(x), value(y));
        if self.channel == 5 && (x - y).abs() > std::f32::consts::PI {
            x = x.rem_euclid(std::f32::consts::TAU);
            y = y.rem_euclid(std::f32::consts::TAU);
        }
        Some(x + (y - x) * ((time - start) / (end - start)))
    }
}

pub(crate) struct RandomUnit {
    period: f32,
    amplitude: f32,
    mode: u8,
    current: u32,
    keys: [f32; 4],
}
impl RandomUnit {
    fn key(rng: &mut u32, a: f32) -> f32 {
        (1. - (next_random(rng) >> 8) as f32 * (2. / 16_777_215.)) * a
    }
    pub(crate) fn new(period: f32, amplitude: f32, mode: u8, rng: &mut u32) -> Self {
        Self {
            period,
            amplitude,
            mode,
            current: 0,
            keys: std::array::from_fn(|_| Self::key(rng, amplitude)),
        }
    }
    pub(crate) fn sample(&mut self, seconds: f32, rng: &mut u32) -> f32 {
        let period = (seconds / self.period) as u32;
        if self.current != period {
            self.keys.rotate_left(1);
            self.keys[3] = Self::key(rng, self.amplitude);
            self.current = period;
        }
        let t = ((seconds - period as f32 * self.period) / self.period).clamp(0., 1.);
        let [a, b, c, d] = self.keys;
        match self.mode {
            1 => {
                let t2 = t * t;
                let t3 = t2 * t;
                0.5 * (b - a) * (t3 - 2. * t2 + t)
                    + b * (2. * t3 - 3. * t2 + 1.)
                    + c * (3. * t2 - 2. * t3)
                    + 0.5 * (d - c) * (t3 - t2)
            }
            2 => b,
            _ => b + (c - b) * t,
        }
    }
}

#[cfg(feature = "uid-debug")]
#[derive(Clone, Copy)]
pub(crate) struct Clock {
    pub(crate) start: f32,
    pub(crate) end: f32,
    pub(crate) since: f32,
    pub(crate) looping: bool,
}
#[cfg(feature = "uid-debug")]
impl Clock {
    pub(crate) fn at(self, now: f32) -> f32 {
        let elapsed = (now - self.since).max(0.);
        let duration = (self.end - self.start).max(0.);
        if self.looping && duration > 0. && elapsed > duration {
            self.start + elapsed % duration
        } else {
            self.start + elapsed.min(duration)
        }
    }
}
