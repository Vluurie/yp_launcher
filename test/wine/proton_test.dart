import 'package:flutter_test/flutter_test.dart';
import 'package:yp_launcher/services/wine/proton.dart';

import 'fake_steam_tree.dart';

void main() {
  late FakeSteamTree tree;

  setUp(() => tree = FakeSteamTree.create());
  tearDown(() => tree.dispose());

  group('findProtonPath', () {
    test('finds a build under compatibilitytools.d', () {
      final root = tree.addSteamRoot();
      final proton = tree.addProton('GE-Proton');

      expect(findProtonPath(root, root), proton);
    });

    test('is null when no Proton is installed', () {
      final root = tree.addSteamRoot();
      expect(findProtonPath(root, root), isNull);
    });

    test('does not find Proton that lives in a different library', () {
      tree.addSteamRoot();
      tree.addProton('GE-Proton');
      // game sits on a separate games partition with no Proton next to it
      const gameLibrary = '/mnt/games/SteamLibrary';
      expect(findProtonPath(gameLibrary, gameLibrary), isNull);
    });
  });

  group('findAnyProtonPath', () {
    test('finds Proton from any Steam root even when game is elsewhere', () {
      tree.addSteamRoot();
      final proton = tree.addProton('GE-Proton');
      tree.runNative(() {
        expect(findAnyProtonPath(), proton);
        expect(hasAnyProtonInstall(), isTrue);
      });
    });

    test('is null when no Steam root has Proton', () {
      tree.addSteamRoot();
      tree.runNative(() {
        expect(findAnyProtonPath(), isNull);
        expect(hasAnyProtonInstall(), isFalse);
      });
    });
  });

  group('steamHomeForClient', () {
    test('resolves the confined home holding steamclient.so', () {
      final root = tree.addSteamRoot();
      tree.addSteamClient();

      expect(steamHomeForClient(root), tree.home);
    });

    test('is null when no steamclient home is found', () {
      final root = tree.addSteamRoot();
      expect(steamHomeForClient(root), isNull);
    });
  });
}
