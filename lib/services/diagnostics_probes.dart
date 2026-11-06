import 'dart:convert';
import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as p;
import 'package:yp_launcher/services/platform/win32_process.dart';

class FileFacts {
  final String name;
  final int size;
  final DateTime? modified;
  final String? shortSha1;

  const FileFacts({
    required this.name,
    required this.size,
    this.modified,
    this.shortSha1,
  });
}

class TextExcerpt {
  final String name;
  final int size;
  final DateTime? modified;
  final List<String> lines;

  const TextExcerpt({
    required this.name,
    required this.size,
    this.modified,
    this.lines = const [],
  });
}

class MountFacts {
  final String target;
  final String fsType;
  final String options;

  const MountFacts({
    required this.target,
    required this.fsType,
    this.options = '',
  });

  bool get risky => DiagnosticsProbes.riskyFilesystems.contains(fsType);
}

class DiagnosticsProbes {
  DiagnosticsProbes._();

  static const riskyFilesystems = <String>{
    'ntfs',
    'ntfs3',
    'fuseblk',
    'exfat',
    'vfat',
    'msdos',
  };

  static MountFacts? parseFindmnt(String json) {
    final decoded = jsonDecode(json);
    if (decoded is! Map) return null;
    final list = decoded['filesystems'];
    if (list is! List || list.isEmpty || list.first is! Map) return null;
    final fs = list.first as Map;
    final target = fs['target'];
    final type = fs['fstype'];
    if (target is! String || type is! String) return null;
    final options = fs['options'];
    return MountFacts(
      target: target,
      fsType: type.toLowerCase(),
      options: options is String ? options : '',
    );
  }

  static Future<MountFacts?> mountOf(String path) async {
    if (!Platform.isLinux || path.isEmpty) return null;
    try {
      final result = await Process.run('findmnt', [
        '-J',
        '-T',
        path,
        '-o',
        'TARGET,FSTYPE,OPTIONS',
      ]).timeout(const Duration(seconds: 5));
      if (result.exitCode != 0) return null;
      return parseFindmnt(result.stdout as String);
    } catch (_) {
      return null;
    }
  }

  static Map<String, String> parseKeyValueLines(String content) {
    final out = <String, String>{};
    for (final raw in const LineSplitter().convert(content)) {
      final line = raw.trim();
      if (line.isEmpty || line.startsWith('#')) continue;
      final eq = line.indexOf('=');
      if (eq <= 0) continue;
      var value = line.substring(eq + 1).trim();
      if (value.length >= 2 &&
          (value.startsWith('"') && value.endsWith('"') ||
              value.startsWith("'") && value.endsWith("'"))) {
        value = value.substring(1, value.length - 1);
      }
      out[line.substring(0, eq).trim()] = value;
    }
    return out;
  }

  static Map<String, String> linuxContext() {
    if (!Platform.isLinux) return const {};
    final env = Platform.environment;
    var distro = '?';
    try {
      final release = parseKeyValueLines(
        File('/etc/os-release').readAsStringSync(),
      );
      distro = release['PRETTY_NAME'] ?? release['NAME'] ?? '?';
    } catch (_) {}
    final flatpak =
        (env['FLATPAK_ID'] ?? '').isNotEmpty ||
        File('/.flatpak-info').existsSync();
    return {
      'Distro': distro,
      'Session': env['XDG_SESSION_TYPE'] ?? '?',
      'Desktop': env['XDG_CURRENT_DESKTOP'] ?? '?',
      'AppImage': (env['APPIMAGE'] ?? '').isNotEmpty ? 'yes' : 'no',
      'Flatpak': flatpak ? 'yes' : 'no',
      'Steam Deck': (env['SteamDeck'] ?? '') == '1' ? 'yes' : 'no',
    };
  }

  static Future<bool?> isSteamRunning() async {
    try {
      if (Platform.isWindows) return isWin32ProcessRunning('steam.exe');
      final args = Platform.isLinux
          ? const ['-x', 'steam']
          : const ['-f', 'steam.exe'];
      final result = await Process.run(
        'pgrep',
        args,
      ).timeout(const Duration(seconds: 5));
      return result.exitCode == 0;
    } catch (_) {
      return null;
    }
  }

