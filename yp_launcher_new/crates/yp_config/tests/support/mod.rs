#![allow(dead_code)]

use std::path::Path;

use tempfile::TempDir;
use yp_config::ensure_configs;
use yp_core::{GamePaths, HostEnv, Os};

pub struct GameDir {
    pub dir: TempDir,
    pub game: GamePaths,
    pub host: HostEnv,
}

impl GameDir {
    pub fn empty() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let game = GamePaths::new(dir.path().join("game"));
        std::fs::create_dir_all(game.root()).unwrap();
        let host = HostEnv::for_test(Os::Windows, &[], &dir.path().join("host"));
        Self { dir, game, host }
    }

    pub fn with_nams(nams_toml: Option<&str>) -> Self {
        let game_dir = Self::empty();
        if let Some(text) = nams_toml {
            game_dir.write_nams_file("nams.toml", text);
        }
        game_dir
    }

    pub fn root(&self) -> &Path {
        self.game.root()
    }

    pub fn write_nams_file(&self, name: &str, text: &str) {
        std::fs::create_dir_all(self.game.nams_dir()).unwrap();
        std::fs::write(self.game.nams_dir().join(name), text).unwrap();
    }

    pub fn read_nams_file(&self, name: &str) -> String {
        std::fs::read_to_string(self.game.nams_dir().join(name)).unwrap()
    }

    pub fn ensure(&self) {
        ensure_configs(&self.game, &self.host).unwrap();
    }

    pub fn raw_nams(&self) -> String {
        self.read_nams_file("nams.toml")
    }

    pub fn raw_naiom(&self) -> String {
        self.read_nams_file("naiom.toml")
    }

    pub fn nams(&self) -> toml::Table {
        parse(&self.raw_nams())
    }

    pub fn naiom(&self) -> toml::Table {
        parse(&self.raw_naiom())
    }
}

pub fn parse(raw: &str) -> toml::Table {
    toml::from_str(raw).unwrap_or_else(|err| panic!("invalid TOML ({err}):\n{raw}"))
}

pub fn table<'a>(values: &'a toml::Table, key: &str) -> &'a toml::Table {
    values
        .get(key)
        .and_then(toml::Value::as_table)
        .unwrap_or_else(|| panic!("[{key}] missing"))
}

pub fn index_of(haystack: &str, needle: &str) -> usize {
    haystack
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not found in:\n{haystack}"))
}

pub fn index_of_opt(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}
