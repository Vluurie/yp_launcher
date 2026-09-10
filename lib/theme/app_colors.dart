import 'package:flutter/material.dart';
import 'package:yp_launcher/theme/app_theme.dart';

class AppColors {
  AppColors._();

  static AppTheme active = AppTheme.dark;

  static bool get isLight => active.brightness == Brightness.light;

  static Color get backgroundPrimary => active.backgroundPrimary;
  static Color get backgroundSecondary => active.backgroundSecondary;
  static Color get backgroundCard => active.backgroundCard;
  static Color get surfaceLight => active.surfaceLight;
  static Color get surfaceMedium => active.surfaceMedium;
  static Color get textPrimary => active.textPrimary;
  static Color get textSecondary => active.textSecondary;
  static Color get textMuted => active.textMuted;
  static Color get accentPrimary => active.accentPrimary;
  static Color get accentSecondary => active.accentSecondary;
  static Color get accentHighlight => active.accentHighlight;
  static Color get borderLight => active.borderLight;
  static Color get borderMedium => active.borderMedium;
  static Color get success => active.success;
  static Color get error => active.error;
  static Color get warning => active.warning;
  static Color get buttonBackground => active.buttonBackground;
  static Color get buttonText => active.buttonText;
  static Color get inputBackground => active.inputBackground;
  static Color get shadow => active.shadow;
  static Color get logBackground => active.logBackground;
  static Color get logHeaderBackground => active.logHeaderBackground;
  static Color get logText => active.logText;
}
