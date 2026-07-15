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
        NamsFields.outfitSwapVisualEffects,
        NamsFields.disableReShadeLoading,
        NamsFields.disableTextureInjection,
        NamsFields.disableSplashScreen,
        NamsFields.fixWindTimerBug,
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
        NamsFields.disablePodPet,
        NamsFields.fixAimAcceleration,
        NamsFields.aimSensitivity,
        NamsFields.aimOutputMultiplier,
        NamsFields.debugMenuKey,
        NamsFields.evadeKey,
      ]) {
        expect(raw.contains(f.key), isTrue,
            reason: '${f.key} missing from the [mouse] fallback');
      }
    });
  });
}
