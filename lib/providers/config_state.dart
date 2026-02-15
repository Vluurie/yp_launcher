import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as path;
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

part 'config_state.g.dart';

final configPanelOpenProvider = StateProvider<bool>((ref) => false);

class ConfigData {
  final Map<String, dynamic> lodmodValues;
  final Map<String, dynamic> textureInjectionValues;
  final String lodmodRawContent;
  final String textureInjectionRawContent;
  final bool isLoading;
  final bool hasUnsavedChanges;

  const ConfigData({
    this.lodmodValues = const {},
    this.textureInjectionValues = const {},
    this.lodmodRawContent = '',
    this.textureInjectionRawContent = '',
    this.isLoading = false,
    this.hasUnsavedChanges = false,
  });

  ConfigData copyWith({
    Map<String, dynamic>? lodmodValues,
    Map<String, dynamic>? textureInjectionValues,
    String? lodmodRawContent,
    String? textureInjectionRawContent,
    bool? isLoading,
    bool? hasUnsavedChanges,
  }) {
    return ConfigData(
      lodmodValues: lodmodValues ?? this.lodmodValues,
      textureInjectionValues:
          textureInjectionValues ?? this.textureInjectionValues,
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

    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final texturePath = path.join(gameDir, 'nams', 'texture_injection.toml');

    final lodmodRaw = await TomlService.readTomlFile(lodmodPath);
    final textureRaw = await TomlService.readTomlFile(texturePath);

    state = ConfigData(
      lodmodValues: TomlService.parse(lodmodRaw),
      textureInjectionValues: TomlService.parse(textureRaw),
      lodmodRawContent: lodmodRaw,
      textureInjectionRawContent: textureRaw,
    );
  }

  void updateLodmod(String key, dynamic value) {
    final updated = Map<String, dynamic>.from(state.lodmodValues);
    updated[key] = value;
    state = state.copyWith(lodmodValues: updated, hasUnsavedChanges: true);
  }

  void updateTextureInjection(String key, dynamic value) {
    final updated = Map<String, dynamic>.from(state.textureInjectionValues);
    updated[key] = value;
    state = state.copyWith(
      textureInjectionValues: updated,
      hasUnsavedChanges: true,
    );
  }

  Future<void> saveConfigs(String gameDir) async {
    final lodmodPath = path.join(gameDir, 'nams', 'lodmod.toml');
    final texturePath = path.join(gameDir, 'nams', 'texture_injection.toml');

    final updatedLodmod = TomlService.updateToml(
      state.lodmodRawContent,
      state.lodmodValues,
    );
    final updatedTexture = TomlService.updateToml(
      state.textureInjectionRawContent,
      state.textureInjectionValues,
    );

    await TomlService.writeTomlFile(lodmodPath, updatedLodmod);
    await TomlService.writeTomlFile(texturePath, updatedTexture);

    state = state.copyWith(
      lodmodRawContent: updatedLodmod,
      textureInjectionRawContent: updatedTexture,
      hasUnsavedChanges: false,
    );
  }

  Future<void> discardChanges(String gameDir) async {
    await loadConfigs(gameDir);
  }
}
