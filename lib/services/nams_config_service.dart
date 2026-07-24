import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/naiom_keys.dart';
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
content_buddy_ruby_selector = true

# Play the visual effects during an outfit hot-swap: the pod spawn-in blinder
# animation, the curtain, and the hacking-screen glitch filter. Set to false for
# an instant, effect-free swap (the model still reloads). Live toggle.
outfit_swap_visual_effects = true

# EXPERIMENTAL. Default-outfit boot mods: mods listed in nams/default_mods.toml
# are active from game start, as if their files were placed in NieRAutomata/data.
# Off by default while the feature stabilizes. Requires restart.
experimental_default_outfits = false

disable_reshade_loading = false
disable_3dmigoto_loading = false
disable_texture_injection = false

# Skip the startup splash window shown while the game loads. Vanilla revealed
# the game window before it was ready, causing resize/flicker artifacts during
# startup. NAMS finished the implementation so the splash covers startup and
# only reveals the window once ready. Set to true to bring the artifacts back.
disable_splash_screen = false

fix_wind_timer_bug = true

[mouse]
fix_camera_acceleration = false
sensitivity = 2.0
third_person_mode = false
third_person_char_follow = false
third_person_sensitivity_x = 1.0
third_person_sensitivity_y = 1.0
aim_mode = false
aim_crosshair = false
aim_crosshair_always = false
aim_sensitivity = 0.001
aim_sensitivity_x = 1.0
aim_sensitivity_y = 1.0
aim_output_multiplier = 15.0
movement_disable_tap_evade = false
misc_disable_pod_pet = false
misc_open_debug_menu = 0
misc_custom_cursor_menu = ""
misc_custom_cursor_hacking = ""
misc_disable_default_cursor = false

[mouse.bindings]
standard_move_forward = ""
standard_move_left = ""
standard_move_backward = ""
standard_move_right = ""
standard_jump = ""
standard_walk = ""
standard_auto_run = ""
standard_light_attack = ""
standard_heavy_attack = ""
standard_program = ""
standard_lock_on = ""
standard_use = ""
standard_self_destruct = ""
standard_light = ""
standard_reset_camera = ""
standard_menu_up = ""
standard_menu_left = ""
standard_menu_down = ""
standard_menu_right = ""
standard_menu_open = ""
standard_menu_back = ""
standard_menu_enter = ""
standard_shortcut_menu = ""
standard_switch_weapon = ""
standard_next_program = ""
standard_previous_program = ""
standard_fire = ""
non_standard_evade = ""
non_standard_auto_fire = ""
non_standard_next_item = ""
non_standard_previous_item = ""
non_standard_use_item = ""
third_person_mode_toggle = ""
aim_mode_toggle = ""

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
# Earlier entries win when the same texture hash exists in multiple packs.
# Loose .dds files directly in nams/inject/textures/ win over the packs below.
#
# Example (Androids Remastered wins over GPUnity HD Pack):
#   load_order = ["Androids Remastered", "GPUnity HD Pack"]
load_order = []
''';

  static const _mouseSectionDefault = '''
# Mouse input fixes.
[mouse]
# Remove the deadzone and acceleration curve from camera rotation.
fix_camera_acceleration = false
# Sensitivity multiplier for fix_camera_acceleration. Higher = faster rotation.
sensitivity = 2.0
# Raw-input third-person camera driven directly from raw mouse deltas.
third_person_mode = false
# Keep the game's automatic camera-follow while moving.
third_person_char_follow = false
# Per-axis sensitivity for third_person_mode. Negative inverts the axis.
third_person_sensitivity_x = 1.0
third_person_sensitivity_y = 1.0
# Remove the clamp and deadzone from pod/mech aiming.
aim_mode = false
# Aim follows the hidden mouse cursor and draws a crosshair. Needs aim_mode.
aim_crosshair = false
# Keep the crosshair visible even when not firing. Off = only shown while shooting.
aim_crosshair_always = false
# Aim sensitivity for top-down/side-scroll. 0.001 fits ~3500 DPI, 0.003 ~800 DPI.
aim_sensitivity = 0.001
# Per-axis multipliers on top of aim_sensitivity. Negative inverts the axis.
aim_sensitivity_x = 1.0
aim_sensitivity_y = 1.0
# Raw multiplier applied to aim output after normalization.
aim_output_multiplier = 15.0
# Disable dodge on double-tapping movement keys (use with non_standard_evade).
movement_disable_tap_evade = false
# Disable the pod petting animation that triggers when moving the mouse.
misc_disable_pod_pet = false
# Virtual key code for opening the after-clearance debug menu. 0 = disabled.
misc_open_debug_menu = 0
# Custom mouse cursor. Path to a .cur/.ani file; "" uses the bundled default.
misc_custom_cursor_menu = ""
# Cursor for the hacking minigame. "" falls back to the menu cursor.
misc_custom_cursor_hacking = ""
# Keep the system cursor instead of the bundled default.
misc_disable_default_cursor = false
''';

  static const _bindingsSectionDefault = '''

