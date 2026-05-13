import 'dart:io';
import 'dart:typed_data';
import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/models/config_fields.dart';

enum ReShadeStatus { notFound, detected, incompatibleAddon }

enum ExeVariant { missing, original, wolfLimitBreak, unknown }

class DetectionService {
  DetectionService._();

  static final _iniToTomlMap = <String, String>{
    'LODMultiplier': LodModFields.lodMultiplier.key,
    'AOMultiplierWidth': LodModFields.aoMultiplierWidth.key,
    'AOMultiplierHeight': LodModFields.aoMultiplierHeight.key,
    'ShadowResolution': LodModFields.shadowResolution.key,
    'ShadowDistanceMultiplier': LodModFields.shadowDistanceMultiplier.key,
    'ShadowDistanceMinimum': LodModFields.shadowDistanceMinimum.key,
    'ShadowDistanceMaximum': LodModFields.shadowDistanceMaximum.key,
    'ShadowDistancePSS': LodModFields.shadowDistancePss.key,
    'ShadowFilterStrengthBias': LodModFields.shadowFilterStrengthBias.key,
    'ShadowFilterStrengthMinimum': LodModFields.shadowFilterStrengthMinimum.key,
    'ShadowFilterStrengthMaximum': LodModFields.shadowFilterStrengthMaximum.key,
    'ShadowModelHQ': LodModFields.shadowModelHq.key,
    'ShadowModelForceAll': LodModFields.shadowModelForceAll.key,
    'DisableManualCulling': LodModFields.disableManualCulling.key,
    'DisableVignette': LodModFields.disableVignette.key,
  };

  static Future<Map<String, dynamic>?> detectLegacyLodMod(
    String gameDir,
  ) async {
    final iniPath = path.join(gameDir, 'LodMod.ini');
    {
      final file = File(iniPath);
      if (await file.exists()) {
        final content = await file.readAsString();
        final parsed = _parseIniFile(content);
        if (parsed.isNotEmpty) return parsed;
      }
    }
    return null;
  }

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

  static Future<List<String>> detectHDTextures(String gameDir) async {
    final found = <String>[];

    final skRes = Directory(path.join(gameDir, 'SK_Res'));
    if (await skRes.exists() && await _isNonEmpty(skRes)) {
      found.add('SK_Res/');
    }

    return found;
  }

  static Future<bool> migrateLodModIni(
    String gameDir,
    Map<String, dynamic> iniValues,
  ) async {
    final tomlPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final rawContent = await TomlService.readTomlFile(tomlPath);
    if (rawContent.isEmpty) return false;

    final currentValues = TomlService.parse(rawContent);
    if (currentValues[LodModFields.enabled.key] == true) return false;

    final updates = <String, dynamic>{LodModFields.enabled.key: true};
    for (final entry in iniValues.entries) {
      final tomlKey = _iniToTomlMap[entry.key];
      if (tomlKey != null) {
        updates[tomlKey] = entry.value;
      }
    }

    final updatedContent = TomlService.updateToml(rawContent, updates);
    await TomlService.writeTomlFile(tomlPath, updatedContent);
    return true;
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

  static Map<String, dynamic> _parseIniFile(String content) {
    final result = <String, dynamic>{};

    for (final line in content.split('\n')) {
      final trimmed = line.trim();
      if (trimmed.isEmpty ||
          trimmed.startsWith(';') ||
          trimmed.startsWith('[')) {
        continue;
      }

      final eqIndex = trimmed.indexOf('=');
      if (eqIndex == -1) continue;

      final key = trimmed.substring(0, eqIndex).trim();
      var value = trimmed.substring(eqIndex + 1).trim();

      final commentIdx = value.indexOf(';');
      if (commentIdx != -1) {
        value = value.substring(0, commentIdx).trim();
      }

      if (!_iniToTomlMap.containsKey(key)) continue;

      result[key] = _parseIniValue(value);
    }

    return result;
  }

  static dynamic _parseIniValue(String value) {
    final lower = value.toLowerCase();
    if (lower == 'true') return true;
    if (lower == 'false') return false;
    final intVal = int.tryParse(value);
    if (intVal != null && !value.contains('.')) return intVal;
    final doubleVal = double.tryParse(value);
    if (doubleVal != null) return doubleVal;
    return value;
  }

  static Future<bool> _isNonEmpty(Directory dir) async {
    await for (final _ in dir.list(recursive: false)) {
      return true;
    }
    return false;
  }

  static const String wolfLimitBreakSha256 =
      'c9904a39e448d6cb3c98c28a90159cf9314753b0cfea5ceb4e76a12d3308a355';

  /// Checks if DLC `data100.cpk` is present and ~928 MB. Steam's
  /// "3C3C-Concert & Costume Pack" DLC ships exactly this file at this
  /// size (with a small tolerance). Other unrelated mods can ship a
  /// `data100.cpk` of a different size, hence the size guard.
  static Future<bool> hasDlc(String gameDir) async {
    final file = File(path.join(gameDir, 'data', 'data100.cpk'));
    if (!await file.exists()) return false;
    final size = await file.length();
    const lower = (928 - 5) * 1024 * 1024;
    const upper = 937 * 1024 * 1024;
    return size >= lower && size < upper;
  }

  static Future<ExeVariant> detectExeVariant(String gameDir) async {
    final exePath = path.join(gameDir, AppStrings.gameExeName);
    if (!await File(exePath).exists()) return ExeVariant.missing;
    final hash = await IsolateService.run(_hashFileSync, exePath);
    if (hash == null) return ExeVariant.unknown;
    if (hash == wolfLimitBreakSha256) return ExeVariant.wolfLimitBreak;
    return ExeVariant.original;
  }
}

String? _hashFileSync(String filePath) {
  try {
    final bytes = File(filePath).readAsBytesSync();
    return sha256.convert(bytes).toString();
  } catch (_) {
    return null;
  }
}
