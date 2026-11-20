use std::path::PathBuf;

use yp_core::GamePaths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConfigFile {
    Nams,
    Naiom,
    LodMod,
    TextureInjection,
}

impl ConfigFile {
    pub const ALL: [ConfigFile; 4] = [
        ConfigFile::Nams,
        ConfigFile::Naiom,
        ConfigFile::LodMod,
        ConfigFile::TextureInjection,
    ];

    pub fn file_name(self) -> &'static str {
        match self {
            ConfigFile::Nams => "nams.toml",
            ConfigFile::Naiom => "naiom.toml",
            ConfigFile::LodMod => "lodmod.toml",
            ConfigFile::TextureInjection => "texture_injection.toml",
        }
    }

    pub fn from_file_name(name: &str) -> Option<ConfigFile> {
        ConfigFile::ALL
            .into_iter()
            .find(|file| file.file_name() == name)
    }

    pub fn path(self, game: &GamePaths) -> PathBuf {
        match self {
            ConfigFile::Nams => game.nams_toml(),
            ConfigFile::Naiom => game.naiom_toml(),
            ConfigFile::LodMod => game.lodmod_toml(),
            ConfigFile::TextureInjection => game.texture_injection_toml(),
        }
    }
}
