import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mods_service.dart';

class FirstModFiles {
  FirstModFiles._();

  static const modId = 'my_first_mod';
  static const itemFile = 'item_9999.toml';

  static const manifest = '''id = "my_first_mod"
display_name = "My First Mod"
author = "Your Name"
version = "1.0.0"
''';

  static const item = '''name = "item_test_pebble"
id_type = 0
buy = -1
sell = -1
no_trash = true

[auto_give]
quantity = 1

[text.name]
_us = "Test Pebble"

[text.help]
_us = "A small stone. Proof your mod loaded."
''';
}

class FirstModService {
  FirstModService._();

  static String modRoot(String gameDir) =>
      p.join(ModsService.modsDir(gameDir), FirstModFiles.modId);

  static String manifestPath(String gameDir) =>
      p.join(modRoot(gameDir), 'mod.toml');

  static String entitiesDir(String gameDir) =>
      p.join(modRoot(gameDir), 'entities');

  static String itemPath(String gameDir) =>
      p.join(entitiesDir(gameDir), FirstModFiles.itemFile);

  static Future<bool> hasFolder(String gameDir) =>
      Directory(modRoot(gameDir)).exists();

  static Future<bool> hasManifest(String gameDir) =>
      File(manifestPath(gameDir)).exists();

  static Future<bool> hasItem(String gameDir) =>
      File(itemPath(gameDir)).exists();

  static Future<void> createFolder(String gameDir) async {
    await Directory(entitiesDir(gameDir)).create(recursive: true);
  }

  static Future<void> writeManifest(String gameDir, [String? content]) async {
    final file = File(manifestPath(gameDir));
    await file.parent.create(recursive: true);
    await file.writeAsString(_withNewline(content ?? FirstModFiles.manifest));
  }

  static Future<void> writeItem(String gameDir, [String? content]) async {
    final file = File(itemPath(gameDir));
    await file.parent.create(recursive: true);
    await file.writeAsString(_withNewline(content ?? FirstModFiles.item));
  }

  static Future<String> readManifest(String gameDir) async {
    final file = File(manifestPath(gameDir));
    if (!await file.exists()) return FirstModFiles.manifest;
    return file.readAsString();
  }

  static Future<String> readItem(String gameDir) async {
    final file = File(itemPath(gameDir));
    if (!await file.exists()) return FirstModFiles.item;
    return file.readAsString();
  }

  static String _withNewline(String text) =>
      text.endsWith('\n') ? text : '$text\n';

  static Future<void> removeAll(String gameDir) async {
    final dir = Directory(modRoot(gameDir));
    if (await dir.exists()) await dir.delete(recursive: true);
  }
}
