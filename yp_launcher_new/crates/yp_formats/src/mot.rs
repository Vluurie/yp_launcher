use crate::bytes::{array, le_f32, le_u16, le_u32, nul_terminated, slice};

pub const FRAMES_PER_SECOND: f32 = 60.0;
const VERSION: u32 = 0x2012_0405;
const MAX_TRACKS: usize = 65_536;
const MAX_SAMPLES: usize = 2_000_000;

#[derive(Clone, Debug, PartialEq)]
pub struct Motion {
    pub name: String,
    pub frames: u16,
    pub tracks: Vec<Track>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub bone: i16,
    pub channel: u8,
    curve: Curve,
}

#[derive(Clone, Debug, PartialEq)]
enum Curve {
    Constant(f32),
    Samples(Vec<f32>),
    Spline(Vec<Key>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Key {
    frame: u16,
    value: f32,
    incoming: f32,
    outgoing: f32,
}

impl Motion {
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.get(..4)? != b"mot\0" || le_u32(bytes, 4)? != VERSION {
            return None;
        }
        let frames = le_u16(bytes, 10)?;
        let offset = le_u32(bytes, 12)? as usize;
        let count = le_u32(bytes, 16)? as usize;
        if frames == 0 || frames > i16::MAX as u16 || count > MAX_TRACKS || offset < 44 {
            return None;
        }
        let name = String::from_utf8_lossy(nul_terminated(slice(bytes, 24, 20)?)).into_owned();
        let records = slice(bytes, offset, count.checked_mul(12)?)?;
        let mut tracks = Vec::with_capacity(count);
        let mut remaining = MAX_SAMPLES;
        for (i, record) in records.chunks_exact(12).enumerate() {
            let bone = i16::from_le_bytes(array(record, 0)?);
            let channel = record[2];
            let encoding = record[3];
            if bone < -1 || !matches!(channel, 0..=5 | 7..=9) || encoding == 0xff {
                continue;
            }
            let n = le_u16(record, 4)? as usize;
            let curve = if encoding == 0 {
                Curve::Constant(finite(le_f32(record, 8)?)?)
            } else {
                if n == 0 || n > i16::MAX as usize {
                    return None;
                }
                remaining = remaining.checked_sub(n)?;
                let start = (offset + i * 12).checked_add(le_u32(record, 8)? as usize)?;
                decode_curve(bytes.get(start..)?, encoding, n)?
            };
            tracks.push(Track {
                bone,
                channel,
                curve,
            });
        }
        (!tracks.is_empty()).then_some(Self {
            name,
            frames,
            tracks,
        })
    }
}

impl Track {
    pub fn sample(&self, frame: u16) -> f32 {
        match &self.curve {
            Curve::Constant(value) => *value,
            Curve::Samples(values) => values[usize::from(frame).min(values.len() - 1)],
            Curve::Spline(keys) => {
                let next = keys.partition_point(|key| key.frame <= frame);
                if next == 0 {
                    return keys[0].value;
                }
                let a = keys[next - 1];
                let Some(b) = keys.get(next) else {
                    return a.value;
                };
                let t = f32::from(frame - a.frame) / f32::from(b.frame - a.frame);
                let u = 1.0 - t;
                a.value * (1.0 + 2.0 * t) * u * u
                    + a.outgoing * t * u * u
                    + b.value * (3.0 - 2.0 * t) * t * t
                    - b.incoming * t * t * u
            }
        }
    }
}

fn finite(value: f32) -> Option<f32> {
    value.is_finite().then_some(value)
}

fn pg_half(bits: u16) -> f32 {
    let sign = if bits & 0x8000 == 0 { 1.0 } else { -1.0 };
    let exponent = i32::from((bits >> 9) & 63);
    let fraction = f32::from(bits & 511) / 512.0;
    if exponent == 0 {
        sign * fraction * 2.0_f32.powi(-46)
    } else {
        sign * (1.0 + fraction) * 2.0_f32.powi(exponent - 47)
    }
}

fn decode_curve(data: &[u8], kind: u8, count: usize) -> Option<Curve> {
    if kind == 1 {
        let values = slice(data, 0, count * 4)?
            .chunks_exact(4)
            .map(|v| finite(le_f32(v, 0)?))
            .collect::<Option<_>>()?;
        return Some(Curve::Samples(values));
    }
    let scalar = |i: usize| match kind {
        2 | 5 => finite(le_f32(data, i * 4)?),
        _ => Some(pg_half(le_u16(data, i * 2)?)),
    };
    if matches!(kind, 2 | 3) {
        let (base, step) = (scalar(0)?, scalar(1)?);
        let (header, stride) = if kind == 2 { (8, 2) } else { (4, 1) };
        let values = slice(data, header, count * stride)?
            .chunks_exact(stride)
            .map(|v| {
                let q = if kind == 2 {
                    le_u16(v, 0)?
                } else {
                    u16::from(v[0])
                };
                finite(base + step * f32::from(q))
            })
            .collect::<Option<_>>()?;
        return Some(Curve::Samples(values));
    }
    let (header, stride) = match kind {
        4 => (0, 16),
        5 => (24, 8),
        6 | 7 => (12, 4),
        8 => (12, 5),
        _ => return None,
    };
    let rows = slice(data, header, count * stride)?;
    let mut scale = [0.0; 6];
    if kind != 4 {
        for (i, value) in scale.iter_mut().enumerate() {
            *value = scalar(i)?;
        }
    }
    let mut keys = Vec::with_capacity(count);
    let mut previous = 0u16;
    for row in rows.chunks_exact(stride) {
        let (frame, values) = match kind {
            4 => (
                le_u16(row, 0)?,
                [le_f32(row, 4)?, le_f32(row, 8)?, le_f32(row, 12)?],
            ),
            _ => {
                let (frame, q) = match kind {
                    5 => (
                        le_u16(row, 0)?,
                        [le_u16(row, 2)?, le_u16(row, 4)?, le_u16(row, 6)?],
                    ),
                    8 => (
                        u16::from_be_bytes(array(row, 0)?),
                        row[2..5]
                            .try_into()
                            .ok()
                            .map(|v: [u8; 3]| v.map(u16::from))?,
                    ),
                    _ => {
                        let f = u16::from(row[0]);
                        (
                            if kind == 7 {
                                previous.checked_add(f)?
                            } else {
                                f
                            },
                            [u16::from(row[1]), u16::from(row[2]), u16::from(row[3])],
                        )
                    }
                };
                (
                    frame,
                    std::array::from_fn(|i| scale[2 * i] + scale[2 * i + 1] * f32::from(q[i])),
                )
            }
        };
        if frame < previous {
            return None;
        }
        previous = frame;
        keys.push(Key {
            frame,
            value: finite(values[0])?,
            incoming: finite(values[1])?,
            outgoing: finite(values[2])?,
        });
    }
    Some(Curve::Spline(keys))
}
