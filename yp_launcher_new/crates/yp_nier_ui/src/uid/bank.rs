use super::{UidError, UidResources, format::Bytes};
use image::{ImageBuffer, Rgba, RgbaImage};

fn decode(dds: &[u8]) -> Result<RgbaImage, UidError> {
    let b = Bytes(dds);
    if b.slice(0, 4)? != b"DDS " {
        return Err(UidError(
            "expected a PC DDS texture in the WTP payload".into(),
        ));
    }
    let (height, width) = (b.u32(12)?, b.u32(16)?);
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 67_108_864 {
        return Err(UidError("invalid DDS dimensions".into()));
    }
    if b.u32(80)? == 2 && b.u32(88)? == 8 {
        let stride = if b.u32(8)? & 8 != 0 {
            b.u32(20)?.max(width)
        } else {
            width
        };
        let pixels = b.slice(
            128,
            (u64::from(stride) * u64::from(height))
                .try_into()
                .map_err(|_| UidError("DDS pitch overflow".into()))?,
        )?;
        return Ok(ImageBuffer::from_fn(width, height, |x, y| {
            let a = pixels[y as usize * stride as usize + x as usize];
            Rgba([a, a, a, a])
        }));
    }
    image::load_from_memory_with_format(dds, image::ImageFormat::Dds)
        .map(|i| i.into_rgba8())
        .map_err(|e| UidError(e.to_string()))
}

impl UidResources {
    pub fn load_wta(
        &mut self,
        index: &[u8],
        payload: &[u8],
        wanted: impl Fn(u32) -> bool,
    ) -> Result<usize, UidError> {
        let b = Bytes(index);
        if b.slice(0, 4)? != b"WTB\0" {
            return Err(UidError("expected WTB texture index".into()));
        }
        let count = b.u32(8)? as usize;
        let offsets = b.array(b.u32(12)? as usize, count, 4)?;
        let sizes = b.array(b.u32(16)? as usize, count, 4)?;
        let ids = b.array(b.u32(24)? as usize, count, 4)?;
        let mut images = Vec::new();
        for ((at, size), id) in offsets.zip(sizes).zip(ids) {
            let hash = b.u32(id)?;
            if !wanted(hash) {
                continue;
            }
            let texture = Bytes(payload).slice(b.u32(at)? as usize, b.u32(size)? as usize)?;
            images.push((
                hash,
                decode(texture).map_err(|e| UidError(format!("texture {hash:#010x}: {e}")))?,
            ));
        }
        let n = images.len();
        for (hash, image) in images {
            self.insert_texture(hash, image);
        }
        Ok(n)
    }
}
