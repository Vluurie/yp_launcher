mod support;

use std::collections::BTreeSet;

use support::{declared_placeholders, message_keys, read_arb};
use yp_i18n::{Locale, PluralCategory, Selector, icu};

fn locale_categories(locale: Locale) -> BTreeSet<PluralCategory> {
    (0..200).map(|n| yp_i18n::category(locale, n)).collect()
}

#[test]
fn every_locale_has_exactly_the_template_keys() {
    let template = read_arb(Locale::En);
    let expected: BTreeSet<String> = message_keys(&template).into_iter().collect();
    for locale in Locale::ALL {
        let arb = read_arb(locale);
        let keys: BTreeSet<String> = message_keys(&arb).into_iter().collect();
        assert_eq!(keys, expected, "{} keys differ from en", locale.code());
    }
}

#[test]
fn every_message_parses_in_every_locale() {
    let mut failures = Vec::new();
    for locale in Locale::ALL {
        let arb = read_arb(locale);
        for key in message_keys(&arb) {
            let source = arb[&key].as_str().unwrap();
            if let Err(err) = icu::parse(source) {
                failures.push(format!("{} {key}: {err}", locale.code()));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn placeholders_match_the_template_declarations() {
    let template = read_arb(Locale::En);
    let mut failures = Vec::new();
    for locale in Locale::ALL {
        let arb = read_arb(locale);
        for key in message_keys(&template) {
            let declared: BTreeSet<String> = declared_placeholders(&template, &key)
                .into_iter()
                .map(|(name, _)| name)
                .collect();
            let message = icu::parse(arb[&key].as_str().unwrap()).unwrap();
            let used = message.arguments();
            if !used.is_subset(&declared) {
                failures.push(format!(
                    "{} {key}: uses {used:?}, declared {declared:?}",
                    locale.code()
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn placeholder_types_are_string_or_int_and_plurals_use_ints() {
    let template = read_arb(Locale::En);
    let arbs: Vec<_> = Locale::ALL
        .into_iter()
        .map(|locale| (locale, read_arb(locale)))
        .collect();
    for key in message_keys(&template) {
        let declared = declared_placeholders(&template, &key);
        for (name, kind) in &declared {
            assert!(
                kind == "String" || kind == "int",
                "{key}.{name} has type {kind}"
            );
        }
        for (locale, arb) in &arbs {
            let message = icu::parse(arb[&key].as_str().unwrap()).unwrap();
            for plural in message.plurals() {
                let kind = declared
                    .iter()
                    .find(|(name, _)| *name == plural.argument)
                    .map(|(_, kind)| kind.as_str());
                assert_eq!(
                    kind,
                    Some("int"),
                    "{} {key}: plural over {}",
                    locale.code(),
                    plural.argument
                );
            }
        }
    }
}

#[test]
fn plural_categories_match_en_and_the_locale_rules() {
    let template = read_arb(Locale::En);
    let arbs: Vec<_> = Locale::ALL
        .into_iter()
        .map(|locale| (locale, read_arb(locale)))
        .collect();
    let mut failures = Vec::new();
    for key in message_keys(&template) {
        let en_message = icu::parse(template[&key].as_str().unwrap()).unwrap();
        let en_plurals = en_message.plurals();
        if en_plurals.is_empty() {
            continue;
        }
        let en_arguments: Vec<&str> = en_plurals
            .iter()
            .map(|plural| plural.argument.as_str())
            .collect();
        for (locale, arb) in &arbs {
            let locale = *locale;
            let message = icu::parse(arb[&key].as_str().unwrap()).unwrap();
            let plurals = message.plurals();
            let arguments: BTreeSet<&str> = plurals
                .iter()
                .map(|plural| plural.argument.as_str())
                .collect();
            let expected: BTreeSet<&str> = en_arguments.iter().copied().collect();
            if arguments != expected {
                failures.push(format!(
                    "{} {key}: plural arguments {arguments:?} vs en {expected:?}",
                    locale.code()
                ));
            }
            let allowed_categories = locale_categories(locale);
            for plural in &plurals {
                if !plural.cases.iter().any(|case| case.selector.is_other()) {
                    failures.push(format!("{} {key}: plural without other", locale.code()));
                }
                for case in &plural.cases {
                    if let Selector::Category(category) = case.selector
                        && case.selector.explicit_number().is_none()
                        && !allowed_categories.contains(&category)
                    {
                        failures.push(format!(
                            "{} {key}: category {category:?} never selected",
                            locale.code()
                        ));
                    }
                }
                let en_plural = en_plurals.iter().find(|en| en.argument == plural.argument);
                let shares_en_rules = allowed_categories.contains(&PluralCategory::One);
                if let Some(en_plural) = en_plural
                    && shares_en_rules
                    && en_plural.explicit_numbers() != plural.explicit_numbers()
                {
                    failures.push(format!(
                        "{} {key}: explicit cases {:?} vs en {:?}",
                        locale.code(),
                        plural.explicit_numbers(),
                        en_plural.explicit_numbers()
                    ));
                }
                if let Some(en_plural) = en_plural
                    && !plural
                        .explicit_numbers()
                        .is_subset(&en_plural.explicit_numbers())
                {
                    failures.push(format!("{} {key}: explicit cases not in en", locale.code()));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
