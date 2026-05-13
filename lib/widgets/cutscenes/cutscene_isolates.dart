import 'dart:io';
import 'dart:isolate';
import 'dart:typed_data';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/isolate_service.dart';

const maxModNameLength = 27;

class CutsceneMod {
  final String name;
  final String fullPath;
  final int usmCount;
  final List<String> usmFiles;
  final List<String> matchingOriginals;
  final List<String> missingOriginals;
  final bool hasH264;
  final int maxWidth;
  final int maxHeight;
  final String? bundledWithModId;

  const CutsceneMod({
    required this.name,
    required this.fullPath,
    required this.usmCount,
    required this.usmFiles,
    required this.matchingOriginals,
    required this.missingOriginals,
    this.hasH264 = false,
    this.maxWidth = 0,
    this.maxHeight = 0,
    this.bundledWithModId,
  });
}

class ScanParams {
  final String cutscenesDir;
  final String originalMovieDir;
  final String modsDir;
  const ScanParams({
    required this.cutscenesDir,
    required this.originalMovieDir,
    required this.modsDir,
  });
}

Map<String, dynamic> _scanModEntry(Directory entry, {String? bundledWithModId}) {
  final modName = path.basename(entry.path);
  final movieDir = Directory(path.join(entry.path, 'movie'));
  final usmFiles = <String>[];
  if (movieDir.existsSync()) {
    for (final f in movieDir.listSync()) {
      if (f is File && f.path.toLowerCase().endsWith('.usm')) {
        usmFiles.add(path.basename(f.path));
      }
    }
  }
  usmFiles.sort();

  bool hasH264 = false;
  int maxW = 0;
  int maxH = 0;
  if (movieDir.existsSync()) {
    for (final f in movieDir.listSync()) {
      if (f is! File || !f.path.toLowerCase().endsWith('.usm')) continue;
      final info = _quickParseUsm(f);
      if (info == null) continue;
      if (info.codec == 5) hasH264 = true;
      if (info.width > maxW) maxW = info.width;
      if (info.height > maxH) maxH = info.height;
    }
  }

  return {
    'name': modName,
    'fullPath': entry.path,
    'usmCount': usmFiles.length,
    'usmFiles': usmFiles,
    'matching': usmFiles,
    'missing': <String>[],
    'hasH264': hasH264,
    'maxWidth': maxW,
    'maxHeight': maxH,
    'bundledWithModId': bundledWithModId,
  };
}

Map<String, dynamic> scanCutscenesIsolate(ScanParams params) {
  final mods = <Map<String, dynamic>>[];

  final originals = <String>{};
  final originalDir = Directory(params.originalMovieDir);
  if (originalDir.existsSync()) {
    for (final entity in originalDir.listSync()) {
      if (entity is File && entity.path.toLowerCase().endsWith('.usm')) {
        originals.add(path.basename(entity.path).toLowerCase());
      }
    }
  }

  final cutscenesDir = Directory(params.cutscenesDir);
  if (cutscenesDir.existsSync()) {
    for (final entity in cutscenesDir.listSync()) {
      if (entity is! Directory) continue;
      final n = path.basename(entity.path);
      if (n.startsWith('.') || n.startsWith('_')) continue;
      mods.add(_scanModEntry(entity));
    }
  }

  final modsDir = Directory(params.modsDir);
  if (modsDir.existsSync()) {
    for (final modEntry in modsDir.listSync()) {
      if (modEntry is! Directory) continue;
      final modId = path.basename(modEntry.path);
      if (modId.startsWith('.') || modId.startsWith('_')) continue;
      final bundledCutscenes = Directory(path.join(modEntry.path, 'cutscenes'));
      if (!bundledCutscenes.existsSync()) continue;
      for (final entity in bundledCutscenes.listSync()) {
        if (entity is! Directory) continue;
        final n = path.basename(entity.path);
        if (n.startsWith('.') || n.startsWith('_')) continue;
        mods.add(_scanModEntry(entity, bundledWithModId: modId));
      }
    }
  }

  mods.sort((a, b) => (a['name'] as String).compareTo(b['name'] as String));

  final directOverrides = <String>[];
  if (originalDir.existsSync()) {
    for (final entity in originalDir.listSync()) {
      if (entity is! File) continue;
      if (!entity.path.toLowerCase().endsWith('.usm')) continue;
      final info = _quickParseUsm(entity);
      if (info == null) continue;
      if (info.codec != 1) {
        directOverrides.add(path.basename(entity.path));
        continue;
      }
      final isVanillaShape =
          (info.width == 1600 && info.height == 904) ||
          (info.width == 1920 && info.height == 1080);
      if (!isVanillaShape) {
        directOverrides.add(path.basename(entity.path));
      }
    }
  }

  return {
    'mods': mods,
    'directOverrides': directOverrides,
    'originalFiles': originals.toList()..sort(),
  };
}

