import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/l10n/app_localizations_en.dart';
import 'package:yp_launcher/services/platform/linux_adapter.dart';
import 'package:yp_launcher/services/platform/macos_adapter.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/platform/wine_adapter_base.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';
import 'package:yp_launcher/services/wine/proton.dart';

import '../support/posix_only.dart';
import '../wine/fake_bottle.dart';
import '../wine/fake_steam_tree.dart';

final _l10n = AppLocalizationsEn();

void main() {
  group('PlatformAdapter.current', () {
    tearDown(() => PlatformAdapter.overrideCurrent(null));

    test('resolves to the host adapter', () {
      final adapter = PlatformAdapter.current;
      expect(adapter, Platform.isMacOS ? isA<MacOSAdapter>() : isNotNull);
    });

    test('can be overridden for tests', () {
      final linux = LinuxAdapter();
      PlatformAdapter.overrideCurrent(linux);
      expect(PlatformAdapter.current, same(linux));
    });
  });

  group('seven zip', () {
    test('macOS looks in the runtime dir first, then Homebrew', () {
      expect(MacOSAdapter().sevenZipCandidates('/run/bins'), [
        '/run/bins/7zz',
        '/opt/homebrew/bin/7zz',
        '/usr/local/bin/7zz',
      ]);
    });

    test('linux looks in the runtime dir first, then the system', () {
      expect(LinuxAdapter().sevenZipCandidates('/run/bins'), [
        '/run/bins/7zz',
        '/usr/bin/7zz',
        '/usr/local/bin/7zz',
      ]);
    });
  }, skip: skipOnWindows);

  group('resolveRuntimeDir', () {
    test('linux honours XDG', () async {
      final dir = await LinuxAdapter().resolveRuntimeDir();
      expect(dir, endsWith(p.join('yp_launcher', 'bins')));
    });
  });

  group('LinuxAdapter', () {
    test('is flagged experimental', () {
      expect(LinuxAdapter().isExperimental, isTrue);
      expect(MacOSAdapter().isExperimental, isFalse);
    });

    test('discoverFast finds a native Steam install', () async {
      final tree = FakeSteamTree.create();
      addTearDown(tree.dispose);
      tree.addSteamRoot();
      final dir = tree.addNier();

      overrideSteamRoots = [tree.steamRoot];
      addTearDown(() => overrideSteamRoots = null);

      final found = await LinuxAdapter().discoverFast();
      expect(found.map((i) => i.path), contains(dir));
    });

    test('accepts a native Steam exe path', () {
      expect(
        LinuxAdapter().rejectGameSelection(
          '/home/u/.local/share/Steam/steamapps/common/NieRAutomata'
          '/NieRAutomata.exe',
          _l10n,
        ),
        isNull,
      );
    }, skip: skipOnWindows);

    test('resolveNamsSettingsPath lands in the compat prefix', () async {
      final path = await LinuxAdapter().resolveNamsSettingsPath(
        '/home/u/.local/share/Steam/steamapps/common/NieRAutomata',
      );

      expect(path, contains(p.join('steamapps', 'compatdata', '524220', 'pfx')));
      expect(path, endsWith(p.join('NAMS', 'settings.json')));
    }, skip: skipOnWindows);
  });

  group('rejectGameSelection', () {
    final adapter = MacOSAdapter();

    test('accepts an exe inside a prefix', () {
      expect(
        adapter.rejectGameSelection(
          '/u/Library/Application Support/CrossOver/Bottles/Steam/drive_c'
          '/game/NieRAutomata.exe',
          _l10n,
        ),
        isNull,
      );
    });

    test('rejects an exe outside any prefix', () {
      final reason =
          adapter.rejectGameSelection('/Users/d/Games/NieRAutomata.exe', _l10n);

      expect(reason, isNotNull);
      expect(reason, contains('NieRAutomata.exe'));
    });
  });

  group('resolveNamsSettingsPath', () {
    final adapter = MacOSAdapter();
    late FakeBottleTree tree;

    setUp(() => tree = FakeBottleTree.create());
    tearDown(() => tree.dispose());

    test('lands in the bottle Roaming dir', () async {
      final bottle = tree.addBottle('Steam');
      final gameDir = p.join(bottle, 'drive_c', 'game');

      expect(
        await adapter.resolveNamsSettingsPath(gameDir),
        p.join(bottle, 'drive_c', 'users', 'crossover', 'AppData', 'Roaming',
            'NAMS', 'settings.json'),
      );
    }, skip: skipOnWindows);

    test('is null before a game dir is known', () async {
      expect(await adapter.resolveNamsSettingsPath(null), isNull);
    });

    test('is null for a game dir outside any prefix', () async {
      expect(await adapter.resolveNamsSettingsPath('/Users/d/Games'), isNull);
    });
  });

  group('buildLaunchCommand', () {
    late FakeBottleTree tree;
    late WineAdapterBase adapter;

    setUp(() {
      tree = FakeBottleTree.create();
      adapter = MacOSAdapter();
    });
    tearDown(() => tree.dispose());

    test('drives CrossOver with the bottle and both path forms', () async {
      final bottle = tree.addBottle('Steam');
      final exe = tree.addNier('Steam');
      final gameDir = p.dirname(exe);
      Directory(p.join(bottle, 'dosdevices', 'z:')).createSync(recursive: true);
      final wine = tree.addWineScript();

      final cmd = await tree.run(
        () => adapter.buildLaunchCommand(
          namsExe: '/run/bins/NAMS.exe',
          gameDir: gameDir,
          gameExe: exe,
          launcherDir: '/run/bins',
          l10n: _l10n,
        ),
        wineCommand: wine,
      );

      expect(cmd.command, wine);
      expect(cmd.args.sublist(0, 4), ['--bottle', 'Steam', '--workdir', '/run/bins']);
      expect(cmd.args[4], '/run/bins/NAMS.exe');
      expect(cmd.args.sublist(5), [
        'run',
        '--nier-path',
        r'C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata',
      ]);
      expect(cmd.env!['CX_BOTTLE'], 'Steam');
      expect(cmd.env!['WINEPREFIX'], bottle);
      expect(cmd.label, 'CrossOver Wine (Steam)');
    }, skip: skipOnWindows);

    test('refuses a prefix without a Z: drive', () async {
      final exe = tree.addNier('Steam');
      tree.addBottle('Steam');
      final wine = tree.addWineScript();

      await expectLater(
        tree.run(
          () => adapter.buildLaunchCommand(
            namsExe: '/run/bins/NAMS.exe',
            gameDir: p.dirname(exe),
            gameExe: exe,
            launcherDir: '/run/bins',
            l10n: _l10n,
          ),
          wineCommand: wine,
        ),
        throwsA(isA<LaunchUnavailable>()),
      );
    });

    test('buildNamsCommand routes verify args through the same runtime',
        () async {
      final bottle = tree.addBottle('Steam');
      final exe = tree.addNier('Steam');
      final gameDir = p.dirname(exe);
      Directory(p.join(bottle, 'dosdevices', 'z:')).createSync(recursive: true);
      final wine = tree.addWineScript();

      final cmd = await tree.run(
        () => adapter.buildNamsCommand(
          namsArgs: namsVerifyArgs,
          namsExe: '/run/bins/NAMS.exe',
          gameDir: gameDir,
          gameExe: exe,
          launcherDir: '/run/bins',
          l10n: _l10n,
        ),
        wineCommand: wine,
      );

      expect(cmd.command, wine);
      expect(cmd.args[4], '/run/bins/NAMS.exe');
      expect(cmd.args.sublist(5), [
        'verify',
        '--nier-path',
        r'C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata',
        '--json',
      ]);
    }, skip: skipOnWindows);
  });
}
