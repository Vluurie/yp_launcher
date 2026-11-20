use gpui::{App, Global, SharedString};
use yp_i18n::{Locale, Msg, Strings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct I18n {
    locale: Locale,
}

impl Global for I18n {}

impl I18n {
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }

    pub fn locale(cx: &App) -> Locale {
        cx.try_global::<I18n>()
            .map(|i18n| i18n.locale)
            .unwrap_or_default()
    }

    pub fn strings(cx: &App) -> &'static Strings {
        Strings::for_locale(Self::locale(cx))
    }

    pub fn set(locale: Locale, cx: &mut App) {
        if cx.try_global::<I18n>().map(|i18n| i18n.locale) == Some(locale) {
            return;
        }
        cx.set_global(I18n { locale });
        cx.refresh_windows();
    }
}

pub trait ActiveStrings {
    fn t(&self) -> &'static Strings;
    fn active_locale(&self) -> Locale;

    fn tr(&self, msg: &Msg) -> SharedString {
        match msg.resolve(self.t()) {
            std::borrow::Cow::Borrowed(text) => SharedString::new_static(text),
            std::borrow::Cow::Owned(text) => SharedString::from(text),
        }
    }
}

impl ActiveStrings for App {
    fn t(&self) -> &'static Strings {
        I18n::strings(self)
    }

    fn active_locale(&self) -> Locale {
        I18n::locale(self)
    }
}

pub fn init(cx: &mut App) {
    if cx.try_global::<I18n>().is_none() {
        cx.set_global(I18n::default());
    }
}

#[cfg(test)]
mod tests {
    use gpui::TestAppContext;
    use yp_i18n::MsgKey;

    use super::*;
    use crate::actions::SetLocale;

    #[gpui::test]
    fn strings_follow_the_locale_global(cx: &mut TestAppContext) {
        cx.update(crate::init_for_tests);
        cx.update(|cx| {
            assert_eq!(cx.active_locale(), Locale::En);
            let english = cx.t().action_cancel();
            I18n::set(Locale::De, cx);
            assert_eq!(cx.active_locale(), Locale::De);
            assert_ne!(cx.t().action_cancel(), english);
            assert_eq!(cx.t().locale(), Locale::De);
        });
    }

    #[gpui::test]
    fn the_set_locale_action_updates_the_global(cx: &mut TestAppContext) {
        cx.update(crate::init_for_tests);
        cx.update(|cx| {
            cx.dispatch_action(&SetLocale { locale: Locale::Th });
            assert_eq!(I18n::locale(cx), Locale::Th);
            let message = Msg::from(MsgKey::ALL[0]);
            assert_eq!(
                cx.tr(&message).as_ref(),
                Strings::for_locale(Locale::Th).get(MsgKey::ALL[0])
            );
        });
    }

    #[gpui::test]
    fn component_strings_exist_in_every_locale(cx: &mut TestAppContext) {
        cx.update(crate::init_for_tests);
        cx.update(|cx| {
            for locale in Locale::ALL {
                I18n::set(locale, cx);
                let strings = cx.t();
                assert!(!strings.ui_confirm().is_empty());
                assert!(!strings.ui_no_matches().is_empty());
                assert_eq!(strings.ui_tour_step(2, 7), "2 / 7");
                assert!(strings.ui_selected_count(3).contains('3'));
                assert!(strings.ui_input_too_long(27).contains("27"));
            }
            I18n::set(Locale::De, cx);
            assert_eq!(cx.t().ui_search(), "Suchen");
            I18n::set(Locale::Zh, cx);
            assert_eq!(cx.t().ui_confirm(), "\u{786e}\u{8ba4}");
        });
    }
}
