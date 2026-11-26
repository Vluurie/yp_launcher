use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::SystemTime;

type Cache = HashMap<(PathBuf, String), CachedBank>;

struct CachedBank {
    size: u64,
    modified: SystemTime,
    bytes: Weak<[u8]>,
}

pub(super) fn packages(data: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(data) else {
        return Vec::new();
    };
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("cpk"))
        })
        .collect();
    paths.sort();
    paths
}

pub(super) fn bank(path: &Path, name: &str) -> Option<Arc<[u8]>> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    let cache = CACHE.get_or_init(Mutex::default);
    let key = (path.to_owned(), name.to_owned());
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok();
    if let Ok(banks) = cache.lock()
        && let Some(hit) = banks.get(&key)
        && Some(hit.modified) == modified
        && hit.size == metadata.len()
        && let Some(bytes) = hit.bytes.upgrade()
    {
        return Some(bytes);
    }
    let bytes: Arc<[u8]> = yp_formats::cpk::read_member(path, "pl", name, 64 * 1024 * 1024)?.into();
    if let Some(modified) = modified
        && let Ok(mut banks) = cache.lock()
    {
        banks.retain(|_, hit| hit.bytes.strong_count() > 0);
        banks.insert(
            key,
            CachedBank {
                size: metadata.len(),
                modified,
                bytes: Arc::downgrade(&bytes),
            },
        );
    }
    Some(bytes)
}
