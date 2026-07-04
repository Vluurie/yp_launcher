import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/mods_service.dart';

import 'support/corpus_reader.dart';

const _textureWrappers = ['sk_res', 'far_res', 'inject'];

bool _dirHasPlModel(Directory root) {
  final pl = Directory(p.join(root.path, 'data', 'pl'));
  if (!pl.existsSync()) return false;
  return pl.listSync().whereType<File>().any((f) {
    final n = p.basename(f.path).toLowerCase();
    return n.startsWith('pl') && (n.endsWith('.dat') || n.endsWith('.dtt'));
  });
}

void main() {
  final root = defaultCorpusRoot();
  final archive = File(
    p.join(root, '335', '2P - The Mock Machine - All in One.7z'),
  );

  if (!archive.existsSync()) {
    test('multi-outfit + sibling texture sets (skipped)', () {},
        skip: 'sample archive not present at ${archive.path}');
    return;
  }

  group('multi outfit + combinable sibling texture sets', () {
    setUpAll(() {
      final sevenZip = bundledSevenZipPath();
      expect(File(sevenZip).existsSync(), isTrue);
      ArchiveService.overrideSevenZipPath = sevenZip;
    });

    tearDownAll(() {
      ArchiveService.overrideSevenZipPath = null;
    });

    test('detect exposes outfit variants and >1 texture set', () async {
      final detected = await ModsService.detectDrop(archive.path);

      expect(detected.hasVariants, isTrue,
          reason: 'expected outfit variants');
      expect(detected.hasTextureVariants, isTrue,
          reason: 'expected several combinable texture sets');
      expect(detected.textureVariants.length, greaterThan(1));
    });

    test('install one outfit + chosen texture sets merges flat into textures/',
        () async {
      final gameDir = Directory.systemTemp.createTempSync('yp_multi_tex_');
      addTearDown(() {
        try {
          gameDir.deleteSync(recursive: true);
        } catch (_) {}
      });

      final detected = await ModsService.detectDrop(archive.path);

      final outfit = detected.variants.firstWhere(
        (v) => _dirHasPlModel(
            Directory(p.join(detected.unwrappedRoot, v.subPath))),
        orElse: () => detected.variants.first,
      );

      final wanted = detected.textureVariants
          .take(2)
          .map((t) => p.relative(t.path, from: detected.unwrappedRoot))
          .toList();
      expect(wanted.length, 2);

      final result = await ModsService.install(
        gameDir.path,
        archive.path,
        requestedName: 'multi_tex_mod',
        variantSubPath: outfit.subPath,
        texturePackSubPaths: wanted,
      );

      expect(result.success, isTrue, reason: result.errorMessage);
      final modDir =
          Directory(p.join(gameDir.path, 'nams', 'mods', result.installedId!));
      expect(modDir.existsSync(), isTrue);
      expect(_dirHasPlModel(modDir), isTrue,
          reason: 'outfit model not installed');

      final texDir = Directory(p.join(modDir.path, 'textures'));
      expect(texDir.existsSync(), isTrue, reason: 'textures/ not created');

      final installedDds = texDir
          .listSync()
          .whereType<File>()
          .where((f) => f.path.toLowerCase().endsWith('.dds'))
          .toList();
      expect(installedDds, isNotEmpty,
          reason: 'chosen texture sets were not merged into textures/');

      for (final child in modDir.listSync()) {
        final name = p.basename(child.path).toLowerCase();
        if (child is Directory) {
          expect(name == 'data' || name == 'textures', isTrue,
              reason: 'unexpected leftover dir in mod folder: $name');
          expect(_textureWrappers.contains(name), isFalse);
        }
      }
    });

    test('batch install: one extract installs several outfits as own mods',
        () async {
      final gameDir = Directory.systemTemp.createTempSync('yp_batch_');
      addTearDown(() {
        try {
          gameDir.deleteSync(recursive: true);
        } catch (_) {}
      });

      final detected = await ModsService.detectDrop(archive.path);
      final outfits =
          detected.variants.where((v) => v.isOutfit).take(3).toList();
      expect(outfits.length, greaterThanOrEqualTo(2));

      final allTex = [
        for (final t in detected.textureVariants)
          p.relative(t.path, from: detected.unwrappedRoot),
      ];

      final requests = [
        for (final o in outfits)
          VariantInstallRequest(
            requestedName: 'batch - ${o.label}',
            variantSubPath: o.subPath,
            texturePackSubPaths: allTex,
          ),
      ];

      final results =
          await ModsService.installVariants(gameDir.path, archive.path, requests);

      expect(results.length, outfits.length);
      expect(results.every((r) => r.success), isTrue,
          reason: results.map((r) => r.errorMessage).toList().toString());

      final ids = results.map((r) => r.installedId).toSet();
      expect(ids.length, outfits.length, reason: 'ids not unique: $ids');

      for (final r in results) {
        final modDir =
            Directory(p.join(gameDir.path, 'nams', 'mods', r.installedId!));
        expect(_dirHasPlModel(modDir), isTrue,
            reason: 'no pl model for ${r.installedId}');
        final tex = Directory(p.join(modDir.path, 'textures'));
        final dds = tex.existsSync()
            ? tex
                .listSync()
                .whereType<File>()
                .where((f) => f.path.toLowerCase().endsWith('.dds'))
                .length
            : 0;
        expect(dds, greaterThan(0),
            reason: 'no textures bundled for ${r.installedId}');
      }
    });
  });
}
