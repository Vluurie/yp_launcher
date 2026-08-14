import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

Directory _gameDirWith(String? namsToml) {
  final dir = Directory.systemTemp.createTempSync('yp_cfg_');
  if (namsToml != null) {
    final namsDir = Directory(p.join(dir.path, 'nams'))
      ..createSync(recursive: true);
    File(p.join(namsDir.path, 'nams.toml')).writeAsStringSync(namsToml);
  }
  return dir;
}

Map<String, dynamic> _readNams(Directory gameDir) {
  final raw =
      File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();
  return TomlService.parse(raw);
}

void main() {
  group('nams.toml matches the NAMS config surface', () {
    test('fresh config has every top-level key with NAMS defaults', () async {
      final gameDir = _gameDirWith(null);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);

      final expected = <ConfigField<bool>>[
        NamsFields.validateModelData,
        NamsFields.validateScripts,
        NamsFields.loadingStallHints,
        NamsFields.disablePluginLoading,
        NamsFields.disableContentFeatures,
        NamsFields.contentItems,
        NamsFields.contentAccessories,
        NamsFields.contentAssembleMeshes,
        NamsFields.contentQuestIntegration,
        NamsFields.contentEffectsApplier,
        NamsFields.contentEquipTracker,
        NamsFields.contentMcd,
        NamsFields.contentBuddyRubySelector,
        NamsFields.experimentalDefaultOutfits,
        NamsFields.outfitSwapVisualEffects,
        NamsFields.disableReShadeLoading,
        NamsFields.disable3dmigotoLoading,
        NamsFields.disableGameModsLoading,
        NamsFields.disableTextureInjection,
        NamsFields.disableSplashScreen,
        NamsFields.fixWindTimerBug,
        NamsFields.disableDebugHotkeys,
        NamsFields.disableInputFeatures,
      ];

      for (final f in expected) {
        expect(cfg.containsKey(f.key), isTrue,
            reason: '${f.key} missing from generated nams.toml');
        expect(cfg[f.key], f.defaultValue,
            reason: '${f.key} default does not match NAMS');
      }
    });

    test('migration adds new keys to an old config at top level', () async {
      final gameDir = _gameDirWith('''# NAMS General Config.

validate_model_data = false
loading_stall_hints = true
disable_plugin_loading = false

[mouse]
fix_camera_acceleration = false
sensitivity = 2.0
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);

      expect(cfg[NamsFields.outfitSwapVisualEffects.key], isTrue);
      expect(cfg[NamsFields.disableSplashScreen.key], isFalse);
      expect(cfg[NamsFields.disableContentFeatures.key], isFalse);
      expect(cfg[NamsFields.contentItems.key], isTrue);
      expect(cfg[NamsFields.disable3dmigotoLoading.key], isFalse,
          reason: 'disable_3dmigoto_loading migrated into an old config');
      expect(cfg[NamsFields.disableReShadeLoading.key], isFalse,
          reason: 'disable_reshade_loading migrated into an old config');
      expect(cfg[NamsFields.disableDebugHotkeys.key], isTrue,
          reason: 'disable_debug_hotkeys migrated into an old config');
      expect(cfg[NamsFields.disableInputFeatures.key], isFalse,
          reason: 'disable_input_features migrated into an old config');
      expect((cfg['cutscene'] as Map)[NamsFields.solidLetterboxBars.key],
          isFalse,
          reason: 'solid_letterbox_bars migrated into [cutscene]');

      expect((cfg['mouse'] as Map)['sensitivity'], 2.0,
          reason: 'existing [mouse] values must survive migration');
      expect(cfg.containsKey('sensitivity'), isFalse,
          reason: 'new top-level keys must not push mouse keys out of [mouse]');
      for (final key in [
        NamsFields.outfitSwapVisualEffects.key,
        NamsFields.disableSplashScreen.key,
      ]) {
        expect((cfg['mouse'] as Map).containsKey(key), isFalse,
            reason: '$key must not land inside [mouse]');
      }
    });

    test('solid_letterbox_bars lands in an existing [cutscene] section',
        () async {
      final gameDir = _gameDirWith('''validate_model_data = false

[cutscene]
hd_cutscenes = true
enable_h264 = true
hide_subtitle_overlay = true
keep_subtitle_text = true
hide_subtitle_in_events = false

[heap]
global_heap_extra = 0
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);
      final cutscene = cfg['cutscene'] as Map;

      expect(cutscene[NamsFields.solidLetterboxBars.key], isFalse);
      expect(cutscene['hd_cutscenes'], isTrue,
          reason: 'existing [cutscene] values must survive migration');
      expect(cutscene['hide_subtitle_overlay'], isTrue);
      expect(cfg.containsKey(NamsFields.solidLetterboxBars.key), isFalse,
          reason: 'solid_letterbox_bars must not land at top level');
      expect((cfg['heap'] as Map)['global_heap_extra'], 0,
          reason: 'the following [heap] section must stay intact');
    });

    test('migration keeps validate_scripts off, matching the NAMS default',
        () async {
      final gameDir = _gameDirWith('''validate_model_data = false
loading_stall_hints = true
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);

      expect(_readNams(gameDir)[NamsFields.validateScripts.key], isFalse);
    });

    test('migration preserves user values and is idempotent', () async {
      final gameDir = _gameDirWith('''validate_model_data = true
loading_stall_hints = false
outfit_swap_visual_effects = false
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final first =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();

      await NamsConfigService.ensureConfigs(gameDir.path);
      final second =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();

      expect(second, first, reason: 'second migration must be a no-op');

      final cfg = _readNams(gameDir);
      expect(cfg[NamsFields.validateModelData.key], isTrue);
      expect(cfg[NamsFields.loadingStallHints.key], isFalse);
      expect(cfg[NamsFields.outfitSwapVisualEffects.key], isFalse,
          reason: 'user opt-out must survive migration');
    });

    test('mouse fallback section carries every mouse field', () async {
      final gameDir = _gameDirWith('''validate_model_data = false
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final raw =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();

      for (final f in <ConfigField>[
        NamsFields.fixCameraAcceleration,
        NamsFields.sensitivity,
        NamsFields.thirdPersonMode,
        NamsFields.thirdPersonCharFollow,
        NamsFields.thirdPersonSensitivityX,
        NamsFields.thirdPersonSensitivityY,
        NamsFields.aimMode,
        NamsFields.aimCrosshair,
        NamsFields.aimCrosshairAlways,
        NamsFields.aimSensitivity,
        NamsFields.aimSensitivityX,
        NamsFields.aimSensitivityY,
        NamsFields.aimOutputMultiplier,
        NamsFields.movementDisableTapEvade,
        NamsFields.miscDisablePodPet,
        NamsFields.miscOpenDebugMenu,
        NamsFields.miscCustomCursorMenu,
        NamsFields.miscCustomCursorHacking,
        NamsFields.miscDisableDefaultCursor,
      ]) {
        expect(raw.contains(f.key), isTrue,
            reason: '${f.key} missing from the [mouse] fallback');
      }
      for (final f in NamsBindingFields.all) {
        expect(raw.contains(f.key), isTrue,
            reason: '${f.key} missing from the [mouse.bindings] fallback');
      }
    });
  });

  group('NAIOM key rename migration', () {
    const oldMouseConfig = '''# NAMS General Config. Restart game after changes.

validate_model_data = false
validate_scripts = false
loading_stall_hints = true
disable_plugin_loading = false
disable_content_features = false

content_items = true
content_accessories = true
content_assemble_meshes = true
content_quest_integration = true
content_effects_applier = true
content_equip_tracker = true
content_mcd = true
content_buddy_ruby_selector = true

outfit_swap_visual_effects = true
disable_reshade_loading = false
disable_texture_injection = false
disable_splash_screen = false
fix_wind_timer_bug = true

# Mouse input fixes.
[mouse]
# Remove the deadzone and acceleration curve from camera rotation.
fix_camera_acceleration = true
# Sensitivity multiplier. Higher = faster camera rotation.
sensitivity = 3.75
# Disable the pod petting animation that triggers when moving the mouse.
disable_pod_pet = true
# Remove the clamp and deadzone from pod/mech aiming.
fix_aim_acceleration = true
# Aim sensitivity for top-down/side-scroll.
aim_sensitivity = 0.002
# Raw multiplier applied to aim output after normalization.
aim_output_multiplier = 12.0
# Virtual key code for opening the debug menu. 0 = disabled.
debug_menu_key = 112
# Virtual key code for dedicated evade/dodge. 0 = disabled.
evade_key = 82

[cutscene]
hd_cutscenes = true
enable_h264 = false

[heap]
global_heap_extra = 0
''';

    test('old mouse keys are renamed with values preserved', () async {
      final gameDir = _gameDirWith(oldMouseConfig);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);
      final mouse = cfg['mouse'] as Map<String, dynamic>;

      expect(mouse['aim_mode'], isTrue,
          reason: 'fix_aim_acceleration value must move to aim_mode');
      expect(mouse['misc_disable_pod_pet'], isTrue,
          reason: 'disable_pod_pet value must move to misc_disable_pod_pet');
      expect(mouse['misc_open_debug_menu'], 112,
          reason: 'debug_menu_key value must move to misc_open_debug_menu');
      expect(mouse.containsKey('fix_aim_acceleration'), isFalse);
      expect(mouse.containsKey('disable_pod_pet'), isFalse);
      expect(mouse.containsKey('debug_menu_key'), isFalse);
      expect(mouse.containsKey('evade_key'), isFalse,
          reason: 'evade_key was never read by NAMS and must be removed');

      expect(mouse['sensitivity'], 3.75);
      expect(mouse['aim_sensitivity'], 0.002);
      expect(mouse['aim_output_multiplier'], 12.0);
      expect(mouse['fix_camera_acceleration'], isTrue);
    });

    test('evade_key VK code becomes non_standard_evade key name', () async {
      final gameDir = _gameDirWith(oldMouseConfig);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);
      final bindings =
          (cfg['mouse'] as Map<String, dynamic>)['bindings']
              as Map<String, dynamic>;

      expect(bindings['non_standard_evade'], 'R',
          reason: 'VK 82 (0x52) is the R key');
    });

    test('new mouse keys and all bindings are added inside their sections',
        () async {
      final gameDir = _gameDirWith(oldMouseConfig);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final raw =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();
      final cfg = TomlService.parse(raw);
      final mouse = cfg['mouse'] as Map<String, dynamic>;
      final bindings = mouse['bindings'] as Map<String, dynamic>;

      for (final key in [
        'third_person_mode',
        'third_person_char_follow',
        'third_person_sensitivity_x',
        'third_person_sensitivity_y',
        'aim_crosshair',
        'aim_crosshair_always',
        'aim_sensitivity_x',
        'aim_sensitivity_y',
        'movement_disable_tap_evade',
        'misc_custom_cursor_menu',
        'misc_custom_cursor_hacking',
        'misc_disable_default_cursor',
      ]) {
        expect(mouse.containsKey(key), isTrue,
            reason: '$key must be added to [mouse]');
      }
      for (final f in NamsBindingFields.all) {
        expect(bindings.containsKey(f.key), isTrue,
            reason: '${f.key} must exist in [mouse.bindings]');
      }
      expect(raw.indexOf('third_person_mode = false'),
          lessThan(raw.indexOf('[cutscene]')),
          reason: 'new mouse keys must land inside [mouse], not at EOF');
      expect(raw, contains('# Remove the deadzone and acceleration curve'));
      expect((cfg['cutscene'] as Map)['hd_cutscenes'], isTrue);
    });

    test('rename migration is idempotent', () async {
      final gameDir = _gameDirWith(oldMouseConfig);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final first =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();
      await NamsConfigService.ensureConfigs(gameDir.path);
      final second =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();

      expect(second, first);
    });

    test('subtitle keys are added to an existing [cutscene] section', () async {
      final gameDir = _gameDirWith(oldMouseConfig);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);
      final raw =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();
      final cutscene = cfg['cutscene'] as Map<String, dynamic>;

      for (final f in [
        NamsFields.hideSubtitleOverlay,
        NamsFields.keepSubtitleText,
        NamsFields.hideSubtitleInEvents,
      ]) {
        expect(cutscene.containsKey(f.key), isTrue,
            reason: '${f.key} must be added to [cutscene]');
        expect(cutscene[f.key], f.defaultValue);
      }

      expect(cutscene['hd_cutscenes'], isTrue,
          reason: 'existing auto-detected value must survive');
      expect(raw.indexOf('hide_subtitle_overlay'),
          lessThan(raw.indexOf('[heap]')),
          reason: 'new keys must land inside [cutscene], not at EOF');
    });

    test('a config with no [cutscene] section gains the full section',
        () async {
      final gameDir = _gameDirWith('validate_model_data = false\n');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cutscene =
          _readNams(gameDir)['cutscene'] as Map<String, dynamic>;

      expect(cutscene['hide_subtitle_overlay'], isFalse);
      expect(cutscene['keep_subtitle_text'], isFalse);
      expect(cutscene['hide_subtitle_in_events'], isFalse);
    });

    test('game mods flag is added to an old config', () async {
      final gameDir = _gameDirWith('''validate_model_data = false
disable_reshade_loading = false
disable_3dmigoto_loading = false

[mouse]
sensitivity = 2.0
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);
      final raw =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();

      expect(cfg[NamsFields.disableGameModsLoading.key], isFalse);
      expect(raw.indexOf('disable_game_mods_loading'),
          lessThan(raw.indexOf('[mouse]')),
          reason: 'must be inserted as a top-level key, not into [mouse]');
    });

    test('cutscene migration is idempotent', () async {
      final gameDir = _gameDirWith(oldMouseConfig);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final first =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();
      await NamsConfigService.ensureConfigs(gameDir.path);
      final second =
          File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();

      expect(second, first);
    });

    test('unbindable evade_key VK is dropped, binding stays empty', () async {
      final gameDir = _gameDirWith('''[mouse]
evade_key = 112
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);
      final bindings =
          (cfg['mouse'] as Map<String, dynamic>)['bindings']
              as Map<String, dynamic>;

      expect(bindings['non_standard_evade'], '',
          reason: 'VK 112 (F1) has no NAMS key name and must not be migrated');
    });
  });
}
