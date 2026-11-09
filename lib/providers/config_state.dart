import 'dart:async';
import 'dart:io';

import 'package:flutter_riverpod/legacy.dart';
import 'package:path/path.dart' as path;
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/default_mods_state.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/lodmod_config_service.dart';
import 'package:yp_launcher/services/cutscene_detection_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

part 'config_state.g.dart';

final configPanelOpenProvider = StateProvider<bool>((ref) => false);

class ConfigData {
  final Map<String, dynamic> namsValues;
  final Map<String, dynamic> lodmodValues;
  final Map<String, dynamic> textureInjectionValues;
  final String namsRawContent;
  final String naiomRawContent;
  final String lodmodRawContent;
  final String textureInjectionRawContent;
  final bool isLoading;
  final bool hasUnsavedChanges;
  final String? namsError;
  final String? naiomError;
  final String? lodmodError;
  final String? textureInjectionError;

  const ConfigData({
    this.namsValues = const {},
    this.lodmodValues = const {},
    this.textureInjectionValues = const {},
    this.namsRawContent = '',
    this.naiomRawContent = '',
    this.lodmodRawContent = '',
    this.textureInjectionRawContent = '',
    this.isLoading = false,
    this.hasUnsavedChanges = false,
    this.namsError,
    this.naiomError,
    this.lodmodError,
    this.textureInjectionError,
  });

  ConfigData copyWith({
    Map<String, dynamic>? namsValues,
    Map<String, dynamic>? lodmodValues,
    Map<String, dynamic>? textureInjectionValues,
    String? namsRawContent,
    String? naiomRawContent,
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
      naiomRawContent: naiomRawContent ?? this.naiomRawContent,
      lodmodRawContent: lodmodRawContent ?? this.lodmodRawContent,
      textureInjectionRawContent:
          textureInjectionRawContent ?? this.textureInjectionRawContent,
      isLoading: isLoading ?? this.isLoading,
      hasUnsavedChanges: hasUnsavedChanges ?? this.hasUnsavedChanges,
      namsError: namsError,
      naiomError: naiomError,
      lodmodError: lodmodError,
      textureInjectionError: textureInjectionError,
    );
  }
}

@Riverpod(keepAlive: true)
class ConfigStateController extends _$ConfigStateController {
  static const _configFileNames = {
    'nams.toml',
    'naiom.toml',
    'lodmod.toml',
    'texture_injection.toml',
  };

  static const naiomSection = 'mouse';

  StreamSubscription<FileSystemEvent>? _watchSub;
  String? _watchedDir;
  Timer? _reloadDebounce;
  DateTime _lastSelfWrite = DateTime.fromMillisecondsSinceEpoch(0);

  @override
  ConfigData build() {
    ref.onDispose(() {
      _lodmodAutosaveTimer?.cancel();
      _lodmodAutosaveTimer = null;
      _naiomAutosaveTimer?.cancel();
      _naiomAutosaveTimer = null;
      _reloadDebounce?.cancel();
      _reloadDebounce = null;
      _watchSub?.cancel();
      _watchSub = null;
    });
    return const ConfigData();
  }

  void _watchConfigDir(String gameDir) {
    final dirPath = path.join(gameDir, 'nams');
    if (_watchedDir == dirPath && _watchSub != null) return;
    _watchSub?.cancel();
    _watchSub = null;
    _watchedDir = null;

    final dir = Directory(dirPath);
    if (!dir.existsSync()) return;

    try {
      _watchSub = dir.watch().listen((event) {
        if (_configFileNames.contains(path.basename(event.path))) {
          _scheduleExternalReload(gameDir);
        }
      }, onError: (_) {});
      _watchedDir = dirPath;
    } catch (_) {}
  }

