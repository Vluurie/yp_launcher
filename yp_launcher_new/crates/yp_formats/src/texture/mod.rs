pub mod misctex;
pub mod preview;

pub use misctex::{MiscTex, create_misctex, misctex_stem};
pub use preview::{
    DecodedTexture, RgbaTexture, decode_dds, decode_dds_rgba, decode_misctex, encode_png,
};

pub const DDS_HEADER_SIZE: usize = 128;
pub const MIN_MISCTEX_SIDE: u32 = 4;
pub const MAX_MISCTEX_SIDE: u32 = 4096;
pub const MAX_PREVIEW_SIDE: u32 = 8192;
