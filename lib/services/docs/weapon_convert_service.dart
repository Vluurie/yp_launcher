import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/docs/weapon_builder_service.dart';
import 'package:yp_launcher/services/mods_service.dart';
import 'package:yp_launcher/src/rust/api/weapon.dart';

class WeaponSource {
  final String datPath;
  final String dttPath;
  final String stem;
  final WeaponType type;

  const WeaponSource({
    required this.datPath,
    required this.dttPath,
    required this.stem,
    required this.type,
  });
}

class WeaponConvertService {
  WeaponConvertService._();

  static bool isWeaponFile(String path) {
    final extension = p.extension(path).toLowerCase();
    if (extension != '.dat' && extension != '.dtt') return false;
    return stemType(p.basenameWithoutExtension(path)) != null;
  }

  static WeaponType? stemType(String stem) {
    final lower = stem.toLowerCase();
    if (!lower.startsWith('wp') || lower.length < 6) return null;
    final number = int.tryParse(lower.substring(2, 6), radix: 16);
    if (number == null) return null;
    return WeaponBuilderService.typeFromObjId(0x30000 + number);
  }

  static Future<WeaponSource?> inspect(String path) async {
    final stem = p.basenameWithoutExtension(path);
    final type = stemType(stem);
    if (type == null) return null;

    final dir = p.dirname(path);
    final dat = p.join(dir, '$stem.dat');
    final dtt = p.join(dir, '$stem.dtt');
    if (!await File(dat).exists() || !await File(dtt).exists()) return null;

    return WeaponSource(datPath: dat, dttPath: dtt, stem: stem, type: type);
  }

  static String stemForObjId(int objId) {
    final number = objId - 0x30000;
    return 'wp${number.toRadixString(16).padLeft(4, '0')}';
  }

  static Future<Set<int>> takenObjIds(String gameDir) async {
    final taken = await WeaponBuilderService.takenObjIds(gameDir);
    final root = Directory(ModsService.modsDir(gameDir));
    if (!await root.exists()) return taken;

    await for (final entry in root.list(recursive: true, followLinks: false)) {
      if (entry is! File) continue;
      if (p.extension(entry.path).toLowerCase() != '.dat') continue;
      final stem = p.basenameWithoutExtension(entry.path).toLowerCase();
      if (!stem.startsWith('wp') || stem.length < 6) continue;
      final number = int.tryParse(stem.substring(2, 6), radix: 16);
      if (number != null) taken.add(0x30000 + number);
    }
    return taken;
  }

  static String modelDirFor(String gameDir, String modId) {
    return p.join(ModsService.modsDir(gameDir), modId, 'data', 'wp');
  }

  static Future<WeaponRenameResult> convert({
    required WeaponSource source,
    required int targetObjId,
    required String gameDir,
    required String modId,
  }) {
    return renameWeapon(
      datPath: source.datPath,
      dttPath: source.dttPath,
      toStem: stemForObjId(targetObjId),
      outDir: modelDirFor(gameDir, modId),
    );
  }
}
