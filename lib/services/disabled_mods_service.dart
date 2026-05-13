import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/toml_service.dart';


class DisabledModsService {
  DisabledModsService._();

  static String _filePath(String gameDir) =>
      path.join(gameDir, 'nams', 'disabled_mods.toml');

  static const _defaultFile = '''# Mods listed here are completely ignored at boot.
# NAMS does not mount, scan, or load anything inside a disabled path.
# Restart the game after editing.
#
# Paths are relative to the nams/ folder. Prefix matching is used, so
# disabling a folder also disables everything inside it.

disabled = []
''';

  static String normalize(String relPath) =>
      relPath.replaceAll('\\', '/').trim().replaceAll(RegExp(r'^/+|/+$'), '');

  static String? toRelative(String gameDir, String absolutePath) {
    final rel = path.relative(absolutePath, from: path.join(gameDir, 'nams'));
    if (rel.startsWith('..')) return null;
    return normalize(rel);
  }

  static Future<List<String>> list(String gameDir) =>
      IsolateService.run(_listSync, gameDir);

  static Future<void> setDisabled(
    String gameDir,
    String relPath,
    bool disabled,
  ) =>
      IsolateService.run(
        _setSync,
        _SetParams(gameDir: gameDir, relPath: normalize(relPath), disabled: disabled),
      );

  static bool matches(List<String> entries, String relPath) {
    final rel = normalize(relPath);
    for (final e in entries) {
      if (rel == e || rel.startsWith('$e/')) return true;
    }
    return false;
  }
}

class _SetParams {
  final String gameDir;
  final String relPath;
  final bool disabled;
  const _SetParams({
    required this.gameDir,
    required this.relPath,
    required this.disabled,
  });
}

List<String> _listSync(String gameDir) {
  final file = File(DisabledModsService._filePath(gameDir));
  if (!file.existsSync()) return const [];
  return _parse(file.readAsStringSync());
}

List<String> _parse(String raw) {
  final parsed = TomlService.parse(raw);
  final value = parsed['disabled'];
  if (value is! List) return [];
  return value
      .whereType<String>()
      .map(DisabledModsService.normalize)
      .where((s) => s.isNotEmpty)
      .toList();
}

void _setSync(_SetParams p) {
  final file = File(DisabledModsService._filePath(p.gameDir));
  file.parent.createSync(recursive: true);
  final raw = file.existsSync()
      ? file.readAsStringSync()
      : DisabledModsService._defaultFile;

  final entries = _parse(raw);
  if (p.disabled) {
    if (!entries.contains(p.relPath)) entries.add(p.relPath);
  } else {
    entries.remove(p.relPath);
  }

  file.writeAsStringSync(TomlService.updateToml(raw, {'disabled': entries}));
}
