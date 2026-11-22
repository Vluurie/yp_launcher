use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use yp_ui::{Assets, EmbeddedFile};

pub const EMBEDDED_PREFIX: &str = "docs/";
pub const UNPACKED_DIR_NAME: &str = "embedded_docs";
const STAMP_FILE_NAME: &str = ".embedded";

pub fn unpack_embedded(into: &Path) -> std::io::Result<Option<PathBuf>> {
    let files = Assets::embedded_under(EMBEDDED_PREFIX);
    if files.is_empty() {
        return Ok(None);
    }
    unpack(&files, into)?;
    Ok(Some(into.to_path_buf()))
}

fn stamp_of(files: &[EmbeddedFile]) -> String {
    let mut stamp = String::new();
    for file in files {
        for byte in file.sha256 {
            let _ = write!(stamp, "{byte:02x}");
        }
        let _ = writeln!(stamp, " {}", file.path);
    }
    stamp
}

fn unpack(files: &[EmbeddedFile], into: &Path) -> std::io::Result<()> {
    let stamp = stamp_of(files);
    let stamp_file = into.join(STAMP_FILE_NAME);
    if std::fs::read_to_string(&stamp_file).is_ok_and(|found| found == stamp) {
        return Ok(());
    }
    if into.exists() {
        std::fs::remove_dir_all(into)?;
    }
    for file in files {
        let relative = file.path.strip_prefix(EMBEDDED_PREFIX).unwrap_or(&file.path);
        let target = relative
            .split('/')
            .filter(|part| !part.is_empty() && *part != "." && *part != "..")
            .fold(into.to_path_buf(), |path, part| path.join(part));
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, &file.data)?;
    }
    std::fs::write(stamp_file, stamp)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    fn file(path: &str, text: &'static str, mark: u8) -> EmbeddedFile {
        EmbeddedFile {
            path: path.to_owned(),
            data: Cow::Borrowed(text.as_bytes()),
            sha256: [mark; 32],
        }
    }

    #[test]
    fn embedded_docs_land_on_disk_once_and_follow_the_embedded_set() {
        let root = tempfile::tempdir().unwrap();
        let into = root.path().join(UNPACKED_DIR_NAME);
        let first = [
            file("docs/native/01_first_mod.md", "# First", 1),
            file("docs/images/shop.png", "png", 2),
        ];
        unpack(&first, &into).unwrap();
        let page = into.join("native").join("01_first_mod.md");
        assert_eq!(std::fs::read_to_string(&page).unwrap(), "# First");
        assert_eq!(std::fs::read(into.join("images").join("shop.png")).unwrap(), b"png");

        std::fs::write(&page, "edited on disk").unwrap();
        unpack(&first, &into).unwrap();
        assert_eq!(std::fs::read_to_string(&page).unwrap(), "edited on disk");

        let second = [file("docs/native/01_first_mod.md", "# Second", 3)];
        unpack(&second, &into).unwrap();
        assert_eq!(std::fs::read_to_string(&page).unwrap(), "# Second");
        assert!(!into.join("images").exists());
    }

    #[test]
    fn nothing_is_unpacked_when_no_docs_are_embedded() {
        let root = tempfile::tempdir().unwrap();
        let into = root.path().join(UNPACKED_DIR_NAME);
        let unpacked = unpack_embedded(&into).unwrap();
        assert_eq!(unpacked.is_some(), !Assets::embedded_under(EMBEDDED_PREFIX).is_empty());
        assert_eq!(into.exists(), unpacked.is_some());
    }
}
