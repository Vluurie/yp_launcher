import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/models/config_fields.dart';
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/thirdparty/migoto_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_classifier.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';
import 'package:yp_launcher/services/toml_service.dart';

import 'support/corpus_reader.dart';

String _shaderCorpusRoot() =>
    Platform.environment['YP_SHADER_CORPUS_DIR'] ??
    (Platform.isWindows
        ? r'E:\all-outfit-mods\downloads\shaders'
        : p.join(Platform.environment['HOME'] ?? '', 'yp-shader-corpus'));

void _extract(String archivePath, String sevenZ, String outDir) {
  Process.runSync(
    sevenZ,
    ['x', archivePath, '-o$outDir', '-y', '-bso0', '-bsp0'],
  );
}

CorpusArchive? _find(List<CorpusArchive> all, String modIdPart) {
  for (final a in all) {
    if (a.modId.toLowerCase().contains(modIdPart.toLowerCase())) return a;
  }
  return null;
}

Directory _freshGame() {
  final dir = Directory.systemTemp.createTempSync('yp_migoto_');
  Directory(p.join(dir.path, 'nams')).createSync(recursive: true);
  File(p.join(dir.path, 'nams', 'nams.toml'))
      .writeAsStringSync('disable_3dmigoto_loading = true\n');
  return dir;
}

String? _loaderTarget(String d3dx) {
  var inLoader = false;
  for (final raw in d3dx.replaceAll('\r', '').split('\n')) {
    final t = raw.trim();
    if (t.startsWith('[') && t.endsWith(']')) {
      inLoader = t.toLowerCase() == '[loader]';
      continue;
    }
    if (!inLoader || t.startsWith(';')) continue;
    if (t.toLowerCase().startsWith('target')) {
      final eq = t.indexOf('=');
      if (eq != -1) return t.substring(eq + 1).trim();
    }
  }
  return null;
}

ThirdPartyClassification _classifyArchive(
  CorpusArchive a,
  String sevenZ,
  Directory into,
) {
  final x = p.join(into.path, 'x');
  Directory(x).createSync(recursive: true);
  _extract(a.path, sevenZ, x);
  return ThirdPartyClassifier.classify(x);
}

