import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/services/detection/graphics_dll_id.dart';
import 'package:yp_launcher/models/config_fields.dart';

enum ReShadeStatus { notFound, detected, incompatibleAddon }

class ReShadeDetection {
  ReShadeDetection._();

  /// Detects a game-root ReShade install by byte-identifying any DLL under a
  /// name ReShade commonly ships as (regardless of the packaged filename). Uses
  /// [GraphicsDllId] robust string markers (reshade.me / crosire), which match
  /// old builds (4.x) through current — the previous long-string byte signature
  /// missed every compressed build and is no longer used.
  static Future<ReShadeStatus> detectReShade(String gameDir) async {
    const reshadeDllCandidates = [
      'ReShade64.dll',
      'reshade64.dll',
      'ReShade64_Addon.dll',
      'dxgi.dll',
      'd3d11.dll',
    ];

    for (final candidate in reshadeDllCandidates) {
      final filePath = path.join(gameDir, candidate);
      final kind = GraphicsDllId.identifyFile(filePath);
      if (kind == GraphicsDll.reshade) return ReShadeStatus.detected;
    }
    return ReShadeStatus.notFound;
  }

  /// When ReShade is present we default `disable_reshade_loading = true` so
  /// users start with NAMS's native depth-of-field path instead of stacking
  /// ReShade on top of it. Only writes when the key is currently `false`
  /// (the unmodified default), so a user who already toggled it themselves
  /// in the NAMS tab won't be overridden.
  static Future<bool> autoDisableReShadeLoading(String gameDir) async {
    final tomlPath = path.join(gameDir, 'nams', 'nams.toml');
    final rawContent = await TomlService.readTomlFile(tomlPath);
    if (rawContent.isEmpty) return false;

    final currentValues = TomlService.parse(rawContent);
    final current = currentValues[NamsFields.disableReShadeLoading.key];
    if (current == true) return false;

    final updatedContent = TomlService.updateToml(
      rawContent,
      {NamsFields.disableReShadeLoading.key: true},
    );
    await TomlService.writeTomlFile(tomlPath, updatedContent);
    return true;
  }
}
