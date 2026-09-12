import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/naiom_keys.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/nams_settings_service.dart';
import 'package:yp_launcher/services/wine/proton.dart';
import 'package:yp_launcher/services/wine/wine_paths.dart';
import 'package:yp_launcher/models/config_fields.dart';

class NamsConfigService {
  static const _lodmodToml =
      '''# NAMS LodMod Ext - Visual quality improvements. Inspired by Automata-LodMod by
# emoose and extended well past it. Most settings apply live while the game is
# running; the ones that need a restart say so.

# Enable/disable all LodMod Ext patches
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

# Shadow cascades (EXPERIMENTAL): 4 = vanilla, 8 adds four far cascades. With 8 the
# atlas is tiled 4x4, so each cascade gets a quarter of shadow_resolution.
shadow_cascades = 4
# How far past the vanilla shadow distance the four extra cascades reach. Range 1.05..8.0.
shadow_cascade_range = 4.0
# Shadow blur per cascade, multiplier on the game's authored value, near to far.
# 1.0 = authored, 0.5 = half the blur (crisper contact shadows). Takes effect live.
shadow_blur_scale = [1.0, 1.0, 1.0, 1.0]

# Stops objects popping in/out. Recommended: true
disable_manual_culling = false

# Bloom shimmer fix. 0 = vanilla 2021 (bloom scales with resolution and shimmers
# at high res), 900 = 2017-style bloom footprint at any resolution
bloom_reference_height = 0

# Bloom blur width. 0 = vanilla (blur shrinks relative to the screen at high res),
# any other value pins the blur width to that height at any resolution. Keeps the
# targets at full size, so it can be used instead of bloom_reference_height
bloom_kernel_reference_height = 0

# Extra bloom blur rounds (0.1 steps, widens and brightens the glow). 0 = vanilla
bloom_extra_blur = 0.0

# Removes the shimmery, flickering bloom layers while keeping the small glow around
# light sources. 0 = vanilla, 1 = removes the widest layer, 2 = removes the two
# widest layers. Based on Void's Bloom Fix
bloom_drop_coarse_levels = 0

# Reflection cubemap selection (what FAR calls "global illumination"). A per-pixel compute
# pass walks all loaded reflection cubemaps; with this on only the gi_workgroup_size boxes
# nearest to the camera stay in that walk. FAR cuts the list in load order, so nearby surfaces
# can go dull and dark; this keeps the cubemaps closest to you. Takes effect live.
gi_enabled = false
# Cubemaps kept per frame. 128 = everything vanilla keeps, 64/32/16 = cheaper, 0 = none.
gi_workgroup_size = 128

# FPS uncap (FAR-style). WARNING: in gameplay breaks physics/animation/cutscene timing.
fps_uncap_in_menus = false
fps_uncap_in_gameplay = false
# Drop back to the vanilla cap while the hacking minigame runs, even with
# fps_uncap_in_gameplay = true. The minigame's timing assumes 60 fps.
fps_cap_in_hacking = true
# Drop back to the vanilla cap while a cutscene, event, soft event, movie or
# caption is running. Event scripts assume 60 fps and can soft-lock otherwise.
fps_cap_in_events = true
# FPS limit applied when uncap is active. 0 = unlimited.
# Cap at half your monitor refresh rate for smoother motion than vanilla 60
# (e.g. 72 on a 144Hz monitor, 82 on 165Hz). Range 60..=1000.
fps_limit = 0

# Removes dark edges around the screen. Recommended: true
disable_vignette = false

# Play in SDR even on an HDR display, like Special K's HideHDRSupport.
# Turn on if HDR looks washed out. Takes effect live, but the swap chain rebuild can take a
# moment and the frame rate may drop until it is done; if it does not apply, restart the game.
disable_hdr = false

# FXAA, cheap edge smoothing. Use with the in-game anti-aliasing set to Off. Takes effect live.
fxaa = false

# Fixes the black dots on leaves, grass and hair with the in-game anti-aliasing (MSAA) on.
# No measurable cost. Takes effect live.
msaa_prepass_fix = true

# Removes the bright rim around shadowed characters and objects with the in-game anti-aliasing
# (MSAA) on. The game averages the scene depth and the shadow mask before the shadow receiver and
# the materials read them, so at every silhouette the shadowed side picks up light from the
# background. With this on they read the multisampled textures per sample. No measurable cost.
# Takes effect live.
msaa_shadow_mask_fix = true

# Increases performance with the in-game anti-aliasing (MSAA) on (EXPERIMENTAL). The game shades
# every material once per sample, so 8x MSAA shades every model eight times. With this on the
# materials shade once per pixel and MSAA only smooths the edges. Needs msaa_shadow_mask_fix.
# Takes effect live.
msaa_per_pixel = false

# Extends msaa_per_pixel to the terrain materials, the biggest part of the gain (EXPERIMENTAL).
# Wet ground can flicker bright for a moment, stronger than vanilla. Takes effect live.
msaa_per_pixel_terrain = false

# Stops ambient occlusion (AO) from showing through walls and jumping when a model switches LOD.
# During a LOD cross-fade the game draws both LOD versions solid into the AO depth, so AO appears
# for geometry you can barely see. This skips that extra draw. lod_multiplier = 0 also hides the
# bug because there are no LOD switches at all; this fix removes the cause and keeps LODs on.
# No cost. Takes effect live.
ao_fade_fix = true

# The next two only help when the CPU is the bottleneck. If your CPU is slower than your GPU,
# both together can bring up to 20% more performance; on a GPU-bound system they change nothing.

# Increases performance. Fixes vanilla re-sending shader parameters to the graphics card every
# frame even when they did not change. Vanilla has the check but never reaches it. Takes effect
# live.
constant_buffer_dedup = false

# Increases performance. Fixes vanilla sending every shader parameter block to the graphics card
# each frame, including the ones nothing on screen uses. A block is only sent once a draw binds
# it. Takes effect live.
constant_buffer_upload_on_bind = false

# Less stutter while the map loads. When a new part of the map loads around you, the game builds
# everything for it in one go, which shows as a hitch every time. With this on, that work is
# spread over several frames and prepared in the background, so most of those hitches
# disappear. Work in progress, more improvements will follow. You will still notice small hitches
# here and there while the map loads, for example when effects or new objects appear. Do not
# expect a higher frame rate, faster loading, or stutter-free loading with 4K texture packs.
# Takes effect live.
decrease_stutter_during_grid_loading = false

# Internal render scale (EXPERIMENTAL). 1.0 = native. 2.0 renders the game at twice the
# display resolution and scales back down, 0.5 at half of it and scales up. Range 0.5..=4.0,
# GPU load follows the factor squared. Works in every display mode. Takes effect live, but the
# swap chain rebuild can take a moment and the frame rate may drop until it is done; if it does
# not apply, restart the game.
render_scale = 1.0

# High-res map grids (EXPERIMENTAL, WORK IN PROGRESS). Vanilla keeps 7 high-res grids loaded
# around you, the rest of the map is low-res. This keeps more of them loaded. Costs memory,
# loading time and frame rate, since the extra grids are also rendered. Needs a restart.
#
# Known issue, this feature is not finished: the culling rules for the higher ring counts
# still have to be written. Until they are, hidden_meshes / blocked_in_room /
# blocked_from_grid below are mostly empty, so far grids can show map parts that belong
# somewhere else.
#
# Contributions welcome: the rule lists below are the part that needs filling in. Meshes and
# grid ids can be picked directly in-game with the YP Devkit (Map Manager -> Mesh Picker) and
# written straight into a rule.
[high_grids]
# The launcher turns disable_manual_culling on together with this, and back off with it.
enabled = false
# Hex rings of grids around you: 1 = vanilla (7 grids), 2 = 19, 3 = 37, 4 = 61.
# Every ring costs memory, loading time and frame rate - more geometry stays loaded
# and is drawn, so the GPU cost grows with the ring count. 4 is the tested value.
rings = 1

# Room ids: 0x100 Ruined City, 0x110 Resistance Camp, 0x120 Factory, 0x130 Amusement Park,
# 0x140 Pascal's Village, 0x150 Desert, 0x160 Forest, 0x170 Flooded City, 0x200 City Ruins
# (route B), 0x520 Underground. Grid ids are the low 4 hex digits of a grid number (the gXXXXX
# in a map file name, without the first digit), for example 0x1121.

# Fewer rings in a room, for closed areas where the far grids show map parts from elsewhere.
room_rings = []

# Grids that never load while you are in a room.
blocked_in_room = []

# Grids that never load while you stand on a specific grid.
blocked_from_grid = []

# Meshes hidden once enough rings are loaded. A grid draws low-detail stand-ins for map
# parts it expects to be missing; with more rings those parts are really there and the
# stand-in sits inside them. grid = 0x0 applies to every grid. Find names with the
# mesh picker in the YP Devkit.
hidden_meshes = [
    # { mesh = "nolowmap_komono1", from_rings = 2, grid = 0x0 },
]
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
content_effect_areas = true
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
disable_game_mods_loading = false
disable_texture_injection = false

# Skip the startup splash window shown while the game loads. Vanilla revealed
# the game window before it was ready, causing resize/flicker artifacts during
# startup. NAMS finished the implementation so the splash covers startup and
# only reveals the window once ready. Set to true to bring the artifacts back.
disable_splash_screen = false

# Skip the Platinum Games / Square Enix logo movies played while the game
# loads. Loading itself is untouched; the game already lets you skip them
# with a button press.
skip_startup_logos = false

fix_wind_timer_bug = true

# Ignore the leftover developer hotkeys Shift+3/6/K/L, which force every render
# target to 1600x900 and break the look until you restart.
disable_debug_hotkeys = true

# Master switch for the mouse and keyboard layer. When true, all mouse and
# keybind settings are ignored and the game keeps stock input. Requires restart.
disable_input_features = false

[mouse]
fix_camera_acceleration = false
sensitivity = 2.0
third_person_mode = false
third_person_char_follow = false
third_person_smoothing = 0.0
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
hide_subtitle_overlay = false
keep_subtitle_text = false
hide_subtitle_in_events = false
solid_letterbox_bars = false

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
# Camera easing for third_person_mode: 0.0 = follows the mouse without delay,
# 1.0 = the game's own smoothing, in between blends the two.
third_person_smoothing = 0.0
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
# Hide the black bars and caption overlay shown during cutscenes. Only affects
# cutscenes; menus, item descriptions and normal dialogue are untouched.
hide_subtitle_overlay = false
# Keep the caption text and remove only the dark backdrop behind it.
keep_subtitle_text = false
# EXPERIMENTAL. Also apply during in-engine event scenes, not just pre-rendered
# movies. May miss scenes or hide text you wanted to keep.
hide_subtitle_in_events = false
# Draw the letterbox as solid black bars of equal height instead of vanilla's
# translucent, uneven ones. Wins over hide_subtitle_overlay when both are set.
solid_letterbox_bars = false
''';

