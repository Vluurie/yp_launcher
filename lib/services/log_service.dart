import 'dart:async';
import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:yp_launcher/constants/app_strings.dart';
import 'package:yp_launcher/services/launcher_setup_service.dart';

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
    return path.join(
      LauncherSetupService.launcherDirectory,
      AppStrings.logsDirName,
    );
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

  /// Watch a log file for new entries. Emits a list of newly-parsed
  /// entries each tick. Uses size-based tailing (reliable across platforms,
  /// where Directory.watch on actively-written files is flaky on Windows).
  ///
  /// The first emission contains the full current file content. Subsequent
  /// emissions contain only entries appended since the previous read.
  /// When the file is truncated or replaced, we restart from the beginning.
  static Stream<List<LogEntry>> watchLog(
    String filename, {
    Duration interval = const Duration(milliseconds: 750),
  }) {
    final filePath = path.join(logsDirectory, filename);
    final controller = StreamController<List<LogEntry>>();
    Timer? timer;
    var lastSize = 0;
    var leftover = '';

    Future<void> tick() async {
      try {
        final file = File(filePath);
        if (!await file.exists()) {
          if (lastSize != 0 || leftover.isNotEmpty) {
            lastSize = 0;
            leftover = '';
            controller.add(const []);
          }
          return;
        }
        final size = await file.length();
        if (size < lastSize) {
          lastSize = 0;
          leftover = '';
        }
        if (size == lastSize) return;

        final raf = await file.open();
        try {
          await raf.setPosition(lastSize);
          final bytes = await raf.read(size - lastSize);
          lastSize = size;
          final text = leftover + String.fromCharCodes(bytes);
          final newlineIdx = text.lastIndexOf('\n');
          final consumable = newlineIdx == -1 ? '' : text.substring(0, newlineIdx);
          leftover = newlineIdx == -1 ? text : text.substring(newlineIdx + 1);
          if (consumable.isEmpty) return;
          final entries = consumable
              .split('\n')
              .where((l) => l.trim().isNotEmpty)
              .map(_parseLine)
              .whereType<LogEntry>()
              .toList();
          if (entries.isNotEmpty && !controller.isClosed) {
            controller.add(entries);
          }
        } finally {
          await raf.close();
        }
      } catch (_) {}
    }

    controller.onListen = () {
      tick();
      timer = Timer.periodic(interval, (_) => tick());
    };
    controller.onCancel = () async {
      timer?.cancel();
      timer = null;
      await controller.close();
    };
    return controller.stream;
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
