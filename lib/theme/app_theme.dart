import 'package:flutter/material.dart';

enum AppThemeId { dark, nier }

class AppTheme {
  final AppThemeId id;
  final Brightness brightness;

  final Color backgroundPrimary;
  final Color backgroundSecondary;
  final Color backgroundCard;
  final Color surfaceLight;
  final Color surfaceMedium;
  final Color textPrimary;
  final Color textSecondary;
  final Color textMuted;
  final Color accentPrimary;
  final Color accentSecondary;
  final Color accentHighlight;
  final Color borderLight;
  final Color borderMedium;
  final Color success;
  final Color error;
  final Color warning;
  final Color buttonBackground;
  final Color buttonText;
  final Color inputBackground;
  final Color shadow;
  final Color logBackground;
  final Color logHeaderBackground;
  final Color logText;

  const AppTheme({
    required this.id,
    required this.brightness,
    required this.backgroundPrimary,
    required this.backgroundSecondary,
    required this.backgroundCard,
    required this.surfaceLight,
    required this.surfaceMedium,
    required this.textPrimary,
    required this.textSecondary,
    required this.textMuted,
    required this.accentPrimary,
    required this.accentSecondary,
    required this.accentHighlight,
    required this.borderLight,
    required this.borderMedium,
    required this.success,
    required this.error,
    required this.warning,
    required this.buttonBackground,
    required this.buttonText,
    required this.inputBackground,
    required this.shadow,
    required this.logBackground,
    required this.logHeaderBackground,
    required this.logText,
  });

  static const AppTheme dark = AppTheme(
    id: AppThemeId.dark,
    brightness: Brightness.dark,
    backgroundPrimary: Color(0xFF18181C),
    backgroundSecondary: Color(0xFF141418),
    backgroundCard: Color(0xFF222228),
    surfaceLight: Color(0xFF2C2C34),
    surfaceMedium: Color(0xFF1E1E24),
    textPrimary: Color(0xFFFFFFFF),
    textSecondary: Color(0xFFD4D0C8),
    textMuted: Color(0xFF9E9A90),
    accentPrimary: Color(0xFFD4A86A),
    accentSecondary: Color(0xFFB89860),
    accentHighlight: Color(0xFFE4C88B),
    borderLight: Color(0xFF3A3A42),
    borderMedium: Color(0xFF4E4E58),
    success: Color(0xFF8BC34A),
    error: Color(0xFFEF5350),
    warning: Color(0xFFFFB74D),
    buttonBackground: Color(0xFFD4A86A),
    buttonText: Color(0xFF1A1A1E),
    inputBackground: Color(0xFF2A2A32),
    shadow: Color(0x55000000),
    logBackground: Color(0xFF222228),
    logHeaderBackground: Color(0xFF1E1E24),
    logText: Color(0xFFD4D0C8),
  );

  static const AppTheme nier = AppTheme(
    id: AppThemeId.nier,
    brightness: Brightness.light,
    backgroundPrimary: Color(0xFFDAD4BB),
    backgroundSecondary: Color(0xFFD2CBB0),
    backgroundCard: Color(0xFFD0C9AE),
    surfaceLight: Color(0xFFC9C2A6),
    surfaceMedium: Color(0xFFB4AF9A),
    textPrimary: Color(0xFF3A382F),
    textSecondary: Color(0xFF4E4B42),
    textMuted: Color(0xFF6E6857),
    accentPrimary: Color(0xFF4E4B42),
    accentSecondary: Color(0xFF635F53),
    accentHighlight: Color(0xFF3A3830),
    borderLight: Color(0xFFB4AF9A),
    borderMedium: Color(0xFF908A78),
    success: Color(0xFF57632F),
    error: Color(0xFFD75028),
    warning: Color(0xFFB4682A),
    buttonBackground: Color(0xFF4E4B42),
    buttonText: Color(0xFFDAD4BB),
    inputBackground: Color(0xFFC9C2A6),
    shadow: Color(0x334E4B42),
    logBackground: Color(0xFFD0C9AE),
    logHeaderBackground: Color(0xFFB4AF9A),
    logText: Color(0xFF4E4B42),
  );

  static AppTheme byId(AppThemeId id) =>
      id == AppThemeId.nier ? nier : dark;
}
