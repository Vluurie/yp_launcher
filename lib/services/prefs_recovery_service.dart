import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

class PrefsRecoveryService {
  PrefsRecoveryService._();

  static const _fileName = 'shared_preferences.json';
  static const _backupName = 'shared_preferences.corrupt.json';

  static Future<void> ensureReadable() async {
    if (await _readable()) return;
    if (!Platform.isWindows && !Platform.isLinux) return;

    final File file;
    try {
      final dir = await getApplicationSupportDirectory();
      file = File(p.join(dir.path, _fileName));
      if (!file.existsSync()) return;
    } catch (_) {
      return;
    }

    try {
      file.copySync(p.join(file.parent.path, _backupName));
    } catch (_) {}
    try {
      file.deleteSync();
    } catch (_) {
      return;
    }

    await _readable();
  }

  static Future<bool> _readable() async {
    try {
      await SharedPreferences.getInstance();
      return true;
    } catch (_) {
      return false;
    }
  }
}
