import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/nams_settings_service.dart';

part 'nams_settings_state.g.dart';

class NamsSettingsData {
  final Map<String, dynamic> settings;
  final bool isLoading;
  final bool hasUnsavedChanges;

  const NamsSettingsData({
    this.settings = const {},
    this.isLoading = false,
    this.hasUnsavedChanges = false,
  });

  NamsSettingsData copyWith({
    Map<String, dynamic>? settings,
    bool? isLoading,
    bool? hasUnsavedChanges,
  }) {
    return NamsSettingsData(
      settings: settings ?? this.settings,
      isLoading: isLoading ?? this.isLoading,
      hasUnsavedChanges: hasUnsavedChanges ?? this.hasUnsavedChanges,
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
  @override
  NamsSettingsData build() {
    return const NamsSettingsData();
  }

  Future<void> loadSettings() async {
    state = state.copyWith(isLoading: true);
    final settings = await NamsSettingsService.loadSettings();
    state = NamsSettingsData(settings: settings);
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

  Future<void> saveSettings() async {
    await NamsSettingsService.saveSettings(state.settings);
    state = state.copyWith(hasUnsavedChanges: false);
  }

  Future<void> discardChanges() async {
    await loadSettings();
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
