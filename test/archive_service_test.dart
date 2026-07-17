import 'dart:io';

import 'package:archive/archive.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/archive_service.dart';

String get _bundled7zz => p.join(Directory.current.path, 'assets', 'bins',
    Platform.isWindows ? '7z.exe' : '7zz');

void main() {
  setUp(() => ArchiveService.overrideSevenZipPath = _bundled7zz);
  tearDown(() => ArchiveService.overrideSevenZipPath = null);

  group('bundled 7-Zip', () {
    test('the bundled binary exists', () {
      expect(File(_bundled7zz).existsSync(), isTrue,
          reason: 'fetch it per the README before running this');
    });

    test('lists and extracts a zip through ArchiveService', () async {
      final tmp = Directory.systemTemp.createTempSync('arch_fixture_');
      addTearDown(() => tmp.deleteSync(recursive: true));

      final archive = Archive()
        ..addFile(ArchiveFile.string('mod/info.txt', 'PRJ_028'))
        ..addFile(ArchiveFile.string('mod/data/a.bin', 'hello'));
      final zipPath = p.join(tmp.path, 'sample.zip');
      File(zipPath).writeAsBytesSync(ZipEncoder().encode(archive));

      final entries = await ArchiveService.listEntries(zipPath);
      expect(entries, isNotNull);
      final normalized =
          entries!.map((e) => e.replaceAll('\\', '/')).toList();
      expect(normalized, containsAll(['mod/info.txt', 'mod/data/a.bin']));

      final outDir = await ArchiveService.extract(zipPath);
      expect(outDir, isNotNull, reason: ArchiveService.lastExtractError);
      addTearDown(() => Directory(outDir!).deleteSync(recursive: true));

      expect(File(p.join(outDir!, 'mod', 'info.txt')).readAsStringSync(),
          'PRJ_028');
      expect(File(p.join(outDir, 'mod', 'data', 'a.bin')).readAsStringSync(),
          'hello');
    });
  });

  group('real mods in ~/Downloads', () {
    final home = Platform.environment['HOME'];
    final downloads = home == null ? null : Directory(p.join(home, 'Downloads'));
    final zips = (downloads?.existsSync() ?? false)
        ? downloads!
            .listSync()
            .whereType<File>()
            .where((f) => f.path.toLowerCase().endsWith('.zip'))
            .toList()
        : <File>[];

    for (final zip in zips) {
      test('lists ${p.basename(zip.path)}', () async {
        final entries = await ArchiveService.listEntries(zip.path);
        expect(entries, isNotNull);
        expect(entries, isNotEmpty);
      });
    }
  }, skip: (Platform.environment['HOME'] == null));
}
