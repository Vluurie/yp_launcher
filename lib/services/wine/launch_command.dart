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

String formatLaunchCommandScript(
  LaunchCommand command, {
  required bool windows,
}) {
  final quote = windows ? _cmdQuote : _posixQuote;
  final line = [command.command, ...command.args].map(quote).join(' ');
  final env = command.env ?? const <String, String>{};

  if (windows) {
    return [
      'cd /d ${quote(command.cwd)}',
      for (final entry in env.entries) 'set ${entry.key}=${entry.value}',
      line,
    ].join('\r\n');
  }

  final invocation = [
    for (final entry in env.entries) '${entry.key}=${quote(entry.value)}',
    line,
  ].join(' ');
  return 'cd ${quote(command.cwd)} && $invocation';
}

String _posixQuote(String value) => "'${value.replaceAll("'", r"'\''")}'";

String _cmdQuote(String value) => '"$value"';

List<String> namsRunArgs(String nierPath) => [
  AppStrings.argRun,
  AppStrings.argNierPath,
  nierPath,
];

List<String> namsVerifyArgs(String nierPath) => [
  AppStrings.argVerify,
  AppStrings.argNierPath,
  nierPath,
  AppStrings.argJson,
];

LaunchCommand buildNativeLaunchCommand({
  required String namsExe,
  required String gameDir,
  required String launcherDir,
  required List<String> Function(String nierPath) namsArgs,
}) => LaunchCommand(
  command: namsExe,
  args: namsArgs(gameDir.replaceAll('/', '\\')),
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
  required List<String> Function(String nierPath) namsArgs,
}) => LaunchCommand(
  command: wineBinary,
  args: [
    '--bottle',
    bottle.name,
    '--workdir',
    launcherDir,
    namsExe,
    ...namsArgs(toWinePath(gameDir)),
  ],
  cwd: launcherDir,
  env: {...createWineEnv(), 'CX_BOTTLE': bottle.name, 'WINEPREFIX': prefix},
  label: 'CrossOver Wine (${bottle.name})',
);

LaunchCommand buildPlainWineLaunchCommand({
  required String namsExe,
  required String gameDir,
  required String launcherDir,
  required String wineBinary,
  required List<String> Function(String nierPath) namsArgs,
  String? prefix,
}) => LaunchCommand(
  command: wineBinary,
  args: [namsExe, ...namsArgs(toWinePath(gameDir))],
  cwd: launcherDir,
  env: {...createWineEnv(), if (prefix != null) 'WINEPREFIX': prefix},
  label: 'Wine',
);

LaunchCommand? buildProtonLaunchCommand({
  required String namsExe,
  required String gameDir,
  required String gameExe,
  required String launcherDir,
  required String protonPath,
  required List<String> Function(String nierPath) namsArgs,
}) {
  if (!File(protonPath).existsSync()) return null;

  final compatData = getProtonCompatDataPath(gameExe);
  try {
    Directory(compatData).createSync(recursive: true);
  } catch (_) {}

  final steamRoot = getProtonSteamRoot(gameExe);
  final steamHome = steamHomeForClient(steamRoot);

  return LaunchCommand(
    command: protonPath,
    args: ['run', namsExe, ...namsArgs(toWinePath(gameDir))],
    cwd: launcherDir,
    env: {
      ...createWineEnv(),
      'STEAM_COMPAT_CLIENT_INSTALL_PATH': steamRoot,
      'STEAM_COMPAT_DATA_PATH': compatData,
      'SteamAppId': nierSteamAppId,
      'SteamGameId': nierSteamAppId,
      if (steamHome != null) 'HOME': steamHome,
    },
    label: 'Proton',
  );
}
