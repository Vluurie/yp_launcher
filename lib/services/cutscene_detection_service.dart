import 'dart:io';
import 'dart:typed_data';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/isolate_service.dart';

class CutsceneDetection {
  final bool hasHdCutscenes;
  final bool needsH264;
  final int filesScanned;
  final int largestWidth;
  final int largestHeight;

  const CutsceneDetection({
    this.hasHdCutscenes = false,
    this.needsH264 = false,
    this.filesScanned = 0,
    this.largestWidth = 0,
    this.largestHeight = 0,
  });
}

const _magicCrid = 0x43524944;
const _magicSfv = 0x40534656;

bool _isVanillaUsm(_UsmInfo info) {
  if (info.codec != 1) return false;
  if (info.width == 1600 && info.height == 904) return true;
  if (info.width == 1920 && info.height == 1080) return true;
  return false;
}

CutsceneDetection _scanSync(String gameDir) {
  var largestPixels = 0;
  var largestW = 0;
  var largestH = 0;
  var needsH264 = false;
  var customCount = 0;

  void scanDir(Directory dir) {
    if (!dir.existsSync()) return;
    for (final entity in dir.listSync()) {
      if (entity is! File) continue;
      if (!entity.path.toLowerCase().endsWith('.usm')) continue;
      final result = _parseUsmSync(entity);
      if (result == null) continue;
      // Files under nams/cutscenes/ or nams/mods/<id>/cutscenes/ are mods
      // by definition. No vanilla-shape check here — that filter is only
      // for data/movie/, which is the vanilla payload.
      customCount++;
      if (result.codec == 5) needsH264 = true;
      final px = result.width * result.height;
      if (px > largestPixels) {
        largestPixels = px;
        largestW = result.width;
        largestH = result.height;
      }
    }
  }

  // Only scan user-installed cutscene mods. The vanilla data/movie/ folder
  // is not a mod and must not be reported. A separate helper detects users
  // who installed cutscene packs into data/movie/ and warns them.
  final cutscenesDir = Directory(path.join(gameDir, 'nams', 'cutscenes'));
  if (cutscenesDir.existsSync()) {
    for (final modDir in cutscenesDir.listSync()) {
      if (modDir is! Directory) continue;
      final name = path.basename(modDir.path);
      if (name.startsWith('.') || name.startsWith('_')) continue;
      scanDir(Directory(path.join(modDir.path, 'movie')));
    }
  }

  // Also scan cutscenes bundled inside mod folders:
  //   nams/mods/<mod_id>/cutscenes/<name>/movie/*.usm
  final modsDir = Directory(path.join(gameDir, 'nams', 'mods'));
  if (modsDir.existsSync()) {
    for (final modEntry in modsDir.listSync()) {
      if (modEntry is! Directory) continue;
      final modId = path.basename(modEntry.path);
      if (modId.startsWith('.') || modId.startsWith('_')) continue;
      final bundledCutscenes =
          Directory(path.join(modEntry.path, 'cutscenes'));
      if (!bundledCutscenes.existsSync()) continue;
      for (final csEntry in bundledCutscenes.listSync()) {
        if (csEntry is! Directory) continue;
        final csName = path.basename(csEntry.path);
        if (csName.startsWith('.') || csName.startsWith('_')) continue;
        scanDir(Directory(path.join(csEntry.path, 'movie')));
      }
    }
  }

  return CutsceneDetection(
    hasHdCutscenes: customCount > 0,
    needsH264: needsH264,
    filesScanned: customCount,
    largestWidth: largestW,
    largestHeight: largestH,
  );
}

bool _hasCustomInDataMovieSync(String gameDir) {
  final dir = Directory(path.join(gameDir, 'data', 'movie'));
  if (!dir.existsSync()) return false;
  for (final entity in dir.listSync()) {
    if (entity is! File) continue;
    if (!entity.path.toLowerCase().endsWith('.usm')) continue;
    final result = _parseUsmSync(entity);
    if (result == null) continue;
    if (!_isVanillaUsm(result)) return true;
  }
  return false;
}

_UsmInfo? _parseUsmSync(File file) {
  RandomAccessFile? raf;
  try {
    raf = file.openSync(mode: FileMode.read);
    final size = raf.lengthSync();

    var foundCrid = false;
    while (raf.positionSync() < size) {
      final pos = raf.positionSync();
      final stmId = _readU32Sync(raf);
      if (stmId == null) break;
      final blockSize = _readU32Sync(raf);
      if (blockSize == null) break;

      final hdr = _readBytesSync(raf, 0x18);
      if (hdr == null) break;
      final blockType = hdr[7];
      final end = pos + 8 + blockSize;

      if (stmId == _magicCrid) {
        foundCrid = true;
      } else if (!foundCrid) {
        return null;
      } else if (stmId == _magicSfv && blockType == 1) {
        return _parseSfvSync(raf, end);
      }

      raf.setPositionSync(end);
    }
  } catch (_) {
  } finally {
    raf?.closeSync();
  }
  return null;
}

