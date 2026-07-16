import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/wine/wine_paths.dart';

/// Overrides the wine binary. Point it at a script to test launching without
/// CrossOver installed.
const wineCommandEnv = 'YP_WINE_COMMAND';

/// Overrides the bottle search roots (`:`-separated). Lets tests inject a
/// fake bottle tree.
const bottleRootsEnv = 'YP_BOTTLE_ROOTS';

@visibleForTesting
String? overrideWineCommand;

@visibleForTesting
List<String>? overrideBottleRoots;

String? _configuredWineCommand() {
  final value = overrideWineCommand ?? Platform.environment[wineCommandEnv];
  return (value == null || value.isEmpty) ? null : value;
}

List<String>? _configuredBottleRoots() {
  if (overrideBottleRoots != null) return overrideBottleRoots;
  final value = Platform.environment[bottleRootsEnv];
  if (value == null || value.isEmpty) return null;
  return value.split(':');
}

class CrossOverBottle {
  final String name;
  final String path;
  final String? gamePath;

  const CrossOverBottle({
    required this.name,
    required this.path,
    this.gamePath,
  });

  CrossOverBottle withGamePath(String? value) => CrossOverBottle(
        name: name,
        path: path,
        gamePath: value ?? gamePath,
      );

  @override
  String toString() => 'CrossOverBottle($name at $path)';
}

/// CrossOver >= 22 ships only `wine`; `wine64` is kept for CX <= 21.
const _wineCandidates = [
  '/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine64',
  '/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine',
  '/Applications/CrossOver Preview.app/Contents/SharedSupport/CrossOver/bin/wine64',
  '/Applications/CrossOver Preview.app/Contents/SharedSupport/CrossOver/bin/wine',
  '/opt/cxoffice/bin/wine64',
  '/opt/cxoffice/bin/wine',
  '/opt/crossover/bin/wine64',
  '/opt/crossover/bin/wine',
];

String? findCrossOverWine() {
  final override = _configuredWineCommand();
  if (override != null) {
    return File(override).existsSync() ? override : null;
  }
  for (final candidate in _wineCandidates) {
    if (File(candidate).existsSync()) return candidate;
  }
  return null;
}

/// Where CrossOver keeps bottles, which differs between macOS and Linux.
List<String> crossOverBottleRoots() {
  final override = _configuredBottleRoots();
  if (override != null) {
    return override
        .where((root) => root.isNotEmpty && Directory(root).existsSync())
        .toList();
  }

  final home = Platform.environment['HOME'];
  if (home == null || home.isEmpty) return const [];

  final candidates = Platform.isMacOS
      ? [
          p.join(home, 'Library', 'Application Support', 'CrossOver', 'Bottles'),
          p.join(home, 'Library', 'Application Support', 'CrossOver Preview',
              'Bottles'),
        ]
      : [
          p.join(home, '.cxoffice'),
          p.join(home, '.var', 'app', 'com.codeweavers.CrossOver', 'data',
              'crossover', 'bottles'),
        ];

  return candidates.where((root) => Directory(root).existsSync()).toList();
}

/// A directory is a bottle when it contains `drive_c`.
List<CrossOverBottle> listCrossOverBottles() {
  final bottles = <CrossOverBottle>[];
  for (final root in crossOverBottleRoots()) {
    try {
      for (final entity in Directory(root).listSync(followLinks: false)) {
        if (entity is! Directory) continue;
        if (!Directory(p.join(entity.path, 'drive_c')).existsSync()) continue;
        bottles.add(CrossOverBottle(
          name: p.basename(entity.path),
          path: entity.path,
        ));
      }
    } catch (_) {}
  }
  return bottles;
}

CrossOverBottle? createBottleFromGamePath(String gamePath) {
  final prefix = inferWinePrefixFromPath(gamePath);
  if (prefix == null || !isCrossOverPrefix(prefix)) return null;
  return CrossOverBottle(
    name: p.basename(prefix),
    path: prefix,
    gamePath: gamePath,
  );
}

/// Resolves the bottle for [gamePath]: derived from the path itself, then
/// `$CX_BOTTLE`, then the single bottle that has the game installed.
CrossOverBottle? resolveCrossOverBottle(
  String gamePath, {
  bool searchContainingGame = true,
}) {
  final fromGamePath = createBottleFromGamePath(gamePath);
  if (fromGamePath != null) return fromGamePath;

  final envBottle = Platform.environment['CX_BOTTLE'];
  if (envBottle != null && envBottle.isNotEmpty) {
    final named = _bottleByName(envBottle);
    if (named != null) return _withResolvedGamePath(named, gamePath);
  }

  if (!searchContainingGame) return null;
  return _bottleContainingGame(gamePath);
}

List<String> nierCandidatesInBottle(String bottlePath) => [
      for (final programFiles in const ['Program Files (x86)', 'Program Files'])
        p.join(bottlePath, 'drive_c', programFiles, 'Steam', 'steamapps',
            'common', 'NieRAutomata', AppStrings.gameExeName),
    ];

CrossOverBottle? _bottleByName(String name) {
  final target = name.toLowerCase();
  for (final bottle in listCrossOverBottles()) {
    if (bottle.name.toLowerCase() == target) return bottle;
  }
  return null;
}

CrossOverBottle? _bottleContainingGame(String gamePath) {
  final bottles = listCrossOverBottles();

  for (final bottle in bottles) {
    if (_isInside(gamePath, bottle.path)) return bottle.withGamePath(gamePath);
  }

  final withGame = bottles
      .where((bottle) => nierCandidatesInBottle(bottle.path)
          .any((candidate) => File(candidate).existsSync()))
      .toList();
  return withGame.length == 1
      ? _withResolvedGamePath(withGame.first, gamePath)
      : null;
}

CrossOverBottle _withResolvedGamePath(CrossOverBottle bottle, String gamePath) {
  if (_isInside(gamePath, bottle.path)) return bottle.withGamePath(gamePath);
  for (final candidate in nierCandidatesInBottle(bottle.path)) {
    if (File(candidate).existsSync()) return bottle.withGamePath(candidate);
  }
  return bottle;
}

bool _isInside(String target, String parent) {
  final relative = p.relative(p.normalize(target), from: p.normalize(parent));
  return relative == '.' ||
      (!relative.startsWith('..') && !p.isAbsolute(relative));
}
