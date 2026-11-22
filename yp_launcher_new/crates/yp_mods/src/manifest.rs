use std::path::Path;

use crate::model::ManifestInfo;
use crate::text::dart_trim;
use crate::toml_doc::read_values;

pub const MANIFEST_FILE_NAME: &str = "mod.toml";

pub fn load(content_root: &Path) -> Option<ManifestInfo> {
    let file = content_root.join(MANIFEST_FILE_NAME);
    if !file.is_file() {
        return None;
    }
    let raw = yp_core::fs::read_text_opt(&file).ok().flatten()?;
    if dart_trim(&raw).is_empty() {
        return None;
    }
    let parsed = read_values(&raw);
    let section = match parsed.get("mod") {
        Some(toml::Value::Table(nested)) => nested,
        _ => &parsed,
    };
    Some(ManifestInfo {
        id: trimmed_string(section.get("id")),
        display_name: trimmed_string(section.get("display_name")),
        author: trimmed_string(section.get("author")),
        version: trimmed_string(section.get("version")),
        nams_min_version: trimmed_string(section.get("nams_min_version")),
        requires: trimmed_list(section.get("requires")),
        requires_plugins: trimmed_list(section.get("requires_plugins")),
        replaces: trimmed_list(section.get("replaces")),
    })
}

fn trimmed_string(value: Option<&toml::Value>) -> Option<String> {
    let trimmed = dart_trim(value?.as_str()?);
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn trimmed_list(value: Option<&toml::Value>) -> Vec<String> {
    let Some(array) = value.and_then(toml::Value::as_array) else {
        return Vec::new();
    };
    array
        .iter()
        .filter_map(toml::Value::as_str)
        .map(dart_trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_and_flat_manifests() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(MANIFEST_FILE_NAME),
            "[mod]\nid = \" cool \"\ndisplay_name = \"\"\nrequires = [\"a\", \" \", 3]\n",
        )
        .unwrap();
        let manifest = load(dir.path()).unwrap();
        assert_eq!(manifest.id.as_deref(), Some("cool"));
        assert_eq!(manifest.display_name, None);
        assert_eq!(manifest.requires, vec!["a".to_owned()]);

        std::fs::write(dir.path().join(MANIFEST_FILE_NAME), "id = \"flat\"\n").unwrap();
        assert_eq!(load(dir.path()).unwrap().id.as_deref(), Some("flat"));
    }

    #[test]
    fn blank_manifest_is_absent_and_broken_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), None);
        std::fs::write(dir.path().join(MANIFEST_FILE_NAME), "  \n").unwrap();
        assert_eq!(load(dir.path()), None);
        std::fs::write(dir.path().join(MANIFEST_FILE_NAME), "id = = 1").unwrap();
        assert_eq!(load(dir.path()), Some(ManifestInfo::default()));
    }
}
