import 'dart:io';

import 'package:path/path.dart' as p;

/// Wine prefixes only exist on POSIX hosts, so prefix paths are always treated
/// as POSIX regardless of the host. Keeps this logic testable on Windows.
final _posix = p.posix;

const _driveC = 'drive_c';

/// Environment passed to every wine invocation.
Map<String, String> createWineEnv() => {
      ...Platform.environment,
      'WINEDEBUG': Platform.environment['WINEDEBUG'] ?? '-all',
    };

/// The prefix root containing [targetPath], or null when it is not inside one.
/// The `drive_c` component is the pivot.
String? inferWinePrefixFromPath(String targetPath) {
  final parts = _posix.normalize(targetPath).split('/');
  final index = _driveCIndex(parts);
  if (index <= 0) return null;
  final prefix = parts.sublist(0, index).join('/');
  return prefix.isEmpty ? '/' : prefix;
}

bool isCrossOverPrefix(String prefix) {
  final normalized = prefix.toLowerCase();
  return normalized.contains('/crossover/bottles/') ||
      normalized.contains('/crossover preview/bottles/') ||
      normalized.contains('/.cxoffice/');
}

/// Translates a host path to the Windows path Wine exposes: `C:\` when inside a
/// prefix, otherwise `Z:\` (Wine maps Z: to the host root).
String toWinePath(String targetPath) {
  final normalized = _posix.normalize(targetPath);
  final parts = normalized.split('/');
  final index = _driveCIndex(parts);
  if (index >= 0) {
    return 'C:\\${parts.sublist(index + 1).join('\\')}';
  }
  return 'Z:${normalized.replaceAll('/', '\\')}';
}

String getWineRoamingPath(String winePrefix) {
  final usersPath = _posix.join(winePrefix, _driveC, 'users');
  return _posix.join(
    usersPath,
    resolveWineUserName(usersPath),
    'AppData',
    'Roaming',
  );
}

/// Picks the prefix user directory, preferring the host user and the names
/// CrossOver and Proton conventionally create.
String resolveWineUserName(String usersPath) {
  final env = Platform.environment;
  final preferred = <String>[
    if (env['USER'] != null) env['USER']!,
    if (env['USERNAME'] != null) env['USERNAME']!,
    if (env['LOGNAME'] != null) env['LOGNAME']!,
    'steamuser',
    'crossover',
  ];

  final existing = _listDirNames(usersPath)
      .where((name) => name.toLowerCase() != 'public')
      .toList();

  for (final candidate in preferred) {
    for (final name in existing) {
      if (name.toLowerCase() == candidate.toLowerCase()) return name;
    }
  }

  if (existing.isNotEmpty) return existing.first;
  return preferred.first;
}

int _driveCIndex(List<String> parts) =>
    parts.indexWhere((part) => part.toLowerCase() == _driveC);

List<String> _listDirNames(String dirPath) {
  try {
    return Directory(dirPath)
        .listSync(followLinks: false)
        .whereType<Directory>()
        .map((entity) => _posix.basename(entity.path))
        .toList();
  } catch (_) {
    return const [];
  }
}
