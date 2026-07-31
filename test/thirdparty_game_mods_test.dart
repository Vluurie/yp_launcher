import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/thirdparty/game_mods_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_classifier.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_paths.dart';
import 'package:yp_launcher/services/toml_service.dart';

const _runtime = GameModsRuntime();

/// A DLL with no ReShade/3DMigoto marker bytes — what a game-offset mod
/// (celestial and friends) looks like to the classifier.
void _writePlainDll(String path, {String body = 'celestial path timing'}) {
  File(path)
    ..createSync(recursive: true)
    ..writeAsBytesSync([
      0x4D, 0x5A, 0x90, 0x00,
      ...body.codeUnits,
    ]);
}

void _writeMarkedDll(String path, String marker) {
  File(path)
    ..createSync(recursive: true)
    ..writeAsBytesSync([0x4D, 0x5A, 0x90, 0x00, ...marker.codeUnits]);
}

Directory _freshGame() {
  final dir = Directory.systemTemp.createTempSync('yp_gamemods_');
  Directory(p.join(dir.path, 'nams')).createSync(recursive: true);
  File(p.join(dir.path, 'nams', 'nams.toml'))
      .writeAsStringSync('disable_game_mods_loading = true\n');
  return dir;
}

Map<String, dynamic> _readNams(String gameDir) => TomlService.parse(
      File(p.join(gameDir, 'nams', 'nams.toml')).readAsStringSync(),
    );

Map<String, dynamic> _readGameToml(String gameDir) => TomlService.parse(
      File(_runtime.configPath(gameDir)).readAsStringSync(),
    );