_UsmInfo? _parseSfvSync(RandomAccessFile raf, int blockEnd) {
  final base = raf.positionSync();
  final magic = _readU32Sync(raf);
  if (magic != 0x40555446) return null;
  final tableSize = _readU32Sync(raf);
  if (tableSize == null) return null;
  final end = base + 8 + tableSize;

  _readU8Sync(raf);
  _readU8Sync(raf);
  final rowsOff = _readU16Sync(raf);
  final stringsOff = _readU32Sync(raf);
  final dataOff = _readU32Sync(raf);
  _readU32Sync(raf);
  final numCols = _readU16Sync(raf);
  _readU16Sync(raf);
  final numRows = _readU32Sync(raf);

  if (rowsOff == null ||
      stringsOff == null ||
      dataOff == null ||
      numCols == null ||
      numRows == null)
    return null;
  final limit = end < blockEnd ? end : blockEnd;
  if (base + 8 + stringsOff >= limit) return null;

  final descs = <_ColDesc>[];
  for (var i = 0; i < numCols; i++) {
    if (raf.positionSync() >= limit) return null;
    final tb = _readU8Sync(raf);
    if (tb == null) return null;
    final nameOff = _readU32Sync(raf);
    if (nameOff == null) return null;
    var constVal = 0;
    if ((tb & 0x20) != 0) {
      constVal = _readConstantSync(raf, tb & 0x0F) ?? 0;
    }
    descs.add(_ColDesc(tb: tb, nameOff: nameOff, constVal: constVal));
  }

  final strSize = dataOff - stringsOff;
  raf.setPositionSync(base + 8 + stringsOff);
  final strTab = _readBytesSync(raf, strSize);
  if (strTab == null) return null;

  final names = descs.map((d) => _strFromTable(strTab, d.nameOff)).toList();

  if (numRows == 0) return null;
  raf.setPositionSync(base + 8 + rowsOff);

  final fields = <String, int>{};
  for (var i = 0; i < descs.length; i++) {
    final d = descs[i];
    int val;
    if ((d.tb & 0x20) != 0) {
      val = d.constVal;
    } else {
      val = _readFieldValueSync(raf, d.tb & 0x0F) ?? 0;
    }
    fields[names[i]] = val;
  }

  final w = [
    fields['width'] ?? 0,
    fields['disp_width'] ?? 0,
    fields['mat_width'] ?? 0,
  ].reduce((a, b) => a > b ? a : b);
  final h = [
    fields['height'] ?? 0,
    fields['disp_height'] ?? 0,
    fields['mat_height'] ?? 0,
  ].reduce((a, b) => a > b ? a : b);
  final codec = fields['mpeg_codec'] ?? 0;

  return _UsmInfo(width: w, height: h, codec: codec);
}

String _strFromTable(Uint8List table, int offset) {
  if (offset >= table.length) return '';
  final end = table.indexOf(0, offset);
  final e = end == -1 ? table.length : end;
  return String.fromCharCodes(table.sublist(offset, e));
}

int? _readU8Sync(RandomAccessFile f) {
  final b = _readBytesSync(f, 1);
  return b?[0];
}

int? _readU16Sync(RandomAccessFile f) {
  final b = _readBytesSync(f, 2);
  if (b == null) return null;
  return (b[0] << 8) | b[1];
}

int? _readU32Sync(RandomAccessFile f) {
  final b = _readBytesSync(f, 4);
  if (b == null) return null;
  return (b[0] << 24) | (b[1] << 16) | (b[2] << 8) | b[3];
}

int? _readU64Sync(RandomAccessFile f) {
  final b = _readBytesSync(f, 8);
  if (b == null) return null;
  return (b[0] << 56) |
      (b[1] << 48) |
      (b[2] << 40) |
      (b[3] << 32) |
      (b[4] << 24) |
      (b[5] << 16) |
      (b[6] << 8) |
      b[7];
}

Uint8List? _readBytesSync(RandomAccessFile f, int n) {
  try {
    final b = f.readSync(n);
    if (b.length < n) return null;
    return b;
  } catch (_) {
    return null;
  }
}

int? _readConstantSync(RandomAccessFile f, int ft) {
  switch (ft) {
    case 0:
    case 1:
      _readU8Sync(f);
      return 0;
    case 2:
    case 3:
      _readU16Sync(f);
      return 0;
    case 4:
    case 5:
    case 8:
    case 0xa:
      return _readU32Sync(f);
    case 6:
    case 7:
    case 9:
      _readU64Sync(f);
      return 0;
    default:
      return 0;
  }
}

int? _readFieldValueSync(RandomAccessFile f, int ft) {
  switch (ft) {
    case 0:
    case 1:
      return _readU8Sync(f);
    case 2:
    case 3:
      return _readU16Sync(f);
    case 4:
    case 5:
    case 8:
    case 0xa:
      return _readU32Sync(f);
    case 6:
    case 7:
    case 9:
      _readU64Sync(f);
      return 0;
    case 0xb:
      _readU32Sync(f);
      _readU32Sync(f);
      return 0;
    default:
      return 0;
  }
}

class CutsceneDetectionService {
  static Future<CutsceneDetection> scan(String gameDir) {
    return IsolateService.run(_scanSync, gameDir);
  }

  static Future<bool> hasCustomInDataMovie(String gameDir) {
    return IsolateService.run(_hasCustomInDataMovieSync, gameDir);
  }
}

class _UsmInfo {
  final int width;
  final int height;
  final int codec;
  _UsmInfo({required this.width, required this.height, required this.codec});
}

class _ColDesc {
  final int tb;
  final int nameOff;
  final int constVal;
  _ColDesc({required this.tb, required this.nameOff, required this.constVal});
}
