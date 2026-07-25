import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/file_ops.dart';
import 'package:yp_launcher/services/thirdparty/ini_patch.dart';
import 'package:yp_launcher/services/thirdparty/shaderfix_names.dart';

void main() {
  group('ShaderFixNames map', () {
    test('assigns mod name to new files, keeps existing, prunes gone', () {
      final dir = Directory.systemTemp.createTempSync('yp_sfn_');
      addTearDown(() => dir.deleteSync(recursive: true));

      ShaderFixNames.assign(dir.path, ['aaa.txt', 'bbb.txt'], 'Bande Desinee');
      expect(ShaderFixNames.read(dir.path), {
        'aaa.txt': 'Bande Desinee',
        'bbb.txt': 'Bande Desinee',
      });

      ShaderFixNames.assign(dir.path, ['aaa.txt', 'ccc.txt'], 'Bloom Fix');
      final map = ShaderFixNames.read(dir.path);
      expect(map['aaa.txt'], 'Bande Desinee',
          reason: 'existing entry not overwritten by a later mod');
      expect(map['ccc.txt'], 'Bloom Fix');

      ShaderFixNames.prune(dir.path, {'aaa.txt', 'ccc.txt'});
      final pruned = ShaderFixNames.read(dir.path);
      expect(pruned.containsKey('bbb.txt'), isFalse,
          reason: 'removed file dropped from the map');
      expect(pruned.keys.toSet(), {'aaa.txt', 'ccc.txt'});
    });

    test('read returns empty for a missing or malformed file', () {
      final dir = Directory.systemTemp.createTempSync('yp_sfn2_');
      addTearDown(() => dir.deleteSync(recursive: true));
      expect(ShaderFixNames.read(dir.path), isEmpty);
      File(p.join(dir.path, ShaderFixNames.fileName)).writeAsStringSync('{bad');
      expect(ShaderFixNames.read(dir.path), isEmpty);
    });
  });

  group('FileOps.sizeLabel', () {
    test('formats bytes / KB / MB and null for a missing file', () {
      final dir = Directory.systemTemp.createTempSync('yp_size_');
      addTearDown(() => dir.deleteSync(recursive: true));

      File(p.join(dir.path, 'small')).writeAsBytesSync(List.filled(512, 0));
      File(p.join(dir.path, 'kb')).writeAsBytesSync(List.filled(4096, 0));
      File(p.join(dir.path, 'mb'))
          .writeAsBytesSync(List.filled(2 * 1024 * 1024, 0));

      expect(FileOps.sizeLabel(p.join(dir.path, 'small')), '512 B');
      expect(FileOps.sizeLabel(p.join(dir.path, 'kb')), '4 KB');
      expect(FileOps.sizeLabel(p.join(dir.path, 'mb')), '2.0 MB');
      expect(FileOps.sizeLabel(p.join(dir.path, 'nope')), isNull);
    });
  });

  group('IniPatch.setKey', () {
    test('replaces an existing key in its section, leaving the rest intact', () {
      const ini = '[Loader]\n'
          'target = NAMS.exe\n'
          '\n'
          '[Hunting]\n'
          'hunting = 0\n'
          'marking_mode = skip\n';
      final out = IniPatch.setKey(ini, 'Hunting', 'hunting', '1');
      expect(IniPatch.getKey(out, 'Hunting', 'hunting'), '1');
      expect(IniPatch.getKey(out, 'Hunting', 'marking_mode'), 'skip');
      expect(IniPatch.getKey(out, 'Loader', 'target'), 'NAMS.exe');
    });

    test('adds the key when the section exists but the key does not', () {
      const ini = '[Hunting]\nhunting = 1\n';
      final out = IniPatch.setKey(ini, 'Hunting', 'marking_mode', 'pink');
      expect(IniPatch.getKey(out, 'Hunting', 'marking_mode'), 'pink');
      expect(IniPatch.getKey(out, 'Hunting', 'hunting'), '1');
    });

    test('appends a new section when it is missing', () {
      const ini = '[Loader]\ntarget = NAMS.exe\n';
      final out = IniPatch.setKey(ini, 'Rendering', 'cache_shaders', '0');
      expect(IniPatch.getKey(out, 'Rendering', 'cache_shaders'), '0');
      expect(IniPatch.getKey(out, 'Loader', 'target'), 'NAMS.exe');
    });

    test('leaves a commented key as a comment and adds the active key', () {
      const ini = '[Hunting]\n;hunting = 1\n';
      final out = IniPatch.setKey(ini, 'Hunting', 'hunting', '0');
      expect(IniPatch.getKey(out, 'Hunting', 'hunting'), '0');
      // The comment must remain a comment, not become a second active key.
      expect(
        RegExp(r'^hunting = ', multiLine: true).allMatches(out).length,
        1,
        reason: 'active key must appear exactly once:\n$out',
      );
      expect(out.contains(';hunting = 1'), isTrue);
    });

    test('does not duplicate a key when a comment mentions it', () {
      // Real 3DMigoto d3dx.ini ships a documentation comment above the
      // active key. setKey must replace the active key in place, not treat
      // the comment as a match and append a duplicate.
      const ini = '[System]\n'
          '; hook = all is the safest option\n'
          'hook = recommended\n'
          'proxy_d3d11 =\n';
      final out = IniPatch.setKey(ini, 'System', 'hook', 'recommended');
      expect(
        RegExp(r'^hook = ', multiLine: true).allMatches(out).length,
        1,
        reason: 'hook must not be duplicated:\n$out',
      );
      expect(IniPatch.getKey(out, 'System', 'hook'), 'recommended');
    });

    test('applying the same key twice is idempotent', () {
      const ini = '[System]\nload_library_redirect = 1\n';
      var out = IniPatch.setKey(ini, 'System', 'load_library_redirect', '0');
      out = IniPatch.setKey(out, 'System', 'load_library_redirect', '0');
      expect(
        RegExp(r'^load_library_redirect = ', multiLine: true)
            .allMatches(out)
            .length,
        1,
      );
      expect(IniPatch.getKey(out, 'System', 'load_library_redirect'), '0');
    });

    test('does not bleed into the next section', () {
      const ini = '[Hunting]\nhunting = 0\n[Logging]\nverbose_overlay = 0\n';
      final out = IniPatch.setKey(ini, 'Hunting', 'hunting', '2');
      expect(IniPatch.getKey(out, 'Hunting', 'hunting'), '2');
      expect(IniPatch.getKey(out, 'Logging', 'verbose_overlay'), '0');
    });

    test('getKey returns null for an absent section or key', () {
      const ini = '[Hunting]\nhunting = 1\n';
      expect(IniPatch.getKey(ini, 'Nope', 'x'), isNull);
      expect(IniPatch.getKey(ini, 'Hunting', 'nope'), isNull);
    });
  });
}