class ProgressInstallParams {
  final String sourcePath;
  final String movieDir;
  final String targetDir;
  final SendPort sendPort;
  const ProgressInstallParams({
    required this.sourcePath,
    required this.movieDir,
    required this.targetDir,
    required this.sendPort,
  });
}

void installWithProgress(ProgressInstallParams params) {
  final targetMovie = Directory(path.join(params.targetDir, 'movie'));
  targetMovie.createSync(recursive: true);

  final usmFiles = Directory(params.movieDir)
      .listSync(recursive: true)
      .whereType<File>()
      .where((f) => f.path.toLowerCase().endsWith('.usm'))
      .toList();

  if (usmFiles.isEmpty) {
    params.sendPort.send({'type': 'done', 'count': 0});
    return;
  }

  int totalBytes = 0;
  for (final f in usmFiles) {
    totalBytes += f.lengthSync();
  }

  int copiedBytes = 0;
  int count = 0;

  for (final file in usmFiles) {
    final rel = path.relative(file.path, from: params.movieDir);
    final fileName = path.basename(file.path);
    final target = File(path.join(targetMovie.path, rel));
    target.parent.createSync(recursive: true);

    params.sendPort.send({
      'type': 'progress',
      'current': count + 1,
      'total': usmFiles.length,
      'name': fileName,
      'mbDone': (copiedBytes / (1024 * 1024)).toStringAsFixed(0),
      'mbTotal': (totalBytes / (1024 * 1024)).toStringAsFixed(0),
      'percent': totalBytes > 0 ? copiedBytes / totalBytes : 0.0,
    });

    final input = file.openSync();
    final output = target.openSync(mode: FileMode.write);
    const chunkSize = 1024 * 1024;
    int lastReport = 0;

    while (true) {
      final chunk = input.readSync(chunkSize);
      if (chunk.isEmpty) break;
      output.writeFromSync(chunk);
      copiedBytes += chunk.length;

      if (copiedBytes - lastReport > 5 * 1024 * 1024) {
        lastReport = copiedBytes;
        params.sendPort.send({
          'type': 'progress',
          'current': count + 1,
          'total': usmFiles.length,
          'name': fileName,
          'mbDone': (copiedBytes / (1024 * 1024)).toStringAsFixed(0),
          'mbTotal': (totalBytes / (1024 * 1024)).toStringAsFixed(0),
          'percent': totalBytes > 0 ? copiedBytes / totalBytes : 0.0,
        });
      }
    }

    input.closeSync();
    output.closeSync();
    count++;
  }

  params.sendPort.send({'type': 'done', 'count': count});
}

Future<String?> findMovieDir(String root) =>
    IsolateService.run(_findMovieDirSync, root);

