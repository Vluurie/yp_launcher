import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/platform/platform_adapter.dart';
import 'package:yp_launcher/services/platform/windows_adapter.dart';
import 'package:yp_launcher/services/thirdparty/game_mods_runtime.dart';
import 'package:yp_launcher/services/thirdparty/graphic_mods_runtime.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_migration.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';

class _GameNotRunning extends WindowsAdapter {
  @override
  Future<bool> isGameRunning() async => false;
}

void _write(String path, String content) {
  File(path)
    ..createSync(recursive: true)
    ..writeAsStringSync(content);
}

void main() {
  late Directory root;
  late String launcherDir;
  late String gameDir;
  late String legacy;

  setUp(() {
    root = Directory.systemTemp.createTempSync('yp_tp_migrate_');
    launcherDir = p.join(root.path, 'launcher');
    gameDir = p.join(root.path, 'game');
    legacy = p.join(launcherDir, 'thirdparty');
    Directory(launcherDir).createSync(recursive: true);
    Directory(p.join(gameDir, 'nams')).createSync(recursive: true);
    LauncherSetupService.setRuntimeDirForTest(launcherDir);
    PlatformAdapter.overrideCurrent(_GameNotRunning());
  });

  tearDown(() {
    LauncherSetupService.setRuntimeDirForTest(null);
    PlatformAdapter.overrideCurrent(null);
    try {
      root.deleteSync(recursive: true);
    } catch (_) {}
  });

  test('nothing to do without the old folder', () async {
    final result = await ThirdPartyMigration.run(gameDir);
    expect(result.movedFiles, 0);
    expect(result.leftBehind, isEmpty);
  });

  test('every kind lands in mod folders and the old folder is gone', () async {
    _write(p.join(legacy, 'game', 'celestial.dll'), 'celestial');
    _write(p.join(legacy, 'game', 'celestial.ini'), 'x=1');
    _write(p.join(legacy, 'game', 'wax.dll'), 'wax');
    _write(p.join(legacy, 'game', 'game.toml'), '["wax.dll"]\ndisabled = true\n');
    _write(p.join(legacy, 'graphic', 'd3d11.dll'), 'dlaa');
    _write(p.join(legacy, 'graphic', 'nvngx_dlss.dll'), 'dlss');
    _write(p.join(legacy, 'graphic', 'NIER_DLAA.ini'), 'x=1');
    _write(p.join(legacy, 'graphic', 'disabled', 'dxgi.dll'), 'parked');
    _write(p.join(legacy, 'reshade', 'ReShade64.dll'), 'reshade');
    _write(
      p.join(legacy, 'reshade', 'reshade-shaders', 'Shaders', 'a.fx'),
      'fx',
    );
    _write(p.join(legacy, '3dmigoto', '3dmigoto.dll'), 'migoto');
    _write(p.join(legacy, '3dmigoto', 'nvapi64.dll'), 'nvapi');

    final result = await ThirdPartyMigration.run(gameDir);

    expect(result.leftBehind, isEmpty);
    expect(result.movedFiles, 12);
    expect(Directory(legacy).existsSync(), isFalse);

    final game = ThirdPartyPaths.gameMods(gameDir);
    expect(File(p.join(game, 'celestial', 'celestial.dll')).existsSync(), isTrue);
    expect(File(p.join(game, 'celestial', 'celestial.ini')).existsSync(), isTrue);
    expect(File(p.join(game, 'wax', 'wax.dll')).existsSync(), isTrue);
    expect(File(p.join(game, 'game.toml')).existsSync(), isTrue,
        reason: 'game.toml stays beside the mod folders');
    final gameStatus = await const GameModsRuntime().status(gameDir);
    expect(
      gameStatus.gameModsInfo!.mods.firstWhere((m) => m.fileName == 'wax.dll')
          .disabled,
      isTrue,
    );

    final graphic = p.join(ThirdPartyPaths.graphicMods(gameDir), 'd3d11+dxgi');
    for (final name in ['d3d11.dll', 'nvngx_dlss.dll', 'NIER_DLAA.ini']) {
      expect(File(p.join(graphic, name)).existsSync(), isTrue, reason: name);
    }
    expect(
      File(p.join(ThirdPartyPaths.graphicModsDisabled(gameDir), 'd3d11+dxgi',
              'dxgi.dll'))
          .existsSync(),
      isTrue,
      reason: 'a turned-off DLL stays turned off',
    );
    final graphicMods =
        (await const GraphicModsRuntime().status(gameDir)).graphicModsInfo!.mods;
    expect(graphicMods.where((m) => m.disabled).map((m) => m.fileName),
        ['dxgi.dll']);

    final reshade = const ReShadeRuntime().installDir(gameDir);
    expect(File(p.join(reshade, 'ReShade64.dll')).existsSync(), isTrue);
    expect(
      File(p.join(reshade, 'reshade-shaders', 'Shaders', 'a.fx')).existsSync(),
      isTrue,
    );

    final migoto = const MigotoRuntime().installDir(gameDir);
    expect(File(p.join(migoto, '3dmigoto.dll')).existsSync(), isTrue);
    expect(File(p.join(migoto, 'nvapi64.dll')).existsSync(), isTrue);
  });

  test('a file whose new copy differs is kept in the old folder', () async {
    _write(p.join(legacy, 'game', 'celestial.dll'), 'old build');
    _write(
      p.join(ThirdPartyPaths.gameMods(gameDir), 'Celestial', 'celestial.dll'),
      'new build',
    );

    final result = await ThirdPartyMigration.run(gameDir);

    final old = p.join(legacy, 'game', 'celestial.dll');
    expect(result.leftBehind, [old]);
    expect(File(old).readAsStringSync(), 'old build');
    expect(
      File(p.join(ThirdPartyPaths.gameMods(gameDir), 'Celestial',
              'celestial.dll'))
          .readAsStringSync(),
      'new build',
    );
  });

  test('an identical copy in the new folder completes the move', () async {
    _write(p.join(legacy, 'game', 'celestial.dll'), 'same');
    _write(
      p.join(ThirdPartyPaths.gameMods(gameDir), 'celestial', 'celestial.dll'),
      'same',
    );

    final result = await ThirdPartyMigration.run(gameDir);

    expect(result.leftBehind, isEmpty);
    expect(Directory(legacy).existsSync(), isFalse);
  });

  test('loose files stay when they belong to none of several mods', () async {
    _write(p.join(legacy, 'game', 'a.dll'), 'a');
    _write(p.join(legacy, 'game', 'b.dll'), 'b');
    _write(p.join(legacy, 'game', 'notes.txt'), 'mine');

    final result = await ThirdPartyMigration.run(gameDir);

    final notes = p.join(legacy, 'game', 'notes.txt');
    expect(result.leftBehind, [notes]);
    expect(File(notes).existsSync(), isTrue);
    final game = ThirdPartyPaths.gameMods(gameDir);
    expect(File(p.join(game, 'a', 'a.dll')).existsSync(), isTrue);
    expect(File(p.join(game, 'b', 'b.dll')).existsSync(), isTrue);
  });

  test('a running game blocks the move', () async {
    PlatformAdapter.overrideCurrent(_GameRunning());
    _write(p.join(legacy, 'game', 'celestial.dll'), 'celestial');

    final result = await ThirdPartyMigration.run(gameDir);

    expect(result.movedFiles, 0);
    expect(File(p.join(legacy, 'game', 'celestial.dll')).existsSync(), isTrue);
  });
}

class _GameRunning extends WindowsAdapter {
  @override
  Future<bool> isGameRunning() async => true;
}
