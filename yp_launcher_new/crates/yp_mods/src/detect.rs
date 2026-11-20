use std::path::{Path, PathBuf};

use indexmap::IndexSet;
use yp_core::{CancelToken, ProgressSink};

use crate::fsx;
use crate::loose_paths::data_dir_for_loose_file;
use crate::manifest;
use crate::model::{DetectedDrop, DropRejection, ManifestInfo, ModKind, ModVariant, TexturePack};
use crate::probe::{
    classify_kind, dir_has_dds, dir_has_direct_dds, entities_has_content, has_compat_config,
    has_nams_payload, has_unsupported_cpk, is_cpk_file, is_hidden_name, is_mod_payload_dir,
    is_texture_pack_root, is_texture_wrapper, primary_data_category_at, unwrap_single_child,
};
use crate::scan::{scan_data_if_present, scan_native};
use crate::staging::{DropStaging, StagedDrop, source_base_name};
use crate::tables::{is_data_dir_key, player_model_label};
use crate::text::{dart_trim, file_extension, is_regex_whitespace, utf16_len};

const MAX_VARIANT_DEPTH: usize = 4;

pub fn is_loose_data_file(file_name: &str) -> bool {
    data_dir_for_loose_file(file_name).is_some() || is_cpk_file(file_name)
}

pub fn is_texture_only_listing<I, S>(entries: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut has_dds = false;
    for entry in entries {
        let normalized = entry.as_ref().replace('\\', "/");
        let segments: Vec<String> = normalized
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(str::to_lowercase)
            .collect();
        let Some(name) = segments.last() else {
            continue;
        };
        if segments
            .iter()
            .any(|segment| segment == "entities" || segment == "wax")
        {
            return false;
        }
        if name == "mod.toml" {
            return false;
        }
        match file_extension(name) {
            ".dat" | ".dtt" | ".cpk" => return false,
            ".dds" => has_dds = true,
            _ => {}
        }
    }
    has_dds
}

pub fn detect_texture_packs(root: &Path) -> Vec<TexturePack> {
    detect_packs(root, false)
}

pub(crate) fn detect_packs(root: &Path, ignore_payload: bool) -> Vec<TexturePack> {
    let mut collector = PackCollector {
        root_name: fsx::file_name(root),
        packs: Vec::new(),
        seen: IndexSet::new(),
    };
    if !ignore_payload && has_nams_payload(root) {
        return Vec::new();
    }
    if !ignore_payload && dir_has_direct_dds(root) {
        collector.add(root, &collector.root_name.clone());
    } else {
        for child in fsx::child_dirs(root) {
            if is_hidden_name(&child.name) {
                continue;
            }
            if ignore_payload && is_mod_payload_dir(&child.name.to_lowercase()) {
                continue;
            }
            collector.scan(&child.path, None);
        }
    }
    let mut packs = collector.packs;
    packs.sort_by(|a, b| crate::text::utf16_cmp(&a.label.to_lowercase(), &b.label.to_lowercase()));
    packs
}

struct PackCollector {
    root_name: String,
    packs: Vec<TexturePack>,
    seen: IndexSet<PathBuf>,
}

impl PackCollector {
    fn add(&mut self, dir: &Path, label: &str) {
        if self.seen.insert(dir.to_path_buf()) {
            self.packs.push(TexturePack {
                path: dir.to_path_buf(),
                label: label.to_owned(),
            });
        }
    }

    fn scan(&mut self, dir: &Path, named_parent: Option<&str>) {
        if !dir.is_dir() {
            return;
        }
        let name = fsx::file_name(dir);
        let label: Option<String> = if is_texture_wrapper(&name) {
            named_parent.map(str::to_owned)
        } else {
            Some(name)
        };

        if dir_has_direct_dds(dir) {
            let label = label.unwrap_or_else(|| self.root_name.clone());
            self.add(dir, &label);
            return;
        }

        let dds_subdirs: Vec<fsx::Child> = fsx::child_dirs(dir)
            .into_iter()
            .filter(|sub| dir_has_dds(&sub.path))
            .collect();
        if dds_subdirs.is_empty() {
            return;
        }

        let has_named_child = dds_subdirs.iter().any(|sub| !is_texture_wrapper(&sub.name));
        if !has_named_child && let Some(label) = &label {
            let deeper_named = dds_subdirs
                .iter()
                .filter(|sub| is_texture_wrapper(&sub.name))
                .any(|wrapper| {
                    fsx::child_dirs(&wrapper.path)
                        .iter()
                        .any(|inner| dir_has_dds(&inner.path) && !is_texture_wrapper(&inner.name))
                });
            if !deeper_named {
                self.add(dir, label);
                return;
            }
        }

        for sub in dds_subdirs {
            self.scan(&sub.path, label.as_deref());
        }
    }
}

