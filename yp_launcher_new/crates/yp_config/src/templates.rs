use crate::files::ConfigFile;

pub const NAMS: &str = include_str!("../templates/nams.toml");
pub const NAIOM: &str = include_str!("../templates/naiom.toml");
pub const LODMOD: &str = include_str!("../templates/lodmod.toml");
pub const TEXTURE_INJECTION: &str = include_str!("../templates/texture_injection.toml");

pub const NAMS_ADDITIONS: &str = include_str!("../templates/migrations/nams_additions.toml");
pub const NAMS_CUTSCENE_KEYS: &str =
    include_str!("../templates/migrations/nams_cutscene_keys.toml");
pub const NAMS_CUTSCENE_SECTION: &str =
    include_str!("../templates/migrations/nams_cutscene_section.toml");
pub const TEXTURE_INJECTION_ADDITIONS: &str =
    include_str!("../templates/migrations/texture_injection_additions.toml");

pub fn template(file: ConfigFile) -> &'static str {
    match file {
        ConfigFile::Nams => NAMS,
        ConfigFile::Naiom => NAIOM,
        ConfigFile::LodMod => LODMOD,
        ConfigFile::TextureInjection => TEXTURE_INJECTION,
    }
}

pub fn shipped() -> [(ConfigFile, &'static str); 4] {
    ConfigFile::ALL.map(|file| (file, template(file)))
}

pub fn migration_templates() -> [&'static str; 4] {
    [
        NAMS_ADDITIONS,
        NAMS_CUTSCENE_KEYS,
        NAMS_CUTSCENE_SECTION,
        TEXTURE_INJECTION_ADDITIONS,
    ]
}