  void _scheduleExternalReload(String gameDir) {
    _reloadDebounce?.cancel();
    _reloadDebounce = Timer(const Duration(milliseconds: 400), () {
      if (state.hasUnsavedChanges) return;
      if (DateTime.now().difference(_lastSelfWrite) <
          const Duration(seconds: 2)) {
        return;
      }
      unawaited(loadConfigs(gameDir).catchError((_) {}));
    });
  }

  Future<void> loadConfigs(String gameDir) async {
    await _flushPendingNaiom();
    state = state.copyWith(isLoading: true);

    await NamsConfigService.ensureConfigs(gameDir);

    final namsPath = path.join(gameDir, 'nams', 'nams.toml');
    final naiomPath = path.join(gameDir, 'nams', 'naiom.toml');
    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final texturePath = path.join(gameDir, 'nams', 'texture_injection.toml');

    final namsRaw = await TomlService.readTomlFile(namsPath);
    final naiomRaw = await TomlService.readTomlFile(naiomPath);
    final lodmodRaw = await TomlService.readTomlFile(lodmodPath);
    final textureRaw = await TomlService.readTomlFile(texturePath);

    final nams = TomlService.parseStrict(namsRaw);
    final naiom = TomlService.parseStrict(naiomRaw);
    final lodmod = TomlService.parseStrict(lodmodRaw);
    final texture = TomlService.parseStrict(textureRaw);

    state = ConfigData(
      namsValues: {...nams.values, naiomSection: naiom.values},
      lodmodValues: lodmod.values,
      textureInjectionValues: texture.values,
      namsRawContent: namsRaw,
      naiomRawContent: naiomRaw,
      lodmodRawContent: lodmodRaw,
      textureInjectionRawContent: textureRaw,
      namsError: nams.error,
      naiomError: naiom.error,
      lodmodError: lodmod.error,
      textureInjectionError: texture.error,
    );

    _watchConfigDir(gameDir);

    unawaited(autoDetectCutscenes(gameDir).catchError((_) {}));
  }

  Future<void> autoDetectCutscenes(String gameDir) async {
    final result = await CutsceneDetectionService.scan(gameDir);

    await NamsConfigService.ensureConfigs(gameDir);

    final namsPath = path.join(gameDir, 'nams', 'nams.toml');
    final raw = await TomlService.readTomlFile(namsPath);
    final strict = TomlService.parseStrict(raw);
    if (!strict.ok) return;
    final parsed = strict.values;
    final rawCutscene = parsed['cutscene'];
    final cutscene = rawCutscene is Map<String, dynamic>
        ? rawCutscene
        : <String, dynamic>{};

    final desiredH264 = result.needsH264;

    final updates = <String, dynamic>{};
    if (cutscene['enable_h264'] != desiredH264) {
      updates['enable_h264'] = desiredH264;
    }
    if (updates.isEmpty) return;

    final mergedCutscene = Map<String, dynamic>.from(cutscene)..addAll(updates);
    final mergedAll = Map<String, dynamic>.from(parsed)
      ..['cutscene'] = mergedCutscene;
    final newRaw = TomlService.updateToml(raw, mergedAll);
    _lastSelfWrite = DateTime.now();
    await TomlService.writeTomlFile(namsPath, newRaw);

    final memRaw = state.namsValues['cutscene'];
    final memCutscene = Map<String, dynamic>.from(
      memRaw is Map<String, dynamic> ? memRaw : const {},
    )..addAll(updates);
    final memNams = Map<String, dynamic>.from(state.namsValues)
      ..['cutscene'] = memCutscene;
    state = state.copyWith(namsValues: memNams, namsRawContent: newRaw);
  }

