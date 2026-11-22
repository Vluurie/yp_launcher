use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use sha2::{Digest, Sha256};

const ARCHIVE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures.zip");
const CACHE_DIR_NAME: &str = "yp_launcher_test_fixtures";

static ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn root() -> &'static Path {
    ROOT.get_or_init(|| unpack().unwrap_or_else(|error| panic!("{ARCHIVE}: {error}")))
}

pub fn path(name: &str) -> PathBuf {
    name.split('/')
        .fold(root().to_path_buf(), |path, part| path.join(part))
}

fn unpack() -> std::io::Result<PathBuf> {
    let bytes = std::fs::read(ARCHIVE)?;
    let version: String = Sha256::digest(&bytes)
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let cache = std::env::temp_dir().join(CACHE_DIR_NAME);
    let unpacked = cache.join(version);
    if unpacked.is_dir() {
        return Ok(unpacked);
    }
    std::fs::create_dir_all(&cache)?;
    let staging = tempfile::tempdir_in(&cache)?;
    zip::ZipArchive::new(Cursor::new(bytes))
        .and_then(|mut archive| archive.extract(staging.path()))
        .map_err(std::io::Error::other)?;
    match std::fs::rename(staging.path(), &unpacked) {
        Ok(()) => Ok(unpacked),
        Err(_) if unpacked.is_dir() => Ok(unpacked),
        Err(error) => Err(error),
    }
}
