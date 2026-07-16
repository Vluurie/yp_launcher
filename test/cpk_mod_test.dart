import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/models/installed_mod.dart';
import 'package:yp_launcher/services/archive_service.dart';
import 'package:yp_launcher/services/mods_service.dart';

import 'support/corpus_reader.dart';

List<File> _cpksIn(Directory modDir) {
  final data = Directory(p.join(modDir.path, 'data'));
  if (!data.existsSync()) return const [];
  return data
      .listSync()
      .whereType<File>()
      .where((f) => p.basename(f.path).toLowerCase().endsWith('.cpk'))
      .toList();
}

Directory _makeCpkDrop(
  String prefix, {
  required List<String> cpkNames,
  String? wrapper,
}) {
  final tmp = Directory.systemTemp.createTempSync(prefix);
  var root = tmp;
  if (wrapper != null) {
    root = Directory(p.join(tmp.path, wrapper))..createSync(recursive: true);
  }
  for (final name in cpkNames) {
    File(p.join(root.path, name)).writeAsBytesSync(
      List<int>.filled(64, 0x43),
    );
  }
  return tmp;
}

void main() {
  group('cpk drops: detect + install into data/', () {
    test('bare cpk folder is a data mod, not an unknown drop', () async {
      final drop = _makeCpkDrop('yp_cpk_bare_', cpkNames: ['data100.cpk']);
      addTearDown(() => drop.deleteSync(recursive: true));

      final detected = await ModsService.detectDrop(drop.path);

      expect(detected.kind, ModKind.data,
          reason: 'cpk drop must classify as a data mod: '
              '${detected.errorReason}');
      expect(detected.errorReason, isNull);
    });

    test('wrapped cpk folder unwraps and still detects as data', () async {
      final drop = _makeCpkDrop(
        'yp_cpk_wrapped_',
        cpkNames: ['data100.cpk', 'data201.cpk'],
        wrapper: 'Nierdatafile',
      );
      addTearDown(() => drop.deleteSync(recursive: true));

      final detected = await ModsService.detectDrop(drop.path);

      expect(detected.kind, ModKind.data, reason: detected.errorReason);
      expect(p.basename(detected.unwrappedRoot), 'Nierdatafile',
          reason: 'wrapper should be unwrapped to the cpk-bearing dir');
    });

    test('install normalizes loose cpks into <mod>/data/', () async {
      final drop = _makeCpkDrop(
        'yp_cpk_install_',
        cpkNames: ['data100.cpk', 'data201.cpk'],
        wrapper: 'Nierdatafile',
      );
      final gameDir = Directory.systemTemp.createTempSync('yp_cpk_game_');
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
        requestedName: 'cpk_mod',
      );

      expect(result.success, isTrue, reason: result.errorMessage);

      final modDir =
          Directory(p.join(gameDir.path, 'nams', 'mods', result.installedId!));
      expect(modDir.existsSync(), isTrue);

      final installed = _cpksIn(modDir).map((f) => p.basename(f.path)).toSet();
      expect(installed, {'data100.cpk', 'data201.cpk'},
          reason: 'NAMS reads <mod>/data/*.cpk non-recursively');

      final strayCpk = modDir
          .listSync()
          .whereType<File>()
          .where((f) => f.path.toLowerCase().endsWith('.cpk'));
      expect(strayCpk, isEmpty,
          reason: 'cpks must not stay at the mod root');
    });

    test('installed cpk mod is listed with an archive data entry', () async {
      final drop = _makeCpkDrop('yp_cpk_list_', cpkNames: ['data100.cpk']);
      final gameDir = Directory.systemTemp.createTempSync('yp_cpk_listgame_');
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
        requestedName: 'cpk_listed',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final mods = await ModsService.listInstalled(gameDir.path);
      final mod = mods.firstWhere((m) => m.id == result.installedId);

      expect(mod.kind, ModKind.data);
      final archive = mod.data?.entries
          .where((e) => e.category == DataCategory.archive)
          .toList();
      expect(archive, isNotNull);
      expect(archive!.length, 1);
      expect(archive.first.fileCount, 1);
    });
  });

  group('cpk drops: real archive', () {
    final fixture = File(p.join('test', 'fixtures', 'cpk_mod_709.zip'));

    setUpAll(() {
      ArchiveService.overrideSevenZipPath = bundledSevenZipPath();
    });

    tearDownAll(() {
      ArchiveService.overrideSevenZipPath = null;
    });

    test('nexus 709 cpk pack detects and installs cpks into data/', () async {
      final gameDir = Directory.systemTemp.createTempSync('yp_cpk_709_');
      addTearDown(() {
        try {
          gameDir.deleteSync(recursive: true);
        } catch (_) {}
      });

      final detected = await ModsService.detectDrop(fixture.path);
      expect(detected.kind, ModKind.data,
          reason: 'real cpk pack rejected: ${detected.errorReason}');
      expect(detected.hasVariants, isFalse,
          reason: 'a flat cpk pack must not ask for variant selection');

      final result = await ModsService.install(
        gameDir.path,
        fixture.path,
        requestedName: 'nier_cpk_709',
      );
      expect(result.success, isTrue, reason: result.errorMessage);

      final modDir =
          Directory(p.join(gameDir.path, 'nams', 'mods', result.installedId!));
      final names = _cpksIn(modDir).map((f) => p.basename(f.path)).toSet();
      expect(names, containsAll(<String>{'data100.cpk', 'data201.cpk'}));
    }, timeout: const Timeout(Duration(minutes: 10)),
        skip: fixture.existsSync()
            ? false
            : 'fixture not present at ${fixture.path}');
  },
      skip: File(bundledSevenZipPath()).existsSync()
          ? false
          : '7-Zip not present at ${bundledSevenZipPath()}');
}
