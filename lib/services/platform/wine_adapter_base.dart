import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/nier_installation.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';
import 'package:yp_launcher/services/wine/wine_paths.dart';
import 'package:yp_launcher/services/wine/wine_runtime.dart';
import 'package:yp_launcher/services/wine/wine_steam.dart';

/// Hosts that run the Windows binaries through a compatibility layer.
abstract class WineAdapterBase extends PlatformAdapter {
  List<String> get systemSevenZipCandidates;

  @override
  String get sevenZipExeName => '7zz';

  @override
  List<String> sevenZipCandidates(String runtimeDir) => [
        p.join(runtimeDir, sevenZipExeName),
        ...systemSevenZipCandidates,
      ];

  @override
  bool get canLaunchGame => isWineRuntimeAvailable();

  @override
  bool get usesNativeTitleBar => true;

  @override
  Future<LaunchCommand> buildLaunchCommand({
    required String namsExe,
    required String gameDir,
    required String gameExe,
    required String launcherDir,
    required AppLocalizations l10n,
  }) async {
    final runtime = detectWineRuntime(gameExePath: gameExe);
    if (runtime == null) {
      throw LaunchUnavailable(
        l10n.errorNoCompatLayer,
        l10n.errorNoCompatLayerBody,
      );
    }

    _requireHostDriveMapping(runtime, namsExe, l10n);

    switch (runtime.source) {
      case WineSource.crossOver:
        return buildCrossOverLaunchCommand(
          namsExe: namsExe,
          gameDir: gameDir,
          launcherDir: launcherDir,
          wineBinary: runtime.binary,
          bottle: runtime.bottle!,
          prefix: runtime.prefix!,
        );
      case WineSource.proton:
        final command = buildProtonLaunchCommand(
          namsExe: namsExe,
          gameDir: gameDir,
          gameExe: gameExe,
          launcherDir: launcherDir,
          protonPath: runtime.binary,
        );
        if (command == null) {
          throw LaunchUnavailable(
            l10n.errorProtonMissing,
            l10n.errorProtonMissingBody(runtime.binary),
          );
        }
        return command;
      case WineSource.wine:
        return buildPlainWineLaunchCommand(
          namsExe: namsExe,
          gameDir: gameDir,
          launcherDir: launcherDir,
          wineBinary: runtime.binary,
          prefix: runtime.prefix,
        );
    }
  }

  /// NAMS.exe lives outside the prefix, so without a Z: mapping the Windows
  /// side cannot reach it.
  void _requireHostDriveMapping(
    WineRuntime runtime,
    String namsExe,
    AppLocalizations l10n,
  ) {
    final prefix = runtime.prefix;
    if (prefix == null) return;
    if (!toWinePath(namsExe).startsWith('Z:')) return;
    if (Directory(p.join(prefix, 'dosdevices', 'z:')).existsSync()) return;

    throw LaunchUnavailable(l10n.errorNoZDrive, l10n.errorNoZDriveBody(prefix));
  }

  /// Matches the game, not NAMS.exe: our own wine command line carries the
  /// NAMS.exe path in its argv and would match itself.
  @override
  Future<bool> isGameRunning() => _pgrep(AppStrings.gameExeName);

  @override
  Future<bool> terminateGame() async {
    try {
      final result = await Process.run('pkill', ['-f', AppStrings.gameExeName]);
      return result.exitCode == 0;
    } catch (_) {
      return false;
    }
  }

  @override
  Future<List<NierInstallation>> discoverFast() async => findNierInPrefixes();

  @override
  Future<List<NierInstallation>> discoverDeep() async => findNierInPrefixes();

  @override
  String? rejectGameSelection(String pickedExePath, AppLocalizations l10n) {
    if (inferWinePrefixFromPath(pickedExePath) != null) return null;
    return l10n.errorExeOutsidePrefixBody(
      AppStrings.gameExeName,
      pickedExePath,
    );
  }

  @override
  Future<String?> resolveNamsSettingsPath(String? gameDir) async {
    if (gameDir == null || gameDir.isEmpty) return null;
    final prefix = inferWinePrefixFromPath(gameDir);
    if (prefix == null) return null;
    return p.join(getWineRoamingPath(prefix), 'NAMS', 'settings.json');
  }

  Future<bool> _pgrep(String pattern) async {
    try {
      final result = await Process.run('pgrep', ['-f', pattern]);
      return result.exitCode == 0;
    } catch (_) {
      return false;
    }
  }
}