  static String stripAnsi(String input) {
    if (!input.contains('\x1B')) return input;
    final out = StringBuffer();
    var i = 0;
    while (i < input.length) {
      final c = input.codeUnitAt(i);
      if (c == 0x1B && i + 1 < input.length && input[i + 1] == '[') {
        var j = i + 2;
        while (j < input.length) {
          final k = input.codeUnitAt(j);
          if (k >= 0x40 && k <= 0x7E) break;
          j++;
        }
        i = j + 1;
        continue;
      }
      out.writeCharCode(c);
      i++;
    }
    return out.toString();
  }

  static List<String> _decodeLines(List<int> bytes) => const LineSplitter()
      .convert(
        stripAnsi(const Utf8Decoder(allowMalformed: true).convert(bytes)),
      )
      .map((l) => l.trimRight())
      .toList();

  static TextExcerpt? tail(
    String path, {
    int maxLines = 40,
    int maxBytes = 64 * 1024,
  }) {
    final file = File(path);
    if (!file.existsSync()) return null;
    try {
      final size = file.lengthSync();
      final raf = file.openSync();
      List<int> bytes;
      try {
        final start = size > maxBytes ? size - maxBytes : 0;
        raf.setPositionSync(start);
        bytes = raf.readSync(size - start);
      } finally {
        raf.closeSync();
      }
      var lines = _decodeLines(bytes);
      if (size > maxBytes && lines.isNotEmpty) lines = lines.sublist(1);
      lines = lines.where((l) => l.isNotEmpty).toList();
      if (lines.length > maxLines) {
        lines = lines.sublist(lines.length - maxLines);
      }
      return TextExcerpt(
        name: p.basename(path),
        size: size,
        modified: _modified(file),
        lines: lines,
      );
    } catch (_) {
      return TextExcerpt(name: p.basename(path), size: 0);
    }
  }

  static TextExcerpt? head(String path, {int maxLines = 60}) {
    final file = File(path);
    if (!file.existsSync()) return null;
    try {
      final size = file.lengthSync();
      final raf = file.openSync();
      List<int> bytes;
      try {
        bytes = raf.readSync(size > 64 * 1024 ? 64 * 1024 : size);
      } finally {
        raf.closeSync();
      }
      final lines = _decodeLines(bytes);
      return TextExcerpt(
        name: p.basename(path),
        size: size,
        modified: _modified(file),
        lines: lines.length > maxLines ? lines.sublist(0, maxLines) : lines,
      );
    } catch (_) {
      return TextExcerpt(name: p.basename(path), size: 0);
    }
  }

  static FileFacts? facts(String path, {bool hash = false}) {
    final file = File(path);
    if (!file.existsSync()) return null;
    try {
      String? digest;
      if (hash) {
        digest = sha1
            .convert(file.readAsBytesSync())
            .toString()
            .substring(0, 12);
      }
      return FileFacts(
        name: p.basename(path),
        size: file.lengthSync(),
        modified: _modified(file),
        shortSha1: digest,
      );
    } catch (_) {
      return FileFacts(name: p.basename(path), size: 0);
    }
  }

  static List<FileFacts> filesIn(
    String dir, {
    bool Function(String lowerName)? where,
    bool hash = false,
  }) {
    final d = Directory(dir);
    if (!d.existsSync()) return const [];
    final out = <FileFacts>[];
    for (final f in d.listSync(followLinks: false).whereType<File>()) {
      final name = p.basename(f.path);
      if (where != null && !where(name.toLowerCase())) continue;
      final fact = facts(f.path, hash: hash);
      if (fact != null) out.add(fact);
    }
    out.sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
    return out;
  }

  static List<File> newestFirst(
    String dir,
    bool Function(String lowerName) where,
  ) {
    final d = Directory(dir);
    if (!d.existsSync()) return const [];
    final files = d
        .listSync(followLinks: false)
        .whereType<File>()
        .where((f) => where(p.basename(f.path).toLowerCase()))
        .toList();
    final stamps = {for (final f in files) f.path: _modified(f) ?? DateTime(0)};
    files.sort((a, b) => stamps[b.path]!.compareTo(stamps[a.path]!));
    return files;
  }

  static String? firstLine(String path) {
    try {
      final file = File(path);
      if (!file.existsSync()) return null;
      final lines = const LineSplitter().convert(file.readAsStringSync());
      for (final l in lines) {
        if (l.trim().isNotEmpty) return l.trim();
      }
    } catch (_) {}
    return null;
  }

  static DateTime? _modified(File f) {
    try {
      return f.statSync().modified;
    } catch (_) {
      return null;
    }
  }
}
