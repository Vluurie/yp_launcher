use super::{SpriteKey, UidDocument, UidError, format::Bytes};
use image::RgbaImage;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

pub struct UidResources {
    id: u64,
    textures: HashMap<u32, Arc<RgbaImage>>,
    names: HashMap<String, SpriteKey>,
    regions: HashMap<SpriteKey, [u32; 4]>,
    decoded: RefCell<HashMap<SpriteKey, Arc<RgbaImage>>>,
    messages: HashMap<(String, u32, i32), (u16, String)>,
    documents: HashMap<(String, u32), Arc<UidDocument>>,
}
fn next_id() -> u64 {
    static ID: AtomicU64 = AtomicU64::new(1);
    ID.fetch_add(1, Ordering::Relaxed)
}

impl Default for UidResources {
    fn default() -> Self {
        Self {
            id: next_id(),
            textures: HashMap::new(),
            regions: HashMap::new(),
            decoded: Default::default(),
            names: HashMap::new(),
            messages: HashMap::new(),
            documents: HashMap::new(),
        }
    }
}
impl UidResources {
    pub fn sprite_key(&self, name: &str) -> Option<SpriteKey> {
        self.names.get(name).copied()
    }
    pub(crate) fn id(&self) -> u64 {
        self.id
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert_texture(&mut self, id: u32, texture: RgbaImage) {
        self.textures.insert(id, Arc::new(texture));
        self.decoded.get_mut().retain(|k, _| k.texture != id);
        self.id = next_id();
    }
    pub fn insert_sprite(&mut self, key: SpriteKey, image: RgbaImage) {
        self.decoded.get_mut().insert(key, Arc::new(image));
        self.id = next_id();
    }
    pub fn insert_message(
        &mut self,
        bank: impl Into<String>,
        event: u32,
        paragraph: i32,
        font: u16,
        text: impl Into<String>,
    ) {
        self.messages
            .insert((bank.into(), event, paragraph), (font, text.into()));
    }
    pub fn insert_document(&mut self, bank: impl Into<String>, part: u32, doc: Arc<UidDocument>) {
        self.documents.insert((bank.into(), part), doc);
    }
    pub fn document(&self, bank: &str, part: u32) -> Option<&Arc<UidDocument>> {
        self.documents.get(&(bank.to_owned(), part))
    }
    pub fn message(&self, bank: &str, id: u32, paragraph: i32) -> Option<(u16, &str)> {
        self.messages
            .get(&(bank.to_owned(), id, paragraph))
            .map(|(f, t)| (*f, t.as_str()))
    }
    pub fn load_uvd(&mut self, bytes: &[u8]) -> Result<usize, UidError> {
        let b = Bytes(bytes);
        let mut read = Vec::new();
        for at in b.array(b.u32(8)? as usize, b.u32(4)? as usize, 96)? {
            let [x, y, w, h] = b.floats::<4>(at + 72)?;
            if [x, y, w, h].iter().any(|&v| v < 0. || v > u32::MAX as f32) {
                return Err(UidError("invalid UVD rectangle".into()));
            }
            let raw = b.slice(at, 64)?;
            let end = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
            let name = String::from_utf8_lossy(&raw[..end]).into_owned();
            read.push((
                name,
                SpriteKey {
                    sprite: b.u32(at + 64)?,
                    texture: b.u32(at + 68)?,
                },
                [x as u32, y as u32, w as u32, h as u32],
            ));
        }
        let n = read.len();
        for (name, key, rect) in read {
            self.names.insert(name, key);
            self.regions.insert(key, rect);
        }
        self.decoded.get_mut().clear();
        self.id = next_id();
        Ok(n)
    }
    pub fn sprite(&self, key: SpriteKey) -> Option<Arc<RgbaImage>> {
        if let Some(img) = self.decoded.borrow().get(&key) {
            return Some(img.clone());
        }
        let &[x, y, w, h] = self.regions.get(&key)?;
        let src = self.textures.get(&key.texture)?;
        if x.checked_add(w)? > src.width() || y.checked_add(h)? > src.height() {
            return None;
        }
        let img = image::imageops::crop_imm(src.as_ref(), x, y, w, h).to_image();
        let img = Arc::new(img);
        self.decoded.borrow_mut().insert(key, img.clone());
        Some(img)
    }
}
impl crate::paint::SpriteSource for UidResources {
    fn id(&self) -> u64 {
        self.id
    }
    fn sprite(&self, key: SpriteKey) -> Option<Arc<RgbaImage>> {
        self.sprite(key)
    }
}
