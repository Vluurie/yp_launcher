import 'dart:io';
import 'package:archive/archive.dart';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/services/isolate_service.dart';

class ArchiveService {
  static String? _cached7zPath;

  static String? overrideSevenZipPath;

  static String _get7zPath() {
    final override = overrideSevenZipPath;
    if (override != null && File(override).existsSync()) return override;
    if (_cached7zPath != null) return _cached7zPath!;
    final appDir = File(Platform.resolvedExecutable).parent.path;
    final candidates = [
      path.join(appDir, 'data', 'flutter_assets', 'assets', 'bins', '7z.exe'),
      path.join(appDir, 'data', 'assets', 'bins', '7z.exe'),
      path.join(appDir, 'assets', 'bins', '7z.exe'),
    ];
    for (final p in candidates) {
      if (File(p).existsSync()) {
        _cached7zPath = p;
        return _cached7zPath!;
      }
    }
    _cached7zPath = candidates.first;
    return _cached7zPath!;
  }

  static bool isArchive(String filePath) {
    final lower = filePath.toLowerCase();
    return lower.endsWith('.zip') ||
        lower.endsWith('.7z') ||
        lower.endsWith('.rar') ||
        lower.endsWith('.tar') ||
        lower.endsWith('.gz') ||
        lower.endsWith('.bz2');
  }

  static bool isZipOnly(String filePath) {
    return filePath.toLowerCase().endsWith('.zip');
  }

  /// List the file paths inside an archive without extracting it.
  /// Returns null if 7z is unavailable or the listing fails.
  static Future<List<String>?> listEntries(String archivePath) async {
    try {
      final sevenZip = _get7zPath();
      if (!File(sevenZip).existsSync()) return null;
      final result = await Process.run(
        sevenZip,
        ['l', '-slt', '-ba', archivePath],
      );
      if (result.exitCode != 0) return null;
      final entries = <String>[];
      for (final line in (result.stdout as String).split('\n')) {
        final trimmed = line.trimRight();
        if (trimmed.startsWith('Path = ')) {
          entries.add(trimmed.substring('Path = '.length));
        }
      }
      return entries;
    } catch (_) {
      return null;
    }
  }

  /// Returns true if any entry in the archive matches the given file
  /// extensions (case-insensitive). Returns null if listing failed.
  static Future<bool?> archiveContainsExtension(
    String archivePath,
    Iterable<String> extensions,
  ) async {
    final entries = await listEntries(archivePath);
    if (entries == null) return null;
    final exts = extensions.map((e) => e.toLowerCase()).toList();
    for (final entry in entries) {
      final lower = entry.toLowerCase();
      if (exts.any(lower.endsWith)) return true;
    }
    return false;
  }

  /// Returns true if the archive contains any entry for which [predicate]
  /// returns true. The predicate receives each entry path lower-cased and
  /// with backslashes normalized to forward slashes. Returns null if listing
  /// failed.
  static Future<bool?> archiveContainsAny(
    String archivePath,
    bool Function(String entryPath) predicate,
  ) async {
    final entries = await listEntries(archivePath);
    if (entries == null) return null;
    for (final entry in entries) {
      final normalized = entry.toLowerCase().replaceAll('\\', '/');
      if (predicate(normalized)) return true;
    }
    return false;
  }

  static Future<String?> extract(
    String archivePath, {
    void Function(double percent, String? currentFile)? onProgress,
  }) async {
    final lower = archivePath.toLowerCase();
    if (!isArchive(archivePath)) return null;

    final sevenZip = _get7zPath();
    if (File(sevenZip).existsSync()) {
      return _extract7z(archivePath, onProgress: onProgress);
    }

    if (lower.endsWith('.zip')) {
      return _extractZip(archivePath);
    }
    return null;
  }

  static Future<String?> _extractZip(String archivePath) {
    return IsolateService.run(_extractZipSync, archivePath);
  }

  static Future<String?> _extract7z(
    String archivePath, {
    void Function(double percent, String? currentFile)? onProgress,
  }) async {
    Directory? tempDir;
    try {
      final sevenZip = _get7zPath();
      if (!File(sevenZip).existsSync()) {
        _lastExtractError = '7z.exe not found at ${sevenZip}';
        return null;
      }

      tempDir = await Directory.systemTemp.createTemp('archive_');

      final process = await Process.start(
        sevenZip,
        ['x', archivePath, '-o${tempDir.path}', '-y', '-bsp1', '-bse1'],
      );
      final stderrBuffer = StringBuffer();
      final progressRegex = RegExp(r'(\d{1,3})%(?:\s+\d+)?(?:\s+[+\-=UT]\s+(.+))?');
      process.stdout.transform(const SystemEncoding().decoder).listen((chunk) {
        if (onProgress == null) return;
        for (final line in chunk.split(RegExp(r'[\r\n]+'))) {
          final m = progressRegex.firstMatch(line);
          if (m == null) continue;
          final pct = int.tryParse(m.group(1)!);
          if (pct == null) continue;
          onProgress(pct / 100.0, m.group(2)?.trim());
        }
      });
      process.stderr.transform(const SystemEncoding().decoder).listen(stderrBuffer.write);

      final exitCode = await process.exitCode;
      if (exitCode != 0) {
        _lastExtractError =
            '7z exit $exitCode: ${stderrBuffer.toString().trim()}';
        try { await tempDir.delete(recursive: true); } catch (_) {}
        return null;
      }

      final hasFiles = await tempDir.list().any((_) => true);
      if (!hasFiles) {
        _lastExtractError = '7z produced no files';
        try { await tempDir.delete(recursive: true); } catch (_) {}
        return null;
      }

      _lastExtractError = null;
      return tempDir.path;
    } catch (e) {
      _lastExtractError = '$e';
      try { await tempDir?.delete(recursive: true); } catch (_) {}
      return null;
    }
  }

  static String? _lastExtractError;
  static String? get lastExtractError => _lastExtractError;
}

String? _extractZipSync(String archivePath) {
  try {
    final bytes = File(archivePath).readAsBytesSync();
    final archive = ZipDecoder().decodeBytes(bytes);
    final tempDir = Directory.systemTemp.createTempSync('archive_');

    for (final file in archive) {
      final outPath = path.join(tempDir.path, file.name);
      if (file.isFile) {
        final outFile = File(outPath);
        outFile.parent.createSync(recursive: true);
        outFile.writeAsBytesSync(file.content as List<int>);
      } else {
        Directory(outPath).createSync(recursive: true);
      }
    }

    return tempDir.path;
  } catch (_) {
    return null;
  }
}
