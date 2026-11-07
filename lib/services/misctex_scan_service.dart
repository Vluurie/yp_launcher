import 'dart:io';

import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/mods_service.dart';

class MisctexEntry {
  final String modId;
  final String textureName;
  final String datPath;
  final String dttPath;
  final int totalBytes;

  const MisctexEntry({
    required this.modId,
    required this.textureName,
    required this.datPath,
    required this.dttPath,
    required this.totalBytes,
  });

  String get fileStem => 'misctex_$textureName';

  String get directory => p.dirname(datPath);
}

class MisctexScanService {
  MisctexScanService._();

  static Future<List<MisctexEntry>> scan(String gameDir) async {
    if (gameDir.isEmpty) return const [];

    final modsRoot = Directory(ModsService.modsDir(gameDir));
    if (!await modsRoot.exists()) return const [];

    final found = <MisctexEntry>[];

    await for (final mod in modsRoot.list(followLinks: false)) {
      if (mod is! Directory) continue;
      final modId = p.basename(mod.path);
      final misctexDir = Directory(p.join(mod.path, 'data', 'misctex'));
      if (!await misctexDir.exists()) continue;

      await for (final file in misctexDir.list(followLinks: false)) {
        if (file is! File) continue;
        final name = p.basename(file.path);
        if (!name.startsWith('misctex_') || !name.endsWith('.dat')) continue;

        final stem = name.substring(0, name.length - 4);
        final textureName = stem.substring('misctex_'.length);
        final dtt = File(p.join(misctexDir.path, '$stem.dtt'));

        var bytes = await file.length();
        if (await dtt.exists()) bytes += await dtt.length();

        found.add(
          MisctexEntry(
            modId: modId,
            textureName: textureName,
            datPath: file.path,
            dttPath: dtt.path,
            totalBytes: bytes,
          ),
        );
      }
    }

    found.sort((a, b) {
      final byMod = a.modId.compareTo(b.modId);
      return byMod != 0 ? byMod : a.textureName.compareTo(b.textureName);
    });
    return found;
  }

  static Future<void> remove(MisctexEntry entry) async {
    for (final path in [entry.datPath, entry.dttPath]) {
      final file = File(path);
      if (await file.exists()) await file.delete();
    }
  }

  static Stream<void>? watch(String gameDir) {
    if (gameDir.isEmpty) return null;
    final modsRoot = Directory(ModsService.modsDir(gameDir));
    if (!modsRoot.existsSync()) return null;

    try {
      return modsRoot
          .watch(recursive: true)
          .where((event) => event.path.endsWith('.dat') ||
              event.path.endsWith('.dtt'))
          .map((_) {});
    } catch (_) {
      return null;
    }
  }
}
