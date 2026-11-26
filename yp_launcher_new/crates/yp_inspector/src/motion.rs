#![forbid(unsafe_code)]

mod packed;

use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use yp_formats::DatArchive;
use yp_formats::mot::Motion;
use yp_formats::wmb3::Skeleton;

use crate::ArchivePaths;

const MAX_CLIP_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MotionSources {
    pub mod_root: Option<PathBuf>,
    pub game_data: Option<PathBuf>,
    pub mod_libraries: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
enum Source {
    Disk {
        path: PathBuf,
        entry: Option<u32>,
    },
    Packed {
        bank: Arc<[u8]>,
        range: Range<usize>,
    },
}

impl Source {
    fn read(&self, name: &str, skeleton: &Skeleton) -> Option<Motion> {
        let (path, entry) = match self {
            Self::Packed { bank, range } => {
                let motion = Motion::parse(bank.get(range.clone())?)?;
                return skeleton.accepts(&motion).then_some(motion);
            }
            Self::Disk { path, entry } => (path, *entry),
        };
        let bytes = if let Some(index) = entry {
            let mut archive = DatArchive::open(path).ok()?;
            let entry = archive.entry(index)?.clone();
            if entry.size > MAX_CLIP_SIZE || !entry.name.eq_ignore_ascii_case(name) {
                return None;
            }
            archive.read_entry(&entry).ok()?
        } else {
            let file = File::open(path).ok()?;
            if file.metadata().ok()?.len() > MAX_CLIP_SIZE {
                return None;
            }
            let mut bytes = Vec::new();
            file.take(MAX_CLIP_SIZE + 1).read_to_end(&mut bytes).ok()?;
            if bytes.len() as u64 > MAX_CLIP_SIZE {
                return None;
            }
            bytes
        };
        let motion = Motion::parse(&bytes)?;
        skeleton.accepts(&motion).then_some(motion)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Clip {
    name: String,
    sources: Vec<Source>,
    priority: (u8, u8),
}

impl Clip {
    fn read(&self, skeleton: &Skeleton) -> Option<Motion> {
        self.sources
            .iter()
            .find_map(|source| source.read(&self.name, skeleton))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Library {
    clips: Vec<Clip>,
    pub selected: Option<(usize, Motion)>,
}

impl Library {
    pub fn open(
        paths: &ArchivePaths,
        sources: &MotionSources,
        stem: &str,
        skeleton: &Skeleton,
    ) -> Self {
        let mut candidates = BTreeMap::<String, Clip>::new();
        let mut scanned = HashSet::new();
        let owners = Owners::new(stem);
        let mut visit = |path: &Path, rank: u8| {
            if !scanned.insert(path.to_path_buf()) {
                return;
            }
            let mut record = |name: &str, entry: Option<u32>| {
                owners.record(
                    &mut candidates,
                    name,
                    rank,
                    Source::Disk {
                        path: path.to_owned(),
                        entry,
                    },
                );
            };
            match path
                .extension()
                .and_then(|s| s.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("mot") => {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        record(name, None);
                    }
                }
                Some("dat" | "dtt") => {
                    if let Ok(archive) = DatArchive::open(path) {
                        for entry in archive
                            .entries()
                            .iter()
                            .filter(|e| e.size <= MAX_CLIP_SIZE && e.has_extension("mot"))
                        {
                            record(&entry.name, Some(entry.index));
                        }
                    }
                }
                _ => {}
            }
        };
        for path in [&paths.dat, &paths.dtt].into_iter().flatten() {
            visit(path, 0);
        }
        let model_dir = paths
            .dat
            .as_ref()
            .or(paths.dtt.as_ref())
            .and_then(|p| p.parent());
        if let Some(dir) = sources.mod_root.as_deref().or(model_dir) {
            let mut files = Vec::new();
            let mut dirs = vec![dir.to_path_buf()];
            while let Some(dir) = dirs.pop() {
                let Ok(entries) = fs::read_dir(dir) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let Ok(kind) = entry.file_type() else {
                        continue;
                    };
                    if kind.is_dir() && sources.mod_root.is_some() {
                        dirs.push(entry.path());
                    } else if kind.is_file()
                        && entry
                            .path()
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|ext| {
                                ["mot", "dat", "dtt"]
                                    .iter()
                                    .any(|s| ext.eq_ignore_ascii_case(s))
                            })
                    {
                        files.push(entry.path());
                    }
                }
            }
            files.sort();
            for path in files {
                visit(&path, 1);
            }
        }
        let mut banks = Vec::new();
        if let Some(family) = owners.family.as_deref() {
            banks.push(bank_name(family));
        }
        if let Some(shared) = owners.shared.as_deref()
            && Some(shared) != owners.family.as_deref()
        {
            banks.push(bank_name(shared));
        }
        if !banks.is_empty() {
            let visit_library = |library: &Path, rank, visit: &mut dyn FnMut(&Path, u8)| {
                let Ok(entries) = fs::read_dir(library) else {
                    return;
                };
                let mut mods: Vec<_> = entries
                    .flatten()
                    .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                    .map(|entry| entry.path())
                    .collect();
                mods.sort();
                for folder in mods {
                    for sub in ["data/pl", "pl"] {
                        for bank in &banks {
                            visit(&folder.join(sub).join(bank), rank);
                        }
                    }
                }
            };
            let mut libraries = sources.mod_libraries.iter();
            if let Some(active) = libraries.next() {
                visit_library(active, 2, &mut visit);
            }
            if let Some(data) = &sources.game_data {
                for bank in &banks {
                    visit(&data.join("pl").join(bank), 3);
                }
            }
            for library in libraries {
                visit_library(library, 4, &mut visit);
            }
        }
        let local = Self::from_candidates(candidates, skeleton);
        if local.selected.is_some() {
            return local;
        }
        if let (Some(data), Some(bank_name)) = (&sources.game_data, shared_player_bank(stem)) {
            for path in packed::packages(data) {
                let Some(bank) = packed::bank(&path, &bank_name) else {
                    continue;
                };
                let Ok(archive) = DatArchive::from_reader(Cursor::new(bank.as_ref())) else {
                    continue;
                };
                let mut candidates = BTreeMap::new();
                for entry in archive
                    .entries()
                    .iter()
                    .filter(|e| e.size <= MAX_CLIP_SIZE && e.has_extension("mot"))
                {
                    let start = entry.offset as usize;
                    let end = start + entry.size as usize;
                    owners.record(
                        &mut candidates,
                        &entry.name,
                        5,
                        Source::Packed {
                            bank: bank.clone(),
                            range: start..end,
                        },
                    );
                }
                let fallback = Self::from_candidates(candidates, skeleton);
                if fallback.selected.is_some() {
                    return fallback;
                }
            }
        }
        local
    }

    fn from_candidates(candidates: BTreeMap<String, Clip>, skeleton: &Skeleton) -> Self {
        let mut clips: Vec<_> = candidates.into_values().collect();
        clips.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
        });
        let selected = clips
            .iter()
            .enumerate()
            .find_map(|(i, clip)| clip.read(skeleton).map(|motion| (i, motion)));
        Self { clips, selected }
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.clips.iter().map(|c| c.name.as_str())
    }

    pub fn choose(&mut self, index: usize, skeleton: &Skeleton) -> bool {
        let Some(clip) = self.clips.get(index) else {
            return false;
        };
        let Some(motion) = clip.read(skeleton) else {
            return false;
        };
        self.selected = Some((index, motion));
        true
    }
}

