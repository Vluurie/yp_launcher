import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/native_steam.dart';
import 'package:yp_launcher/services/wine/proton.dart';
import 'package:yp_launcher/services/wine/steam_vdf.dart';

const _defaultToolKey = '0';

const _systemCompatToolDirs = [
  '/usr/share/steam/compatibilitytools.d',
  '/usr/local/share/steam/compatibilitytools.d',
];

String? findSteamConfiguredProton(String steamRoot, String libraryRoot) {
  final Map<dynamic, dynamic> config;
  try {
    config = parseVdf(
      File(p.join(steamRoot, 'config', 'config.vdf')).readAsStringSync(),
    );
  } catch (_) {
    return null;
  }
  for (final appId in const [nierSteamAppId, _defaultToolKey]) {
    final name = steamCompatToolName(config, appId);
    if (name == null) continue;
    final proton = resolveCompatTool(
      name,
      steamRoot: steamRoot,
      libraryRoots: [libraryRoot],
    );
    if (proton != null) return proton;
  }
  return null;
}

String? steamCompatToolName(Map<dynamic, dynamic> config, String appId) {
  final mapping = vdfFindBlock(config, 'CompatToolMapping');
  final entry = mapping == null ? null : vdfBlock(mapping, appId);
  final name = entry == null ? null : vdfString(entry, 'name');
  return name == null || name.isEmpty ? null : name;
}

String? resolveCompatTool(
  String name, {
  required String steamRoot,
  Iterable<String> libraryRoots = const [],
}) {
  final wanted = name.toLowerCase();
  for (final toolsDir in _compatToolDirs(steamRoot)) {
    final proton = _customTool(toolsDir, wanted);
    if (proton != null) return proton;
  }
  final libraries = {steamRoot, ...libraryRoots, ...nativeSteamLibraries()};
  for (final library in libraries) {
    final proton = _officialTool(
      p.join(library, 'steamapps', 'common'),
      wanted,
    );
    if (proton != null) return proton;
  }
  return null;
}

String? officialProtonToolName(String dirName) {
  final lower = dirName.toLowerCase();
  if (!lower.startsWith('proton')) return null;
  if (lower.contains('experimental')) return 'proton_experimental';
  if (lower.contains('hotfix')) return 'proton_hotfix';
  for (final word in lower.split(' ')) {
    final parts = word.split('.');
    if (parts.length != 2) continue;
    final major = int.tryParse(parts[0]);
    final minor = int.tryParse(parts[1]);
    if (major == null || minor == null) continue;
    return minor == 0 ? 'proton_$major' : 'proton_${parts[0]}${parts[1]}';
  }
  return null;
}

List<String> _compatToolDirs(String steamRoot) => [
  p.join(steamRoot, 'compatibilitytools.d'),
  ...?Platform.environment['STEAM_EXTRA_COMPAT_TOOLS_PATHS']
      ?.split(':')
      .where((dir) => dir.isNotEmpty),
  ..._systemCompatToolDirs,
];

String? _customTool(String toolsDir, String wanted) {
  for (final dir in _subdirs(toolsDir)) {
    final Map<dynamic, dynamic> manifest;
    try {
      manifest = parseVdf(
        File(p.join(dir.path, 'compatibilitytool.vdf')).readAsStringSync(),
      );
    } catch (_) {
      continue;
    }
    final tools = vdfFindBlock(manifest, 'compat_tools');
    if (tools == null) continue;
    for (final entry in tools.entries) {
      final tool = entry.value;
      if ('${entry.key}'.toLowerCase() != wanted || tool is! Map) continue;
      final installPath = vdfString(tool, 'install_path') ?? '.';
      final proton = p.normalize(p.join(dir.path, installPath, 'proton'));
      if (File(proton).existsSync()) return proton;
    }
  }
  return null;
}

String? _officialTool(String commonDir, String wanted) {
  for (final dir in _subdirs(commonDir)) {
    if (officialProtonToolName(p.basename(dir.path)) != wanted) continue;
    final proton = p.join(dir.path, 'proton');
    if (File(proton).existsSync()) return proton;
  }
  return null;
}

List<Directory> _subdirs(String path) {
  try {
    return Directory(path).listSync().whereType<Directory>().toList();
  } catch (_) {
    return const [];
  }
}
