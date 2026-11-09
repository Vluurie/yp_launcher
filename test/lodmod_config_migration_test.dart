import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/providers/config_state.dart';
import 'package:yp_launcher/services/lodmod_config_service.dart';
import 'package:yp_launcher/services/nams_config_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

Map<String, dynamic> readConfig(String raw) {
  final parsed = TomlService.parseStrict(raw);
  expect(parsed.error, isNull);
  return parsed.values;
}

void main() {
  group('LodMod migration', () {
    test(
      'adds the current NAMS fields at root before indented commented tables',
      () {
        const raw = '''enabled = true
lod_multiplier = 0.75
custom_root = 42
  [ high_grids ] # keep my rules
enabled = true
rings = 3
room_rings = [
  { room = 0x170, rings = 1 },
]
hidden_meshes = []
custom_grid = "untouched"
''';
        final migrated = LodmodConfigService.migrate(raw);
        final config = readConfig(migrated);
        final grids = config['high_grids'] as Map;

        expect(config['enabled'], isTrue);
        expect(config['lod_multiplier'], 0.75);
        expect(config['custom_root'], 42);
        expect(config['render_scale'], 1.0);
        expect(config['shadow_cascades'], 4);
        expect(config['shadow_cascade_range'], 4.0);
        expect(config['shadow_blur_scale'], [1.0, 1.0, 1.0, 1.0]);
        expect(config['disable_hdr'], isFalse);
        expect(config['fxaa'], isFalse);
        expect(config['msaa_prepass_fix'], isTrue);
        expect(config['msaa_shadow_mask_fix'], isTrue);
        expect(config['msaa_per_pixel'], isFalse);
        expect(config['msaa_per_pixel_terrain'], isFalse);
        expect(config['ao_fade_fix'], isTrue);
        expect(config['ao_full_resolution'], isFalse);
        expect(config['constant_buffer_dedup'], isFalse);
        expect(config['constant_buffer_upload_on_bind'], isFalse);
        expect(config['decrease_stutter_during_grid_loading'], isFalse);
        expect(config['fps_cap_in_hacking'], isTrue);
        expect(config['fps_cap_in_events'], isTrue);
        expect(grids['enabled'], isTrue);
        expect(grids['rings'], 3);
        expect(grids['room_rings'], [
          {'room': 0x170, 'rings': 1},
        ]);
        expect(grids['hidden_meshes'], isEmpty);
        expect(grids['custom_grid'], 'untouched');
        expect(grids.containsKey('render_scale'), isFalse);
        expect(migrated, contains('# keep my rules'));
        expect(LodmodConfigService.migrate(migrated), migrated);
      },
    );

    test('an indented table header at EOF is not duplicated', () {
      final migrated = LodmodConfigService.migrate(
        'enabled = true\n  [high_grids]',
      );
      final config = readConfig(migrated);
      expect(config['enabled'], isTrue);
      expect(config['high_grids']['enabled'], isFalse);
      expect(config['high_grids']['rings'], 1);
      expect(
        config['high_grids']['hidden_meshes'],
        LodModFields.highGridsHiddenMeshes.defaultValue,
      );
    });

    test(
      'repairs misplaced root values and keeps explicit root values authoritative',
      () {
        const raw = '''enabled = false
fxaa = false
[high_grids]
enabled = true
rings = 2
fxaa = true
render_scale = 1.5
shadow_blur_scale = [
  0.5, 0.6, 0.7, 0.8,
]
constant_buffer_dedup = true
''';
        final migrated = LodmodConfigService.migrate(raw);
        final config = readConfig(migrated);
        final grids = config['high_grids'] as Map;
        expect(config['enabled'], isFalse);
        expect(config['fxaa'], isFalse);
        expect(config['render_scale'], 1.5);
        expect(config['shadow_blur_scale'], [0.5, 0.6, 0.7, 0.8]);
        expect(config['constant_buffer_dedup'], isTrue);
        expect(grids['enabled'], isTrue);
        expect(grids['rings'], 2);
        for (final key in [
          'fxaa',
          'render_scale',
          'shadow_blur_scale',
          'constant_buffer_dedup',
        ]) {
          expect(grids.containsKey(key), isFalse);
        }
        expect(LodmodConfigService.migrate(migrated), migrated);
      },
    );

    test('supports quoted headers, CRLF and missing trailing newlines', () {
      const raw =
          'enabled = true\r\n ["high_grids"] # grids\r\n'
          'rings = 4\r\nrender_scale = 2.0';
      final config = readConfig(LodmodConfigService.migrate(raw));
      expect(config['render_scale'], 2.0);
      expect(config['high_grids']['rings'], 4);
      expect(config['high_grids'].containsKey('render_scale'), isFalse);
    });

    for (final source in [
      'high_grids.enabled = true\nhigh_grids.rings = 3\nhigh_grids.render_scale = 1.5\n',
      'high_grids = { enabled = true, rings = 3, render_scale = 1.5 }\n',
    ]) {
      test('migrates alternate table syntax: $source', () {
        final migrated = LodmodConfigService.migrate(source);
        final config = readConfig(migrated);
        expect(config['render_scale'], 1.5);
        expect(config['high_grids']['enabled'], isTrue);
        expect(config['high_grids']['rings'], 3);
        expect(config['high_grids'].containsKey('render_scale'), isFalse);
        expect(LodmodConfigService.migrate(migrated), migrated);
      });
    }

    test('keeps array-of-table rules and multiline strings intact', () {
      const raw = '''enabled = true
note = """
[not_a_table]
render_scale = 9.0
"""
[high_grids] # grid rules
enabled = true
rings = 4
[[high_grids.room_rings]]
room = 0x170
rings = 1
[[high_grids.room_rings]]
room = 0x150
rings = 2
''';
      final migrated = LodmodConfigService.migrate(raw);
      final config = readConfig(migrated);
      expect(config['note'], '[not_a_table]\nrender_scale = 9.0\n');
      expect(config['render_scale'], 1.0);
      expect(config['high_grids']['room_rings'], [
        {'room': 0x170, 'rings': 1},
        {'room': 0x150, 'rings': 2},
      ]);
      expect(migrated, contains('# grid rules'));
    });

    test('leaves malformed input and invalid table types untouched', () {
      for (final raw in [
        'enabled = true\nenabled = false\n',
        '[high_grids\nrings = 2\n',
        'high_grids = "not a table"\n',
      ]) {
        expect(LodmodConfigService.migrate(raw), raw);
      }
    });
  });

  test(
    'ensureConfigs, GUI save and reload keep migrated fields in their NAMS scope',
    () async {
      final gameDir = Directory.systemTemp.createTempSync(
        'yp_lodmod_migration_',
      );
      final container = ProviderContainer();
      addTearDown(() {
        container.dispose();
        gameDir.deleteSync(recursive: true);
      });
      final file = File(path.join(gameDir.path, 'nams', 'lodmod.toml'));
      file.parent.createSync(recursive: true);
      file.writeAsStringSync(
        'enabled = true\n [high_grids] # my settings\n'
        'enabled = true\nrings = 3\nfxaa = true\nrender_scale = 1.25\n',
      );

      await NamsConfigService.ensureConfigs(gameDir.path);
      final first = file.readAsStringSync();
      expect(readConfig(first)['render_scale'], 1.25);
      await NamsConfigService.ensureConfigs(gameDir.path);
      expect(file.readAsStringSync(), first);

      final notifier = container.read(configStateControllerProvider.notifier);
      await notifier.loadConfigs(gameDir.path);
      notifier.updateLodmod(LodModFields.renderScale.key, 1.75);
      notifier.updateLodmod(
        LodModFields.highGridsRings.key,
        4,
        section: 'high_grids',
      );
      await notifier.saveConfigs(gameDir.path);
      await notifier.loadConfigs(gameDir.path);

      final config = readConfig(file.readAsStringSync());
      expect(config['fxaa'], isTrue);
      expect(config['render_scale'], 1.75);
      expect(config['high_grids']['rings'], 4);
      expect(config['high_grids'].containsKey('render_scale'), isFalse);
      expect(container.read(configStateControllerProvider).lodmodError, isNull);
    },
  );
}