String? _findMovieDirSync(String root) {
  final rootDir = Directory(root);
  if (rootDir.existsSync()) {
    final rootHasUsm = rootDir.listSync().any(
      (e) => e is File && e.path.toLowerCase().endsWith('.usm'),
    );
    if (rootHasUsm) return root;
  }

  final direct = Directory(path.join(root, 'movie'));
  if (direct.existsSync()) {
    final hasUsm = direct.listSync().any(
      (e) => e is File && e.path.toLowerCase().endsWith('.usm'),
    );
    if (hasUsm) return direct.path;
  }

  for (final entity in Directory(root).listSync(recursive: true)) {
    if (entity is Directory &&
        path.basename(entity.path).toLowerCase() == 'movie') {
      final hasUsm = entity.listSync().any(
        (e) => e is File && e.path.toLowerCase().endsWith('.usm'),
      );
      if (hasUsm) return entity.path;
    }
  }
  return null;
}

bool deleteCutsceneModIsolate(String modPath) {
  final dir = Directory(modPath);
  if (dir.existsSync()) {
    dir.deleteSync(recursive: true);
    return true;
  }
  return false;
}

class ScanResult {
  final List<CutsceneMod> mods;
  final List<String> directOverrides;
  const ScanResult({required this.mods, required this.directOverrides});
}

class _ScanProgressParams {
  final String cutscenesDir;
  final String originalMovieDir;
  final String modsDir;
  final SendPort sendPort;
  const _ScanProgressParams({
    required this.cutscenesDir,
    required this.originalMovieDir,
    required this.modsDir,
    required this.sendPort,
  });
}

void _scanWithProgressEntry(_ScanProgressParams params) {
  final scanParams = ScanParams(
    cutscenesDir: params.cutscenesDir,
    originalMovieDir: params.originalMovieDir,
    modsDir: params.modsDir,
  );

  final allUsm = <File>[];

  final cutscenesDir = Directory(params.cutscenesDir);
  if (cutscenesDir.existsSync()) {
    for (final entity in cutscenesDir.listSync()) {
      if (entity is! Directory) continue;
      final n = path.basename(entity.path);
      if (n.startsWith('.') || n.startsWith('_')) continue;
      final movieDir = Directory(path.join(entity.path, 'movie'));
      if (!movieDir.existsSync()) continue;
      for (final f in movieDir.listSync()) {
        if (f is File && f.path.toLowerCase().endsWith('.usm')) {
          allUsm.add(f);
        }
      }
    }
  }

  final modsDir = Directory(params.modsDir);
  if (modsDir.existsSync()) {
    for (final modEntry in modsDir.listSync()) {
      if (modEntry is! Directory) continue;
      final modId = path.basename(modEntry.path);
      if (modId.startsWith('.') || modId.startsWith('_')) continue;
      final bundledCutscenes = Directory(path.join(modEntry.path, 'cutscenes'));
      if (!bundledCutscenes.existsSync()) continue;
      for (final entity in bundledCutscenes.listSync()) {
        if (entity is! Directory) continue;
        final n = path.basename(entity.path);
        if (n.startsWith('.') || n.startsWith('_')) continue;
        final movieDir = Directory(path.join(entity.path, 'movie'));
        if (!movieDir.existsSync()) continue;
        for (final f in movieDir.listSync()) {
          if (f is File && f.path.toLowerCase().endsWith('.usm')) {
            allUsm.add(f);
          }
        }
      }
    }
  }

  final total = allUsm.length;
  var scanned = 0;
  params.sendPort.send({'type': 'progress', 'scanned': 0, 'total': total});

  for (final file in allUsm) {
    _quickParseUsm(file);
    scanned++;
    if (scanned % 5 == 0 || scanned == total) {
      params.sendPort.send({'type': 'progress', 'scanned': scanned, 'total': total});
    }
  }

  final result = scanCutscenesIsolate(scanParams);
  params.sendPort.send({'type': 'done', 'result': result});
}

