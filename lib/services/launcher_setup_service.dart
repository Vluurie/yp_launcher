import 'dart:async';
import 'dart:io';
import 'package:crypto/crypto.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/platform_detection_service.dart';

class LauncherSetupService {
  static bool? _setupCached;
  static Future<void>? _ensureFuture;

  static Future<void> ensureReady() {
    return _ensureFuture ??= _runEnsure();
  }

  static Future<void> _runEnsure() async {
    unawaited(_sweepStaleTempArchives());
    if (await isLauncherSetup()) return;
    await setupLauncher();
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

  static Future<String> getLauncherDirectory() async {
    if (Platform.isWindows) {
      final appData = Platform.environment['APPDATA'];
      if (appData == null) {
        throw Exception(AppStrings.errorAppDataNotFound);
      }
      return path.join(appData, AppStrings.launcherDirName);
    } else if (PlatformDetectionService.isWine) {
      final winePrefix = PlatformDetectionService.getWinePrefix();
      if (winePrefix == null) {
        throw Exception(AppStrings.errorWinePrefixNotFound);
      }

      final driveC = path.join(winePrefix, 'drive_c');
      final usersDir = Directory(path.join(driveC, 'users'));

      if (!await usersDir.exists()) {
        throw Exception(AppStrings.errorWineUsersNotFound(winePrefix));
      }

      final users = await usersDir
          .list()
          .where((e) => e is Directory && !e.path.endsWith('Public'))
          .toList();

      if (users.isEmpty) {
        throw Exception(AppStrings.errorNoWineUser);
      }

      final userDir = users.first.path;
      final appDataRoaming = path.join(userDir, 'AppData', 'Roaming');

      final appDataDir = Directory(appDataRoaming);
      if (!await appDataDir.exists()) {
        await appDataDir.create(recursive: true);
      }

      return path.join(appDataRoaming, AppStrings.launcherDirName);
    } else if (Platform.isMacOS) {
      final appSupport = await getApplicationSupportDirectory();
      return path.join(appSupport.path, AppStrings.launcherDirName);
    } else if (Platform.isLinux) {
      final home = Platform.environment['HOME'];
      if (home == null) {
        throw Exception(AppStrings.errorHomeNotFound);
      }
      return path.join(home, '.local', 'share', AppStrings.launcherDirName);
    } else {
      throw UnsupportedError(
        AppStrings.errorPlatformNotSupported(Platform.operatingSystem),
      );
    }
  }

  static Future<Directory> ensureLauncherDirectory() async {
    final launcherPath = await getLauncherDirectory();
    final directory = Directory(launcherPath);

    if (!await directory.exists()) {
      await directory.create(recursive: true);
    }

    return directory;
  }

  static Future<File> copyAssetToLauncher(
    String assetPath,
    String fileName,
  ) async {
    final launcherDir = await ensureLauncherDirectory();
    final targetPath = path.join(launcherDir.path, fileName);
    final targetFile = File(targetPath);

    final byteData = await rootBundle.load(assetPath);
    final bytes = byteData.buffer.asUint8List(
      byteData.offsetInBytes,
      byteData.lengthInBytes,
    );

    final raf = await targetFile.open(mode: FileMode.write);
    try {
      const chunkSize = 1024 * 1024;
      for (var offset = 0; offset < bytes.length; offset += chunkSize) {
        final end = (offset + chunkSize).clamp(0, bytes.length);
        await raf.writeFrom(bytes, offset, end);
        await Future<void>.delayed(Duration.zero);
      }
    } finally {
      await raf.close();
    }

    if (Platform.isLinux || Platform.isMacOS) {
      await Process.run('chmod', ['+x', targetPath]);
    }

    return targetFile;
  }

  static Future<Map<String, String>> setupLauncher() async {
    _setupCached = null;
    final launcherDir = await ensureLauncherDirectory();

    final launcherExe = await copyAssetToLauncher(
      AppStrings.assetLauncherExe,
      AppStrings.launcherExeName,
    );

    final modloaderDll = await copyAssetToLauncher(
      AppStrings.assetModloaderDll,
      AppStrings.modloaderDllName,
    );

    final yorhaDll = await copyAssetToLauncher(
      AppStrings.assetYorhaDll,
      AppStrings.yorhaDllName,
    );

    return {
      'launcherDir': launcherDir.path,
      'launcherExe': launcherExe.path,
      'modloaderDll': modloaderDll.path,
      'yorhaDll': yorhaDll.path,
    };
  }

  static Future<bool> isLauncherSetup() async {
    if (_setupCached != null) return _setupCached!;
    try {
      final launcherDir = await getLauncherDirectory();
      final directory = Directory(launcherDir);

      if (!await directory.exists()) {
        _setupCached = false;
        return false;
      }

      final files = [
        (AppStrings.assetLauncherExe, AppStrings.launcherExeName),
        (AppStrings.assetModloaderDll, AppStrings.modloaderDllName),
        (AppStrings.assetYorhaDll, AppStrings.yorhaDllName),
      ];

      for (final (assetPath, fileName) in files) {
        final cachedFile = File(path.join(launcherDir, fileName));
        if (!await cachedFile.exists()) {
          _setupCached = false;
          return false;
        }

        if (!await _assetMatchesCached(assetPath, cachedFile)) {
          await Directory(launcherDir).delete(recursive: true);
          _setupCached = false;
          return false;
        }
      }

      _setupCached = true;
      return true;
    } catch (_) {
      _setupCached = false;
      return false;
    }
  }

  static Future<bool> _assetMatchesCached(
    String assetPath,
    File cachedFile,
  ) async {
    final byteData = await rootBundle.load(assetPath);
    final assetBytes = byteData.buffer.asUint8List(
      byteData.offsetInBytes,
      byteData.lengthInBytes,
    );
    final assetHash = sha256.convert(assetBytes).toString();
    final cachedHash = await IsolateService.run(
      _hashFileSync,
      cachedFile.path,
    );
    return cachedHash != null && assetHash == cachedHash;
  }

  static Future<void> invalidateDecryptCacheIfChanged(
    String sourceExePath,
  ) async {
    try {
      final launcherDir = await getLauncherDirectory();
      final decryptDir = Directory(path.join(launcherDir, 'nierdecrypt'));
      if (!await decryptDir.exists()) return;

      final cachedExe = File(path.join(decryptDir.path, AppStrings.gameExeName));
      if (!await cachedExe.exists()) return;

      final sourceHash = await IsolateService.run(
        _hashFileSync,
        sourceExePath,
      );
      if (sourceHash == null) return;

      final sidecar = File(
        path.join(decryptDir.path, '${AppStrings.gameExeName}.source.sha256'),
      );
      String? recordedHash;
      if (await sidecar.exists()) {
        try {
          recordedHash = (await sidecar.readAsString()).trim();
        } catch (_) {}
      }

      if (recordedHash == sourceHash) return;

      try {
        await cachedExe.delete();
      } catch (_) {}
      try {
        await sidecar.writeAsString(sourceHash);
      } catch (_) {}
    } catch (_) {}
  }

  static Future<void> recordSourceExeHash(String sourceExePath) async {
    try {
      final launcherDir = await getLauncherDirectory();
      final decryptDir = Directory(path.join(launcherDir, 'nierdecrypt'));
      if (!await decryptDir.exists()) {
        await decryptDir.create(recursive: true);
      }
      final sourceHash = await IsolateService.run(
        _hashFileSync,
        sourceExePath,
      );
      if (sourceHash == null) return;
      final sidecar = File(
        path.join(decryptDir.path, '${AppStrings.gameExeName}.source.sha256'),
      );
      await sidecar.writeAsString(sourceHash);
    } catch (_) {}
  }

  static Future<Map<String, String>> getLauncherPaths() async {
    final launcherDir = await getLauncherDirectory();

    return {
      'launcherDir': launcherDir,
      'launcherExe': path.join(launcherDir, AppStrings.launcherExeName),
      'modloaderDll': path.join(launcherDir, AppStrings.modloaderDllName),
      'yorhaDll': path.join(launcherDir, AppStrings.yorhaDllName),
    };
  }

  static Future<List<String>> findMissingFiles() async {
    final launcherDir = await getLauncherDirectory();
    if (!await Directory(launcherDir).exists()) return [];
    final missing = <String>[];
    for (final name in [
      AppStrings.launcherExeName,
      AppStrings.modloaderDllName,
      AppStrings.yorhaDllName,
    ]) {
      if (!await File(path.join(launcherDir, name)).exists()) {
        missing.add(name);
      }
    }
    return missing;
  }
}

String? _hashFileSync(String filePath) {
  try {
    final bytes = File(filePath).readAsBytesSync();
    return sha256.convert(bytes).toString();
  } catch (_) {
    return null;
  }
}
