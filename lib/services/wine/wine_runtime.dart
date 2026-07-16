import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/crossover.dart';
import 'package:yp_launcher/services/wine/proton.dart';
import 'package:yp_launcher/services/wine/wine_paths.dart';

enum WineSource { crossOver, proton, wine }

class WineRuntime {
  final String binary;
  final WineSource source;
  final String label;
  final CrossOverBottle? bottle;
  final String? prefix;

  const WineRuntime({
    required this.binary,
    required this.source,
    required this.label,
    this.bottle,
    this.prefix,
  });

  @override
  String toString() => '$label ($binary)';
}

/// How to run a Windows binary for [gameExePath], in priority order: env
/// override, the prefix the game sits in, Proton, CrossOver, wine on PATH.
WineRuntime? detectWineRuntime({String? gameExePath}) {
  if (Platform.isWindows) return null;

  final gamePath = gameExePath == null ? null : p.normalize(gameExePath);

  final envProton = Platform.environment[protonPathEnv];
  if (envProton != null && envProton.isNotEmpty && File(envProton).existsSync()) {
    return WineRuntime(
      binary: envProton,
      source: WineSource.proton,
      label: 'Proton',
      prefix: p.join(getProtonCompatDataPath(gamePath), 'pfx'),
    );
  }

  final wine = findCrossOverWine();

  if (gamePath != null) {
    final bottle = createBottleFromGamePath(gamePath);
    if (wine != null && bottle != null) return _crossOver(wine, bottle);

    final steam = inferSteamContext(gamePath);
    if (steam != null) {
      final proton = findProtonPath(steam.steamRoot, steam.libraryRoot);
      if (proton != null) {
        return WineRuntime(
          binary: proton,
          source: WineSource.proton,
          label: 'Proton',
          prefix: p.join(steam.compatDataPath, 'pfx'),
        );
      }
    }

    if (wine != null) {
      final resolved = resolveCrossOverBottle(gamePath);
      if (resolved != null) return _crossOver(wine, resolved);
    }
  }

  final onPath = _findOnPath(const ['wine64', 'wine']);
  if (onPath != null) {
    return WineRuntime(
      binary: onPath,
      source: WineSource.wine,
      label: 'Wine',
      prefix: _plainWinePrefix(gamePath),
    );
  }

  return null;
}

bool isWineRuntimeAvailable() =>
    findCrossOverWine() != null ||
    _findOnPath(const ['wine64', 'wine']) != null ||
    (Platform.isLinux && hasAnyProtonInstall());

WineRuntime _crossOver(String wine, CrossOverBottle bottle) => WineRuntime(
      binary: wine,
      source: WineSource.crossOver,
      label: 'CrossOver Wine (${bottle.name})',
      bottle: bottle,
      prefix: bottle.gamePath == null
          ? bottle.path
          : inferWinePrefixFromPath(bottle.gamePath!) ?? bottle.path,
    );

String _plainWinePrefix(String? gamePath) {
  final fromGame = gamePath == null ? null : inferWinePrefixFromPath(gamePath);
  if (fromGame != null) return fromGame;
  final env = Platform.environment['WINEPREFIX'];
  if (env != null && env.isNotEmpty) return env;
  return p.join(Platform.environment['HOME'] ?? '', '.wine');
}

String? _findOnPath(List<String> names) {
  for (final name in names) {
    try {
      final result = Process.runSync('which', [name]);
      if (result.exitCode != 0) continue;
      final found = (result.stdout as String).trim();
      if (found.isNotEmpty && File(found).existsSync()) return found;
    } catch (_) {}
  }
  return null;
}
