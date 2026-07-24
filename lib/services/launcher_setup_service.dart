import 'dart:async';
import 'dart:io';
import 'package:meta/meta.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/runtime_assets_service.dart';

class LauncherSetupService {
  static Future<void>? _ensureFuture;
  static String? _resolvedRuntimeDir;

  static Future<void> ensureReady() {
    return _ensureFuture ??= _runEnsure().onError((error, stack) {
      _ensureFuture = null;
      throw error!;
    });
  }

  static Future<void> _runEnsure() async {
    _resolvedRuntimeDir = await PlatformAdapter.current.resolveRuntimeDir();
    await RuntimeAssetsService.ensureExtracted();
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

  static String get launcherDirectory {
    final resolved = _resolvedRuntimeDir;
    if (resolved == null) {
      throw StateError(
        'LauncherSetupService.ensureReady() must complete before the '
        'launcher directory is known.',
      );
    }
    return resolved;
  }

  @visibleForTesting
  static void setRuntimeDirForTest(String? dir) {
    _resolvedRuntimeDir = dir;
  }

  static Future<String> getLauncherDirectory() async {
    await ensureReady();
    return launcherDirectory;
  }

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

  static Future<bool> isLauncherDirWritable() async =>
      isDirWritable(await getLauncherDirectory());

  static Future<Map<String, String>> getLauncherPaths() async {
    final dir = await getLauncherDirectory();
    return {
      'launcherDir': dir,
      'namsExe': path.join(dir, AppStrings.namsExeName),
      'pluginsDir': path.join(dir, AppStrings.pluginsDirName),
    };
  }

  static Future<List<String>> findMissingFiles() async {
    final dir = await getLauncherDirectory();
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
