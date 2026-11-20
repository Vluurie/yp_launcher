use std::path::Path;

use indexmap::IndexMap;
use toml_edit::{DocumentMut, Item, Value};

use crate::toml_doc::{root_array_mut, set_root_value, string_array};

pub(crate) const LOAD_ORDER_KEY: &str = "load_order";
pub(crate) const DISABLED_PACKS_KEY: &str = "disabled_packs";
const PRUNED_KEYS: [&str; 2] = [LOAD_ORDER_KEY, DISABLED_PACKS_KEY];
const DISABLED_PACKS_COMMENT: &str = "\n# Pack folder names skipped during scanning. Launcher-managed\n# (overwritten on every profile switch).\n";

fn parse(text: &str) -> Option<DocumentMut> {
    text.parse::<DocumentMut>().ok()
}

pub(crate) fn without_packs(text: &str, removed: &[String]) -> Option<String> {
    let mut doc = parse(text)?;
    let mut changed = false;
    for key in PRUNED_KEYS {
        let Some(array) = root_array_mut(&mut doc, key) else {
            continue;
        };
        let before = array.len();
        array.retain(|item| {
            item.as_str()
                .is_some_and(|name| !removed.iter().any(|gone| gone == name))
        });
        if array.len() != before {
            array.fmt();
            changed = true;
        }
    }
    changed.then(|| doc.to_string())
}

pub(crate) fn with_disabled_packs(text: &str, packs: &[String]) -> Option<String> {
    let mut doc = parse(text)?;
    let value = string_array(packs.iter().cloned());
    let exists = matches!(doc.as_table().get(DISABLED_PACKS_KEY), Some(Item::Value(_)));
    set_root_value(&mut doc, DISABLED_PACKS_KEY, value);
    if !exists && let Some(mut key) = doc.as_table_mut().key_mut(DISABLED_PACKS_KEY) {
        key.leaf_decor_mut().set_prefix(DISABLED_PACKS_COMMENT);
    }
    let rendered = doc.to_string();
    (rendered != text).then_some(rendered)
}

pub(crate) fn with_renamed_load_order(
    text: &str,
    renames: &IndexMap<String, Option<String>>,
) -> Option<String> {
    if renames.is_empty() {
        return None;
    }
    let mut doc = parse(text)?;
    let array = root_array_mut(&mut doc, LOAD_ORDER_KEY)?;
    let mut renamed: Vec<String> = Vec::new();
    for item in array.iter() {
        let Some(name) = item.as_str() else {
            continue;
        };
        match renames.get(name) {
            Some(Some(mapped)) => renamed.push(mapped.clone()),
            Some(None) => {}
            None => renamed.push(name.to_owned()),
        }
    }
    let mut value: Value = string_array(renamed);
    *value.decor_mut() = array.decor().clone();
    if let Some(Item::Value(slot)) = doc.as_table_mut().get_mut(LOAD_ORDER_KEY) {
        *slot = value;
    }
    let rendered = doc.to_string();
    (rendered != text).then_some(rendered)
}

pub(crate) fn prune_file_quiet(file: &Path, removed: &[String]) {
    if removed.is_empty() || !file.is_file() {
        return;
    }
    let Ok(Some(text)) = yp_core::fs::read_text_opt(file) else {
        return;
    };
    if let Some(updated) = without_packs(&text, removed) {
        let _ = std::fs::write(file, updated);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn prunes_both_lists() {
        let text = "# c\nload_order = [\"a\", \"b\", \"c\"]\ndisabled_packs = [\"b\"]\nhot_reload = true\n";
        let out = without_packs(text, &names(&["b"])).unwrap();
        assert_eq!(
            out,
            "# c\nload_order = [\"a\", \"c\"]\ndisabled_packs = []\nhot_reload = true\n"
        );
        assert_eq!(without_packs(text, &names(&["zzz"])), None);
    }

    #[test]
    fn writes_disabled_packs_in_place_or_appends_with_comment() {
        let text = "load_order = []\ndisabled_packs = [] # managed\n";
        let out = with_disabled_packs(text, &names(&["p (m@b)"])).unwrap();
        assert_eq!(
            out,
            "load_order = []\ndisabled_packs = [\"p (m@b)\"] # managed\n"
        );

        let out = with_disabled_packs("load_order = []\n", &names(&["x"])).unwrap();
        assert!(out.contains("# (overwritten on every profile switch).\ndisabled_packs = [\"x\"]"));
        assert_eq!(with_disabled_packs(text, &[]), None);
    }

    #[test]
    fn renames_and_drops_load_order_entries() {
        let mut renames = IndexMap::new();
        renames.insert("a".to_owned(), Some("a (m@p)".to_owned()));
        renames.insert("b".to_owned(), None);
        let out = with_renamed_load_order("load_order = [\"a\", \"b\", \"c\"] # keep\n", &renames)
            .unwrap();
        assert_eq!(out, "load_order = [\"a (m@p)\", \"c\"] # keep\n");
    }
}
