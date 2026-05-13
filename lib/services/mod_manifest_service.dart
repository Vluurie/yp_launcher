import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/toml_service.dart';

class ModManifestService {
  ModManifestService._();

  static const _manifestFileName = 'mod.toml';

  static ManifestInfo? loadSync(String contentRoot) {
    final file = File(path.join(contentRoot, _manifestFileName));
    if (!file.existsSync()) return null;
    final raw = file.readAsStringSync();
    if (raw.trim().isEmpty) return null;

    final parsed = TomlService.parse(raw);
    final nested = parsed['mod'];
    final Map<String, dynamic> section =
        nested is Map<String, dynamic> ? nested : parsed;

    return ManifestInfo(
      id: _string(section['id']),
      displayName: _string(section['display_name']),
      author: _string(section['author']),
      version: _string(section['version']),
      namsMinVersion: _string(section['nams_min_version']),
      requires: _stringList(section['requires']),
      requiresPlugins: _stringList(section['requires_plugins']),
      replaces: _stringList(section['replaces']),
    );
  }

  static String? _string(dynamic v) {
    if (v is String) {
      final t = v.trim();
      return t.isEmpty ? null : t;
    }
    return null;
  }

  static List<String> _stringList(dynamic v) {
    if (v is List) {
      return v.whereType<String>().map((s) => s.trim()).where((s) => s.isNotEmpty).toList();
    }
    return const [];
  }
}
