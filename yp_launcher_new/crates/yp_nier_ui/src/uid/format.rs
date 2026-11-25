use super::{Clip, Graphic, ImageLayer, NineGrid, SpriteKey, Track, UidError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) struct Bytes<'a>(pub &'a [u8]);
type Result<T> = std::result::Result<T, UidError>;

impl<'a> Bytes<'a> {
    pub fn slice(&self, offset: usize, length: usize) -> Result<&'a [u8]> {
        self.0
            .get(offset..offset.saturating_add(length))
            .ok_or_else(|| {
                UidError(format!(
                    "truncated data at byte {offset:#x}, expected {length} bytes"
                ))
            })
    }
    pub fn u32(&self, at: usize) -> Result<u32> {
        Ok(u32::from_le_bytes(self.slice(at, 4)?.try_into().unwrap()))
    }
    pub fn u16(&self, at: usize) -> Result<u16> {
        Ok(u16::from_le_bytes(self.slice(at, 2)?.try_into().unwrap()))
    }
    pub fn byte(&self, at: usize) -> Result<u8> {
        Ok(self.slice(at, 1)?[0])
    }
    pub fn f32(&self, at: usize) -> Result<f32> {
        let v = f32::from_bits(self.u32(at)?);
        if v.is_finite() {
            Ok(v)
        } else {
            Err(UidError(format!("nonfinite value at {at:#x}")))
        }
    }
    pub fn floats<const N: usize>(&self, at: usize) -> Result<[f32; N]> {
        let mut values = [0.; N];
        for (i, v) in values.iter_mut().enumerate() {
            *v = self.f32(at + 4 * i)?;
        }
        Ok(values)
    }
    pub fn string(&self, at: usize, length: usize) -> Result<String> {
        let bytes = self.slice(at, length)?;
        Ok(
            String::from_utf8_lossy(bytes.split(|b| *b == 0).next().unwrap_or_default())
                .into_owned(),
        )
    }
    pub fn array(
        &self,
        offset: usize,
        count: usize,
        stride: usize,
    ) -> Result<std::iter::StepBy<std::ops::Range<usize>>> {
        let length = count
            .checked_mul(stride)
            .ok_or_else(|| UidError("array size overflow".into()))?;
        if count != 0 && offset == 0 {
            return Err(UidError("nonempty table has a null offset".into()));
        }
        self.slice(offset, length)?;
        Ok((offset..offset + length).step_by(stride))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UidDocument {
    pub frame_duration: f32,
    pub alignment: u32,
    pub layers: Vec<Layer>,
    pub clips: Vec<Clip>,
    pub parts: HashMap<u32, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomChannel {
    pub channel: u8,
    pub period: f32,
    pub amplitude: f32,
    pub interpolation: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: u32,
    pub parent: Option<usize>,
    pub visible: bool,
    pub values: [f32; 16],
    pub ignore_parent: [bool; 4],
    pub layout: u8,
    pub z_order: f32,
    pub gradient: [i16; 3],
    pub tracks: Vec<Track>,
    pub random: Vec<RandomChannel>,
    pub kind: LayerKind,
    pub extension: Option<Extension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub id: u32,
    pub words: Vec<u32>,
}

fn white() -> [f32; 4] {
    [1.; 4]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub shadow_offset: [i8; 2],
    #[serde(default)]
    pub shadow_color: [f32; 4],
    #[serde(default = "white")]
    pub main_color: [f32; 4],
    pub bank: String,
    pub events: [u32; 8],
    pub paragraph: i32,
    pub anchor: u8,
    pub letter_spacing: f32,
    pub line_spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerKind {
    Locator {
        name: String,
        part: u32,
    },
    Image(ImageLayer),
    NineGrid(NineGrid),
    Message(Message),
    String {
        font: u32,
        anchor: [u8; 2],
        letter_spacing: f32,
    },
    Text {
        font: String,
        scale: [f32; 2],
        anchor: [u8; 2],
    },
    Mask {
        mode: u32,
        anchor: [u8; 2],
        size: [f32; 2],
    },
    Empty,
}

fn graphic(
    b: &Bytes<'_>,
    anchor: usize,
    hash: usize,
    flags: usize,
    flip: usize,
) -> Result<Graphic> {
    Ok(Graphic {
        key: SpriteKey {
            texture: b.u32(hash)?,
            sprite: b.u32(hash + 4)?,
        },
        anchor: [b.byte(anchor)?, b.byte(anchor + 4)?],
        blend: b.byte(anchor + 8)?,
        channels: b.byte(anchor + 12)?,
        flip: [b.byte(flip)? != 0, b.byte(flip + 1)? != 0],
        linear: b.byte(flags)? != 0,
    })
}

fn kind(ty: u32, b: &Bytes<'_>) -> Result<LayerKind> {
    Ok(match ty {
        0 => LayerKind::Locator {
            name: b.string(0, 16)?,
            part: b.u32(16)?,
        },
        1 => LayerKind::Image(ImageLayer {
            graphic: graphic(b, 8, 28, 37, 40)?,
            size: b.floats(0)?,
            mode: b.byte(24)?,
            tiled: b.byte(39)?,
            crop: if b.byte(36)? != 0 {
                Some(b.floats(44)?)
            } else {
                None
            },
            frame_delay: b.u32(60)?,
            frames: b.u32(64)?,
        }),
        2 => LayerKind::NineGrid(NineGrid {
            graphic: graphic(b, 16, 32, 41, 42)?,
            center: b.floats(0)?,
            border: b.floats(8)?,
            use_sprite_size: b.byte(40)? != 0,
            scale_mode: b.byte(45)?,
        }),
        3 => {
            let mut events = [0; 8];
            for (i, v) in events.iter_mut().enumerate() {
                *v = b.u32(20 + i * 4)?;
            }
            LayerKind::Message(Message {
                bank: b.string(4, 16)?,
                shadow_offset: [b.byte(90)? as i8, b.byte(91)? as i8],
                shadow_color: b.floats(116)?,
                main_color: b.floats(132)?,
                events,
                paragraph: b.u32(52)? as i32,
                anchor: b.byte(88)?,
                letter_spacing: b.f32(92)?,
                line_spacing: b.f32(96)?,
            })
        }
        4 => LayerKind::String {
            font: b.u32(28)?,
            anchor: [b.byte(8)?, b.byte(12)?],
            letter_spacing: b.f32(16)?,
        },
        5 => LayerKind::Text {
            font: b.string(0, 36)?,
            scale: b.floats(56)?,
            anchor: [b.byte(64)?, b.byte(65)?],
        },
        6 => LayerKind::Mask {
            mode: b.u32(0)?,
            anchor: [b.byte(4)?, b.byte(8)?],
            size: b.floats(12)?,
        },
        _ => return Err(UidError(format!("unknown UID layer type {ty}"))),
    })
}

fn tracks(b: &Bytes<'_>, entry: usize, flags: &[[bool; 3]; 4]) -> Result<Vec<Track>> {
    let count = b.u32(entry + 0x1a0)? as usize;
    let start = b.u32(entry + 0x1ac)? as usize;
    let mut result = Vec::new();
    for t in b.array(start, count, 32)? {
        let ch = b.u32(t + 8)?;
        let group = match ch {
            0..=2 => 0,
            3..=5 => 1,
            6..=7 => 2,
            8..=15 => 3,
            _ => continue,
        };
        if !flags[group][1] {
            continue;
        }
        let mut keys = Vec::new();
        for k in b.array(b.u32(t)? as usize, b.u32(t + 4)? as usize, 8)? {
            let time = b.f32(k)?;
            if keys.last().is_some_and(|&(t, _)| t > time) {
                return Err(UidError("unordered UID keys".into()));
            }
            keys.push((time, b.u16(k + 4)?));
        }
        if !keys.is_empty() {
            result.push(Track {
                channel: ch as u8,
                base: b.f32(t + 12)?,
                step: b.f32(t + 16)?,
                range: b.floats(t + 20)?,
                wrap: b.u32(t + 28)? != 0,
                keys,
            });
        }
    }
    Ok(result)
}

impl UidDocument {
    pub fn from_compiled(data: &[u8]) -> Result<Self> {
        let doc: Self = serde_json::from_slice(data).map_err(|e| UidError(e.to_string()))?;
        let count = doc.layers.len();
        if doc.frame_duration <= 0.
            || doc.parts.values().any(|&i| i >= count)
            || doc
                .clips
                .iter()
                .any(|c| c.end < c.start || c.members.iter().any(|&i| i >= count))
            || doc.layers.iter().enumerate().any(|(i, l)| {
                l.parent.is_some_and(|p| p >= i)
                    || l.tracks
                        .iter()
                        .any(|t| t.channel >= 16 || t.keys.windows(2).any(|k| k[0].0 > k[1].0))
                    || l.random.iter().any(|r| r.channel >= 16 || r.period <= 0.)
            })
        {
            return Err(UidError("invalid UID layout or animation reference".into()));
        }
        Ok(doc)
    }
    pub fn parse(data: &[u8]) -> Result<Self> {
        let b = Bytes(data);
        b.slice(0, 32)?;
        let duration = b.f32(24)?;
        let align = b.u32(28)?;
        let entries: Vec<_> = b
            .array(b.u32(12)? as usize, b.u32(0)? as usize, 0x1b4)?
            .collect();
        let mut parts = HashMap::new();
        for e in b.array(b.u32(16)? as usize, b.u32(4)? as usize, 8)? {
            let index = b.u32(e + 4)? as usize;
            if index < entries.len() {
                parts.insert(b.u32(e)?, index);
            }
        }
        let mut clips = Vec::new();
        for a in b.array(b.u32(20)? as usize, b.u32(8)? as usize, 64)? {
            let members = b
                .slice(a + 16, 48)?
                .iter()
                .filter(|&&id| id != 255)
                .filter_map(|&id| parts.get(&u32::from(id)).copied())
                .collect();
            clips.push(Clip {
                id: b.u32(a)?,
                start: b.f32(a + 4)?,
                end: b.f32(a + 8)?,
                looping: b.u32(a + 12)? != 0,
                members,
            });
        }
        let mut boundaries = vec![data.len()];
        for &e in &entries {
            for field in [0x1a4, 0x1a8, 0x1ac, 0x1b0] {
                let pos = b.u32(e + field)? as usize;
                if pos > 0 {
                    b.slice(pos, 0)?;
                    boundaries.push(pos);
                }
            }
        }
        boundaries.sort_unstable();
        boundaries.dedup();
        let mut layers = Vec::new();
        for e in entries {
            let parent = b.u32(e + 0x44)? as i32;
            if parent >= layers.len() as i32 {
                return Err(UidError(format!(
                    "invalid parent {parent} for UID layer {}",
                    layers.len()
                )));
            }
            let mut flags = [[false; 3]; 4];
            for (i, group) in flags.iter_mut().enumerate() {
                for (j, v) in group.iter_mut().enumerate() {
                    *v = b.u32(e + 0x68 + (i * 3 + j) * 4)? != 0;
                }
            }
            let mut values = [0.; 16];
            values[..8].copy_from_slice(&b.floats::<8>(e)?);
            values[8..].copy_from_slice(&b.floats::<8>(e + 36)?);
            let mut random = Vec::new();
            for ch in 0..16 {
                let g = match ch {
                    0..=2 => 0,
                    3..=5 => 1,
                    6..=7 => 2,
                    _ => 3,
                };
                let a = e + 0x98 + ch * 16;
                let duration = b.f32(a)?;
                let cycles = b.u32(a + 4)? as i32;
                let amp = b.f32(a + 8)?;
                if flags[g][2] && duration > 0. && cycles > 0 && amp > 0. && random.len() < 8 {
                    random.push(RandomChannel {
                        channel: ch as u8,
                        period: duration / cycles as f32,
                        amplitude: amp,
                        interpolation: if ch < 8 { b.byte(a + 12)? } else { 0 },
                    });
                }
            }
            let extent = |at: usize| -> Result<&[u8]> {
                let end = boundaries
                    .get(boundaries.partition_point(|&n| n <= at))
                    .copied()
                    .unwrap_or(data.len());
                b.slice(at, end - at)
            };
            let p = b.u32(e + 0x1a4)? as usize;
            let x = b.u32(e + 0x1a8)? as usize;
            let id = b.u32(e + 0x19c)?;
            let extension = if x != 0 && id != 0 {
                Some(Extension {
                    id,
                    words: extent(x)?
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|b| u32::from_le_bytes(*b))
                        .collect(),
                })
            } else {
                None
            };
            layers.push(Layer {
                id: b.u32(e + 0x48)?,
                parent: (parent >= 0).then_some(parent as usize),
                visible: b.u32(e + 0x4c)? != 0,
                values,
                ignore_parent: flags.map(|g| g[0]),
                layout: b.byte(e + 0x60)?,
                z_order: b.f32(e + 0x64)?,
                gradient: [
                    b.u16(e + 0x58)? as i16,
                    b.u16(e + 0x5a)? as i16,
                    b.u16(e + 0x5c)? as i16,
                ],
                tracks: tracks(&b, e, &flags)?,
                random,
                kind: if p == 0 {
                    LayerKind::Empty
                } else {
                    kind(b.u32(e + 0x198)?, &Bytes(extent(p)?))?
                },
                extension,
            });
        }
        Ok(Self {
            frame_duration: duration,
            alignment: align,
            layers,
            clips,
            parts,
        })
    }
}
