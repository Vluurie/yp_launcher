import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/toml_service.dart';

class DetectionService {
  DetectionService._();

  // INI key → TOML key mapping
  static const _iniToTomlMap = <String, String>{
    'LODMultiplier': 'lod_multiplier',
    'AOMultiplierWidth': 'ao_multiplier_width',
    'AOMultiplierHeight': 'ao_multiplier_height',
    'ShadowResolution': 'shadow_resolution',
    'ShadowDistanceMultiplier': 'shadow_distance_multiplier',
    'ShadowDistanceMinimum': 'shadow_distance_minimum',
    'ShadowDistanceMaximum': 'shadow_distance_maximum',
    'ShadowDistancePSS': 'shadow_distance_pss',
    'ShadowFilterStrengthBias': 'shadow_filter_strength_bias',
    'ShadowFilterStrengthMinimum': 'shadow_filter_strength_minimum',
    'ShadowFilterStrengthMaximum': 'shadow_filter_strength_maximum',
    'ShadowModelHQ': 'shadow_model_hq',
    'ShadowModelForceAll': 'shadow_model_force_all',
    'DisableManualCulling': 'disable_manual_culling',
    'DisableVignette': 'disable_vignette',
    'DisableFakeHDR': 'disable_fake_hdr',
  };

  /// Searches for a legacy LodMod.ini and returns its parsed key-value pairs,
  /// or null if not found. Only returns values for keys we can map.
  static Future<Map<String, dynamic>?> detectLegacyLodMod(String gameDir) async {
    final searchPaths = [
      path.join(gameDir, 'LodMod.ini'),
      _documentsLodModPath(),
    ];

    for (final iniPath in searchPaths) {
      if (iniPath == null) continue;
      final file = File(iniPath);
      if (await file.exists()) {
        final content = await file.readAsString();
        final parsed = _parseIniFile(content);
        if (parsed.isNotEmpty) return parsed;
      }
    }
    return null;
  }

  /// Returns true if ReShade files are detected next to the game exe.
  static Future<bool> detectReShade(String gameDir) async {
    const reshadeFiles = ['dxgi.dll', 'd3d11.dll', 'ReShade.ini'];
    for (final name in reshadeFiles) {
      if (await File(path.join(gameDir, name)).exists()) return true;
    }
    return false;
  }

  static Future<List<String>> detectHDTextures(String gameDir) async {
    final found = <String>[];

    final skRes = Directory(path.join(gameDir, 'SK_Res'));
    if (await skRes.exists() && await _isNonEmpty(skRes)) {
      found.add('SK_Res/');
    }

    final waxMods = Directory(path.join(gameDir, 'wax', 'mods'));
    if (await waxMods.exists() && await _isNonEmpty(waxMods)) {
      found.add('wax/mods/');
    }

    return found;
  }

  /// Migrates INI values into the existing lodmod.toml.
  /// Only runs if lodmod.toml currently has enabled = false.
  /// Returns true if migration was performed.
  static Future<bool> migrateLodModIni(
    String gameDir,
    Map<String, dynamic> iniValues,
  ) async {
    final tomlPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final rawContent = await TomlService.readTomlFile(tomlPath);
    if (rawContent.isEmpty) return false;

    final currentValues = TomlService.parse(rawContent);
    if (currentValues['enabled'] == true) return false;

    // Build the update map from INI values + enable
    final updates = <String, dynamic>{'enabled': true};
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

  /// Parses a Windows INI file, returning a flat map of key → value.
  /// Handles [Section] headers, ; comments, and inline comments.
  static Map<String, dynamic> _parseIniFile(String content) {
    final result = <String, dynamic>{};

    for (final line in content.split('\n')) {
      final trimmed = line.trim();
      if (trimmed.isEmpty || trimmed.startsWith(';') || trimmed.startsWith('[')) {
        continue;
      }

      final eqIndex = trimmed.indexOf('=');
      if (eqIndex == -1) continue;

      final key = trimmed.substring(0, eqIndex).trim();
      var value = trimmed.substring(eqIndex + 1).trim();

      // Strip inline comments
      final commentIdx = value.indexOf(';');
      if (commentIdx != -1) {
        value = value.substring(0, commentIdx).trim();
      }

      // Only keep keys we can map
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

  static String? _documentsLodModPath() {
    final userProfile = Platform.environment['USERPROFILE'];
    if (userProfile == null) return null;
    return path.join(
      userProfile,
      'Documents',
      'My Games',
      'NieR_Automata',
      'LodMod.ini',
    );
  }

  static Future<bool> _isNonEmpty(Directory dir) async {
    await for (final _ in dir.list(recursive: false)) {
      return true;
    }
    return false;
  }
}
