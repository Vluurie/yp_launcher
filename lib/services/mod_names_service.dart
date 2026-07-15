import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/isolate_service.dart';

class ModNamesService {
  ModNamesService._();

  static String _filePath(String gameDir) =>
      path.join(gameDir, 'nams', 'mod_names.json');

  static Future<Map<String, String>> load(String gameDir) =>
      IsolateService.run(_loadSync, gameDir);

  static Future<void> setName(String gameDir, String modId, String? name) =>
      IsolateService.run(
        _setSync,
        _SetParams(gameDir: gameDir, modId: modId, name: name),
      );
}

class _SetParams {
  final String gameDir;
  final String modId;
  final String? name;
  const _SetParams({
    required this.gameDir,
    required this.modId,
    required this.name,
  });
}

Map<String, String> _loadSync(String gameDir) {
  if (gameDir.isEmpty) return const {};
  try {
    final file = File(ModNamesService._filePath(gameDir));
    if (!file.existsSync()) return const {};
    final decoded = jsonDecode(file.readAsStringSync());
    if (decoded is! Map) return const {};
    final out = <String, String>{};
    for (final entry in decoded.entries) {
      final value = entry.value;
      if (value is String && value.trim().isNotEmpty) {
        out[entry.key.toString()] = value;
      }
    }
    return out;
  } catch (_) {}
  return const {};
}

void _setSync(_SetParams p) {
  if (p.gameDir.isEmpty) return;
  final names = Map<String, String>.from(_loadSync(p.gameDir));
  final trimmed = p.name?.trim();
  if (trimmed == null || trimmed.isEmpty) {
    names.remove(p.modId);
  } else {
    names[p.modId] = trimmed;
  }
  try {
    final file = File(ModNamesService._filePath(p.gameDir));
    file.parent.createSync(recursive: true);
    if (names.isEmpty) {
      if (file.existsSync()) file.deleteSync();
      return;
    }
    file.writeAsStringSync(jsonEncode(names));
  } catch (_) {}
}
