use yp_core::{HostEnv, Os, SettingsStore};
use yp_i18n::locale::{resolve_locale, save_locale_quiet, saved_locale_code, startup_locale};
use yp_i18n::{APP_LOCALE, Locale};

#[test]
fn supported_locales_keep_flutter_order_and_names() {
    let codes: Vec<&str> = Locale::ALL.iter().map(|locale| locale.code()).collect();
    assert_eq!(codes, vec!["en", "de", "zh", "th"]);
    let names: Vec<&str> = Locale::ALL
        .iter()
        .map(|locale| locale.display_name())
        .collect();
    assert_eq!(names, vec!["English", "Deutsch", "简体中文", "ไทย"]);
    assert_eq!(Locale::FALLBACK, Locale::En);
    assert_eq!(Locale::default(), Locale::En);
}

#[test]
fn codes_and_language_tags_resolve() {
    for locale in Locale::ALL {
        assert_eq!(Locale::from_code(locale.code()), Some(locale));
    }
    assert_eq!(Locale::from_code("fr"), None);
    assert_eq!(Locale::from_code("DE"), None);
    assert_eq!(Locale::from_language_tag("de-DE"), Some(Locale::De));
    assert_eq!(Locale::from_language_tag("zh_Hant_TW"), Some(Locale::Zh));
    assert_eq!(Locale::from_language_tag("th_TH.UTF-8"), Some(Locale::Th));
    assert_eq!(Locale::from_language_tag("EN-us"), Some(Locale::En));
    assert_eq!(Locale::from_language_tag("fr-FR"), None);
    assert_eq!(Locale::from_language_tag(""), None);
}

#[test]
fn saved_locale_beats_system_language() {
    assert_eq!(resolve_locale(Some("de"), Some("th")), Locale::De);
    assert_eq!(resolve_locale(Some("fr"), Some("th")), Locale::En);
    assert_eq!(resolve_locale(Some(""), Some("th")), Locale::Th);
    assert_eq!(resolve_locale(None, Some("zh-CN")), Locale::Zh);
    assert_eq!(resolve_locale(None, Some("fr")), Locale::En);
    assert_eq!(resolve_locale(None, None), Locale::En);
}

#[test]
fn locale_choice_round_trips_through_settings() {
    let settings = SettingsStore::in_memory();
    assert_eq!(APP_LOCALE.name(), "app_locale");
    assert_eq!(saved_locale_code(&settings), None);
    assert!(save_locale_quiet(&settings, Locale::Th));
    assert_eq!(saved_locale_code(&settings).as_deref(), Some("th"));
    assert_eq!(settings.get(&APP_LOCALE), "th");
}

#[test]
fn startup_locale_uses_settings_then_host_language() {
    let root = tempfile::tempdir().unwrap();
    let host = HostEnv::for_test(Os::Linux, &[("LANG", "de_DE.UTF-8")], root.path());
    let settings = SettingsStore::in_memory();
    assert_eq!(startup_locale(&settings, &host), Locale::De);
    assert!(save_locale_quiet(&settings, Locale::Zh));
    assert_eq!(startup_locale(&settings, &host), Locale::Zh);
}
