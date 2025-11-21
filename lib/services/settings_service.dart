import 'package:shared_preferences/shared_preferences.dart';

class SettingsService {
  static const String _keyNierDirectory = 'nier_directory';
  static const String _keyLauncherExeOverride = 'launcher_exe_override';
  static const String _keyModloaderDllOverride = 'modloader_dll_override';
  static const String _keyYorhaDllOverride = 'yorha_dll_override';

  final SharedPreferences _prefs;

  SettingsService(this._prefs);

  static Future<SettingsService> initialize() async {
    final prefs = await SharedPreferences.getInstance();
    return SettingsService(prefs);
  }

  // NieR:Automata directory
  String? get nierDirectory => _prefs.getString(_keyNierDirectory);

  Future<void> setNierDirectory(String path) async {
    await _prefs.setString(_keyNierDirectory, path);
  }

  Future<void> clearNierDirectory() async {
    await _prefs.remove(_keyNierDirectory);
  }

  // Launcher executable override
  String? get launcherExeOverride => _prefs.getString(_keyLauncherExeOverride);

  Future<void> setLauncherExeOverride(String? path) async {
    if (path == null || path.isEmpty) {
      await _prefs.remove(_keyLauncherExeOverride);
    } else {
      await _prefs.setString(_keyLauncherExeOverride, path);
    }
  }

  // Modloader DLL override
  String? get modloaderDllOverride => _prefs.getString(_keyModloaderDllOverride);

  Future<void> setModloaderDllOverride(String? path) async {
    if (path == null || path.isEmpty) {
      await _prefs.remove(_keyModloaderDllOverride);
    } else {
      await _prefs.setString(_keyModloaderDllOverride, path);
    }
  }

  // YorHa Protocol DLL override
  String? get yorhaDllOverride => _prefs.getString(_keyYorhaDllOverride);

  Future<void> setYorhaDllOverride(String? path) async {
    if (path == null || path.isEmpty) {
      await _prefs.remove(_keyYorhaDllOverride);
    } else {
      await _prefs.setString(_keyYorhaDllOverride, path);
    }
  }

  // Check if any overrides are set
  bool get hasOverrides =>
      launcherExeOverride != null ||
      modloaderDllOverride != null ||
      yorhaDllOverride != null;

  // Clear all overrides
  Future<void> clearAllOverrides() async {
    await Future.wait([
      _prefs.remove(_keyLauncherExeOverride),
      _prefs.remove(_keyModloaderDllOverride),
      _prefs.remove(_keyYorhaDllOverride),
    ]);
  }
}
