import 'dart:convert';
import 'dart:io';

import 'package:yp_launcher/services/launcher_setup_service.dart';

class NamsTexturePack {
  final String name;
  final String source;
  final String? mod;
  final String? character;
  final bool outfitConditional;
  final bool enabled;
  final int ddsCount;
  final int? loadOrderIndex;
  final String path;

  const NamsTexturePack({
    required this.name,
    required this.source,
    required this.outfitConditional,
    required this.enabled,
    required this.ddsCount,
    required this.path,
    this.mod,
    this.character,
    this.loadOrderIndex,
  });

  factory NamsTexturePack.fromJson(Map<String, dynamic> json) {
    return NamsTexturePack(
      name: json['name'] as String? ?? '',
      source: json['source'] as String? ?? 'standalone',
      mod: json['mod'] as String?,
      character: json['character'] as String?,
      outfitConditional: json['outfit_conditional'] as bool? ?? false,
      enabled: json['enabled'] as bool? ?? true,
      ddsCount: (json['dds_count'] as num?)?.toInt() ?? 0,
      loadOrderIndex: (json['load_order_index'] as num?)?.toInt(),
      path: json['path'] as String? ?? '',
    );
  }
}

class NamsCliService {
  NamsCliService._();

  static Future<List<NamsTexturePack>?> texturesList(String gameDir) async {
    try {
      final paths = await LauncherSetupService.getLauncherPaths();
      final namsExe = paths['namsExe'];
      final launcherDir = paths['launcherDir'];
      if (namsExe == null || !File(namsExe).existsSync()) return null;

      final result = await Process.run(
        namsExe,
        ['textures', 'list', '--nier-path', gameDir, '--json'],
        workingDirectory: launcherDir,
      );
      if (result.exitCode != 0) return null;
      final decoded = jsonDecode(result.stdout as String);
      if (decoded is! Map || decoded['packs'] is! List) return null;
      return (decoded['packs'] as List)
          .whereType<Map>()
          .map((m) => NamsTexturePack.fromJson(Map<String, dynamic>.from(m)))
          .toList();
    } catch (_) {
      return null;
    }
  }
}
