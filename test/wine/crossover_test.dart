import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/crossover.dart';

import '../support/posix_only.dart';
import 'fake_bottle.dart';

void main() {
  group('createBottleFromGamePath', () {
    test('derives name and path from a bottle-internal exe', () {
      const exe = '/u/Library/Application Support/CrossOver/Bottles/Steam'
          '/drive_c/Program Files (x86)/Steam/steamapps/common/NieRAutomata'
          '/NieRAutomata.exe';

      final bottle = createBottleFromGamePath(exe);

      expect(bottle, isNotNull);
      expect(bottle!.name, 'Steam');
      expect(
        bottle.path,
        '/u/Library/Application Support/CrossOver/Bottles/Steam',
      );
      expect(bottle.gamePath, exe);
    });

    test('rejects an exe outside any prefix', () {
      expect(createBottleFromGamePath('/Users/d/Games/NieRAutomata.exe'),
          isNull);
    });

    test('rejects a prefix that is not CrossOver', () {
      expect(
        createBottleFromGamePath(
          '/home/d/.steam/steam/steamapps/compatdata/524220/pfx'
          '/drive_c/game/NieRAutomata.exe',
        ),
        isNull,
      );
    });
  });

  group('listCrossOverBottles', () {
    late FakeBottleTree tree;

    setUp(() => tree = FakeBottleTree.create());
    tearDown(() => tree.dispose());

    test('lists dirs that contain drive_c', () {
      tree.addBottle('Steam');
      tree.addBottle('Other');

      final names =
          tree.run(() => listCrossOverBottles().map((b) => b.name).toSet());

      expect(names, {'Steam', 'Other'});
    });

    test('ignores dirs without drive_c', () {
      tree.addBottle('Steam');
      Directory(p.join(tree.root, 'NotABottle')).createSync();

      final names =
          tree.run(() => listCrossOverBottles().map((b) => b.name).toList());

      expect(names, ['Steam']);
    });

    test('an empty root yields no bottles', () {
      expect(tree.run(listCrossOverBottles), isEmpty);
    });
  });

  group('nierCandidatesInBottle', () {
    test('probes both Program Files variants', () {
      final candidates = nierCandidatesInBottle('/b/Steam');

      expect(candidates, [
        '/b/Steam/drive_c/Program Files (x86)/Steam/steamapps/common'
            '/NieRAutomata/NieRAutomata.exe',
        '/b/Steam/drive_c/Program Files/Steam/steamapps/common'
            '/NieRAutomata/NieRAutomata.exe',
      ]);
    }, skip: skipOnWindows);
  });

  group('findCrossOverWine', () {
    late FakeBottleTree tree;

    setUp(() => tree = FakeBottleTree.create());
    tearDown(() => tree.dispose());

    test('honours the override when the file exists', () {
      final script = tree.addWineScript();

      expect(tree.run(findCrossOverWine, wineCommand: script), script);
    });

    test('a missing override resolves to nothing', () {
      final found = tree.run(
        findCrossOverWine,
        wineCommand: p.join(tree.root, 'does-not-exist'),
      );

      expect(found, isNull);
    });
  });
}
