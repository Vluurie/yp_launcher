import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/toml_service.dart';

void main() {
  group('parseStrict', () {
    test('accepts a valid manifest', () {
      const toml = 'id = "my_first_mod"\n'
          'display_name = "My First Mod"\n'
          'version = "1.0.0"\n';
      final result = TomlService.parseStrict(toml);
      expect(result.ok, isTrue);
      expect(result.error, isNull);
      expect(result.values['id'], 'my_first_mod');
    });

    test('rejects an unterminated string', () {
      const toml = 'id = "my_first_mod\n'
          'version = "1.0.0"\n';
      final result = TomlService.parseStrict(toml);
      expect(result.ok, isFalse);
      expect(result.error, isNotNull);
    });

    test('rejects a missing closing quote at the end', () {
      const toml = 'id = "my_first_mod"\n'
          'display_name = "My First Mod\n';
      final result = TomlService.parseStrict(toml);
      expect(result.ok, isFalse);
    });

    test('rejects a bare value', () {
      final result = TomlService.parseStrict('id = my_first_mod\n');
      expect(result.ok, isFalse);
    });

    test('rejects a broken table header', () {
      const toml = 'name = "x"\n'
          '[text.name\n'
          '_us = "y"\n';
      final result = TomlService.parseStrict(toml);
      expect(result.ok, isFalse);
    });

    test('rejects a duplicate key', () {
      const toml = 'id = "a"\nid = "b"\n';
      final result = TomlService.parseStrict(toml);
      expect(result.ok, isFalse);
    });

    test('empty input is not an error', () {
      final result = TomlService.parseStrict('   \n');
      expect(result.ok, isTrue);
      expect(result.values, isEmpty);
    });

    test('accepts the item example with its tables', () {
      const toml = 'name = "item_test_pebble"\n'
          'id_type = 0\n'
          'no_trash = true\n'
          '\n'
          '[auto_give]\n'
          'quantity = 1\n'
          '\n'
          '[text.name]\n'
          '_us = "Test Pebble"\n';
      final result = TomlService.parseStrict(toml);
      expect(result.ok, isTrue);
      expect(result.values['auto_give']['quantity'], 1);
      expect(result.values['text']['name']['_us'], 'Test Pebble');
    });

    test('lenient parse still swallows what strict rejects', () {
      const broken = 'id = "my_first_mod\n';
      expect(TomlService.parseStrict(broken).ok, isFalse);
      expect(() => TomlService.parse(broken), returnsNormally);
    });
  });
}
