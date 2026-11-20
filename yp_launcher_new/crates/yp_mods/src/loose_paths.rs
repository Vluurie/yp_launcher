use std::sync::LazyLock;

use indexmap::{IndexMap, IndexSet};
use serde::Deserialize;

use crate::tables::DATA_DIR_CATEGORIES;
use crate::text::strip_ascii_suffix;

#[derive(Debug, Deserialize)]
pub struct LooseFilePaths {
    pub subdir_for_stem: IndexMap<String, String>,
    pub stems_with_vanilla_pair: IndexSet<String>,
}

static TABLE: LazyLock<LooseFilePaths> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/loose_file_paths.json"))
        .expect("embedded loose_file_paths.json is valid")
});

pub fn table() -> &'static LooseFilePaths {
    &TABLE
}

pub fn subdir_for_stem(stem: &str) -> Option<&'static str> {
    TABLE.subdir_for_stem.get(stem).map(String::as_str)
}

pub fn has_vanilla_pair(stem: &str) -> bool {
    TABLE.stems_with_vanilla_pair.contains(stem)
}

pub fn data_dir_for_loose_file(file_name: &str) -> Option<String> {
    let lower = file_name.to_lowercase();
    let stem = strip_ascii_suffix(&lower, ".dat").or_else(|| strip_ascii_suffix(&lower, ".dtt"))?;
    if let Some(exact) = subdir_for_stem(stem) {
        return Some(exact.to_owned());
    }
    let mut best: Option<&str> = None;
    for (key, _) in DATA_DIR_CATEGORIES {
        if stem.starts_with(key) && best.is_none_or(|current| key.len() > current.len()) {
            best = Some(key);
        }
    }
    best.map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_exact_and_prefix_matches() {
        assert_eq!(
            data_dir_for_loose_file("corehap.dat").as_deref(),
            Some("core")
        );
        assert_eq!(data_dir_for_loose_file("PL0000.DTT").as_deref(), Some("pl"));
        assert_eq!(data_dir_for_loose_file("readme.txt"), None);
        assert_eq!(data_dir_for_loose_file("zzzz.dat"), None);
        assert_eq!(
            data_dir_for_loose_file("wd1xyz.dat").as_deref(),
            Some("wd1")
        );
    }
}
