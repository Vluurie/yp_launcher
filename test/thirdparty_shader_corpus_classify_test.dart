import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/thirdparty/thirdparty_classifier.dart';
import 'package:yp_launcher/services/thirdparty/thirdparty_models.dart';

import 'support/corpus_reader.dart';

String _shaderCorpusRoot() =>
    Platform.environment['YP_SHADER_CORPUS_DIR'] ??
    (Platform.isWindows
        ? r'E:\all-outfit-mods\downloads\shaders'
        : p.join(Platform.environment['HOME'] ?? '', 'yp-shader-corpus'));

const _supported = {
  ThirdPartyKind.reshadeWholeInstall,
  ThirdPartyKind.reshadePreset,
  ThirdPartyKind.migoto,
};

const _routedElsewhere = {
  ThirdPartyKind.lodmod,
  ThirdPartyKind.gameData,
  ThirdPartyKind.textures,
  ThirdPartyKind.multiBundle,
};

String? _extract(String archivePath, String sevenZ, String outDir) {
  final r = Process.runSync(
    sevenZ,
    ['x', archivePath, '-o$outDir', '-y', '-bso0', '-bsp0'],
  );
  if (r.exitCode != 0) return null;
  return outDir;
}

void main() {
  final sevenZ = bundledSevenZipPath();
  final corpusRoot = _shaderCorpusRoot();
  final haveTools =
      File(sevenZ).existsSync() && Directory(corpusRoot).existsSync();
  final skipReason =
      haveTools ? null : 'shader corpus / 7z missing ($corpusRoot)';

  final archives = haveTools ? discoverCorpus(corpusRoot) : <CorpusArchive>[];

  test(
    'most real shader archives classify as an installable runtime',
    () {
      final byKind = <ThirdPartyKind, int>{};
      final unsupported = <String>[];
      final tmp = Directory.systemTemp.createTempSync('yp_shader_classify_');
      addTearDown(() {
        try {
          tmp.deleteSync(recursive: true);
        } catch (_) {}
      });

      for (final archive in archives) {
        final out = p.join(tmp.path, archive.uniqueId);
        Directory(out).createSync(recursive: true);
        if (_extract(archive.path, sevenZ, out) == null) {
          byKind.update(ThirdPartyKind.unknown, (v) => v + 1,
              ifAbsent: () => 1);
          unsupported.add('${archive.modId}/${archive.basename}  '
              '(extract failed)');
          Directory(out).deleteSync(recursive: true);
          continue;
        }

        ThirdPartyClassification? c;
        Object? error;
        try {
          c = ThirdPartyClassifier.classify(out);
        } catch (e) {
          error = e;
        }
        if (c == null) {
          byKind.update(ThirdPartyKind.unknown, (v) => v + 1,
              ifAbsent: () => 1);
          unsupported.add('${archive.modId}/${archive.basename}  '
              '(classify threw: ${error.runtimeType})');
        } else {
          byKind.update(c.kind, (v) => v + 1, ifAbsent: () => 1);
          if (!_supported.contains(c.kind) &&
              !_routedElsewhere.contains(c.kind)) {
            unsupported
                .add('${archive.modId}/${archive.basename}  -> ${c.kind.name}');
          }
        }
        try {
          Directory(out).deleteSync(recursive: true);
        } catch (_) {}
      }

      final total = archives.length;
      final supportedCount = byKind.entries
          .where((e) => _supported.contains(e.key))
          .fold<int>(0, (s, e) => s + e.value);
      final routedCount = byKind.entries
          .where((e) => _routedElsewhere.contains(e.key))
          .fold<int>(0, (s, e) => s + e.value);
      final unsupportedCount = unsupported.length;

      final report = StringBuffer()
        ..writeln('Shader corpus classification over $total archives:')
        ..writeln('  installable (reshade/migoto): $supportedCount')
        ..writeln('  routed to another tab:        $routedCount')
        ..writeln('  UNSUPPORTED:                  $unsupportedCount');
      final kinds = byKind.entries.toList()
        ..sort((a, b) => b.value.compareTo(a.value));
      for (final e in kinds) {
        report.writeln('    ${e.key.name.padRight(20)} ${e.value}');
      }
      if (unsupported.isNotEmpty) {
        report.writeln('  --- unsupported archives ---');
        for (final u in unsupported) {
          report.writeln('    $u');
        }
      }
      printOnFailure(report.toString());

      expect(total, greaterThan(0), reason: 'no shader archives discovered');

      final threw = unsupported.where((u) => u.contains('classify threw'));
      expect(
        threw,
        isEmpty,
        reason: 'classify() must never throw on a real archive '
            '(long/nested paths, unreadable dirs).\n$report',
      );

      final handledRatio = (supportedCount + routedCount) / total;
      expect(
        handledRatio,
        greaterThanOrEqualTo(0.75),
        reason: 'at least 75% of shader archives should be recognized as an '
            'installable runtime or routed to another tab, not shown as '
            '"file type not supported".\n$report',
      );
    },
    skip: skipReason,
    timeout: const Timeout(Duration(minutes: 10)),
  );
}
