import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/mod_group.dart';

class ModGroupsService {
  ModGroupsService._();

  static const _fileName = 'mod_groups.json';

  static String _filePath(String gameDir) =>
      path.join(gameDir, 'nams', _fileName);

  static Future<ModGroupsData> load(String gameDir) async {
    if (gameDir.isEmpty) return const ModGroupsData();
    try {
      final file = File(_filePath(gameDir));
      if (!await file.exists()) return const ModGroupsData();
      final raw = await file.readAsString();
      if (raw.trim().isEmpty) return const ModGroupsData();
      final decoded = jsonDecode(raw);
      if (decoded is! Map) return const ModGroupsData();
      return ModGroupsData.fromJson(Map<String, dynamic>.from(decoded));
    } catch (_) {
      return const ModGroupsData();
    }
  }

  static Future<void> save(String gameDir, ModGroupsData data) async {
    if (gameDir.isEmpty) return;
    try {
      final namsDir = Directory(path.join(gameDir, 'nams'));
      if (!await namsDir.exists()) {
        await namsDir.create(recursive: true);
      }
      final file = File(_filePath(gameDir));
      await file.writeAsString(jsonEncode(data.toJson()));
    } catch (_) {}
  }
}
