mod support;

use std::collections::BTreeSet;

use support::{arb_dir, extra_set_names, message_keys, read_arb, read_extra_set};
use yp_i18n::{Locale, Msg, Strings};

fn main_keys() -> BTreeSet<String> {
    let path = arb_dir().join("app_en.arb");
    let text = std::fs::read_to_string(path).unwrap();
    let document: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
    message_keys(&document).into_iter().collect()
}

#[test]
fn extra_sets_never_redefine_a_main_key_or_each_other() {
    let mut seen = main_keys();
    for set in extra_set_names() {
        for key in message_keys(&read_extra_set(&set, Locale::En)) {
            assert!(seen.insert(key.clone()), "{set}: duplicate key {key}");
        }
    }
}

#[test]
fn every_extra_key_is_generated_for_every_locale() {
    for set in extra_set_names() {
        let template = read_extra_set(&set, Locale::En);
        for key in message_keys(&template) {
            let declared = support::declared_placeholders(&template, &key);
            if !declared.is_empty() {
                continue;
            }
            let message = Msg::from_args(&key, &[]).unwrap_or_else(|| panic!("{set}: {key}"));
            for locale in Locale::ALL {
                let expected = read_extra_set(&set, locale)[&key]
                    .as_str()
                    .unwrap()
                    .to_owned();
                let resolved = message.resolve(Strings::for_locale(locale));
                assert_eq!(resolved, expected, "{set} {key} {}", locale.code());
            }
        }
    }
}

#[test]
fn extra_keys_keep_the_template_order_after_the_main_keys() {
    let merged = message_keys(&read_arb(Locale::En));
    let main = main_keys();
    let first_extra = merged.iter().position(|key| !main.contains(key));
    if let Some(first_extra) = first_extra {
        assert!(merged[first_extra..].iter().all(|key| !main.contains(key)));
        assert_eq!(first_extra, main.len());
    }
}