fn is_model_id(stem: &str) -> bool {
    stem.len() == 6
        && stem.as_bytes()[..2].iter().all(u8::is_ascii_alphabetic)
        && stem.as_bytes()[2..].iter().all(u8::is_ascii_hexdigit)
}

pub fn shared_player_bank(stem: &str) -> Option<String> {
    let owners = Owners::new(stem);
    owners.shared.as_deref().map(bank_name)
}

fn bank_name(family: &str) -> String {
    format!("{}f.dat", &family[..5])
}

struct Owners<'a> {
    stem: &'a str,
    family: Option<String>,
    shared: Option<String>,
}

impl<'a> Owners<'a> {
    fn new(stem: &'a str) -> Self {
        let family =
            (is_model_id(stem) && stem.starts_with("pl")).then(|| format!("{}0", &stem[..5]));
        let shared = match family.as_deref() {
            Some("pl0100") => Some("pl0000".to_owned()),
            _ => family.clone(),
        };
        Self {
            stem,
            family,
            shared,
        }
    }

    fn record(
        &self,
        candidates: &mut BTreeMap<String, Clip>,
        name: &str,
        rank: u8,
        source: Source,
    ) {
        let lower = name.to_ascii_lowercase();
        let Some(id) = lower.strip_suffix(".mot") else {
            return;
        };
        let owner = id.rsplit_once('_').map(|(owner, _)| owner);
        let matches = owner.is_some_and(|s| {
            s == self.stem || Some(s) == self.family.as_deref() || Some(s) == self.shared.as_deref()
        });
        if !matches
            && (rank > 1
                || ((rank == 1 || self.family.is_some()) && owner.is_some_and(is_model_id)))
        {
            return;
        }
        let preference = if matches && self.family.is_some() && id.ends_with("_003f") {
            0
        } else if id.contains("idle") || id.ends_with("_0000") {
            1
        } else {
            2
        };
        let clip = candidates.entry(lower).or_insert_with(|| Clip {
            name: name.to_owned(),
            sources: Vec::new(),
            priority: (rank, preference),
        });
        clip.sources.push(source);
    }
}
