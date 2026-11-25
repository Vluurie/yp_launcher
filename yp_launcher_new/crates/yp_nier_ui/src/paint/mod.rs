pub(crate) mod geometry;
pub(crate) mod graphics;
pub(crate) mod raster;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub(crate) trait SpriteSource {
    fn id(&self) -> u64;
    fn sprite(&self, key: SpriteKey) -> Option<Arc<RgbaImage>>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpriteKey {
    pub texture: u32,
    pub sprite: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graphic {
    pub key: SpriteKey,
    pub anchor: [u8; 2],
    pub blend: u8,
    pub channels: u8,
    pub flip: [bool; 2],
    pub linear: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageLayer {
    pub graphic: Graphic,
    pub size: [f32; 2],
    pub mode: u8,
    pub tiled: u8,
    pub crop: Option<[f32; 4]>,
    pub frame_delay: u32,
    pub frames: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NineGrid {
    pub graphic: Graphic,
    pub center: [f32; 2],
    pub border: [f32; 2],
    pub use_sprite_size: bool,
    pub scale_mode: u8,
}
