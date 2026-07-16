import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/mods_service.dart';

import 'support/corpus_reader.dart';

Directory _makeLooseDataDirDrop(String prefix, String dirName) {
  final tmp = Directory.systemTemp.createTempSync(prefix);
  final modelDir = Directory(p.join(tmp.path, dirName, 'model'))
    ..createSync(recursive: true);
  File(p.join(modelDir.path, '0315.dat'))
      .writeAsBytesSync(List<int>.filled(32, 0));
  File(p.join(modelDir.path, '0315.dtt'))
      .writeAsBytesSync(List<int>.filled(32, 0));
  return tmp;
}

void main() {
  group('root-level data dirs without their own tab', () {
    for (final dirName in const ['effect', 'enlighten', 'misctex', 'quest']) {
      test('bare $dirName/ drop installs as a data mod', () async {
        final drop = _makeLooseDataDirDrop('yp_${dirName}_', dirName);
        final gameDir = Directory.systemTemp.createTempSync('yp_${dirName}_g_');
        addTearDown(() {
          try {
            drop.deleteSync(recursive: true);
          } catch (_) {}
          try {
            gameDir.deleteSync(recursive: true);
          } catch (_) {}
        });

        final detected = await ModsService.detectDrop(drop.path);
        expect(detected.kind, ModKind.data,
            reason: '$dirName/ drop rejected: ${detected.errorReason}');

        final result = await ModsService.install(
          gameDir.path,
          drop.path,
          requestedName: '${dirName}_mod',
        );
        expect(result.success, isTrue, reason: result.errorMessage);

        final moved = Directory(
          p.join(gameDir.path, 'nams', 'mods', result.installedId!, 'data',
              dirName, 'model'),
        );
        expect(moved.existsSync(), isTrue,
            reason: 'loose $dirName/ must be normalized under data/');
      });
    }
  });

  group('real archive: Fixed Sand Footsteps (nexus 359)', () {
    final fixture =
        File(p.join('test', 'fixtures', 'sand_footsteps_359.7z'));

    setUpAll(() {
      ArchiveService.overrideSevenZipPath = bundledSevenZipPath();
    });

    tearDownAll(() {
      ArchiveService.overrideSevenZipPath = null;
    });

    test('detects as a data mod and installs effect models', () async {
      final gameDir = Directory.systemTemp.createTempSync('yp_sand_');
      addTearDown(() {
        try {
          gameDir.deleteSync(recursive: true);
        } catch (_) {}
      });

      final detected = await ModsService.detectDrop(fixture.path);
      expect(detected.kind, ModKind.data,
          reason: 'sand footsteps rejected: ${detected.errorReason}');
      expect(detected.errorReason, isNull);

      final result = await ModsService.install(
        gameDir.path,
        fixture.path,
        requestedName: 'fixed_sand_footsteps',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final modelDir = Directory(
        p.join(gameDir.path, 'nams', 'mods', result.installedId!, 'data',
            'effect', 'model'),
      );
      expect(modelDir.existsSync(), isTrue,
          reason: 'effect/model/ not installed under data/');

      final names = modelDir
          .listSync()
          .whereType<File>()
          .map((f) => p.basename(f.path))
          .toSet();
      expect(names, containsAll(<String>{'0315.dat', '0315.dtt'}));
    }, skip: fixture.existsSync()
        ? false
        : 'fixture not present at ${fixture.path}');
  },
      skip: File(bundledSevenZipPath()).existsSync()
          ? false
          : '7-Zip not present at ${bundledSevenZipPath()}');
}