void main() {
  late Directory launcherDir;

  setUp(() {
    launcherDir = Directory.systemTemp.createTempSync('yp_gm_launcher_');
    LauncherSetupService.setRuntimeDirForTest(launcherDir.path);
  });

  tearDown(() {
    LauncherSetupService.setRuntimeDirForTest(null);
    try {
      launcherDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  group('classification', () {
    test('a loose game-offset DLL is a gameMod', () {
      final drop = Directory.systemTemp.createTempSync('yp_gm_drop_');
      addTearDown(() => drop.deleteSync(recursive: true));
      _writePlainDll(p.join(drop.path, 'celestial.dll'));

      final c = ThirdPartyClassifier.classify(drop.path);

      expect(c.kind, ThirdPartyKind.gameMod);
      expect(c.gameModDlls.map(p.basename), ['celestial.dll']);
    });

    test('a ReShade DLL is never claimed as a game mod', () {
      final drop = Directory.systemTemp.createTempSync('yp_gm_rs_');
      addTearDown(() => drop.deleteSync(recursive: true));
      _writeMarkedDll(p.join(drop.path, 'dxgi.dll'), 'reshade.me crosire');

      final c = ThirdPartyClassifier.classify(drop.path);

      expect(c.kind, ThirdPartyKind.reshadeWholeInstall);
    });

    test('a 3DMigoto DLL is never claimed as a game mod', () {
      final drop = Directory.systemTemp.createTempSync('yp_gm_mg_');
      addTearDown(() => drop.deleteSync(recursive: true));
      _writeMarkedDll(p.join(drop.path, 'd3d11.dll'), 'HackerDevice 3DMigoto');

      final c = ThirdPartyClassifier.classify(drop.path);

      expect(c.kind, ThirdPartyKind.migoto);
    });

    test('several DLLs in one drop are all collected', () {
      final drop = Directory.systemTemp.createTempSync('yp_gm_multi_');
      addTearDown(() => drop.deleteSync(recursive: true));
      _writePlainDll(p.join(drop.path, 'b_mod.dll'));
      _writePlainDll(p.join(drop.path, 'a_mod.dll'));

      final c = ThirdPartyClassifier.classify(drop.path);

      expect(c.kind, ThirdPartyKind.gameMod);
      expect(c.gameModDlls.map(p.basename), ['a_mod.dll', 'b_mod.dll']);
    });
  });

  group('install', () {
    test('the DLL lands in thirdparty/game and the flag is switched on',
        () async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_i_');
      addTearDown(() {
        game.deleteSync(recursive: true);
        drop.deleteSync(recursive: true);
      });
      _writePlainDll(p.join(drop.path, 'celestial.dll'));

      final c = ThirdPartyClassifier.classify(drop.path);
      final result = await _runtime.install(game.path, c);

      expect(result.ok, isTrue);
      expect(
        File(p.join(ThirdPartyPaths.gameMods(), 'celestial.dll')).existsSync(),
        isTrue,
      );
      expect(_readNams(game.path)[NamsFields.disableGameModsLoading.key],
          isFalse);
    });

    test('installed DLLs are reported by status', () async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_s_');
      addTearDown(() {
        game.deleteSync(recursive: true);
        drop.deleteSync(recursive: true);
      });
      _writePlainDll(p.join(drop.path, 'celestial.dll'));

      await _runtime.install(game.path, ThirdPartyClassifier.classify(drop.path));
      final status = await _runtime.status(game.path);

      expect(status.installed, isTrue);
      expect(status.enabled, isTrue);
      final mods = status.gameModsInfo!.mods;
      expect(mods.single.fileName, 'celestial.dll');
      expect(mods.single.disabled, isFalse);
      expect(mods.single.sizeLabel, isNotNull);
    });

    test('an empty status is reported when nothing is installed', () async {
      final game = _freshGame();
      addTearDown(() => game.deleteSync(recursive: true));

      final status = await _runtime.status(game.path);

      expect(status.installed, isFalse);
      expect(status.gameModsInfo!.mods, isEmpty);
    });

    test('re-installing a changed DLL is offered as an update', () async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_u_');
      addTearDown(() {
        game.deleteSync(recursive: true);
        drop.deleteSync(recursive: true);
      });
      _writePlainDll(p.join(drop.path, 'celestial.dll'), body: 'v1');
      await _runtime.install(game.path, ThirdPartyClassifier.classify(drop.path));

      _writePlainDll(p.join(drop.path, 'celestial.dll'), body: 'v2 is longer');
      final update = await _runtime.wouldUpdate(
        game.path,
        ThirdPartyClassifier.classify(drop.path),
      );

      expect(update, isNotNull);
      expect(update!.runtime, ThirdPartyRuntime.gameMods);
    });

    test('an identical DLL is not offered as an update', () async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_nu_');
      addTearDown(() {
        game.deleteSync(recursive: true);
        drop.deleteSync(recursive: true);
      });
      _writePlainDll(p.join(drop.path, 'celestial.dll'));
      final c = ThirdPartyClassifier.classify(drop.path);
      await _runtime.install(game.path, c);

      expect(await _runtime.wouldUpdate(game.path, c), isNull);
    });
  });

  group('per-DLL disable via game.toml', () {
    Future<Directory> installed() async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_d_');
      addTearDown(() => drop.deleteSync(recursive: true));
      _writePlainDll(p.join(drop.path, 'celestial.dll'));
      await _runtime.install(
          game.path, ThirdPartyClassifier.classify(drop.path));
      return game;
    }

    test('disabling writes a per-DLL section and shows up in status', () async {
      final game = await installed();
      addTearDown(() => game.deleteSync(recursive: true));

      await _runtime.setModDisabled(game.path, 'celestial.dll', true);

      final section =
          _readGameToml(game.path)['celestial.dll'] as Map<String, dynamic>;
      expect(section['disabled'], isTrue);

      final status = await _runtime.status(game.path);
      expect(status.gameModsInfo!.mods.single.disabled, isTrue);
      expect(status.gameModsInfo!.enabledCount, 0);
      expect(
        File(p.join(ThirdPartyPaths.gameMods(), 'celestial.dll')).existsSync(),
        isTrue,
        reason: 'disabling must not delete the DLL',
      );
    });

    test('re-enabling flips the flag back', () async {
      final game = await installed();
      addTearDown(() => game.deleteSync(recursive: true));

      await _runtime.setModDisabled(game.path, 'celestial.dll', true);
      await _runtime.setModDisabled(game.path, 'celestial.dll', false);

      final status = await _runtime.status(game.path);
      expect(status.gameModsInfo!.mods.single.disabled, isFalse);
    });

    test('an existing game.toml keeps its other entries and comments',
        () async {
      final game = await installed();
      addTearDown(() => game.deleteSync(recursive: true));
      File(_runtime.configPath(game.path)).writeAsStringSync('''# keep me
[defaults]
size_of_image = 26177536

["other.dll"]
disabled = true
''');

      await _runtime.setModDisabled(game.path, 'celestial.dll', true);

      final raw = File(_runtime.configPath(game.path)).readAsStringSync();
      final cfg = TomlService.parse(raw);
      expect(raw, contains('# keep me'));
      expect((cfg['defaults'] as Map)['size_of_image'], 26177536);
      expect((cfg['other.dll'] as Map)['disabled'], isTrue);
      expect((cfg['celestial.dll'] as Map)['disabled'], isTrue);
    });

    test('the defaults section is not mistaken for a DLL', () async {
      final game = await installed();
      addTearDown(() => game.deleteSync(recursive: true));
      File(_runtime.configPath(game.path)).writeAsStringSync('''[defaults]
disabled = true
''');

      final status = await _runtime.status(game.path);

      expect(status.gameModsInfo!.mods.single.disabled, isFalse,
          reason: '[defaults] must not disable an unrelated DLL');
    });

    test('a case-mismatched section still matches the DLL', () async {
      final game = await installed();
      addTearDown(() => game.deleteSync(recursive: true));
      File(_runtime.configPath(game.path))
          .writeAsStringSync('["CELESTIAL.DLL"]\ndisabled = true\n');

      final status = await _runtime.status(game.path);

      expect(status.gameModsInfo!.mods.single.disabled, isTrue);
    });
  });

  group('removal', () {
    test('removing one DLL leaves the others alone', () async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_r_');
      addTearDown(() {
        game.deleteSync(recursive: true);
        drop.deleteSync(recursive: true);
      });
      _writePlainDll(p.join(drop.path, 'celestial.dll'));
      _writePlainDll(p.join(drop.path, 'other.dll'));
      await _runtime.install(
          game.path, ThirdPartyClassifier.classify(drop.path));

      await _runtime.removeMod(game.path, 'celestial.dll');

      final status = await _runtime.status(game.path);
      expect(status.gameModsInfo!.mods.map((m) => m.fileName), ['other.dll']);
    });

    test('removing the runtime clears the DLLs and sets the disable flag',
        () async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_ra_');
      addTearDown(() {
        game.deleteSync(recursive: true);
        drop.deleteSync(recursive: true);
      });
      _writePlainDll(p.join(drop.path, 'celestial.dll'));
      await _runtime.install(
          game.path, ThirdPartyClassifier.classify(drop.path));

      await _runtime.remove(game.path);

      final status = await _runtime.status(game.path);
      expect(status.installed, isFalse);
      expect(
          _readNams(game.path)[NamsFields.disableGameModsLoading.key], isTrue);
    });

    test('game.toml survives a runtime removal', () async {
      final game = _freshGame();
      final drop = Directory.systemTemp.createTempSync('yp_gm_rc_');
      addTearDown(() {
        game.deleteSync(recursive: true);
        drop.deleteSync(recursive: true);
      });
      _writePlainDll(p.join(drop.path, 'celestial.dll'));
      await _runtime.install(
          game.path, ThirdPartyClassifier.classify(drop.path));
      await _runtime.setModDisabled(game.path, 'celestial.dll', true);

      await _runtime.remove(game.path);

      expect(File(_runtime.configPath(game.path)).existsSync(), isTrue,
          reason: 'per-DLL settings should outlive the DLLs themselves');
    });
  });

  test('the install dir matches what NAMS scans', () {
    expect(
      p.basename(ThirdPartyPaths.gameMods()),
      AppStrings.gameModsDirName,
    );
    expect(
      p.basename(p.dirname(ThirdPartyPaths.gameMods())),
      AppStrings.thirdPartyDirName,
    );
  });
}
