import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/theme/app_colors.dart';
import 'package:yp_launcher/theme/app_theme.dart';

part 'app_theme_state.g.dart';

const _prefKeyAppTheme = 'app_theme';

@Riverpod(keepAlive: true)
class AppThemeController extends _$AppThemeController {
  @override
  AppThemeId build() {
    _load();
    return AppColors.active.id;
  }

  Future<void> _load() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final saved = prefs.getString(_prefKeyAppTheme);
      final id = saved == AppThemeId.nier.name
          ? AppThemeId.nier
          : AppThemeId.dark;
      AppColors.active = AppTheme.byId(id);
      state = id;
    } catch (_) {}
  }

  Future<void> setTheme(AppThemeId id) async {
    AppColors.active = AppTheme.byId(id);
    state = id;
    try {
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString(_prefKeyAppTheme, id.name);
    } catch (_) {}
  }

  Future<void> toggle() async {
    await setTheme(
      state == AppThemeId.dark ? AppThemeId.nier : AppThemeId.dark,
    );
  }
}
