import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as path;

class ShaderFixNames {
  ShaderFixNames._();

  static const fileName = 'nams.shaderfix_names.json';

  static File _file(String migotoDir) =>
      File(path.join(migotoDir, fileName));

  static Map<String, String> read(String migotoDir) {
    final f = _file(migotoDir);
    if (!f.existsSync()) return {};
    try {
      final decoded = jsonDecode(f.readAsStringSync());
      if (decoded is Map) {
        return decoded.map((k, v) => MapEntry('$k', '$v'));
      }
    } catch (_) {}
    return {};
  }

  static void assign(
    String migotoDir,
    Iterable<String> shaderFixFileNames,
    String modName,
  ) {
    final map = read(migotoDir);
    var changed = false;
    for (final name in shaderFixFileNames) {
      if (map.containsKey(name)) continue;
      map[name] = modName;
      changed = true;
    }
    if (changed) _write(migotoDir, map);
  }

  static void prune(String migotoDir, Set<String> existingFileNames) {
    final map = read(migotoDir);
    final before = map.length;
    map.removeWhere((k, _) => !existingFileNames.contains(k));
    if (map.length != before) _write(migotoDir, map);
  }

  static void _write(String migotoDir, Map<String, String> map) {
    try {
      _file(migotoDir).writeAsStringSync(
        const JsonEncoder.withIndent('  ').convert(map),
      );
    } catch (_) {}
  }
}
