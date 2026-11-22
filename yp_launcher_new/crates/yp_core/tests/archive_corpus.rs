mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

use support::listing_mismatch;
use yp_core::archive::{ArchiveFormat, Extractor, is_archive};
use yp_core::{CancelToken, NoProgress};

const CORPUS_DIR_VAR: &str = "YP_CORPUS_DIR";
const CORPUS_FILTER_VAR: &str = "YP_CORPUS_FILTER";
const CORPUS_JOBS_VAR: &str = "YP_CORPUS_JOBS";
const CORPUS_OEM_CODE_PAGE_VAR: &str = "YP_CORPUS_OEM_CODE_PAGE";
const REFERENCE_TOOL_VAR: &str = "YP_CORPUS_REFERENCE_7Z";
const DEFAULT_JOBS: usize = 4;

fn corpus_archives(root: &Path, filter: Option<&str>) -> Vec<PathBuf> {
    let mut archives: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_archive(entry.path()))
        .map(walkdir::DirEntry::into_path)
        .filter(|path| filter.is_none_or(|needle| path.to_string_lossy().contains(needle)))
        .collect();
    archives.sort();
    archives
}

type Fingerprint = BTreeMap<String, Option<(u64, [u8; 32])>>;

fn fingerprint(root: &Path) -> Result<Fingerprint, String> {
    let mut found = Fingerprint::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1) {
        let entry = entry.map_err(|error| error.to_string())?;
        let relative = yp_core::fs::rel_slash(root, entry.path()).ok_or("entry outside the root")?;
        if entry.file_type().is_dir() {
            found.insert(relative, None);
        } else {
            let size = entry.metadata().map_err(|error| error.to_string())?.len();
            let hash = yp_core::hash::sha256_file(entry.path()).map_err(|error| error.to_string())?;
            found.insert(relative, Some((size, hash)));
        }
    }
    Ok(found)
}

fn first_difference(ours: &Fingerprint, reference: &Fingerprint) -> Option<String> {
    for (path, entry) in reference {
        match ours.get(path) {
            None => return Some(format!("only the reference tool wrote {path}")),
            Some(found) if found != entry => return Some(format!("content differs for {path}")),
            Some(_) => {}
        }
    }
    ours.keys()
        .find(|path| !reference.contains_key(*path))
        .map(|path| format!("only the launcher wrote {path}"))
}

