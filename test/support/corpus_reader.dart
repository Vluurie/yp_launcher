import 'dart:io';

import 'package:path/path.dart' as p;

class CorpusArchive {
  final String path;
  final String ext;
  final String modId;
  final String basename;

  const CorpusArchive({
    required this.path,
    required this.ext,
    required this.modId,
    required this.basename,
  });

  String get uniqueId => '${modId}_$basename';

  @override
  String toString() => '$modId/$basename.$ext';
}

const _archiveExts = {'zip', '7z', 'rar'};

String defaultCorpusRoot() =>
    Platform.environment['YP_CORPUS_DIR'] ??
    (Platform.isWindows
        ? r'E:\all-outfit-mods\downloads'
        : p.join(Platform.environment['HOME'] ?? '', 'yp-corpus'));

String repoRoot() => Directory.current.path;

String bundledSevenZipPath() =>
    Platform.environment['YP_7Z_PATH'] ??
    p.join(
      repoRoot(),
      'assets',
      'bins',
      Platform.isWindows ? '7z.exe' : '7zz',
    );

List<String> listArchiveEntries(String archivePath, String sevenZExe) {
  final result = Process.runSync(
    sevenZExe,
    ['l', '-slt', '-ba', archivePath],
  );
  if (result.exitCode != 0) return const [];
  final out = <String>[];
  for (final line in (result.stdout as String).split('\n')) {
    final t = line.trim();
    if (t.startsWith('Path = ')) out.add(t.substring(7));
  }
  return out;
}

bool archiveHasModContent(List<String> entries) {
  for (final raw in entries) {
    final e = raw.toLowerCase().replaceAll('\\', '/');
    final base = e.split('/').last;
    if (base == 'mod.toml') return true;
    if (e.contains('/entities/') || e.startsWith('entities/')) return true;
    if (e.contains('/wax/') || e.startsWith('wax/')) return true;
    if (e.contains('/data/') || e.startsWith('data/')) return true;
    if (e.endsWith('.dds')) return true;
    if (e.endsWith('.cpk')) return true;
    if (e.endsWith('.dat') || e.endsWith('.dtt')) {
      if (RegExp(r'^(pl|wp|em|bg|ba|bh|et|it|um|wd)').hasMatch(base)) {
        return true;
      }
    }
  }
  return false;
}

List<CorpusArchive> discoverCorpus(String root) {
  final dir = Directory(root);
  if (!dir.existsSync()) return const [];
  final out = <CorpusArchive>[];
  for (final entity in dir.listSync()) {
    if (entity is! Directory) continue;
    final modId = p.basename(entity.path);
    for (final file in entity.listSync()) {
      if (file is! File) continue;
      final ext = p.extension(file.path).replaceFirst('.', '').toLowerCase();
      if (!_archiveExts.contains(ext)) continue;
      out.add(CorpusArchive(
        path: file.path,
        ext: ext,
        modId: modId,
        basename: p.basenameWithoutExtension(file.path),
      ));
    }
  }
  out.sort((a, b) => a.path.toLowerCase().compareTo(b.path.toLowerCase()));
  return out;
}
