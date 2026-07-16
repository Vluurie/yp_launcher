import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/services/nier_finder_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/platform/win32_process.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';

class WindowsAdapter extends PlatformAdapter {
  @override
  Future<String> resolveRuntimeDir() async => windowsRuntimeDir;

  static String get windowsRuntimeDir => p.join(
        p.dirname(Platform.resolvedExecutable),
        'data',
        'flutter_assets',
        'assets',
        'bins',
      );

  @override
  String get sevenZipExeName => '7z.exe';

  @override
  List<String> sevenZipCandidates(String runtimeDir) {
    final appDir = p.dirname(Platform.resolvedExecutable);
    return [
      p.join(appDir, 'data', 'flutter_assets', 'assets', 'bins', '7z.exe'),
      p.join(appDir, 'data', 'assets', 'bins', '7z.exe'),
      p.join(appDir, 'assets', 'bins', '7z.exe'),
    ];
  }

  @override
  bool get canLaunchGame => true;

  @override
  bool get supportsDesktopShortcut => true;

  @override
  Future<LaunchCommand> buildLaunchCommand({
    required String namsExe,
    required String gameDir,
    required String gameExe,
    required String launcherDir,
    required AppLocalizations l10n,
  }) async =>
      buildNativeLaunchCommand(
        namsExe: namsExe,
        gameDir: gameDir,
        launcherDir: launcherDir,
      );

  @override
  Future<bool> isGameRunning() async {
    try {
      final result = await Process.run('tasklist', [
        '/FI',
        'IMAGENAME eq ${AppStrings.namsExeName}',
        '/NH',
      ]);
      return result.stdout
          .toString()
          .toLowerCase()
          .contains(AppStrings.namsExeName.toLowerCase());
    } catch (_) {
      return isWin32ProcessRunning(AppStrings.namsExeName);
    }
  }

  @override
  Future<bool> terminateGame() async =>
      terminateWin32ProcessByName(AppStrings.namsExeName);

  @override
  Future<List<NierInstallation>> discoverFast() =>
      NierFinderService.findViaWindowsRegistryAndSteam();

  @override
  Future<List<NierInstallation>> discoverDeep() =>
      NierFinderService.findViaDriveScan();

  @override
  Future<String?> resolveNamsSettingsPath(String? gameDir) async {
    final appData = Platform.environment['APPDATA'];
    if (appData == null || appData.isEmpty) return null;
    return p.join(appData, 'NAMS', 'settings.json');
  }
}
