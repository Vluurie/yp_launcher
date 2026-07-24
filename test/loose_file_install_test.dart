import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/models/loose_file_paths.dart';
import 'package:yp_launcher/services/mods_service.dart';

Directory _drop(String prefix, Map<String, List<String>> filesByStem) {
  final tmp = Directory.systemTemp.createTempSync(prefix);
  for (final entry in filesByStem.entries) {
    for (final ext in entry.value) {
      File(p.join(tmp.path, '${entry.key}.$ext'))
          .writeAsBytesSync(List<int>.filled(32, 0));
    }
  }
  return tmp;
}

String _expectedDir(String gameDir, String modId, String subdir) {
  final base = p.join(gameDir, 'nams', 'mods', modId, 'data');
  return subdir.isEmpty ? base : p.joinAll([base, ...subdir.split('/')]);
}

void main() {
  group('loose_file_paths table consistency', () {
    test('counts match the generated corpus', () {
      expect(LooseFilePaths.subdirForStem.length, 4632);
      expect(LooseFilePaths.stemsWithVanillaPair.length, 2847);
    });

    test('every subdir value is well-formed', () {
      final bad = <String>[];
      for (final entry in LooseFilePaths.subdirForStem.entries) {
        final v = entry.value;
        if (v.startsWith('/') ||
            v.endsWith('/') ||
            v.contains('\\') ||
            v.contains('..') ||
            v.contains('_extracted')) {
          bad.add('${entry.key} -> "$v"');
        }
      }
      expect(bad, isEmpty, reason: 'malformed subdirs: ${bad.take(10)}');
    });

    test('paired stems are a subset of all stems', () {
      final orphan = LooseFilePaths.stemsWithVanillaPair
          .where((s) => !LooseFilePaths.subdirForStem.containsKey(s))
          .toList();
      expect(orphan, isEmpty, reason: 'paired-but-unknown: ${orphan.take(10)}');
    });
  });

  group('loose file placement', () {
    Directory representativeStems() {
      final byTop = <String, String>{};
      String? twoDeep;
      String? rootLevel;
      for (final entry in LooseFilePaths.subdirForStem.entries) {
        final sub = entry.value;
        final top = sub.isEmpty ? '' : sub.split('/').first;
        byTop.putIfAbsent(top, () => entry.key);
        if (twoDeep == null && sub.split('/').length == 2) twoDeep = entry.key;
        if (rootLevel == null && sub.isEmpty) rootLevel = entry.key;
      }
      final picked = <String>{
        ...byTop.values,
        if (twoDeep != null) twoDeep,
        if (rootLevel != null) rootLevel,
      };
      return _drop(
        'yp_loose_cov_',
        {for (final s in picked) s: const ['dat']},
      );
    }

    test('every subdir type places files at the exact vanilla location',
        () async {
      final drop = representativeStems();
      final gameDir = Directory.systemTemp.createTempSync('yp_loose_cov_g_');
      addTearDown(() {
        try {
          drop.deleteSync(recursive: true);
        } catch (_) {}
        try {
          gameDir.deleteSync(recursive: true);
        } catch (_) {}
      });

      final droppedStems = drop
          .listSync()
          .whereType<File>()
          .map((f) => p.basenameWithoutExtension(f.path))
          .toList();

      final detected = await ModsService.detectDrop(drop.path);
      expect(detected.kind, ModKind.data,
          reason: 'loose drop rejected: ${detected.errorReason}');

      final result = await ModsService.install(
        gameDir.path,
        drop.path,
        requestedName: 'loose_coverage',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      for (final stem in droppedStems) {
        final subdir = LooseFilePaths.subdirForStem[stem]!;
        final dest =
            File(p.join(_expectedDir(gameDir.path, result.installedId!, subdir),
                '$stem.dat'));
        expect(dest.existsSync(), isTrue,
            reason: '$stem.dat expected at ${dest.path}');
      }
    });
  });

  group('unpaired warning semantics', () {
    test('paired stem dropped without its .dtt warns', () async {
      final stem = LooseFilePaths.stemsWithVanillaPair.first;
      final drop = _drop('yp_loose_pair_', {
        stem: const ['dat'],
      });
      final gameDir = Directory.systemTemp.createTempSync('yp_loose_pair_g_');
      addTearDown(() {
        try {
          drop.deleteSync(recursive: true);
        } catch (_) {}
        try {
          gameDir.deleteSync(recursive: true);
        } catch (_) {}
      });

      final result = await ModsService.install(
        gameDir.path,
        drop.path,
        requestedName: 'loose_pair',
      );
      expect(result.success, isTrue, reason: result.errorMessage);
      expect(result.unpairedWarnings, contains('$stem.dtt'));
    });

    test('vanilla dat-only stem dropped alone does not warn', () async {
      final stem = LooseFilePaths.subdirForStem.keys.firstWhere(
        (s) => !LooseFilePaths.stemsWithVanillaPair.contains(s),
      );
      final drop = _drop('yp_loose_solo_', {
        stem: const ['dat'],
      });
      final gameDir = Directory.systemTemp.createTempSync('yp_loose_solo_g_');
      addTearDown(() {
        try {
          drop.deleteSync(recursive: true);
        } catch (_) {}
        try {
          gameDir.deleteSync(recursive: true);
        } catch (_) {}
      });

      final result = await ModsService.install(
        gameDir.path,
        drop.path,
        requestedName: 'loose_solo',
      );
      expect(result.success, isTrue, reason: result.errorMessage);
      expect(result.unpairedWarnings, isEmpty);
    });
  });
}
