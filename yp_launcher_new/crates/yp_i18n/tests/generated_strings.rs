mod support;

use std::borrow::Cow;

use support::{declared_placeholders, message_keys, read_arb};
use yp_i18n::{ArgValue, Locale, Msg, MsgKey, Strings, icu, render};

const SAMPLE_COUNTS: [i64; 7] = [0, 1, 2, 3, 5, 11, 21];

#[test]
fn static_keys_return_the_arb_text_for_every_locale() {
    for locale in Locale::ALL {
        let arb = read_arb(locale);
        let strings = Strings::for_locale(locale);
        for key in MsgKey::ALL {
            assert_eq!(
                strings.get(*key),
                arb[key.name()].as_str().unwrap(),
                "{} {}",
                locale.code(),
                key.name()
            );
        }
    }
}

#[test]
fn every_static_arb_key_has_a_msg_key() {
    let template = read_arb(Locale::En);
    let static_keys: Vec<String> = message_keys(&template)
        .into_iter()
        .filter(|key| declared_placeholders(&template, key).is_empty())
        .collect();
    assert_eq!(static_keys.len(), MsgKey::COUNT);
    assert_eq!(MsgKey::ALL.len(), MsgKey::COUNT);
    for (key, msg_key) in static_keys.iter().zip(MsgKey::ALL) {
        assert_eq!(msg_key.name(), key);
        assert_eq!(MsgKey::from_name(key), Some(*msg_key));
    }
    assert_eq!(MsgKey::from_name("doesNotExist"), None);
}

#[test]
fn generated_code_matches_the_icu_interpreter() {
    let template = read_arb(Locale::En);
    let mut checked = 0;
    for locale in Locale::ALL {
        let arb = read_arb(locale);
        let strings = Strings::for_locale(locale);
        for key in message_keys(&template) {
            let declared = declared_placeholders(&template, &key);
            let message = icu::parse(arb[&key].as_str().unwrap()).unwrap();
            let counts: &[i64] = if declared.iter().any(|(_, kind)| kind == "int") {
                &SAMPLE_COUNTS
            } else {
                &[0]
            };
            for &count in counts {
                let texts: Vec<String> = declared
                    .iter()
                    .map(|(name, _)| format!("<{name}>"))
                    .collect();
                let args: Vec<(&str, ArgValue<'_>)> = declared
                    .iter()
                    .zip(&texts)
                    .map(|((name, kind), text)| {
                        let value = if kind == "int" {
                            ArgValue::Int(count)
                        } else {
                            ArgValue::Text(text.as_str())
                        };
                        (name.as_str(), value)
                    })
                    .collect();
                let msg = Msg::from_args(&key, &args).unwrap();
                assert_eq!(msg.name(), key);
                let expected = render(&message, locale, &args).unwrap();
                assert_eq!(
                    msg.resolve(strings),
                    expected,
                    "{} {key} count={count}",
                    locale.code()
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 4 * 1800);
}

#[test]
fn msg_from_key_resolves_to_borrowed_static_text() {
    let strings = Strings::for_locale(Locale::De);
    for key in MsgKey::ALL {
        let resolved = Msg::from(*key).resolve(strings);
        assert!(matches!(resolved, Cow::Borrowed(_)));
        assert_eq!(resolved, key.text(strings));
    }
}

#[test]
fn typed_methods_format_placeholders() {
    let en = Strings::for_locale(Locale::En);
    assert_eq!(en.app_title(), "YoRHa Protocol Launcher");
    assert_eq!(
        en.trouble_missing_files_summary("NAMS.exe"),
        "Missing: NAMS.exe"
    );
    assert_eq!(
        en.trouble_clear_cache_done(3),
        "Cache cleared (3 log files removed)."
    );
    assert_eq!(en.chip_mods_count(1), "1 mod");
    assert_eq!(en.chip_mods_count(4), "4 mods");
    assert_eq!(en.third_party_game_mods_count(0), "No DLLs");
    assert_eq!(en.third_party_game_mods_count(1), "1 DLL");
    assert_eq!(en.third_party_game_mods_count(7), "7 DLLs");
    assert_eq!(en.high_grids_cell_count(1, 7), "1 ring - 7 grids (vanilla)");
    assert_eq!(en.high_grids_cell_count(3, 37), "3 rings - 37 grids");
    assert_eq!(
        en.onboarding_migration_label_existing_mods(1),
        "1 mod in nams/mods/"
    );
    assert_eq!(
        en.onboarding_migration_label_existing_mods(2),
        "2 mods in nams/mods/"
    );
}

#[test]
fn missing_or_mistyped_arguments_do_not_build_a_msg() {
    assert_eq!(Msg::from_args("troubleClearCacheDone", &[]), None);
    assert_eq!(
        Msg::from_args("troubleClearCacheDone", &[("count", ArgValue::Text("3"))]),
        None
    );
    assert_eq!(
        Msg::from_args("troubleClearCacheDone", &[("count", ArgValue::Int(3))]),
        Some(Msg::TroubleClearCacheDone { count: 3 })
    );
    assert_eq!(Msg::from_args("noSuchKey", &[]), None);
}

#[test]
fn strings_report_their_locale() {
    for locale in Locale::ALL {
        assert_eq!(Strings::for_locale(locale).locale(), locale);
        assert_eq!(locale.strings().locale(), locale);
    }
}
