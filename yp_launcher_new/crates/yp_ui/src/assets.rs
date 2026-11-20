use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use gpui::{App, AssetSource, ImageSource, Resource, Result, SharedString};

#[derive(rust_embed::RustEmbed)]
#[folder = "../../assets"]
#[include = "fonts/*.ttf"]
#[include = "icons/**/*.svg"]
#[cfg_attr(feature = "embedded-images", include = "images/**/*")]
#[cfg_attr(all(feature = "embedded-docs", feature = "docs-library"), include = "docs/**/*")]
#[cfg_attr(
    all(feature = "embedded-docs", not(feature = "docs-library")),
    include = "docs/native/35_third_party.md"
)]
#[include = "icon/*"]
#[include = "licenses/*.txt"]
struct Embedded;

pub const UNRAR_LICENSE: &str = "licenses/UnRAR-license.txt";
const IMAGES_PREFIX: &str = "images/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedFile {
    pub path: String,
    pub data: Cow<'static, [u8]>,
    pub sha256: [u8; 32],
}

#[derive(Debug, Clone, Default)]
pub struct Assets {
    folder: Option<PathBuf>,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(file) = Embedded::get(path) {
            return Ok(Some(file.data));
        }
        Ok(self.read_from_folder(path).map(Cow::Owned))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Embedded::iter()
            .filter(|entry| entry.starts_with(path))
            .map(|entry| SharedString::from(entry.to_string()))
            .collect())
    }
}

impl Assets {
    pub fn embedded() -> Self {
        Self::default()
    }

    pub fn with_folder(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: Some(folder.into()),
        }
    }

    fn read_from_folder(&self, path: &str) -> Option<Vec<u8>> {
        let folder = self.folder.as_deref()?;
        let relative = Path::new(path);
        let stays_inside = relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
        if !path.starts_with(IMAGES_PREFIX) || !stays_inside {
            return None;
        }
        match std::fs::read(folder.join(relative)) {
            Ok(bytes) => Some(bytes),
            Err(error) => {
                log::warn!("launcher asset {path} is not in {}: {error}", folder.display());
                None
            }
        }
    }

    pub fn font_paths() -> Vec<String> {
        let mut paths: Vec<String> = Embedded::iter()
            .filter(|path| path.starts_with("fonts/") && path.ends_with(".ttf"))
            .map(|path| path.to_string())
            .collect();
        paths.sort();
        paths
    }

    pub fn exists(path: &str) -> bool {
        Embedded::get(path).is_some()
    }

    pub fn embedded_under(prefix: &str) -> Vec<EmbeddedFile> {
        let mut files: Vec<EmbeddedFile> = Embedded::iter()
            .filter(|path| path.starts_with(prefix))
            .filter_map(|path| {
                let file = Embedded::get(&path)?;
                Some(EmbeddedFile {
                    path: path.to_string(),
                    sha256: file.metadata.sha256_hash(),
                    data: file.data,
                })
            })
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        files
    }

    pub fn license_paths() -> Vec<String> {
        let mut paths: Vec<String> = Embedded::iter()
            .filter(|path| path.starts_with("licenses/"))
            .map(|path| path.to_string())
            .collect();
        paths.sort();
        paths
    }
}

pub fn load_fonts(cx: &App) -> Result<()> {
    let fonts = Assets::font_paths()
        .into_iter()
        .filter_map(|path| Embedded::get(&path).map(|file| file.data))
        .collect();
    cx.text_system().add_fonts(fonts)
}

