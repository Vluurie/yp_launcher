import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:toml/toml.dart';
import 'package:yp_launcher/constants/naiom_keys.dart';
import 'package:yp_launcher/services/isolate_service.dart';
import 'package:yp_launcher/services/lodmod_config_service.dart';
import 'package:yp_launcher/services/nams_settings_service.dart';
import 'package:yp_launcher/services/toml_service.dart';
import 'package:yp_launcher/services/wine/proton.dart';
import 'package:yp_launcher/services/wine/wine_paths.dart';
import 'package:yp_launcher/models/config_fields.dart';

class NamsConfigService {
  static Map<String, String> get shippedTemplates => {
    'nams.toml': _namsToml,
    'naiom.toml': _naiomToml,
    'lodmod.toml': _lodmodToml,
    'texture_injection.toml': _textureInjectionToml,
  };

  static const _lodmodToml =
      '''# NAMS LodMod Ext - Visual quality improvements. Inspired by Automata-LodMod by
# emoose and extended well past it. Most settings apply live while the game is
# running; the ones that need a restart say so.

# Enable/disable all LodMod Ext patches
enabled = false

# 0 = best quality (no LOD pop-in), 1 = vanilla, higher = further LOD distance
lod_multiplier = 0.0

# How far from you objects placed on the map (grass, bushes, rubble, small props) appear, in
# meters. 100 = vanilla. Lower values save frame rate, these objects then appear closer to you.
# Higher values fill the view further out and cost frame rate in areas with a lot of grass. Takes
# effect live for objects that are not placed yet; objects already placed stay until their part of
# the map unloads.
layout_spawn_range = 100.0

# Places all of these objects as soon as their part of the map is loaded, however far away you
# are. Ignores layout_spawn_range. Takes effect live for objects that are not placed yet.
layout_spawn_everything = false

# Stops grass, bushes and props from vanishing and reappearing when the camera moves. The map
# marks areas in which some objects are hidden while the camera is inside them, because from there
# they were meant to be out of sight. With a longer view distance they are not, and you see them
# pop. With this on they stay visible, which costs some frame rate. Takes effect live.
disable_camera_culling = false

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
# Advanced: even shadow distribution distance. 0 = off, try ~1500 for large open areas
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
# It also turns disable_camera_culling on if it is not on yet.
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
    { mesh = "nolowmap_enkei", from_rings = 2, grid = 0x1121 },
    { mesh = "nolowmap_seaplane", from_rings = 2, grid = 0x0922 },
    # { mesh = "nolowmap_komono1", from_rings = 2, grid = 0x0 },
]
''';

  static final String _namsToml =
      '''# NAMS General Config. Restart game after changes.

validate_model_data = false
validate_scripts = false
loading_stall_hints = true
disable_plugin_loading = false
disable_content_features = false

# Skip starting the main Flutter overlay, the in-game mod UI that also hosts the
# YoRHa Protocol UI. Plugins and every NAMS feature still run; only the overlay
# and its panels, the YoRHa Protocol UI included, are gone, together with the
# memory they use. Requires restart.
disable_main_overlay = false

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

# Skip loading Direct3D proxies from thirdparty/graphic/: DLLs named like a
# system DLL (d3d11.dll, dxgi.dll, ...) that the game's calls have to pass
# through. NAMS points its own and the game's imports of that DLL at the mod.
disable_graphic_mods_loading = false

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

${_contentSectionSync(const {})}
[cutscene]
enable_h264 = false
hide_subtitle_overlay = false
keep_subtitle_text = false
hide_subtitle_in_events = false
solid_letterbox_bars = false
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

# Load the textures of the built-in priority list (the GPUnity HD Pack) at startup and
# keep them in VRAM for the whole session, so they never pop in.
# Set to false on GPUs with 4 GB VRAM or less: they then go through vram_budget_mb like
# every other texture and get unloaded when the game needs the memory.
pin_priority_textures = true

# Load order for texture pack folders inside nams/inject/textures/.
# Earlier entries win when the same texture hash exists in multiple packs.
# Loose .dds files directly in nams/inject/textures/ win over the packs below.
#
# Example (Androids Remastered wins over GPUnity HD Pack):
#   load_order = ["Androids Remastered", "GPUnity HD Pack"]
load_order = []

# Pack folder names to skip during scanning. The launcher writes this list
# to disable specific packs per profile without moving folders on disk.
disabled_packs = []

# Watch the texture folders and apply changed .dds files while the game runs.
# Same-size changes apply instantly, size/format changes on the next scene load.
hot_reload = true
''';

