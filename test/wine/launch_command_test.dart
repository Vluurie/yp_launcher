import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/wine/crossover.dart';
import 'package:yp_launcher/services/wine/launch_command.dart';
import 'package:yp_launcher/services/wine/proton.dart';

import '../support/posix_only.dart';
import 'fake_steam_tree.dart';

const _bottlePath =
    '/Users/d/Library/Application Support/CrossOver/Bottles/Steam';
const _gameDir =
    '$_bottlePath/drive_c/Program Files (x86)/Steam/steamapps/common/NieRAutomata';
const _launcherDir =
    '/Users/d/Library/Application Support/com.vluurie.yplauncher/bins';
const _namsExe = '$_launcherDir/NAMS.exe';

const _bottle = CrossOverBottle(name: 'Steam', path: _bottlePath);

void main() {
  group('buildCrossOverLaunchCommand', () {
    final cmd = buildCrossOverLaunchCommand(
      namsExe: _namsExe,
      gameDir: _gameDir,
      launcherDir: _launcherDir,
      wineBinary: '/Applications/CrossOver.app/Contents/SharedSupport'
          '/CrossOver/bin/wine',
      bottle: _bottle,
      prefix: _bottlePath,
      namsArgs: namsRunArgs,
    );

    test('runs the wine wrapper, not the exe', () {
      expect(
        cmd.command,
        '/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine',
      );
    });

    test('passes the bottle and the launcher workdir', () {
      expect(cmd.args.sublist(0, 4), [
        '--bottle',
        'Steam',
        '--workdir',
        _launcherDir,
      ]);
    });

    test('the exe stays a host path so wine can translate it', () {
      expect(cmd.args[4], _namsExe);
    });

    test('--nier-path is a Windows path because NAMS reads it Windows-side', () {
      expect(cmd.args.sublist(5), [
        'run',
        '--nier-path',
        r'C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata',
      ]);
    });

    test('sets both CX_BOTTLE and WINEPREFIX', () {
      expect(cmd.env!['CX_BOTTLE'], 'Steam');
      expect(cmd.env!['WINEPREFIX'], _bottlePath);
      expect(cmd.env!['WINEDEBUG'], '-all');
    });

    test('names the bottle in the label', () {
      expect(cmd.label, 'CrossOver Wine (Steam)');
    });
  });

  group('buildPlainWineLaunchCommand', () {
    test('passes the exe first with no bottle flags', () {
      final cmd = buildPlainWineLaunchCommand(
        namsExe: _namsExe,
        gameDir: _gameDir,
        launcherDir: _launcherDir,
        wineBinary: '/usr/bin/wine',
        prefix: _bottlePath,
        namsArgs: namsRunArgs,
      );

      expect(cmd.command, '/usr/bin/wine');
      expect(cmd.args, [
        _namsExe,
        'run',
        '--nier-path',
        r'C:\Program Files (x86)\Steam\steamapps\common\NieRAutomata',
      ]);
      expect(cmd.env!['WINEPREFIX'], _bottlePath);
      expect(cmd.env!.containsKey('CX_BOTTLE'), isFalse);
    });

    test('omits WINEPREFIX when there is no prefix', () {
      final cmd = buildPlainWineLaunchCommand(
        namsExe: _namsExe,
        gameDir: _gameDir,
        launcherDir: _launcherDir,
        wineBinary: '/usr/bin/wine',
        namsArgs: namsRunArgs,
      );

      expect(cmd.env!.containsKey('WINEPREFIX'), isFalse);
    });
  });

  group('buildNativeLaunchCommand', () {
    test('runs the exe directly with a backslash path', () {
      final cmd = buildNativeLaunchCommand(
        namsExe: r'C:\yp\bins\NAMS.exe',
        gameDir: r'D:/Games/NieRAutomata',
        launcherDir: r'C:\yp\bins',
        namsArgs: namsRunArgs,
      );

      expect(cmd.command, r'C:\yp\bins\NAMS.exe');
      expect(cmd.args, ['run', '--nier-path', r'D:\Games\NieRAutomata']);
      expect(cmd.env, isNull);
      expect(cmd.label, 'Windows');
    });

    test('namsVerifyArgs produces the verify subcommand with --json', () {
      final cmd = buildNativeLaunchCommand(
        namsExe: r'C:\yp\bins\NAMS.exe',
        gameDir: r'D:/Games/NieRAutomata',
        launcherDir: r'C:\yp\bins',
        namsArgs: namsVerifyArgs,
      );
      expect(cmd.args,
          ['verify', '--nier-path', r'D:\Games\NieRAutomata', '--json']);
    });
  });

  group('buildProtonLaunchCommand', () {
    test('a missing proton binary yields no command', () {
      expect(
        buildProtonLaunchCommand(
          namsExe: _namsExe,
          gameDir: _gameDir,
          gameExe: '$_gameDir/NieRAutomata.exe',
          launcherDir: _launcherDir,
          protonPath: '/nonexistent/proton',
          namsArgs: namsRunArgs,
        ),
        isNull,
      );
    });

    test('injects the confined HOME when the game is a native Steam install',
        () {
      final tree = FakeSteamTree.create();
      addTearDown(tree.dispose);
      tree.addSteamRoot();
      tree.addSteamClient();
      final gameDir = tree.addNier();
      final proton = tree.addProton('GE-Proton');

      final gameExe = p.join(gameDir, 'NieRAutomata.exe');
      final cmd = buildProtonLaunchCommand(
        namsExe: '/run/bins/NAMS.exe',
        gameDir: gameDir,
        gameExe: gameExe,
        launcherDir: '/run/bins',
        protonPath: proton,
        namsArgs: namsRunArgs,
      );

      expect(cmd, isNotNull);
      expect(cmd!.label, 'Proton');
      expect(cmd.args, ['run', '/run/bins/NAMS.exe', 'run', '--nier-path',
          'Z:${gameDir.replaceAll('/', '\\')}']);
      expect(cmd.env!['HOME'], tree.home);
      expect(cmd.env!['SteamAppId'], nierSteamAppId);
      expect(cmd.env!['STEAM_COMPAT_DATA_PATH'], endsWith('524220'));
    }, skip: skipOnWindows);
  });
}
