import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/wine/native_steam.dart';

import 'fake_steam_tree.dart';

void main() {
  late FakeSteamTree tree;

  setUp(() => tree = FakeSteamTree.create());
  tearDown(() => tree.dispose());

  group('nativeSteamLibraries', () {
    test('includes the root and its vdf entries', () {
      final root = tree.addSteamRoot();
      tree.addLibraryFolders('''
"libraryfolders"
{
	"0" { "path" "$root" }
	"1" { "path" "/mnt/games/SteamLibrary" }
}
''');

      expect(
        tree.runNative(nativeSteamLibraries),
        containsAll([root, '/mnt/games/SteamLibrary']),
      );
    });

    test('is just the root when there is no vdf', () {
      final root = tree.addSteamRoot();
      expect(tree.runNative(nativeSteamLibraries), [root]);
    });
  });

  group('findNierInNativeSteam', () {
    test('finds the game in the root library', () {
      tree.addSteamRoot();
      final dir = tree.addNier();

      final found = tree.runNative(findNierInNativeSteam);

      expect(found, hasLength(1));
      expect(found.single.path, dir);
      expect(found.single.hasData, isFalse);
    });

    test('finds the game in a secondary library from the vdf', () {
      final root = tree.addSteamRoot();
      final dir = tree.addNier(library: root);
      tree.addLibraryFolders('''
"libraryfolders"
{
	"0" { "path" "$root" }
}
''');

      expect(tree.runNative(findNierInNativeSteam).single.path, dir);
    });

    test('reports nothing when the game is absent', () {
      tree.addSteamRoot();
      expect(tree.runNative(findNierInNativeSteam), isEmpty);
    });
  });
}
