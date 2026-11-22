use yp_core::{HostEnv, SettingKey, SettingsStore};

use crate::strings::Strings;

pub const APP_LOCALE: SettingKey<String> = SettingKey::new("app_locale", String::new);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Locale {
    #[default]
    En,
    De,
    Zh,
    Th,
}

impl Locale {
    pub const ALL: [Locale; 4] = [Locale::En, Locale::De, Locale::Zh, Locale::Th];
    pub const FALLBACK: Locale = Locale::En;

    pub fn code(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::De => "de",
            Locale::Zh => "zh",
            Locale::Th => "th",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::De => "Deutsch",
            Locale::Zh => "简体中文",
            Locale::Th => "ไทย",
        }
    }

    pub fn from_code(code: &str) -> Option<Locale> {
        Locale::ALL.into_iter().find(|locale| locale.code() == code)
    }

    pub fn from_language_tag(tag: &str) -> Option<Locale> {
        let language = tag
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        Locale::from_code(&language)
    }

    pub fn strings(self) -> &'static Strings {
        Strings::for_locale(self)
    }
}

pub fn resolve_locale(saved: Option<&str>, system_language: Option<&str>) -> Locale {
    match saved.filter(|saved| !saved.is_empty()) {
        Some(saved) => Locale::from_code(saved).unwrap_or(Locale::FALLBACK),
        None => system_language
            .and_then(Locale::from_language_tag)
            .unwrap_or(Locale::FALLBACK),
    }
}

pub fn saved_locale_code(settings: &SettingsStore) -> Option<String> {
    Some(settings.get(&APP_LOCALE)).filter(|code| !code.is_empty())
}

pub fn startup_locale(settings: &SettingsStore, host: &HostEnv) -> Locale {
    resolve_locale(
        saved_locale_code(settings).as_deref(),
        host.system_language().as_deref(),
    )
}

pub fn save_locale_quiet(settings: &SettingsStore, locale: Locale) -> bool {
    match settings.set(&APP_LOCALE, &locale.code().to_owned()) {
        Ok(()) => true,
        Err(err) => {
            log::warn!("could not save locale {}: {err}", locale.code());
            false
        }
    }
}
