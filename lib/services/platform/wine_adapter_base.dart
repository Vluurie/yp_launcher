import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/l10n/app_localizations.dart';
import 'package:yp_launcher/models/nier_installation.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';
import 'package:yp_launcher/services/wine/proton.dart';
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
  Future<LaunchCommand> buildNamsCommand({
    required List<String> Function(String nierPath) namsArgs,
    required String namsExe,
    required String gameDir,
    required String gameExe,
    required String launcherDir,
    required AppLocalizations l10n,
  }) async {
    final runtime = detectWineRuntime(gameExePath: gameExe);
    if (runtime == null) {
      if (Platform.isLinux) {
        throw LaunchUnavailable(
          l10n.errorNoCompatLayerLinux,
          l10n.errorNoCompatLayerLinuxBody,
        );
      }
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
          namsArgs: namsArgs,
        );
      case WineSource.proton:
        final command = buildProtonLaunchCommand(
          namsExe: namsExe,
          gameDir: gameDir,
          gameExe: gameExe,
          launcherDir: launcherDir,
          protonPath: runtime.binary,
          namsArgs: namsArgs,
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
          namsArgs: namsArgs,
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
    // Proton creates the Z: mapping on first run; only enforce an existing pfx.
    if (!Directory(prefix).existsSync()) return;
    if (Directory(p.join(prefix, 'dosdevices', 'z:')).existsSync()) return;

    throw LaunchUnavailable(l10n.errorNoZDrive, l10n.errorNoZDriveBody(prefix));
  }

  /// NAMS hosts the game in-process, so NAMS.exe is what runs and what has to
  /// be stopped. Wine rewrites argv to the Windows exe path, which is what
  /// makes this match.
  @override
  Future<bool> isGameRunning() => _pgrep(AppStrings.namsExeName);

  @override
  Future<bool> terminateGame() async {
    try {
      final result = await Process.run('pkill', ['-f', AppStrings.namsExeName]);
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
    if (inferSteamContext(pickedExePath) != null) return null;
    return l10n.errorExeOutsidePrefixBody(
      AppStrings.gameExeName,
      pickedExePath,
    );
  }

  @override
  Future<String?> resolveNamsSettingsPath(String? gameDir) async {
    if (gameDir == null || gameDir.isEmpty) return null;
    final prefix = inferWinePrefixFromPath(gameDir);
    if (prefix != null) {
      return p.join(getWineRoamingPath(prefix), 'NAMS', 'settings.json');
    }
    final steam = inferSteamContext(gameDir);
    if (steam != null) {
      final pfx = p.join(steam.compatDataPath, 'pfx');
      return p.join(getWineRoamingPath(pfx), 'NAMS', 'settings.json');
    }
    return null;
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
