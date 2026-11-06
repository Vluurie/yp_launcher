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

String _rawNams(Directory gameDir) =>
    File(p.join(gameDir.path, 'nams', 'nams.toml')).readAsStringSync();

String _rawNaiom(Directory gameDir) =>
    File(p.join(gameDir.path, 'nams', 'naiom.toml')).readAsStringSync();

Map<String, dynamic> _readNaiom(Directory gameDir) =>
    TomlService.parse(_rawNaiom(gameDir));

Map<String, dynamic> _content(Map<String, dynamic> cfg) =>
    cfg['content'] as Map<String, dynamic>;

const _legacyContentKeys = [
  'content_items',
  'content_accessories',
  'content_assemble_meshes',
  'content_effect_areas',
  'content_quest_integration',
  'content_effects_applier',
  'content_equip_tracker',
  'content_mcd',
  'content_buddy_ruby_selector',
];

const _legacyMapping = <String, Set<String>>{
  'content_items': {
    'custom_items',
    'custom_weapons',
    'weapon_patches',
    'custom_outfits',
    'custom_hairsprays',
    'shop_items',
    'item_shortcut_menu',
  },
  'content_accessories': {'accessories'},
  'content_assemble_meshes': {'player_mesh_swaps', 'mesh_visibility'},
  'content_effect_areas': {'effect_areas', 'effect_type_mute', 'global_effects'},
  'content_quest_integration': {
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
  },
  'content_effects_applier': {'effect_rules'},
  'content_equip_tracker': {},
  'content_mcd': {'custom_texts'},
  'content_buddy_ruby_selector': {'buddy_outfit_menu'},
};

String _legacyConfig(Map<String, bool> overrides) {
  final lines = [
    for (final key in _legacyContentKeys) '$key = ${overrides[key] ?? true}',
  ];
  return '''# NAMS General Config. Restart game after changes.

validate_model_data = false
validate_scripts = false
loading_stall_hints = true
disable_plugin_loading = false
disable_content_features = false

# Per-feature content-layer toggles. All default to true. Useful for bisecting
# mod bugs without disabling the entire content layer.
${lines.join('\n')}

outfit_swap_visual_effects = true
experimental_default_outfits = false

# Mouse input fixes.
[mouse]
sensitivity = 2.0

[cutscene]
hd_cutscenes = true
''';
}

void _expectContentOff(Map<String, dynamic> cfg, Set<String> off) {
  final content = _content(cfg);
  for (final f in NamsContentFields.all) {
    expect(
      content[f.key],
      !off.contains(f.key),
      reason: '${f.key} should be ${off.contains(f.key) ? 'off' : 'on'}',
    );
  }
}