  Future<void> updateNamsNow(
    String gameDir,
    String key,
    dynamic value, {
    String? section,
  }) async {
    final namsPath = path.join(gameDir, 'nams', 'nams.toml');
    final raw = await TomlService.readTomlFile(namsPath);
    final parsed = TomlService.parseStrict(raw);
    if (!parsed.ok) return;

    final merged = Map<String, dynamic>.from(parsed.values);
    if (section == null) {
      merged[key] = value;
    } else {
      var parent = merged;
      for (final part in section.split('.')) {
        final child = parent[part];
        final next = child is Map<String, dynamic>
            ? Map<String, dynamic>.from(child)
            : <String, dynamic>{};
        parent[part] = next;
        parent = next;
      }
      parent[key] = value;
    }

    final newRaw = TomlService.updateToml(raw, merged);
    _lastSelfWrite = DateTime.now();
    await TomlService.writeTomlFile(namsPath, newRaw);
    state = state.copyWith(
      namsValues: _withValue(state.namsValues, key, value, section: section),
      namsRawContent: newRaw,
    );
  }

  Timer? _naiomAutosaveTimer;
  String? _naiomAutosaveDir;
  static const _naiomAutosaveDelay = Duration(milliseconds: 300);

  void updateNaiomLive(
    String gameDir,
    String key,
    dynamic value, {
    required String section,
  }) {
    state = state.copyWith(
      namsValues: _withValue(state.namsValues, key, value, section: section),
    );
    _naiomAutosaveTimer?.cancel();
    _naiomAutosaveDir = gameDir;
    _naiomAutosaveTimer = Timer(_naiomAutosaveDelay, () {
      unawaited(_flushNaiom(gameDir));
    });
  }

  Future<void> _flushPendingNaiom() async {
    final dir = _naiomAutosaveDir;
    if (!(_naiomAutosaveTimer?.isActive ?? false) || dir == null) return;
    _naiomAutosaveTimer!.cancel();
    await _flushNaiom(dir);
  }

  Future<void> _flushNaiom(String gameDir) async {
    if (state.naiomError != null) return;
    final values = state.namsValues[naiomSection];
    final newRaw = TomlService.updateToml(
      state.naiomRawContent,
      values is Map<String, dynamic> ? values : const {},
    );
    if (newRaw == state.naiomRawContent) return;
    _lastSelfWrite = DateTime.now();
    await TomlService.writeTomlFile(
      path.join(gameDir, 'nams', 'naiom.toml'),
      newRaw,
    );
    state = state.copyWith(naiomRawContent: newRaw);
  }

  void updateNams(String key, dynamic value, {String? section}) {
    final updated = Map<String, dynamic>.from(state.namsValues);
    if (section != null) {
      var parent = updated;
      for (final part in section.split('.')) {
        final child = Map<String, dynamic>.from(
          (parent[part] as Map<String, dynamic>?) ?? {},
        );
        parent[part] = child;
        parent = child;
      }
      parent[key] = value;
    } else {
      updated[key] = value;
    }
    state = state.copyWith(namsValues: updated, hasUnsavedChanges: true);
  }

  void updateLodmod(String key, dynamic value, {String? section}) {
    final updated = _withValue(
      state.lodmodValues,
      key,
      value,
      section: section,
    );
    state = state.copyWith(lodmodValues: updated, hasUnsavedChanges: true);
  }

  static Map<String, dynamic> _withValue(
    Map<String, dynamic> values,
    String key,
    dynamic value, {
    String? section,
  }) {
    final updated = Map<String, dynamic>.from(values);
    if (section == null) {
      updated[key] = value;
      return updated;
    }
    var parent = updated;
    for (final part in section.split('.')) {
      final child = Map<String, dynamic>.from(
        (parent[part] as Map<String, dynamic>?) ?? {},
      );
      parent[part] = child;
      parent = child;
    }
    parent[key] = value;
    return updated;
  }

  Timer? _lodmodAutosaveTimer;
  static const _lodmodAutosaveDelay = Duration(milliseconds: 100);

