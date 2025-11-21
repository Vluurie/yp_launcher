import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/settings_service.dart';

class LauncherSetupService {
  static const String launcherDirName = 'YorHaProtocolLauncher';

  /// Get the launcher directory path (cross-platform)
  static Future<String> getLauncherDirectory() async {
    if (Platform.isWindows) {
      final appData = Platform.environment['APPDATA'];
      if (appData == null) {
        throw Exception('APPDATA environment variable not found');
      }
      return path.join(appData, launcherDirName);
    } else if (Platform.isMacOS) {
      final appSupport = await getApplicationSupportDirectory();
      return path.join(appSupport.path, launcherDirName);
    } else if (Platform.isLinux) {
      final home = Platform.environment['HOME'];
      if (home == null) {
        throw Exception('HOME environment variable not found');
      }
      return path.join(home, '.local', 'share', launcherDirName);
    } else {
      throw UnsupportedError('Platform ${Platform.operatingSystem} is not supported');
    }
  }

  /// Ensure the launcher directory exists
  static Future<Directory> ensureLauncherDirectory() async {
    final launcherPath = await getLauncherDirectory();
    final directory = Directory(launcherPath);

    if (!await directory.exists()) {
      await directory.create(recursive: true);
    }

    return directory;
  }

  /// Copy a file from assets to the launcher directory
  static Future<File> copyAssetToLauncher(
    String assetPath,
    String fileName,
  ) async {
    final launcherDir = await ensureLauncherDirectory();
    final targetPath = path.join(launcherDir.path, fileName);
    final targetFile = File(targetPath);

    // Load the asset as bytes
    final byteData = await rootBundle.load(assetPath);
    final buffer = byteData.buffer;

    // Write to the target file
    await targetFile.writeAsBytes(
      buffer.asUint8List(byteData.offsetInBytes, byteData.lengthInBytes),
    );

    // On Unix-like systems, make the file executable
    if (Platform.isLinux || Platform.isMacOS) {
      await Process.run('chmod', ['+x', targetPath]);
    }

    return targetFile;
  }

  /// Setup the launcher by copying all necessary files (runs in isolate)
  static Future<Map<String, String>> setupLauncher() async {
    return await compute(_setupLauncherInIsolate, null);
  }

  /// Check if the launcher is already set up
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

  /// Get paths to the launcher files (assumes already set up)
  static Future<Map<String, String>> getLauncherPaths() async {
    final launcherDir = await getLauncherDirectory();

    return {
      'launcherDir': launcherDir,
      'launcherExe': path.join(launcherDir, 'launch_nier.exe'),
      'modloaderDll': path.join(launcherDir, 'modloader.dll'),
      'yorhaDll': path.join(launcherDir, 'yorha_protocol.dll'),
    };
  }

  /// Get paths with settings overrides applied
  static Future<Map<String, String>> getLauncherPathsWithOverrides(
    SettingsService settings,
  ) async {
    final defaultPaths = await getLauncherPaths();

    return {
      'launcherDir': defaultPaths['launcherDir']!,
      'launcherExe': settings.launcherExeOverride ?? defaultPaths['launcherExe']!,
      'modloaderDll': settings.modloaderDllOverride ?? defaultPaths['modloaderDll']!,
      'yorhaDll': settings.yorhaDllOverride ?? defaultPaths['yorhaDll']!,
    };
  }
}

// Top-level function for isolate
Future<Map<String, String>> _setupLauncherInIsolate(void _) async {
  final launcherDir = await LauncherSetupService.ensureLauncherDirectory();

  // Copy the launcher executable
  final launcherExe = await LauncherSetupService.copyAssetToLauncher(
    'assets/bins/launch_nier.exe',
    'launch_nier.exe',
  );

  // Copy the modloader DLL
  final modloaderDll = await LauncherSetupService.copyAssetToLauncher(
    'assets/bins/modloader.dll',
    'modloader.dll',
  );

  // Copy the YorHa Protocol DLL
  final yorhaDll = await LauncherSetupService.copyAssetToLauncher(
    'assets/bins/yorha_protocol.dll',
    'yorha_protocol.dll',
  );

  return {
    'launcherDir': launcherDir.path,
    'launcherExe': launcherExe.path,
    'modloaderDll': modloaderDll.path,
    'yorhaDll': yorhaDll.path,
  };
}
