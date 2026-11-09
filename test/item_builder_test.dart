import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/docs/item_builder_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('item_builder_test');
  });

  tearDown(() {
    if (gameDir.existsSync()) gameDir.deleteSync(recursive: true);
  });

  group('internalNameFor', () {
    test('turns a display name into a safe identifier', () {
      expect(ItemDraft.internalNameFor('My Thing'), 'item_my_thing');
      expect(ItemDraft.internalNameFor('Rusty Bolt'), 'item_rusty_bolt');
    });

    test('drops punctuation and collapses separators', () {
      expect(
        ItemDraft.internalNameFor("Engineer's  Wrench!"),
        'item_engineer_s_wrench',
      );
    });

    test('keeps digits', () {
      expect(ItemDraft.internalNameFor('Type 40 Sword'), 'item_type_40_sword');
    });

    test('falls back when nothing usable is left', () {
      expect(ItemDraft.internalNameFor('!!!'), 'item_unnamed');
      expect(ItemDraft.internalNameFor(''), 'item_unnamed');
    });

    test('handles non-latin input without producing a bare prefix', () {
      expect(ItemDraft.internalNameFor('わたしのもの'), 'item_unnamed');
    });

    test('always starts with item_, which the engine requires', () {
      const inputs = [
        'My Thing',
        'item already prefixed',
        '   ',
        '???',
        'わたしのもの',
        '42',
      ];
      for (final input in inputs) {
        expect(
          ItemDraft.internalNameFor(input),
          startsWith('item_'),
          reason: 'input: "$input"',
        );
      }
    });

    test('never leaves a bare prefix or a trailing underscore', () {
      for (final input in ['', '!', 'x!', 'a  b  ']) {
        final name = ItemDraft.internalNameFor(input);
        expect(name, isNot('item_'));
        expect(name.endsWith('_'), isFalse, reason: 'input: "$input"');
      }
    });
  });

  group('toToml', () {
    test('produces valid toml with the required fields', () {
      const draft = ItemDraft(
        displayName: 'My Thing',
        description: 'Something useful.',
        id: 8500,
      );
      final parsed = TomlService.parseStrict(ItemBuilderService.toToml(draft));

      expect(parsed.ok, isTrue);
      expect(parsed.values['name'], 'item_my_thing');
      expect(parsed.values['id_type'], 0);
      expect(parsed.values['text']['name']['_us'], 'My Thing');
      expect(parsed.values['text']['help']['_us'], 'Something useful.');
    });

    test('omits max_num when it is the default', () {
      const draft = ItemDraft(displayName: 'A', description: 'B', id: 8000);
      expect(ItemBuilderService.toToml(draft), isNot(contains('max_num')));
    });

    test('writes max_num when the player can carry more', () {
      const draft = ItemDraft(
        displayName: 'A',
        description: 'B',
        id: 8000,
        maxCarry: 99,
      );
      final parsed = TomlService.parseStrict(ItemBuilderService.toToml(draft));
      expect(parsed.values['max_num'], 99);
    });

    test('sellable writes a price, otherwise no_trash', () {
      const sellable = ItemDraft(
        displayName: 'A',
        description: 'B',
        id: 8000,
        sellable: true,
        sellPrice: 250,
      );
      final withPrice = TomlService.parseStrict(
        ItemBuilderService.toToml(sellable),
      );
      expect(withPrice.values['sell'], 250);
      expect(withPrice.values.containsKey('no_trash'), isFalse);

      const keepsafe = ItemDraft(
        displayName: 'A',
        description: 'B',
        id: 8000,
      );
      final locked = TomlService.parseStrict(
        ItemBuilderService.toToml(keepsafe),
      );
      expect(locked.values['no_trash'], isTrue);
    });

    test('auto_give can be switched off', () {
      const draft = ItemDraft(
        displayName: 'A',
        description: 'B',
        id: 8000,
        autoGive: false,
      );
      expect(ItemBuilderService.toToml(draft), isNot(contains('auto_give')));
    });

    test('quotes in user text do not break the file', () {
      const draft = ItemDraft(
        displayName: 'The "Real" Thing',
        description: r'Has a \ backslash and a " quote.',
        id: 8000,
      );
      final parsed = TomlService.parseStrict(ItemBuilderService.toToml(draft));
      expect(parsed.ok, isTrue);
      expect(parsed.values['text']['name']['_us'], 'The "Real" Thing');
    });

    test('writes an icon section only when a texture is set', () {
      const without = ItemDraft(displayName: 'A', description: 'B', id: 8000);
      expect(ItemBuilderService.toToml(without), isNot(contains('[icon]')));

      const with_ = ItemDraft(
        displayName: 'A',
        description: 'B',
        id: 8500,
        misctexName: 'item_thumb_8500',
      );
      final parsed = TomlService.parseStrict(ItemBuilderService.toToml(with_));
      expect(parsed.ok, isTrue);
      expect(parsed.values['icon']['misctex_name'], 'item_thumb_8500');
    });

    test('never writes an id, since the filename carries it', () {
      const draft = ItemDraft(displayName: 'A', description: 'B', id: 8500);
      final parsed = TomlService.parseStrict(ItemBuilderService.toToml(draft));
      expect(parsed.values.containsKey('id'), isFalse);
      expect(ItemBuilderService.fileNameFor(draft), 'item_8500.toml');
    });
  });

  group('idFromFileName', () {
    test('reads the id back out', () {
      expect(ItemBuilderService.idFromFileName('item_8500.toml'), 8500);
    });

    test('ignores files that are not items', () {
      expect(ItemBuilderService.idFromFileName('weapon_1500.toml'), isNull);
      expect(ItemBuilderService.idFromFileName('item_abc.toml'), isNull);
      expect(ItemBuilderService.idFromFileName('item_8500.md'), isNull);
    });
  });

  group('nextFreeId', () {
    test('starts at 8000 on an empty install', () async {
      expect(await ItemBuilderService.nextFreeId(gameDir.path), 8000);
    });

    test('skips ids already used by any mod', () async {
      const draft = ItemDraft(displayName: 'A', description: 'B', id: 8000);
      await ItemBuilderService.write(
        gameDir: gameDir.path,
        modId: 'mod_a',
        draft: draft,
      );
      await ItemBuilderService.write(
        gameDir: gameDir.path,
        modId: 'mod_b',
        draft: draft.copyWith(id: 8001),
      );

      expect(await ItemBuilderService.nextFreeId(gameDir.path), 8002);
    });
  });

  test('write puts the file under entities/', () async {
    const draft = ItemDraft(
      displayName: 'My Thing',
      description: 'Something useful.',
      id: 8500,
    );
    final file = await ItemBuilderService.write(
      gameDir: gameDir.path,
      modId: 'my_mod',
      draft: draft,
    );

    expect(file.existsSync(), isTrue);
    expect(
      file.path,
      p.join(
        gameDir.path,
        'nams',
        'mods',
        'my_mod',
        'entities',
        'item_8500.toml',
      ),
    );
  });
}