void main() {
  group('nams.toml [content] table', () {
    test('NamsContentFields lists the 35 NAMS keys once each', () {
      final keys = NamsContentFields.all.map((f) => f.key).toList();
      expect(keys, hasLength(35));
      expect(keys.toSet(), hasLength(35));
      for (final f in NamsContentFields.all) {
        expect(f.section, 'content');
        expect(f.defaultValue, isTrue);
        expect(f.restartRequired, isTrue);
      }
    });

    test('fresh config has [content] with every key on and no old keys',
        () async {
      final gameDir = _gameDirWith(null);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);

      expect(
        _content(cfg).keys.toSet(),
        NamsContentFields.all.map((f) => f.key).toSet(),
      );
      _expectContentOff(cfg, const {});
      for (final key in _legacyContentKeys) {
        expect(_rawNams(gameDir), isNot(contains(key)),
            reason: '$key must not be written any more');
      }
    });

    test('old content_items and content_quest_integration convert to [content]',
        () async {
      final gameDir = _gameDirWith(_legacyConfig({
        'content_items': false,
        'content_quest_integration': false,
      }));
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final raw = _rawNams(gameDir);
      final cfg = TomlService.parse(raw);

      _expectContentOff(cfg, {
        ..._legacyMapping['content_items']!,
        ..._legacyMapping['content_quest_integration']!,
      });

      for (final key in _legacyContentKeys) {
        expect(cfg.containsKey(key), isFalse, reason: '$key must be removed');
        expect(raw, isNot(contains(key)), reason: '$key line must be gone');
      }
      expect(raw, isNot(contains('Per-feature content-layer toggles')),
          reason: 'the old comment block must be removed too');

      final contentAt = raw.indexOf('[content]');
      for (final entry in cfg.entries) {
        if (entry.value is Map) continue;
        expect(raw.indexOf('${entry.key} ='), lessThan(contentAt),
            reason: 'top-level ${entry.key} must stay above [content]');
      }
      expect(contentAt, lessThan(raw.indexOf('# Mouse input fixes.')),
          reason: '[content] goes in front of [mouse] and its comment');
      expect((cfg['mouse'] as Map)['sensitivity'], 2.0);
      expect((cfg['cutscene'] as Map)['hd_cutscenes'], isTrue);
    });

    for (final entry in _legacyMapping.entries) {
      test('${entry.key} = false turns off exactly what NAMS turns off',
          () async {
        final gameDir = _gameDirWith(_legacyConfig({entry.key: false}));
        addTearDown(() => gameDir.deleteSync(recursive: true));

        await NamsConfigService.ensureConfigs(gameDir.path);

        _expectContentOff(_readNams(gameDir), entry.value);
      });
    }

    test('old keys that are not literally false keep the feature on',
        () async {
      final gameDir = _gameDirWith('''validate_model_data = false
content_items = "false"
content_mcd = 0

[mouse]
sensitivity = 2.0
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = _readNams(gameDir);

      _expectContentOff(cfg, const {});
      expect(cfg.containsKey('content_items'), isFalse);
      expect(cfg.containsKey('content_mcd'), isFalse);
    });

    test('existing [content] keeps user values and ignores leftover old keys',
        () async {
      final gameDir = _gameDirWith('''validate_model_data = false
content_items = false
content_mcd = false

# Content features.
[content]
custom_items = true
custom_texts = true
novels = false

# Mouse input fixes.
[mouse]
sensitivity = 2.0
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final raw = _rawNams(gameDir);
      final cfg = TomlService.parse(raw);

      expect(cfg.containsKey('content_items'), isFalse);
      expect(cfg.containsKey('content_mcd'), isFalse);
      expect(raw, isNot(contains('content_items')));
      expect(raw, isNot(contains('content_mcd')));

      _expectContentOff(cfg, {'novels'});

      final contentAt = raw.indexOf('[content]');
      final mouseAt = raw.indexOf('# Mouse input fixes.');
      for (final f in NamsContentFields.all) {
        final at = raw.indexOf('\n${f.key} =');
        expect(at, greaterThan(contentAt), reason: '${f.key} below [content]');
        expect(at, lessThan(mouseAt), reason: '${f.key} above [mouse]');
      }
      expect(raw, contains('# Content features.\n[content]'),
          reason: 'the user comment stays on its table');
      expect((cfg['mouse'] as Map)['sensitivity'], 2.0);
    });

    test('content migration is idempotent', () async {
      for (final fixture in [
        null,
        _legacyConfig({'content_items': false, 'content_mcd': false}),
        '''content_items = false

[content]
novels = false
''',
      ]) {
        final gameDir = _gameDirWith(fixture);
        addTearDown(() => gameDir.deleteSync(recursive: true));

        await NamsConfigService.ensureConfigs(gameDir.path);
        final first = _rawNams(gameDir);
        await NamsConfigService.ensureConfigs(gameDir.path);

        expect(_rawNams(gameDir), first);
      }
    });

    test('saving a [content] toggle rewrites only that line', () async {
      final gameDir = _gameDirWith(null);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final raw = _rawNams(gameDir);
      final values = TomlService.parse(raw);
      values['content'] = {
        ..._content(values),
        NamsContentFields.novels.key: false,
      };

      final updated = TomlService.updateToml(raw, values);

      expect(
        updated.trimRight(),
        raw.replaceFirst('novels = true', 'novels = false').trimRight(),
      );
    });
  });

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
        NamsFields.experimentalDefaultOutfits,
        NamsFields.outfitSwapVisualEffects,
        NamsFields.disableReShadeLoading,
        NamsFields.disable3dmigotoLoading,
        NamsFields.disableGameModsLoading,
        NamsFields.disableGraphicModsLoading,
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
      expect(_content(cfg)[NamsContentFields.customItems.key], isTrue);
      expect(cfg[NamsFields.disable3dmigotoLoading.key], isFalse,
          reason: 'disable_3dmigoto_loading migrated into an old config');
      expect(cfg[NamsFields.disableReShadeLoading.key], isFalse,
          reason: 'disable_reshade_loading migrated into an old config');
      expect(cfg[NamsFields.disableDebugHotkeys.key], isTrue,
          reason: 'disable_debug_hotkeys migrated into an old config');
      expect(cfg[NamsFields.disableInputFeatures.key], isFalse,
          reason: 'disable_input_features migrated into an old config');
      expect(cfg[NamsFields.disableGraphicModsLoading.key], isFalse,
          reason: 'disable_graphic_mods_loading migrated into an old config');
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

    test('naiom.toml carries every mouse field', () async {
      final gameDir = _gameDirWith('''validate_model_data = false
''');
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final raw = _rawNaiom(gameDir);

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
            reason: '${f.key} missing from naiom.toml');
      }
      for (final f in NamsBindingFields.all) {
        expect(raw.contains(f.key), isTrue,
            reason: '${f.key} missing from naiom.toml [bindings]');
      }
    });
  });

  group('texture_injection.toml matches the NAMS config surface', () {
    Map<String, dynamic> readTextures(Directory gameDir) => TomlService.parse(
          File(p.join(gameDir.path, 'nams', 'texture_injection.toml'))
              .readAsStringSync(),
        );

    test('fresh config has every key with NAMS defaults', () async {
      final gameDir = _gameDirWith(null);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = readTextures(gameDir);

      final expected = <ConfigField>[
        TextureInjectionFields.vramBudgetMb,
        TextureInjectionFields.streamingEnabled,
        TextureInjectionFields.loadOnlyRelevant,
        TextureInjectionFields.pinPriorityTextures,
        TextureInjectionFields.loadOrder,
        TextureInjectionFields.disabledPacks,
        TextureInjectionFields.hotReload,
      ];
      for (final f in expected) {
        expect(cfg.containsKey(f.key), isTrue,
            reason: '${f.key} missing from generated texture_injection.toml');
        expect(cfg[f.key].toString(), f.defaultValue.toString(),
            reason: '${f.key} default does not match NAMS');
      }
    });

    test('migration adds pin_priority_textures and keeps user values',
        () async {
      final gameDir = _gameDirWith(null);
      addTearDown(() => gameDir.deleteSync(recursive: true));
      final namsDir = Directory(p.join(gameDir.path, 'nams'))
        ..createSync(recursive: true);
      File(p.join(namsDir.path, 'texture_injection.toml')).writeAsStringSync(
        'vram_budget_mb = 2048\n'
        'streaming_enabled = false\n'
        'load_only_relevant = true\n'
        'load_order = ["A", "B"]\n'
        'hot_reload = false\n',
      );

      await NamsConfigService.ensureConfigs(gameDir.path);
      final cfg = readTextures(gameDir);

      expect(cfg[TextureInjectionFields.pinPriorityTextures.key], isTrue);
      expect(cfg[TextureInjectionFields.vramBudgetMb.key], 2048);
      expect(cfg[TextureInjectionFields.streamingEnabled.key], isFalse);
      expect(cfg[TextureInjectionFields.loadOnlyRelevant.key], isTrue);
      expect(cfg[TextureInjectionFields.loadOrder.key], ['A', 'B']);
      expect(cfg[TextureInjectionFields.hotReload.key], isFalse);
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
      final bindings =
          _readNaiom(gameDir)['bindings'] as Map<String, dynamic>;

      expect(bindings['non_standard_evade'], 'R',
          reason: 'VK 82 (0x52) is the R key');
    });

    test('naiom.toml gets the old [mouse] values and every new key',
        () async {
      final gameDir = _gameDirWith(oldMouseConfig);
      addTearDown(() => gameDir.deleteSync(recursive: true));

      await NamsConfigService.ensureConfigs(gameDir.path);
      final raw = _rawNaiom(gameDir);
      final mouse = TomlService.parse(raw);
      final bindings = mouse['bindings'] as Map<String, dynamic>;

      for (final key in [
        'third_person_mode',
        'third_person_char_follow',
        'third_person_smoothing',
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
            reason: '$key must be in naiom.toml');
      }
      for (final f in NamsBindingFields.all) {
        expect(bindings.containsKey(f.key), isTrue,
            reason: '${f.key} must exist in naiom.toml [bindings]');
      }
      expect(raw.indexOf('third_person_mode = false'),
          lessThan(raw.indexOf('[bindings]')),
          reason: 'mouse keys must stay top-level, not inside [bindings]');
      expect(raw, contains('# Remove the deadzone and acceleration curve'));
      expect(mouse['sensitivity'], 3.75);
      expect(mouse['aim_mode'], isTrue);
      expect(mouse['misc_disable_pod_pet'], isTrue);
      expect(mouse['misc_open_debug_menu'], 112);
      expect(mouse['aim_sensitivity'], 0.002);
      expect(mouse['aim_output_multiplier'], 12.0);
      expect(mouse['fix_camera_acceleration'], isTrue);
      expect(mouse.containsKey('evade_key'), isFalse);
      expect((_readNams(gameDir)['cutscene'] as Map)['hd_cutscenes'], isTrue);
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
      final bindings =
          _readNaiom(gameDir)['bindings'] as Map<String, dynamic>;

      expect(bindings['non_standard_evade'], '',
          reason: 'VK 112 (F1) has no NAMS key name and must not be migrated');
    });
  });
}