fn reference_tree(tool: &Path, archive: &Path, dest: &Path) -> Result<Fingerprint, String> {
    let mut out_flag = std::ffi::OsString::from("-o");
    out_flag.push(dest.as_os_str());
    let status = Command::new(tool)
        .arg("x")
        .arg(archive)
        .arg(out_flag)
        .args(["-y", "-bso0", "-bsp0", "-bse0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("reference tool did not start: {error}"))?;
    if !status.success() {
        return Err(format!("reference tool failed with {status}"));
    }
    fingerprint(dest)
}

struct Checked {
    format: ArchiveFormat,
    files: u64,
    bytes: u64,
    compared: bool,
}

fn check(extractor: &Extractor, archive: &Path, reference: Option<&Path>) -> Result<Checked, String> {
    let format = ArchiveFormat::detect(archive).map_err(|error| format!("detect: {error}"))?;
    let listing = extractor.list(archive).map_err(|error| format!("list: {error}"))?;
    let work = tempfile::Builder::new()
        .prefix("yp_corpus_")
        .tempdir()
        .map_err(|error| error.to_string())?;
    let ours = work.path().join("ours");
    extractor
        .extract(archive, &ours, &NoProgress, &CancelToken::new())
        .map_err(|error| format!("extract: {error}"))?;
    if let Some(mismatch) = listing_mismatch(&listing, &ours) {
        return Err(mismatch);
    }
    let files = listing.iter().filter(|entry| !entry.is_dir).count() as u64;
    let bytes = listing.iter().map(|entry| entry.size).sum();
    let mut compared = false;
    if let Some(tool) = reference {
        let theirs = work.path().join("reference");
        std::fs::create_dir_all(&theirs).map_err(|error| error.to_string())?;
        let expected = reference_tree(tool, archive, &theirs)?;
        let found = fingerprint(&ours)?;
        if let Some(difference) = first_difference(&found, &expected) {
            return Err(difference);
        }
        compared = true;
    }
    Ok(Checked {
        format,
        files,
        bytes,
        compared,
    })
}

#[test]
#[ignore = "needs the archive corpus (YP_CORPUS_DIR)"]
fn every_corpus_archive_lists_and_extracts_as_listed() {
    let Some(root) = std::env::var_os(CORPUS_DIR_VAR).map(PathBuf::from) else {
        eprintln!("skipped: {CORPUS_DIR_VAR} is not set");
        return;
    };
    let filter = std::env::var(CORPUS_FILTER_VAR).ok();
    let archives = corpus_archives(&root, filter.as_deref());
    assert!(!archives.is_empty(), "no archives below {}", root.display());
    let reference = std::env::var_os(REFERENCE_TOOL_VAR)
        .map(PathBuf::from)
        .filter(|tool| tool.is_file());
    let code_page = std::env::var(CORPUS_OEM_CODE_PAGE_VAR)
        .ok()
        .and_then(|value| value.parse().ok());
    let extractor = Extractor::with_oem_code_page(code_page);
    let jobs = std::env::var(CORPUS_JOBS_VAR)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_JOBS)
        .max(1);

    let next = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let per_format = Mutex::new(BTreeMap::<&str, u64>::new());
    let files = AtomicU64::new(0);
    let bytes = AtomicU64::new(0);
    let compared = AtomicU64::new(0);
    let started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(archive) = archives.get(index) else {
                        return;
                    };
                    match check(&extractor, archive, reference.as_deref()) {
                        Ok(checked) => {
                            *per_format
                                .lock()
                                .unwrap()
                                .entry(checked.format.extension())
                                .or_default() += 1;
                            files.fetch_add(checked.files, Ordering::SeqCst);
                            bytes.fetch_add(checked.bytes, Ordering::SeqCst);
                            compared.fetch_add(u64::from(checked.compared), Ordering::SeqCst);
                        }
                        Err(failure) => {
                            eprintln!("FAIL {}: {failure}", archive.display());
                            failures.lock().unwrap().push(format!("{}: {failure}", archive.display()));
                        }
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    eprintln!(
        "corpus: {} archives {:?}, {} files, {} bytes, {} compared with the reference tool, {} failed, {:.1} s with {jobs} jobs",
        archives.len(),
        per_format.into_inner().unwrap(),
        files.load(Ordering::SeqCst),
        bytes.load(Ordering::SeqCst),
        compared.load(Ordering::SeqCst),
        failures.len(),
        started.elapsed().as_secs_f64()
    );
    assert!(
        failures.is_empty(),
        "{} of {} archives failed:\n{}",
        failures.len(),
        archives.len(),
        failures.join("\n")
    );
}

#[test]
fn corpus_discovery_keeps_archives_at_any_depth() {
    let dir = tempfile::tempdir().unwrap();
    for relative in ["100/A.7z", "200/b.zip", "shaders/Pack/c.rar", "200/readme.txt", "300/d.tar.gz"] {
        let path = relative.split('/').fold(dir.path().to_path_buf(), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "x").unwrap();
    }
    let found: Vec<String> = corpus_archives(dir.path(), None)
        .iter()
        .map(|path| yp_core::fs::rel_slash(dir.path(), path).unwrap())
        .collect();
    assert_eq!(found, ["100/A.7z", "200/b.zip", "shaders/Pack/c.rar"]);
    assert_eq!(corpus_archives(dir.path(), Some("Pack")).len(), 1);
}

#[test]
fn tree_comparison_names_the_first_difference() {
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    for root in [a.path(), b.path()] {
        std::fs::create_dir_all(root.join("mod").join("empty")).unwrap();
        std::fs::write(root.join("mod").join("same.bin"), "same").unwrap();
    }
    assert_eq!(first_difference(&fingerprint(a.path()).unwrap(), &fingerprint(b.path()).unwrap()), None);
    std::fs::write(b.path().join("mod").join("same.bin"), "diff").unwrap();
    assert_eq!(
        first_difference(&fingerprint(a.path()).unwrap(), &fingerprint(b.path()).unwrap()).as_deref(),
        Some("content differs for mod/same.bin")
    );
    std::fs::write(b.path().join("mod").join("same.bin"), "same").unwrap();
    std::fs::write(a.path().join("extra.txt"), "x").unwrap();
    assert_eq!(
        first_difference(&fingerprint(a.path()).unwrap(), &fingerprint(b.path()).unwrap()).as_deref(),
        Some("only the launcher wrote extra.txt")
    );
    std::fs::remove_dir(a.path().join("mod").join("empty")).unwrap();
    assert_eq!(
        first_difference(&fingerprint(a.path()).unwrap(), &fingerprint(b.path()).unwrap()).as_deref(),
        Some("only the reference tool wrote mod/empty")
    );
}
