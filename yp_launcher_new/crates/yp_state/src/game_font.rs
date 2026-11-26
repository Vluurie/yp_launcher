use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use gpui::{App, AppContext, Global, Task};
use image::GrayImage;
use yp_core::GamePaths;
use yp_formats::font::{self, GameFont};
use yp_host::session::AppSession;
use yp_i18n::{Locale, MsgKey};
use yp_ui::nier::use_bitmap_font;

use crate::env::AppEnv;

const CACHE_FILE_NAME: &str = "game_font.bin";

#[derive(Default)]
struct GameFontState {
    stamp: Option<u32>,
    task: Option<Task<()>>,
}

impl Global for GameFontState {}

fn cache_file(game: &GamePaths) -> PathBuf {
    game.internal_cache_dir().join(CACHE_FILE_NAME)
}

fn launcher_characters() -> BTreeSet<char> {
    Locale::ALL
        .iter()
        .flat_map(|locale| {
            let strings = locale.strings();
            MsgKey::ALL.iter().map(move |key| strings.get(*key))
        })
        .flat_map(str::chars)
        .collect()
}

fn install(font: GameFont, cx: &mut App) {
    let stamp = font.stamp;
    let Some(coverage) = GrayImage::from_raw(font.width, font.height, font.coverage) else {
        return;
    };
    use_bitmap_font(
        coverage,
        font.glyphs
            .iter()
            .map(|glyph| ((glyph.font, glyph.code), glyph.rect, glyph.advance)),
        font.kerning,
        cx,
    );
    cx.default_global::<GameFontState>().stamp = Some(stamp);
}

pub(crate) fn restore(env: &AppEnv, cx: &mut App) {
    let Some(root) = AppSession::load(&env.settings).directory() else {
        return;
    };
    let Ok(bytes) = fs::read(cache_file(&GamePaths::new(root))) else {
        return;
    };
    match GameFont::decode(&bytes) {
        Ok(font) => install(font, cx),
        Err(error) => log::warn!("the cached game font is unreadable and will be rebuilt: {error}"),
    }
}

pub(crate) fn refresh(game: &GamePaths, cx: &mut App) {
    let cache = cache_file(game);
    let root = game.root().to_path_buf();
    let installed = cx.default_global::<GameFontState>().stamp;
    let job = cx.background_spawn(async move { load(&root, &cache, installed) });
    let task = cx.spawn(async move |cx| {
        if let Some(font) = job.await {
            cx.update(|cx| install(font, cx));
        }
    });
    cx.default_global::<GameFontState>().task = Some(task);
}

fn load(root: &Path, cache: &Path, installed: Option<u32>) -> Option<GameFont> {
    let stamp = font::source_stamp(root)
        .inspect_err(|error| log::warn!("the game font sources are unreadable: {error}"))
        .ok()?;
    if installed == Some(stamp) {
        return None;
    }
    let characters = launcher_characters();
    let font = font::build(root, |ch| {
        font::in_common_ranges(ch) || characters.contains(&ch)
    })
    .inspect_err(|error| log::warn!("the game font could not be built: {error}"))
    .ok()?;
    if let Err(error) = write_cache(cache, &font) {
        log::warn!("the game font could not be cached: {error}");
    }
    Some(font)
}

fn write_cache(path: &Path, font: &GameFont) -> io::Result<()> {
    let bytes = font.encode().map_err(io::Error::other)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let partial = path.with_extension("partial");
    fs::write(&partial, bytes)?;
    fs::rename(partial, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use yp_formats::font::Glyph;

    fn sample(stamp: u32) -> GameFont {
        GameFont {
            stamp,
            width: 2,
            height: 1,
            coverage: vec![0, 255],
            glyphs: vec![Glyph {
                font: 36,
                code: 'A' as u16,
                rect: [0, 0, 2, 1],
                advance: 1.,
            }],
            kerning: vec![([36, 'A' as u16, 'V' as u16], -2)],
        }
    }

    #[test]
    fn launcher_strings_of_every_locale_are_wanted() {
        let characters = launcher_characters();
        for ch in ['A', 'ü', '设', 'ต'] {
            assert!(characters.contains(&ch), "{ch}");
        }
    }

    #[test]
    fn caches_are_written_whole_and_read_back() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join(CACHE_FILE_NAME);
        write_cache(&path, &sample(7)).unwrap();
        assert_eq!(
            GameFont::decode(&fs::read(&path).unwrap()).unwrap(),
            sample(7)
        );
        assert!(!path.with_extension("partial").exists());
    }

    #[test]
    fn an_installed_font_from_the_same_archives_is_not_rebuilt() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("data")).unwrap();
        fs::write(dir.path().join("data").join("data009.cpk"), b"cpk").unwrap();
        let cache = dir.path().join(CACHE_FILE_NAME);
        let stamp = font::source_stamp(dir.path()).unwrap();
        assert!(load(dir.path(), &cache, Some(stamp)).is_none());
        assert!(load(dir.path(), &cache, None).is_none());
        assert!(!cache.exists());
    }
}
