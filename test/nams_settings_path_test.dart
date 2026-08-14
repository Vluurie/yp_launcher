import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/nams_settings_service.dart';

import 'support/posix_only.dart';
import 'wine/fake_bottle.dart';

void main() {
  group('NAMS settings resolve inside a wine prefix', () {
  late FakeBottleTree tree;

  setUp(() => tree = FakeBottleTree.create());
  tearDown(() => tree.dispose());

  test('settings land where NAMS reads them inside the prefix', () async {
    final bottle = tree.addBottle('Steam');
    final gameDir = p.join(bottle, 'drive_c', 'game');

    final path = await NamsSettingsService.resolveSettingsPath(gameDir);

    expect(
      path,
      p.join(bottle, 'drive_c', 'users', 'crossover', 'AppData', 'Roaming',
          'NAMS', 'settings.json'),
    );
  });

  test('saving without a game dir is refused rather than written elsewhere',
      () async {
    expect(await NamsSettingsService.saveSettings({'a': 1}, null), isFalse);
  });

  test('saving for a dir outside any prefix is refused', () async {
    expect(
      await NamsSettingsService.saveSettings({'a': 1}, '/Users/d/Games'),
      isFalse,
    );
  });

  test('an unknown location yields defaults, not a crash', () async {
    final settings = await NamsSettingsService.loadSettings(null);
    expect(settings['firstPlaythrough'], isTrue);
  });

  test('a round trip through the prefix preserves values', () async {
    final bottle = tree.addBottle('Steam');
    final gameDir = p.join(bottle, 'drive_c', 'game');

    final saved = await NamsSettingsService.saveSettings(
      {'firstPlaythrough': false, 'shadersEnabled': false},
      gameDir,
    );
    expect(saved, isTrue);

    final loaded = await NamsSettingsService.loadSettings(gameDir);
    expect(loaded['firstPlaythrough'], isFalse);
    expect(loaded['shadersEnabled'], isFalse);
  });

  test('impeller defaults to true when the key is absent', () async {
    final bottle = tree.addBottle('Steam');
    final gameDir = p.join(bottle, 'drive_c', 'game');

    await NamsSettingsService.saveSettings({'firstPlaythrough': false}, gameDir);

    expect(await NamsSettingsService.loadImpeller(gameDir), isTrue);
  });

  test('impeller defaults to true when the key is not a bool', () async {
    final bottle = tree.addBottle('Steam');
    final gameDir = p.join(bottle, 'drive_c', 'game');

    await NamsSettingsService.saveSettings({'impeller': 'yes'}, gameDir);

    expect(await NamsSettingsService.loadImpeller(gameDir), isTrue);
  });

  test('writing impeller keeps every other key intact', () async {
    final bottle = tree.addBottle('Steam');
    final gameDir = p.join(bottle, 'drive_c', 'game');

    await NamsSettingsService.saveSettings({
      'firstPlaythrough': false,
      'overlayLanguage': 'de',
      'keybinds': {
        'main': {'yorha_protocol': 'F2'},
      },
      'randomizerConfig': {'autoStartOnStartup': true},
    }, gameDir);

    expect(await NamsSettingsService.saveImpeller(gameDir, false), isTrue);

    final loaded = await NamsSettingsService.loadSettings(gameDir);
    expect(loaded['impeller'], isFalse);
    expect(loaded['firstPlaythrough'], isFalse);
    expect(loaded['overlayLanguage'], 'de');
    expect((loaded['keybinds'] as Map)['main']['yorha_protocol'], 'F2');
    expect((loaded['randomizerConfig'] as Map)['autoStartOnStartup'], isTrue);
    expect(await NamsSettingsService.loadImpeller(gameDir), isFalse);
  });
  }, skip: skipOnWindows);
}
