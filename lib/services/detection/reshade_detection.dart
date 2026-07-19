import 'dart:io';
import 'dart:typed_data';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/models/config_fields.dart';

enum ReShadeStatus { notFound, detected, incompatibleAddon }

class ReShadeDetection {
  ReShadeDetection._();

  static Future<ReShadeStatus> detectReShade(String gameDir) async {
    const reshadeDllCandidates = ['ReShade64.dll', 'reshade64.dll', 'dxgi.dll'];
    File? sourceDll;
    bool isNamedReShade = false;

    for (final candidate in reshadeDllCandidates) {
      final file = File(path.join(gameDir, candidate));
      if (await file.exists()) {
        sourceDll = file;
        isNamedReShade = candidate.toLowerCase() == 'reshade64.dll';
        break;
      }
    }

    if (sourceDll == null) return ReShadeStatus.notFound;

    if (!isNamedReShade) {
      final Uint8List bytes;
      try {
        bytes = await sourceDll.readAsBytes();
      } catch (_) {
        return ReShadeStatus.notFound;
      }

      if (!_containsBytes(bytes, _reshadeSignature)) {
        return ReShadeStatus.notFound;
      }

      if (_containsBytes(bytes, _imguiAddonSignature)) {
        return ReShadeStatus.incompatibleAddon;
      }
    }

    return ReShadeStatus.detected;
  }

  static const _reshadeSignature = [
    0x52, 0x65, 0x53, 0x68, 0x61, 0x64, 0x65, 0x20, // "ReShade "
    0x64, 0x65, 0x70, 0x74, 0x68, 0x20, // "depth "
    0x62, 0x61, 0x63, 0x6B, 0x75, 0x70, 0x20, // "backup "
    0x74, 0x65, 0x78, 0x74, 0x75, 0x72, 0x65, // "texture"
  ];

  static const _imguiAddonSignature = [
    0x42, 0x65, 0x67, 0x69, 0x6E, 0x4D, 0x65, 0x6E, 0x75, // "BeginMenu"
    0x42, 0x61, 0x72, 0x40, 0x49, 0x6D, 0x47, 0x75, 0x69, // "Bar@ImGui"
  ];

  static bool _containsBytes(Uint8List haystack, List<int> needle) {
    if (needle.length > haystack.length) return false;
    outer:
    for (var i = 0; i <= haystack.length - needle.length; i++) {
      for (var j = 0; j < needle.length; j++) {
        if (haystack[i + j] != needle[j]) continue outer;
      }
      return true;
    }
    return false;
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
