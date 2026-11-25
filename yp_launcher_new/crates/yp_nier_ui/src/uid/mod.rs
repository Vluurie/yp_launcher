mod bank;
mod format;
use crate::paint::{geometry, graphics, raster};
mod playback;
mod render;
mod resources;

pub use crate::animation::{Clip, Track};
pub use crate::paint::{Graphic, ImageLayer, NineGrid, SpriteKey};
pub use format::{Extension, Layer, LayerKind, Message, RandomChannel, UidDocument};
pub use playback::UidPlayer;
pub use render::UidElement;
pub use resources::UidResources;

use std::fmt;

#[derive(Debug)]
pub struct UidError(pub(crate) String);

impl fmt::Display for UidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for UidError {}
