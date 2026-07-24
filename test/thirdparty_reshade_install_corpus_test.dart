import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/launcher_setup_service.dart';
import 'package:yp_launcher/services/thirdparty/reshade_runtime.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_classifier.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';

import 'support/corpus_reader.dart';

String _shaderCorpusRoot() =>
    Platform.environment['YP_SHADER_CORPUS_DIR'] ??
    (Platform.isWindows
        ? r'E:\all-outfit-mods\downloads\shaders'
        : p.join(Platform.environment['HOME'] ?? '', 'yp-shader-corpus'));

String? _extract(String archivePath, String sevenZ, String outDir) {
  final r = Process.runSync(
    sevenZ,
    ['x', archivePath, '-o$outDir', '-y', '-bso0', '-bsp0'],
  );
  if (r.exitCode != 0) return null;
  return outDir;
}

bool _hasAbsolutePathValue(String iniContent) {
  for (final line in iniContent.replaceAll('\r', '').split('\n')) {
    final eq = line.indexOf('=');
    if (eq <= 0) continue;
    final value = line.substring(eq + 1).trim();
    if (value.isEmpty) continue;
    if (value.startsWith('/') || value.startsWith('\\')) return true;
    if (value.length >= 3 &&
        value[1] == ':' &&
        (value[2] == '\\' || value[2] == '/')) {
      return true;
    }
  }
  return false;
}

void main() {
  final sevenZ = bundledSevenZipPath();
  final corpusRoot = _shaderCorpusRoot();
  final haveTools = File(sevenZ).existsSync() && Directory(corpusRoot).existsSync();
  final skipReason =
      haveTools ? null : 'shader corpus / 7z missing ($corpusRoot)';

  const reshade = ReShadeRuntime();
  final archives = haveTools ? discoverCorpus(corpusRoot) : <CorpusArchive>[];

  group('ReShade install from real corpus mods keeps ini paths valid', () {
    for (final archive in archives) {
      test('${archive.modId}/${archive.basename}', () async {
        final tmp = Directory.systemTemp.createTempSync('yp_tp_corpus_');
        final launcherDir = p.join(tmp.path, 'launcher');
        Directory(launcherDir).createSync(recursive: true);
        LauncherSetupService.setRuntimeDirForTest(launcherDir);
        addTearDown(() {
          LauncherSetupService.setRuntimeDirForTest(null);
          try {
            tmp.deleteSync(recursive: true);
          } catch (_) {}
        });

        final extractDir = p.join(tmp.path, 'extract');
        Directory(extractDir).createSync(recursive: true);
        if (_extract(archive.path, sevenZ, extractDir) == null) {
          markTestSkipped('extract failed: ${archive.path}');
          return;
        }

        final classification = ThirdPartyClassifier.classify(extractDir);
        if (!reshade.canInstall(classification)) {
          expect(
            classification.kind,
            isNot(anyOf(
              ThirdPartyKind.reshadeWholeInstall,
              ThirdPartyKind.reshadePreset,
            )),
          );
          return;
        }

        final gameDir = p.join(tmp.path, 'game');
        Directory(p.join(gameDir, 'nams')).createSync(recursive: true);
        File(p.join(gameDir, 'nams', 'nams.toml'))
            .writeAsStringSync('disable_reshade_loading = true\n');

        final result = await reshade.install(gameDir, classification);
        expect(result.ok, isTrue);

        final tp = reshade.installDir(gameDir);
        expect(p.isWithin(launcherDir, tp), isTrue,
            reason: 'thirdparty install tree lives under the launcher dir');
        expect(Directory(p.join(gameDir, 'thirdparty')).existsSync(), isFalse,
            reason: 'nothing written into the NieRAutomata game dir');
        final ini = File(p.join(tp, 'ReShade.ini'));

        if (ini.existsSync()) {
          final content = ini.readAsStringSync();

          expect(
            _hasAbsolutePathValue(content),
            isFalse,
            reason: 'no absolute machine paths survive in ReShade.ini '
                'for ${archive.modId}',
          );
        }

        if (classification.hasShaders) {
          expect(result.shadersMissing, isFalse,
              reason: 'pack shipped shaders, so none missing after install');
          expect(
            Directory(p.join(tp, 'reshade-shaders', 'Shaders')).existsSync(),
            isTrue,
            reason: 'shaders relocated into the canonical folder',
          );
        }
      });
    }
  }, skip: skipReason);
}