fn detect_variants(root: &Path) -> Vec<ModVariant> {
    let mut out = Vec::new();
    collect_variants(root, root, "", 0, &mut out);
    out.sort_by(|a, b| crate::text::utf16_cmp(&a.label.to_lowercase(), &b.label.to_lowercase()));
    out
}

fn collect_variants(
    base_root: &Path,
    current: &Path,
    label_prefix: &str,
    depth: usize,
    out: &mut Vec<ModVariant>,
) {
    if depth > MAX_VARIANT_DEPTH || !current.is_dir() {
        return;
    }
    for sub in fsx::child_dirs(current) {
        if is_hidden_name(&sub.name) {
            continue;
        }
        let unwrapped = unwrap_single_child(&sub.path);
        let label = if label_prefix.is_empty() {
            sub.name.clone()
        } else {
            format!("{label_prefix} / {}", sub.name)
        };
        let kind = classify_kind(&unwrapped);
        if kind != ModKind::Unknown {
            let texture_only = !has_nams_payload(&unwrapped) && is_texture_pack_root(&unwrapped);
            out.push(ModVariant {
                sub_path: fsx::relative(&unwrapped, base_root).unwrap_or_else(|| unwrapped.clone()),
                label,
                kind,
                texture_only,
                category: primary_data_category_at(&unwrapped),
            });
        } else {
            collect_variants(base_root, &unwrapped, &label, depth + 1, out);
        }
    }
}

pub(crate) fn detect_dir(work_dir: &Path, source_base_name: &str) -> DetectedDrop {
    let unwrapped = unwrap_single_child(work_dir);

    if has_unsupported_cpk(work_dir) {
        let suggested = suggest_id(None, source_base_name, &unwrapped);
        return DetectedDrop::rejected(unwrapped, suggested, DropRejection::UnsupportedNasa);
    }

    let manifest = manifest::load(&unwrapped);
    let kind = classify_kind(&unwrapped);

    let mut rejection = None;
    let mut variants = Vec::new();
    if kind == ModKind::Unknown {
        variants = detect_variants(&unwrapped);
        if variants.is_empty() {
            rejection = Some(
                if !has_nams_payload(&unwrapped) && is_texture_pack_root(&unwrapped) {
                    DropRejection::TextureOnly
                } else if entities_has_content(&unwrapped.join("entities"))
                    && has_compat_config(&unwrapped)
                {
                    DropRejection::InvalidMixed
                } else {
                    DropRejection::UnknownDrop
                },
            );
        }
    }

    let native = (kind == ModKind::Native).then(|| scan_native(&unwrapped.join("entities")));
    let data = scan_data_if_present(&unwrapped);

    let packs = detect_packs(&unwrapped, true);
    let texture_variants = if packs.len() > 1 { packs } else { Vec::new() };

    let suggested_id = suggest_id(manifest.as_ref(), source_base_name, &unwrapped);
    DetectedDrop {
        unwrapped_root: unwrapped,
        kind,
        manifest,
        native,
        data,
        suggested_id,
        rejection,
        variants,
        texture_variants,
    }
}

pub fn detect_drop(
    staging: &DropStaging,
    source: &Path,
    progress: &dyn ProgressSink,
    cancel: &CancelToken,
) -> yp_core::Result<DetectedDrop> {
    let base_name = source_base_name(source);
    let staged = if DropStaging::is_archive(source) {
        if let Some(listing) = staging.list_entries(source)
            && is_texture_only_listing(&listing)
        {
            return Ok(DetectedDrop::rejected(
                source.to_path_buf(),
                base_name,
                DropRejection::TextureOnly,
            ));
        }
        match staging.extract(source, None, progress, cancel) {
            Ok(dir) => StagedDrop::Temp(dir),
            Err(yp_core::Error::Archive { failure, .. }) => {
                return Ok(DetectedDrop::rejected(
                    source.to_path_buf(),
                    String::new(),
                    DropRejection::ArchiveExtractFailed(failure),
                ));
            }
            Err(err) => return Err(err),
        }
    } else if source.is_file() {
        match staging.stage_loose_file(source, None) {
            Some(dir) => StagedDrop::Temp(dir),
            None => {
                return Ok(DetectedDrop::rejected(
                    source.to_path_buf(),
                    String::new(),
                    DropRejection::UnknownDrop,
                ));
            }
        }
    } else {
        StagedDrop::InPlace(source.to_path_buf())
    };
    cancel.check()?;
    Ok(detect_dir(staged.root(), &base_name))
}