void main() {
  final sevenZ = bundledSevenZipPath();
  final corpusRoot = _shaderCorpusRoot();
  final haveTools =
      File(sevenZ).existsSync() && Directory(corpusRoot).existsSync();
  final skipReason = haveTools ? null : 'shader corpus / 7z missing';

  const migoto = MigotoRuntime();
  final all = haveTools ? discoverCorpus(corpusRoot) : <CorpusArchive>[];

  late Directory launcherDir;
  setUp(() {
    launcherDir = Directory.systemTemp.createTempSync('yp_launcher_');
    LauncherSetupService.setRuntimeDirForTest(launcherDir.path);
  });
  tearDown(() {
    LauncherSetupService.setRuntimeDirForTest(null);
    launcherDir.deleteSync(recursive: true);
  });

  group('MigotoRuntime install from real corpus mods', () {
    test('loaderTarget defaults to NAMS.exe (the host), not the game exe', () {
      expect(migoto.loaderTarget, AppStrings.namsExeName);
      expect(migoto.loaderTarget, isNot(AppStrings.gameExeName));
    });

    test('Bande Desinee (no [Loader] section) gets target = NAMS.exe',
        () async {
      final archive = _find(all, 'Bande_Desinee');
      if (archive == null) {
        markTestSkipped('Bande Desinee missing');
        return;
      }
      final src = Directory.systemTemp.createTempSync('yp_src_');
      final game = _freshGame();
      addTearDown(() {
        src.deleteSync(recursive: true);
        game.deleteSync(recursive: true);
      });

      final c = _classifyArchive(archive, sevenZ, src);
      expect(c.kind, ThirdPartyKind.migoto);
      final result = await migoto.install(game.path, c);
      expect(result.ok, isTrue);

      final dest = migoto.installDir(game.path);
      expect(File(p.join(dest, 'd3d11.dll')).existsSync(), isTrue);
      expect(Directory(p.join(dest, 'ShaderFixes')).existsSync(), isTrue);

      expect(p.isWithin(launcherDir.path, dest), isTrue,
          reason: 'thirdparty install tree lives under the launcher dir');
      expect(
          Directory(p.join(game.path, AppStrings.thirdPartyDirName))
              .existsSync(),
          isFalse,
          reason: 'nothing written into the NieRAutomata game dir');

      final d3dx = File(p.join(dest, 'd3dx.ini')).readAsStringSync();
      expect(_loaderTarget(d3dx), AppStrings.namsExeName,
          reason: 'target added even when [Loader] was absent');

      final nams = TomlService.parse(
          File(p.join(game.path, 'nams', 'nams.toml')).readAsStringSync());
      expect(nams[NamsFields.disable3dmigotoLoading.key], isFalse);
    }, skip: skipReason);

    test('BloomFix (commented target) gets target set to NAMS.exe', () async {
      final archive = _find(all, 'Bloom_Fix');
      if (archive == null) {
        markTestSkipped('BloomFix missing');
        return;
      }
      final src = Directory.systemTemp.createTempSync('yp_src_');
      final game = _freshGame();
      addTearDown(() {
        src.deleteSync(recursive: true);
        game.deleteSync(recursive: true);
      });

      final c = _classifyArchive(archive, sevenZ, src);
      expect(c.kind, ThirdPartyKind.migoto);
      await migoto.install(game.path, c);

      final dest = migoto.installDir(game.path);
      final d3dx = File(p.join(dest, 'd3dx.ini')).readAsStringSync();
      expect(_loaderTarget(d3dx), AppStrings.namsExeName);
    }, skip: skipReason);

    test('two migoto mods stack: ShaderFixes merge, not overwrite', () async {
      final bande = _find(all, 'Bande_Desinee');
      final bloom = _find(all, 'Bloom_Fix');
      if (bande == null || bloom == null) {
        markTestSkipped('need both Bande + BloomFix');
        return;
      }
      final src1 = Directory.systemTemp.createTempSync('yp_s1_');
      final src2 = Directory.systemTemp.createTempSync('yp_s2_');
      final game = _freshGame();
      addTearDown(() {
        src1.deleteSync(recursive: true);
        src2.deleteSync(recursive: true);
        game.deleteSync(recursive: true);
      });

      final c1 = _classifyArchive(bande, sevenZ, src1);
      final c2 = _classifyArchive(bloom, sevenZ, src2);

      await migoto.install(game.path, c1);
      final dest = migoto.installDir(game.path);
      final afterFirst = Directory(p.join(dest, 'ShaderFixes'))
          .listSync(recursive: true)
          .whereType<File>()
          .map((f) => p.basename(f.path))
          .toSet();

      await migoto.install(game.path, c2);
      final afterSecond = Directory(p.join(dest, 'ShaderFixes'))
          .listSync(recursive: true)
          .whereType<File>()
          .map((f) => p.basename(f.path))
          .toSet();

      expect(afterSecond.containsAll(afterFirst), isTrue,
          reason: 'first mod ShaderFixes still present after second install');
      expect(afterSecond.length, greaterThanOrEqualTo(afterFirst.length),
          reason: 'merge, not overwrite');
    }, skip: skipReason);

    test('wouldUpdate is null when nothing is installed yet', () async {
      final archive = _find(all, 'Bande_Desinee');
      if (archive == null) {
        markTestSkipped('Bande Desinee missing');
        return;
      }
      final src = Directory.systemTemp.createTempSync('yp_src_');
      final game = _freshGame();
      addTearDown(() {
        src.deleteSync(recursive: true);
        game.deleteSync(recursive: true);
      });

      final c = _classifyArchive(archive, sevenZ, src);
      expect(await migoto.wouldUpdate(game.path, c), isNull,
          reason: 'no installed dll to compare against');
    }, skip: skipReason);

    test('ShaderFixes get the dropped mod name, shown in status', () async {
      final archive = _find(all, 'Bande_Desinee');
      if (archive == null) {
        markTestSkipped('Bande Desinee missing');
        return;
      }
      final src = Directory.systemTemp.createTempSync('yp_src_');
      final game = _freshGame();
      addTearDown(() {
        src.deleteSync(recursive: true);
        game.deleteSync(recursive: true);
      });

      final c = _classifyArchive(archive, sevenZ, src)
          .withSourceName('Bande Desinee');
      await migoto.install(game.path, c);

      final status = await migoto.status(game.path);
      final names = status.migotoInfo!.shaderFixNames;
      expect(names, contains('Bande Desinee'),
          reason: 'hash-named ShaderFixes surface under the mod name');
      expect(names.any((n) => n.contains('-ps_replace')), isFalse,
          reason: 'raw hash filenames are hidden once named');
    }, skip: skipReason);
  });

  group('FileOps.filesDiffer (update detection core)', () {
    test('same content → false, changed byte → true, missing → true', () {
      final dir = Directory.systemTemp.createTempSync('yp_diff_');
      addTearDown(() => dir.deleteSync(recursive: true));

      final a = File(p.join(dir.path, 'a.bin'))
        ..writeAsBytesSync([1, 2, 3, 4]);
      final b = File(p.join(dir.path, 'b.bin'))
        ..writeAsBytesSync([1, 2, 3, 4]);
      expect(FileOps.filesDiffer(a.path, b.path), isFalse);

      b.writeAsBytesSync([1, 2, 3, 5]);
      expect(FileOps.filesDiffer(a.path, b.path), isTrue,
          reason: 'same length, different content');

      b.writeAsBytesSync([1, 2, 3, 4, 5]);
      expect(FileOps.filesDiffer(a.path, b.path), isTrue,
          reason: 'different length');

      expect(FileOps.filesDiffer(a.path, p.join(dir.path, 'nope.bin')), isTrue,
          reason: 'missing counterpart counts as differing');
    });
  });
}