  static const _naiomToml =
      '''# NAIOM: mouse and keyboard settings. Ignored while disable_input_features is
# set in nams.toml.

# Remove the deadzone and acceleration curve from camera rotation.
fix_camera_acceleration = false
# Sensitivity multiplier for fix_camera_acceleration. Higher = faster rotation.
sensitivity = 2.0
# Raw-input third-person camera: the mouse also turns the camera by its raw
# counts, and the camera turn by mouse, keys and gamepad has no deadzone.
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

# Additional key bindings. All bindings are additive - the vanilla keys keep
# working. "" = unbound. Supported key names: A-Z, 0-9, SPACE, END, UPARROW,
# DOWNARROW, LEFTARROW, RIGHTARROW, LSHIFT, RSHIFT, LCTRL, RCTRL, MOUSE1-MOUSE5.
# Combinations use +, e.g. "CTRL+X". SHIFT, CTRL and ALT match either side
# and are modifier-only.
[bindings]
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

  static const _contentHeaderComment =
      '# Every content feature on its own. false turns it off: no mod entry of that\n'
      '# feature is loaded, and neither is any mod entry that needs it, e.g. a shop\n'
      '# entry selling a modded outfit while custom_outfits is off.\n'
      '# disable_content_features turns all of them off at once. Requires restart.\n';

  static const _contentKeyComments = <String, String>{
    'custom_items':
        '# New items from mods (consumables, materials, key items) with their icons and\n'
        '# texts, and items mods hand out automatically.',
    'custom_weapons':
        '# New weapons from mods with their stats, upgrades, texts and story images.',
    'weapon_patches': '# Mod changes to existing weapons.',
    'custom_outfits':
        '# New outfits from mods, WAX outfit mods, and the "Default Outfit" item that\n'
        '# takes a modded outfit off again.',
    'custom_hairsprays': '# New hairsprays from mods.',
    'accessories':
        '# New accessories from mods, and wearing several accessories at once.',
    'player_mesh_swaps':
        '# NAMS puts the 2B, 9S and A2 models together itself, so outfit and accessory\n'
        "# mods can swap, hide or add model parts. false = the game's own model setup.",
    'mesh_visibility': '# Mods that hide or show single parts of a model.',
    'shop_items': '# Items that mods add to shops.',
    'item_shortcut_menu':
        '# Modded healing items you own show up in the quick-item menu.',
    'effect_rules':
        '# What modded weapons, outfits, accessories and items do to the player, such as\n'
        '# damage, dodging, immunities and flashlight charge.',
    'outfit_selector_npc':
        '# The outfit selection NPC that offers modded outfits.',
    'buddy_outfit_menu':
        '# EXPERIMENTAL. A "Change outfit" entry when talking to your buddy that lists\n'
        '# modded buddy outfits. Turn off if it clashes with other script mods.',
    'custom_quests':
        '# New quests from mods with their texts, rewards and conditions.',
    'custom_mails': '# New mails from mods.',
    'custom_voice_lines': '# New voice lines from mods.',
    'custom_maps':
        '# New map rooms and story phases from mods, with their Chapter Select entries\n'
        '# and save slot names.',
    'map_layouts':
        '# Mod rules for objects placed on the map, such as removing them or how far\n'
        '# away they appear.',
    'novels': '# New novels from mods in the Data menu.',
    'archives': '# New archives from mods in the Data menu.',
    'cutscene_replacements':
        '# Cutscene videos that mods replace, and in-game cutscenes that mods move.',
    'camera_settings': '# Mod settings for the player camera.',
    'script_overlays': "# Mod additions and changes to the game's scripts.",
    'script_groups':
        '# Mod rules that turn groups of game scripts on or off per story phase.',
    'ruby_patches': "# Mod edits to the game's Ruby scripts.",
    'custom_music': '# New background music from mods.',
    'ambient_sounds':
        '# Ambient sounds and crowd voices that mods add or mute.',
    'custom_sounds': '# New sound effects from mods.',
    'effect_areas': '# Effects that mods place in rooms or remove from them.',
    'effect_type_mute':
        '# Mods that mute whole effect types in a room or story phase.',
    'global_effects':
        '# Effects that mods run across a whole room or story phase.',
    'lighting_filters': '# Mod color and lighting filters for rooms.',
    'cloth_physics': '# Extra cloth physics parts on modded models.',
    'custom_texts':
        "# Mod texts in menus, such as item names and descriptions. false = the game's\n"
        '# own texts.',
    'ui_overrides':
        '# Menu and HUD layouts that mods replace. false = the shipped layouts.',
  };

  static const _legacyContentKeys = <String, List<String>>{
    'content_items': [
      'custom_items',
      'custom_weapons',
      'weapon_patches',
      'custom_outfits',
      'custom_hairsprays',
      'shop_items',
      'item_shortcut_menu',
    ],
    'content_accessories': ['accessories'],
    'content_assemble_meshes': ['player_mesh_swaps', 'mesh_visibility'],
    'content_effect_areas': [
      'effect_areas',
      'effect_type_mute',
      'global_effects',
    ],
    'content_quest_integration': [
      'custom_quests',
      'custom_mails',
      'custom_voice_lines',
      'custom_maps',
      'custom_music',
      'ambient_sounds',
      'custom_sounds',
      'script_groups',
      'lighting_filters',
      'outfit_selector_npc',
      'buddy_outfit_menu',
      'cutscene_replacements',
    ],
    'content_effects_applier': ['effect_rules'],
    'content_equip_tracker': [],
    'content_mcd': ['custom_texts'],
    'content_buddy_ruby_selector': ['buddy_outfit_menu'],
  };

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

    final evadeName = _migrateNamsTomlSync(
      path.join(namsDir.path, 'nams.toml'),
    );
    _ensureNaiomTomlSync(namsDir.path, evadeName);
    final lodmodFile = File(path.join(namsDir.path, 'lodmod.toml'));
    final lodmodRaw = lodmodFile.readAsStringSync();
    final migratedLodmod = LodmodConfigService.migrate(lodmodRaw);
    if (migratedLodmod != lodmodRaw) {
      lodmodFile.writeAsStringSync(migratedLodmod);
    }
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

  static void _ensureNaiomTomlSync(String namsDirPath, String? evadeName) {
    final file = File(path.join(namsDirPath, 'naiom.toml'));
    if (file.existsSync()) return;
    final nams = File(path.join(namsDirPath, 'nams.toml'));
    final parsed = nams.existsSync()
        ? TomlService.parse(nams.readAsStringSync())['mouse']
        : null;
    final legacy = parsed is Map<String, dynamic>
        ? Map<String, dynamic>.from(parsed)
        : <String, dynamic>{};
    if (evadeName != null) {
      final evadeKey = NamsBindingFields.nonStandardEvade.key;
      final rawBindings = legacy['bindings'];
      final bindings = rawBindings is Map<String, dynamic>
          ? Map<String, dynamic>.from(rawBindings)
          : <String, dynamic>{};
      final current = bindings[evadeKey];
      if (current is! String || current.isEmpty) {
        bindings[evadeKey] = evadeName;
        legacy['bindings'] = bindings;
      }
    }
    file.writeAsStringSync(
      legacy.isEmpty ? _naiomToml : TomlService.updateToml(_naiomToml, legacy),
    );
  }

  static void _writeIfMissingSync(String filePath, String content) {
    final file = File(filePath);
    if (!file.existsSync()) {
      file.writeAsStringSync(content);
    }
  }

  static String? _migrateNamsTomlSync(String filePath) {
    final file = File(filePath);
    if (!file.existsSync()) return null;

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
        content = _insertTopLevelSync(
          content,
          'disable_content_features = false\n',
        );
        modified = true;
      }
    }

    final topLevelAdditions = <String, String>{
      NamsFields.disableMainOverlay.key:
          '# Skip starting the main Flutter overlay, the in-game mod UI that also\n'
          '# hosts the YoRHa Protocol UI. Plugins and every NAMS feature still run;\n'
          '# only the overlay and its panels, the YoRHa Protocol UI included, are\n'
          '# gone, together with the memory they use. Requires restart.\n'
          'disable_main_overlay = false\n',
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
      NamsFields.disableGraphicModsLoading.key:
          '# Skip loading Direct3D proxies from thirdparty/graphic/: DLLs named\n'
          '# like a system DLL (d3d11.dll, dxgi.dll, ...) that the game\'s calls\n'
          '# have to pass through. Requires restart.\n'
          'disable_graphic_mods_loading = false\n',
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

    final withContent = _migrateContentSectionSync(content);
    if (withContent != content) {
      content = withContent;
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
    return migratedEvadeName;
  }

  static String _insertTopLevelSync(String content, String block) {
    final at = _firstTableStartSync(content);
    if (at == -1) {
      final prefix = content.endsWith('\n') || content.isEmpty ? '' : '\n';
      return '$content$prefix\n$block';
    }
    final before = content.substring(0, at);
    final gap =
        before.isEmpty || before.endsWith('\n\n') || before.endsWith('\n\r\n')
        ? ''
        : '\n';
    return '$before$gap$block\n${content.substring(at)}';
  }

  static int _firstTableStartSync(String content) {
    var lineStart = 0;
    var commentStart = -1;
    while (lineStart < content.length) {
      final newline = content.indexOf('\n', lineStart);
      final lineEnd = newline == -1 ? content.length : newline;
      final line = content.substring(lineStart, lineEnd).trim();
      if (line.startsWith('[')) {
        return commentStart == -1 ? lineStart : commentStart;
      }
      if (!line.startsWith('#')) {
        commentStart = -1;
      } else if (commentStart == -1) {
        commentStart = lineStart;
      }
      if (newline == -1) break;
      lineStart = newline + 1;
    }
    return -1;
  }

  static String _contentSectionSync(Set<String> off) {
    final out = StringBuffer(_contentHeaderComment)..writeln('[content]');
    for (final entry in _contentKeyComments.entries) {
      out
        ..writeln(entry.value)
        ..writeln('${entry.key} = ${!off.contains(entry.key)}');
    }
    return out.toString();
  }

  static String _migrateContentSectionSync(String content) {
    final parsed = TomlService.parseStrict(content);
    if (!parsed.ok) return content;
    final existing = parsed.values[NamsContentFields.section];
    final off = <String>{};
    if (existing == null) {
      for (final entry in _legacyContentKeys.entries) {
        if (parsed.values[entry.key] == false) off.addAll(entry.value);
      }
    }

    final result = _withoutLegacyContentKeysSync(content);
    if (existing == null) {
      return _insertTopLevelSync(result, _contentSectionSync(off));
    }
    if (existing is! Map) return result;

    final missing = [
      for (final entry in _contentKeyComments.entries)
        if (!existing.containsKey(entry.key))
          '${entry.value}\n${entry.key} = true\n',
    ];
    if (missing.isEmpty) return result;
    return _appendToTableSync(
      result,
      NamsContentFields.section,
      missing.join(),
    );
  }

  static String _withoutLegacyContentKeysSync(String content) {
    final spans = TomlService.sourceExpressions(content).toList();
    String source(int i) => content.substring(spans[i].start, spans[i].end);
    bool isComment(int i) =>
        spans[i].expression == null && source(i).trimLeft().startsWith('#');

    final drop = List<bool>.filled(spans.length, false);
    for (var i = 0; i < spans.length; i++) {
      final expression = spans[i].expression;
      if (expression is TomlTable) break;
      if (expression is! TomlKeyValuePair) continue;
      final key = expression.key.parts.map((part) => part.name).join('.');
      if (!_legacyContentKeys.containsKey(key)) continue;
      drop[i] = true;
      for (var j = i - 1; j >= 0 && isComment(j); j--) {
        drop[j] = true;
      }
    }
    if (!drop.contains(true)) return content;

    final out = StringBuffer();
    var lastKeptBlank = true;
    var afterDrop = false;
    for (var i = 0; i < spans.length; i++) {
      if (drop[i]) {
        afterDrop = true;
        continue;
      }
      final text = source(i);
      final blank = text.trim().isEmpty;
      if (blank && lastKeptBlank && afterDrop) continue;
      out.write(text);
      lastKeptBlank = blank;
      afterDrop = false;
    }
    return out.toString();
  }

  static String _appendToTableSync(String content, String table, String block) {
    final spans = TomlService.sourceExpressions(content).toList();
    String source(int i) => content.substring(spans[i].start, spans[i].end);
    bool isComment(int i) =>
        spans[i].expression == null && source(i).trimLeft().startsWith('#');
    String? tableName(int i) {
      final expression = spans[i].expression;
      if (expression is! TomlTable) return null;
      return expression.name.parts.map((part) => part.name).join('.');
    }

    var header = 0;
    while (header < spans.length && tableName(header) != table) {
      header++;
    }
    if (header == spans.length) return content;

    var end = header + 1;
    while (end < spans.length && spans[end].expression is! TomlTable) {
      end++;
    }
    if (end < spans.length) {
      while (end - 1 > header && isComment(end - 1)) {
        end--;
      }
    }
    while (end - 1 > header && source(end - 1).trim().isEmpty) {
      end--;
    }

    final at = spans[end - 1].end;
    final before = content.substring(0, at);
    final sep = before.endsWith('\n') ? '' : '\n';
    return '$before$sep$block${content.substring(at)}';
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

    if (!content.contains(TextureInjectionFields.pinPriorityTextures.key)) {
      content += '''

# Load the priority-list textures at startup and keep them in VRAM for the
# whole session. Set to false on GPUs with 4 GB VRAM or less.
pin_priority_textures = true
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
