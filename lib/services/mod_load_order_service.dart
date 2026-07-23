import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/toml_service.dart';


class ModLoadOrderService {
  ModLoadOrderService._();

  static String _filePath(String gameDir) =>
      path.join(gameDir, 'nams', 'mod_load_order.toml');

  static const _defaultFile = '''# Decides which mod wins when two mods replace the same game file.
# Mods not listed here keep no defined order between each other.
# Restart the game after editing.
#
# Later entries override earlier ones, so the LAST entry wins a conflict.
#
# Paths are relative to the nams/ folder. Prefix matching is used.

load_order = []
''';

  static String normalize(String relPath) =>
      relPath.replaceAll('\\', '/').trim().replaceAll(RegExp(r'^/+|/+$'), '');

  static Future<List<String>> list(String gameDir) =>
      IsolateService.run(_listSync, gameDir);

  static Future<void> save(String gameDir, List<String> order) =>
      IsolateService.run(
        _saveSync,
        _SaveParams(
          gameDir: gameDir,
          order: order.map(normalize).where((s) => s.isNotEmpty).toList(),
        ),
      );
}

class _SaveParams {
  final String gameDir;
  final List<String> order;
  const _SaveParams({required this.gameDir, required this.order});
}

List<String> _listSync(String gameDir) {
  final file = File(ModLoadOrderService._filePath(gameDir));
  if (!file.existsSync()) return const [];
  return _parse(file.readAsStringSync());
}

List<String> _parse(String raw) {
  final parsed = TomlService.parse(raw);
  final value = parsed['load_order'];
  if (value is! List) return [];
  return value
      .whereType<String>()
      .map(ModLoadOrderService.normalize)
      .where((s) => s.isNotEmpty)
      .toList();
}

void _saveSync(_SaveParams p) {
  final file = File(ModLoadOrderService._filePath(p.gameDir));
  file.parent.createSync(recursive: true);
  final raw = file.existsSync()
      ? file.readAsStringSync()
      : ModLoadOrderService._defaultFile;

  file.writeAsStringSync(
    TomlService.updateToml(raw, {'load_order': p.order}),
  );
}
