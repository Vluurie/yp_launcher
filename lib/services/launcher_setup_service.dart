import 'dart:io';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/platform_detection_service.dart';

class LauncherSetupService {
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
    final buffer = byteData.buffer;

    await targetFile.writeAsBytes(
      buffer.asUint8List(byteData.offsetInBytes, byteData.lengthInBytes),
    );

    if (Platform.isLinux || Platform.isMacOS) {
      await Process.run('chmod', ['+x', targetPath]);
    }

    return targetFile;
  }

  static Future<Map<String, String>> setupLauncher() async {
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
    try {
      final launcherDir = await getLauncherDirectory();
      final directory = Directory(launcherDir);

      if (!await directory.exists()) {
        return false;
      }

      final launcherExe = File(path.join(launcherDir, AppStrings.launcherExeName));
      final modloaderDll = File(path.join(launcherDir, AppStrings.modloaderDllName));
      final yorhaDll = File(path.join(launcherDir, AppStrings.yorhaDllName));

      return await launcherExe.exists() &&
          await modloaderDll.exists() &&
          await yorhaDll.exists();
    } catch (_) {
      return false;
    }
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
}