Future<ScanResult> scanCutscenes(
  String cutscenesDir,
  String originalMovieDir,
  String modsDir, {
  void Function(int scanned, int total)? onProgress,
}) async {
  if (onProgress == null) {
    final result = await IsolateService.run(
      scanCutscenesIsolate,
      ScanParams(
        cutscenesDir: cutscenesDir,
        originalMovieDir: originalMovieDir,
        modsDir: modsDir,
      ),
    );
    return _parseScanResult(result);
  }

  final receivePort = ReceivePort();
  final isolate = await Isolate.spawn(
    _scanWithProgressEntry,
    _ScanProgressParams(
      cutscenesDir: cutscenesDir,
      originalMovieDir: originalMovieDir,
      modsDir: modsDir,
      sendPort: receivePort.sendPort,
    ),
  );

  Map<String, dynamic>? rawResult;
  await for (final message in receivePort) {
    if (message is Map<String, dynamic>) {
      if (message['type'] == 'progress') {
        onProgress(message['scanned'] as int, message['total'] as int);
      } else if (message['type'] == 'done') {
        rawResult = message['result'] as Map<String, dynamic>;
        break;
      }
    }
  }

  isolate.kill(priority: Isolate.immediate);
  receivePort.close();

  return _parseScanResult(rawResult!);
}

ScanResult _parseScanResult(Map<String, dynamic> result) {

  final rawMods = result['mods'] as List<Map<String, dynamic>>;
  final mods = rawMods
      .map(
        (m) => CutsceneMod(
          name: m['name'] as String,
          fullPath: m['fullPath'] as String,
          usmCount: m['usmCount'] as int,
          usmFiles: List<String>.from(m['usmFiles'] as List),
          matchingOriginals: List<String>.from(m['matching'] as List),
          missingOriginals: List<String>.from(m['missing'] as List),
          hasH264: m['hasH264'] as bool? ?? false,
          maxWidth: m['maxWidth'] as int? ?? 0,
          maxHeight: m['maxHeight'] as int? ?? 0,
          bundledWithModId: m['bundledWithModId'] as String?,
        ),
      )
      .toList();

  return ScanResult(
    mods: mods,
    directOverrides: List<String>.from(result['directOverrides'] as List),
  );
}

class _UsmQuickInfo {
  final int width;
  final int height;
  final int codec;
  _UsmQuickInfo({required this.width, required this.height, required this.codec});
}

