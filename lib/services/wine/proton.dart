import 'dart:io';

import 'package:path/path.dart' as p;

const nierSteamAppId = '524220';

const protonPathEnv = 'YP_PROTON_PATH';

const _libraryMarker = '/steamapps/common/';

class SteamContext {
  final String steamRoot;
  final String libraryRoot;
  final String compatDataPath;

  const SteamContext({
    required this.steamRoot,
    required this.libraryRoot,
    required this.compatDataPath,
  });
}

bool isProtonRuntimePath(String runtimePath) =>
    p.basename(runtimePath).toLowerCase() == 'proton';

/// Splits a Steam-installed game path into its library and Steam roots.
SteamContext? inferSteamContext(String gamePath) {
  final normalized = p.normalize(gamePath);
  final index = normalized.toLowerCase().indexOf(_libraryMarker);
  if (index == -1) return null;

  final libraryRoot = normalized.substring(0, index);
  return SteamContext(
    steamRoot: _steamRootForLibrary(libraryRoot) ?? libraryRoot,
    libraryRoot: libraryRoot,
    compatDataPath:
        p.join(libraryRoot, 'steamapps', 'compatdata', nierSteamAppId),
  );
}

String getProtonCompatDataPath(String? gamePath) {
  final fromEnv = Platform.environment['STEAM_COMPAT_DATA_PATH'];
  if (fromEnv != null && fromEnv.isNotEmpty) return fromEnv;

  final context = gamePath == null ? null : inferSteamContext(gamePath);
  if (context != null) return context.compatDataPath;

  return p.join(_home, '.steam', 'steam', 'steamapps', 'compatdata',
      nierSteamAppId);
}

String getProtonSteamRoot(String? gamePath) {
  final fromEnv = Platform.environment['STEAM_COMPAT_CLIENT_INSTALL_PATH'];
  if (fromEnv != null && fromEnv.isNotEmpty) return fromEnv;

  final context = gamePath == null ? null : inferSteamContext(gamePath);
  if (context != null) return context.steamRoot;

  return p.join(_home, '.steam', 'steam');
}

/// Newest Proton build across the usual install locations.
String? findProtonPath(String steamRoot, String libraryRoot) {
  final candidates = [
    ..._protonCandidates(p.join(steamRoot, 'compatibilitytools.d')),
    ..._protonCandidates(p.join(steamRoot, 'steamapps', 'common')),
    ..._protonCandidates(p.join(libraryRoot, 'steamapps', 'common')),
  ];
  return candidates.isEmpty ? null : candidates.first;
}

/// Native Steam install roots: deb/tarball, Snap and Flatpak.
List<String> steamRootCandidates() => [
      p.join(_home, '.steam', 'steam'),
      p.join(_home, '.local', 'share', 'Steam'),
      p.join(_home, 'snap', 'steam', 'common', '.local', 'share', 'Steam'),
      p.join(_home, '.var', 'app', 'com.valvesoftware.Steam', '.local', 'share',
          'Steam'),
    ].where((root) => Directory(root).existsSync()).toList();

/// True when any Steam root has a Proton build.
bool hasAnyProtonInstall() {
  for (final root in steamRootCandidates()) {
    if (findProtonPath(root, root) != null) return true;
  }
  return false;
}

/// The `$HOME` a Proton child needs so lsteamclient finds
/// `$HOME/.steam/sdk64/steamclient.so`. Null when the real home already has it.
String? steamHomeForClient(String steamClientInstallPath) {
  if (_hasSteamClient(_home)) return null;
  for (final candidate in _steamHomeCandidatesFor(steamClientInstallPath)) {
    if (_hasSteamClient(candidate)) return candidate;
  }
  return null;
}

bool _hasSteamClient(String home) => home.isNotEmpty &&
    File(p.join(home, '.steam', 'sdk64', 'steamclient.so')).existsSync();

List<String> _steamHomeCandidatesFor(String steamRoot) {
  final candidates = <String>[];
  var dir = steamRoot;
  for (var i = 0; i < 4; i++) {
    final parent = p.dirname(dir);
    if (parent == dir) break;
    candidates.add(parent);
    dir = parent;
  }
  return candidates;
}

String get _home => Platform.environment['HOME'] ?? '';

String? _steamRootForLibrary(String libraryRoot) {
  final target = p.normalize(libraryRoot);
  for (final candidate in steamRootCandidates()) {
    if (p.normalize(candidate) == target) return candidate;
  }
  return null;
}

List<String> _protonCandidates(String parentDir) {
  final parent = Directory(parentDir);
  if (!parent.existsSync()) return const [];

  try {
    final builds = parent
        .listSync(followLinks: false)
        .whereType<Directory>()
        .where((dir) => p.basename(dir.path).toLowerCase().contains('proton'))
        .map((dir) => (path: p.join(dir.path, 'proton'), dir: dir))
        .where((build) => File(build.path).existsSync())
        .toList()
      ..sort((a, b) => b.dir
          .statSync()
          .modified
          .compareTo(a.dir.statSync().modified));

    return builds.map((build) => build.path).toList();
  } catch (_) {
    return const [];
  }
}
