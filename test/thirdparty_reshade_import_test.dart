import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/detection/graphics_dll_id.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/toml_service.dart';

String _realGameDir() =>
    Platform.environment['YP_NIER_DIR'] ??
    r'F:\SteamLibrary\steamapps\common\NieRAutomata';

bool _hasRealReShade(String gameDir) {
  if (!Directory(gameDir).existsSync()) return false;
  for (final n in const ['dxgi.dll', 'd3d11.dll', 'ReShade64.dll']) {
    final f = File(p.join(gameDir, n));
    if (f.existsSync() &&
        GraphicsDllId.identifyFile(f.path) == GraphicsDll.reshade) {
      return true;
    }
  }
  return false;
}

void _copyTree(String src, String dst) {
  Directory(dst).createSync(recursive: true);
  for (final e in Directory(src).listSync(recursive: true, followLinks: false)) {
    final rel = p.relative(e.path, from: src);
    final target = p.join(dst, rel);
    if (e is Directory) {
      Directory(target).createSync(recursive: true);
    } else if (e is File) {
      Directory(p.dirname(target)).createSync(recursive: true);
      e.copySync(target);
    }
  }
}

Directory _stageRealInstall(String realGameDir) {
  final dir = Directory.systemTemp.createTempSync('yp_reshade_real_');
  for (final name in const [
    'dxgi.dll',
    'ReShade.ini',
    'ReShadePreset.ini',
    'reshade-shaders',
  ]) {
    final src = p.join(realGameDir, name);
    if (Directory(src).existsSync()) {
      _copyTree(src, p.join(dir.path, name));
    } else if (File(src).existsSync()) {
      File(src).copySync(p.join(dir.path, name));
    }
  }
  File(p.join(dir.path, 'd3dcompiler_47.dll'))
      .writeAsBytesSync(List.filled(64, 0x90));
  final nams = Directory(p.join(dir.path, 'nams'))..createSync(recursive: true);
  File(p.join(nams.path, 'nams.toml'))
      .writeAsStringSync('disable_reshade_loading = true\n');
  return dir;
}

Map<String, dynamic> _nams(String gameDir) => TomlService.parse(
      File(p.join(gameDir, 'nams', 'nams.toml')).readAsStringSync(),
    );

void main() {
  const reshade = ReShadeRuntime();
  final realGameDir = _realGameDir();
  final available = _hasRealReShade(realGameDir);
  final skipReason = available
      ? null
      : 'no real ReShade install at $realGameDir (set YP_NIER_DIR)';

  group('ReShadeRuntime.importFromGameRoot against a real ReShade install', () {
    late Directory gameDir;
    late Directory launcherDir;

    setUp(() {
      if (!available) return;
      gameDir = _stageRealInstall(realGameDir);
      launcherDir = Directory.systemTemp.createTempSync('yp_launcher_');
      LauncherSetupService.setRuntimeDirForTest(launcherDir.path);
    });

    tearDown(() {
      if (!available) return;
      LauncherSetupService.setRuntimeDirForTest(null);
      try {
        gameDir.deleteSync(recursive: true);
        launcherDir.deleteSync(recursive: true);
      } catch (_) {}
    });

    test('relocates dll + ini + shaders into thirdparty/reshade', () async {
      final ok = await reshade.importFromGameRoot(gameDir.path);
      expect(ok, isTrue);

      final tp = reshade.installDir(gameDir.path);
      final dll = File(p.join(tp, 'ReShade64.dll'));
      final addonDll = File(p.join(tp, 'ReShade64_Addon.dll'));
      expect(dll.existsSync() || addonDll.existsSync(), isTrue);

      final relocated = dll.existsSync() ? dll : addonDll;
      expect(GraphicsDllId.identifyFile(relocated.path), GraphicsDll.reshade);

      expect(File(p.join(tp, 'ReShade.ini')).existsSync(), isTrue);
      expect(
        Directory(p.join(tp, 'reshade-shaders', 'Shaders')).existsSync(),
        isTrue,
      );
    }, skip: skipReason);

    test('ReShade.ini search paths point at dirs that exist after relocate',
        () async {
      await reshade.importFromGameRoot(gameDir.path);
      final tp = reshade.installDir(gameDir.path);
      final ini = File(p.join(tp, 'ReShade.ini')).readAsStringSync();

      expect(ini, contains(r'.\reshade-shaders\Shaders'));
      expect(Directory(p.join(tp, 'reshade-shaders', 'Shaders')).existsSync(),
          isTrue);

      final shaderCount = Directory(p.join(tp, 'reshade-shaders', 'Shaders'))
          .listSync(recursive: true)
          .whereType<File>()
          .where((f) => f.path.toLowerCase().endsWith('.fx'))
          .length;
      expect(shaderCount, greaterThan(0),
          reason: 'real shaders relocated, so EffectSearchPaths has targets');
    }, skip: skipReason);

    test('keeps the real NieR depth defs from the installer ini', () async {
      await reshade.importFromGameRoot(gameDir.path);
      final tp = reshade.installDir(gameDir.path);
      final ini = File(p.join(tp, 'ReShade.ini')).readAsStringSync();
      expect(ini, contains('RESHADE_DEPTH_LINEARIZATION_FAR_PLANE=1000'));
    }, skip: skipReason);

    test('relocates d3dcompiler_47.dll so NAMS can preload it under Wine',
        () async {
      await reshade.importFromGameRoot(gameDir.path);
      final tp = reshade.installDir(gameDir.path);
      expect(File(p.join(tp, 'd3dcompiler_47.dll')).existsSync(), isTrue,
          reason: 'compiler moved into thirdparty/reshade for the preload');
    }, skip: skipReason);

    test('leaves the game-root original untouched (NAMS runs separately)',
        () async {
      final before = gameDir
          .listSync()
          .whereType<File>()
          .map((f) => p.basename(f.path))
          .toSet();

      await reshade.importFromGameRoot(gameDir.path);

      final after = gameDir
          .listSync()
          .whereType<File>()
          .map((f) => p.basename(f.path))
          .toSet();

      expect(after, equals(before),
          reason: 'game-root files copied into thirdparty, not moved/renamed');
      expect(
        gameDir
            .listSync()
            .whereType<File>()
            .where((f) => f.path.endsWith('.nams-backup')),
        isEmpty,
        reason: 'no backup files created',
      );
    }, skip: skipReason);

    test('sets disable_reshade_loading = false', () async {
      await reshade.importFromGameRoot(gameDir.path);
      expect(_nams(gameDir.path)[NamsFields.disableReShadeLoading.key], isFalse);
    }, skip: skipReason);

    test('status reports installed + enabled + shaders present', () async {
      await reshade.importFromGameRoot(gameDir.path);
      final s = await reshade.status(gameDir.path);
      expect(s.installed, isTrue);
      expect(s.enabled, isTrue);
      expect(s.shadersMissing, isFalse);
    }, skip: skipReason);
  });
}
