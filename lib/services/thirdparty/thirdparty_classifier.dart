import 'package:yp_launcher/services/detection/graphics_dll_id.dart';
import 'package:yp_launcher/services/detection/lodmod_detection.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';

class ThirdPartyClassifier {
  ThirdPartyClassifier._();

  static ThirdPartyClassification classify(String extractedRoot) {
    final root = FileOps.unwrapSingleChild(extractedRoot);

    final inner = findInnerArchives(root);
    if (inner.length >= 2) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.multiBundle,
        sourceRoot: root,
        innerArchives: inner,
      );
    }

    final lodmodIni = FileOps.filesWhere(
      root,
      (rel, _) => rel == 'lodmod.ini' || rel.endsWith('/lodmod.ini'),
    ).firstOrNull;
    if (lodmodIni != null) {
      Map<String, dynamic>? values;
      try {
        values = LodModDetection.parseLodModIni(lodmodIni.readAsStringSync());
      } catch (_) {}
      return ThirdPartyClassification(
        kind: ThirdPartyKind.lodmod,
        sourceRoot: root,
        lodmodValues: values,
        lodmodCount: values?.length ?? 0,
      );
    }

    final migotoDll = findGraphicsDll(root, GraphicsDll.migoto);
    final hasD3dx = hasFileMatching(root, (rel) => baseName(rel) == 'd3dx.ini');
    final hasShaderFixes = hasDirNamed(root, 'shaderfixes');
    if (migotoDll != null || (hasD3dx && hasShaderFixes)) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.migoto,
        sourceRoot: root,
      );
    }

    if (hasFileMatching(root, (rel) => baseName(rel) == 'dinput8.dll') ||
        hasFileMatching(
          root,
          (rel) => rel.contains('specialk') || baseName(rel).startsWith('sk_'),
        )) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.specialK,
        sourceRoot: root,
      );
    }

    if (hasFileMatching(root, (rel) => baseName(rel) == 'd3d11_shaders.ini')) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.skShaderEdit,
        sourceRoot: root,
      );
    }

    final reshadeDll = findGraphicsDll(root, GraphicsDll.reshade);
    final shaders = hasReShadeShaders(root);
    final presets = findPresets(root);

    final hasDxgiIni = hasFileMatching(root, (rel) => baseName(rel) == 'dxgi.ini');
    if (hasDxgiIni && reshadeDll == null && presets.isEmpty && !shaders) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.brokenLegacy,
        sourceRoot: root,
      );
    }

    if (reshadeDll != null || shaders) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.reshadeWholeInstall,
        sourceRoot: root,
        hasReShadeDll: reshadeDll != null,
        isAddonBuild: hasAddonFiles(root) || (reshadeDll?.isAddon ?? false),
        hasShaders: shaders,
        presetCount: presets.length,
      );
    }

    if (presets.isNotEmpty) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.reshadePreset,
        sourceRoot: root,
        hasShaders: shaders,
        presetCount: presets.length,
      );
    }

    if (hasFileMatching(root, (rel) {
      final b = baseName(rel);
      return b.endsWith('.dtt') ||
          b.endsWith('.dat') ||
          b.endsWith('.cpk') ||
          b.endsWith('.pak');
    })) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.gameData,
        sourceRoot: root,
      );
    }

    if (hasFileMatching(root, (rel) => baseName(rel).endsWith('.dds'))) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.textures,
        sourceRoot: root,
      );
    }

    if (hasFileMatching(root, (rel) => baseName(rel).endsWith('.cfg')) ||
        onlyImages(root)) {
      return ThirdPartyClassification(
        kind: ThirdPartyKind.brokenLegacy,
        sourceRoot: root,
      );
    }

    return ThirdPartyClassification(
      kind: ThirdPartyKind.unknown,
      sourceRoot: root,
    );
  }
}

extension _IterableFirstOrNull<E> on Iterable<E> {
  E? get firstOrNull {
    final it = iterator;
    return it.moveNext() ? it.current : null;
  }
}
