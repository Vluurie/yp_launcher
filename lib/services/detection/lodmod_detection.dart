import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/models/config_fields.dart';

class LodModDetection {
  LodModDetection._();

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
}
