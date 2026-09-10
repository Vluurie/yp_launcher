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
      final cmd = tree.runNative(() => buildProtonLaunchCommand(
            namsExe: '/run/bins/NAMS.exe',
            gameDir: gameDir,
            gameExe: gameExe,
            launcherDir: '/run/bins',
            protonPath: proton,
            namsArgs: namsRunArgs,
          ));

      expect(cmd, isNotNull);
      expect(cmd!.label, 'Proton');
      expect(cmd.args, ['run', '/run/bins/NAMS.exe', 'run', '--nier-path',
          'Z:${gameDir.replaceAll('/', '\\')}']);
      expect(cmd.env!['HOME'], tree.home);
      expect(cmd.env!['SteamAppId'], nierSteamAppId);
      expect(cmd.env!['STEAM_COMPAT_DATA_PATH'], endsWith('524220'));
    }, skip: skipOnWindows);

    test(
        'a game in a second library keeps compatdata there but points the '
        'client install at the real steam root', () {
      final tree = FakeSteamTree.create();
      addTearDown(tree.dispose);
      final root = tree.addSteamRoot();
      final library = p.join(tree.home, 'ssd', 'SteamLibrary');
      final gameDir = tree.addNier(library: library);
      final proton = tree.addProton('GE-Proton');

      final cmd = tree.runNative(() => buildProtonLaunchCommand(
            namsExe: '/run/bins/NAMS.exe',
            gameDir: gameDir,
            gameExe: p.join(gameDir, 'NieRAutomata.exe'),
            launcherDir: '/run/bins',
            protonPath: proton,
            namsArgs: namsRunArgs,
          ));

      expect(cmd, isNotNull);
      expect(cmd!.env!['STEAM_COMPAT_CLIENT_INSTALL_PATH'], root);
      expect(
        cmd.env!['STEAM_COMPAT_DATA_PATH'],
        p.join(library, 'steamapps', 'compatdata', nierSteamAppId),
      );
    }, skip: skipOnWindows);
  });

  group('formatLaunchCommandScript', () {
    const proton = LaunchCommand(
      command: '/steam/GE-Proton/proton',
      args: ['run', '/run/bins/NAMS.exe'],
      cwd: '/run/bins',
      label: 'Proton',
      env: {
        'STEAM_COMPAT_DATA_PATH': '/mnt/games/Lib/steamapps/compatdata/524220',
        'STEAM_COMPAT_CLIENT_INSTALL_PATH': '/home/d/.steam/steam',
      },
    );

    test('posix script exports the env inline before the command', () {
      expect(
        formatLaunchCommandScript(proton, windows: false),
        "cd '/run/bins' && "
        "STEAM_COMPAT_DATA_PATH='/mnt/games/Lib/steamapps/compatdata/524220' "
        "STEAM_COMPAT_CLIENT_INSTALL_PATH='/home/d/.steam/steam' "
        "'/steam/GE-Proton/proton' 'run' '/run/bins/NAMS.exe'",
      );
    });

    test('quotes env values with spaces', () {
      const spaced = LaunchCommand(
        command: 'wine',
        args: ['NAMS.exe'],
        cwd: '/r',
        label: 'Wine',
        env: {'WINEPREFIX': '/home/d/My Games/pfx'},
      );
      expect(
        formatLaunchCommandScript(spaced, windows: false),
        "cd '/r' && WINEPREFIX='/home/d/My Games/pfx' 'wine' 'NAMS.exe'",
      );
    });

    test('windows script sets the env line by line', () {
      const win = LaunchCommand(
        command: r'C:\yp\NAMS.exe',
        args: ['run'],
        cwd: r'C:\yp',
        label: 'Windows',
        env: {'SteamAppId': '524220'},
      );
      expect(
        formatLaunchCommandScript(win, windows: true),
        'cd /d "C:\\yp"\r\nset SteamAppId=524220\r\n"C:\\yp\\NAMS.exe" "run"',
      );
    });

    test('no env yields the bare command', () {
      const bare = LaunchCommand(
        command: 'wine',
        args: ['NAMS.exe'],
        cwd: '/r',
        label: 'Wine',
      );
      expect(
        formatLaunchCommandScript(bare, windows: false),
        "cd '/r' && 'wine' 'NAMS.exe'",
      );
    });
  });
}
