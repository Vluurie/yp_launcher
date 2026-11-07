import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/docs/first_mod_service.dart';
import 'package:yp_launcher/services/toml_service.dart';

void main() {
  late Directory gameDir;

  setUp(() {
    gameDir = Directory.systemTemp.createTempSync('first_mod_test');
  });

  tearDown(() {
    if (gameDir.existsSync()) gameDir.deleteSync(recursive: true);
  });

  test('writes the mod under nams/mods/my_first_mod', () async {
    await FirstModService.createFolder(gameDir.path);
    await FirstModService.writeManifest(gameDir.path);
    await FirstModService.writeItem(gameDir.path);

    final root = p.join(gameDir.path, 'nams', 'mods', 'my_first_mod');
    expect(Directory(root).existsSync(), isTrue);
    expect(File(p.join(root, 'mod.toml')).existsSync(), isTrue);
    expect(
      File(p.join(root, 'entities', 'item_9999.toml')).existsSync(),
      isTrue,
    );
  });

  test('the manifest parses and carries the documented fields', () async {
    await FirstModService.writeManifest(gameDir.path);
    final raw =
        File(FirstModService.manifestPath(gameDir.path)).readAsStringSync();
    final parsed = TomlService.parse(raw);

    expect(parsed['id'], 'my_first_mod');
    expect(parsed['version'], '1.0.0');
    expect(parsed['display_name'], isNotNull);
  });

  test('the item parses with its text tables', () async {
    await FirstModService.writeItem(gameDir.path);
    final raw = File(FirstModService.itemPath(gameDir.path)).readAsStringSync();
    final parsed = TomlService.parse(raw);

    expect(parsed['name'], 'item_test_pebble');
    expect(parsed['id_type'], 0);
    expect(parsed['no_trash'], isTrue);
    expect(parsed['auto_give']['quantity'], 1);
    expect(parsed['text']['name']['_us'], 'Test Pebble');
    expect(parsed['text']['help']['_us'], isNotNull);
  });

  test('progress checks report what is on disk', () async {
    expect(await FirstModService.hasFolder(gameDir.path), isFalse);
    expect(await FirstModService.hasManifest(gameDir.path), isFalse);
    expect(await FirstModService.hasItem(gameDir.path), isFalse);

    await FirstModService.createFolder(gameDir.path);
    expect(await FirstModService.hasFolder(gameDir.path), isTrue);
    expect(await FirstModService.hasManifest(gameDir.path), isFalse);

    await FirstModService.writeManifest(gameDir.path);
    await FirstModService.writeItem(gameDir.path);
    expect(await FirstModService.hasManifest(gameDir.path), isTrue);
    expect(await FirstModService.hasItem(gameDir.path), isTrue);
  });

  test('writing twice overwrites instead of failing', () async {
    await FirstModService.writeItem(gameDir.path);
    await FirstModService.writeItem(gameDir.path);
    final raw = File(FirstModService.itemPath(gameDir.path)).readAsStringSync();
    expect('Test Pebble'.allMatches(raw).length, 1);
  });

  test('reset removes the whole mod folder', () async {
    await FirstModService.writeItem(gameDir.path);
    await FirstModService.removeAll(gameDir.path);
    expect(Directory(FirstModService.modRoot(gameDir.path)).existsSync(),
        isFalse);
  });

  test('reset on a missing folder does not throw', () async {
    await FirstModService.removeAll(gameDir.path);
  });

  test('the written files match the documented snippets', () async {
    final doc =
        File('assets/docs/native/01_first_mod.md').readAsStringSync();
    expect(doc.contains('id = "my_first_mod"'), isTrue);
    expect(doc.contains('name = "item_test_pebble"'), isTrue);
    expect(doc.contains('item_9999.toml'), isTrue);

    for (final line in FirstModFiles.item.split('\n')) {
      final trimmed = line.trim();
      if (trimmed.isEmpty) continue;
      expect(doc.contains(trimmed), isTrue, reason: 'doc is missing: $trimmed');
    }
  });
}
