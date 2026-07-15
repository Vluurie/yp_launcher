import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/app_state.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/services/mods_service.dart';

part 'mods_state.g.dart';

class ModsData {
  final List<InstalledMod> mods;
  final bool isLoading;
  final String? error;

  const ModsData({
    this.mods = const [],
    this.isLoading = false,
    this.error,
  });

  ModsData copyWith({
    List<InstalledMod>? mods,
    bool? isLoading,
    String? error,
    bool clearError = false,
  }) {
    return ModsData(
      mods: mods ?? this.mods,
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
    );
  }
}

@Riverpod(keepAlive: true)
class ModsStateController extends _$ModsStateController {
  @override
  ModsData build() => const ModsData();

  Future<void> loadMods(String gameDir) async {
    if (gameDir.isEmpty) return;
    state = state.copyWith(isLoading: true, clearError: true);
    try {
      final mods = await ModsService.listInstalled(gameDir);
      state = ModsData(mods: mods);
    } catch (e) {
      state = ModsData(error: e.toString());
    }
  }

  Future<InstallResult> installMod(
    String gameDir,
    String sourcePath, {
    String? requestedName,
    String? variantSubPath,
    List<String>? texturePackSubPaths,
    void Function(double percent, String? currentFile)? onExtractProgress,
  }) async {
    final result = await ModsService.install(
      gameDir,
      sourcePath,
      requestedName: requestedName,
      variantSubPath: variantSubPath,
      texturePackSubPaths: texturePackSubPaths,
      onExtractProgress: onExtractProgress,
    );
    if (result.success) {
      await loadMods(gameDir);
      await ref
          .read(configStateControllerProvider.notifier)
          .autoDetectCutscenes(gameDir);
      ref.read(detectionRefreshProvider.notifier).state++;
    }
    return result;
  }

  Future<List<InstallResult>> installVariants(
    String gameDir,
    String sourcePath,
    List<VariantInstallRequest> requests, {
    void Function(double percent, String? currentFile)? onExtractProgress,
  }) async {
    final results = await ModsService.installVariants(
      gameDir,
      sourcePath,
      requests,
      onExtractProgress: onExtractProgress,
    );
    if (results.any((r) => r.success)) {
      await loadMods(gameDir);
      await ref
          .read(configStateControllerProvider.notifier)
          .autoDetectCutscenes(gameDir);
      ref.read(detectionRefreshProvider.notifier).state++;
    }
    return results;
  }

  Future<void> uninstallMod(
    String gameDir,
    String modId, {
    bool deleteBundledTextures = false,
  }) async {
    await ModsService.uninstall(
      gameDir,
      modId,
      deleteBundledTextures: deleteBundledTextures,
    );
    await loadMods(gameDir);
    await ref
        .read(configStateControllerProvider.notifier)
        .autoDetectCutscenes(gameDir);
    ref.read(detectionRefreshProvider.notifier).state++;
  }
}
