import 'dart:async';
import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/services/nams_settings_service.dart';

part 'nams_settings_state.g.dart';

class NamsSettingsData {
  final Map<String, dynamic> settings;
  final bool isLoading;
  final bool hasUnsavedChanges;

  /// Null while the location is unknown, e.g. before a game directory
  /// identifies the Wine prefix.
  final String? settingsPath;

  const NamsSettingsData({
    this.settings = const {},
    this.isLoading = false,
    this.hasUnsavedChanges = false,
    this.settingsPath,
  });

  NamsSettingsData copyWith({
    Map<String, dynamic>? settings,
    bool? isLoading,
    bool? hasUnsavedChanges,
    String? settingsPath,
  }) {
    return NamsSettingsData(
      settings: settings ?? this.settings,
      isLoading: isLoading ?? this.isLoading,
      hasUnsavedChanges: hasUnsavedChanges ?? this.hasUnsavedChanges,
      settingsPath: settingsPath ?? this.settingsPath,
    );
  }

  Map<String, dynamic> get keybinds =>
      (settings['keybinds'] as Map<String, dynamic>?) ?? {};

  Map<String, dynamic> get mainKeybinds =>
      (keybinds['main'] as Map<String, dynamic>?) ?? {};

  Map<String, dynamic> get ypKeybinds =>
      (keybinds['yorha_protocol'] as Map<String, dynamic>?) ?? {};

  bool get gameKeybindsGlobal => settings['gameKeybindsGlobal'] == true;
  bool get loadingSpeedupEnabled => settings['loadingSpeedupEnabled'] == true;
  bool get shadersEnabled => settings['shadersEnabled'] == true;
  bool get soundEnabled => settings['soundEnabled'] == true;

  Map<String, dynamic> get randomizerConfig =>
      (settings['randomizerConfig'] as Map<String, dynamic>?) ?? {};

  Map<String, dynamic> get defaultOutfits =>
      (settings['defaultOutfits'] as Map<String, dynamic>?) ?? {};

  String? defaultOutfitFor(String characterType) =>
      defaultOutfits[characterType] as String?;

  Map<String, dynamic> get cheatsConfig =>
      (settings['cheatsConfig'] as Map<String, dynamic>?) ?? {};
}

@Riverpod(keepAlive: true)
class NamsSettingsStateController extends _$NamsSettingsStateController {
  StreamSubscription<FileSystemEvent>? _watchSub;
  String? _watchedPath;
  Timer? _reloadDebounce;
  DateTime _lastSelfWrite = DateTime.fromMillisecondsSinceEpoch(0);

  @override
  NamsSettingsData build() {
    ref.onDispose(() {
      _reloadDebounce?.cancel();
      _watchSub?.cancel();
    });
    return const NamsSettingsData();
  }

  Future<void> loadSettings() async {
    state = state.copyWith(isLoading: true);
    final gameDir = _gameDir;
    final settings = await NamsSettingsService.loadSettings(gameDir);
    final settingsPath =
        await NamsSettingsService.resolveSettingsPath(gameDir);
    state = NamsSettingsData(
      settings: settings,
      settingsPath: settingsPath,
    );
    _watchSettingsFile(settingsPath);
  }

  void _watchSettingsFile(String? settingsPath) {
    if (settingsPath == null) return;
    final dirPath = p.dirname(settingsPath);
    if (_watchedPath == dirPath && _watchSub != null) return;
    _watchSub?.cancel();
    _watchSub = null;
    _watchedPath = null;

    final dir = Directory(dirPath);
    if (!dir.existsSync()) return;

    try {
      _watchSub = dir.watch().listen(
        (event) {
          if (p.equals(event.path, settingsPath)) {
            _scheduleExternalReload();
          }
        },
        onError: (_) {},
      );
      _watchedPath = dirPath;
    } catch (_) {}
  }

  void _scheduleExternalReload() {
    _reloadDebounce?.cancel();
    _reloadDebounce = Timer(const Duration(milliseconds: 400), () async {
      if (state.hasUnsavedChanges) return;
      if (DateTime.now().difference(_lastSelfWrite) <
          const Duration(seconds: 2)) {
        return;
      }
      final gameDir = _gameDir;
      final settings = await NamsSettingsService.loadSettings(gameDir);
      state = state.copyWith(settings: settings);
    });
  }

  String? get _gameDir {
    final dir = ref.read(appStateControllerProvider).selectedDirectory;
    return dir.isEmpty ? null : dir;
  }

  void updateKeybind(String category, String action, String key) {
    final updated = _deepCopy(state.settings);
    final keybinds =
        updated.putIfAbsent('keybinds', () => <String, dynamic>{})
            as Map<String, dynamic>;
    final cat =
        keybinds.putIfAbsent(category, () => <String, dynamic>{})
            as Map<String, dynamic>;
    cat[action] = key;
    state = state.copyWith(settings: updated, hasUnsavedChanges: true);
  }

  void updateToggle(String key, bool value) {
    final updated = _deepCopy(state.settings);
    updated[key] = value;
    state = state.copyWith(settings: updated, hasUnsavedChanges: true);
  }

  void setDefaultOutfit(String characterType, String? folderName) {
    final updated = _deepCopy(state.settings);
    final outfits = Map<String, dynamic>.from(
      (updated['defaultOutfits'] as Map<String, dynamic>?) ?? {},
    );
    if (folderName == null) {
      outfits.remove(characterType);
    } else {
      outfits[characterType] = folderName;
    }
    if (outfits.isEmpty) {
      updated.remove('defaultOutfits');
    } else {
      updated['defaultOutfits'] = outfits;
    }
    state = state.copyWith(settings: updated, hasUnsavedChanges: true);
  }

  void updateCheat(String key, dynamic value) {
    final updated = _deepCopy(state.settings);
    final cheats =
        updated.putIfAbsent('cheatsConfig', () => <String, dynamic>{})
            as Map<String, dynamic>;
    cheats[key] = value;
    state = state.copyWith(settings: updated, hasUnsavedChanges: true);
  }

  void updateRandomizer(String key, dynamic value) {
    final updated = _deepCopy(state.settings);
    final randomizer =
        updated.putIfAbsent('randomizerConfig', () => <String, dynamic>{})
            as Map<String, dynamic>;
    randomizer[key] = value;
    state = state.copyWith(settings: updated, hasUnsavedChanges: true);
  }

  Future<bool> saveSettings() async {
    _lastSelfWrite = DateTime.now();
    final saved = await NamsSettingsService.saveSettings(state.settings, _gameDir);
    if (saved) state = state.copyWith(hasUnsavedChanges: false);
    return saved;
  }

  Future<void> discardChanges() async {
    await loadSettings();
  }

  Future<void> reloadIfClean() async {
    if (state.hasUnsavedChanges) return;
    final settings = await NamsSettingsService.loadSettings(_gameDir);
    state = state.copyWith(settings: settings);
  }

  Map<String, dynamic> _deepCopy(Map<String, dynamic> source) {
    return source.map((key, value) {
      if (value is Map<String, dynamic>) {
        return MapEntry(key, _deepCopy(value));
      }
      if (value is List) {
        return MapEntry(key, List.from(value));
      }
      return MapEntry(key, value);
    });
  }
}
