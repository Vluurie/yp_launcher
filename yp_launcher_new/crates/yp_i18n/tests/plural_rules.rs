use yp_i18n::{Locale, PluralCategory, Strings, category, choose, icu};

#[test]
fn english_and_german_split_one_from_other() {
    for locale in [Locale::En, Locale::De] {
        assert_eq!(category(locale, 1), PluralCategory::One);
        for n in [0, 2, 3, 5, 11, 21, 101] {
            assert_eq!(
                category(locale, n),
                PluralCategory::Other,
                "{} {n}",
                locale.code()
            );
        }
    }
}

#[test]
fn chinese_and_thai_only_have_other() {
    for locale in [Locale::Zh, Locale::Th] {
        for n in [0, 1, 2, 5, 21] {
            assert_eq!(category(locale, n), PluralCategory::Other);
        }
    }
}

fn pick(source: &str, locale: Locale, count: i64) -> String {
    let message = icu::parse(source).unwrap();
    let icu::Part::Plural(plural) = &message.parts[0] else {
        panic!("not a plural");
    };
    choose(plural, locale, count).plain_text().unwrap()
}

#[test]
fn explicit_cases_win_over_categories() {
    let source = "{n, plural, =0{none} =1{single} other{many}}";
    for locale in Locale::ALL {
        assert_eq!(pick(source, locale, 0), "none");
        assert_eq!(pick(source, locale, 1), "single");
        assert_eq!(pick(source, locale, 2), "many");
    }
}

#[test]
fn one_category_acts_as_exactly_one_like_flutter_intl() {
    let source = "{n, plural, one{one} other{other}}";
    assert_eq!(pick(source, Locale::En, 1), "one");
    assert_eq!(pick(source, Locale::Zh, 1), "one");
    assert_eq!(pick(source, Locale::Th, 1), "one");
    assert_eq!(pick(source, Locale::De, 21), "other");
    assert_eq!(pick(source, Locale::Zh, 2), "other");
}

#[test]
fn few_and_many_fall_back_to_other_in_supported_locales() {
    let source = "{n, plural, few{few} many{many} other{other}}";
    for locale in Locale::ALL {
        for n in [0, 1, 3, 11, 25] {
            assert_eq!(pick(source, locale, n), "other");
        }
    }
}

#[test]
fn chinese_strings_use_other_where_english_has_one() {
    let zh = Strings::for_locale(Locale::Zh);
    let en = Strings::for_locale(Locale::En);
    assert_ne!(
        en.tools_icons_count(1),
        en.tools_icons_count(2).replace('2', "1")
    );
    assert_eq!(
        zh.tools_icons_count(1),
        zh.tools_icons_count(2).replace('2', "1")
    );
}
