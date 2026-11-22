use std::path::Path;

use yp_core::ArchiveFailure;

use crate::detect::is_loose_data_file;
use crate::fsx;
use crate::staging::DropStaging;
use crate::tables::is_data_dir_key;

pub const MAX_WRAPPER_STRIPS: usize = 4;
pub const MAX_SCAN_DEPTH: usize = 4;

pub const ROOT_DATA_SUBDIRS: [&str; 27] = [
    "pl",
    "wp",
    "em",
    "ba",
    "bg",
    "bh",
    "et",
    "it",
    "um",
    "wd1",
    "wd2",
    "wd3",
    "wd4",
    "wd5",
    "wda",
    "core",
    "ph1",
    "ph2",
    "ph3",
    "phf",
    "quest",
    "st1",
    "st2",
    "st5",
    "effect",
    "enlighten",
    "misctex",
];

const MANIFEST_NAME: &str = "mod.toml";
const ENTITIES_DIR: &str = "entities";
const WAX_DIR: &str = "wax";
const DATA_DIR: &str = "data";
const CUTSCENE_DIR: &str = "movie";
const TEXTURE_SUFFIX: &str = ".dds";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropGate {
    Mod,
    MisroutedCutscenes,
    NotAMod,
    UnreadableArchive(ArchiveFailure),
}

impl DropGate {
    pub fn is_mod(self) -> bool {
        self == DropGate::Mod
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntrySignal {
    Mod,
    Cutscene,
    Neutral,
}

pub fn classify(staging: &DropStaging, source: &Path) -> DropGate {
    if DropStaging::is_archive(source) {
        return match staging.list(source) {
            Ok(entries) => classify_entry_paths(entries),
            Err(error) => error
                .archive_failure()
                .map_or(DropGate::NotAMod, DropGate::UnreadableArchive),
        };
    }
    if source.is_file() {
        return if is_loose_data_file(&fsx::file_name(source)) {
            DropGate::Mod
        } else {
            DropGate::NotAMod
        };
    }
    if source.is_dir() {
        return classify_dir(source);
    }
    DropGate::NotAMod
}

pub fn classify_dir(dir: &Path) -> DropGate {
    if !dir.is_dir() {
        return DropGate::NotAMod;
    }
    let mut paths = Vec::new();
    collect_shallow_paths(dir, "", 1, &mut paths);
    classify_entry_paths(paths)
}

fn collect_shallow_paths(dir: &Path, prefix: &str, depth: usize, out: &mut Vec<String>) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    for child in fsx::children_no_follow(dir) {
        let relative = if prefix.is_empty() {
            child.name.clone()
        } else {
            format!("{prefix}/{}", child.name)
        };
        out.push(relative.clone());
        if child.is_dir {
            collect_shallow_paths(&child.path, &relative, depth + 1, out);
        }
    }
}

pub fn classify_entry_paths<I, S>(entries: I) -> DropGate
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let normalized: Vec<String> = entries
        .into_iter()
        .map(|entry| entry.as_ref().to_lowercase().replace('\\', "/"))
        .collect();
    let mut cutscene_content = false;
    for entry in strip_common_wrapper(normalized) {
        match entry_signal(&entry) {
            EntrySignal::Mod => return DropGate::Mod,
            EntrySignal::Cutscene => cutscene_content = true,
            EntrySignal::Neutral => {}
        }
    }
    if cutscene_content {
        DropGate::MisroutedCutscenes
    } else {
        DropGate::NotAMod
    }
}

fn is_loose_pl_file(name: &str) -> bool {
    name.starts_with("pl") && (name.ends_with(".dat") || name.ends_with(".dtt"))
}

fn is_overlay_data_pair(parent: &str, child: &str) -> bool {
    parent == DATA_DIR && child != CUTSCENE_DIR && is_data_dir_key(child)
}

fn entry_signal(entry: &str) -> EntrySignal {
    let segments: Vec<&str> = entry.split('/').collect();
    let (Some(first), Some(last)) = (segments.first().copied(), segments.last().copied()) else {
        return EntrySignal::Neutral;
    };
    if first.is_empty() {
        return EntrySignal::Neutral;
    }
    if is_loose_data_file(last) || last == MANIFEST_NAME || last.ends_with(TEXTURE_SUFFIX) {
        return EntrySignal::Mod;
    }
    let nested_marker = segments.windows(2).any(|pair| {
        pair[0] == ENTITIES_DIR || pair[0] == WAX_DIR || is_overlay_data_pair(pair[0], pair[1])
    });
    if nested_marker
        || first == MANIFEST_NAME
        || first == ENTITIES_DIR
        || first == WAX_DIR
        || is_loose_pl_file(first)
        || ROOT_DATA_SUBDIRS.contains(&first)
    {
        return EntrySignal::Mod;
    }
    if first == DATA_DIR && segments.len() >= 2 {
        return if segments[1] == CUTSCENE_DIR {
            EntrySignal::Cutscene
        } else {
            EntrySignal::Neutral
        };
    }
    if first == CUTSCENE_DIR {
        return EntrySignal::Cutscene;
    }
    EntrySignal::Neutral
}

fn root_segment(entry: &str) -> &str {
    entry.split('/').next().unwrap_or(entry)
}

