import 'package:toml/toml.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/toml_service.dart';

class LodmodConfigService {
  static final fields = <ConfigField<dynamic>>[
    LodModFields.enabled,
    LodModFields.lodMultiplier,
    LodModFields.layoutSpawnRange,
    LodModFields.layoutSpawnEverything,
    LodModFields.disableCameraCulling,
    LodModFields.aoFullResolution,
    LodModFields.bloomReferenceHeight,
    LodModFields.bloomKernelReferenceHeight,
    LodModFields.bloomExtraBlur,
    LodModFields.bloomDropCoarseLevels,
    LodModFields.shadowResolution,
    LodModFields.shadowDistanceMinimum,
    LodModFields.shadowDistanceMaximum,
    LodModFields.shadowDistanceMultiplier,
    LodModFields.shadowDistancePss,
    LodModFields.shadowFilterStrengthBias,
    LodModFields.shadowFilterStrengthMinimum,
    LodModFields.shadowFilterStrengthMaximum,
    LodModFields.shadowCascades,
    LodModFields.shadowCascadeRange,
    LodModFields.shadowBlurScale,
    LodModFields.shadowModelHq,
    LodModFields.shadowModelForceAll,
    LodModFields.shadowForceAllFilters,
    LodModFields.shadowModelForceObjectParam,
    LodModFields.disableManualCulling,
    LodModFields.giEnabled,
    LodModFields.giWorkgroupSize,
    LodModFields.fpsUncapInMenus,
    LodModFields.fpsUncapInGameplay,
    LodModFields.fpsCapInHacking,
    LodModFields.fpsCapInEvents,
    LodModFields.fpsLimit,
    LodModFields.disableVignette,
    LodModFields.disableHdr,
    LodModFields.renderScale,
    LodModFields.fxaa,
    LodModFields.msaaPrepassFix,
    LodModFields.msaaShadowMaskFix,
    LodModFields.msaaPerPixel,
    LodModFields.msaaPerPixelTerrain,
    LodModFields.aoFadeFix,
    LodModFields.constantBufferDedup,
    LodModFields.constantBufferUploadOnBind,
    LodModFields.decreaseStutterDuringGridLoading,
    LodModFields.highGridsEnabled,
    LodModFields.highGridsRings,
    LodModFields.highGridsRoomRings,
    LodModFields.highGridsBlockedInRoom,
    LodModFields.highGridsBlockedFromGrid,
    LodModFields.highGridsHiddenMeshes,
  ];

  static String migrate(String raw) {
    final parsed = TomlService.parseStrict(raw);
    if (!parsed.ok) return raw;
    final existingGrids = parsed.values['high_grids'];
    if (existingGrids != null && existingGrids is! Map<String, dynamic>) {
      return raw;
    }
    final values = Map<String, dynamic>.from(parsed.values);
    final grids = Map<String, dynamic>.from(existingGrids ?? const {});
    final gridKeys = fields
        .where((f) => f.section == 'high_grids')
        .map((f) => f.key)
        .toSet();
    final misplaced = fields
        .where((f) => f.section == null)
        .map((f) => f.key)
        .where((key) => !gridKeys.contains(key) && grids.containsKey(key))
        .toSet();
    for (final key in misplaced) {
      values.putIfAbsent(key, () => grids[key]);
      grids.remove(key);
    }
    values['high_grids'] = grids;
    for (final field in fields) {
      final target = field.section == null ? values : grids;
      target.putIfAbsent(field.key, () => field.defaultValue);
    }
    if (TomlService.sameValues(values, parsed.values)) return raw;

    final repaired = StringBuffer();
    final recovered = StringBuffer();
    var section = <String>[];
    for (final span in TomlService.sourceExpressions(raw)) {
      final expression = span.expression;
      final source = raw.substring(span.start, span.end);
      if (expression is TomlTable) {
        section = expression.name.parts.map((part) => part.name).toList();
      }
      if (expression is TomlKeyValuePair) {
        final fullPath = [
          ...section,
          ...expression.key.parts.map((part) => part.name),
        ];
        if (fullPath.length == 2 &&
            fullPath.first == 'high_grids' &&
            misplaced.contains(fullPath.last)) {
          if (!parsed.values.containsKey(fullPath.last)) {
            if (expression.key.parts.length == 1) {
              recovered.write(source);
            } else {
              recovered.writeln('${fullPath.last} = ${expression.value}');
            }
            if (!recovered.toString().endsWith('\n')) recovered.writeln();
          }
          continue;
        }
      }
      repaired.write(source);
    }
    final migrated = TomlService.updateToml('$recovered$repaired', values);
    final result = TomlService.parseStrict(migrated);
    if (result.ok && TomlService.sameValues(result.values, values)) {
      return migrated;
    }
    return TomlService.serialize(values, commentsFrom: raw);
  }
}
