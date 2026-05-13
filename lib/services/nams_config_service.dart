import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/models/config_fields.dart';

class NamsConfigService {
  static const _lodmodToml =
      '''# NAMS LodMod - Visual quality improvements. Restart game after changes.

# Enable/disable all LodMod patches
enabled = false

# 0 = best quality (no LOD pop-in), 1 = vanilla, higher = further LOD distance
lod_multiplier = 0.0

# AO sharpness (0.1-2.0). Set to 2.0 for crisp ambient occlusion
ao_multiplier_width = 1.0
ao_multiplier_height = 1.0

# Shadow texture size. 4096 or 8192 for sharper shadows (vanilla: 2048)
shadow_resolution = 2048

# How far shadows reach. 2.0 = twice as far (vanilla: 1.0)
shadow_distance_multiplier = 1.0
# Advanced: min/max clamp for shadow distance. 0 = off
shadow_distance_minimum = 0.0
shadow_distance_maximum = 0.0
# Advanced: even shadow distribution factor. 0 = off, try 0.5-0.9
shadow_distance_pss = 0.0

# Advanced: shadow softness tweaks. 0 = no change
shadow_filter_strength_bias = 0.0
shadow_filter_strength_minimum = 0.0
shadow_filter_strength_maximum = 0.0

# Better shadow models on trees/foliage. Recommended: true
shadow_model_hq = false
# More objects cast shadows. Recommended: true
shadow_model_force_all = false

# Stops objects popping in/out. Recommended: true
disable_manual_culling = false

# Removes dark edges around the screen. Recommended: true
disable_vignette = false
# Disables the game's fake HDR effect
disable_fake_hdr = false
''';

  static const _namsToml = '''# NAMS General Config. Restart game after changes.

validate_model_data = false
validate_scripts = false
loading_stall_hints = true
disable_plugin_loading = false
disable_content_features = false

# Per-feature content-layer toggles. All default to true. Useful for bisecting
# mod bugs without disabling the entire content layer.
content_items = true
content_accessories = true
content_assemble_meshes = true
content_quest_integration = true
content_effects_applier = true
content_equip_tracker = true
content_mcd = true

disable_reshade_loading = false
disable_texture_injection = false
fix_wind_timer_bug = true

[mouse]
fix_camera_acceleration = false
sensitivity = 2.0
disable_pod_pet = false
fix_aim_acceleration = false
aim_sensitivity = 0.001
aim_output_multiplier = 15.0
debug_menu_key = 0
evade_key = 0

[cutscene]
hd_cutscenes = false
enable_h264 = false

[heap]
global_heap_extra = 0
pl_file_heap_extra = 0
pl_vram_heap_extra = 0
em_bg_file_heap_extra = 0
em_bg_vram_heap_extra = 0
''';

  static const _textureInjectionToml =
      '''# NAMS Texture Injection Config - Restart game after changes.

# VRAM budget in MB for replacement textures.
# 0 = auto-detect from your GPU (recommended for most users).
# When the budget is full, least-recently-used textures are swapped out
# to make room for textures in the current area.
# Only change this if auto-detect gives bad results for your GPU.
vram_budget_mb = 0

# Enable async background streaming.
# true = textures load in the background without stutter (recommended).
# false = textures load when the game needs them (may cause stutter on slow disks).
streaming_enabled = true

# Only load textures matching the built-in priority hash list (curated from GPUnity HD Pack).
# When true: texture packs with 400+ files are filtered to only the important hashes.
#            Small packs (<400 files, like clothing or weapon mods) are loaded fully.
# When false: all textures from all packs are loaded.
# Turn this on if you have a huge pack and want to save VRAM / loading time.
load_only_relevant = false

# Load order for texture pack folders inside nams/inject/textures/.
# Later entries override earlier ones when the same texture hash exists in multiple packs.
# Loose .dds files directly in nams/inject/textures/ always have the lowest priority.
#
# Example:
#   load_order = ["GPUnity HD Pack", "Androids Remastered"]
load_order = []
''';

  static const _mouseSectionDefault = '''
# Mouse input fixes.
[mouse]
# Remove the deadzone and acceleration curve from camera rotation.
fix_camera_acceleration = false
# Sensitivity multiplier. Higher = faster camera rotation.
sensitivity = 2.0
# Disable the pod petting animation that triggers when moving the mouse.
disable_pod_pet = false
# Remove the clamp and deadzone from pod/mech aiming.
fix_aim_acceleration = false
# Aim sensitivity for top-down/side-scroll.
aim_sensitivity = 0.001
# Virtual key code for opening the debug menu. 0 = disabled.
debug_menu_key = 0
# Virtual key code for dedicated evade/dodge. 0 = disabled.
evade_key = 0
''';

  static const _cutsceneSectionDefault = '''

# Cutscene settings.
[cutscene]
# Enable HD cutscene support. Set true if you installed HD cutscene mods.
hd_cutscenes = false
# Enable H264 codec for USM playback. Required for H264-encoded cutscene mods.
enable_h264 = false
''';

  static Future<void> ensureConfigs(String gameDir) {
    return IsolateService.run(_ensureConfigsSync, gameDir);
  }

  static void _ensureConfigsSync(String gameDir) {
    final namsDir = Directory(path.join(gameDir, 'nams'));
    if (!namsDir.existsSync()) {
      namsDir.createSync(recursive: true);
    }

    _writeIfMissingSync(path.join(namsDir.path, 'nams.toml'), _namsToml);
    _writeIfMissingSync(path.join(namsDir.path, 'lodmod.toml'), _lodmodToml);
    _writeIfMissingSync(
      path.join(namsDir.path, 'texture_injection.toml'),
      _textureInjectionToml,
    );

    _migrateNamsTomlSync(path.join(namsDir.path, 'nams.toml'));
    _migrateTextureInjectionTomlSync(
      path.join(namsDir.path, 'texture_injection.toml'),
    );
  }

