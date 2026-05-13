import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as path;
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/cutscene_detection_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

part 'config_state.g.dart';

final configPanelOpenProvider = StateProvider<bool>((ref) => false);

class ConfigData {
  final Map<String, dynamic> namsValues;
  final Map<String, dynamic> lodmodValues;
  final Map<String, dynamic> textureInjectionValues;
  final String namsRawContent;
  final String lodmodRawContent;
  final String textureInjectionRawContent;
  final bool isLoading;
  final bool hasUnsavedChanges;

  const ConfigData({
    this.namsValues = const {},
    this.lodmodValues = const {},
    this.textureInjectionValues = const {},
    this.namsRawContent = '',
    this.lodmodRawContent = '',
    this.textureInjectionRawContent = '',
    this.isLoading = false,
    this.hasUnsavedChanges = false,
  });

  ConfigData copyWith({
    Map<String, dynamic>? namsValues,
    Map<String, dynamic>? lodmodValues,
    Map<String, dynamic>? textureInjectionValues,
    String? namsRawContent,
    String? lodmodRawContent,
    String? textureInjectionRawContent,
    bool? isLoading,
    bool? hasUnsavedChanges,
  }) {
    return ConfigData(
      namsValues: namsValues ?? this.namsValues,
      lodmodValues: lodmodValues ?? this.lodmodValues,
      textureInjectionValues:
          textureInjectionValues ?? this.textureInjectionValues,
      namsRawContent: namsRawContent ?? this.namsRawContent,
      lodmodRawContent: lodmodRawContent ?? this.lodmodRawContent,
      textureInjectionRawContent:
          textureInjectionRawContent ?? this.textureInjectionRawContent,
      isLoading: isLoading ?? this.isLoading,
      hasUnsavedChanges: hasUnsavedChanges ?? this.hasUnsavedChanges,
    );
  }
}

@Riverpod(keepAlive: true)
class ConfigStateController extends _$ConfigStateController {
  @override
  ConfigData build() {
    return const ConfigData();
  }

  Future<void> loadConfigs(String gameDir) async {
    state = state.copyWith(isLoading: true);

    await NamsConfigService.ensureConfigs(gameDir);

    final namsPath = path.join(gameDir, 'nams', 'nams.toml');
    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final texturePath = path.join(gameDir, 'nams', 'texture_injection.toml');

    final namsRaw = await TomlService.readTomlFile(namsPath);
    var lodmodRaw = await TomlService.readTomlFile(lodmodPath);
    final textureRaw = await TomlService.readTomlFile(texturePath);

    final migrated = _migrateLodmod(lodmodRaw);
    if (migrated != lodmodRaw) {
      lodmodRaw = migrated;
      await TomlService.writeTomlFile(lodmodPath, lodmodRaw);
    }

    state = ConfigData(
      namsValues: TomlService.parse(namsRaw),
      lodmodValues: TomlService.parse(lodmodRaw),
      textureInjectionValues: TomlService.parse(textureRaw),
      namsRawContent: namsRaw,
      lodmodRawContent: lodmodRaw,
      textureInjectionRawContent: textureRaw,
    );

    autoDetectCutscenes(gameDir);
  }

  Future<void> autoDetectCutscenes(String gameDir) async {
    final result = await CutsceneDetectionService.scan(gameDir);

    await NamsConfigService.ensureConfigs(gameDir);

    final namsPath = path.join(gameDir, 'nams', 'nams.toml');
    final raw = await TomlService.readTomlFile(namsPath);
    final parsed = TomlService.parse(raw);
    final cutscene =
        (parsed['cutscene'] as Map<String, dynamic>?) ?? <String, dynamic>{};

    final desiredHd = result.hasHdCutscenes;
    final desiredH264 = result.needsH264;

    final updates = <String, dynamic>{};
    if (cutscene['hd_cutscenes'] != desiredHd) {
      updates['hd_cutscenes'] = desiredHd;
    }
    if (cutscene['enable_h264'] != desiredH264) {
      updates['enable_h264'] = desiredH264;
    }
    if (updates.isEmpty) return;

    final mergedCutscene = Map<String, dynamic>.from(cutscene)..addAll(updates);
    final mergedAll = Map<String, dynamic>.from(parsed)
      ..['cutscene'] = mergedCutscene;
    final newRaw = TomlService.updateToml(raw, mergedAll);
    await TomlService.writeTomlFile(namsPath, newRaw);

    final memCutscene = Map<String, dynamic>.from(
      (state.namsValues['cutscene'] as Map<String, dynamic>?) ?? const {},
    )..addAll(updates);
    final memNams = Map<String, dynamic>.from(state.namsValues)
      ..['cutscene'] = memCutscene;
    state = state.copyWith(
      namsValues: memNams,
      namsRawContent: newRaw,
    );
  }

  void updateNams(String key, dynamic value, {String? section}) {
    final updated = Map<String, dynamic>.from(state.namsValues);
    if (section != null) {
      final sectionMap = Map<String, dynamic>.from(
        (updated[section] as Map<String, dynamic>?) ?? {},
      );
      sectionMap[key] = value;
      updated[section] = sectionMap;
    } else {
      updated[key] = value;
    }
    state = state.copyWith(namsValues: updated, hasUnsavedChanges: true);
  }

