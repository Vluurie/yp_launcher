import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:shared_preferences/shared_preferences.dart';

class Win10CompatSetupService {
  Win10CompatSetupService._();

  static const String _batName = 'win10setup.bat';
  static const String _prefKeyPrefix = 'win10setup_done_';

  /// On Windows 11, NieR's first-run compatibility setup fails OS verification
  /// and never runs. The shipped `win10setup.bat` cleans up the unused
  /// compatibility files and only needs to run once per install.
  ///
  /// Fires once per game directory, only on Windows 11+, only when the bat
  /// file is present. Errors are swallowed — this is best-effort.
  static Future<void> ensureRanForDirectory(String gameDir) async {
    if (!Platform.isWindows) return;
    if (!_isWindows11OrNewer()) return;

    final batPath = path.join(gameDir, _batName);
    if (!await File(batPath).exists()) return;

    final prefs = await SharedPreferences.getInstance();
    final key = '$_prefKeyPrefix${_normalizeKey(gameDir)}';
    if (prefs.getBool(key) == true) return;

    try {
      await Process.run(
        'cmd.exe',
        ['/c', _batName],
        workingDirectory: gameDir,
        runInShell: false,
      );
    } catch (_) {}

    await prefs.setBool(key, true);
  }

  static bool _isWindows11OrNewer() {
    try {
      final raw = Platform.operatingSystemVersion;
      final m = RegExp(r'Build\s+(\d+)').firstMatch(raw);
      if (m == null) return false;
      final build = int.tryParse(m.group(1)!);
      if (build == null) return false;
      return build >= 22000;
    } catch (_) {
      return false;
    }
  }

  static String _normalizeKey(String dir) =>
      dir.toLowerCase().replaceAll(RegExp(r'[^a-z0-9]'), '_');
}