_UsmQuickInfo? _quickParseUsm(File file) {
  RandomAccessFile? raf;
  try {
    raf = file.openSync(mode: FileMode.read);
    final size = raf.lengthSync();
    var foundCrid = false;

    while (raf.positionSync() < size) {
      final pos = raf.positionSync();
      final stmIdBytes = raf.readSync(4);
      if (stmIdBytes.length < 4) break;
      final stmId = (stmIdBytes[0] << 24) | (stmIdBytes[1] << 16) | (stmIdBytes[2] << 8) | stmIdBytes[3];
      final blockSizeBytes = raf.readSync(4);
      if (blockSizeBytes.length < 4) break;
      final blockSize = (blockSizeBytes[0] << 24) | (blockSizeBytes[1] << 16) | (blockSizeBytes[2] << 8) | blockSizeBytes[3];

      final hdr = raf.readSync(0x18);
      if (hdr.length < 0x18) break;
      final blockType = hdr[7];
      final end = pos + 8 + blockSize;

      if (stmId == 0x43524944) {
        foundCrid = true;
      } else if (!foundCrid) {
        return null;
      } else if (stmId == 0x40534656 && blockType == 1) {
        final base = raf.positionSync();
        final magic = raf.readSync(4);
        if (magic.length < 4) return null;
        final m = (magic[0] << 24) | (magic[1] << 16) | (magic[2] << 8) | magic[3];
        if (m != 0x40555446) return null;
        final tsBuf = raf.readSync(4);
        if (tsBuf.length < 4) return null;
        // ignore: unused_local_variable
        final tableSize = (tsBuf[0] << 24) | (tsBuf[1] << 16) | (tsBuf[2] << 8) | tsBuf[3];

        raf.readSync(2);
        final roBytes = raf.readSync(2);
        if (roBytes.length < 2) return null;
        final rowsOff = (roBytes[0] << 8) | roBytes[1];
        final soBytes = raf.readSync(4);
        if (soBytes.length < 4) return null;
        final stringsOff = (soBytes[0] << 24) | (soBytes[1] << 16) | (soBytes[2] << 8) | soBytes[3];
        final doBytes = raf.readSync(4);
        if (doBytes.length < 4) return null;
        final dataOff = (doBytes[0] << 24) | (doBytes[1] << 16) | (doBytes[2] << 8) | doBytes[3];
        raf.readSync(4);
        final ncBytes = raf.readSync(2);
        if (ncBytes.length < 2) return null;
        final numCols = (ncBytes[0] << 8) | ncBytes[1];
        raf.readSync(2);
        final nrBytes = raf.readSync(4);
        if (nrBytes.length < 4) return null;
        final numRows = (nrBytes[0] << 24) | (nrBytes[1] << 16) | (nrBytes[2] << 8) | nrBytes[3];

        if (numRows == 0) return null;

        final descs = <(int, int, int)>[];
        for (var i = 0; i < numCols; i++) {
          final tb = raf.readSync(1);
          if (tb.isEmpty) return null;
          final no = raf.readSync(4);
          if (no.length < 4) return null;
          final nameOff = (no[0] << 24) | (no[1] << 16) | (no[2] << 8) | no[3];
          var cv = 0;
          if ((tb[0] & 0x20) != 0) {
            final ft = tb[0] & 0x0F;
            if (ft <= 1) { raf.readSync(1); }
            else if (ft <= 3) { raf.readSync(2); }
            else if (ft == 4 || ft == 5 || ft == 8 || ft == 0xa) { final b = raf.readSync(4); if (b.length >= 4) cv = (b[0] << 24) | (b[1] << 16) | (b[2] << 8) | b[3]; }
            else if (ft <= 9) { raf.readSync(8); }
          }
          descs.add((tb[0], nameOff, cv));
        }

        final strSize = dataOff - stringsOff;
        raf.setPositionSync(base + 8 + stringsOff);
        final strTab = Uint8List.fromList(raf.readSync(strSize));

        String nameFromTable(int offset) {
          if (offset >= strTab.length) return '';
          final e = strTab.indexOf(0, offset);
          final end2 = e == -1 ? strTab.length : e;
          return String.fromCharCodes(strTab.sublist(offset, end2));
        }

        final names = descs.map((d) => nameFromTable(d.$2)).toList();

        raf.setPositionSync(base + 8 + rowsOff);
        final fields = <String, int>{};
        for (var i = 0; i < descs.length; i++) {
          final d = descs[i];
          int val;
          if ((d.$1 & 0x20) != 0) {
            val = d.$3;
          } else {
            final ft = d.$1 & 0x0F;
            if (ft <= 1) { final b = raf.readSync(1); val = b.isNotEmpty ? b[0] : 0; }
            else if (ft <= 3) { final b = raf.readSync(2); val = b.length >= 2 ? (b[0] << 8) | b[1] : 0; }
            else if (ft == 4 || ft == 5 || ft == 8 || ft == 0xa) { final b = raf.readSync(4); val = b.length >= 4 ? (b[0] << 24) | (b[1] << 16) | (b[2] << 8) | b[3] : 0; }
            else if (ft <= 9) { raf.readSync(8); val = 0; }
            else if (ft == 0xb) { raf.readSync(8); val = 0; }
            else { val = 0; }
          }
          fields[names[i]] = val;
        }

        final w = [fields['width'] ?? 0, fields['disp_width'] ?? 0, fields['mat_width'] ?? 0].reduce((a, b) => a > b ? a : b);
        final h = [fields['height'] ?? 0, fields['disp_height'] ?? 0, fields['mat_height'] ?? 0].reduce((a, b) => a > b ? a : b);
        return _UsmQuickInfo(width: w, height: h, codec: fields['mpeg_codec'] ?? 0);
      }

      raf.setPositionSync(end);
    }
  } catch (_) {
  } finally {
    raf?.closeSync();
  }
  return null;
}