  void updateLodmod(String key, dynamic value) {
    final updated = Map<String, dynamic>.from(state.lodmodValues);
    updated[key] = value;
    state = state.copyWith(lodmodValues: updated, hasUnsavedChanges: true);
  }

  String _migrateLodmod(String raw) {
    final parsed = TomlService.parse(raw);
    final missing = _lodmodFields.where((f) => !parsed.containsKey(f.key));
    if (missing.isEmpty) return raw;

    final buf = StringBuffer(raw);
    if (!raw.endsWith('\n')) buf.write('\n');
    for (final field in missing) {
      buf.writeln();
      buf.writeln('${field.key} = ${_formatTomlDefault(field.defaultValue)}');
    }
    return buf.toString();
  }

  static String _formatTomlDefault(dynamic value) {
    if (value is bool) return value.toString();
    if (value is int) return value.toString();
    if (value is double) {
      final s = value.toString();
      return s.contains('.') ? s : '$s.0';
    }
    return '"$value"';
  }

  static final List<ConfigField<dynamic>> _lodmodFields = [
    LodModFields.enabled,
    LodModFields.lodMultiplier,
    LodModFields.disableManualCulling,
    LodModFields.aoMultiplierWidth,
    LodModFields.aoMultiplierHeight,
    LodModFields.disableVignette,
    LodModFields.shadowResolution,
    LodModFields.shadowDistanceMultiplier,
    LodModFields.shadowDistanceMinimum,
    LodModFields.shadowDistanceMaximum,
    LodModFields.shadowDistancePss,
    LodModFields.shadowFilterStrengthBias,
    LodModFields.shadowFilterStrengthMinimum,
    LodModFields.shadowFilterStrengthMaximum,
    LodModFields.shadowModelHq,
    LodModFields.shadowModelForceAll,
    LodModFields.giEnabled,
    LodModFields.giWorkgroupSize,
    LodModFields.giMinLightExtent,
    LodModFields.fpsUncapInMenus,
    LodModFields.fpsUncapInGameplay,
  ];

  Timer? _lodmodAutosaveTimer;
  static const _lodmodAutosaveDelay = Duration(milliseconds: 250);

  void updateLodmodLive(String gameDir, String key, dynamic value) {
    final updated = Map<String, dynamic>.from(state.lodmodValues);
    updated[key] = value;
    state = state.copyWith(lodmodValues: updated);

    _lodmodAutosaveTimer?.cancel();
    _lodmodAutosaveTimer = Timer(_lodmodAutosaveDelay, () {
      _flushLodmod(gameDir);
    });
  }

  Future<void> resetLodmodToDefaults(String gameDir) async {
    _lodmodAutosaveTimer?.cancel();
    final defaults = <String, dynamic>{
      for (final f in _lodmodFields) f.key: f.defaultValue,
    };
    state = state.copyWith(lodmodValues: defaults);
    await _flushLodmod(gameDir);
  }

  Future<void> _flushLodmod(String gameDir) async {
    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final newRaw = TomlService.updateToml(
      state.lodmodRawContent,
      state.lodmodValues,
    );
    if (newRaw == state.lodmodRawContent) return;
    await TomlService.writeTomlFile(lodmodPath, newRaw);
    state = state.copyWith(lodmodRawContent: newRaw);
  }

  void updateTextureInjection(String key, dynamic value) {
    final updated = Map<String, dynamic>.from(state.textureInjectionValues);
    updated[key] = value;
    state = state.copyWith(
      textureInjectionValues: updated,
      hasUnsavedChanges: true,
    );
  }

  void updateTextureInjectionSilent(String key, dynamic value) {
    final updated = Map<String, dynamic>.from(state.textureInjectionValues);
    updated[key] = value;
    state = state.copyWith(textureInjectionValues: updated);
  }

  Future<void> saveConfigs(String gameDir) async {
    final namsPath = path.join(gameDir, 'nams', 'nams.toml');
    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final texturePath = path.join(gameDir, 'nams', 'texture_injection.toml');

    final updatedNams = TomlService.updateToml(
      state.namsRawContent,
      state.namsValues,
    );
    final updatedLodmod = TomlService.updateToml(
      state.lodmodRawContent,
      state.lodmodValues,
    );
    final updatedTexture = TomlService.updateToml(
      state.textureInjectionRawContent,
      state.textureInjectionValues,
    );

    await TomlService.writeTomlFile(namsPath, updatedNams);
    await TomlService.writeTomlFile(lodmodPath, updatedLodmod);
    await TomlService.writeTomlFile(texturePath, updatedTexture);

    state = state.copyWith(
      namsRawContent: updatedNams,
      lodmodRawContent: updatedLodmod,
      textureInjectionRawContent: updatedTexture,
      hasUnsavedChanges: false,
    );
  }

  Future<void> discardChanges(String gameDir) async {
    await loadConfigs(gameDir);
  }
}