# Additional key bindings. All bindings are additive - the vanilla keys keep
# working. "" = unbound. Supported key names: A-Z, 0-9, SPACE, END, UPARROW,
# DOWNARROW, LEFTARROW, RIGHTARROW, LSHIFT, RSHIFT, LCTRL, RCTRL, MOUSE1-MOUSE5.
# Combinations use +, e.g. "CTRL+X". SHIFT, CTRL and ALT match either side
# and are modifier-only.
[mouse.bindings]
standard_move_forward = ""
standard_move_left = ""
standard_move_backward = ""
standard_move_right = ""
standard_jump = ""
standard_walk = ""
standard_auto_run = ""
standard_light_attack = ""
standard_heavy_attack = ""
standard_program = ""
standard_lock_on = ""
standard_use = ""
standard_self_destruct = ""
standard_light = ""
standard_reset_camera = ""
standard_menu_up = ""
standard_menu_left = ""
standard_menu_down = ""
standard_menu_right = ""
standard_menu_open = ""
standard_menu_back = ""
standard_menu_enter = ""
standard_shortcut_menu = ""
standard_switch_weapon = ""
standard_next_program = ""
standard_previous_program = ""
standard_fire = ""
# Dedicated dodge/evade - same distance and duration as the double-tap version.
non_standard_evade = ""
# Toggles continuous pod fire on/off.
non_standard_auto_fire = ""
# Quick-item shortcuts: switch/use items directly without opening the menu.
non_standard_next_item = ""
non_standard_previous_item = ""
non_standard_use_item = ""
# Toggle the camera raw-input fix on/off while playing.
third_person_mode_toggle = ""
# Toggle the aim fix on/off while playing.
aim_mode_toggle = ""
''';

  static const _cutsceneSectionDefault = '''

