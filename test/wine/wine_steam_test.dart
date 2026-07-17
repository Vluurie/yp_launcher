import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/wine_steam.dart';

import '../support/posix_only.dart';
import 'fake_bottle.dart';

const _realVdf = '''
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\\\Program Files (x86)\\\\Steam"
		"label"		""
		"apps"
		{
			"228980"		"102931551"
			"524220"		"43881494773"
		}
	}
}
''';

void main() {
  group('normalizeSteamLibraryPath', () {
    test('C: resolves to drive_c', () {
      expect(
        normalizeSteamLibraryPath(r'C:\\Program Files (x86)\\Steam', '/b/Steam'),
        '/b/Steam/drive_c/Program Files (x86)/Steam',
      );
    });

    test('another drive resolves through dosdevices', () {
      expect(
        normalizeSteamLibraryPath(r'D:\\SteamLibrary', '/b/Steam'),
        '/b/Steam/dosdevices/d:/SteamLibrary',
      );
    });

    test('drive letter is matched case-insensitively', () {
      expect(
        normalizeSteamLibraryPath(r'c:\\Games', '/b/Steam'),
        '/b/Steam/drive_c/Games',
      );
    });

    test('a host path is returned untouched', () {
      expect(
        normalizeSteamLibraryPath('/mnt/games/SteamLibrary', '/b/Steam'),
        '/mnt/games/SteamLibrary',
      );
    });

    test('escaped quotes are unescaped', () {
      expect(
        normalizeSteamLibraryPath(r'C:\\a\"b', '/b/Steam'),
        '/b/Steam/drive_c/a"b',
      );
    });
  }, skip: skipOnWindows);

  group('parseLibraryFoldersInPrefix', () {
    late FakeBottleTree tree;

    setUp(() => tree = FakeBottleTree.create());
    tearDown(() => tree.dispose());

    test('reads the real-world vdf shape', () {
      final bottle = tree.addBottle('Steam');
      tree.addLibraryFolders('Steam', _realVdf);

      expect(
        parseLibraryFoldersInPrefix(bottle),
        [p.join(bottle, 'drive_c', 'Program Files (x86)', 'Steam')],
      );
    });

    test('a prefix without a vdf yields nothing', () {
      final bottle = tree.addBottle('Steam');
      expect(parseLibraryFoldersInPrefix(bottle), isEmpty);
    });
  });

  group('nierDirsInPrefix', () {
    late FakeBottleTree tree;

    setUp(() => tree = FakeBottleTree.create());
    tearDown(() => tree.dispose());

    test('finds the game at the canonical Steam location', () {
      final bottle = tree.addBottle('Steam');
      final exe = tree.addNier('Steam');

      expect(nierDirsInPrefix(bottle), [p.dirname(exe)]);
    });

    test('an empty prefix yields nothing', () {
      expect(nierDirsInPrefix(tree.addBottle('Steam')), isEmpty);
    });

    test('a dir found by both probes is reported once', () {
      final bottle = tree.addBottle('Steam');
      final exe = tree.addNier('Steam');
      tree.addLibraryFolders('Steam', _realVdf);

      expect(nierDirsInPrefix(bottle), [p.dirname(exe)]);
    });
  });

  group('findNierInPrefixes', () {
    late FakeBottleTree tree;

    setUp(() => tree = FakeBottleTree.create());
    tearDown(() => tree.dispose());

    test('reports the install and whether data/ exists', () {
      tree.addBottle('Steam');
      final exe = tree.addNier('Steam');

      final found = tree.run(findNierInPrefixes);

      expect(found, hasLength(1));
      expect(found.single.path, p.dirname(exe));
      expect(found.single.hasData, isFalse);
    });

    test('bottles without the game are skipped', () {
      tree.addBottle('Empty');
      expect(tree.run(findNierInPrefixes), isEmpty);
    });
  });
}
