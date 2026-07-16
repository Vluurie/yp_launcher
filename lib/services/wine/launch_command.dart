import 'dart:io';

import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/wine/crossover.dart';
import 'package:yp_launcher/services/wine/proton.dart';
import 'package:yp_launcher/services/wine/wine_paths.dart';

class LaunchCommand {
  final String command;
  final List<String> args;
  final String cwd;
  final Map<String, String>? env;
  final String label;

  const LaunchCommand({
    required this.command,
    required this.args,
    required this.cwd,
    required this.label,
    this.env,
  });

  String get display => [command, ...args].join(' ');
}

List<String> _namsArgs(String gameDirForNams) =>
    [AppStrings.argRun, AppStrings.argNierPath, gameDirForNams];

LaunchCommand buildNativeLaunchCommand({
  required String namsExe,
  required String gameDir,
  required String launcherDir,
}) =>
    LaunchCommand(
      command: namsExe,
      args: _namsArgs(gameDir.replaceAll('/', '\\')),
      cwd: launcherDir,
      label: 'Windows',
    );

/// [namsExe] stays a host path: CrossOver's wine wrapper translates it.
/// [gameDir] becomes a Windows path, because NAMS consumes it Windows-side.
LaunchCommand buildCrossOverLaunchCommand({
  required String namsExe,
  required String gameDir,
  required String launcherDir,
  required String wineBinary,
  required CrossOverBottle bottle,
  required String prefix,
}) =>
    LaunchCommand(
      command: wineBinary,
      args: [
        '--bottle',
        bottle.name,
        '--workdir',
        launcherDir,
        namsExe,
        ..._namsArgs(toWinePath(gameDir)),
      ],
      cwd: launcherDir,
      env: {
        ...createWineEnv(),
        'CX_BOTTLE': bottle.name,
        'WINEPREFIX': prefix,
      },
      label: 'CrossOver Wine (${bottle.name})',
    );

LaunchCommand buildPlainWineLaunchCommand({
  required String namsExe,
  required String gameDir,
  required String launcherDir,
  required String wineBinary,
  String? prefix,
}) =>
    LaunchCommand(
      command: wineBinary,
      args: [namsExe, ..._namsArgs(toWinePath(gameDir))],
      cwd: launcherDir,
      env: {
        ...createWineEnv(),
        if (prefix != null) 'WINEPREFIX': prefix,
      },
      label: 'Wine',
    );

LaunchCommand? buildProtonLaunchCommand({
  required String namsExe,
  required String gameDir,
  required String gameExe,
  required String launcherDir,
  required String protonPath,
}) {
  if (!File(protonPath).existsSync()) return null;

  final compatData = getProtonCompatDataPath(gameExe);
  try {
    Directory(compatData).createSync(recursive: true);
  } catch (_) {}

  return LaunchCommand(
    command: protonPath,
    args: ['run', namsExe, ..._namsArgs(toWinePath(gameDir))],
    cwd: launcherDir,
    env: {
      ...createWineEnv(),
      'STEAM_COMPAT_CLIENT_INSTALL_PATH': getProtonSteamRoot(gameExe),
      'STEAM_COMPAT_DATA_PATH': compatData,
      'SteamAppId': nierSteamAppId,
      'SteamGameId': nierSteamAppId,
    },
    label: 'Proton',
  );
}