pub fn asset_image(path: impl Into<SharedString>) -> ImageSource {
    ImageSource::Resource(Resource::Embedded(path.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::IconName;

    #[test]
    fn every_icon_name_points_at_an_embedded_svg() {
        for icon in IconName::ALL {
            let path = icon.path();
            let data = Assets::embedded()
                .load(&path)
                .unwrap()
                .unwrap_or_else(|| panic!("{path} is missing"));
            let text = std::str::from_utf8(&data).unwrap();
            assert!(text.contains("<svg"), "{path} is not an svg");
            assert_eq!(IconName::from_name(icon.name()), Some(icon));
        }
        assert_eq!(IconName::from_name("no_such_icon"), None);
    }

    #[test]
    fn the_unrar_licence_ships_inside_the_launcher() {
        assert_eq!(Assets::license_paths(), [UNRAR_LICENSE]);
        let data = Assets::embedded()
            .load(UNRAR_LICENSE)
            .unwrap()
            .expect("the UnRAR licence is embedded");
        let text = std::str::from_utf8(&data).unwrap();
        let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for required in [
            "UnRAR source code may be used in any software to handle RAR archives without limitations free of charge,",
            "but cannot be used to develop RAR (WinRAR) compatible archiver and to re-create RAR compression algorithm,",
            "is included in license, or in documentation if license is not available,",
            "Alexander L. Roshal",
        ] {
            assert!(collapsed.contains(required), "{required}");
        }
        let on_disk = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets")
            .join(UNRAR_LICENSE);
        assert_eq!(std::fs::read(on_disk).unwrap(), data.as_ref());
    }

    #[test]
    fn icon_names_are_unique() {
        let mut names: Vec<&str> = IconName::ALL.into_iter().map(IconName::name).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total);
    }

    #[test]
    fn the_five_rajdhani_weights_and_the_ornaments_are_bundled() {
        let fonts = Assets::font_paths();
        assert_eq!(fonts.len(), 5);
        assert!(fonts.iter().all(|path| path.starts_with("fonts/Rajdhani-")));
        assert!(Assets::exists("icons/extra/nier_rings_outer.svg"));
        assert!(Assets::exists("icons/extra/nier_border.svg"));
        assert!(!Assets::exists("icons/extra/does_not_exist.svg"));
        let listed = Assets::embedded().list("icons/extra/").unwrap();
        assert!(listed.len() >= 7);
    }

    #[test]
    fn preview_images_are_only_embedded_with_the_feature() {
        let crosshair = "images/config/crosshair.webp";
        assert_eq!(
            Assets::embedded().load(crosshair).unwrap().is_some(),
            cfg!(feature = "embedded-images")
        );
        let repo_assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let on_disk = std::fs::read(repo_assets.join(crosshair)).unwrap();
        let loaded = Assets::with_folder(&repo_assets).load(crosshair).unwrap();
        assert_eq!(loaded.as_deref(), Some(on_disk.as_slice()));
    }

    #[test]
    fn the_docs_are_only_embedded_with_the_feature() {
        let docs = Assets::embedded_under("docs/");
        assert_eq!(!docs.is_empty(), cfg!(feature = "embedded-docs"));
        if cfg!(all(feature = "embedded-docs", not(feature = "docs-library"))) {
            let paths: Vec<&str> = docs.iter().map(|file| file.path.as_str()).collect();
            assert_eq!(paths, ["docs/native/35_third_party.md"]);
        }
        let repo_docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        for file in &docs {
            let on_disk = std::fs::read(repo_docs.join(&file.path)).unwrap();
            assert_eq!(file.data.as_ref(), on_disk.as_slice(), "{}", file.path);
        }
        let icons = Assets::embedded_under("icons/extra/");
        assert!(icons.len() >= 7);
        assert!(icons.windows(2).all(|pair| pair[0].path < pair[1].path));
    }

    #[test]
    fn the_assets_folder_serves_images_and_nothing_else() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("images").join("config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(config.join("only-in-folder.webp"), b"webp bytes").unwrap();
        std::fs::write(root.path().join("images").join("top.png"), b"png bytes").unwrap();
        std::fs::create_dir_all(root.path().join("fonts")).unwrap();
        std::fs::write(root.path().join("fonts").join("Other.ttf"), b"font").unwrap();
        std::fs::write(root.path().join("secret.txt"), b"secret").unwrap();
        let assets = Assets::with_folder(root.path());
        let load = |path: &str| assets.load(path).unwrap().map(|bytes| bytes.into_owned());

        assert_eq!(load("images/config/only-in-folder.webp"), Some(b"webp bytes".to_vec()));
        assert_eq!(load("images/top.png"), Some(b"png bytes".to_vec()));
        assert_eq!(load("images/config/missing.webp"), None);
        assert_eq!(load("fonts/Other.ttf"), None);
        assert_eq!(load("secret.txt"), None);
        assert_eq!(load("images/../secret.txt"), None);
        assert_eq!(Assets::embedded().load("images/config/only-in-folder.webp").unwrap(), None);
        assert!(load("icons/extra/nier_border.svg").is_some());
    }
}