# Cutscene settings.
[cutscene]
# Enable HD cutscene support. Set true if you installed HD cutscene mods.
hd_cutscenes = false
# Enable H264 codec for USM playback. Required for H264-encoded cutscene mods.
enable_h264 = false
''';

  static const _mouseKeyRenames = <String, String>{
    'fix_aim_acceleration': 'aim_mode',
    'disable_pod_pet': 'misc_disable_pod_pet',
    'debug_menu_key': 'misc_open_debug_menu',
  };

  static const _newMouseKeyBlocks = <String, String>{
    'third_person_mode':
        '# Raw-input third-person camera driven directly from raw mouse deltas.\n'
        'third_person_mode = false',
    'third_person_char_follow':
        '# Keep the game\'s automatic camera-follow while moving.\n'
        'third_person_char_follow = false',
    'third_person_sensitivity_x':
        '# Per-axis sensitivity for third_person_mode. Negative inverts the axis.\n'
        'third_person_sensitivity_x = 1.0',
    'third_person_sensitivity_y': 'third_person_sensitivity_y = 1.0',
    'aim_crosshair':
        '# Aim follows the hidden mouse cursor and draws a crosshair. Needs aim_mode.\n'
        'aim_crosshair = false',
    'aim_crosshair_always':
        '# Keep the crosshair visible even when not firing. Off = only while shooting.\n'
        'aim_crosshair_always = false',
    'aim_sensitivity_x':
        '# Per-axis multipliers on top of aim_sensitivity. Negative inverts the axis.\n'
        'aim_sensitivity_x = 1.0',
    'aim_sensitivity_y': 'aim_sensitivity_y = 1.0',
    'movement_disable_tap_evade':
        '# Disable dodge on double-tapping movement keys (use with non_standard_evade).\n'
        'movement_disable_tap_evade = false',
    'misc_custom_cursor_menu':
        '# Custom mouse cursor. Path to a .cur/.ani file; "" uses the bundled default.\n'
        'misc_custom_cursor_menu = ""',
    'misc_custom_cursor_hacking':
        '# Cursor for the hacking minigame. "" falls back to the menu cursor.\n'
        'misc_custom_cursor_hacking = ""',
    'misc_disable_default_cursor':
        '# Keep the system cursor instead of the bundled default.\n'
        'misc_disable_default_cursor = false',
  };

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
              'validate_scripts = false\n'
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
      NamsFields.contentBuddyRubySelector.key,
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

    final topLevelAdditions = <String, String>{
      NamsFields.outfitSwapVisualEffects.key:
          '# Play the visual effects during an outfit hot-swap: the pod spawn-in\n'
          '# blinder animation, the curtain, and the hacking-screen glitch filter.\n'
          '# Set to false for an instant, effect-free swap. Live toggle.\n'
          'outfit_swap_visual_effects = true\n',
      NamsFields.disableSplashScreen.key:
          '# Skip the startup splash window shown while the game loads. Setting this\n'
          '# to true brings back the vanilla resize/flicker artifacts on startup.\n'
          'disable_splash_screen = false\n',
      'experimental_default_outfits':
          '# EXPERIMENTAL. Default-outfit boot mods: mods listed in\n'
          '# nams/default_mods.toml are active from game start. Off by default\n'
          '# while the feature stabilizes. Requires restart.\n'
          'experimental_default_outfits = false\n',
      NamsFields.disable3dmigotoLoading.key:
          '# Skip loading the 3DMigoto runtime from thirdparty/3dmigoto/. Managed\n'
          '# by the Third Party tab; set to true to stop loading it.\n'
          'disable_3dmigoto_loading = false\n',
      NamsFields.disableReShadeLoading.key:
          '# Skip loading ReShade from thirdparty/reshade/. Managed by the Third\n'
          '# Party tab; set to true to run without ReShade.\n'
          'disable_reshade_loading = false\n',
      NamsFields.loadingStallHints.key:
          '# Show escalating hints when the "Loading Map" screen takes too long.\n'
          'loading_stall_hints = true\n',
      NamsFields.fixWindTimerBug.key:
          '# Fix the vanilla bug where wind animation stops after max playtime.\n'
          'fix_wind_timer_bug = true\n',
    };
    for (final entry in topLevelAdditions.entries) {
      if (content.contains(entry.key)) continue;
      content = _insertTopLevelSync(content, entry.value);
      modified = true;
    }

    for (final entry in _mouseKeyRenames.entries) {
      final re = RegExp('^${entry.key}(\\s*=)', multiLine: true);
      if (re.hasMatch(content)) {
        content = content.replaceAllMapped(
          re,
          (m) => '${entry.value}${m.group(1)}',
        );
        modified = true;
      }
    }

    String? migratedEvadeName;
    final evadeMatch = RegExp(
      r'^evade_key\s*=\s*(\S+).*$',
      multiLine: true,
    ).firstMatch(content);
    if (evadeMatch != null) {
      final vk = int.tryParse(evadeMatch.group(1)!);
      if (vk != null && vk != 0) {
        migratedEvadeName = NaiomKeys.vkToKeyName(vk);
      }
      content = content.replaceFirst(
        RegExp(
          r'^# Virtual key code for dedicated evade/dodge\. 0 = disabled\.\r?\n',
          multiLine: true,
        ),
        '',
      );
      content = content.replaceFirst(
        RegExp(r'^evade_key\s*=.*\r?\n?', multiLine: true),
        '',
      );
      modified = true;
    }

    if (!content.contains('[mouse]')) {
      content += _mouseSectionDefault;
      modified = true;
    } else {
      final missingBlocks = <String>[];
      for (final entry in _newMouseKeyBlocks.entries) {
        final re = RegExp('^${entry.key}\\s*=', multiLine: true);
        if (!re.hasMatch(content)) {
          missingBlocks.add(entry.value);
        }
      }
      if (missingBlocks.isNotEmpty) {
        final afterMouse = content.indexOf('[mouse]') + '[mouse]'.length;
        final nextSection = content.indexOf(
          RegExp(r'^\[', multiLine: true),
          afterMouse,
        );
        final block = '${missingBlocks.join('\n')}\n';
        if (nextSection == -1) {
          final sep = content.endsWith('\n') ? '' : '\n';
          content = '$content$sep$block';
        } else {
          content =
              '${content.substring(0, nextSection)}$block\n'
              '${content.substring(nextSection)}';
        }
        modified = true;
      }
    }

    if (!content.contains('[mouse.bindings]')) {
      content += _bindingsSectionDefault;
      modified = true;
    }

    if (migratedEvadeName != null) {
      final unboundEvade = RegExp(
        r'^non_standard_evade\s*=\s*""',
        multiLine: true,
      );
      if (unboundEvade.hasMatch(content)) {
        content = content.replaceFirst(
          unboundEvade,
          'non_standard_evade = "$migratedEvadeName"',
        );
        modified = true;
      }
    }

    if (!content.contains('[cutscene]')) {
      content += _cutsceneSectionDefault;
      modified = true;
    }

    if (modified) {
      file.writeAsStringSync(content);
    }
  }

  static String _insertTopLevelSync(String content, String block) {
    final firstSectionIdx = content.indexOf(RegExp(r'^\[', multiLine: true));
    final prefix = content.endsWith('\n') || content.isEmpty ? '' : '\n';
    if (firstSectionIdx == -1) {
      return '$content$prefix\n$block';
    }
    return '${content.substring(0, firstSectionIdx)}$block\n'
        '${content.substring(firstSectionIdx)}';
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
