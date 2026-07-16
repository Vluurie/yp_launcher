import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/mods_service.dart';

import 'support/corpus_reader.dart';

bool _hasPayload(Directory modDir) {
  for (final sub in ['entities', 'data', 'wax']) {
    final d = Directory(p.join(modDir.path, sub));
    if (d.existsSync() && d.listSync(recursive: true).any((e) => e is File)) {
      return true;
    }
  }
  return false;
}

Future<void> _assertInstall(
  String gameDir,
  String archivePath,
  String requestedName,
  InstallResult result,
) async {
  if (!result.success) {
    stderr.writeln('WARN install-failed $archivePath -> ${result.errorMessage}');
  }
  expect(result.success, isTrue,
      reason: 'install failed for $archivePath: ${result.errorMessage}');
  expect(result.installedId, isNotNull);

  final modDir = Directory(p.join(gameDir, 'nams', 'mods', result.installedId!));
  final hasTextures = result.sideInstalledTexturePacks.isNotEmpty;

  if (hasTextures) {
    for (final pack in result.sideInstalledTexturePacks) {
      final packDir =
          Directory(p.join(gameDir, 'nams', 'inject', 'textures', pack));
      expect(packDir.existsSync(), isTrue,
          reason: 'texture pack not placed: $pack for $archivePath');
    }
  }

  if (modDir.existsSync()) {
    expect(_hasPayload(modDir), isTrue,
        reason: 'mod folder has no payload for $archivePath');
  } else {
    expect(hasTextures, isTrue,
        reason: 'no mod folder and no textures for $archivePath');
  }
}

void main() {
  final root = defaultCorpusRoot();
  final corpus = discoverCorpus(root);

  if (corpus.isEmpty) {
    test('outfit corpus (skipped)', () {}, skip: 'corpus not mounted at $root');
    return;
  }

  group('outfit corpus: real drop -> detect -> install', () {
    setUpAll(() {
      ArchiveService.overrideSevenZipPath = bundledSevenZipPath();
    });

    tearDownAll(() {
      ArchiveService.overrideSevenZipPath = null;
    });

    for (final archive in corpus) {
      test(archive.toString(), () async {
        final gameDir = Directory.systemTemp.createTempSync('yp_corpus_');
        addTearDown(() {
          try {
            gameDir.deleteSync(recursive: true);
          } catch (_) {}
        });

        final detected = await ModsService.detectDrop(archive.path);

        if (detected.hasVariants) {
          for (final variant in detected.variants) {
            final id = '${archive.uniqueId}__${variant.label}';
            final result = await ModsService.install(
              gameDir.path,
              archive.path,
              requestedName: id,
              variantSubPath: variant.subPath,
            );
            await _assertInstall(gameDir.path, archive.path, id, result);
          }
        } else {
          final result = await ModsService.install(
            gameDir.path,
            archive.path,
            requestedName: archive.uniqueId,
          );
          if (!result.success) {
            if (result.errorMessage == 'texture_only') {
              stderr.writeln('SKIP texture-only ${archive.path} '
                  '(belongs in Textures tab)');
              return;
            }
            final entries =
                listArchiveEntries(archive.path, bundledSevenZipPath());
            if (!archiveHasModContent(entries)) {
              stderr.writeln('SKIP non-mod ${archive.path} '
                  '(${result.errorMessage}, no mod content)');
              return;
            }
          }
          await _assertInstall(
              gameDir.path, archive.path, archive.uniqueId, result);
        }
      }, timeout: const Timeout(Duration(minutes: 2)));
    }
  },
      skip: File(bundledSevenZipPath()).existsSync()
          ? false
          : '7-Zip not present at ${bundledSevenZipPath()}');
}
