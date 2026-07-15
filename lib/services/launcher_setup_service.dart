import 'dart:async';
import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';

class LauncherSetupService {
  static Future<void>? _ensureFuture;

  static Future<void> ensureReady() {
    return _ensureFuture ??= _runEnsure();
  }

  static Future<void> _runEnsure() async {
    unawaited(_sweepStaleTempArchives());
  }

  static Future<void> _sweepStaleTempArchives() async {
    try {
      final tempDir = Directory.systemTemp;
      if (!await tempDir.exists()) return;
      await for (final entity in tempDir.list(followLinks: false)) {
        if (entity is! Directory) continue;
        final name = path.basename(entity.path);
        if (!name.startsWith('archive_') && !name.startsWith('texture_')) {
          continue;
        }
        try {
          await entity.delete(recursive: true);
        } catch (_) {}
      }
    } catch (_) {}
  }

  static String get launcherDirectory => path.join(
        path.dirname(Platform.resolvedExecutable),
        'data',
        'flutter_assets',
        'assets',
        'bins',
      );

  static Future<String> getLauncherDirectory() async => launcherDirectory;

  static Future<bool> isDirWritable(String dirPath) async {
    try {
      final dir = Directory(dirPath);
      if (!await dir.exists()) {
        await dir.create(recursive: true);
      }
      final probe = File(path.join(dirPath, '.write_test'));
      await probe.writeAsString('ok', flush: true);
      await probe.delete();
      return true;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> isLauncherDirWritable() =>
      isDirWritable(launcherDirectory);

  static Future<Map<String, String>> getLauncherPaths() async {
    final dir = launcherDirectory;
    return {
      'launcherDir': dir,
      'namsExe': path.join(dir, AppStrings.namsExeName),
      'pluginsDir': path.join(dir, AppStrings.pluginsDirName),
    };
  }

  static Future<List<String>> findMissingFiles() async {
    final dir = launcherDirectory;
    final missing = <String>[];

    final namsExe = File(path.join(dir, AppStrings.namsExeName));
    if (!await namsExe.exists()) {
      missing.add(AppStrings.namsExeName);
    }

    final yorhaDll = File(
      path.join(dir, AppStrings.pluginsDirName, AppStrings.yorhaDllName),
    );
    if (!await yorhaDll.exists()) {
      missing.add(AppStrings.yorhaDllName);
    }

    return missing;
  }
}
