import 'dart:ui';

import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';

part 'locale_state.g.dart';

const List<Locale> kSupportedLocales = [
  Locale('en'),
  Locale('de'),
  Locale('zh'),
  Locale('th'),
];

String localeDisplayName(Locale locale) {
  switch (locale.languageCode) {
    case 'de':
      return 'Deutsch';
    case 'zh':
      return '简体中文';
    case 'th':
      return 'ไทย';
    default:
      return 'English';
  }
}

@Riverpod(keepAlive: true)
class LocaleController extends _$LocaleController {
  @override
  Locale? build() {
    _loadSavedLocale();
    return null;
  }

  Future<void> _loadSavedLocale() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final saved = prefs.getString(AppStrings.prefKeyLocale);
      if (saved != null && saved.isNotEmpty) {
        state = Locale(saved);
      }
    } catch (_) {}
  }

  Future<void> setLocale(Locale locale) async {
    state = locale;

    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(AppStrings.prefKeyLocale, locale.languageCode);
    } catch (_) {}
  }
}