fn suggest_id(
    manifest: Option<&ManifestInfo>,
    source_base_name: &str,
    unwrapped_root: &Path,
) -> String {
    if let Some(id) = manifest.and_then(|manifest| manifest.id.as_deref())
        && !dart_trim(id).is_empty()
    {
        return sanitize_id(id);
    }
    let candidates = [source_base_name.to_owned(), fsx::file_name(unwrapped_root)];
    sanitize_id(&first_usable_name(&candidates, unwrapped_root))
}

fn first_usable_name(candidates: &[String], unwrapped_root: &Path) -> String {
    if let Some(usable) = candidates
        .iter()
        .find(|candidate| is_usable_name(candidate))
    {
        return usable.clone();
    }
    if let Some(from_players) = name_from_player_files(unwrapped_root) {
        return from_players;
    }
    candidates
        .iter()
        .find(|candidate| !dart_trim(candidate).is_empty())
        .cloned()
        .unwrap_or_else(|| "mod".to_owned())
}

fn is_hex_run(text: &str, min_len: usize) -> bool {
    text.len() >= min_len && text.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_archive_hash_name(lower: &str) -> bool {
    let Some(rest) = lower.strip_prefix("archive") else {
        return false;
    };
    let digits = rest.strip_prefix(['_', '-']).unwrap_or(rest);
    is_hex_run(digits, 4)
}

pub fn is_usable_name(name: &str) -> bool {
    let trimmed = dart_trim(name);
    if utf16_len(trimmed) < 3 {
        return false;
    }
    let lower = trimmed.to_lowercase();
    if is_data_dir_key(&lower) || matches!(lower.as_str(), "data" | "mod" | "mods") {
        return false;
    }
    let ascii_lower = trimmed.to_ascii_lowercase();
    !is_archive_hash_name(&ascii_lower) && !is_hex_run(&ascii_lower, 6)
}

fn name_from_player_files(root: &Path) -> Option<String> {
    if !root.is_dir() {
        return None;
    }
    let scan_roots = [
        root.to_path_buf(),
        root.join("data").join("pl"),
        root.join("pl"),
    ];
    let mut stems: IndexSet<String> = IndexSet::new();
    for scan_root in &scan_roots {
        if !scan_root.is_dir() {
            continue;
        }
        for file in fsx::child_files(scan_root) {
            let base = file.name.to_lowercase();
            if !base.starts_with("pl") {
                continue;
            }
            let Some(stem) = base
                .strip_suffix(".dat")
                .or_else(|| base.strip_suffix(".dtt"))
            else {
                continue;
            };
            if utf16_len(stem) >= 6 {
                stems.insert(stem.chars().take(6).collect());
            }
        }
    }
    let labels: IndexSet<&str> = stems
        .iter()
        .filter_map(|stem| player_model_label(stem))
        .collect();
    if labels.is_empty() {
        return None;
    }
    Some(labels.into_iter().collect::<Vec<_>>().join(" & "))
}

pub fn sanitize_id(raw: &str) -> String {
    let lower = raw.to_lowercase();
    let mut cleaned = String::with_capacity(lower.len());
    for c in dart_trim(&lower).chars() {
        let mapped = if c.is_ascii_lowercase() || c.is_ascii_digit() {
            c
        } else if is_regex_whitespace(c) || matches!(c, '-' | '_' | '(' | ')' | '.' | ',') {
            '_'
        } else {
            continue;
        };
        if mapped == '_' && cleaned.ends_with('_') {
            continue;
        }
        cleaned.push(mapped);
    }
    let trimmed = cleaned.strip_prefix('_').unwrap_or(&cleaned);
    let trimmed = trimmed.strip_suffix('_').unwrap_or(trimmed);
    if trimmed.is_empty() {
        "mod".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_ids() {
        assert_eq!(sanitize_id("  Kimono Wax (v1.2)  "), "kimono_wax_v1_2");
        assert_eq!(sanitize_id("batch - 2B / Outfit"), "batch_2b_outfit");
        assert_eq!(sanitize_id("___"), "mod");
        assert_eq!(sanitize_id("Ünï"), "n");
        assert_eq!(sanitize_id(""), "mod");
        assert_eq!(sanitize_id("a\u{3000}b"), "a_b");
    }

    #[test]
    fn usable_names_reject_hashes_and_data_dirs() {
        assert!(is_usable_name("Kimono"));
        assert!(!is_usable_name("pl"));
        assert!(!is_usable_name("Data"));
        assert!(!is_usable_name("ab"));
        assert!(!is_usable_name("archive_deadbeef"));
        assert!(!is_usable_name("Archive-1234"));
        assert!(!is_usable_name("archive12ab"));
        assert!(is_usable_name("archive_xyz1"));
        assert!(!is_usable_name("0a1b2c"));
        assert!(is_usable_name("0a1b2"));
        assert!(!is_usable_name("core"));
    }
}
