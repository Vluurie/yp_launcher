import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/detection/game_detection.dart';
import 'package:yp_launcher/services/diagnostics_service.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';

const _dlcBytes = 930 * 1024 * 1024;

void _makeSizedFile(String path, int size) {
  final raf = File(path).openSync(mode: FileMode.write);
  try {
    raf.setPositionSync(size - 1);
    raf.writeByteSync(0);
  } finally {
    raf.closeSync();
  }
}

Directory _fakeGameDir() {
  final dir = Directory.systemTemp.createTempSync('yp_diag_');
  final data = Directory(p.join(dir.path, 'data'))..createSync(recursive: true);
  File(p.join(data.path, 'data000.cpk')).writeAsBytesSync(List.filled(16, 0));
  _makeSizedFile(p.join(data.path, 'data100.cpk'), _dlcBytes);
  File(p.join(data.path, 'hack.dll')).writeAsBytesSync(List.filled(8, 0));
  File(p.join(data.path, 'enemy.dtt')).writeAsBytesSync(List.filled(8, 0));
  final sound = Directory(p.join(data.path, 'sound'))..createSync();
  File(p.join(sound.path, 'bgm.wsp')).writeAsBytesSync(List.filled(8, 0));
  File(p.join(dir.path, AppStrings.gameExeName))
      .writeAsBytesSync(List.filled(GameDetection.windows10Size, 0));
  File(p.join(dir.path, 'steam_api64.dll')).writeAsBytesSync(List.filled(8, 0));
  final nams = Directory(p.join(dir.path, 'nams'))..createSync();
  File(p.join(nams.path, 'nams.toml'))
      .writeAsStringSync('disable_splash_screen = true\n');
  File(p.join(nams.path, 'lodmod.toml')).writeAsStringSync('');
  File(p.join(nams.path, 'texture_injection.toml')).writeAsStringSync('');
  return dir;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  SharedPreferences.setMockInitialValues({});

  late Directory launcherDir;
  setUp(() {
    launcherDir = Directory.systemTemp.createTempSync('yp_diag_launcher_');
    LauncherSetupService.setRuntimeDirForTest(launcherDir.path);
  });
  tearDown(() {
    LauncherSetupService.setRuntimeDirForTest(null);
    try {
      launcherDir.deleteSync(recursive: true);
    } catch (_) {}
  });

  test('vanilla drops list loose files in data/, not cpk or subfolders',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    final paths = r.vanillaDropped.map((f) => f.path).toList();

    expect(paths, contains('data/hack.dll'));
    expect(paths, contains('data/enemy.dtt'));
    expect(paths, isNot(contains('data/data000.cpk')),
        reason: 'core vanilla cpk must not be reported as a drop');
    expect(paths, isNot(contains('data/sound/bgm.wsp')),
        reason: 'vanilla subfolder contents are never scanned');
    expect(
      r.vanillaDropped
          .firstWhere((f) => f.path == 'data/hack.dll')
          .bucket,
      'data-root-loose',
    );
  });

  test('game identity detects DLC and a supported exe', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    expect(r.gameIdentity.hasDlc, isTrue);
    expect(r.gameIdentity.exeSize, GameDetection.windows10Size);
    expect(r.gameIdentity.exeSizeMatchesWin10, isTrue);
    expect(ExeVariant.values.map((v) => v.name),
        contains(r.gameIdentity.exeVariant));
  });

  test('third-party runtimes report not installed for a clean dir', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    expect(r.thirdParty.reshade.installed, isFalse);
    expect(r.thirdParty.migoto.installed, isFalse);
  });

  test('config deltas surface a non-default nams.toml setting', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    final delta = r.configDeltas
        .where((d) => d.key == 'disable_splash_screen')
        .toList();
    expect(delta, hasLength(1));
    expect(delta.first.value, 'true');
    expect(delta.first.defaultValue, 'false');
    expect(delta.first.file, 'nams.toml');
  });

  test('nams health reports missing runtime files and unavailable textures',
      () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final r = await DiagnosticsService.collect(game.path);
    expect(r.namsHealth.missingFiles, contains(AppStrings.namsExeName));
    expect(r.namsHealth.missingFiles, contains(AppStrings.yorhaDllName));
    expect(r.namsHealth.namsExePresent, isFalse);
    expect(r.texturePacksAvailable, isFalse);
    expect(r.recentLogIssues, isEmpty);
  });

  test('empty gameDir returns an all-empty report without throwing', () async {
    final r = await DiagnosticsService.collect('');
    expect(r.vanillaDropped, isEmpty);
    expect(r.mods, isEmpty);
    expect(r.thirdParty.reshade.installed, isFalse);
    expect(r.configDeltas, isEmpty);
    expect(() => DiagnosticsService.formatFullReport(r), returnsNormally);
    expect(() => DiagnosticsService.formatSummary(r), returnsNormally);
  });

  test('full report is English and redacts the launch command path', () async {
    final game = _fakeGameDir();
    addTearDown(() => game.deleteSync(recursive: true));

    final preview = '${game.path}${p.separator}NAMS.exe run '
        '--nier-path ${game.path}';
    final r =
        await DiagnosticsService.collect(game.path, launchCommandPreview: preview);
    final full = DiagnosticsService.formatFullReport(r);

    expect(full, contains('YP Launcher Diagnostics Report'));
    expect(full, contains('Game identity'));
    expect(full, contains('<gameDir>'),
        reason: 'gameDir path must be redacted');
    expect(full, isNot(contains(game.path)),
        reason: 'raw game path must not leak into the report');
  });
}
