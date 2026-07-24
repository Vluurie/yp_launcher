import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/log_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';

class CacheClearResult {
  final int logsDeleted;
  final bool detectionReset;
  final bool namsSettingsDeleted;
  final bool stampDeleted;

  const CacheClearResult({
    this.logsDeleted = 0,
    this.detectionReset = false,
    this.namsSettingsDeleted = false,
    this.stampDeleted = false,
  });
}

class CacheService {
  CacheService._();

  static const _prefKeyDetectedDir = 'detected_notifications_dir';

  static Future<CacheClearResult> clearAll(String gameDir) async {
    final logs = await _clearLogs();
    final detection = await _resetDetectionCache();
    final nams = await _deleteNamsSettings(gameDir);
    final stamp = await _deleteRuntimeStamp();
    return CacheClearResult(
      logsDeleted: logs,
      detectionReset: detection,
      namsSettingsDeleted: nams,
      stampDeleted: stamp,
    );
  }

  static Future<int> _clearLogs() async {
    var deleted = 0;
    try {
      final dir = Directory(LogService.logsDirectory);
      if (!await dir.exists()) return 0;
      await for (final entity in dir.list(followLinks: false)) {
        if (entity is! File) continue;
        try {
          await entity.delete();
          deleted++;
        } catch (_) {}
      }
    } catch (_) {}
    return deleted;
  }

  static Future<bool> _resetDetectionCache() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      return await prefs.remove(_prefKeyDetectedDir);
    } catch (_) {
      return false;
    }
  }

  static Future<bool> _deleteNamsSettings(String gameDir) async {
    try {
      final path = await PlatformAdapter.current
          .resolveNamsSettingsPath(gameDir.isEmpty ? null : gameDir);
      if (path == null) return false;
      final file = File(path);
      if (!await file.exists()) return false;
      await file.delete();
      return true;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> _deleteRuntimeStamp() async {
    try {
      final dir = await LauncherSetupService.getLauncherDirectory();
      final stamp = File(p.join(dir, '.stamp'));
      if (!await stamp.exists()) return false;
      await stamp.delete();
      return true;
    } catch (_) {
      return false;
    }
  }
}
