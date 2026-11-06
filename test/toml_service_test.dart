import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/toml_service.dart';

void main() {
  test('saving after a commented header keeps fields in their table', () {
    const raw = 'enabled = true\n  [high_grids] # rules\nrings = 2\n';
    final output = TomlService.updateToml(raw, {
      'render_scale': 1.5,
      'high_grids': {'rings': 3},
    });
    final parsed = TomlService.parseStrict(output);
    expect(parsed.error, isNull);
    expect(parsed.values['render_scale'], 1.5);
    expect(parsed.values['high_grids']['rings'], 3);
    expect(parsed.values['high_grids'].containsKey('render_scale'), isFalse);
    expect(output, contains('# rules'));
  });

  test('updates array-of-table rules without duplicate declarations', () {
    const raw =
        '[high_grids] # keep rules\nrings = 2\n'
        '[[high_grids.room_rings]]\nroom = 0x170\nrings = 1\n';
    final output = TomlService.updateToml(raw, {
      'fxaa': true,
      'high_grids': {
        'room_rings': [
          {'room': 0x150, 'rings': 2},
        ],
      },
    });
    final parsed = TomlService.parseStrict(output);
    expect(parsed.error, isNull);
    expect(parsed.values['fxaa'], isTrue);
    expect(parsed.values['high_grids']['rings'], 2);
    expect(parsed.values['high_grids']['room_rings'], [
      {'room': 0x150, 'rings': 2},
    ]);
    expect(output, contains('# keep rules'));
  });

  test('unknown multiline values and escaped strings survive a save', () {
    const raw =
        'enabled = true\n'
        'note = """\n[not_a_table]\ntext = 1\n"""\n'
        '[high_grids]\nrings = 2\n';
    const label = 'quoted "name" with \\ and\na newline';
    final output = TomlService.updateToml(raw, {'label': label});
    final parsed = TomlService.parseStrict(output);
    expect(parsed.error, isNull);
    expect(parsed.values['label'], label);
    expect(parsed.values['note'], '[not_a_table]\ntext = 1\n');
    expect(parsed.values['high_grids']['rings'], 2);
  });

  test('refuses to rewrite malformed TOML', () {
    expect(
      () => TomlService.updateToml('enabled = true\nenabled = false\n', {
        'fxaa': true,
      }),
      throwsFormatException,
    );
  });

  group('TomlService.updateToml section handling', () {
    const original = '''# NAMS General Config.

validate_scripts = false

[mouse]
# Remove the deadzone and acceleration curve.
fix_camera_acceleration = false
# Sensitivity multiplier.
sensitivity = 2.0

[cutscene]
hd_cutscenes = false
''';

    test('writes new keys into an existing section', () {
      final updated = TomlService.updateToml(original, {
        'mouse': {'sensitivity': 3.5, 'third_person_mode': true},
      });

      expect(updated, contains('sensitivity = 3.5'));
      expect(updated, contains('third_person_mode = true'));
      expect(
        updated.indexOf('third_person_mode = true'),
        lessThan(updated.indexOf('[cutscene]')),
        reason: 'new key must land inside [mouse], not at EOF',
      );
    });

    test('creates a missing nested section with its keys', () {
      final updated = TomlService.updateToml(original, {
        'mouse': {
          'bindings': {'standard_jump': 'SPACE', 'non_standard_evade': ''},
        },
      });

      expect(updated, contains('[mouse.bindings]'));
      expect(updated, contains('standard_jump = "SPACE"'));
      expect(updated, contains('non_standard_evade = ""'));
      expect(
        updated,
        isNot(contains('bindings = ')),
        reason: 'nested maps must never be serialized as scalar values',
      );
    });

    test('preserves comments', () {
      final updated = TomlService.updateToml(original, {
        'validate_scripts': true,
        'mouse': {'sensitivity': 3.5},
      });

      expect(updated, contains('# NAMS General Config.'));
      expect(
        updated,
        contains('# Remove the deadzone and acceleration curve.'),
      );
      expect(updated, contains('# Sensitivity multiplier.'));
    });

    test('round-trips through parse', () {
      final updated = TomlService.updateToml(original, {
        'mouse': {
          'sensitivity': 3.5,
          'bindings': {'standard_jump': 'SPACE'},
        },
      });
      final parsed = TomlService.parse(updated);
      final mouse = parsed['mouse'] as Map<String, dynamic>;

      expect(mouse['sensitivity'], 3.5);
      expect((mouse['bindings'] as Map)['standard_jump'], 'SPACE');
      expect((parsed['cutscene'] as Map)['hd_cutscenes'], isFalse);
    });

    test('write-back of a parsed document is a no-op', () {
      final updated = TomlService.updateToml(original, {
        'mouse': {
          'bindings': {'standard_jump': 'SPACE'},
        },
      });
      final twice = TomlService.updateToml(updated, TomlService.parse(updated));

      expect(twice, updated);
    });

    test('updates keys inside an existing dotted section in place', () {
      const withNested = '''
[mouse]
sensitivity = 2.0

[mouse.bindings]
standard_jump = "SPACE"

[cutscene]
hd_cutscenes = true
''';
      final updated = TomlService.updateToml(withNested, {
        'mouse': {
          'sensitivity': 4.0,
          'bindings': {'standard_jump': 'E', 'standard_walk': 'LSHIFT'},
        },
      });

      expect(updated, contains('standard_jump = "E"'));
      expect(
        updated.indexOf('standard_walk = "LSHIFT"'),
        greaterThan(updated.indexOf('[mouse.bindings]')),
      );
      expect(
        updated.indexOf('standard_walk = "LSHIFT"'),
        lessThan(updated.indexOf('[cutscene]')),
        reason: 'new nested key must land inside [mouse.bindings]',
      );
      expect(
        updated.split('[mouse.bindings]').first,
        isNot(contains('standard_jump')),
        reason: 'nested keys must not be misattributed to [mouse]',
      );
      expect(updated, contains('hd_cutscenes = true'));
    });
  });
}
