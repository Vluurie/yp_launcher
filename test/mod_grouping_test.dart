import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/models/mod_grouping.dart';

InstalledMod _mod({
  required String id,
  ModKind kind = ModKind.data,
  List<String> playerFiles = const [],
  List<DataCategory> categories = const [],
}) {
  return InstalledMod(
    id: id,
    displayName: id,
    rootPath: 'mods/$id',
    kind: kind,
    installedAt: DateTime.fromMillisecondsSinceEpoch(0),
    data: DataSummary(
      entries: [
        for (final c in categories)
          DataDirEntry(dirName: c.name, category: c, fileCount: 1),
      ],
      players: [
        for (final f in playerFiles) PlayerModelEntry(fileName: f, label: f),
      ],
    ),
  );
}

void main() {
  group('outfits group by player character', () {
    final cases = <String, ModGroupKind>{
      'pl0000.dat': ModGroupKind.outfit2b,
      'pl000d.dat': ModGroupKind.outfit2b,
      'plf00d.dat': ModGroupKind.outfit2b,
      'pl0200.dat': ModGroupKind.outfit9s,
      'pl020d.dat': ModGroupKind.outfit9s,
      'pl0100.dat': ModGroupKind.outfitA2,
      'pl0101.dat': ModGroupKind.outfitA2,
      'pl010d.dat': ModGroupKind.outfitA2,
    };

    cases.forEach((file, expected) {
      test('$file -> ${expected.name}', () {
        final groups = groupsForMod(_mod(id: 'm', playerFiles: [file]));
        expect(groups, {expected});
      });
    });

    test('non-main player models land in other outfits', () {
      expect(groupsForMod(_mod(id: 'm', playerFiles: ['pl1010.dat'])),
          {ModGroupKind.outfitOther});
    });

    test('a mod replacing 2B and 9S is listed under both', () {
      final groups = groupsForMod(
        _mod(id: 'm', playerFiles: ['pl0000.dat', 'pl0200.dat']),
      );
      expect(groups, {ModGroupKind.outfit2b, ModGroupKind.outfit9s});
    });

    test('several files for the same character yield one group', () {
      final groups = groupsForMod(
        _mod(id: 'm', playerFiles: ['pl0000.dat', 'pl0000.dtt', 'plf000.dat']),
      );
      expect(groups, {ModGroupKind.outfit2b});
    });
  });

  group('non-outfit mods group by category', () {
    test('weapons', () {
      expect(
        groupsForMod(_mod(id: 'w', categories: [DataCategory.weapon])),
        {ModGroupKind.weapon},
      );
    });

    test('effects', () {
      expect(
        groupsForMod(_mod(id: 'e', categories: [DataCategory.effects])),
        {ModGroupKind.effect},
      );
    });

    test('native mods without data land in the native group', () {
      final mod = InstalledMod(
        id: 'n',
        displayName: 'n',
        rootPath: 'mods/n',
        kind: ModKind.native,
        installedAt: DateTime.fromMillisecondsSinceEpoch(0),
      );
      expect(groupsForMod(mod), {ModGroupKind.native});
    });

    test('a native mod with an outfit overlay stays native only', () {
      final mod = _mod(
        id: 'n',
        kind: ModKind.native,
        playerFiles: ['pl0000.dat'],
        categories: [DataCategory.player],
      );
      expect(groupsForMod(mod), {ModGroupKind.native},
          reason: 'native mods must not be duplicated into outfit groups');
    });

    test('groupMods lists a native outfit mod once, under native', () {
      final grouped = groupMods([
        _mod(id: 'n', kind: ModKind.native, playerFiles: ['pl0000.dat']),
      ]);
      expect(grouped.keys.toList(), [ModGroupKind.native]);
      expect(grouped[ModGroupKind.native]!.single.id, 'n');
    });

    test('player data without recognised slots is an outfit', () {
      expect(
        groupsForMod(_mod(id: 'p', categories: [DataCategory.player])),
        {ModGroupKind.outfitOther},
      );
    });
  });

  group('groupMods', () {
    test('orders groups by modGroupOrder and keeps every mod', () {
      final mods = [
        _mod(id: 'weapon', categories: [DataCategory.weapon]),
        _mod(id: 'a2', playerFiles: ['pl0100.dat']),
        _mod(id: 'twob', playerFiles: ['pl0000.dat']),
        _mod(id: 'nams', kind: ModKind.native),
      ];
      final grouped = groupMods(mods);

      expect(grouped.keys.toList(), [
        ModGroupKind.native,
        ModGroupKind.outfit2b,
        ModGroupKind.outfitA2,
        ModGroupKind.weapon,
      ]);
      expect(grouped[ModGroupKind.outfit2b]!.single.id, 'twob');
    });

    test('a multi-character mod appears in each of its groups', () {
      final grouped = groupMods([
        _mod(id: 'pack', playerFiles: ['pl0000.dat', 'pl0200.dat']),
      ]);

      expect(grouped[ModGroupKind.outfit2b]!.single.id, 'pack');
      expect(grouped[ModGroupKind.outfit9s]!.single.id, 'pack');
    });

    test('empty input yields no groups', () {
      expect(groupMods(const []), isEmpty);
    });
  });
}