fn is_root_marker(name: &str) -> bool {
    name == MANIFEST_NAME
        || name == ENTITIES_DIR
        || name == WAX_DIR
        || name == DATA_DIR
        || name == CUTSCENE_DIR
        || ROOT_DATA_SUBDIRS.contains(&name)
        || is_loose_data_file(name)
}

fn strip_common_wrapper(mut entries: Vec<String>) -> Vec<String> {
    for _ in 0..MAX_WRAPPER_STRIPS {
        match strip_one_wrapper(&entries) {
            Some(stripped) => entries = stripped,
            None => break,
        }
    }
    entries
}

fn strip_one_wrapper(entries: &[String]) -> Option<Vec<String>> {
    let wrapper = root_segment(entries.first()?);
    if entries.iter().any(|entry| root_segment(entry) != wrapper) {
        return None;
    }
    if wrapper.is_empty() || is_root_marker(wrapper) {
        return None;
    }
    Some(
        entries
            .iter()
            .filter_map(|entry| entry.strip_prefix(wrapper)?.strip_prefix('/'))
            .filter(|rest| !rest.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn root_data_subdirs_are_known_data_dirs_without_the_ambiguous_ones() {
        assert!(ROOT_DATA_SUBDIRS.iter().all(|name| is_data_dir_key(name)));
        for ambiguous in [
            "movie", "sound", "font", "ui", "novel", "subtitle", "txtmess",
        ] {
            assert!(is_data_dir_key(ambiguous));
            assert!(!ROOT_DATA_SUBDIRS.contains(&ambiguous));
        }
    }

    #[test]
    fn strips_up_to_four_shared_wrappers() {
        assert_eq!(
            strip_common_wrapper(owned(&["a", "a/b", "a/b/c", "a/b/c/x.txt", "a/b/c/y.txt"])),
            owned(&["x.txt", "y.txt"])
        );
        assert_eq!(
            strip_common_wrapper(owned(&["1/2/3/4/5/x.txt", "1/2/3/4/5/y.txt"])),
            owned(&["5/x.txt", "5/y.txt"])
        );
    }

    #[test]
    fn markers_and_mixed_roots_are_never_stripped() {
        for marker in [
            "data",
            "entities",
            "wax",
            "movie",
            "mod.toml",
            "pl",
            "corehap.dat",
            "x.cpk",
        ] {
            let entries = owned(&[marker, &format!("{marker}/child")]);
            assert_eq!(strip_common_wrapper(entries.clone()), entries, "{marker}");
        }
        let mixed = owned(&["a/x", "b/y"]);
        assert_eq!(strip_common_wrapper(mixed.clone()), mixed);
        assert_eq!(
            strip_common_wrapper(owned(&["/x", "/y"])),
            owned(&["/x", "/y"])
        );
        assert!(strip_common_wrapper(Vec::new()).is_empty());
    }

    #[test]
    fn a_single_unknown_entry_strips_to_nothing() {
        assert!(strip_common_wrapper(owned(&["readme.txt"])).is_empty());
        assert!(strip_common_wrapper(owned(&["folder", "folder/"])).is_empty());
    }

    #[test]
    fn entry_signals() {
        assert_eq!(entry_signal(""), EntrySignal::Neutral);
        assert_eq!(entry_signal("/x.dds"), EntrySignal::Neutral);
        assert_eq!(entry_signal("docs/readme.txt"), EntrySignal::Neutral);
        assert_eq!(entry_signal("data"), EntrySignal::Neutral);
        assert_eq!(
            entry_signal("data/flutter_assets/a.bin"),
            EntrySignal::Neutral
        );
        assert_eq!(entry_signal("data/movie"), EntrySignal::Cutscene);
        assert_eq!(entry_signal("data/movie/ev0010.usm"), EntrySignal::Cutscene);
        assert_eq!(entry_signal("movie"), EntrySignal::Cutscene);
        assert_eq!(entry_signal("movie/ev0010.usm"), EntrySignal::Cutscene);
        assert_eq!(entry_signal("data/sound"), EntrySignal::Mod);
        assert_eq!(entry_signal("a/b/data/pl"), EntrySignal::Mod);
        assert_eq!(entry_signal("a/data/movie/x.usm"), EntrySignal::Neutral);
        assert_eq!(entry_signal("variant/entities/x"), EntrySignal::Mod);
        assert_eq!(entry_signal("variant/wax/mods"), EntrySignal::Mod);
        assert_eq!(entry_signal("entities"), EntrySignal::Mod);
        assert_eq!(entry_signal("wax"), EntrySignal::Mod);
        assert_eq!(entry_signal("mod.toml"), EntrySignal::Mod);
        assert_eq!(entry_signal("deep/mod.toml"), EntrySignal::Mod);
        assert_eq!(entry_signal("textures/a.dds"), EntrySignal::Mod);
        assert_eq!(entry_signal("anything/pl0000.dtt"), EntrySignal::Mod);
        assert_eq!(entry_signal("x/data100.cpk"), EntrySignal::Mod);
        assert_eq!(entry_signal("plweird.dat/readme.txt"), EntrySignal::Mod);
        assert_eq!(entry_signal("wp"), EntrySignal::Mod);
        assert_eq!(entry_signal("effect/model/x.bin"), EntrySignal::Mod);
        assert_eq!(entry_signal("sound/x.bin"), EntrySignal::Neutral);
        assert_eq!(entry_signal("font"), EntrySignal::Neutral);
    }
}