  static void _writeIfMissingSync(String filePath, String content) {
    final file = File(filePath);
    if (!file.existsSync()) {
      file.writeAsStringSync(content);
    }
  }

  static void _migrateNamsTomlSync(String filePath) {
    final file = File(filePath);
    if (!file.existsSync()) return;

    var content = file.readAsStringSync();
    var modified = false;

    if (!content.contains(NamsFields.validateScripts.key)) {
      final insertPoint = content.indexOf(NamsFields.validateModelData.key);
      if (insertPoint != -1) {
        final lineEnd = content.indexOf('\n', insertPoint);
        if (lineEnd != -1) {
          content =
              '${content.substring(0, lineEnd + 1)}\n'
              '# Catch script errors and show a dialog instead of silently crashing.\n'
              'validate_scripts = true\n'
              '${content.substring(lineEnd + 1)}';
          modified = true;
        }
      }
    }

    if (!content.contains(NamsFields.disableContentFeatures.key)) {
      final insertPoint = content.indexOf(NamsFields.disablePluginLoading.key);
      if (insertPoint != -1) {
        final lineEnd = content.indexOf('\n', insertPoint);
        if (lineEnd != -1) {
          content =
              '${content.substring(0, lineEnd + 1)}'
              'disable_content_features = false\n'
              '${content.substring(lineEnd + 1)}';
          modified = true;
        }
      } else {
        content += '\ndisable_content_features = false\n';
        modified = true;
      }
    }

    final contentToggleKeys = <String>[
      NamsFields.contentItems.key,
      NamsFields.contentAccessories.key,
      NamsFields.contentAssembleMeshes.key,
      NamsFields.contentQuestIntegration.key,
      NamsFields.contentEffectsApplier.key,
      NamsFields.contentEquipTracker.key,
      NamsFields.contentMcd.key,
    ];
    final missingContent = contentToggleKeys.where((k) => !content.contains(k)).toList();
    if (missingContent.isNotEmpty) {
      // Insert before any [section] header so new keys land at the top level.
      final firstSectionIdx = content.indexOf(RegExp(r'^\[', multiLine: true));
      final block = StringBuffer();
      if (!content.endsWith('\n')) block.write('\n');
      block.writeln();
      block.writeln(
        '# Per-feature content-layer toggles. All default to true. Useful for bisecting',
      );
      block.writeln(
        '# mod bugs without disabling the entire content layer.',
      );
      for (final k in missingContent) {
        block.writeln('$k = true');
      }
      block.writeln();
      if (firstSectionIdx == -1) {
        content = '$content${block.toString()}';
      } else {
        content =
            '${content.substring(0, firstSectionIdx)}${block.toString()}${content.substring(firstSectionIdx)}';
      }
      modified = true;
    }

    if (!content.contains('[mouse]')) {
      content += _mouseSectionDefault;
      modified = true;
    }

    if (!content.contains('[cutscene]')) {
      content += _cutsceneSectionDefault;
      modified = true;
    }

    if (modified) {
      file.writeAsStringSync(content);
    }
  }

  static void _migrateTextureInjectionTomlSync(String filePath) {
    final file = File(filePath);
    if (!file.existsSync()) return;

    var content = file.readAsStringSync();
    var modified = false;

    final hasLegacyPreload = content.contains('preload_max_dimension') ||
        content.contains('preload_all');

    if (hasLegacyPreload) {
      final lines = content.split('\n');
      final kept = <String>[];
      var skipNextBlank = false;
      for (final line in lines) {
        final trimmed = line.trimLeft();
        if (trimmed.startsWith('preload_max_dimension') ||
            trimmed.startsWith('preload_all')) {
          skipNextBlank = true;
          continue;
        }
        if (skipNextBlank && trimmed.isEmpty) {
          skipNextBlank = false;
          continue;
        }
        skipNextBlank = false;
        kept.add(line);
      }
      content = kept.join('\n');
      content = content.replaceAll(
        RegExp(r'vram_budget_mb\s*=\s*4096'),
        'vram_budget_mb = 0',
      );
      modified = true;
    }

    if (!content.contains(TextureInjectionFields.vramBudgetMb.key)) {
      content += '''

# VRAM budget in MB for replacement textures. 0 = auto-detect from GPU.
vram_budget_mb = 0
''';
      modified = true;
    }

    if (!content.contains(TextureInjectionFields.streamingEnabled.key)) {
      content += '''

# Enable async background streaming. true = no stutter (recommended).
streaming_enabled = true
''';
      modified = true;
    }

    if (!content.contains(TextureInjectionFields.loadOnlyRelevant.key)) {
      content += '''

# Only load priority textures from huge packs (400+ files). Small packs loaded fully.
load_only_relevant = false
''';
      modified = true;
    }

    if (!content.contains(TextureInjectionFields.loadOrder.key)) {
      content += '''

# Load order for texture pack folders inside nams/inject/textures/.
# Later entries override earlier ones when the same texture hash exists in multiple packs.
#
# Example:
#   load_order = ["GPUnity HD Pack", "Androids Remastered"]
load_order = []
''';
      modified = true;
    }

    if (modified) file.writeAsStringSync(content);
  }
}