  static const _newCutsceneKeyBlocks = <String, String>{
    'hide_subtitle_overlay':
        '# Hide the black bars and caption overlay shown during cutscenes. Only\n'
        '# affects cutscenes; menus and normal dialogue are untouched.\n'
        'hide_subtitle_overlay = false',
    'keep_subtitle_text':
        '# Keep the caption text and remove only the dark backdrop behind it.\n'
        'keep_subtitle_text = false',
    'hide_subtitle_in_events':
        '# EXPERIMENTAL. Also apply during in-engine event scenes, not just\n'
        '# pre-rendered movies. May miss scenes or hide text you wanted to keep.\n'
        'hide_subtitle_in_events = false',
    'solid_letterbox_bars':
        '# Draw the letterbox as solid black bars of equal height instead of\n'
        "# vanilla's translucent, uneven ones. Wins over hide_subtitle_overlay.\n"
        'solid_letterbox_bars = false',
  };

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
    'third_person_smoothing':
        '# Camera easing for third_person_mode: 0.0 = follows the mouse without delay,\n'
        '# 1.0 = the game\'s own smoothing, in between blends the two.\n'
        'third_person_smoothing = 0.0',
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
    _migrateOverlaySettingsSync(gameDir, namsDir.path);
  }

  static void _migrateOverlaySettingsSync(String gameDir, String namsDirPath) {
    final target = File(
      path.join(namsDirPath, '_internal', 'cache', 'settings.json'),
    );
    if (target.existsSync()) return;

    for (final root in _legacySettingsRoots(gameDir)) {
      final old = File(path.join(root, 'NAMS', 'settings.json'));
      if (!old.existsSync()) continue;

      target.parent.createSync(recursive: true);
      target.writeAsStringSync(withImpellerOff(old.readAsStringSync()));
      old.deleteSync();
      return;
    }
  }

  static String withImpellerOff(String raw) {
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return raw;
      if (decoded[NamsSettingsService.impellerKey] != true) return raw;
      decoded[NamsSettingsService.impellerKey] = false;
      return const JsonEncoder.withIndent('  ').convert(decoded);
    } catch (_) {
      return raw;
    }
  }

  static List<String> _legacySettingsRoots(String gameDir) {
    final appData = Platform.environment['APPDATA'] ?? '';
    if (Platform.isWindows) {
      return appData.isEmpty ? const [] : [appData];
    }

    final prefixes = <String>[];
    final containing = inferWinePrefixFromPath(gameDir);
    if (containing != null) prefixes.add(containing);

    final steam = inferSteamContext(gameDir);
    if (steam != null) prefixes.add(path.join(steam.compatDataPath, 'pfx'));

    return [
      for (final prefix in prefixes) getWineRoamingPath(prefix),
      if (appData.isNotEmpty) appData,
    ];
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
      NamsFields.contentEffectAreas.key,
      NamsFields.contentQuestIntegration.key,
      NamsFields.contentEffectsApplier.key,
      NamsFields.contentEquipTracker.key,
      NamsFields.contentMcd.key,
      NamsFields.contentBuddyRubySelector.key,
    ];
    final missingContent = contentToggleKeys
        .where((k) => !content.contains(k))
        .toList();
    if (missingContent.isNotEmpty) {
      // Insert before any [section] header so new keys land at the top level.
      final firstSectionIdx = content.indexOf(RegExp(r'^\[', multiLine: true));
      final block = StringBuffer();
      if (!content.endsWith('\n')) block.write('\n');
      block.writeln();
      block.writeln(
        '# Per-feature content-layer toggles. All default to true. Useful for bisecting',
      );
      block.writeln('# mod bugs without disabling the entire content layer.');
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
      NamsFields.disableGameModsLoading.key:
          '# Skip loading third-party game-offset mods from thirdparty/game/.\n'
          '# These are DLLs built for a stock NieRAutomata.exe; NAMS rewrites\n'
          '# their imports so game.bin answers as the main module. Managed by\n'
          '# the Third Party tab. Per-DLL tweaks: thirdparty/game/game.toml\n'
          'disable_game_mods_loading = false\n',
      NamsFields.loadingStallHints.key:
          '# Show escalating hints when the "Loading Map" screen takes too long.\n'
          'loading_stall_hints = true\n',
      NamsFields.fixWindTimerBug.key:
          '# Fix the vanilla bug where wind animation stops after max playtime.\n'
          'fix_wind_timer_bug = true\n',
      NamsFields.disableDebugHotkeys.key:
          '# Ignore the leftover developer hotkeys Shift+3/6/K/L, which force every\n'
          '# render target to 1600x900 and break the look until you restart.\n'
          'disable_debug_hotkeys = true\n',
      NamsFields.disableInputFeatures.key:
          '# Master switch for the mouse and keyboard layer. When true, all mouse\n'
          '# and keybind settings are ignored and the game keeps stock input. Restart.\n'
          'disable_input_features = false\n',
      NamsFields.skipStartupLogos.key:
          '# Skip the Platinum Games / Square Enix logo movies played while the\n'
          '# game loads. Loading itself is untouched; the game already lets you\n'
          '# skip them with a button press.\n'
          'skip_startup_logos = false\n',
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
    } else {
      final missingCutscene = <String>[];
      for (final entry in _newCutsceneKeyBlocks.entries) {
        final re = RegExp('^${entry.key}\\s*=', multiLine: true);
        if (!re.hasMatch(content)) {
          missingCutscene.add(entry.value);
        }
      }
      if (missingCutscene.isNotEmpty) {
        final afterCutscene =
            content.indexOf('[cutscene]') + '[cutscene]'.length;
        final nextSection = content.indexOf(
          RegExp(r'^\[', multiLine: true),
          afterCutscene,
        );
        final block = '${missingCutscene.join('\n')}\n';
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

    final hasLegacyPreload =
        content.contains('preload_max_dimension') ||
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

    if (!content.contains(TextureInjectionFields.hotReload.key)) {
      content += '''

# Watch the texture folders and apply changed .dds files while the game runs.
hot_reload = true
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
