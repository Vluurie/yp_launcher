import 'dart:io';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/settings_service.dart';
import 'package:yp_launcher/services/platform_detection_service.dart';
import 'package:yp_launcher/services/logging_service.dart';

class LauncherSetupService {
  static const String launcherDirName = 'YorHaProtocolLauncher';

  static Future<String> getLauncherDirectory() async {
    if (Platform.isWindows) {
      final appData = Platform.environment['APPDATA'];
      if (appData == null) {
        throw Exception('APPDATA environment variable not found');
      }
      return path.join(appData, launcherDirName);
    } else if (PlatformDetectionService.isWine) {
      final winePrefix = PlatformDetectionService.getWinePrefix();
      if (winePrefix == null) {
        throw Exception(
          'Wine prefix not found. Please set WINEPREFIX environment variable.',
        );
      }

      final driveC = path.join(winePrefix, 'drive_c');
      final usersDir = Directory(path.join(driveC, 'users'));

      if (!await usersDir.exists()) {
        throw Exception(
          'Wine drive_c/users directory not found in $winePrefix',
        );
      }

      final users = await usersDir
          .list()
          .where((e) => e is Directory && !e.path.endsWith('Public'))
          .toList();

      if (users.isEmpty) {
        throw Exception('No user directory found in Wine prefix');
      }

      final userDir = users.first.path;
      final appDataRoaming = path.join(userDir, 'AppData', 'Roaming');

      final appDataDir = Directory(appDataRoaming);
      if (!await appDataDir.exists()) {
        await appDataDir.create(recursive: true);
      }

      final launcherPath = path.join(appDataRoaming, launcherDirName);
      await LoggingService.log('Using Wine AppData path: $launcherPath');
      return launcherPath;
    } else if (Platform.isMacOS) {
      final appSupport = await getApplicationSupportDirectory();
      final launcherPath = path.join(appSupport.path, launcherDirName);
      await LoggingService.log('Using macOS launcher path: $launcherPath');
      return launcherPath;
    } else if (Platform.isLinux) {
      final home = Platform.environment['HOME'];
      if (home == null) {
        throw Exception('HOME environment variable not found');
      }
      final launcherPath = path.join(home, '.local', 'share', launcherDirName);
      await LoggingService.log('Using Linux launcher path: $launcherPath');
      return launcherPath;
    } else {
      throw UnsupportedError(
        'Platform ${Platform.operatingSystem} is not supported',
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
    await LoggingService.log('Setting up launcher files...');
    final launcherDir = await ensureLauncherDirectory();

    await LoggingService.log('Copying launch_nier.exe...');
    final launcherExe = await copyAssetToLauncher(
      'assets/bins/launch_nier.exe',
      'launch_nier.exe',
    );

    await LoggingService.log('Copying modloader.dll...');
    final modloaderDll = await copyAssetToLauncher(
      'assets/bins/modloader.dll',
      'modloader.dll',
    );

    await LoggingService.log('Copying yorha_protocol.dll...');
    final yorhaDll = await copyAssetToLauncher(
      'assets/bins/yorha_protocol.dll',
      'yorha_protocol.dll',
    );

    await LoggingService.log('All launcher files copied successfully');

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

      final launcherExe = File(path.join(launcherDir, 'launch_nier.exe'));
      final modloaderDll = File(path.join(launcherDir, 'modloader.dll'));
      final yorhaDll = File(path.join(launcherDir, 'yorha_protocol.dll'));

      return await launcherExe.exists() &&
          await modloaderDll.exists() &&
          await yorhaDll.exists();
    } catch (e) {
      return false;
    }
  }

  static Future<Map<String, String>> getLauncherPaths() async {
    final launcherDir = await getLauncherDirectory();

    return {
      'launcherDir': launcherDir,
      'launcherExe': path.join(launcherDir, 'launch_nier.exe'),
      'modloaderDll': path.join(launcherDir, 'modloader.dll'),
      'yorhaDll': path.join(launcherDir, 'yorha_protocol.dll'),
    };
  }

  static Future<Map<String, String>> getLauncherPathsWithOverrides(
    SettingsService settings,
  ) async {
    final defaultPaths = await getLauncherPaths();

    return {
      'launcherDir': defaultPaths['launcherDir']!,
      'launcherExe':
          settings.launcherExeOverride ?? defaultPaths['launcherExe']!,
      'modloaderDll':
          settings.modloaderDllOverride ?? defaultPaths['modloaderDll']!,
      'yorhaDll': settings.yorhaDllOverride ?? defaultPaths['yorhaDll']!,
    };
  }
}
