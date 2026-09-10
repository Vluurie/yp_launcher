import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/toml_service.dart';

void main() {
  group('updateToml with multi-line arrays', () {
    test('replaces a multi-line array without leaking its old lines', () {
      const raw = '''
[high_grids]
rings = 1
room_rings = [
    { room = 0x170, rings = 1 },
]
far_grid_load_interval = 3
''';
      final values = TomlService.parse(raw);
      final grids = Map<String, dynamic>.from(
        values['high_grids'] as Map<String, dynamic>,
      );
      grids['room_rings'] = [
        {'room': 0x170, 'rings': 1},
        {'room': 0x150, 'rings': 2},
      ];
      values['high_grids'] = grids;

      final out = TomlService.updateToml(raw, values);

      expect(out, isNot(contains('0x170')));
      expect('\n'.allMatches(out).length, lessThan(8));

      final reparsed = TomlService.parse(out);
      final rules = reparsed['high_grids']['room_rings'] as List;
      expect(rules, hasLength(2));
      expect(rules[1]['room'], 0x150);
      expect(reparsed['high_grids']['far_grid_load_interval'], 3);
    });

    test('keeps a multi-line array untouched when the value is unchanged', () {
      const raw = '''
[high_grids]
room_rings = [
    { room = 0x170, rings = 1 },
]
''';
      final out = TomlService.updateToml(raw, TomlService.parse(raw));
      final reparsed = TomlService.parse(out);
      final rules = reparsed['high_grids']['room_rings'] as List;
      expect(rules, hasLength(1));
      expect(rules.first['room'], 0x170);
    });

    test('ignores brackets inside strings and comments', () {
      const raw = '''
name = "a [ b"  # trailing ] comment
other = 1
''';
      final values = TomlService.parse(raw);
      values['other'] = 2;
      final out = TomlService.updateToml(raw, values);

      final reparsed = TomlService.parse(out);
      expect(reparsed['name'], 'a [ b');
      expect(reparsed['other'], 2);
    });
  });
}
