import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';

class LogEntry {
  final String timestamp;
  final String level;
  final String module;
  final String message;

  const LogEntry({
    required this.timestamp,
    required this.level,
    required this.module,
    required this.message,
  });
}

class LogService {
  LogService._();

  static final _ansiRegex = RegExp(r'\x1B\[[0-9;]*m');

  static String get logsDirectory {
    final appData = Platform.environment['APPDATA'] ?? '';
    return path.join(appData, AppStrings.launcherDirName, 'nierdecrypt', 'logs');
  }

  static Future<List<LogEntry>> readLog(String filename) async {
    final file = File(path.join(logsDirectory, filename));
    if (!await file.exists()) return [];

    final content = await file.readAsString();
    final lines = content.split('\n').where((l) => l.trim().isNotEmpty);

    return lines.map(_parseLine).whereType<LogEntry>().toList();
  }

  static LogEntry? _parseLine(String raw) {
    final clean = raw.replaceAll(_ansiRegex, '');

    // Format: "2026-02-14T12:46:29.398602Z  INFO modloader: Startup"
    final match = RegExp(
      r'^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+(INFO|WARN|DEBUG|ERROR|TRACE)\s+(\S+?):\s*(.*)',
    ).firstMatch(clean);

    if (match == null) return null;

    return LogEntry(
      timestamp: _formatTimestamp(match.group(1)!),
      level: match.group(2)!,
      module: match.group(3)!,
      message: match.group(4)!,
    );
  }

  static String _formatTimestamp(String iso) {
    // "2026-02-14T12:46:29.398602Z" -> "12:46:29.398"
    final tIndex = iso.indexOf('T');
    if (tIndex < 0) return iso;
    final timePart = iso.substring(tIndex + 1).replaceAll('Z', '');
    // Trim microseconds to milliseconds
    final dotIndex = timePart.indexOf('.');
    if (dotIndex >= 0 && timePart.length > dotIndex + 4) {
      return timePart.substring(0, dotIndex + 4);
    }
    return timePart;
  }
}
