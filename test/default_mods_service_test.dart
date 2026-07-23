import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/providers/default_mods_state.dart';
import 'package:yp_launcher/services/default_mods_service.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('yp_defaults_');
  });

  tearDown(() {
    try {
      gameDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  File defaultsFile() =>
      File(p.join(gameDir.path, 'nams', 'default_mods.toml'));

  Future<void> star(String relPath, DefaultKind kind, {int? outfitId}) =>
      DefaultModsService.setDefault(gameDir.path, relPath, kind,
          outfitId: outfitId);

  Future<void> unstar(String relPath) =>
      DefaultModsService.setDefault(gameDir.path, relPath, null);

  Future<List<String>> paths() async =>
      (await DefaultModsService.list(gameDir.path)).map((e) => e.path).toList();

  Future<DefaultKind?> kindOf(String relPath) async {
    final entries = await DefaultModsService.list(gameDir.path);
    return DefaultModsService.entryFor(entries, relPath)?.kind;
  }

  test('missing file yields no defaults', () async {
    expect(await DefaultModsService.list(gameDir.path), isEmpty);
  });

  test('a whole mod round-trips through disk', () async {
    await star('mods/my_mod', DefaultKind.outfitBare);

    expect(await paths(), ['mods/my_mod']);
    expect(defaultsFile().existsSync(), isTrue);
  });

  test('unstarring drops the entry', () async {
    await star('mods/my_mod', DefaultKind.outfitBare);
    await unstar('mods/my_mod');

    expect(await DefaultModsService.list(gameDir.path), isEmpty);
  });

  test('starring the same entry twice does not duplicate it', () async {
    await star('mods/my_mod', DefaultKind.outfitBare);
    await star('mods/my_mod', DefaultKind.outfitBare);

    expect(await paths(), ['mods/my_mod']);
  });

  test('backslashes are normalized to forward slashes', () async {
    await star(r'mods\my_mod', DefaultKind.outfitBare);

    expect(await paths(), ['mods/my_mod']);
  });

  test('unrelated mods survive a star', () async {
    await star('mods/a', DefaultKind.outfitBare);
    await star('mods/b', DefaultKind.outfitBare);

    expect(await paths(), containsAll(['mods/a', 'mods/b']));
  });

  group('kinds', () {
    test('each kind round-trips through its own list', () async {
      await star('mods/bare/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/blink/pl/pl000f', DefaultKind.outfitAnimation);

      expect(await kindOf('mods/bare/pl/pl000d'), DefaultKind.outfitBare);
      expect(await kindOf('mods/blink/pl/pl000f'), DefaultKind.outfitAnimation);
    });

    test('a config outfit is stored apart from a bare one', () async {
      await star('mods/configured/pl/pl000d', DefaultKind.outfitConfig);

      final raw = defaultsFile().readAsStringSync();
      expect(raw, contains('outfit_config = ["mods/configured/pl/pl000d"]'));
      expect(raw, contains('outfit_bare = []'));
    });

    test('re-starring under a different kind moves the entry', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig);

      expect(await paths(), ['mods/alpha/pl/pl000d']);
      expect(await kindOf('mods/alpha/pl/pl000d'), DefaultKind.outfitConfig);
    });

    test('unstarring finds the entry whichever list it is in', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig);
      await unstar('mods/alpha/pl/pl000d');

      expect(await DefaultModsService.list(gameDir.path), isEmpty);
    });
  });

  group('outfit choice', () {
    Future<int?> outfitOf(String relPath) async {
      final entries = await DefaultModsService.list(gameDir.path);
      return DefaultModsService.entryFor(entries, relPath)?.outfitId;
    }

    test('a chosen outfit round-trips through disk', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig,
          outfitId: 2);

      expect(await outfitOf('mods/alpha/pl/pl000d'), 2);
    });

    test('no choice means the outfit worn without an item', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig);

      expect(await outfitOf('mods/alpha/pl/pl000d'), isNull);
    });

    test('a chosen outfit is written as a table, a plain one as a string',
        () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig,
          outfitId: 3);
      await star('mods/beta/pl/pl020d', DefaultKind.outfitConfig);

      final raw = defaultsFile().readAsStringSync();
      expect(raw, contains('{ path = "mods/alpha/pl/pl000d", outfit_id = 3 }'));
      expect(raw, contains('"mods/beta/pl/pl020d"'));
    });

    test('re-picking replaces the previous choice', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig,
          outfitId: 1);
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig,
          outfitId: 2);

      expect(await paths(), ['mods/alpha/pl/pl000d']);
      expect(await outfitOf('mods/alpha/pl/pl000d'), 2);
    });

    test('clearing the choice returns to the item-less outfit', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig,
          outfitId: 2);
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig);

      expect(await outfitOf('mods/alpha/pl/pl000d'), isNull);
    });

    test('a hand-written table entry is read back', () async {
      defaultsFile().parent.createSync(recursive: true);
      defaultsFile().writeAsStringSync(
        'outfit_config = [{ path = "mods/alpha/pl/pl000d", outfit_id = 2 }]\n',
      );

      expect(await paths(), ['mods/alpha/pl/pl000d']);
      expect(await outfitOf('mods/alpha/pl/pl000d'), 2);
      expect(await kindOf('mods/alpha/pl/pl000d'), DefaultKind.outfitConfig);
    });

    test('strings and tables mix in one list', () async {
      defaultsFile().parent.createSync(recursive: true);
      defaultsFile().writeAsStringSync(
        'outfit_config = ["mods/plain/pl/pl000d", '
        '{ path = "mods/picked/pl/pl020d", outfit_id = 1 }]\n',
      );

      expect(await outfitOf('mods/plain/pl/pl000d'), isNull);
      expect(await outfitOf('mods/picked/pl/pl020d'), 1);
    });

    test('a table without a path is skipped', () async {
      defaultsFile().parent.createSync(recursive: true);
      defaultsFile()
          .writeAsStringSync('outfit_config = [{ outfit_id = 2 }]\n');

      expect(await DefaultModsService.list(gameDir.path), isEmpty);
    });

    test('the choice survives an unrelated write', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig,
          outfitId: 2);
      await star('mods/beta/pl/pl020d', DefaultKind.outfitBare);

      expect(await outfitOf('mods/alpha/pl/pl000d'), 2);
    });

    test('a lone item outfit is written when the mod becomes the default',
        () async {
      const choices = [
        OutfitChoice(outfitId: 71900, name: 'Divergent', needsItem: true),
      ];
      final id = DefaultModsService.initialOutfitId(
        DefaultKind.outfitConfig,
        choices,
      );
      await star('mods/black_wax/pl/pl020d', DefaultKind.outfitConfig,
          outfitId: id);

      expect(await outfitOf('mods/black_wax/pl/pl020d'), 71900);
      expect(
        defaultsFile().readAsStringSync(),
        contains('{ path = "mods/black_wax/pl/pl020d", outfit_id = 71900 }'),
      );
    });
  });

  group('initialOutfitId', () {
    test('an item outfit is picked when it is the only one', () {
      expect(
        DefaultModsService.initialOutfitId(DefaultKind.outfitConfig, const [
          OutfitChoice(outfitId: 71900, name: 'Divergent', needsItem: true),
        ]),
        71900,
      );
    });

    test('an item-less outfit is left to the game', () {
      expect(
        DefaultModsService.initialOutfitId(DefaultKind.outfitConfig, const [
          OutfitChoice(outfitId: 5, name: 'Replacement', needsItem: false),
          OutfitChoice(outfitId: 6, name: 'Extra', needsItem: true),
        ]),
        isNull,
      );
    });

    test('several item outfits are left for the user to pick', () {
      expect(
        DefaultModsService.initialOutfitId(DefaultKind.outfitConfig, const [
          OutfitChoice(outfitId: 1, name: 'One', needsItem: true),
          OutfitChoice(outfitId: 2, name: 'Two', needsItem: true),
        ]),
        isNull,
      );
    });

    test('no outfits means nothing to write', () {
      expect(
        DefaultModsService.initialOutfitId(
            DefaultKind.outfitConfig, const []),
        isNull,
      );
    });

    test('other kinds never carry an outfit id', () {
      expect(
        DefaultModsService.initialOutfitId(DefaultKind.outfitBare, const [
          OutfitChoice(outfitId: 71900, name: 'Divergent', needsItem: true),
        ]),
        isNull,
      );
    });
  });

  group('stem exclusivity', () {
    test('starring a stem in another mod releases the previous owner',
        () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/beta/pl/pl000d', DefaultKind.outfitBare);

      expect(await paths(), ['mods/beta/pl/pl000d']);
    });

    test('a config outfit displaces a bare one for the same stem', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/beta/pl/pl000d', DefaultKind.outfitConfig);

      expect(await paths(), ['mods/beta/pl/pl000d']);
      expect(await kindOf('mods/beta/pl/pl000d'), DefaultKind.outfitConfig);
    });

    test('a bare outfit displaces a config one for the same stem', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitConfig);
      await star('mods/beta/pl/pl000d', DefaultKind.outfitBare);

      expect(await paths(), ['mods/beta/pl/pl000d']);
      expect(await kindOf('mods/beta/pl/pl000d'), DefaultKind.outfitBare);
    });

    test('an outfit and an animation coexist', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/beta/pl/pl000f', DefaultKind.outfitAnimation);

      expect(await paths(),
          containsAll(['mods/alpha/pl/pl000d', 'mods/beta/pl/pl000f']));
    });

    test('two animations for the same stem do not displace each other',
        () async {
      await star('mods/alpha/pl/pl000f', DefaultKind.outfitAnimation);
      await star('mods/beta/pl/pl000f', DefaultKind.outfitAnimation);

      expect(await paths(),
          containsAll(['mods/alpha/pl/pl000f', 'mods/beta/pl/pl000f']));
    });

    test('each character keeps its own default outfit', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/beta/pl/pl020d', DefaultKind.outfitConfig);
      await star('mods/gamma/pl/pl010d', DefaultKind.outfitBare);

      expect(
          await paths(),
          containsAll([
            'mods/alpha/pl/pl000d',
            'mods/beta/pl/pl020d',
            'mods/gamma/pl/pl010d',
          ]));
    });

    test('an outfit star does not disturb a starred animation', () async {
      await star('mods/blink/pl/pl000f', DefaultKind.outfitAnimation);
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/beta/pl/pl000d', DefaultKind.outfitConfig);

      expect(await paths(),
          containsAll(['mods/blink/pl/pl000f', 'mods/beta/pl/pl000d']));
      expect(await paths(), isNot(contains('mods/alpha/pl/pl000d')));
    });

    test('different stems within one mod coexist', () async {
      await star('mods/alpha/pl/pl000d', DefaultKind.outfitBare);
      await star('mods/alpha/pl/pl000f', DefaultKind.outfitAnimation);

      expect(await paths(),
          containsAll(['mods/alpha/pl/pl000d', 'mods/alpha/pl/pl000f']));
    });

    test('whole-mod entries are not treated as stems', () async {
      await star('mods/alpha', DefaultKind.outfitBare);
      await star('mods/beta', DefaultKind.outfitBare);

      expect(await paths(), containsAll(['mods/alpha', 'mods/beta']));
    });
  });

  group('DefaultKind', () {
    test('outfit kinds compete, animation does not', () {
      expect(DefaultKind.outfitBare.conflictsWith(DefaultKind.outfitConfig),
          isTrue);
      expect(DefaultKind.outfitConfig.conflictsWith(DefaultKind.outfitBare),
          isTrue);
      expect(DefaultKind.outfitBare.conflictsWith(DefaultKind.outfitAnimation),
          isFalse);
      expect(
          DefaultKind.outfitAnimation.conflictsWith(DefaultKind.outfitAnimation),
          isFalse);
    });

    test('every kind has a distinct toml key', () {
      final keys = DefaultKind.values.map((k) => k.key).toSet();
      expect(keys.length, DefaultKind.values.length);
    });

    test('fromKey round-trips every kind', () {
      for (final kind in DefaultKind.values) {
        expect(DefaultKind.fromKey(kind.key), kind);
      }
      expect(DefaultKind.fromKey('nope'), isNull);
    });
  });

  group('kindFor', () {
    test('an animation stem is an animation regardless of config', () {
      expect(DefaultModsService.kindFor('pl000f', hasOutfitConfig: false),
          DefaultKind.outfitAnimation);
      expect(DefaultModsService.kindFor('pl000f', hasOutfitConfig: true),
          DefaultKind.outfitAnimation);
    });

    test('an outfit stem follows whether the mod ships a config', () {
      expect(DefaultModsService.kindFor('pl000d', hasOutfitConfig: false),
          DefaultKind.outfitBare);
      expect(DefaultModsService.kindFor('pl000d', hasOutfitConfig: true),
          DefaultKind.outfitConfig);
    });
  });

  group('stemOf', () {
    test('reads the last segment of a stem entry', () {
      expect(DefaultModsService.stemOf('mods/alpha/pl/pl000d'), 'pl000d');
    });

    test('is null for a whole-mod entry', () {
      expect(DefaultModsService.stemOf('mods/alpha'), isNull);
    });

    test('tolerates backslashes and stray slashes', () {
      expect(DefaultModsService.stemOf(r'/mods\alpha\wp\wp0100/'), 'wp0100');
    });
  });

  group('isOutfitStem', () {
    test('player model stems are outfits', () {
      expect(DefaultModsService.isOutfitStem('pl000d'), isTrue);
      expect(DefaultModsService.isOutfitStem('pl0000'), isTrue);
      expect(DefaultModsService.isOutfitStem('pl020d'), isTrue);
    });

    test('stems ending in f are animations, not outfits', () {
      expect(DefaultModsService.isOutfitStem('pl000f'), isFalse);
      expect(DefaultModsService.isOutfitStem('pl002f'), isFalse);
      expect(DefaultModsService.isOutfitStem('pl010f'), isFalse);
    });

    test('non-player stems are not outfits', () {
      expect(DefaultModsService.isOutfitStem('wp0100'), isFalse);
      expect(DefaultModsService.isOutfitStem('q451'), isFalse);
    });

    test('case does not matter', () {
      expect(DefaultModsService.isOutfitStem('PL000D'), isTrue);
      expect(DefaultModsService.isOutfitStem('PL000F'), isFalse);
    });
  });

  group('matches', () {
    const bare = DefaultEntry(path: 'mods/a', kind: DefaultKind.outfitBare);

    test('an exact path matches', () {
      expect(DefaultModsService.matches([bare], 'mods/a'), isTrue);
    });

    test('a parent entry matches nested paths', () {
      expect(DefaultModsService.matches([bare], 'mods/a/pl/pl000d'), isTrue);
    });

    test('a sibling with a shared name prefix does not match', () {
      expect(DefaultModsService.matches([bare], 'mods/ab'), isFalse);
    });

    test('an unrelated path does not match', () {
      expect(DefaultModsService.matches([bare], 'mods/b'), isFalse);
    });

    test('entryFor reports which kind covers the path', () {
      const config =
          DefaultEntry(path: 'mods/c', kind: DefaultKind.outfitConfig);
      expect(
          DefaultModsService.entryFor([bare, config], 'mods/c/pl/pl000d')?.kind,
          DefaultKind.outfitConfig);
      expect(DefaultModsService.entryFor([bare, config], 'mods/z'), isNull);
    });
  });

  test('comments in the file survive a write', () async {
    defaultsFile().parent.createSync(recursive: true);
    defaultsFile().writeAsStringSync('# keep me\noutfit_bare = []\n');

    await star('mods/my_mod', DefaultKind.outfitBare);

    expect(defaultsFile().readAsStringSync(), contains('# keep me'));
  });

  test('a list this build does not know is left alone', () async {
    defaultsFile().parent.createSync(recursive: true);
    defaultsFile()
        .writeAsStringSync('outfit_bare = []\nfuture_kind = ["mods/x"]\n');

    await star('mods/my_mod', DefaultKind.outfitBare);

    expect(defaultsFile().readAsStringSync(), contains('future_kind'));
  });

  test('a malformed list degrades to no defaults instead of throwing',
      () async {
    defaultsFile().parent.createSync(recursive: true);
    defaultsFile().writeAsStringSync('outfit_bare = "not a list"\n');

    expect(await DefaultModsService.list(gameDir.path), isEmpty);
  });

  group('wouldReplace', () {
    DefaultModsData data(List<DefaultEntry> entries) =>
        DefaultModsData(entries: entries);

    test('names the mod currently holding the stem', () {
      final d = data(const [
        DefaultEntry(path: 'mods/alpha/pl/pl000d', kind: DefaultKind.outfitBare),
      ]);

      final hit = d.wouldReplace('mods/beta/pl/pl000d', DefaultKind.outfitBare);
      expect(hit?.path, 'mods/alpha/pl/pl000d');
    });

    test('a different stem is not replaced', () {
      final d = data(const [
        DefaultEntry(path: 'mods/alpha/pl/pl020d', kind: DefaultKind.outfitBare),
      ]);

      expect(d.wouldReplace('mods/beta/pl/pl000d', DefaultKind.outfitBare),
          isNull);
    });

    test('an animation is not replaced by an outfit', () {
      final d = data(const [
        DefaultEntry(
            path: 'mods/blink/pl/pl000f', kind: DefaultKind.outfitAnimation),
      ]);

      expect(d.wouldReplace('mods/beta/pl/pl000f', DefaultKind.outfitAnimation),
          isNull);
    });

    test('a config outfit replaces a bare one', () {
      final d = data(const [
        DefaultEntry(path: 'mods/alpha/pl/pl000d', kind: DefaultKind.outfitBare),
      ]);

      final hit =
          d.wouldReplace('mods/beta/pl/pl000d', DefaultKind.outfitConfig);
      expect(hit?.path, 'mods/alpha/pl/pl000d');
    });

    test('re-starring the same entry replaces nothing', () {
      final d = data(const [
        DefaultEntry(path: 'mods/alpha/pl/pl000d', kind: DefaultKind.outfitBare),
      ]);

      expect(d.wouldReplace('mods/alpha/pl/pl000d', DefaultKind.outfitBare),
          isNull);
    });

    test('a whole-mod entry has no stem to compete over', () {
      final d = data(const [
        DefaultEntry(path: 'mods/alpha', kind: DefaultKind.outfitBare),
      ]);

      expect(d.wouldReplace('mods/beta', DefaultKind.outfitBare), isNull);
    });
  });

  test('entries in one list do not leak into another', () async {
    defaultsFile().parent.createSync(recursive: true);
    defaultsFile().writeAsStringSync(
      'outfit_bare = ["mods/a"]\n'
      'outfit_config = ["mods/b"]\n'
      'outfit_animation = ["mods/c"]\n',
    );

    expect(await kindOf('mods/a'), DefaultKind.outfitBare);
    expect(await kindOf('mods/b'), DefaultKind.outfitConfig);
    expect(await kindOf('mods/c'), DefaultKind.outfitAnimation);
  });
}