  void updateLodmodLive(
    String gameDir,
    String key,
    dynamic value, {
    String? section,
  }) {
    var updated = _withValue(
      state.lodmodValues,
      key,
      value,
      section: section,
    );

    if (section == LodModFields.highGridsEnabled.section &&
        key == LodModFields.highGridsEnabled.key &&
        value is bool) {
      updated = _withValue(
        updated,
        LodModFields.disableManualCulling.key,
        value,
      );
      if (value) {
        updated = _withValue(
          updated,
          LodModFields.disableCameraCulling.key,
          true,
        );
      }
    }

    state = state.copyWith(lodmodValues: updated);

    _lodmodAutosaveTimer?.cancel();
    _lodmodAutosaveTimer = Timer(_lodmodAutosaveDelay, () {
      _flushLodmod(gameDir);
    });
  }

  Future<void> applyLodmodBloom2017Preset(String gameDir) async {
    _lodmodAutosaveTimer?.cancel();
    var values = state.lodmodValues;
    for (final entry in LodModFields.bloom2017Preset.entries) {
      values = _withValue(values, entry.key, entry.value);
    }
    state = state.copyWith(lodmodValues: values);
    await _flushLodmod(gameDir);
  }

  Future<void> resetLodmodToDefaults(String gameDir) async {
    _lodmodAutosaveTimer?.cancel();
    var defaults = <String, dynamic>{};
    for (final f in LodmodConfigService.fields) {
      defaults = _withValue(
        defaults,
        f.key,
        f.defaultValue,
        section: f.section,
      );
    }
    state = state.copyWith(lodmodValues: defaults);
    await _flushLodmod(gameDir);
  }

  Future<void> _flushLodmod(String gameDir) async {
    if (state.lodmodError != null) return;
    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final newRaw = TomlService.updateToml(
      state.lodmodRawContent,
      state.lodmodValues,
    );
    if (newRaw == state.lodmodRawContent) return;
    _lastSelfWrite = DateTime.now();
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

  Future<String> _writeConfig(
    String filePath,
    String raw,
    Map<String, dynamic> values,
    String? parseError,
  ) async {
    if (parseError != null) return raw;
    final updated = TomlService.updateToml(raw, values);
    _lastSelfWrite = DateTime.now();
    await TomlService.writeTomlFile(filePath, updated);
    return updated;
  }

  Future<void> saveConfigs(String gameDir) async {
    final namsPath = path.join(gameDir, 'nams', 'nams.toml');
    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final texturePath = path.join(gameDir, 'nams', 'texture_injection.toml');

    final defaultOutfitsKey = NamsFields.experimentalDefaultOutfits.key;
    final defaultOutfitsWas =
        TomlService.parse(state.namsRawContent)[defaultOutfitsKey] == true;

    final naiomPath = path.join(gameDir, 'nams', 'naiom.toml');
    final naiomValues = state.namsValues[naiomSection];
    final updatedNams = await _writeConfig(
      namsPath,
      state.namsRawContent,
      Map<String, dynamic>.from(state.namsValues)..remove(naiomSection),
      state.namsError,
    );
    final updatedNaiom = await _writeConfig(
      naiomPath,
      state.naiomRawContent,
      naiomValues is Map<String, dynamic> ? naiomValues : const {},
      state.naiomError,
    );
    final updatedLodmod = await _writeConfig(
      lodmodPath,
      state.lodmodRawContent,
      state.lodmodValues,
      state.lodmodError,
    );
    final updatedTexture = await _writeConfig(
      texturePath,
      state.textureInjectionRawContent,
      state.textureInjectionValues,
      state.textureInjectionError,
    );

    state = state.copyWith(
      namsRawContent: updatedNams,
      naiomRawContent: updatedNaiom,
      lodmodRawContent: updatedLodmod,
      textureInjectionRawContent: updatedTexture,
      hasUnsavedChanges: false,
    );

    if ((state.namsValues[defaultOutfitsKey] == true) != defaultOutfitsWas) {
      await ref.read(defaultModsStateControllerProvider.notifier).load(gameDir);
      ref.read(detectionRefreshProvider.notifier).state++;
    }
  }

  Future<void> discardChanges(String gameDir) async {
    await loadConfigs(gameDir);
  }
}
